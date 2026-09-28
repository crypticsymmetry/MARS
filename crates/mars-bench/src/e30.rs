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
//!
//! E31 (closing the loop): a third ranking adds the rules induced *so far*
//! (`Engine::rule_inferences`: transfer types with enough evidence applied
//! directly to the query), reaching objects no analogue proposes. `--feedback-from
//! combined` shows the user that ranking instead (default: the learned one,
//! so the first two rankings are unaffected by the third).
//!
//! E32: `--identity λ` adds the engine's identity channel (entity-overlap
//! TF-IDF) to retrieval.

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
    let rule_min_n = args.f64("rule-min-n", 20.0);
    let rule_min_p = args.f64("rule-min-p", 0.5);
    let fb_combined = args.str("feedback-from", "learned") == "combined";
    // E32: weight of the identity channel in retrieval (0 = off).
    let identity = args.f64("identity", 0.0);
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
    let cfg = EngineConfig { profile: Profile::surface_only(), fac_weight: 0.5, map: MapConfig { include_attributes: true, ..Default::default() }, first_order_inferences: true, infer_from: k, sq_mode: SqMode::Pipeline { mac_k: 50 }, identity_weight: identity, ..Default::default() };
    let stats = {
        let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
        let raw: Vec<Features> = mem.par_iter().map(|&c| fx.extract(c)).collect();
        FeatureStats::fit(raw.iter())
    };
    let ident_stats = (identity > 0.0).then(|| mars_engine::identity::IdentityIndex::fit(&kb, mem.iter().copied()));
    let mut e = Engine::with_epochs(kb, cfg, stats, ident_stats);
    for q in &queries {
        e.remove_case(q.case);
    }
    let mut order: Vec<usize> = (0..queries.len()).collect();
    Rng::new(seed ^ 0xE30).shuffle(&mut order);
    eprintln!("[e30] {} memory cases, {} queries in stream ({:.1?})", mem.len(), order.len(), t0.elapsed());

    // First-hit ranks per stream position: [learned, raw, learned + induced rules].
    let mut ranks: Vec<[Option<usize>; 3]> = Vec::with_capacity(order.len());
    let (mut rule_only_hits, mut n_rules_fired) = (0usize, 0usize);
    // Is some gold object present in the query case (reachable by substitution or a rule)?
    let mut in_query: Vec<bool> = Vec::with_capacity(order.len());
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
                InfArg::Entity(y) | InfArg::New(y) => Some((y, i.score, i.weight, i.text.as_str())),
                InfArg::Expr => None,
            })
            .collect();
        let (mut learned, mut raw): (FxHashMap<Sym, f64>, FxHashMap<Sym, f64>) = Default::default();
        for &(y, s, w, _) in &preds {
            *learned.entry(y).or_insert(0.0) += s;
            *raw.entry(y).or_insert(0.0) += w;
        }
        // Induced rules so far, applied directly; combined = learned vote share + rule reliability.
        let rules: Vec<(Sym, f64, String)> = e
            .rule_inferences(q.case, rule_min_n, rule_min_p)
            .into_iter()
            .filter(|i| i.functor == r && i.args.len() == 2 && i.args[0] == InfArg::Entity(q.person))
            .filter_map(|i| if let InfArg::Entity(y) = i.args[1] { Some((y, i.reliability, i.text)) } else { None })
            .collect();
        n_rules_fired += !rules.is_empty() as usize;
        let tot: f64 = learned.values().sum::<f64>().max(1e-9);
        let mut combined: FxHashMap<Sym, f64> = learned.iter().map(|(&y, &s)| (y, s / tot)).collect();
        let mut text_of: FxHashMap<Sym, String> = preds.iter().map(|&(y, _, _, t)| (y, t.to_string())).collect();
        for (y, rel, text) in &rules {
            let best = rules.iter().filter(|x| x.0 == *y).map(|x| x.1).fold(0.0, f64::max);
            if *rel == best {
                *combined.entry(*y).or_insert(0.0) += best;
                text_of.entry(*y).or_insert_with(|| text.clone());
            }
        }
        let (h_learned, h_comb) = (first_hit(&learned, q), first_hit(&combined, q));
        if h_comb == Some(0) && !learned.keys().any(|y| q.gold.contains(y)) {
            rule_only_hits += 1;
        }
        ranks.push([h_learned, first_hit(&raw, q), h_comb]);
        in_query.push(e.kb.case_entities(q.case).iter().any(|x| q.gold.contains(x)));
        // The user checks the top suggestions (by the learned or the combined ranking).
        let mut shown: Vec<(Sym, f64, String)> = if fb_combined {
            combined.iter().map(|(&y, &s)| (y, s, text_of[&y].clone())).collect()
        } else {
            preds.iter().map(|&(y, s, _, t)| (y, s, t.to_string())).collect()
        };
        shown.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.2.cmp(&b.2)));
        for (y, _, text) in shown.into_iter().take(fb_top) {
            e.feedback(q.case, &text, q.gold.contains(&y))?;
            n_feedback += 1;
        }
    }
    eprintln!("[e30] stream done ({:.1?})", t0.elapsed());

    let hit = |r: &Option<usize>, n: usize| r.map(|p| p < n) == Some(true);
    let rr = |r: &Option<usize>| r.map(|p| 1.0 / (p + 1) as f64).unwrap_or(0.0);
    let stat = |sel: &[[Option<usize>; 3]], m: usize| -> (f64, f64, f64) {
        (mean(&sel.iter().map(|x| hit(&x[m], 1) as u8 as f64).collect::<Vec<_>>()), mean(&sel.iter().map(|x| hit(&x[m], 10) as u8 as f64).collect::<Vec<_>>()), mean(&sel.iter().map(|x| rr(&x[m])).collect::<Vec<_>>()))
    };
    let mut md = String::new();
    writeln!(md, "# E30: online transfer reliability in the engine ({tag})\n").unwrap();
    writeln!(md, "Data: `{dir}` ({kg} side, {} memory cases); {} hold-out queries (relations: {}; up to {per_rel} per relation), streamed in a seeded random order (seed {seed}). Engine: surface profile, ½FAC + ½FP over the fingerprint top-50 (identity channel weight {identity}), first-order inferences from the top-{k} analogues (the entity's own full case excluded). After each query the top-{fb_top} suggestions are checked and fed back ({n_feedback} feedback events in total).\n", mem.len(), order.len(), rel_names.join(", ")).unwrap();
    let (a1, a10, am) = stat(&ranks, 0);
    let (b1, b10, bm) = stat(&ranks, 1);
    let (c1, c10, cm) = stat(&ranks, 2);
    writeln!(md, "| ranking | Hits@1 | Hits@10 | MRR |\n|---|---|---|---|\n| raw (Σ fused score of proposing analogues) | {b1:.3} | {b10:.3} | {bm:.3} |\n| learned reliability × Σ fused (online feedback) | {a1:.3} | {a10:.3} | {am:.3} |\n| learned + rules induced so far (E31) | {c1:.3} | {c10:.3} | {cm:.3} |\n").unwrap();
    writeln!(md, "Induced rules (≥ {rule_min_n} outcomes, precision ≥ {rule_min_p}) fired on {n_rules_fired} queries; {rule_only_hits} queries were answered correctly at rank 1 only thanks to them (no analogue proposed a correct object). Feedback shown from the {} ranking.\n", if fb_combined { "combined" } else { "learned" }).unwrap();
    let split = |want: bool| -> Vec<[Option<usize>; 3]> { ranks.iter().zip(&in_query).filter(|x| *x.1 == want).map(|x| *x.0).collect() };
    let (inq, outq) = (split(true), split(false));
    writeln!(md, "**Where the answer is** (Hits@1 raw / learned / learned + rules):\n\n| queries | n | raw | learned | learned + rules |\n|---|---|---|---|---|").unwrap();
    for (name, sel) in [("a correct object is already an entity of the query (substitution / rule reachable)", &inq), ("no correct object in the query (only copying from analogues can reach it)", &outq)] {
        writeln!(md, "| {name} | {} | {:.3} | {:.3} | {:.3} |", sel.len(), stat(sel, 1).0, stat(sel, 0).0, stat(sel, 2).0).unwrap();
    }
    writeln!(md).unwrap();
    writeln!(md, "**Learning curve** (Hits@1 per stream segment):\n\n| segment | queries | raw | learned | Δ | learned + rules |\n|---|---|---|---|---|---|").unwrap();
    let seg = order.len().div_ceil(bins);
    let mut curve = Vec::new();
    for (b, chunk) in ranks.chunks(seg).enumerate() {
        let (l, _, _) = stat(chunk, 0);
        let (w, _, _) = stat(chunk, 1);
        let (c, _, _) = stat(chunk, 2);
        writeln!(md, "| {} | {}–{} | {w:.3} | {l:.3} | {:+.3} | {c:.3} |", b + 1, b * seg + 1, b * seg + chunk.len(), l - w).unwrap();
        curve.push(json!({"segment": b + 1, "raw": w, "learned": l, "learned_rules": c}));
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
    let cfgj = json!({"data": dir, "kg": kg, "relations": rel_names, "per_rel": per_rel, "k": k, "feedback_top": fb_top, "seed": seed, "queries": order.len(), "memory": mem.len(), "profile": "surface", "fac_weight": 0.5, "mac_k": 50, "identity_weight": identity, "rule_min_n": rule_min_n, "rule_min_p": rule_min_p, "feedback_from": if fb_combined { "combined" } else { "learned" }});
    // Per stream position: query index, first-hit rank [learned, raw, learned + rules] (null = miss), answer in query.
    let per_query: Vec<serde_json::Value> = order.iter().zip(&ranks).zip(&in_query).map(|((qi, r), iq)| json!({"q": qi, "rel": rel_names[queries[*qi].rel], "hit": r, "in_query": iq})).collect();
    let j = json!({"per_query": per_query, "config": cfgj, "raw": {"hits1": b1, "hits10": b10, "mrr": bm}, "learned": {"hits1": a1, "hits10": a10, "mrr": am}, "learned_rules": {"hits1": c1, "hits10": c10, "mrr": cm, "rule_only_top1": rule_only_hits, "queries_with_rules": n_rules_fired}, "in_query": {"n": inq.len(), "raw": stat(&inq, 1).0, "learned": stat(&inq, 0).0, "learned_rules": stat(&inq, 2).0}, "not_in_query": {"n": outq.len(), "raw": stat(&outq, 1).0, "learned": stat(&outq, 0).0, "learned_rules": stat(&outq, 2).0}, "curve": curve, "feedback_events": n_feedback, "induced_rules": rules.iter().map(|(r, p, n)| json!({"rule": r, "precision": p, "outcomes": n})).collect::<Vec<_>>()});
    std::fs::write(format!("{out_dir}/E30-{tag}.json"), serde_json::to_string_pretty(&j).unwrap()).map_err(|e| e.to_string())?;
    print!("{md}");
    Ok(())
}
