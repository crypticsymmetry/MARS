//! E27: knowledge-graph completion by analogy.
//!
//! Memory: one KG's entity cases from `tools/kg2mars.py` (condition C: the
//! entity's outgoing triples plus label attributes), e.g. the Wikidata side of
//! `data/kg-scientists`. Query: an entity with *all* its facts of one relation
//! removed (and the labels of objects that no longer occur). The task is to
//! predict the removed objects.
//!
//! Analogues come from the other entities, and each one proposes objects in two ways:
//! * **copy**: the analogue's own objects for the relation (nearest-neighbour
//!   voting, the collaborative-filtering reading of analogy);
//! * **analogy**: the candidate inferences of the structure mapping analogue →
//!   query. A projected object is either an existing query entity (a
//!   *substitution*: the analogue's object corresponds to a query entity through
//!   other relations, e.g. "works where they studied") or a skolem for the
//!   analogue's own object (a copy).
//!
//! Votes are summed over the top-m analogues, weighted by similarity.
//! Neighbours: MARS fused (fingerprint top-k re-ranked by ½FAC + ½FP), fingerprint
//! only, lexical TF-IDF over entity and predicate names, and random entities.
//! Baselines: global popularity of the relation's objects; length-1 rules
//! r'(x,y) ⇒ r(x,y) with leave-one-out confidence (the global counterpart of a
//! substitution), alone and added to lexical nearest-neighbour votes.
//! Metrics: Hits@1, Hits@10 and MRR (first correct object), per relation;
//! calibration of the top-1 by support (number of analogues voting for it).

use crate::metrics::mean;
use crate::Args;
use mars_encode::{cosine, lexical_tokens, FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, SparseVec, N_CHANNELS};
use mars_hv::Rng;
use mars_map::{MapConfig, Mapper, Proj};
use mars_rel::{CaseId, CaseKind, ExprId, Kb, PredKind, Sym, Term};
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

struct Query {
    case: CaseId,
    orig: usize,
    rel: usize,
    person: Sym,
    gold: FxHashSet<Sym>,
}

/// One proposed object: (object, weight, is_substitution).
type Vote = (Sym, f64, bool);

/// Ranked objects with their support (number of voting analogues) and whether the
/// top-1 came from a substitution.
struct Ranked {
    list: Vec<(Sym, f64, usize, bool)>,
}

fn rank(votes: &[Vote], pop: &FxHashMap<Sym, f64>) -> Ranked {
    let mut agg: FxHashMap<Sym, (f64, usize, bool)> = FxHashMap::default();
    for &(o, w, sub) in votes {
        let e = agg.entry(o).or_insert((0.0, 0, false));
        e.0 += w;
        e.1 += 1;
        e.2 |= sub;
    }
    let mut list: Vec<(Sym, f64, usize, bool)> = agg.into_iter().map(|(o, (w, n, s))| (o, w + 1e-6 * pop.get(&o).copied().unwrap_or(0.0), n, s)).collect();
    list.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    Ranked { list }
}

/// (hit@1, hit@10, reciprocal rank, covered) of a ranking against the gold objects.
fn score(r: &Ranked, gold: &FxHashSet<Sym>) -> [f64; 4] {
    match r.list.iter().position(|x| gold.contains(&x.0)) {
        Some(p) => [(p < 1) as u8 as f64, (p < 10) as u8 as f64, 1.0 / (p + 1) as f64, 1.0],
        None => [0.0; 4],
    }
}

