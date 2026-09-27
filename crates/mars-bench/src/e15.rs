//! E15: where to pool evidence — at storage (SAGE schemas) or at recall
//! (corroboration across retrieved instances, E11)?
//!
//! Memory: M noisy instances of each of T hidden templates (as E7/E11).
//! Query: a fresh instance with one root higher-order fact deleted. Pools:
//!
//! * instances — every memory case;
//! * schemas (symmetric) — SAGE generalizations + outliers, assimilation by
//!   the symmetric normalized score (E7);
//! * schemas (coverage) — assimilation into established generalizations by
//!   schema coverage `S(g→x)/S(g→g)` (fragmentation fix).
//!
//! Every pool is queried the same way (fingerprint prefilter, fused
//! 0.3·FAC + 0.7·FP re-rank, top-m items), and each candidate inference gets
//! the confidence Σ over proposing items of the Laplace-smoothed probability
//! of its base fact, `(count + 1) / (members + 2)` (an instance: 2/3, so for
//! instances confidence ∝ E11's support count).

use crate::e7::{root_ho_fact, stats_for};
use crate::metrics::mean;
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Layout, Profile, Sketcher, N_CHANNELS};
use mars_engine::sage::{Sage, SageConfig};
use mars_gen::{template_instances, GenConfig, Naming, PerturbOp};
use mars_hv::{HyperVector, Rng};
use mars_map::{Grounding, MapConfig, Mapper};
use mars_rel::{CaseId, CaseKind, ExprId, Kb};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde_json::json;
use std::collections::HashSet;
use std::fmt::Write as _;
use std::time::Instant;

/// A retrievable pool item: its case, and per base fact the (count, members)
/// evidence behind it (`None` = a plain instance).
pub(crate) struct Item {
    pub case: CaseId,
    pub tmpl: usize,
    pub counts: Option<(FxHashMap<ExprId, u32>, u32)>,
}

impl Item {
    fn prob(&self, f: ExprId) -> f64 {
        match &self.counts {
            None => 2.0 / 3.0,
            Some((c, n)) => (c.get(&f).copied().unwrap_or(0) as f64 + 1.0) / (*n as f64 + 2.0),
        }
    }
}

pub(crate) struct Query {
    pub case: CaseId,
    pub tmpl: usize,
    pub deleted: String,
    pub truth: HashSet<String>,
}

/// Per query and per retrieved rank: the proposed (inference text, probability) list.
/// (label, items, build ms per case, (gens, outliers, purity, completeness)).
type Pool = (String, Vec<Item>, f64, Option<(usize, usize, f64, f64)>);
type QueryProposals = Vec<Vec<(String, f64)>>;
pub(crate) type Proposals = Vec<QueryProposals>;

fn sketch_all(kb: &Kb, stats: &FeatureStats, sk: &Sketcher, cases: &[CaseId]) -> Vec<HyperVector> {
    let fx = FeatureExtractor::new(kb, FeatureConfig::default());
    cases
        .par_iter()
        .map(|&c| {
            let mut f = fx.extract(c);
            stats.apply(&mut f, &[true; N_CHANNELS]);
            sk.sketch(&f)
        })
        .collect()
}

