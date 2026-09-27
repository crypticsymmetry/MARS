//! E21: hierarchical generalization — schemas of schemas.
//!
//! E7/E15: SAGE fragments each hidden template into several generalizations
//! (completeness 0.6–0.8), and merging generalizations directly at the
//! assimilation threshold destroyed purity (E7). Here level-1 schemas are
//! kept intact and a *second* SAGE pool runs over the level-1 schema cases
//! (level-2 generalizations = schemas of schemas). Questions:
//!
//! 1. Does level 2 recover the template (completeness) without losing
//!    purity? Clusters are scored over the instances they cover.
//! 2. Do level-2 schemas help inference when added to the E15 best pool
//!    (instances + level-1 schemas)?

use crate::e15::{propose, score, Item, Query};
use crate::e7::{root_ho_fact, stats_for};
use crate::metrics::mean;
use crate::Args;
use mars_encode::{FeatureConfig, Layout, Sketcher};
use mars_engine::sage::{Sage, SageConfig};
use mars_gen::{template_instances, GenConfig, Naming, PerturbOp};
use mars_hv::Rng;
use mars_rel::{CaseId, CaseKind, ExprId};
use rustc_hash::FxHashMap;
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

/// Purity and completeness of clusters given as lists of instances.
fn cluster_quality(clusters: &[Vec<CaseId>], tmpl_of: &FxHashMap<CaseId, usize>, n_t: usize, per: usize) -> (f64, f64) {
    let (mut pure, mut total) = (0usize, 0usize);
    let mut best: FxHashMap<usize, usize> = FxHashMap::default();
    for cl in clusters.iter().filter(|c| c.len() >= 2) {
        let mut counts: FxHashMap<usize, usize> = FxHashMap::default();
        for m in cl {
            *counts.entry(tmpl_of[m]).or_insert(0) += 1;
        }
        pure += counts.values().max().copied().unwrap_or(0);
        total += cl.len();
        for (&t, &c) in &counts {
            let e = best.entry(t).or_insert(0);
            *e = (*e).max(c);
        }
    }
    (pure as f64 / total.max(1) as f64, mean(&(0..n_t).map(|t| best.get(&t).copied().unwrap_or(0) as f64 / per as f64).collect::<Vec<_>>()))
}

fn majority(members: &[CaseId], tmpl_of: &FxHashMap<CaseId, usize>) -> usize {
    let mut counts: FxHashMap<usize, usize> = FxHashMap::default();
    for m in members {
        *counts.entry(tmpl_of[m]).or_insert(0) += 1;
    }
    counts.into_iter().max_by_key(|x| (x.1, std::cmp::Reverse(x.0))).map(|x| x.0).unwrap_or(usize::MAX)
}

