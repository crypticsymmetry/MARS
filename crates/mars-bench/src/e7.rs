//! E7: consolidation (SAGE-style generalization) — template recovery and
//! few-shot inference from schemas vs single instances (H4).

use crate::metrics::mean;
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Layout, Sketcher};
use mars_engine::sage::{Sage, SageConfig};
use mars_gen::{template_instances, GenConfig, Naming, PerturbOp};
use mars_hv::Rng;
use mars_map::{Grounding, MapConfig, Mapper};
use mars_rel::{CaseId, CaseKind, ExprId, Kb, Term};
use rustc_hash::FxHashMap;
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

fn stats_for(kb: &Kb, cases: &[CaseId]) -> FeatureStats {
    let fx = FeatureExtractor::new(kb, FeatureConfig::default());
    let feats: Vec<_> = cases.iter().map(|&c| fx.extract(c)).collect();
    FeatureStats::fit(feats.iter())
}

/// Fraction of `a`'s structural facts matched when mapping a → b.
fn fact_recall(kb: &Kb, a: CaseId, b: CaseId) -> f64 {
    let m = Mapper::new(kb, MapConfig::default()).best(a, b);
    let facts: Vec<ExprId> = kb.case(a).facts.iter().copied().filter(|&f| kb.vocab.kind(kb.expr(f).functor) != mars_rel::PredKind::Attribute).collect();
    if facts.is_empty() {
        return 0.0;
    }
    let Some(m) = m else { return 0.0 };
    let mapped: std::collections::HashSet<Term> = m.correspondences.iter().map(|x| x.0).collect();
    facts.iter().filter(|&&f| mapped.contains(&Term::Expr(f))).count() as f64 / facts.len() as f64
}

fn root_ho_fact(kb: &Kb, c: CaseId) -> Option<ExprId> {
    let nested: std::collections::HashSet<ExprId> =
        kb.case_exprs(c).into_iter().flat_map(|e| kb.expr(e).args.iter().filter_map(|a| if let Term::Expr(x) = *a { Some(x) } else { None }).collect::<Vec<_>>()).collect();
    kb.case(c).facts.iter().copied().find(|&f| kb.order(f) >= 2 && !nested.contains(&f))
}

