//! E14: analogical bootstrapping of vocabulary alignment on real code.
//!
//! The E12 corpus converted in *raw* mode (`MARS_RAW=1`): library calls keep
//! their language-specific names (`py:len`, `py:append`, `js:length`,
//! `js:push`, `js:Math-max`, ...) as unresolved predicates, instead of the
//! hand-written py↔js mapping the converters normally apply. The E13 loop
//! (cross-language neighbours → wildcard mappings → mutual-best predicate
//! pairs under a one-per-language constraint → EM re-estimation) runs
//! unsupervised; the converters' hand mapping only scores the alignment.
//! Retrieval is scored on the E12 task (same algorithm, other language), with
//! the hand-mapped corpus as the reference.

use crate::e9::{load_corpus, Meta};
use crate::metrics::{mrr, recall_at};
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, N_CHANNELS};
use mars_map::{MapConfig, Mapper};
use mars_rel::{Kb, PredKind, Sym, Term};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

const PY_KNOWN: &[&str] = &[
    "len", "min", "max", "abs", "sum", "sorted", "reversed", "enumerate", "zip", "int", "float", "str", "list", "dict", "set", "tuple", "print", "isinstance", "divmod", "pow", "round", "map", "filter", "any", "all", "iter", "next", "ord", "chr", "bool",
    "type", "hash", "append", "pop", "insert", "extend", "remove", "index", "count", "sort", "reverse", "get", "keys", "values", "items", "add", "update", "copy", "clear", "popleft", "appendleft", "join", "split", "format", "startswith", "endswith", "lower", "upper",
    "strip", "setdefault", "discard",
];
const JS_KNOWN: &[(&str, &str)] = &[
    ("length", "len"), ("push", "append"), ("pop", "pop"), ("shift", "pop"), ("unshift", "insert"), ("indexOf", "index"), ("sort", "sort"), ("reverse", "reverse"), ("join", "join"), ("split", "split"), ("keys", "keys"), ("values", "values"), ("add", "add"),
    ("get", "get"), ("has", "get"), ("set", "update"), ("toLowerCase", "lower"), ("toUpperCase", "upper"), ("trim", "strip"), ("Math-min", "min"), ("Math-max", "max"), ("Math-abs", "abs"), ("Math-pow", "pow"), ("Math-round", "round"), ("Math-floor", "int"),
    ("Math-ceil", "int"), ("Math-trunc", "int"), ("parseInt", "int"), ("parseFloat", "float"), ("String", "str"), ("Number", "float"), ("Boolean", "bool"), ("console-log", "print"),
];

fn lang_of(name: &str) -> Option<&str> {
    name.split_once(':').map(|x| x.0).filter(|l| *l == "py" || *l == "js")
}

