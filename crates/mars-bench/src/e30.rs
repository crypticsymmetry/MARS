//! E30: a memory that improves with use — online transfer reliability in the engine.
//!
//! The E27/E28 hold-out queries (an entity with all its facts of one relation
//! removed) arrive as a stream at a `mars-engine` memory of the other entities.
//! For each query the engine draws candidate inferences from its top analogues
//! (`Engine::infer`, first-order inferences on) and ranks the predicted objects
//! for the relation. A user then checks the top `feedback` suggestions, and
//! the outcomes go back into the engine (`Engine::feedback`), which learns the
//! reliability of each transfer type online. Compared with the same engine
//! ranking without learned reliability (Σ fused score of the proposing
//! analogues), over the course of the stream. The induced rules at the end of
//! the stream are reported.

use crate::e27::{build_queries, root_of, Query};
use crate::metrics::mean;
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Features, Profile};
use mars_engine::{Engine, EngineConfig, InfArg, SqMode};
use mars_hv::Rng;
use mars_map::MapConfig;
use mars_rel::{CaseId, Kb, Sym};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

/// Rank of the first gold object (None if absent) in a ranking by the given score.
fn first_hit(scores: &FxHashMap<Sym, f64>, q: &Query) -> Option<usize> {
    let mut v: Vec<(&Sym, &f64)> = scores.iter().collect();
    v.sort_by(|a, b| b.1.total_cmp(a.1).then(a.0.cmp(b.0)));
    v.iter().position(|(o, _)| q.gold.contains(o))
}

