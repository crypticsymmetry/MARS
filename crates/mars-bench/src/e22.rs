//! E22: what score identifies membership in a schema?
//!
//! E21 found that whole-case similarity cannot identify the fragments of a
//! template. Here the decision SAGE makes at assimilation time is isolated:
//! given level-1 schemas (E15 memory) and a fresh case, pick its best schema
//! and decide whether it belongs there. Fresh cases are instances of stored
//! templates (belonging to *some* schema of their template is correct) and
//! of absent templates (nothing is correct). Scores for the candidate
//! schemas (fingerprint top-16 among generalizations with ≥ 2 members):
//!
//! * symmetric normalized FAC (SAGE's default);
//! * coverage S(g→x)/S(g→g) (E15's option);
//! * **core fraction**: share of the schema's *core* facts (probability
//!   ≥ 0.7 among its members) placed in correspondence;
//! * core × symmetric (geometric mean).
//!
//! Each score picks the best schema and is thresholded; reported: AUC of
//! correct vs wrong decisions and recall at precision ≥ 0.9 / 0.95.

use crate::e7::stats_for;
use crate::metrics::auc;
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, Layout, Profile, Sketcher, N_CHANNELS};
use mars_engine::sage::{Sage, SageConfig};
use mars_gen::{template_instances, GenConfig, Naming, PerturbOp};
use mars_hv::Rng;
use mars_map::{MapConfig, Mapper};
use mars_rel::{CaseId, ExprId, Term};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde_json::json;
use std::collections::HashSet;
use std::fmt::Write as _;
use std::time::Instant;

const SCORES: [&str; 4] = ["symmetric FAC (SAGE default)", "coverage S(g→x)/S(g→g)", "core fraction (p ≥ 0.7)", "√(core × symmetric)"];