pub(crate) fn propose(kb: &Kb, stats: &FeatureStats, sk: &Sketcher, items: &[Item], queries: &[Query], m_max: usize, prefilter: usize) -> (Proposals, Vec<usize>, f64) {
    let t = Instant::now();
    let icases: Vec<CaseId> = items.iter().map(|i| i.case).collect();
    let ifp = sketch_all(kb, stats, sk, &icases);
    let qfp = sketch_all(kb, stats, sk, &queries.iter().map(|q| q.case).collect::<Vec<_>>());
    let mapper = Mapper::new(kb, MapConfig::default());
    let iself: Vec<f64> = icases.par_iter().map(|&c| mapper.score(c, c) as f64).collect();
    let profile = Profile::analogy();
    let res: Vec<(QueryProposals, usize)> = queries
        .par_iter()
        .enumerate()
        .map(|(qi, q)| {
            let mut s: Vec<(usize, f64)> = (0..items.len()).map(|i| (i, profile.score(&sk.channel_sims(&qfp[qi], &ifp[i])))).collect();
            s.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            s.truncate(prefilter);
            let qself = mapper.score(q.case, q.case) as f64;
            let mut fused: Vec<(usize, f64)> = s
                .into_iter()
                .map(|(i, fp)| {
                    let raw = mapper.score(items[i].case, q.case) as f64;
                    let fac = if raw == 0.0 { 0.0 } else { (raw / (qself * iself[i]).sqrt()).min(1.0) };
                    (i, 0.3 * fac + 0.7 * fp)
                })
                .collect();
            fused.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let top_t = fused.first().map(|x| items[x.0].tmpl).unwrap_or(usize::MAX);
            let props = fused
                .iter()
                .take(m_max)
                .map(|&(i, _)| {
                    let mut v: Vec<(String, f64)> = mapper
                        .best(items[i].case, q.case)
                        .map(|m| m.inferences.iter().filter(|x| x.grounding == Grounding::Structural && !x.has_skolem).map(|x| (mapper.render_proj(&x.projected), items[i].prob(x.base_fact))).collect())
                        .unwrap_or_default();
                    v.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.total_cmp(&a.1)));
                    v.dedup_by(|a, b| a.0 == b.0);
                    v
                })
                .collect();
            (props, top_t)
        })
        .collect();
    let ms = t.elapsed().as_secs_f64() * 1e3 / queries.len().max(1) as f64;
    let (props, tops) = res.into_iter().unzip();
    (props, tops, ms)
}

pub(crate) struct Scores {
    pub recall: f64,
    pub precision: f64,
    pub per_q: f64,
    pub r_at_p60: f64,
    pub r_at_p80: f64,
    pub calib: Vec<(f64, f64, usize, f64)>,
}

/// Aggregate the top-m proposals by summed probability and sweep a confidence threshold.
pub(crate) fn score(props: &Proposals, queries: &[Query], m: usize) -> Scores {
    // (confidence, true, deleted, query)
    let mut all: Vec<(f64, bool, bool)> = Vec::new();
    for (qi, q) in queries.iter().enumerate() {
        let mut conf: FxHashMap<&str, f64> = FxHashMap::default();
        for p in props[qi].iter().take(m) {
            for (t, pr) in p {
                *conf.entry(t.as_str()).or_insert(0.0) += pr;
            }
        }
        for (t, c) in conf {
            all.push((c, q.truth.contains(t), t == q.deleted));
        }
    }
    let nq = queries.len().max(1) as f64;
    all.sort_by(|a, b| b.0.total_cmp(&a.0));
    let (mut tp, mut acc, mut del) = (0usize, 0usize, 0usize);
    let (mut r60, mut r80) = (0.0f64, 0.0f64);
    let mut i = 0;
    while i < all.len() {
        let c = all[i].0;
        while i < all.len() && all[i].0 == c {
            acc += 1;
            tp += all[i].1 as usize;
            del += all[i].2 as usize;
            i += 1;
        }
        let (p, r) = (tp as f64 / acc as f64, del as f64 / nq);
        if p >= 0.6 {
            r60 = r60.max(r);
        }
        if p >= 0.8 {
            r80 = r80.max(r);
        }
    }
    let edges = [0.0, 0.5, 0.67, 0.8, 1.0, 1.5, 2.5, f64::INFINITY];
    let calib = edges
        .windows(2)
        .filter_map(|w| {
            let b: Vec<&(f64, bool, bool)> = all.iter().filter(|x| x.0 >= w[0] && x.0 < w[1]).collect();
            (!b.is_empty()).then(|| (w[0], w[1], b.len(), b.iter().filter(|x| x.1).count() as f64 / b.len() as f64))
        })
        .collect();
    Scores { recall: del as f64 / nq, precision: tp as f64 / acc.max(1) as f64, per_q: acc as f64 / nq, r_at_p60: r60, r_at_p80: r80, calib }
}

