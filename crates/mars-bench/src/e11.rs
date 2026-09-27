//! E11: inference quality and calibration with corroboration.
//!
//! Memory: M noisy instances of each of T hidden templates. Query: a fresh
//! instance with one root higher-order fact deleted. Candidate inferences
//! are projected from each of the top-m analogues (engine retrieval); an
//! inference's *support* is the number of analogues proposing it. Truth: the
//! inference is a fact of the query's undeleted instance.

use crate::metrics::mean;
use crate::Args;
use mars_engine::{Engine, EngineConfig};
use mars_gen::{template_instances, GenConfig, Naming, PerturbOp};
use mars_map::{Grounding, MapConfig, Mapper};
use mars_rel::{CaseId, CaseKind, ExprId, Term};
use rustc_hash::FxHashMap;
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

pub fn run(args: &Args) -> Result<(), String> {
    let seed = args.u64("seed", 1);
    let n_t = args.usize("templates", 100);
    let per = args.usize("per", 10);
    let n_q = args.usize("queries-per-template", 5);
    let sev = args.usize("severity", 1);
    let out_dir = args.str("out", "results/E11");
    let t0 = Instant::now();
    let cfg = GenConfig { seed, naming: Naming::Canonical, distractors: 2, perturb_ops: PerturbOp::ALL.to_vec(), severity: sev, ..Default::default() };
    let inst = template_instances(&cfg, n_t, per + n_q);
    let mut kb = inst.kb;
    let mut seen: FxHashMap<usize, usize> = FxHashMap::default();
    let mut queries: Vec<(CaseId, CaseId, String)> = Vec::new(); // (query, full instance, deleted fact text)
    let mut drop_after: Vec<CaseId> = Vec::new();
    for &(c, t, clean) in &inst.cases {
        if clean {
            drop_after.push(c); // clean prototypes are not memory
            continue;
        }
        let s = seen.entry(t).or_insert(0);
        *s += 1;
        if *s <= per {
            continue;
        }
        drop_after.push(c);
        let nested: std::collections::HashSet<ExprId> =
            kb.case_exprs(c).into_iter().flat_map(|e| kb.expr(e).args.iter().filter_map(|a| if let Term::Expr(x) = *a { Some(x) } else { None }).collect::<Vec<_>>()).collect();
        let Some(del) = kb.case(c).facts.iter().copied().find(|&f| kb.order(f) >= 2 && !nested.contains(&f)) else { continue };
        let text = kb.render_expr(del);
        let kept: Vec<ExprId> = kb.case(c).facts.iter().copied().filter(|&f| f != del).collect();
        let name = format!("{}-q", kb.name(kb.case(c).name));
        let q = kb.add_case(&name, CaseKind::Query, kept);
        queries.push((q, c, text));
        drop_after.push(q);
    }
    let mut e = Engine::new(kb, EngineConfig::default());
    for c in drop_after {
        e.remove_case(c);
    }
    eprintln!("[e11] memory {} cases, {} queries ({:.1?})", e.n_live(), queries.len(), t0.elapsed());

    let m_max = 5usize;
    // Per query: per analogue rank, the set of inference texts it proposes.
    let mut per_query: Vec<(Vec<Vec<String>>, std::collections::HashSet<String>, String)> = Vec::new();
    for (q, full, deleted) in &queries {
        let analogues = e.query(*q, m_max);
        let mp = Mapper::new(&e.kb, MapConfig::default());
        let props: Vec<Vec<String>> = analogues
            .iter()
            .map(|&(a, _)| {
                let mut v: Vec<String> = mp
                    .best(a, *q)
                    .map(|m| m.inferences.iter().filter(|i| i.grounding == Grounding::Structural && !i.has_skolem).map(|i| mp.render_proj(&i.projected)).collect())
                    .unwrap_or_default();
                v.sort();
                v.dedup();
                v
            })
            .collect();
        let truth: std::collections::HashSet<String> = e.kb.case(*full).facts.iter().map(|&f| e.kb.render_expr(f)).collect();
        per_query.push((props, truth, deleted.clone()));
    }

    let mut md = String::new();
    writeln!(md, "# E11: inference quality and calibration with corroboration\n").unwrap();
    writeln!(md, "{n_t} hidden templates × {per} noisy instances in memory (perturbation severity {sev}, 2 distractors); {} queries = fresh instances with one root higher-order fact deleted. Inferences are projected from each of the top-m analogues (fused retrieval); *support* = number of analogues proposing the inference. An inference is *true* if it is a fact of the query's undeleted instance.\n", queries.len()).unwrap();
    writeln!(md, "| analogues m | min support | deleted-fact recall | precision | inferences / query |").unwrap();
    writeln!(md, "|---|---|---|---|---|").unwrap();
    let mut rows = Vec::new();
    for m in [1usize, 3, 5] {
        for min_sup in 1..=m.min(3) {
            let (mut rec, mut prec_num, mut prec_den, mut per_q) = (Vec::new(), 0usize, 0usize, Vec::new());
            for (props, truth, deleted) in &per_query {
                let mut counts: FxHashMap<&str, usize> = FxHashMap::default();
                for p in props.iter().take(m) {
                    for t in p {
                        *counts.entry(t.as_str()).or_insert(0) += 1;
                    }
                }
                let accepted: Vec<&str> = counts.iter().filter(|x| *x.1 >= min_sup).map(|x| *x.0).collect();
                rec.push(accepted.contains(&deleted.as_str()) as u8 as f64);
                prec_den += accepted.len();
                prec_num += accepted.iter().filter(|t| truth.contains(**t)).count();
                per_q.push(accepted.len() as f64);
            }
            let prec = prec_num as f64 / prec_den.max(1) as f64;
            writeln!(md, "| {m} | {min_sup} | {:.3} | {prec:.3} | {:.2} |", mean(&rec), mean(&per_q)).unwrap();
            rows.push(json!({"m": m, "min_support": min_sup, "recall": mean(&rec), "precision": prec, "per_query": mean(&per_q)}));
        }
    }
    // Calibration at m = 5: precision by exact support.
    writeln!(md, "\n## Calibration (m = {m_max}): precision by support count\n").unwrap();
    writeln!(md, "| support | inferences | precision |").unwrap();
    writeln!(md, "|---|---|---|").unwrap();
    let mut by_sup: FxHashMap<usize, (usize, usize)> = FxHashMap::default();
    for (props, truth, _) in &per_query {
        let mut counts: FxHashMap<&str, usize> = FxHashMap::default();
        for p in props {
            for t in p {
                *counts.entry(t.as_str()).or_insert(0) += 1;
            }
        }
        for (t, c) in counts {
            let e2 = by_sup.entry(c).or_insert((0, 0));
            e2.0 += 1;
            e2.1 += truth.contains(t) as usize;
        }
    }
    let mut cal = Vec::new();
    for s in 1..=m_max {
        if let Some(&(n, ok)) = by_sup.get(&s) {
            writeln!(md, "| {s} | {n} | {:.3} |", ok as f64 / n as f64).unwrap();
            cal.push(json!({"support": s, "n": n, "precision": ok as f64 / n as f64}));
        }
    }
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let tag = args.str("tag", &format!("s{sev}-m{per}"));
    std::fs::write(format!("{out_dir}/E11-{tag}.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E11-{tag}.json"), serde_json::to_string_pretty(&json!({"rows": rows, "calibration": cal})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
