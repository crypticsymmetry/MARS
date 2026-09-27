//! E20: near-miss learning — what matters in a concept?
//!
//! Concepts are hidden templates. Positives are noisy instances; near-misses
//! are noisy instances of a random *re-wiring* of the template (same
//! first-order content, different causal structure: the cases a similarity
//! threshold confuses). Per concept: a SAGE schema from 10 training
//! positives, k ∈ {0, 1, 2, 5} training near-misses, and 5 + 5 fresh test
//! cases. Classifiers:
//!
//! * A — schema threshold: accept iff normalized FAC(schema → x) ≥ the
//!   lowest training positive's (positives only);
//! * B — **near-miss critical facts** (Winston's "must-have" conditions):
//!   schema facts missing from some training near-miss's mapping but
//!   present in ≥ 80% of training positives; accept iff A accepts and x
//!   matches at least the fraction of critical facts every training
//!   positive matched;
//! * C — 1-nearest neighbour by normalized FAC over all labeled training
//!   cases (the discriminative baseline);
//! * D — **near-miss emphasis weights** (soft Winston): each schema fact is
//!   weighted by P(matched | positive) − P(matched | near-miss) over the
//!   training cases; a case scores its weighted matched fraction, and the
//!   threshold sits midway between the training positives' and near-misses'
//!   mean scores (k = 0: classifier A).

use crate::metrics::mean;
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Layout, Sketcher};
use mars_engine::sage::{Diagnostic, Sage, SageConfig};
use mars_gen::{concept_instances, GenConfig, Naming, PerturbOp};
use mars_map::{MapConfig, Mapper};
use mars_rel::{CaseId, ExprId, Kb, Term};
use rayon::prelude::*;
use serde_json::json;
use std::collections::HashSet;
use std::fmt::Write as _;
use std::time::Instant;

const KS: [usize; 4] = [0, 1, 2, 5];

/// Normalized FAC of base → target and the set of base facts in the mapping.
fn map_facts(kb: &Kb, mp: &Mapper, base: CaseId, target: CaseId) -> (f64, HashSet<ExprId>) {
    let Some(m) = mp.best(base, target) else { return (0.0, HashSet::new()) };
    let (sb, st) = (mp.score(base, base) as f64, mp.score(target, target) as f64);
    let norm = if sb == 0.0 || st == 0.0 { 0.0 } else { (m.score as f64 / (sb * st).sqrt()).min(1.0) };
    let facts: HashSet<ExprId> = kb.case(base).facts.iter().copied().collect();
    (norm, m.correspondences.iter().filter_map(|(b, _)| if let Term::Expr(e) = b { facts.contains(e).then_some(*e) } else { None }).collect())
}

struct ConceptResult {
    /// Per k, per classifier (A, B, C, D): (true positives, positives, true negatives, negatives).
    counts: Vec<[(usize, usize, usize, usize); 4]>,
    /// Per k: (critical facts, of which higher-order).
    critical: Vec<(usize, usize)>,
}