pub fn run(args: &Args) -> Result<(), String> {
    let dir = args.str("data", "data/kg-scientists/C2");
    let kg = args.str("kg", "wikidata");
    let rels_arg = args.str("relations", "wdt:p69,wdt:p108,wdt:p101,wdt:p27,wdt:p106,wdt:p166,wdt:p463,wdt:p1412");
    let per_rel = args.usize("per-rel", 300);
    let k = args.usize("k", 10);
    let fb_top = args.usize("feedback", 3);
    let bins = args.usize("bins", 8);
    let seed = args.u64("seed", 1);
    let out_dir = args.str("out", "results/E30");
    let tag = args.str("tag", "scientists-hop2");
    let t0 = Instant::now();

    let mut kb = Kb::new();
    kb.load_str(&std::fs::read_to_string(format!("{dir}/cases.mars")).map_err(|e| format!("{dir}/cases.mars: {e}"))?).map_err(|e| e.to_string())?;
    let manifest: Vec<serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(format!("{dir}/manifest.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let mem: Vec<CaseId> = manifest.iter().filter(|m| m["kg"].as_str() == Some(kg.as_str())).filter_map(|m| kb.case_by_name(m["case"].as_str()?)).collect();
    let roots: Vec<Sym> = mem.iter().map(|&c| root_of(&kb, c).unwrap_or(Sym(0))).collect();
    let rels: Vec<Sym> = rels_arg.split(',').map(|r| kb.sym(r)).collect();
    let queries = build_queries(&mut kb, &mem, &roots, &rels, per_rel, seed);
    let rel_names: Vec<String> = rels.iter().map(|&r| kb.name(r).to_string()).collect();

    // Engine over the memory (IDF epoch from the memory only); query cases retired.
    let cfg = EngineConfig { profile: Profile::surface_only(), fac_weight: 0.5, map: MapConfig { include_attributes: true, ..Default::default() }, first_order_inferences: true, infer_from: k, sq_mode: SqMode::Pipeline { mac_k: 50 }, ..Default::default() };
    let stats = {
        let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
        let raw: Vec<Features> = mem.par_iter().map(|&c| fx.extract(c)).collect();
        FeatureStats::fit(raw.iter())
    };
    let mut e = Engine::with_stats(kb, cfg, stats);
    for q in &queries {
        e.remove_case(q.case);
    }
    let mut order: Vec<usize> = (0..queries.len()).collect();
    Rng::new(seed ^ 0xE30).shuffle(&mut order);
    eprintln!("[e30] {} memory cases, {} queries in stream ({:.1?})", mem.len(), order.len(), t0.elapsed());

    // (with learned reliability, without) first-hit ranks per stream position.
    let mut ranks: Vec<[Option<usize>; 2]> = Vec::with_capacity(order.len());
    let mut n_feedback = 0usize;
    for &qi in &order {
        let q = &queries[qi];
        let r = rels[q.rel];
        let infs = e.infer(q.case, k, &[mem[q.orig]]);
        // Predictions for (r person ?y): the object, with learned and raw scores.
        let preds: Vec<(Sym, f64, f64, &str)> = infs
            .iter()
            .filter(|i| i.functor == r && i.args.len() == 2 && i.args[0] == InfArg::Entity(q.person))
            .filter_map(|i| match i.args[1] {
                InfArg::Entity(y) | InfArg::New(y) => Some((y, i.score, i.score / i.reliability.max(1e-9), i.text.as_str())),
                InfArg::Expr => None,
            })
            .collect();
        let (mut learned, mut raw): (FxHashMap<Sym, f64>, FxHashMap<Sym, f64>) = Default::default();
        for &(y, s, w, _) in &preds {
            *learned.entry(y).or_insert(0.0) += s;
            *raw.entry(y).or_insert(0.0) += w;
        }
        ranks.push([first_hit(&learned, q), first_hit(&raw, q)]);
        // The user checks the top suggestions (by the learned ranking).
        let mut shown: Vec<(Sym, f64, String)> = preds.iter().map(|&(y, s, _, t)| (y, s, t.to_string())).collect();
        shown.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.2.cmp(&b.2)));
        for (y, _, text) in shown.into_iter().take(fb_top) {
            e.feedback(q.case, &text, q.gold.contains(&y))?;
            n_feedback += 1;
        }
    }
    eprintln!("[e30] stream done ({:.1?})", t0.elapsed());

    let hit = |r: &Option<usize>, n: usize| r.map(|p| p < n) == Some(true);
    let rr = |r: &Option<usize>| r.map(|p| 1.0 / (p + 1) as f64).unwrap_or(0.0);
    let stat = |sel: &[[Option<usize>; 2]], m: usize| -> (f64, f64, f64) {
        (mean(&sel.iter().map(|x| hit(&x[m], 1) as u8 as f64).collect::<Vec<_>>()), mean(&sel.iter().map(|x| hit(&x[m], 10) as u8 as f64).collect::<Vec<_>>()), mean(&sel.iter().map(|x| rr(&x[m])).collect::<Vec<_>>()))
    };
    let mut md = String::new();
    writeln!(md, "# E30: online transfer reliability in the engine ({tag})\n").unwrap();
    writeln!(md, "Data: `{dir}` ({kg} side, {} memory cases); {} hold-out queries (relations: {}; up to {per_rel} per relation), streamed in a seeded random order (seed {seed}). Engine: surface profile, ½FAC + ½FP over the fingerprint top-50, first-order inferences from the top-{k} analogues (the entity's own full case excluded). After each query the top-{fb_top} suggestions are checked and fed back ({n_feedback} feedback events in total).\n", mem.len(), order.len(), rel_names.join(", ")).unwrap();
    let (a1, a10, am) = stat(&ranks, 0);
    let (b1, b10, bm) = stat(&ranks, 1);
    writeln!(md, "| ranking | Hits@1 | Hits@10 | MRR |\n|---|---|---|---|\n| raw (Σ fused score of proposing analogues) | {b1:.3} | {b10:.3} | {bm:.3} |\n| learned reliability × Σ fused (online feedback) | {a1:.3} | {a10:.3} | {am:.3} |\n").unwrap();
    writeln!(md, "**Learning curve** (Hits@1 per stream segment):\n\n| segment | queries | raw | learned | Δ |\n|---|---|---|---|---|").unwrap();
    let seg = order.len().div_ceil(bins);
    let mut curve = Vec::new();
    for (b, chunk) in ranks.chunks(seg).enumerate() {
        let (l, _, _) = stat(chunk, 0);
        let (w, _, _) = stat(chunk, 1);
        writeln!(md, "| {} | {}–{} | {w:.3} | {l:.3} | {:+.3} |", b + 1, b * seg + 1, b * seg + chunk.len(), l - w).unwrap();
        curve.push(json!({"segment": b + 1, "raw": w, "learned": l}));
    }
    let rules = e.induced_rules(20.0, 0.5);
    writeln!(md, "\n**Rules induced from analogy by the end of the stream** (transfer types with ≥ 20 feedback outcomes and precision ≥ 0.5; {} in total, top 20):\n\n| rule | precision | outcomes |\n|---|---|---|", rules.len()).unwrap();
    for (rule, p, n) in rules.iter().take(20) {
        writeln!(md, "| `{rule}` | {p:.3} | {n} |").unwrap();
    }
    let worst: Vec<(String, f64, f64)> = e.transfers.table(20.0).into_iter().rev().take(5).collect();
    writeln!(md, "\n**Least reliable transfer types** (≥ 20 outcomes):\n\n| type | precision | outcomes |\n|---|---|---|").unwrap();
    for (key, p, n) in &worst {
        writeln!(md, "| `{key}` | {p:.3} | {n} |").unwrap();
    }
    writeln!(md, "\nRuntime {:.1?}.", t0.elapsed()).unwrap();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E30-{tag}.md"), &md).map_err(|e| e.to_string())?;
    let cfgj = json!({"data": dir, "kg": kg, "relations": rel_names, "per_rel": per_rel, "k": k, "feedback_top": fb_top, "seed": seed, "queries": order.len(), "memory": mem.len(), "profile": "surface", "fac_weight": 0.5, "mac_k": 50});
    let j = json!({"config": cfgj, "raw": {"hits1": b1, "hits10": b10, "mrr": bm}, "learned": {"hits1": a1, "hits10": a10, "mrr": am}, "curve": curve, "feedback_events": n_feedback, "induced_rules": rules.iter().map(|(r, p, n)| json!({"rule": r, "precision": p, "outcomes": n})).collect::<Vec<_>>()});
    std::fs::write(format!("{out_dir}/E30-{tag}.json"), serde_json::to_string_pretty(&j).unwrap()).map_err(|e| e.to_string())?;
    print!("{md}");
    Ok(())
}