pub fn run(args: &Args) -> Result<(), String> {
    let seed = args.u64("seed", 1);
    let n_t = args.usize("templates", 50);
    let sev = args.usize("severity", 1);
    let out_dir = args.str("out", "results/E7");
    let t0 = Instant::now();
    let mut md = String::new();
    let mut j = serde_json::Map::new();
    writeln!(md, "# E7: consolidation — SAGE-style generalization over MARS\n").unwrap();
    let gcfg = |distractors: usize| GenConfig { seed, naming: Naming::Canonical, distractors, perturb_ops: PerturbOp::ALL.to_vec(), severity: sev, ..Default::default() };

    // ------------------------------------------------------------ E7a
    let thresholds: Vec<f64> = args.str("assim-sweep", "0.3,0.4,0.5,0.6").split(',').map(|x| x.parse().unwrap()).collect();
    let mut e7a_rows = Vec::new();
    for (ti, &assim) in thresholds.iter().enumerate() {
        let per = args.usize("per", 40);
        let inst = template_instances(&gcfg(2), n_t, per);
        let mut kb = inst.kb;
        let mut order: Vec<(CaseId, usize)> = inst.cases.iter().filter(|x| !x.2).map(|x| (x.0, x.1)).collect();
        let clean: FxHashMap<usize, CaseId> = inst.cases.iter().filter(|x| x.2).map(|x| (x.1, x.0)).collect();
        Rng::new(seed ^ 0xE7).shuffle(&mut order);
        let tmpl_of: FxHashMap<CaseId, usize> = order.iter().copied().collect();
        let all: Vec<CaseId> = order.iter().map(|x| x.0).collect();
        let stats = stats_for(&kb, &all);
        let mut sage = Sage::new(SageConfig { assimilate: assim, ..Default::default() }, stats, Sketcher::new(Layout::default(), seed ^ 0xF1), FeatureConfig::default());
        let ts = Instant::now();
        for &(c, _) in &order {
            sage.add(&mut kb, c);
        }
        let add_ms = ts.elapsed().as_secs_f64() * 1e3 / order.len() as f64;
        let (n_gens_online, n_out_online) = (sage.gens.len(), sage.outliers.len());
        let tc = Instant::now();
        let merge_t = args.f64("merge", 0.6);
        let (absorbed, merged) = if args.str("sleep", "yes") == "yes" { sage.consolidate(&mut kb, merge_t) } else { (0, 0) };
        let sleep_s = tc.elapsed().as_secs_f64();
        let (mut pure, mut total) = (0usize, 0usize);
        let mut dominant: FxHashMap<usize, (usize, usize)> = FxHashMap::default(); // template -> (gen idx, count)
        let mut gen_major = Vec::new();
        for (gi, g) in sage.gens.iter().enumerate() {
            let mut counts: FxHashMap<usize, usize> = FxHashMap::default();
            for m in &g.members {
                *counts.entry(tmpl_of[m]).or_insert(0) += 1;
            }
            let (&maj, &cnt) = counts.iter().max_by_key(|x| (*x.1, std::cmp::Reverse(*x.0))).unwrap();
            pure += cnt;
            total += g.members.len();
            gen_major.push((gi, maj, g.members.len()));
            let e = dominant.entry(maj).or_insert((gi, 0));
            if cnt > e.1 {
                *e = (gi, cnt);
            }
        }
        let purity = pure as f64 / total.max(1) as f64;
        let coverage = total as f64 / order.len() as f64;
        let completeness = mean(&(0..n_t).map(|t| dominant.get(&t).map(|x| x.1 as f64 / per as f64).unwrap_or(0.0)).collect::<Vec<_>>());
        // Schema fidelity vs never-seen clean template: fact recall both ways.
        let mut schema_rec = Vec::new();
        let mut schema_prec = Vec::new();
        let mut member_rec = Vec::new();
        let mut member_prec = Vec::new();
        for &(gi, maj, size) in &gen_major {
            if size < 5 {
                continue;
            }
            let g = &sage.gens[gi];
            let cl = clean[&maj];
            schema_rec.push(fact_recall(&kb, cl, g.case));
            schema_prec.push(fact_recall(&kb, g.case, cl));
            for &m in g.members.iter().take(5) {
                member_rec.push(fact_recall(&kb, cl, m));
                member_prec.push(fact_recall(&kb, m, cl));
            }
        }
        if ti == 0 {
            writeln!(md, "## E7a: template recovery ({n_t} hidden templates × {per} noisy instances, perturbation severity {sev}, 2 distractors; stream order shuffled)\n").unwrap();
            writeln!(md, "Schema fidelity is measured against the clean template, which is never stored. *Recall* = fraction of clean-template facts matched in the schema/instance; *precision* = fraction of schema/instance facts matched in the clean template (generalizations with ≥ 5 members; instance baseline = 5 members of each).\n").unwrap();
            writeln!(md, "Online = after streaming; *after sleep* = after one idle consolidation pass (outliers re-offered, generalizations merged when one maps onto another at ≥ the merge threshold {}).\n", args.f64("merge", 0.6)).unwrap();
            writeln!(md, "| assimilation threshold | online gens / outliers | after sleep: gens / outliers | coverage | purity | completeness | schema recall / precision | instance recall / precision | ms per add | sleep s |").unwrap();
            writeln!(md, "|---|---|---|---|---|---|---|---|---|---|").unwrap();
        }
        writeln!(md, "| {assim} | {n_gens_online} / {n_out_online} | {} / {} | {coverage:.3} | {purity:.3} | {completeness:.3} | {:.3} / {:.3} | {:.3} / {:.3} | {add_ms:.2} | {sleep_s:.1} |",
            sage.gens.len(), sage.outliers.len(), mean(&schema_rec), mean(&schema_prec), mean(&member_rec), mean(&member_prec)).unwrap();
        let _ = (absorbed, merged);
        e7a_rows.push(json!({"assimilate": assim, "gens": sage.gens.len(), "outliers": sage.outliers.len(), "coverage": coverage, "purity": purity, "completeness": completeness,
            "ms_per_add": add_ms, "schema_recall": mean(&schema_rec), "schema_precision": mean(&schema_prec), "member_recall": mean(&member_rec), "member_precision": mean(&member_prec)}));
        eprintln!("[e7a] assim={assim} done ({:.1?})", t0.elapsed());
    }
    j.insert("e7a".into(), json!(e7a_rows));

    // ------------------------------------------------------------ E7b
    writeln!(md, "\n## E7b: few-shot inference — schema vs best single instance (assimilation threshold {})\n", args.f64("assimilate", 0.4)).unwrap();
    writeln!(md, "Memory: M noisy instances of each of {n_t} templates. Query: a fresh noisy instance with one root higher-order fact deleted. The best pool item (fingerprint prefilter + fused FAC) supplies candidate inferences. *Template acc* = the retrieved item is (or generalizes) the query's template; *deleted-fact recall* = the deleted fact is re-inferred; *CI precision* = fraction of structural candidate inferences present in the query's undeleted instance.\n").unwrap();
    writeln!(md, "| M | pool | items in pool | template acc | deleted-fact recall | CI precision | CIs per query |").unwrap();
    writeln!(md, "|---|---|---|---|---|---|---|").unwrap();
    let mut rows = Vec::new();
    let n_q = args.usize("queries-per-template", 4);
    for m_shot in [1usize, 3, 10] {
        let inst = template_instances(&gcfg(2), n_t, m_shot + n_q);
        let mut kb = inst.kb;
        let mut memory: Vec<(CaseId, usize)> = Vec::new();
        let mut queries: Vec<(CaseId, CaseId, usize, String)> = Vec::new(); // (query, full, template, deleted)
        let mut seen: FxHashMap<usize, usize> = FxHashMap::default();
        let plan: Vec<(CaseId, usize)> = inst.cases.iter().filter(|x| !x.2).map(|x| (x.0, x.1)).collect();
        for (c, t) in plan {
            let s = seen.entry(t).or_insert(0);
            *s += 1;
            if *s <= m_shot {
                memory.push((c, t));
            } else if let Some(del) = root_ho_fact(&kb, c) {
                let text = kb.render_expr(del);
                let kept: Vec<ExprId> = kb.case(c).facts.iter().copied().filter(|&f| f != del).collect();
                let name = format!("{}-q", kb.name(kb.case(c).name));
                let q = kb.add_case(&name, CaseKind::Query, kept);
                queries.push((q, c, t, text));
            }
        }
        let mem_cases: Vec<CaseId> = memory.iter().map(|x| x.0).collect();
        let tmpl_of: FxHashMap<CaseId, usize> = memory.iter().copied().collect();
        let stats = stats_for(&kb, &mem_cases);
        let assim = args.f64("assimilate", 0.4);
        let mk = |stats: FeatureStats| Sage::new(SageConfig { assimilate: assim, ..Default::default() }, stats, Sketcher::new(Layout::default(), seed ^ 0xF1), FeatureConfig::default());
        // Pool A: instances only (every memory case is an outlier; no generalization).
        let mut inst_pool = mk(stats.clone());
        inst_pool.outliers = mem_cases.clone();
        // Pool B: SAGE generalizations over memory.
        let mut schema_pool = mk(stats);
        for &c in &mem_cases {
            schema_pool.add(&mut kb, c);
        }
        if args.str("sleep", "yes") == "yes" {
            schema_pool.consolidate(&mut kb, args.f64("merge", 0.6));
        }
        for (label, pool) in [("instances", &mut inst_pool), ("schemas (SAGE)", &mut schema_pool)] {
            let n_items = pool.gens.len() + pool.outliers.len();
            let (mut acc, mut rec, mut prec, mut n_ci) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            for (q, full, t, deleted) in &queries {
                let Some((is_gen, idx, _, _)) = pool.best_match(&kb, *q) else { continue };
                let (item, item_t) = if is_gen {
                    let g = &pool.gens[idx];
                    let mut counts: FxHashMap<usize, usize> = FxHashMap::default();
                    for m in &g.members {
                        *counts.entry(tmpl_of[m]).or_insert(0) += 1;
                    }
                    (g.case, *counts.iter().max_by_key(|x| x.1).unwrap().0)
                } else {
                    let c = pool.outliers[idx];
                    (c, tmpl_of[&c])
                };
                acc.push((item_t == *t) as u8 as f64);
                let mp = Mapper::new(&kb, MapConfig::default());
                let full_facts: std::collections::HashSet<String> = kb.case(*full).facts.iter().map(|&f| kb.render_expr(f)).collect();
                let cis: Vec<String> = mp
                    .best(item, *q)
                    .map(|m| m.inferences.iter().filter(|i| i.grounding == Grounding::Structural && !i.has_skolem).map(|i| mp.render_proj(&i.projected)).collect())
                    .unwrap_or_default();
                rec.push(cis.iter().any(|s| s == deleted) as u8 as f64);
                if !cis.is_empty() {
                    prec.push(cis.iter().filter(|s| full_facts.contains(*s)).count() as f64 / cis.len() as f64);
                }
                n_ci.push(cis.len() as f64);
            }
            writeln!(md, "| {m_shot} | {label} | {n_items} | {:.3} | {:.3} | {:.3} | {:.2} |", mean(&acc), mean(&rec), mean(&prec), mean(&n_ci)).unwrap();
            rows.push(json!({"m": m_shot, "pool": label, "items": n_items, "template_acc": mean(&acc), "deleted_recall": mean(&rec), "ci_precision": mean(&prec), "cis_per_query": mean(&n_ci)}));
        }
        eprintln!("[e7b] M={m_shot} done ({:.1?})", t0.elapsed());
    }
    j.insert("e7b".into(), json!(rows));
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let tag = args.str("tag", &format!("s{sev}"));
    std::fs::write(format!("{out_dir}/E7-{tag}.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E7-{tag}.json"), serde_json::to_string_pretty(&serde_json::Value::Object(j)).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