pub fn run(args: &Args) -> Result<(), String> {
    let dir = args.str("data", "data/kg-scientists/C");
    let kg = args.str("kg", "wikidata");
    let rels_arg = args.str("relations", "wdt:p69,wdt:p108,wdt:p101,wdt:p27,wdt:p106,wdt:p166,wdt:p463,wdt:p1412");
    let per_rel = args.usize("per-rel", 300);
    let k = args.usize("k", 50);
    let m_max = args.usize("m", 10);
    let seed = args.u64("seed", 1);
    let out_dir = args.str("out", "results/E27");
    let tag = args.str("tag", "wikidata-scientists");
    // Fused = fac_w·FAC + (1 − fac_w)·FP over the fingerprint top-k.
    let fac_w = args.f64("fac-weight", 0.5);
    let t0 = Instant::now();

    let mut kb = Kb::new();
    kb.load_str(&std::fs::read_to_string(format!("{dir}/cases.mars")).map_err(|e| format!("{dir}/cases.mars: {e}"))?).map_err(|e| e.to_string())?;
    let manifest: Vec<serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(format!("{dir}/manifest.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let mem: Vec<CaseId> = manifest.iter().filter(|m| m["kg"].as_str() == Some(kg.as_str())).filter_map(|m| kb.case_by_name(m["case"].as_str()?)).collect();
    let n = mem.len();
    let rels: Vec<Sym> = rels_arg.split(',').map(|r| kb.sym(r)).collect();
    let is_attr = |kb: &Kb, f: ExprId| kb.vocab.kind(kb.expr(f).functor) == PredKind::Attribute;
    let obj = |kb: &Kb, f: ExprId| match kb.expr(f).args.get(1) {
        Some(Term::Ent(o)) => Some(*o),
        _ => None,
    };
    // Objects of relation r in case c.
    let objects = |kb: &Kb, c: CaseId, r: Sym| -> Vec<Sym> { kb.case(c).facts.iter().filter(|&&f| kb.expr(f).functor == r).filter_map(|&f| obj(kb, f)).collect() };
    // Global popularity per relation (tie-break and baseline).
    let pops: Vec<FxHashMap<Sym, f64>> = rels
        .iter()
        .map(|&r| {
            let mut p = FxHashMap::default();
            for &c in &mem {
                for o in objects(&kb, c, r) {
                    *p.entry(o).or_insert(0.0) += 1.0;
                }
            }
            p
        })
        .collect();

    // Length-1 rules r'(x,y) ⇒ r(x,y) (AMIE-style, same object): per-case counts,
    // so each query's confidence is estimated leave-one-out.
    let rule_counts = |kb: &Kb, c: CaseId| -> (FxHashMap<(Sym, Sym), f64>, FxHashMap<Sym, f64>) {
        let mut by_obj: FxHashMap<Sym, FxHashSet<Sym>> = FxHashMap::default();
        for &f in &kb.case(c).facts {
            if let (false, Some(o)) = (kb.vocab.kind(kb.expr(f).functor) == PredKind::Attribute, obj(kb, f)) {
                by_obj.entry(o).or_default().insert(kb.expr(f).functor);
            }
        }
        let (mut pair, mut body) = (FxHashMap::default(), FxHashMap::default());
        for ps in by_obj.values() {
            for &a in ps {
                *body.entry(a).or_insert(0.0) += 1.0;
                for &b in ps {
                    if a != b {
                        *pair.entry((a, b)).or_insert(0.0) += 1.0;
                    }
                }
            }
        }
        (pair, body)
    };
    let per_case_rules: Vec<_> = mem.iter().map(|&c| rule_counts(&kb, c)).collect();
    let (mut rule_pair, mut rule_body): (FxHashMap<(Sym, Sym), f64>, FxHashMap<Sym, f64>) = Default::default();
    for (p, b) in &per_case_rules {
        for (k2, v) in p {
            *rule_pair.entry(*k2).or_insert(0.0) += v;
        }
        for (k2, v) in b {
            *rule_body.entry(*k2).or_insert(0.0) += v;
        }
    }

    // Queries: entities with the relation and at least 3 other relation facts.
    let mut queries: Vec<Query> = Vec::new();
    for (ri, &r) in rels.iter().enumerate() {
        let mut idx: Vec<usize> = (0..n).collect();
        Rng::new(seed ^ 0xE27 ^ ri as u64).shuffle(&mut idx);
        let mut taken = 0;
        for i in idx {
            if taken == per_rel {
                break;
            }
            let c = mem[i];
            let facts = kb.case(c).facts.clone();
            let gold: FxHashSet<Sym> = objects(&kb, c, r).into_iter().collect();
            let kept_rel: Vec<ExprId> = facts.iter().copied().filter(|&f| !is_attr(&kb, f) && kb.expr(f).functor != r).collect();
            if gold.is_empty() || kept_rel.len() < 3 {
                continue;
            }
            let Some(Term::Ent(person)) = kb.expr(kept_rel[0]).args.first().copied() else { continue };
            let live: FxHashSet<Sym> = kept_rel.iter().filter_map(|&f| obj(&kb, f)).collect();
            let kept: Vec<ExprId> = facts.iter().copied().filter(|&f| !is_attr(&kb, f) && kb.expr(f).functor != r || is_attr(&kb, f) && matches!(kb.expr(f).args.first(), Some(Term::Ent(e)) if live.contains(e))).collect();
            let name = format!("q{}-{}", queries.len(), kb.name(kb.case(c).name));
            let q = kb.add_case(&name, CaseKind::Query, kept);
            queries.push(Query { case: q, orig: i, rel: ri, person, gold });
            taken += 1;
        }
    }
    let nq = queries.len();
    eprintln!("[e27] {n} {kg} cases, {} relations, {nq} queries ({:.1?})", rels.len(), t0.elapsed());

    // Representations (statistics from the memory only).
    let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
    let raw: Vec<Features> = mem.par_iter().map(|&c| fx.extract(c)).collect();
    let stats = FeatureStats::fit(raw.iter());
    let sk = Sketcher::new(Layout::default(), 0xF1);
    let sketch = |mut f: Features| {
        stats.apply(&mut f, &[true; N_CHANNELS]);
        sk.sketch(&f)
    };
    let fps: Vec<_> = raw.into_par_iter().map(sketch).collect();
    let qfps: Vec<_> = queries.par_iter().map(|q| sketch(fx.extract(q.case))).collect();
    let lex_raw: Vec<SparseVec> = mem.iter().map(|&c| lexical_tokens(&kb, c)).collect();
    let lex_stats = FeatureStats::fit(lex_raw.iter().map(|v| Features { channels: [v.clone(), vec![], vec![], vec![], vec![]] }).collect::<Vec<_>>().iter());
    let idf = |mut v: SparseVec| {
        lex_stats.apply_sparse(0, &mut v);
        v
    };
    let lex: Vec<SparseVec> = lex_raw.into_iter().map(idf).collect();
    let qlex: Vec<SparseVec> = queries.iter().map(|q| idf(lexical_tokens(&kb, q.case))).collect();
    let mapper = Mapper::new(&kb, MapConfig { include_attributes: true, ..Default::default() });
    let self_s: Vec<f64> = mem.par_iter().map(|&c| mapper.score(c, c) as f64).collect();
    // Fingerprint profile: `surface` (C0: labels and values, which identify
    // instances, E26) by default; `literal` and `analogy` weight structure.
    let (prof_name, prof) = match args.str("profile", "surface").as_str() {
        "literal" => ("literal", Profile::literal()),
        "analogy" => ("analogy", Profile::analogy()),
        _ => ("surface", Profile::surface_only()),
    };

    // Neighbour sets and their votes.
    struct Res {
        copy: Vec<Vec<Vec<Vote>>>, // [MARS fused, fingerprint, lexical, random, hybrid][analogue] votes
        ana: Vec<Vec<Vote>>,       // MARS fused analogues: analogy votes per analogue
        ana_lex: Vec<Vec<Vote>>,   // lexical analogues: analogy votes
        ana_hyb: Vec<Vec<Vote>>,   // hybrid analogues: analogy votes
        rules: Vec<Vote>,          // length-1 rule predictions (confidence as weight)
    }
    let res: Vec<Res> = queries
        .par_iter()
        .enumerate()
        .map(|(qi, q)| {
            let r = rels[q.rel];
            let others = (0..n).filter(|&c| c != q.orig);
            let mut by_fp: Vec<(usize, f64)> = others.clone().map(|c| (c, prof.score(&sk.channel_sims(&qfps[qi], &fps[c])))).collect();
            by_fp.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            by_fp.truncate(k);
            let qs = mapper.score(q.case, q.case) as f64;
            let mut fused: Vec<(usize, f64)> = by_fp
                .iter()
                .map(|&(c, fp)| {
                    let s = mapper.score(mem[c], q.case) as f64;
                    let fac = if s == 0.0 { 0.0 } else { (s / (qs * self_s[c]).sqrt()).min(1.0) };
                    (c, fac_w * fac + (1.0 - fac_w) * fp)
                })
                .collect();
            fused.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let mut lexs: Vec<(usize, f64)> = others.map(|c| (c, cosine(&qlex[qi], &lex[c]))).collect();
            lexs.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let mut rr = Rng::derive(seed ^ 0x5A, qi as u64);
            let mut rand: Vec<(usize, f64)> = Vec::new();
            while rand.len() < m_max {
                let c = rr.index(n);
                if c != q.orig && !rand.iter().any(|x| x.0 == c) {
                    rand.push((c, 1.0));
                }
            }
            // Hybrid: reciprocal-rank fusion of the MARS fused and lexical rankings.
            let mut rrf: FxHashMap<usize, f64> = FxHashMap::default();
            for l in [&fused, &lexs] {
                for (p, &(c, _)) in l.iter().take(k).enumerate() {
                    *rrf.entry(c).or_insert(0.0) += 60.0 / (60.0 + p as f64);
                }
            }
            let mut hybrid: Vec<(usize, f64)> = rrf.into_iter().collect();
            hybrid.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let sets = [&fused, &by_fp, &lexs, &rand, &hybrid];
            let copy = sets.iter().map(|s| s.iter().take(m_max).map(|&(c, w)| objects(&kb, mem[c], r).into_iter().map(|o| (o, w.max(1e-3), false)).collect()).collect()).collect();
            let analogy = |list: &Vec<(usize, f64)>| -> Vec<Vec<Vote>> {
                list.iter()
                    .take(m_max)
                    .map(|&(c, w)| {
                        let Some(m) = mapper.best(mem[c], q.case) else { return Vec::new() };
                        m.inferences
                            .iter()
                            .filter(|i| kb.expr(i.base_fact).functor == r)
                            .filter_map(|i| match &i.projected {
                                Proj::Expr { args, .. } if args.len() == 2 && args[0] == Proj::Target(Term::Ent(q.person)) => match args[1] {
                                    Proj::Target(Term::Ent(y)) => Some((y, w.max(1e-3), true)),
                                    Proj::Skolem(o) => Some((o, w.max(1e-3), false)),
                                    _ => None,
                                },
                                _ => None,
                            })
                            .collect()
                    })
                    .collect()
            };
            // Rules: each query object y of r' proposes r(x,y) with conf(r' ⇒ r), leave-one-out.
            let (own_pair, own_body) = &per_case_rules[q.orig];
            let mut best: FxHashMap<Sym, f64> = FxHashMap::default();
            for &f in &kb.case(q.case).facts {
                let rp = kb.expr(f).functor;
                if kb.vocab.kind(rp) == PredKind::Attribute {
                    continue;
                }
                let Some(y) = obj(&kb, f) else { continue };
                let num = rule_pair.get(&(rp, r)).copied().unwrap_or(0.0) - own_pair.get(&(rp, r)).copied().unwrap_or(0.0);
                let den = rule_body.get(&rp).copied().unwrap_or(0.0) - own_body.get(&rp).copied().unwrap_or(0.0);
                if num > 0.0 && den > 0.0 {
                    let e = best.entry(y).or_insert(0.0);
                    *e = e.max(num / den);
                }
            }
            let rules = best.into_iter().map(|(y, c)| (y, c, true)).collect();
            Res { copy, ana: analogy(&fused), ana_lex: analogy(&lexs), ana_hyb: analogy(&hybrid), rules }
        })
        .collect();
    eprintln!("[e27] mapped ({:.1?})", t0.elapsed());

    // Methods: name → per-query ranking at m analogues.
    let flat = |v: &[Vec<Vote>], m: usize| -> Vec<Vote> { v.iter().take(m).flatten().copied().collect() };
    // Neighbour votes as vote shares (sum 1), plus rule confidences.
    let share_plus = |votes: &[Vote], rules: &[Vote]| -> Vec<Vote> {
        let tot: f64 = votes.iter().map(|v| v.1).sum::<f64>().max(1e-9);
        votes.iter().map(|&(o, w, s)| (o, w / tot, s)).chain(rules.iter().copied()).collect()
    };
    type Method<'a> = (String, Box<dyn Fn(usize, &Res, usize) -> Ranked + Sync + 'a>);
    let methods: Vec<Method> = vec![
        ("popularity".into(), Box::new(|qi, _, _| rank(&pops[queries[qi].rel].iter().map(|(&o, &c)| (o, c, false)).collect::<Vec<_>>(), &pops[queries[qi].rel]))),
        ("copy · random neighbours".into(), Box::new(|qi, r, m| rank(&flat(&r.copy[3], m), &pops[queries[qi].rel]))),
        ("copy · lexical neighbours".into(), Box::new(|qi, r, m| rank(&flat(&r.copy[2], m), &pops[queries[qi].rel]))),
        ("copy · fingerprint neighbours".into(), Box::new(|qi, r, m| rank(&flat(&r.copy[1], m), &pops[queries[qi].rel]))),
        ("copy · MARS fused neighbours".into(), Box::new(|qi, r, m| rank(&flat(&r.copy[0], m), &pops[queries[qi].rel]))),
        ("analogy · lexical neighbours".into(), Box::new(|qi, r, m| rank(&flat(&r.ana_lex, m), &pops[queries[qi].rel]))),
        ("analogy · MARS fused neighbours".into(), Box::new(|qi, r, m| rank(&flat(&r.ana, m), &pops[queries[qi].rel]))),
        ("copy · hybrid neighbours (RRF MARS + lexical)".into(), Box::new(|qi, r, m| rank(&flat(&r.copy[4], m), &pops[queries[qi].rel]))),
        ("analogy · hybrid neighbours (RRF MARS + lexical)".into(), Box::new(|qi, r, m| rank(&flat(&r.ana_hyb, m), &pops[queries[qi].rel]))),
        ("rules (length-1, same object)".into(), Box::new(|qi, r, _| rank(&r.rules, &pops[queries[qi].rel]))),
        ("rules + copy · lexical (vote share + confidence)".into(), Box::new(|qi, r, m| rank(&share_plus(&flat(&r.copy[2], m), &r.rules), &pops[queries[qi].rel]))),
        ("rules + analogy · hybrid".into(), Box::new(|qi, r, m| rank(&share_plus(&flat(&r.ana_hyb, m), &r.rules), &pops[queries[qi].rel]))),
    ];
    let rel_names: Vec<String> = rels.iter().map(|&r| kb.name(r).to_string()).collect();
    let mut md = String::new();
    writeln!(md, "# E27: knowledge-graph completion by analogy ({tag})\n").unwrap();
    writeln!(md, "Data: `{dir}` ({kg} side, {n} entity cases). {nq} queries: an entity with all facts of one relation removed (and the labels of objects no longer mentioned), up to {per_rel} per relation, seed {seed}. Analogues: top-{m_max} (MARS fused = fingerprint-{prof_name} top-{k} re-ranked by {fac_w}·FAC + {:.1}·FP). Votes weighted by similarity; ties broken by popularity.\n", 1.0 - fac_w).unwrap();

    let table = |m: usize, md: &mut String| -> serde_json::Value {
        writeln!(md, "## {m} analogues\n\n| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 ({}) |\n|---|---|---|---|---|---|", rel_names.join(", ")).unwrap();
        let mut out = Vec::new();
        for (name, f) in &methods {
            let sc: Vec<[f64; 4]> = res.par_iter().enumerate().map(|(qi, r)| score(&f(qi, r, m), &queries[qi].gold)).collect();
            let col = |j: usize, sel: &dyn Fn(usize) -> bool| mean(&sc.iter().enumerate().filter(|(qi, _)| sel(*qi)).map(|(_, s)| s[j]).collect::<Vec<_>>());
            let all = |_: usize| true;
            let per: Vec<f64> = (0..rels.len()).map(|ri| col(0, &|qi| queries[qi].rel == ri)).collect();
            writeln!(md, "| {name} | {:.3} | {:.3} | {:.3} | {:.3} | {} |", col(0, &all), col(1, &all), col(2, &all), col(3, &all), per.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>().join(" / ")).unwrap();
            out.push(json!({"method": name, "hits1": col(0, &all), "hits10": col(1, &all), "mrr": col(2, &all), "coverage": col(3, &all), "per_relation_hits1": rel_names.iter().cloned().zip(per.iter().copied()).collect::<FxHashMap<String, f64>>()}));
        }
        writeln!(md).unwrap();
        json!({"m": m, "methods": out})
    };
    let mut tables = Vec::new();
    for m in [1, 5, m_max] {
        if m <= m_max && !tables.iter().any(|t: &serde_json::Value| t["m"] == m) {
            tables.push(table(m, &mut md));
        }
    }

    // Analogy vs copy with the same (MARS) analogues: substitutions and disagreements.
    let (mut n_sub, mut sub_ok, mut n_copy, mut copy_ok, mut dis, mut dis_ana, mut dis_copy) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    let mut sub_per_rel = vec![(0usize, 0usize); rels.len()];
    let mut calib: Vec<(usize, usize)> = vec![(0, 0); 6];
    for (qi, r) in res.iter().enumerate() {
        let q = &queries[qi];
        let a = rank(&flat(&r.ana, m_max), &pops[q.rel]);
        let c = rank(&flat(&r.copy[0], m_max), &pops[q.rel]);
        if let Some(&(o, _, sup, sub)) = a.list.first() {
            let ok = q.gold.contains(&o);
            // "Substitution" top-1: proposed only via correspondence to a query entity.
            if sub && !c.list.iter().any(|x| x.0 == o) {
                n_sub += 1;
                sub_ok += ok as usize;
                sub_per_rel[q.rel].0 += 1;
                sub_per_rel[q.rel].1 += ok as usize;
            } else {
                n_copy += 1;
                copy_ok += ok as usize;
            }
            let b = sup.min(5);
            calib[b].0 += 1;
            calib[b].1 += ok as usize;
            if let Some(&(oc, ..)) = c.list.first() {
                if oc != o {
                    dis += 1;
                    dis_ana += ok as usize;
                    dis_copy += q.gold.contains(&oc) as usize;
                }
            }
        }
    }
    let pct = |a: usize, b: usize| if b == 0 { f64::NAN } else { a as f64 / b as f64 };
    writeln!(md, "## Analogy vs copy (MARS analogues, m = {m_max})\n").unwrap();
    writeln!(md, "- Top-1 predictions that are pure substitutions (a query entity reached through the mapping, not an analogue's own object): {n_sub} of {} ({:.1}%), precision {:.3}; other top-1s precision {:.3}.", n_sub + n_copy, 100.0 * pct(n_sub, n_sub + n_copy), pct(sub_ok, n_sub), pct(copy_ok, n_copy)).unwrap();
    writeln!(md, "- Substitutions per relation (count, precision): {}.", rel_names.iter().zip(&sub_per_rel).map(|(r, &(a, b))| format!("{r} {a} ({:.2})", pct(b, a))).collect::<Vec<_>>().join(", ")).unwrap();
    writeln!(md, "- Where analogy's and copy's top-1 differ ({dis} queries): analogy right {dis_ana}, copy right {dis_copy}.\n").unwrap();
    writeln!(md, "**Calibration** (analogy · MARS, top-1 precision by support = analogues proposing it):\n\n| support | queries | precision |\n|---|---|---|").unwrap();
    for (s, &(a, b)) in calib.iter().enumerate().skip(1) {
        writeln!(md, "| {}{} | {a} | {:.3} |", s, if s == 5 { "+" } else { "" }, pct(b, a)).unwrap();
    }
    writeln!(md, "\nRuntime {:.1?}.", t0.elapsed()).unwrap();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E27-{tag}.md"), &md).map_err(|e| e.to_string())?;
    let cal: Vec<_> = calib.iter().enumerate().skip(1).map(|(s, &(a, b))| json!({"support": s, "n": a, "precision": pct(b, a)})).collect();
    let spr: Vec<_> = rel_names.iter().zip(&sub_per_rel).map(|(r, &(a, b))| json!({"relation": r, "n": a, "correct": b})).collect();
    let cfg = json!({"data": dir, "kg": kg, "relations": rel_names, "per_rel": per_rel, "k": k, "m": m_max, "profile": prof_name, "fac_weight": fac_w, "seed": seed, "queries": nq, "memory": n});
    let j = json!({"config": cfg, "tables": tables, "substitution": {"n": n_sub, "correct": sub_ok, "other_n": n_copy, "other_correct": copy_ok, "per_relation": spr}, "disagreements": {"n": dis, "analogy_right": dis_ana, "copy_right": dis_copy}, "calibration": cal});
    std::fs::write(format!("{out_dir}/E27-{tag}.json"), serde_json::to_string_pretty(&j).unwrap()).map_err(|e| e.to_string())?;
    print!("{md}");
    Ok(())
}
