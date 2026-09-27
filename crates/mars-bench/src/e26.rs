//! E26: analogical vocabulary alignment on real knowledge graphs
//! (DBpedia ↔ Wikidata films).
//!
//! Each film is a case in each KG: its outgoing triples with the KG's own,
//! unresolved property names (`dbo:director`, `wdt:p57`). The E13/E14 loop —
//! other-KG neighbours (fingerprints) → wildcard mappings → mutual-best
//! property pairs, one per KG → canonical clusters, re-estimated each round —
//! learns property correspondences without labels. Ground truth: DBpedia's
//! owl:equivalentProperty links to Wikidata (gold.json), used only to score.
//! Anchoring conditions (tools/kg2mars.py): A structure only; B shared literal
//! values; C values + entity labels. Also measured: retrieval of a film's
//! counterpart in the other KG before and after alignment.

use crate::metrics::{mrr, recall_at};
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, N_CHANNELS};
use mars_map::{MapConfig, Mapper};
use mars_rel::{CaseId, Kb, PredKind, Sym, Term};
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

fn kg_of(name: &str) -> Option<&str> {
    name.split_once(':').map(|x| x.0).filter(|k| *k == "dbo" || *k == "wdt")
}

fn local(name: &str) -> String {
    name.split_once(':').map(|x| x.1).unwrap_or(name).to_lowercase()
}

#[derive(Clone, Copy, PartialEq)]
enum Verdict {
    Correct,
    Wrong,
    Unjudged,
}

struct Meta {
    case: CaseId,
    dbpedia: bool,
    film: usize,
}

fn map_cfg(wildcard: Option<f32>) -> MapConfig {
    MapConfig { include_attributes: true, wildcard, ..Default::default() }
}

/// Counterpart retrieval: each DBpedia case ranks the Wikidata cases; (R@1, R@10, MRR) for FP and fused.
fn retrieval(kb: &Kb, metas: &[Meta], k: usize) -> [(f64, f64, f64); 2] {
    let fx = FeatureExtractor::new(kb, FeatureConfig::default());
    let mut feats: Vec<Features> = metas.par_iter().map(|m| fx.extract(m.case)).collect();
    let stats = FeatureStats::fit(feats.iter());
    feats.par_iter_mut().for_each(|f| stats.apply(f, &[true; N_CHANNELS]));
    let sk = Sketcher::new(Layout::default(), 0xF1);
    let fps: Vec<_> = feats.par_iter().map(|f| sk.sketch(f)).collect();
    let mapper = Mapper::new(kb, map_cfg(None));
    let selfs: Vec<f64> = metas.par_iter().map(|m| mapper.score(m.case, m.case) as f64).collect();
    let lit = Profile::literal();
    let wd: Vec<usize> = (0..metas.len()).filter(|&i| !metas[i].dbpedia).collect();
    let ranks: Vec<[usize; 2]> = (0..metas.len())
        .into_par_iter()
        .filter(|&i| metas[i].dbpedia)
        .filter_map(|i| {
            let target = wd.iter().copied().find(|&j| metas[j].film == metas[i].film)?;
            let mut s: Vec<(usize, f64)> = wd.iter().map(|&j| (j, lit.score(&sk.channel_sims(&fps[i], &fps[j])))).collect();
            s.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let r_fp = s.iter().position(|x| x.0 == target).unwrap();
            let mut f: Vec<(usize, f64)> = s
                .iter()
                .take(k)
                .map(|&(j, fp)| {
                    let raw = mapper.score(metas[j].case, metas[i].case) as f64;
                    let fac = if raw == 0.0 { 0.0 } else { (raw / (selfs[i] * selfs[j]).sqrt()).min(1.0) };
                    (j, 0.5 * fac + 0.5 * fp)
                })
                .collect();
            f.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let r_fu = f.iter().position(|x| x.0 == target).unwrap_or(usize::MAX / 2);
            Some([r_fp, r_fu])
        })
        .collect();
    std::array::from_fn(|m| {
        let r: Vec<usize> = ranks.iter().map(|x| x[m]).collect();
        (recall_at(&r, 1), recall_at(&r, 10), mrr(&r))
    })
}