pub fn run(args: &Args) -> Result<(), String> {
    let seed = args.u64("seed", 1);
    let per = args.usize("per", 10);
    let n_q = args.usize("queries-per-template", 2);
    let max_q = args.usize("max-queries", 500);
    let sev = args.usize("severity", 1);
    let theta = args.f64("assimilate", 0.4);
    let cov_theta = args.f64("cov-theta", 0.5);
    let prefilter = args.usize("prefilter", 16);
    // Optional local-null significance gate on assimilation (E16).
    let min_z: Option<f64> = args.opt("min-z").map(|v| v.parse().unwrap());
    let templates: Vec<usize> = args.str("templates", "100,1000").split(',').map(|x| x.parse().unwrap()).collect();
    let out_dir = args.str("out", "results/E15");
    let which = args.str("pools", "inst,sym,cov,both");
    let want = |k: &str| which.split(',').any(|x| x == k);
    let t0 = Instant::now();
    let mut md = String::new();
    writeln!(md, "# E15: pooling evidence at storage (schemas) vs at recall (corroboration)\n").unwrap();
    writeln!(md, "Memory: {per} noisy instances per hidden template (severity {sev}, 2 distractors). Queries: fresh instances with one root higher-order fact deleted (≤ {max_q}). Each pool is queried identically (fingerprint top-{prefilter}, fused 0.3·FAC+0.7·FP re-rank, top-m items); an inference's confidence is the sum over proposing items of its base fact's Laplace probability (count+1)/(members+2). *recall* = deleted fact proposed; *precision* = proposed facts in the undeleted instance; R@P≥x = best deleted-fact recall at a confidence threshold with precision ≥ x.\n").unwrap();
    let mut rows = Vec::new();
    let mut calib_md = String::new();
    for &n_t in &templates {
        let gcfg = GenConfig { seed, naming: Naming::Canonical, distractors: 2, perturb_ops: PerturbOp::ALL.to_vec(), severity: sev, ..Default::default() };
        let inst = template_instances(&gcfg, n_t, per + n_q);
        let mut kb = inst.kb;
        let mut memory: Vec<(CaseId, usize)> = Vec::new();
        let mut queries: Vec<Query> = Vec::new();
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
        eprintln!("[e15] T={n_t}: {} memory cases, {} queries ({:.1?})", mem.len(), queries.len(), t0.elapsed());

        // Pools.
        let mut pools: Vec<Pool> = Vec::new();
        if want("inst") {
            pools.push(("instances".into(), mem.iter().map(|&c| Item { case: c, tmpl: tmpl_of[&c], counts: None }).collect(), 0.0, None));
        }
        let zlabel = min_z.map(|z| format!(", z≥{z}")).unwrap_or_default();
        for (label, cov) in [(format!("schemas, symmetric θ={theta}{zlabel}"), false), (format!("schemas, coverage θ={cov_theta}{zlabel}"), true)] {
            let needed = want(if cov { "cov" } else { "sym" }) || (!cov && want("both"));
            if !needed {
                continue;
            }
            let tb = Instant::now();
            let mut order = mem.clone();
            Rng::new(seed ^ 0xE15).shuffle(&mut order);
            let mut sage = Sage::new(SageConfig { assimilate: if cov { cov_theta } else { theta }, coverage: cov, namespace: if cov { "cschema".into() } else { "sschema".into() }, min_z, ..Default::default() }, stats.clone(), sk.clone(), FeatureConfig::default());
            for &c in &order {
                sage.add(&mut kb, c);
            }
            sage.consolidate(&mut kb, 0.6);
            let build_ms = tb.elapsed().as_secs_f64() * 1e3 / mem.len() as f64;
            // Cluster quality.
            let (mut pure, mut total) = (0usize, 0usize);
            let mut best_share: FxHashMap<usize, usize> = FxHashMap::default();
            let mut items = Vec::new();
            for g in &sage.gens {
                let mut counts: FxHashMap<usize, usize> = FxHashMap::default();
                for m in &g.members {
                    *counts.entry(tmpl_of[m]).or_insert(0) += 1;
                }
                let (&maj, &cnt) = counts.iter().max_by_key(|x| (*x.1, std::cmp::Reverse(*x.0))).unwrap();
                pure += cnt;
                total += g.members.len();
                for (&t, &c) in &counts {
                    let e = best_share.entry(t).or_insert(0);
                    *e = (*e).max(c);
                }
                items.push(Item { case: g.case, tmpl: maj, counts: Some((g.facts.iter().copied().collect(), g.members.len() as u32)) });
            }
            let completeness = mean(&(0..n_t).map(|t| best_share.get(&t).copied().unwrap_or(0) as f64 / per as f64).collect::<Vec<_>>());
            let (n_g, n_o) = (sage.gens.len(), sage.outliers.len());
            if want("both") && !cov {
                // Instances and schemas side by side: corroboration spans both.
                let mut b: Vec<Item> = items.iter().map(|i| Item { case: i.case, tmpl: i.tmpl, counts: i.counts.clone() }).collect();
                b.extend(mem.iter().map(|&c| Item { case: c, tmpl: tmpl_of[&c], counts: None }));
                pools.push((format!("instances + schemas (θ={theta}{zlabel})"), b, build_ms, None));
            }
            if !want(if cov { "cov" } else { "sym" }) {
                continue;
            }
            for &o in &sage.outliers {
                items.push(Item { case: o, tmpl: tmpl_of[&o], counts: None });
            }
            pools.push((label, items, build_ms, Some((n_g, n_o, pure as f64 / total.max(1) as f64, completeness))));
            eprintln!("[e15] T={n_t}: {} built: {n_g} gens + {n_o} outliers ({:.1?})", pools.last().unwrap().0, t0.elapsed());
        }

        writeln!(md, "\n## T = {n_t} templates ({} memory cases, {} queries)\n", mem.len(), queries.len()).unwrap();
        writeln!(md, "| pool | items (gens + outliers) | purity / completeness | build ms/case | query ms | template acc | m | recall | precision | inferences/q | R@P≥0.6 | R@P≥0.8 |").unwrap();
        writeln!(md, "|---|---|---|---|---|---|---|---|---|---|---|---|").unwrap();
        for (label, items, build_ms, cl) in &pools {
            let (props, tops, q_ms) = propose(&kb, &stats, &sk, items, &queries, 5, prefilter);
            let tacc = mean(&queries.iter().zip(&tops).map(|(q, &t)| (q.tmpl == t) as u8 as f64).collect::<Vec<_>>());
            let size = match cl {
                Some((g, o, _, _)) => format!("{} ({g} + {o})", items.len()),
                None => items.len().to_string(),
            };
            let quality = cl.map(|(_, _, p, c)| format!("{p:.3} / {c:.3}")).unwrap_or("—".into());
            for m in [1usize, 3, 5] {
                let s = score(&props, &queries, m);
                let head = if m == 1 { format!("| {label} | {size} | {quality} | {build_ms:.2} | {q_ms:.2} | {tacc:.3} |") } else { "| | | | | | |".to_string() };
                writeln!(md, "{head} {m} | {:.3} | {:.3} | {:.2} | {:.3} | {:.3} |", s.recall, s.precision, s.per_q, s.r_at_p60, s.r_at_p80).unwrap();
                rows.push(json!({"templates": n_t, "pool": label, "items": items.len(), "cluster": cl.map(|x| json!({"gens": x.0, "outliers": x.1, "purity": x.2, "completeness": x.3})), "build_ms": build_ms, "query_ms": q_ms, "template_acc": tacc, "m": m, "recall": s.recall, "precision": s.precision, "per_query": s.per_q, "r_at_p60": s.r_at_p60, "r_at_p80": s.r_at_p80, "calibration": s.calib}));
                if m == 1 || m == 5 {
                    writeln!(calib_md, "| {n_t} | {label} | {m} | {} |", s.calib.iter().map(|(lo, hi, n, p)| format!("[{lo:.2},{}) n={n} p={p:.2}", if hi.is_finite() { format!("{hi:.2}") } else { "∞".into() })).collect::<Vec<_>>().join("; ")).unwrap();
                }
            }
        }
        eprintln!("[e15] T={n_t} evaluated ({:.1?})", t0.elapsed());
    }
    writeln!(md, "\n## Calibration: precision by confidence bucket\n\n| T | pool | m | buckets |\n|---|---|---|---|\n{calib_md}").unwrap();
    writeln!(md, "Runtime {:.1?}.", t0.elapsed()).unwrap();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let tag = args.str("tag", &format!("s{sev}"));
    std::fs::write(format!("{out_dir}/E15-{tag}.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E15-{tag}.json"), serde_json::to_string_pretty(&json!({"rows": rows})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