pub fn run(args: &Args) -> Result<(), String> {
    let seed = args.u64("seed", 1);
    let per = args.usize("per", 10);
    let sev = args.usize("severity", 1);
    let core_p = args.f64("core", 0.7);
    let templates: Vec<usize> = args.str("templates", "100,1000").split(',').map(|x| x.parse().unwrap()).collect();
    let out_dir = args.str("out", "results/E22");
    let t0 = Instant::now();
    let mut md = String::new();
    writeln!(md, "# E22: what score identifies membership in a schema?\n").unwrap();
    writeln!(md, "Level-1 SAGE schemas over {per} noisy instances per template (severity {sev}; θ = 0.4 + sleep; seed {seed}). Test cases: one fresh instance of each stored template (≤ 500) and 250 instances of absent templates. Each score picks the best of the fingerprint top-16 schemas (≥ 2 members); the decision is *correct* if that schema's majority template is the case's template. Recall = correct acceptances / present-template cases. Core = schema facts with probability ≥ {core_p}.\n").unwrap();
    let mut rows = Vec::new();
    for &n_t in &templates {
        let n_abs = 250usize;
        let gcfg = GenConfig { seed, naming: Naming::Canonical, distractors: 2, perturb_ops: PerturbOp::ALL.to_vec(), severity: sev, ..Default::default() };
        let inst = template_instances(&gcfg, n_t + n_abs, per + 1);
        let mut kb = inst.kb;
        let mut by_t: FxHashMap<usize, Vec<CaseId>> = FxHashMap::default();
        for &(c, t, clean) in &inst.cases {
            if !clean {
                by_t.entry(t).or_default().push(c);
            }
        }
        let mut memory: Vec<(CaseId, usize)> = Vec::new();
        for t in 0..n_t {
            memory.extend(by_t[&t].iter().take(per).map(|&c| (c, t)));
        }
        let mut tests: Vec<(CaseId, usize, bool)> = (0..n_t.min(500)).map(|t| (by_t[&t][per], t, true)).collect();
        tests.extend((n_t..n_t + n_abs).map(|t| (by_t[&t][0], t, false)));
        let mem: Vec<CaseId> = memory.iter().map(|x| x.0).collect();
        let tmpl_of: FxHashMap<CaseId, usize> = memory.iter().copied().collect();
        let stats = stats_for(&kb, &mem);
        let sk = Sketcher::new(Layout::default(), seed ^ 0xF1);
        let mut order = mem.clone();
        Rng::new(seed ^ 0xE15).shuffle(&mut order);
        let mut sage = Sage::new(SageConfig { assimilate: 0.4, namespace: "m22s".into(), ..Default::default() }, stats.clone(), sk.clone(), FeatureConfig::default());
        for &c in &order {
            sage.add(&mut kb, c);
        }
        sage.consolidate(&mut kb, 0.6);
        // Schemas with ≥ 2 members: case, majority template, core facts (with probabilities).
        struct G {
            case: CaseId,
            tmpl: usize,
            core: Vec<ExprId>,
        }
        let gens: Vec<G> = sage
            .gens
            .iter()
            .filter(|g| g.members.len() >= 2)
            .map(|g| {
                let mut counts: FxHashMap<usize, usize> = FxHashMap::default();
                for m in &g.members {
                    *counts.entry(tmpl_of[m]).or_insert(0) += 1;
                }
                let tmpl = counts.into_iter().max_by_key(|x| (x.1, std::cmp::Reverse(x.0))).unwrap().0;
                let n = g.members.len() as f64;
                let in_case: HashSet<ExprId> = kb.case(g.case).facts.iter().copied().collect();
                let core = g.facts.iter().filter(|(f, c)| *c as f64 / n >= core_p && in_case.contains(f)).map(|x| x.0).collect();
                G { case: g.case, tmpl, core }
            })
            .collect();
        let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
        let fp = |c: CaseId| {
            let mut f = fx.extract(c);
            stats.apply(&mut f, &[true; N_CHANNELS]);
            sk.sketch(&f)
        };
        let gfp: Vec<_> = gens.par_iter().map(|g| fp(g.case)).collect();
        let mp = Mapper::new(&kb, MapConfig::default());
        let gself: Vec<f64> = gens.par_iter().map(|g| mp.score(g.case, g.case) as f64).collect();
        let prof = Profile::analogy();
        // Per test: per score, (best score value, correct).
        let decisions: Vec<([(f64, bool); 4], bool)> = tests
            .par_iter()
            .map(|&(x, t, present)| {
                let xf = fp(x);
                let xs = mp.score(x, x) as f64;
                let mut c: Vec<(usize, f64)> = (0..gens.len()).map(|i| (i, prof.score(&sk.channel_sims(&xf, &gfp[i])))).collect();
                c.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                c.truncate(16);
                let vals: Vec<(usize, [f64; 4])> = c
                    .iter()
                    .map(|&(i, _)| {
                        let g = &gens[i];
                        let m = mp.best(g.case, x);
                        let raw = m.as_ref().map(|m| m.score as f64).unwrap_or(0.0);
                        let sym = if raw == 0.0 { 0.0 } else { (raw / (gself[i] * xs).sqrt()).min(1.0) };
                        let cov = if raw == 0.0 { 0.0 } else { (raw / gself[i]).min(1.0) };
                        let matched: HashSet<ExprId> = m.map(|m| m.correspondences.iter().filter_map(|(b, _)| if let Term::Expr(e) = b { Some(*e) } else { None }).collect()).unwrap_or_default();
                        let core = if g.core.is_empty() { 0.0 } else { g.core.iter().filter(|f| matched.contains(f)).count() as f64 / g.core.len() as f64 };
                        (i, [sym, cov, core, (core * sym).sqrt()])
                    })
                    .collect();
                let per_score = std::array::from_fn(|k| {
                    let best = vals.iter().max_by(|a, b| a.1[k].total_cmp(&b.1[k]).then(b.0.cmp(&a.0))).map(|v| (v.1[k], gens[v.0].tmpl == t && present)).unwrap_or((0.0, false));
                    best
                });
                (per_score, present)
            })
            .collect();
        let n_present = decisions.iter().filter(|d| d.1).count().max(1) as f64;
        writeln!(md, "\n## T = {n_t} templates ({} memory cases; {} schemas with ≥ 2 members; {} tests)\n", mem.len(), gens.len(), tests.len()).unwrap();
        writeln!(md, "| score | best-schema correct (any threshold) | AUC correct vs wrong | recall at precision ≥ 0.9 | recall at precision ≥ 0.95 |\n|---|---|---|---|---|").unwrap();
        for (k, name) in SCORES.iter().enumerate() {
            let mut v: Vec<(f64, bool)> = decisions.iter().map(|d| d.0[k]).collect();
            let ok = v.iter().filter(|x| x.1).count() as f64 / n_present;
            let pos: Vec<f64> = v.iter().filter(|x| x.1).map(|x| x.0).collect();
            let neg: Vec<f64> = v.iter().filter(|x| !x.1).map(|x| x.0).collect();
            let a = auc(&pos, &neg);
            v.sort_by(|a, b| b.0.total_cmp(&a.0));
            let (mut tp, mut r90, mut r95) = (0usize, 0.0f64, 0.0f64);
            for (i, &(s, c)) in v.iter().enumerate() {
                tp += c as usize;
                if v.get(i + 1).map(|x| x.0 == s).unwrap_or(false) {
                    continue;
                }
                let p = tp as f64 / (i + 1) as f64;
                let r = tp as f64 / n_present;
                if p >= 0.9 {
                    r90 = r90.max(r);
                }
                if p >= 0.95 {
                    r95 = r95.max(r);
                }
            }
            writeln!(md, "| {name} | {ok:.3} | {a:.3} | {r90:.3} | {r95:.3} |").unwrap();
            rows.push(json!({"templates": n_t, "score": name, "best_correct": ok, "auc": a, "recall_p90": r90, "recall_p95": r95}));
        }
        let core_sizes: Vec<f64> = gens.iter().map(|g| g.core.len() as f64).collect();
        writeln!(md, "\nMean core size {:.1} facts ({:.1?}).", crate::metrics::mean(&core_sizes), t0.elapsed()).unwrap();
        eprintln!("[e22] T={n_t} done ({:.1?})", t0.elapsed());
    }
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let tag = args.str("tag", &format!("s{sev}"));
    std::fs::write(format!("{out_dir}/E22-{tag}.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E22-{tag}.json"), serde_json::to_string_pretty(&json!({"seed": seed, "per": per, "core_p": core_p, "rows": rows})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