pub fn run(args: &Args) -> Result<(), String> {
    let seed = args.u64("seed", 1);
    let per = args.usize("per", 10);
    let sev = args.usize("severity", 1);
    let theta1 = args.f64("assimilate", 0.4);
    let theta2s: Vec<f64> = args.str("theta2", "0.3,0.4,0.5").split(',').map(|x| x.parse().unwrap()).collect();
    let templates: Vec<usize> = args.str("templates", "100,1000").split(',').map(|x| x.parse().unwrap()).collect();
    let max_q = args.usize("max-queries", 500);
    // Optional local-null significance gate at level 2 (E16).
    let l2_z: Option<f64> = args.opt("l2-z").map(|v| v.parse().unwrap());
    let out_dir = args.str("out", "results/E21");
    let t0 = Instant::now();
    let mut md = String::new();
    writeln!(md, "# E21: hierarchical generalization — schemas of schemas\n").unwrap();
    writeln!(md, "Memory: {per} noisy instances per hidden template (severity {sev}, 2 distractors; seed {seed}). Level 1: SAGE over the instances (θ₁ = {theta1}, sleep merge 0.6; the E15 pool). Level 2: SAGE over the level-1 schema cases (θ₂ varied, sleep merge 0.6). Clusters are scored over the instances they cover: purity = majority-template share, completeness = per template, the largest share of its instances in one cluster. Inference: E15 protocol (fresh instances with a root higher-order fact deleted; fused retrieval; confidence = summed Laplace fact probability; R@P≥x = best recall at precision ≥ x over m ∈ {{1,3,5}}).\n").unwrap();
    let mut rows = Vec::new();
    for &n_t in &templates {
        let gcfg = GenConfig { seed, naming: Naming::Canonical, distractors: 2, perturb_ops: PerturbOp::ALL.to_vec(), severity: sev, ..Default::default() };
        let inst = template_instances(&gcfg, n_t, per + 2);
        let mut kb = inst.kb;
        let (mut memory, mut queries) = (Vec::new(), Vec::new());
        let mut seen: FxHashMap<usize, usize> = FxHashMap::default();
        for &(c, t, clean) in &inst.cases {
            if clean {
                continue;
            }
            let s = seen.entry(t).or_insert(0);
            *s += 1;
            if *s <= per {
                memory.push((c, t));
            } else if queries.len() < max_q {
                if let Some(del) = root_ho_fact(&kb, c) {
                    let deleted = kb.render_expr(del);
                    let truth = kb.case(c).facts.iter().map(|&f| kb.render_expr(f)).collect();
                    let kept: Vec<ExprId> = kb.case(c).facts.iter().copied().filter(|&f| f != del).collect();
                    let name = format!("{}-q", kb.name(kb.case(c).name));
                    let q = kb.add_case(&name, CaseKind::Query, kept);
                    queries.push(Query { case: q, tmpl: t, deleted, truth });
                }
            }
        }
        let mem: Vec<CaseId> = memory.iter().map(|x| x.0).collect();
        let tmpl_of: FxHashMap<CaseId, usize> = memory.iter().copied().collect();
        let stats = stats_for(&kb, &mem);
        let sk = Sketcher::new(Layout::default(), seed ^ 0xF1);

        // Level 1.
        let mut order = mem.clone();
        Rng::new(seed ^ 0xE15).shuffle(&mut order);
        let mut l1 = Sage::new(SageConfig { assimilate: theta1, namespace: "h1s".into(), ..Default::default() }, stats.clone(), sk.clone(), FeatureConfig::default());
        for &c in &order {
            l1.add(&mut kb, c);
        }
        l1.consolidate(&mut kb, 0.6);
        let l1_members: FxHashMap<CaseId, Vec<CaseId>> = l1.gens.iter().map(|g| (g.case, g.members.clone())).collect();
        let l1_clusters: Vec<Vec<CaseId>> = l1.gens.iter().map(|g| g.members.clone()).collect();
        let (p1, c1) = cluster_quality(&l1_clusters, &tmpl_of, n_t, per);
        let mut base_items: Vec<Item> = mem.iter().map(|&c| Item { case: c, tmpl: tmpl_of[&c], counts: None }).collect();
        base_items.extend(l1.gens.iter().map(|g| Item { case: g.case, tmpl: majority(&g.members, &tmpl_of), counts: Some((g.facts.iter().copied().collect(), g.members.len() as u32)) }));
        let eval = |kb: &mars_rel::Kb, items: &[Item]| {
            let (props, _, ms) = propose(kb, &stats, &sk, items, &queries, 5, 16);
            let s: Vec<_> = [1usize, 3, 5].iter().map(|&m| score(&props, &queries, m)).collect();
            let r60 = s.iter().map(|x| x.r_at_p60).fold(0.0, f64::max);
            let r80 = s.iter().map(|x| x.r_at_p80).fold(0.0, f64::max);
            (s[0].recall, s[0].precision, r60, r80, ms)
        };
        let e1 = eval(&kb, &base_items);
        writeln!(md, "\n## T = {n_t} templates ({} memory cases, {} queries)\n", mem.len(), queries.len()).unwrap();
        writeln!(md, "| level | θ₂ | generalizations (+ outliers) | purity | completeness | inference pool | m=1 recall / precision | R@P≥0.6 | R@P≥0.8 |").unwrap();
        writeln!(md, "|---|---|---|---|---|---|---|---|---|").unwrap();
        writeln!(md, "| 1 | — | {} (+ {}) | {p1:.3} | {c1:.3} | instances + L1 | {:.3} / {:.3} | {:.3} | {:.3} |", l1.gens.len(), l1.outliers.len(), e1.0, e1.1, e1.2, e1.3).unwrap();
        rows.push(json!({"templates": n_t, "level": 1, "gens": l1.gens.len(), "outliers": l1.outliers.len(), "purity": p1, "completeness": c1, "recall_m1": e1.0, "precision_m1": e1.1, "r_at_p60": e1.2, "r_at_p80": e1.3}));
        eprintln!("[e21] T={n_t}: L1 {} gens, purity {p1:.3}, completeness {c1:.3} ({:.1?})", l1.gens.len(), t0.elapsed());

        // Diagnostic: can any threshold separate fragments of one template from
        // schemas of different templates? Normalized FAC over all L1 pairs.
        {
            use rayon::prelude::*;
            let mp = mars_map::Mapper::new(&kb, mars_map::MapConfig::default());
            let g: Vec<(CaseId, usize, f64)> = l1.gens.iter().filter(|g| g.members.len() >= 2).map(|g| (g.case, majority(&g.members, &tmpl_of), mp.score(g.case, g.case) as f64)).collect();
            let pairs: Vec<(bool, f64, f64)> = (0..g.len())
                .into_par_iter()
                .flat_map_iter(|i| {
                    let g = &g;
                    let mp = &mp;
                    (i + 1..g.len()).map(move |j| {
                        let (a, b) = (&g[i], &g[j]);
                        let raw = mp.score(a.0, b.0).max(mp.score(b.0, a.0)) as f64;
                        let sym = if raw == 0.0 { 0.0 } else { (raw / (a.2 * b.2).sqrt()).min(1.0) };
                        let cov = if raw == 0.0 { 0.0 } else { (raw / a.2.min(b.2)).min(1.0) };
                        (a.1 == b.1, sym, cov)
                    })
                })
                .collect();
            let same: Vec<&(bool, f64, f64)> = pairs.iter().filter(|p| p.0).collect();
            let diff: Vec<&(bool, f64, f64)> = pairs.iter().filter(|p| !p.0).collect();
            let q = |v: &[f64], x: f64| {
                let mut v = v.to_vec();
                v.sort_by(|a, b| a.total_cmp(b));
                v.get(((v.len() as f64 - 1.0) * x) as usize).copied().unwrap_or(f64::NAN)
            };
            for (name, k) in [("symmetric", 1usize), ("smaller-schema coverage", 2)] {
                let sv: Vec<f64> = same.iter().map(|p| if k == 1 { p.1 } else { p.2 }).collect();
                let dv: Vec<f64> = diff.iter().map(|p| if k == 1 { p.1 } else { p.2 }).collect();
                let auc = crate::metrics::auc(&sv, &dv);
                writeln!(md, "Level-1 schema pairs ({name} FAC): same template n={} median {:.2} (q10 {:.2}), different n={} median {:.2} (q99 {:.2}, max {:.2}); AUC {auc:.3}.\n", sv.len(), q(&sv, 0.5), q(&sv, 0.1), dv.len(), q(&dv, 0.5), q(&dv, 0.99), q(&dv, 1.0)).unwrap();
                rows.push(json!({"templates": n_t, "diagnostic": name, "same_n": sv.len(), "same_median": q(&sv, 0.5), "diff_q99": q(&dv, 0.99), "auc": auc}));
            }
        }

        // Level 2 over the level-1 schema cases.
        let l1_cases: Vec<CaseId> = l1.gens.iter().map(|g| g.case).collect();
        for &theta2 in &theta2s {
            let mut l2 = Sage::new(SageConfig { assimilate: theta2, min_z: l2_z, namespace: format!("h2s{}z{}", (theta2 * 100.0) as u32, l2_z.map(|z| z as u32).unwrap_or(0)), ..Default::default() }, stats.clone(), sk.clone(), FeatureConfig::default());
            for &c in &l1_cases {
                l2.add(&mut kb, c);
            }
            l2.consolidate(&mut kb, 0.6);
            // Level-2 clusters over instances: each L2 gen covers its L1 members' instances;
            // an L1 schema left as an L2 outlier remains its own cluster.
            let mut clusters: Vec<Vec<CaseId>> = l2.gens.iter().map(|g| g.members.iter().flat_map(|m| l1_members[m].clone()).collect()).collect();
            clusters.extend(l2.outliers.iter().map(|o| l1_members[o].clone()));
            let (p2, c2) = cluster_quality(&clusters, &tmpl_of, n_t, per);
            let mut items: Vec<Item> = base_items.iter().map(|i| Item { case: i.case, tmpl: i.tmpl, counts: i.counts.clone() }).collect();
            items.extend(l2.gens.iter().map(|g| {
                let inst: Vec<CaseId> = g.members.iter().flat_map(|m| l1_members[m].clone()).collect();
                Item { case: g.case, tmpl: majority(&inst, &tmpl_of), counts: Some((g.facts.iter().copied().collect(), g.members.len() as u32)) }
            }));
            let e2 = eval(&kb, &items);
            let zl = l2_z.map(|z| format!(", z≥{z}")).unwrap_or_default();
            writeln!(md, "| 2 | {theta2}{zl} | {} (+ {} L1 schemas alone) | {p2:.3} | {c2:.3} | instances + L1 + L2 | {:.3} / {:.3} | {:.3} | {:.3} |", l2.gens.len(), l2.outliers.len(), e2.0, e2.1, e2.2, e2.3).unwrap();
            rows.push(json!({"templates": n_t, "level": 2, "theta2": theta2, "l2_z": l2_z, "gens": l2.gens.len(), "outliers": l2.outliers.len(), "purity": p2, "completeness": c2, "recall_m1": e2.0, "precision_m1": e2.1, "r_at_p60": e2.2, "r_at_p80": e2.3}));
            eprintln!("[e21] T={n_t} θ2={theta2}: L2 {} gens, purity {p2:.3}, completeness {c2:.3} ({:.1?})", l2.gens.len(), t0.elapsed());
        }
    }
    writeln!(md, "\nRuntime {:.1?}.", t0.elapsed()).unwrap();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let tag = args.str("tag", &format!("s{sev}"));
    std::fs::write(format!("{out_dir}/E21-{tag}.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E21-{tag}.json"), serde_json::to_string_pretty(&json!({"seed": seed, "per": per, "theta1": theta1, "rows": rows})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
