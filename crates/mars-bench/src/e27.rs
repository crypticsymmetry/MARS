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
//! Baselines: global popularity of the relation's objects; mined rules with
//! leave-one-out confidence (the global counterpart of a substitution), length 1
//! r'(x,y) ⇒ r(x,y) and length ≤ 2 (adding r1(x,z) ∧ r2(z,y) ⇒ r(x,y)), alone and
//! added to neighbour votes.
//!
//! E28 (relational depth) runs the same experiment on `kg2mars --hop2` cases,
//! which also hold second-hop claims (the advisor's employer, the country of
//! origin's official language). Only the root entity's own facts of the relation
//! are held out, and the query keeps only facts still reachable from the root,
//! so a removed object cannot survive as the subject of its own claims.
//!
//! E29 (learning which transfers to trust): every analogical vote is typed as a
//! copy or a substitution via a query path (r1, or r1 · r2). The precision of
//! each (relation, type) is learned from the system's own transfer outcomes on
//! the other fold (2-fold by entity) and weights the votes ("gated analogy").
//! The learned table is reported: rules induced from analogy, next to the mined
//! confidence of the same pattern.
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

pub(crate) struct Query {
    pub(crate) case: CaseId,
    /// Index of the full entity case in the memory list.
    pub(crate) orig: usize,
    /// Index of the held-out relation.
    pub(crate) rel: usize,
    /// The root entity.
    pub(crate) person: Sym,
    /// The held-out objects.
    pub(crate) gold: FxHashSet<Sym>,
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

#[derive(Default)]
struct RuleCounts {
    pair1: FxHashMap<(Sym, Sym), f64>,
    body1: FxHashMap<Sym, f64>,
    pair2: FxHashMap<(Sym, Sym, Sym), f64>,
    body2: FxHashMap<(Sym, Sym), f64>,
}

impl RuleCounts {
    fn add(&mut self, o: &RuleCounts, sign: f64) {
        for (k, v) in &o.pair1 {
            *self.pair1.entry(*k).or_insert(0.0) += sign * v;
        }
        for (k, v) in &o.body1 {
            *self.body1.entry(*k).or_insert(0.0) += sign * v;
        }
        for (k, v) in &o.pair2 {
            *self.pair2.entry(*k).or_insert(0.0) += sign * v;
        }
        for (k, v) in &o.body2 {
            *self.body2.entry(*k).or_insert(0.0) += sign * v;
        }
    }
}

/// How an analogical vote reached its object: a copy of the analogue's own object
/// (skolem), or a substitution to a query entity reached from the root by the
/// relation path r1 (· r2). E29 learns per (relation, transfer) precision.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
enum Transfer {
    Copy,
    Path(Sym, Option<Sym>),
    Other,
}

/// Two-step paths from `x` in case `c`: (r1, r2) → objects y with r1(x,z) ∧ r2(z,y), y ≠ x.
fn paths(kb: &Kb, c: CaseId, x: Sym) -> FxHashMap<(Sym, Sym), FxHashSet<Sym>> {
    let rel = |f: ExprId| -> Option<(Sym, Sym, Sym)> {
        let e = kb.expr(f);
        if kb.vocab.kind(e.functor) == PredKind::Attribute {
            return None;
        }
        match (e.args.first(), e.args.get(1)) {
            (Some(Term::Ent(s)), Some(Term::Ent(o))) => Some((e.functor, *s, *o)),
            _ => None,
        }
    };
    let facts: Vec<(Sym, Sym, Sym)> = kb.case(c).facts.iter().filter_map(|&f| rel(f)).collect();
    let mut out: FxHashMap<(Sym, Sym), FxHashSet<Sym>> = FxHashMap::default();
    for &(a, s, z) in &facts {
        if s != x {
            continue;
        }
        for &(b, s2, y) in &facts {
            if s2 == z && y != x {
                out.entry((a, b)).or_default().insert(y);
            }
        }
    }
    out
}

pub(crate) fn is_attr(kb: &Kb, f: ExprId) -> bool {
    kb.vocab.kind(kb.expr(f).functor) == PredKind::Attribute
}

pub(crate) fn obj(kb: &Kb, f: ExprId) -> Option<Sym> {
    match kb.expr(f).args.get(1) {
        Some(Term::Ent(o)) => Some(*o),
        _ => None,
    }
}

/// The case's root entity: the most frequent subject (`--hop2` cases also hold
/// facts about the root's objects, e.g. the doctoral advisor's employer).
pub(crate) fn root_of(kb: &Kb, c: CaseId) -> Option<Sym> {
    let mut cnt: FxHashMap<Sym, usize> = FxHashMap::default();
    for &f in &kb.case(c).facts {
        if let (false, Some(Term::Ent(x))) = (is_attr(kb, f), kb.expr(f).args.first()) {
            *cnt.entry(*x).or_insert(0) += 1;
        }
    }
    cnt.into_iter().max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0))).map(|x| x.0)
}