pub fn run(args: &Args) -> Result<(), String> {
    let seed = args.u64("seed", 1);
    let n_t = args.usize("concepts", 100);
    let sev = args.usize("severity", 1);
    let (p_train, p_test, n_test) = (10usize, 5usize, 5usize);
    let out_dir = args.str("out", "results/E20");
    let t0 = Instant::now();
    let cfg = GenConfig { seed, naming: Naming::Canonical, distractors: 2, perturb_ops: PerturbOp::ALL.to_vec(), severity: sev, ..Default::default() };
    let n_neg_train = *KS.iter().max().unwrap();
    let cs = concept_instances(&cfg, n_t, p_train + p_test, n_neg_train + n_test);
    let mut kb = cs.kb;
    let nc = cs.n_concepts;
    let per: Vec<(Vec<CaseId>, Vec<CaseId>)> = (0..nc)
        .map(|ci| {
            let p = cs.cases.iter().filter(|x| x.1 == ci && x.2).map(|x| x.0).collect();
            let n = cs.cases.iter().filter(|x| x.1 == ci && !x.2).map(|x| x.0).collect();
            (p, n)
        })
        .collect();
    let train: Vec<CaseId> = per.iter().flat_map(|(p, n)| p[..p_train].iter().chain(&n[..n_neg_train]).copied()).collect();
    let stats = {
        let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
        let f: Vec<_> = train.iter().map(|&c| fx.extract(c)).collect();
        FeatureStats::fit(f.iter())
    };

    // One schema per concept, from its training positives.
    let mut schemas = Vec::new();
    let mut sizes = Vec::new();
    for (ci, (p, _)) in per.iter().enumerate() {
        let mut sage = Sage::new(SageConfig { assimilate: 0.2, namespace: format!("c{ci}s"), ..Default::default() }, stats.clone(), Sketcher::new(Layout::default(), seed ^ 0xF1), FeatureConfig::default());
        for &c in &p[..p_train] {
            sage.add(&mut kb, c);
        }
        sage.consolidate(&mut kb, 0.4);
        let g = sage.gens.iter().max_by_key(|g| (g.members.len(), std::cmp::Reverse(g.id))).map(|g| (g.case, g.members.len()));
        let (case, m) = g.unwrap_or((p[0], 1));
        sizes.push(m as f64);
        schemas.push(case);
    }
    eprintln!("[e20] {nc} concepts, schemas built ({:.1?}); mean members {:.1}/{p_train}", t0.elapsed(), mean(&sizes));

    let mp = Mapper::new(&kb, MapConfig::default());
    let results: Vec<ConceptResult> = (0..nc)
        .into_par_iter()
        .map(|ci| {
            let (p, n) = &per[ci];
            let s = schemas[ci];
            let pos_train: Vec<(f64, HashSet<ExprId>)> = p[..p_train].iter().map(|&x| map_facts(&kb, &mp, s, x)).collect();
            let theta = pos_train.iter().map(|x| x.0).fold(f64::INFINITY, f64::min);
            let tests: Vec<(CaseId, bool)> = p[p_train..].iter().map(|&x| (x, true)).chain(n[n_neg_train..].iter().map(|&x| (x, false))).collect();
            let test_maps: Vec<(f64, HashSet<ExprId>)> = tests.iter().map(|&(x, _)| map_facts(&kb, &mp, s, x)).collect();
            let fac = |a: CaseId, b: CaseId| map_facts(&kb, &mp, a, b).0;
            let mut counts = Vec::new();
            let mut critical = Vec::new();
            for &k in &KS {
                let schema_facts = kb.case(s).facts.clone();
                let nm_maps: Vec<HashSet<ExprId>> = n[..k].iter().map(|&x| map_facts(&kb, &mp, s, x).1).collect();
                let crit: Vec<ExprId> = schema_facts
                    .iter()
                    .copied()
                    .filter(|f| nm_maps.iter().any(|m| !m.contains(f)) && pos_train.iter().filter(|(_, m)| m.contains(f)).count() as f64 >= 0.8 * p_train as f64)
                    .collect();
                let frac = |m: &HashSet<ExprId>| if crit.is_empty() { 1.0 } else { crit.iter().filter(|f| m.contains(f)).count() as f64 / crit.len() as f64 };
                let tau = pos_train.iter().map(|(_, m)| frac(m)).fold(f64::INFINITY, f64::min);
                critical.push((crit.len(), crit.iter().filter(|&&f| kb.order(f) >= 2).count()));
                // D: emphasis weights (mars_engine::sage::Diagnostic).
                let diag = (k > 0).then(|| Diagnostic::train(&kb, &MapConfig::default(), s, &p[..p_train], &n[..k]));
                let labeled: Vec<(CaseId, bool)> = p[..p_train].iter().map(|&x| (x, true)).chain(n[..k].iter().map(|&x| (x, false))).collect();
                let mut c3 = [(0usize, 0usize, 0usize, 0usize); 4];
                for (ti, &(x, label)) in tests.iter().enumerate() {
                    let (score, m) = &test_maps[ti];
                    let a = *score >= theta;
                    let b = a && frac(m) >= tau;
                    let c = labeled.iter().map(|&(y, l)| (fac(y, x), l)).max_by(|u, v| u.0.total_cmp(&v.0).then(v.1.cmp(&u.1))).map(|u| u.1).unwrap_or(true);
                    let d = match &diag {
                        Some(dg) => dg.accepts(&kb, &MapConfig::default(), x),
                        None => a,
                    };
                    for (j, pred) in [a, b, c, d].into_iter().enumerate() {
                        if label {
                            c3[j].1 += 1;
                            c3[j].0 += pred as usize;
                        } else {
                            c3[j].3 += 1;
                            c3[j].2 += (!pred) as usize;
                        }
                    }
                }
                counts.push(c3);
            }
            ConceptResult { counts, critical }
        })
        .collect();

    let mut md = String::new();
    writeln!(md, "# E20: near-miss learning\n").unwrap();
    writeln!(md, "{nc} concepts (hidden templates; severity {sev}, 2 distractors; seed {seed}). Per concept: a SAGE schema from {p_train} training positives (mean {:.1} members), k training near-misses (instances of fresh random re-wirings: same first-order content, different causal structure), and {p_test} + {n_test} fresh test cases. Balanced accuracy = (true-positive rate + true-negative rate) / 2, pooled over concepts. Runtime {:.1?}.\n", mean(&sizes), t0.elapsed()).unwrap();
    writeln!(md, "| k near-misses | A: schema threshold | B: + critical facts (hard) | C: 1-NN over labeled cases | **D: near-miss emphasis weights** | critical facts / concept (higher-order share) |").unwrap();
    writeln!(md, "|---|---|---|---|---|---|").unwrap();
    let mut rows = Vec::new();
    for (ki, &k) in KS.iter().enumerate() {
        let mut cells = Vec::new();
        let mut jr = Vec::new();
        for j in 0..4 {
            let (tp, p, tn, n) = results.iter().fold((0, 0, 0, 0), |acc, r| {
                let c = r.counts[ki][j];
                (acc.0 + c.0, acc.1 + c.1, acc.2 + c.2, acc.3 + c.3)
            });
            let (tpr, tnr) = (tp as f64 / p.max(1) as f64, tn as f64 / n.max(1) as f64);
            cells.push(format!("{:.3} ({tpr:.2} / {tnr:.2})", (tpr + tnr) / 2.0));
            jr.push(json!({"tpr": tpr, "tnr": tnr, "balanced": (tpr + tnr) / 2.0}));
        }
        let crit: f64 = mean(&results.iter().map(|r| r.critical[ki].0 as f64).collect::<Vec<_>>());
        let ho: usize = results.iter().map(|r| r.critical[ki].1).sum();
        let all: usize = results.iter().map(|r| r.critical[ki].0).sum();
        writeln!(md, "| {k} | {} | {} | {} | {} | {crit:.2} ({:.2}) |", cells[0], cells[1], cells[2], cells[3], ho as f64 / all.max(1) as f64).unwrap();
        rows.push(json!({"k": k, "A": jr[0], "B": jr[1], "C": jr[2], "D": jr[3], "critical_per_concept": crit, "critical_ho_share": ho as f64 / all.max(1) as f64}));
    }
    writeln!(md, "\nCells: balanced accuracy (true-positive rate / true-negative rate).").unwrap();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let tag = args.str("tag", &format!("s{sev}"));
    std::fs::write(format!("{out_dir}/E20-{tag}.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E20-{tag}.json"), serde_json::to_string_pretty(&json!({"seed": seed, "concepts": nc, "severity": sev, "p_train": p_train, "rows": rows})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