/// The meaning the hand mapping of the converters assigns (None: outside it).
fn gold(name: &str) -> Option<&'static str> {
    let (l, x) = name.split_once(':')?;
    match l {
        "py" => PY_KNOWN.iter().copied().find(|k| *k == x),
        "js" => JS_KNOWN.iter().find(|k| k.0 == x).map(|k| k.1),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Verdict {
    Correct,
    Wrong,
    /// Outside the hand map, but the identifiers agree up to case and `_`
    /// (`py:is_empty` / `js:isEmpty`): user-defined APIs, judged correct.
    SameName,
    Unverifiable,
}

fn bare(name: &str) -> String {
    name.split_once(':').map(|x| x.1).unwrap_or(name).chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_lowercase()
}

fn verdict(a: &str, b: &str) -> Verdict {
    match (gold(a), gold(b)) {
        (Some(x), Some(y)) if x == y => Verdict::Correct,
        (Some(_), Some(_)) => Verdict::Wrong,
        _ if bare(a) == bare(b) => Verdict::SameName,
        _ => Verdict::Unverifiable,
    }
}

struct Reps {
    fps: Vec<mars_hv::HyperVector>,
    sk: Sketcher,
}

fn represent(kb: &Kb, metas: &[Meta]) -> Reps {
    let fx = FeatureExtractor::new(kb, FeatureConfig::default());
    let mut feats: Vec<Features> = metas.par_iter().map(|m| fx.extract(m.case)).collect();
    let stats = FeatureStats::fit(feats.iter());
    feats.par_iter_mut().for_each(|f| stats.apply(f, &[true; N_CHANNELS]));
    let sk = Sketcher::new(Layout::default(), 0xF1);
    Reps { fps: feats.par_iter().map(|f| sk.sketch(f)).collect(), sk }
}

const METHODS: [&str; 5] = ["fingerprint analogy", "fingerprint literal", "FAC only", "fused ½FAC+½FP-literal", "fused 0.3FAC+0.7FP-literal"];

/// Task A (same algorithm, other language): per method (R@1, R@10, MRR).
fn evaluate(kb: &Kb, metas: &[Meta], reps: &Reps, queries: &[usize]) -> Vec<(f64, f64, f64)> {
    let n = metas.len();
    let mapper = Mapper::new(kb, MapConfig::default());
    let self_s: Vec<f64> = metas.par_iter().map(|m| mapper.score(m.case, m.case) as f64).collect();
    let (an, li) = (Profile::analogy(), Profile::literal());
    let ranks: Vec<[usize; 5]> = queries
        .par_iter()
        .map(|&q| {
            let scores: Vec<[f64; 5]> = (0..n)
                .map(|c| {
                    let sims = reps.sk.channel_sims(&reps.fps[q], &reps.fps[c]);
                    let s = mapper.score(metas[c].case, metas[q].case) as f64;
                    let fac = if s == 0.0 { 0.0 } else { (s / (self_s[q] * self_s[c]).sqrt()).min(1.0) };
                    let (fa, fl) = (an.score(&sims), li.score(&sims));
                    [fa, fl, fac, 0.5 * fac + 0.5 * fl, 0.3 * fac + 0.7 * fl]
                })
                .collect();
            let relevant = |c: usize| metas[c].is_main && metas[c].norm == metas[q].norm && metas[c].lang != metas[q].lang;
            std::array::from_fn(|m| {
                let mut order: Vec<usize> = (0..n).filter(|&c| c != q).collect();
                order.sort_by(|&a, &b| scores[b][m].total_cmp(&scores[a][m]).then(a.cmp(&b)));
                order.iter().position(|&c| relevant(c)).unwrap_or(usize::MAX / 2)
            })
        })
        .collect();
    (0..METHODS.len())
        .map(|m| {
            let r: Vec<usize> = ranks.iter().map(|x| x[m]).collect();
            (recall_at(&r, 1), recall_at(&r, 10), mrr(&r))
        })
        .collect()
}

fn cross_lang_queries(metas: &[Meta]) -> Vec<usize> {
    (0..metas.len())
        .filter(|&i| metas[i].is_main && (0..metas.len()).any(|j| metas[j].is_main && metas[j].norm == metas[i].norm && metas[j].lang != metas[i].lang))
        .collect()
}

fn row(md: &mut String, label: &str, extra: &str, res: &[(f64, f64, f64)]) {
    write!(md, "| {label} | {extra} |").unwrap();
    for (r1, r10, m) in res {
        write!(md, " {m:.3} ({r1:.2}/{r10:.2}) |").unwrap();
    }
    writeln!(md).unwrap();
}

pub fn run(args: &Args) -> Result<(), String> {
    let dir = args.str("data", "data/e14");
    let ref_dir = args.str("reference", "data/e9");
    let rounds = args.usize("rounds", 3);
    let neighbours = args.usize("neighbours", 10);
    let theta = args.f64("theta", 0.2);
    let wildcard = args.f64("wildcard", 0.3) as f32;
    let min_evidence = args.f64("min-evidence", 1.0);
    let reestimate = args.str("reestimate", "yes") == "yes";
    // `raw`: mutual best by summed evidence (E13); `cosine`: by evidence
    // normalized by each predicate's total evidence, which stops frequent
    // predicates from pairing with whatever frequent predicate co-occurs.
    let assoc = args.str("assoc", "raw");
    let out_dir = args.str("out", "results/E14");
    let t0 = Instant::now();
    let (mut kb, metas) = load_corpus(&dir)?;
    let n = metas.len();
    let queries = cross_lang_queries(&metas);
    let preds: Vec<Sym> = {
        let mut v: Vec<Sym> = Vec::new();
        for m in &metas {
            for e in kb.case_exprs(m.case) {
                let f = kb.expr(e).functor;
                if lang_of(kb.name(f)).is_some() && !v.contains(&f) {
                    v.push(f);
                }
            }
        }
        v.sort_by(|a, b| kb.name(*a).cmp(kb.name(*b)));
        v
    };
    let n_py = preds.iter().filter(|&&p| lang_of(kb.name(p)) == Some("py")).count();
    let alignable: usize = {
        // Gold pairs: (py, js) predicates with the same hand-mapped meaning.
        let mut c = 0;
        for &a in &preds {
            for &b in &preds {
                if lang_of(kb.name(a)) == Some("py") && lang_of(kb.name(b)) == Some("js") && verdict(kb.name(a), kb.name(b)) == Verdict::Correct {
                    c += 1;
                }
            }
        }
        c
    };
    eprintln!("[e14] {n} functions, {} queries, {} raw predicates ({n_py} py), {alignable} gold pairs ({:.1?})", queries.len(), preds.len(), t0.elapsed());

    let mut md = String::new();
    writeln!(md, "# E14: bootstrapped vocabulary alignment on real code\n").unwrap();
    writeln!(md, "The E12 corpus ({n} functions, Python + JavaScript) converted in *raw* mode: library calls keep language-specific names (`py:len`, `js:length`, `js:push`, `js:Math-max`, …) as unresolved predicates — {} of them ({n_py} Python, {} JavaScript), {alignable} py↔js pairs mean the same thing according to the converters' hand mapping. Each round retrieves {neighbours} other-language neighbours per function (fingerprint analogy profile), maps each pair with wildcard matching (local score {wildcard}), harvests predicate correspondences from mappings with normalized score ≥ {theta}, and aligns mutual-best pairs (association `{assoc}`, evidence ≥ {min_evidence}), one predicate per language per cluster, re-estimated from scratch each round. The hand mapping is used only to score.\n", preds.len(), preds.len() - n_py).unwrap();
    writeln!(md, "Retrieval: E12 task A ({} queries: main functions whose algorithm exists in the other language, ranked among all {n} functions). Cells: MRR (R@1/R@10).\n", queries.len()).unwrap();
    write!(md, "| round | aligned pairs: correct / same name / wrong / unjudged |").unwrap();
    for m in METHODS {
        write!(md, " {m} |").unwrap();
    }
    writeln!(md, "\n|---|---|{}", "---|".repeat(METHODS.len())).unwrap();

    let mut rows = Vec::new();
    let mut final_pairs: Vec<(Sym, Sym, f64)> = Vec::new();
    let mut aligned_pairs: Vec<(Sym, Sym, f64)> = Vec::new();
    for round in 0..=rounds {
        let tr = Instant::now();
        let reps = represent(&kb, &metas);
        let res = evaluate(&kb, &metas, &reps, &queries);
        let vs: Vec<Verdict> = aligned_pairs.iter().map(|(a, b, _)| verdict(kb.name(*a), kb.name(*b))).collect();
        let count = |v: Verdict| vs.iter().filter(|&&x| x == v).count();
        let (ok, bad, same, unv) = (count(Verdict::Correct), count(Verdict::Wrong), count(Verdict::SameName), count(Verdict::Unverifiable));
        row(&mut md, &round.to_string(), &format!("{ok} / {same} / {bad} / {unv}"), &res);
        rows.push(json!({"round": round, "correct": ok, "same_name": same, "wrong": bad, "unverifiable": unv, "methods": METHODS.iter().zip(&res).map(|(m, r)| json!({"method": m, "r1": r.0, "r10": r.1, "mrr": r.2})).collect::<Vec<_>>()}));
        eprintln!("[e14] round {round}: {ok}/{same}/{bad}/{unv}, fused MRR {:.3} ({:.1?})", res[4].2, tr.elapsed());
        if round == rounds {
            final_pairs = aligned_pairs.clone();
            break;
        }

        // 1. Other-language neighbours (exhaustive: the corpus is small).
        let an = Profile::analogy();
        let pairs: Vec<(usize, usize)> = (0..n)
            .into_par_iter()
            .flat_map_iter(|i| {
                let mut s: Vec<(usize, f64)> = (0..n).filter(|&j| metas[j].lang != metas[i].lang).map(|j| (j, an.score(&reps.sk.channel_sims(&reps.fps[i], &reps.fps[j])))).collect();
                s.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                s.into_iter().take(neighbours).map(move |(j, _)| (i, j))
            })
            .collect();
        // 2–3. Wildcard mappings -> correspondence evidence.
        let wm = Mapper::new(&kb, MapConfig { wildcard: Some(wildcard), ..Default::default() });
        let selfs: Vec<f64> = metas.par_iter().map(|m| wm.score(m.case, m.case) as f64).collect();
        let kbr = &kb;
        let evidence: FxHashMap<(Sym, Sym), f64> = pairs
            .par_iter()
            .fold(FxHashMap::default, |mut acc: FxHashMap<(Sym, Sym), f64>, &(i, j)| {
                if let Some(m) = wm.best(metas[j].case, metas[i].case) {
                    let norm = m.score as f64 / (selfs[i] * selfs[j]).sqrt();
                    if norm >= theta {
                        for (b, t) in &m.correspondences {
                            if let (Term::Expr(be), Term::Expr(te)) = (b, t) {
                                let (fb, ft) = (kbr.expr(*be).functor, kbr.expr(*te).functor);
                                let (lb, lt) = (lang_of(kbr.name(fb)), lang_of(kbr.name(ft)));
                                if lb.is_some() && lt.is_some() && lb != lt {
                                    let key = if fb < ft { (fb, ft) } else { (ft, fb) };
                                    *acc.entry(key).or_insert(0.0) += norm;
                                }
                            }
                        }
                    }
                }
                acc
            })
            .reduce(FxHashMap::default, |mut a, b| {
                for (k, v) in b {
                    *a.entry(k).or_insert(0.0) += v;
                }
                a
            });
        // 4. Mutual best partners (two languages: clusters are pairs).
        let mut total: FxHashMap<Sym, f64> = FxHashMap::default();
        for (&(a, b), &w) in &evidence {
            *total.entry(a).or_insert(0.0) += w;
            *total.entry(b).or_insert(0.0) += w;
        }
        let strength = |a: Sym, b: Sym, w: f64| if assoc == "cosine" { w / (total[&a] * total[&b]).sqrt() } else { w };
        let mut best: FxHashMap<Sym, (Sym, f64)> = FxHashMap::default();
        for (&(a, b), &w) in &evidence {
            let w = strength(a, b, w);
            for (x, y) in [(a, b), (b, a)] {
                let e = best.entry(x).or_insert((y, 0.0));
                if w > e.1 || (w == e.1 && y < e.0) {
                    *e = (y, w);
                }
            }
        }
        let mut merges: Vec<(Sym, Sym, f64)> = evidence
            .iter()
            .filter(|(&(a, b), &w)| w >= min_evidence && best.get(&a).map(|x| x.0) == Some(b) && best.get(&b).map(|x| x.0) == Some(a))
            .map(|(&(a, b), &w)| if lang_of(kb.name(a)) == Some("py") { (a, b, w) } else { (b, a, w) })
            .collect();
        merges.sort_by(|x, y| y.2.total_cmp(&x.2).then(x.0.cmp(&y.0)));
        eprintln!("[e14] round {round}: {} pairs mapped, {} evidence pairs, {} mutual-best", pairs.len(), evidence.len(), merges.len());
        // 5. Re-estimate the alignment from this round's evidence and materialize it.
        if reestimate {
            for &p in &preds {
                kb.vocab.set_parents(p, Vec::new());
            }
            aligned_pairs.clear();
        }
        for (a, b, w) in merges {
            if aligned_pairs.iter().any(|x| x.0 == a || x.1 == b) {
                continue;
            }
            let c = kb.declare(&format!("aligned-{}", kb.name(a).replace(':', "-")), None, PredKind::Relation, false, &[]);
            kb.vocab.set_parents(a, vec![c]);
            kb.vocab.set_parents(b, vec![c]);
            aligned_pairs.push((a, b, w));
        }
    }

    // Oracle alignment of the raw corpus: every hand-mapped meaning becomes one
    // canonical predicate (the ceiling of any learned alignment).
    for &p in &preds {
        kb.vocab.set_parents(p, Vec::new());
    }
    for &p in &preds {
        if let Some(g) = gold(kb.name(p)) {
            let c = kb.declare(&format!("oracle-{g}"), None, PredKind::Relation, false, &[]);
            kb.vocab.set_parents(p, vec![c]);
        }
    }
    let ores = evaluate(&kb, &metas, &represent(&kb, &metas), &queries);
    row(&mut md, "oracle alignment (hand map on raw corpus)", &format!("{alignable} gold pairs"), &ores);
    rows.push(json!({"round": "oracle", "methods": METHODS.iter().zip(&ores).map(|(m, r)| json!({"method": m, "r1": r.0, "r10": r.1, "mrr": r.2})).collect::<Vec<_>>()}));
    // Reference: the hand-mapped corpus (E12).
    let (rkb, rmetas) = load_corpus(&ref_dir)?;
    let rq = cross_lang_queries(&rmetas);
    let rres = evaluate(&rkb, &rmetas, &represent(&rkb, &rmetas), &rq);
    row(&mut md, "hand mapping (E12 corpus)", "—", &rres);
    rows.push(json!({"round": "reference", "methods": METHODS.iter().zip(&rres).map(|(m, r)| json!({"method": m, "r1": r.0, "r10": r.1, "mrr": r.2})).collect::<Vec<_>>()}));

    writeln!(md, "\n## Final alignment (round {rounds}), by evidence\n").unwrap();
    writeln!(md, "| Python | JavaScript | evidence | verdict |\n|---|---|---|---|").unwrap();
    for (a, b, w) in &final_pairs {
        let v = match verdict(kb.name(*a), kb.name(*b)) {
            Verdict::Correct => "correct",
            Verdict::Wrong => "**wrong**",
            Verdict::SameName => "same identifier",
            Verdict::Unverifiable => "unjudged",
        };
        writeln!(md, "| `{}` | `{}` | {w:.2} | {v} |", kb.name(*a), kb.name(*b)).unwrap();
    }
    let found: Vec<bool> = final_pairs.iter().map(|(a, b, _)| verdict(kb.name(*a), kb.name(*b)) == Verdict::Correct).collect();
    writeln!(md, "\nGold-pair recall: {} / {alignable}. Runtime {:.1?}.", found.iter().filter(|&&x| x).count(), t0.elapsed()).unwrap();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let tag = args.str("tag", &assoc);
    std::fs::write(format!("{out_dir}/E14-{tag}.md"), &md).map_err(|e| e.to_string())?;
    let fp_json: Vec<_> = final_pairs.iter().map(|(a, b, w)| json!({"py": kb.name(*a), "js": kb.name(*b), "evidence": w})).collect();
    std::fs::write(format!("{out_dir}/E14-{tag}.json"), serde_json::to_string_pretty(&json!({"rows": rows, "final_pairs": fp_json, "gold_pairs": alignable})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