/// Is `f` a relation fact about entity `x`?
pub(crate) fn own(kb: &Kb, f: ExprId, x: Sym) -> bool {
    !is_attr(kb, f) && kb.expr(f).args.first() == Some(&Term::Ent(x))
}

/// Objects of relation `r` of entity `x` in case `c`.
pub(crate) fn objects(kb: &Kb, c: CaseId, x: Sym, r: Sym) -> Vec<Sym> {
    kb.case(c).facts.iter().filter(|&&f| kb.expr(f).functor == r && own(kb, f, x)).filter_map(|&f| obj(kb, f)).collect()
}

/// Hold-out queries (added to `kb` as query cases): up to `per_rel` entities per
/// relation, each with all its own facts of the relation removed and only the
/// facts still reachable from the root kept.
pub(crate) fn build_queries(kb: &mut Kb, mem: &[CaseId], roots: &[Sym], rels: &[Sym], per_rel: usize, seed: u64) -> Vec<Query> {
    // Queries: entities with the relation and at least 3 other facts of their own.
    let mut queries: Vec<Query> = Vec::new();
    for (ri, &r) in rels.iter().enumerate() {
        let mut idx: Vec<usize> = (0..mem.len()).collect();
        Rng::new(seed ^ 0xE27 ^ ri as u64).shuffle(&mut idx);
        let mut taken = 0;
        for i in idx {
            if taken == per_rel {
                break;
            }
            let (c, person) = (mem[i], roots[i]);
            let facts = kb.case(c).facts.clone();
            let gold: FxHashSet<Sym> = objects(kb, c, person, r).into_iter().collect();
            let held = |f: ExprId| kb.expr(f).functor == r && own(kb, f, person);
            let kept_own = facts.iter().filter(|&&f| own(kb, f, person) && !held(f)).count();
            if gold.is_empty() || kept_own < 3 {
                continue;
            }
            // Keep only relation facts still reachable from the root without the held
            // facts: a held object must not survive as the subject of its own
            // second-hop claims (e.g. the removed alma mater's country).
            let mut reach: FxHashSet<Sym> = FxHashSet::from_iter([person]);
            loop {
                let before = reach.len();
                for &f in &facts {
                    if let (false, false, Some(Term::Ent(x)), Some(y)) = (is_attr(kb, f), held(f), kb.expr(f).args.first(), obj(kb, f)) {
                        if reach.contains(x) {
                            reach.insert(y);
                        }
                    }
                }
                if reach.len() == before {
                    break;
                }
            }
            let kept_rel: Vec<ExprId> = facts.iter().copied().filter(|&f| !is_attr(kb, f) && !held(f) && matches!(kb.expr(f).args.first(), Some(Term::Ent(x)) if reach.contains(x))).collect();
            let live: FxHashSet<Sym> = kept_rel.iter().flat_map(|&f| kb.expr(f).args.iter().filter_map(|a| if let Term::Ent(e) = a { Some(*e) } else { None }).collect::<Vec<_>>()).collect();
            let kept: Vec<ExprId> = kept_rel.iter().copied().chain(facts.iter().copied().filter(|&f| is_attr(kb, f) && matches!(kb.expr(f).args.first(), Some(Term::Ent(e)) if live.contains(e)))).collect();
            let name = format!("q{}-{}", queries.len(), kb.name(kb.case(c).name));
            let q = kb.add_case(&name, CaseKind::Query, kept);
            queries.push(Query { case: q, orig: i, rel: ri, person, gold });
            taken += 1;
        }
    }
    queries
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
    let roots: Vec<Sym> = mem.iter().map(|&c| root_of(&kb, c).unwrap_or(Sym(0))).collect();
    // Global popularity per relation (tie-break and baseline).
    let pops: Vec<FxHashMap<Sym, f64>> = rels
        .iter()
        .map(|&r| {
            let mut p = FxHashMap::default();
            for (i, &c) in mem.iter().enumerate() {
                for o in objects(&kb, c, roots[i], r) {
                    *p.entry(o).or_insert(0.0) += 1.0;
                }
            }
            p
        })
        .collect();

    // Rules (AMIE-style), counted per case so each query's confidence is estimated
    // leave-one-out: length 1, r'(x,y) ⇒ r(x,y); length 2, r1(x,z) ∧ r2(z,y) ⇒ r(x,y).
    // Confidence = #(x,y) with body and head / #(x,y) with body.
    let rule_counts = |kb: &Kb, c: CaseId, x: Sym| -> RuleCounts {
        let mut rc = RuleCounts::default();
        let mine: Vec<(Sym, Sym)> = kb.case(c).facts.iter().filter(|&&f| own(kb, f, x)).filter_map(|&f| Some((kb.expr(f).functor, obj(kb, f)?))).collect();
        let mut by_obj: FxHashMap<Sym, FxHashSet<Sym>> = FxHashMap::default();
        for &(a, y) in &mine {
            by_obj.entry(y).or_default().insert(a);
        }
        for ps in by_obj.values() {
            for &a in ps {
                *rc.body1.entry(a).or_insert(0.0) += 1.0;
                for &b in ps {
                    if a != b {
                        *rc.pair1.entry((a, b)).or_insert(0.0) += 1.0;
                    }
                }
            }
        }
        for ((a, b), ys) in paths(kb, c, x) {
            *rc.body2.entry((a, b)).or_insert(0.0) += ys.len() as f64;
            for y in ys {
                if let Some(heads) = by_obj.get(&y) {
                    for &h in heads {
                        *rc.pair2.entry((a, b, h)).or_insert(0.0) += 1.0;
                    }
                }
            }
        }
        rc
    };
    let per_case_rules: Vec<RuleCounts> = mem.iter().enumerate().map(|(i, &c)| rule_counts(&kb, c, roots[i])).collect();
    let mut all_rules = RuleCounts::default();
    for rc in &per_case_rules {
        all_rules.add(rc, 1.0);
    }

    let queries = build_queries(&mut kb, &mem, &roots, &rels, per_rel, seed);
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
        rules2: Vec<Vote>,         // length ≤ 2 rule predictions
        ypaths: FxHashMap<Sym, Vec<Transfer>>, // query entity → paths reaching it from the root
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
            let copy = sets.iter().map(|s| s.iter().take(m_max).map(|&(c, w)| objects(&kb, mem[c], roots[c], r).into_iter().map(|o| (o, w.max(1e-3), false)).collect()).collect()).collect();
            let analogy = |list: &Vec<(usize, f64)>| -> Vec<Vec<Vote>> {
                list.iter()
                    .take(m_max)
                    .map(|&(c, w)| {
                        let Some(m) = mapper.best(mem[c], q.case) else { return Vec::new() };
                        m.inferences
                            .iter()
                            .filter(|i| kb.expr(i.base_fact).functor == r && kb.expr(i.base_fact).args.first() == Some(&Term::Ent(roots[c])))
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
            // Rules, leave-one-out: each query object y of r' proposes r(x,y) with
            // conf(r' ⇒ r); each 2-path r1(x,z) ∧ r2(z,y) with conf(r1·r2 ⇒ r).
            let mut loo = RuleCounts::default();
            loo.add(&all_rules, 1.0);
            loo.add(&per_case_rules[q.orig], -1.0);
            let conf = |num: Option<&f64>, den: Option<&f64>| match (num.copied().unwrap_or(0.0), den.copied().unwrap_or(0.0)) {
                (a, b) if a > 0.0 && b > 0.0 => Some(a / b),
                _ => None,
            };
            let mut best1: FxHashMap<Sym, f64> = FxHashMap::default();
            for &f in &kb.case(q.case).facts {
                if !own(&kb, f, q.person) {
                    continue;
                }
                let (rp, Some(y)) = (kb.expr(f).functor, obj(&kb, f)) else { continue };
                if let Some(cf) = conf(loo.pair1.get(&(rp, r)), loo.body1.get(&rp)) {
                    let e = best1.entry(y).or_insert(0.0);
                    *e = e.max(cf);
                }
            }
            let mut best2 = best1.clone();
            for ((a, b), ys) in paths(&kb, q.case, q.person) {
                if let Some(cf) = conf(loo.pair2.get(&(a, b, r)), loo.body2.get(&(a, b))) {
                    for y in ys {
                        let e = best2.entry(y).or_insert(0.0);
                        *e = e.max(cf);
                    }
                }
            }
            let rules = best1.into_iter().map(|(y, c)| (y, c, true)).collect();
            let rules2 = best2.into_iter().map(|(y, c)| (y, c, true)).collect();
            let mut ypaths: FxHashMap<Sym, Vec<Transfer>> = FxHashMap::default();
            for &f in &kb.case(q.case).facts {
                if let (true, Some(y)) = (own(&kb, f, q.person), obj(&kb, f)) {
                    ypaths.entry(y).or_default().push(Transfer::Path(kb.expr(f).functor, None));
                }
            }
            for ((a, b), ys) in paths(&kb, q.case, q.person) {
                for y in ys {
                    ypaths.entry(y).or_default().push(Transfer::Path(a, Some(b)));
                }
            }
            Res { copy, ana: analogy(&fused), ana_lex: analogy(&lexs), ana_hyb: analogy(&hybrid), rules, rules2, ypaths }
        })
        .collect();
    eprintln!("[e27] mapped ({:.1?})", t0.elapsed());

    // E29: learn which transfers to trust. Each analogical vote has a type (copy,
    // or substitution via a query path); the precision of (relation, type) is
    // learned from the outcomes of the system's own transfers on the other fold
    // (2-fold by entity) and weights the vote. Smoothed towards the fold's mean
    // vote precision (k = 5 pseudo-votes).
    let fold = |qi: usize| queries[qi].orig % 2;
    let types = |r: &Res, v: &Vote| -> Vec<Transfer> {
        if !v.2 {
            vec![Transfer::Copy]
        } else {
            r.ypaths.get(&v.0).cloned().unwrap_or_else(|| vec![Transfer::Other])
        }
    };
    type Gate = (FxHashMap<(usize, Transfer), (f64, f64)>, f64);
    let learn = |sel: &dyn Fn(&Res) -> &Vec<Vec<Vote>>, keep: &dyn Fn(usize) -> bool| -> Gate {
        let mut t: FxHashMap<(usize, Transfer), (f64, f64)> = FxHashMap::default();
        let (mut h, mut n_all) = (0.0, 0.0);
        for (qi, r) in res.iter().enumerate().filter(|(qi, _)| keep(*qi)) {
            let q = &queries[qi];
            for v in sel(r).iter().take(m_max).flatten() {
                let ok = q.gold.contains(&v.0) as u8 as f64;
                h += ok;
                n_all += 1.0;
                for ty in types(r, v) {
                    let e = t.entry((q.rel, ty)).or_insert((0.0, 0.0));
                    e.0 += ok;
                    e.1 += 1.0;
                }
            }
        }
        (t, if n_all > 0.0 { h / n_all } else { 0.0 })
    };
    let gates_ana: [Gate; 2] = [learn(&|r| &r.ana, &|qi| fold(qi) == 0), learn(&|r| &r.ana, &|qi| fold(qi) == 1)];
    let gates_hyb: [Gate; 2] = [learn(&|r| &r.ana_hyb, &|qi| fold(qi) == 0), learn(&|r| &r.ana_hyb, &|qi| fold(qi) == 1)];
    let gated = |qi: usize, r: &Res, votes: &[Vec<Vote>], gates: &[Gate; 2], m: usize| -> Vec<Vote> {
        let (t, p0) = &gates[1 - fold(qi)];
        let rel = queries[qi].rel;
        votes
            .iter()
            .take(m)
            .flatten()
            .map(|v| {
                let p = types(r, v).iter().map(|ty| t.get(&(rel, *ty)).map(|&(h, n)| (h + 5.0 * p0) / (n + 5.0)).unwrap_or(*p0)).fold(0.0, f64::max);
                (v.0, v.1 * p, v.2)
            })
            .collect()
    };

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
        ("rules ≤ 2 (length-1 and 2-path)".into(), Box::new(|qi, r, _| rank(&r.rules2, &pops[queries[qi].rel]))),
        ("rules ≤ 2 + copy · lexical".into(), Box::new(|qi, r, m| rank(&share_plus(&flat(&r.copy[2], m), &r.rules2), &pops[queries[qi].rel]))),
        ("rules ≤ 2 + analogy · hybrid".into(), Box::new(|qi, r, m| rank(&share_plus(&flat(&r.ana_hyb, m), &r.rules2), &pops[queries[qi].rel]))),
        ("gated analogy · MARS fused (E29)".into(), Box::new(|qi, r, m| rank(&gated(qi, r, &r.ana, &gates_ana, m), &pops[queries[qi].rel]))),
        ("gated analogy · hybrid (E29)".into(), Box::new(|qi, r, m| rank(&gated(qi, r, &r.ana_hyb, &gates_hyb, m), &pops[queries[qi].rel]))),
        ("rules ≤ 2 + gated analogy · hybrid (E29)".into(), Box::new(|qi, r, m| rank(&share_plus(&gated(qi, r, &r.ana_hyb, &gates_hyb, m), &r.rules2), &pops[queries[qi].rel]))),
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
    // Transfers learned from analogy (all queries, MARS analogues): the induced
    // "rules", with the mined rule confidence of the same pattern for comparison.
    let (learned, p0_all) = learn(&|r| &r.ana, &|_| true);
    let name_t = |ty: &Transfer| match ty {
        Transfer::Copy => "copy (analogue's own object)".to_string(),
        Transfer::Path(a, None) => kb.name(*a).to_string(),
        Transfer::Path(a, Some(b)) => format!("{} · {}", kb.name(*a), kb.name(*b)),
        Transfer::Other => "other substitution".to_string(),
    };
    let mined = |rel: usize, ty: &Transfer| -> Option<f64> {
        let r = rels[rel];
        match ty {
            Transfer::Path(a, None) => Some(all_rules.pair1.get(&(*a, r)).copied().unwrap_or(0.0) / all_rules.body1.get(a).copied().unwrap_or(f64::INFINITY)),
            Transfer::Path(a, Some(b)) => Some(all_rules.pair2.get(&(*a, *b, r)).copied().unwrap_or(0.0) / all_rules.body2.get(&(*a, *b)).copied().unwrap_or(f64::INFINITY)),
            _ => None,
        }
    };
    let mut lt: Vec<_> = learned.iter().filter(|(_, v)| v.1 >= 30.0).collect();
    lt.sort_by(|a, b| a.0 .0.cmp(&b.0 .0).then((b.1 .0 / b.1 .1).total_cmp(&(a.1 .0 / a.1 .1))).then(a.0 .1.cmp(&b.0 .1)));
    writeln!(md, "\n## Transfers learned from analogy (E29)\n\nPrecision of analogical votes by (relation, transfer type), MARS analogues, all queries (vote-level precision overall {p0_all:.3}); transfer types with ≥ 30 votes, top 4 per relation. `mined` is the confidence of the same pattern as an explicitly mined rule.\n\n| relation | transfer | votes | precision | mined rule confidence |\n|---|---|---|---|---|").unwrap();
    let mut per_rel_count = vec![0usize; rels.len()];
    let mut lj = Vec::new();
    for ((rel, ty), (h, n)) in lt {
        lj.push(json!({"relation": rel_names[*rel], "transfer": name_t(ty), "votes": n, "precision": h / n, "mined": mined(*rel, ty)}));
        if per_rel_count[*rel] < 4 {
            per_rel_count[*rel] += 1;
            writeln!(md, "| {} | {} | {} | {:.3} | {} |", rel_names[*rel], name_t(ty), n, h / n, mined(*rel, ty).map(|x| format!("{x:.3}")).unwrap_or("—".into())).unwrap();
        }
    }
    writeln!(md, "\nRuntime {:.1?}.", t0.elapsed()).unwrap();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E27-{tag}.md"), &md).map_err(|e| e.to_string())?;
    let cal: Vec<_> = calib.iter().enumerate().skip(1).map(|(s, &(a, b))| json!({"support": s, "n": a, "precision": pct(b, a)})).collect();
    let spr: Vec<_> = rel_names.iter().zip(&sub_per_rel).map(|(r, &(a, b))| json!({"relation": r, "n": a, "correct": b})).collect();
    let cfg = json!({"data": dir, "kg": kg, "relations": rel_names, "per_rel": per_rel, "k": k, "m": m_max, "profile": prof_name, "fac_weight": fac_w, "seed": seed, "queries": nq, "memory": n});
    let j = json!({"config": cfg, "tables": tables, "substitution": {"n": n_sub, "correct": sub_ok, "other_n": n_copy, "other_correct": copy_ok, "per_relation": spr}, "disagreements": {"n": dis, "analogy_right": dis_ana, "copy_right": dis_copy}, "calibration": cal, "learned_transfers": lj});
    std::fs::write(format!("{out_dir}/E27-{tag}.json"), serde_json::to_string_pretty(&j).unwrap()).map_err(|e| e.to_string())?;
    print!("{md}");
    Ok(())
}