pub fn run(args: &Args) -> Result<(), String> {
    let dir = args.str("data", "data/kg-films/C");
    let rounds = args.usize("rounds", 3);
    let neighbours = args.usize("neighbours", 5);
    let theta = args.f64("theta", 0.2);
    let wildcard = args.f64("wildcard", 0.3) as f32;
    let min_evidence = args.f64("min-evidence", 1.0);
    let out_dir = args.str("out", "results/E26");
    let tag = args.str("tag", "run");
    let t0 = Instant::now();
    let mut kb = Kb::new();
    kb.load_str(&std::fs::read_to_string(format!("{dir}/cases.mars")).map_err(|e| format!("{dir}/cases.mars: {e}"))?).map_err(|e| e.to_string())?;
    let manifest: Vec<serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(format!("{dir}/manifest.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let metas: Vec<Meta> = manifest.iter().filter_map(|m| Some(Meta { case: kb.case_by_name(m["case"].as_str()?)?, dbpedia: m["kg"].as_str()? == "dbpedia", film: m["film"].as_u64()? as usize })).collect();
    let gold_raw: Vec<(String, String)> = serde_json::from_str(&std::fs::read_to_string(format!("{dir}/gold.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let gold: FxHashSet<(String, String)> = gold_raw.iter().map(|(a, b)| (a.to_lowercase(), b.to_lowercase())).collect();
    let gold_d: FxHashSet<&str> = gold.iter().map(|x| x.0.as_str()).collect();
    let gold_w: FxHashSet<&str> = gold.iter().map(|x| x.1.as_str()).collect();
    let n = metas.len();
    // Property usage counts (to report the vocabulary and the gold pairs present in the data).
    let mut uses: FxHashMap<Sym, usize> = FxHashMap::default();
    for m in &metas {
        for &f in &kb.case(m.case).facts {
            let p = kb.expr(f).functor;
            if kg_of(kb.name(p)).is_some() {
                *uses.entry(p).or_insert(0) += 1;
            }
        }
    }
    let mut preds: Vec<Sym> = uses.keys().copied().collect();
    preds.sort_by(|a, b| kb.name(*a).cmp(kb.name(*b)));
    let present: FxHashSet<String> = preds.iter().map(|&p| local(kb.name(p))).collect();
    let gold_present: Vec<&(String, String)> = gold.iter().filter(|(a, b)| present.contains(a) && present.contains(b)).collect();
    let verdict = |kb: &Kb, a: Sym, b: Sym| {
        let (x, y) = (local(kb.name(a)), local(kb.name(b)));
        if gold.contains(&(x.clone(), y.clone())) {
            Verdict::Correct
        } else if gold_d.contains(x.as_str()) || gold_w.contains(y.as_str()) {
            Verdict::Wrong
        } else {
            Verdict::Unjudged
        }
    };
    let n_d = preds.iter().filter(|&&p| kg_of(kb.name(p)) == Some("dbo")).count();
    eprintln!("[e26] {n} cases, {} properties ({n_d} dbo), {} gold pairs present ({:.1?})", preds.len(), gold_present.len(), t0.elapsed());

    let mut md = String::new();
    writeln!(md, "# E26: knowledge-graph vocabulary alignment ({tag})\n").unwrap();
    writeln!(md, "Data: `{dir}` — {n} film cases, {} properties ({n_d} DBpedia, {} Wikidata); {} gold property pairs occur in the data. Loop: {neighbours} other-KG neighbours per case (fingerprint, literal profile), wildcard mapping (local score {wildcard}, attributes included), evidence from mappings with normalized score ≥ {theta}, mutual-best pairs (evidence ≥ {min_evidence}), one property per KG per cluster, re-estimated each round. Counterpart retrieval: each DBpedia film ranks the Wikidata films (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).\n", preds.len(), preds.len() - n_d, gold_present.len()).unwrap();
    writeln!(md, "| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |\n|---|---|---|---|---|---|---|").unwrap();
    let mut aligned: Vec<(Sym, Sym, f64)> = Vec::new();
    let mut rows = Vec::new();
    for round in 0..=rounds {
        let tr = Instant::now();
        let ret = retrieval(&kb, &metas, 50);
        let vs: Vec<Verdict> = aligned.iter().map(|(a, b, _)| verdict(&kb, *a, *b)).collect();
        let c = |v: Verdict| vs.iter().filter(|&&x| x == v).count();
        let (ok, bad, unj) = (c(Verdict::Correct), c(Verdict::Wrong), c(Verdict::Unjudged));
        let prec = if ok + bad == 0 { f64::NAN } else { ok as f64 / (ok + bad) as f64 };
        let rec = ok as f64 / gold_present.len().max(1) as f64;
        writeln!(md, "| {round} | {} | {ok} / {bad} / {unj} | {prec:.3} | {rec:.3} | {:.3} / {:.3} | {:.3} / {:.3} |", aligned.len(), ret[0].0, ret[1].0, ret[0].2, ret[1].2).unwrap();
        rows.push(json!({"round": round, "aligned": aligned.len(), "correct": ok, "wrong": bad, "unjudged": unj, "precision": prec, "gold_recall": rec, "fp": {"r1": ret[0].0, "r10": ret[0].1, "mrr": ret[0].2}, "fused": {"r1": ret[1].0, "r10": ret[1].1, "mrr": ret[1].2}}));
        eprintln!("[e26] round {round}: {} aligned ({ok}/{bad}/{unj}), counterpart R@1 {:.3}/{:.3} ({:.1?})", aligned.len(), ret[0].0, ret[1].0, tr.elapsed());
        if round == rounds {
            break;
        }
        // 1. Other-KG neighbours by fingerprint.
        let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
        let mut feats: Vec<Features> = metas.par_iter().map(|m| fx.extract(m.case)).collect();
        let stats = FeatureStats::fit(feats.iter());
        feats.par_iter_mut().for_each(|f| stats.apply(f, &[true; N_CHANNELS]));
        let sk = Sketcher::new(Layout::default(), 0xF1);
        let fps: Vec<_> = feats.par_iter().map(|f| sk.sketch(f)).collect();
        let lit = Profile::literal();
        let pairs: Vec<(usize, usize)> = (0..n)
            .into_par_iter()
            .flat_map_iter(|i| {
                let mut s: Vec<(usize, f64)> = (0..n).filter(|&j| metas[j].dbpedia != metas[i].dbpedia).map(|j| (j, lit.score(&sk.channel_sims(&fps[i], &fps[j])))).collect();
                s.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                s.into_iter().take(neighbours).map(move |(j, _)| (i, j))
            })
            .collect();
        // 2–3. Wildcard mappings -> property correspondence evidence.
        let wm = Mapper::new(&kb, map_cfg(Some(wildcard)));
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
                                let (kbk, ktk) = (kg_of(kbr.name(fb)), kg_of(kbr.name(ft)));
                                if kbk.is_some() && ktk.is_some() && kbk != ktk {
                                    let key = if kbk == Some("dbo") { (fb, ft) } else { (ft, fb) };
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
        // 4. Mutual best (dbo, wdt) pairs.
        let mut best: FxHashMap<Sym, (Sym, f64)> = FxHashMap::default();
        for (&(a, b), &w) in &evidence {
            for (x, y) in [(a, b), (b, a)] {
                let e = best.entry(x).or_insert((y, 0.0));
                if w > e.1 || (w == e.1 && y < e.0) {
                    *e = (y, w);
                }
            }
        }
        let mut merges: Vec<(Sym, Sym, f64)> = evidence.iter().filter(|(&(a, b), &w)| w >= min_evidence && best[&a].0 == b && best[&b].0 == a).map(|(&(a, b), &w)| (a, b, w)).collect();
        merges.sort_by(|x, y| y.2.total_cmp(&x.2).then(x.0.cmp(&y.0)));
        eprintln!("[e26] round {round}: {} pairs mapped, {} evidence pairs, {} mutual-best", pairs.len(), evidence.len(), merges.len());
        // 5. Re-estimate and materialize.
        for &p in &preds {
            kb.vocab.set_parents(p, Vec::new());
        }
        aligned.clear();
        for (a, b, w) in merges {
            if aligned.iter().any(|x| x.0 == a || x.1 == b) {
                continue;
            }
            let c = kb.declare(&format!("aligned-{}", kb.name(a).replace(':', "-")), Some(2), PredKind::Relation, false, &[]);
            kb.vocab.set_parents(a, vec![c]);
            kb.vocab.set_parents(b, vec![c]);
            aligned.push((a, b, w));
        }
    }
    writeln!(md, "\n## Final alignment (top 40 by evidence)\n\n| DBpedia | Wikidata | evidence | verdict |\n|---|---|---|---|").unwrap();
    let mut fin = aligned.clone();
    fin.sort_by(|x, y| y.2.total_cmp(&x.2));
    for (a, b, w) in fin.iter().take(40) {
        let v = match verdict(&kb, *a, *b) {
            Verdict::Correct => "correct",
            Verdict::Wrong => "**wrong**",
            Verdict::Unjudged => "not in gold",
        };
        writeln!(md, "| `{}` | `{}` | {w:.1} | {v} |", kb.name(*a), kb.name(*b)).unwrap();
    }
    let missed: Vec<String> = gold_present.iter().filter(|(a, b)| !aligned.iter().any(|(x, y, _)| local(kb.name(*x)) == *a && local(kb.name(*y)) == *b)).map(|(a, b)| format!("{a}↔{b}")).collect();
    writeln!(md, "\nGold pairs present but not learned ({}): {}\n\nRuntime {:.1?}.", missed.len(), missed.join(", "), t0.elapsed()).unwrap();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E26-{tag}.md"), &md).map_err(|e| e.to_string())?;
    let fj: Vec<_> = fin.iter().map(|(a, b, w)| json!({"dbpedia": kb.name(*a), "wikidata": kb.name(*b), "evidence": w})).collect();
    std::fs::write(format!("{out_dir}/E26-{tag}.json"), serde_json::to_string_pretty(&json!({"data": dir, "rows": rows, "final": fj, "gold_present": gold_present.len()})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
