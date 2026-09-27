//! E2: mapper correctness and cost on generated groups.
//!
//! * Entity-correspondence precision/recall vs ground truth (base→LS, base→TA).
//! * FAC as a discriminator: normalized structural score ranks TA above MA/FOR?
//! * Greedy vs optimal (branch-and-bound) merge gap.
//! * Candidate inferences: delete one fact from TA, check the base→TA⁻ mapping
//!   projects it back (recall), and how many structural inferences are true.
//! * Time per pair vs case size.

use crate::metrics::{mean, win_rate};
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Layout, Profile, Sketcher, N_CHANNELS};
use mars_gen::{generate, Dataset, Family, GenConfig, Naming, VariantClass};
use mars_hv::Rng;
use mars_map::{Grounding, MapConfig, Mapper, Mapping};
use mars_rel::{CaseId, CaseKind, Kb, PredKind};
use rayon::prelude::*;
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

fn correspondence_pr(ds: &Dataset, base_item: usize, other_item: usize, m: Option<&Mapping>) -> (f64, f64) {
    let b = ds.item(base_item);
    let o = ds.item(other_item);
    let Some(m) = m else { return (0.0, 0.0) };
    let em = m.entity_map();
    let n_vars = b.entities.len();
    let (mut mapped, mut correct) = (0usize, 0usize);
    for v in 0..n_vars {
        if let Some(t) = em.get(&b.entities[v]) {
            mapped += 1;
            if *t == o.entities[v] {
                correct += 1;
            }
        }
    }
    let p = if mapped == 0 { 0.0 } else { correct as f64 / mapped as f64 };
    (p, correct as f64 / n_vars as f64)
}

pub fn run(args: &Args) -> Result<(), String> {
    let groups = args.usize("groups", 1000);
    let seed = args.u64("seed", 1);
    let distractors = args.usize("distractors", 2);
    let (ops, severity) = args.perturbation();
    let out_dir = args.str("out", "results/E2");
    let tag = args.str("tag", &format!("d{distractors}"));
    let t0 = Instant::now();
    let gcfg = GenConfig { seed, n_groups: groups, naming: Naming::Canonical, distractors, perturb_ops: ops, severity, ..Default::default() };
    let mut ds = generate(&gcfg);

    // TA⁻: TA with one template fact removed (prefer higher-order facts).
    let mut ta_minus: Vec<(CaseId, String)> = Vec::new();
    for gi in 0..ds.groups.len() {
        let ta_case = ds.case(ds.groups[gi].ta);
        let kb: &Kb = &ds.kb;
        let facts = kb.case(ta_case).facts.clone();
        // Only root facts: a fact nested inside another fact would still be present.
        let nested: std::collections::HashSet<_> = kb.case_exprs(ta_case).into_iter().flat_map(|e| kb.expr(e).args.iter().filter_map(|a| if let mars_rel::Term::Expr(c) = *a { Some(c) } else { None }).collect::<Vec<_>>()).collect();
        let ho: Vec<_> = facts.iter().copied().filter(|&f| !nested.contains(&f) && kb.order(f) >= 2 && kb.vocab.kind(kb.expr(f).functor) != PredKind::Attribute).collect();
        let pool = if ho.is_empty() { facts.clone() } else { ho };
        let mut rng = Rng::derive(seed ^ 0xE2, gi as u64);
        let removed = pool[rng.index(pool.len())];
        let removed_text = kb.render_expr(removed);
        let kept: Vec<_> = facts.into_iter().filter(|&f| f != removed).collect();
        let id = ds.kb.add_case(&format!("g{gi}-TAminus"), CaseKind::Episode, kept);
        ta_minus.push((id, removed_text));
    }
    let ds = ds; // freeze
    let mapper = Mapper::new(&ds.kb, MapConfig::default());
    // Fingerprints (analogy profile) for fused MAC+FAC scoring.
    let fx = FeatureExtractor::new(&ds.kb, FeatureConfig::default());
    let mut feats: Vec<_> = (0..ds.kb.n_cases()).into_par_iter().map(|c| fx.extract(CaseId(c as u32))).collect();
    let stats = FeatureStats::fit(feats.iter());
    feats.par_iter_mut().for_each(|f| stats.apply(f, &[true; N_CHANNELS]));
    let sk = Sketcher::new(Layout::default(), seed ^ 0xF1);
    let fps: Vec<_> = feats.par_iter().map(|f| sk.sketch(f)).collect();
    let analogy = Profile::analogy();
    let fp_score = |a: CaseId, b: CaseId| analogy.score(&sk.channel_sims(&fps[a.0 as usize], &fps[b.0 as usize]));
    eprintln!("[e2] generated {} cases ({:.2?})", ds.kb.n_cases(), t0.elapsed());

    struct Row {
        family: Family,
        ls_p: f64,
        ls_r: f64,
        ta_p: f64,
        ta_r: f64,
        norm: [(VariantClass, f64); 4],
        fp: [(VariantClass, f64); 4],
        disc: bool,
        greedy: f64,
        optimal: Option<f64>,
        ci_recall: bool,
        ci_struct_total: usize,
        ci_struct_correct: usize,
        pair_us: Vec<(usize, f64)>,
    }

    let rows: Vec<Row> = (0..ds.groups.len())
        .into_par_iter()
        .map(|gi| {
            let g = &ds.groups[gi];
            let base = ds.case(g.base);
            let mut pair_us = Vec::new();
            let mut timed = |b: CaseId, t: CaseId| -> Option<Mapping> {
                let s = Instant::now();
                let m = mapper.best(b, t);
                let size = ds.kb.case_exprs(b).len() + ds.kb.case_exprs(t).len();
                pair_us.push((size, s.elapsed().as_secs_f64() * 1e6));
                m
            };
            let m_ls = timed(base, ds.case(g.ls));
            let m_ta = timed(base, ds.case(g.ta));
            let (ls_p, ls_r) = correspondence_pr(&ds, g.base, g.ls, m_ls.as_ref());
            let (ta_p, ta_r) = correspondence_pr(&ds, g.base, g.ta, m_ta.as_ref());
            let self_b = mapper.score(base, base) as f64;
            let norm_of = |i: usize| -> f64 {
                let c = ds.case(i);
                let s = mapper.score(base, c) as f64;
                let sc = mapper.score(c, c) as f64;
                if s == 0.0 { 0.0 } else { s / (self_b * sc).sqrt() }
            };
            let norm = [
                (VariantClass::TA, norm_of(g.ta)),
                (VariantClass::MA, norm_of(g.ma)),
                (VariantClass::FOR, norm_of(g.for_)),
                (VariantClass::RND, g.rnd.iter().map(|&r| norm_of(r)).fold(0.0, f64::max)),
            ];
            let fpv = |i: usize| fp_score(base, ds.case(i));
            let fp = [
                (VariantClass::TA, fpv(g.ta)),
                (VariantClass::MA, fpv(g.ma)),
                (VariantClass::FOR, fpv(g.for_)),
                (VariantClass::RND, g.rnd.iter().map(|&r| fpv(r)).fold(f64::MIN, f64::max)),
            ];
            let disc = g.discriminable(&ds);
            let ms = mapper.match_set(base, ds.case(g.ta));
            let greedy = m_ta.as_ref().map(|m| m.score as f64).unwrap_or(0.0);
            let optimal = mapper.optimal_score(&ms, 22).map(|x| x as f64);
            // Candidate inferences on TA⁻.
            let (tm, removed) = &ta_minus[gi];
            let m_tm = timed(base, *tm);
            let (mut ci_recall, mut tot, mut cor) = (false, 0, 0);
            if let Some(m) = &m_tm {
                let ta_full: std::collections::HashSet<String> = ds.kb.case(ds.case(g.ta)).facts.iter().map(|&f| ds.kb.render_expr(f)).collect();
                for inf in &m.inferences {
                    let text = mapper.render_proj(&inf.projected);
                    if &text == removed {
                        ci_recall = true;
                    }
                    if inf.grounding == Grounding::Structural {
                        tot += 1;
                        if ta_full.contains(&text) {
                            cor += 1;
                        }
                    }
                }
            }
            Row { family: g.family, ls_p, ls_r, ta_p, ta_r, norm, fp, disc, greedy, optimal, ci_recall, ci_struct_total: tot, ci_struct_correct: cor, pair_us }
        })
        .collect();

    type Split = (String, Box<dyn Fn(&Row) -> bool>);
    let fams: Vec<Split> = vec![
        ("all".into(), Box::new(|_| true)),
        ("dev families".into(), Box::new(|r: &Row| !r.family.is_test())),
        ("test families".into(), Box::new(|r: &Row| r.family.is_test())),
        ("discriminable".into(), Box::new(|r: &Row| r.disc)),
    ];
    let top_of = |rs: &[&Row], f: &dyn Fn(&Row, VariantClass) -> f64| -> f64 {
        win_rate(&rs.iter().map(|r| (f(r, VariantClass::TA), f(r, VariantClass::MA).max(f(r, VariantClass::FOR)).max(f(r, VariantClass::RND)))).collect::<Vec<_>>())
    };
    let fac = |r: &Row, c: VariantClass| r.norm.iter().find(|x| x.0 == c).unwrap().1;
    let fpc = |r: &Row, c: VariantClass| r.fp.iter().find(|x| x.0 == c).unwrap().1;
    let fused = |r: &Row, c: VariantClass| 0.5 * fac(r, c) + 0.5 * fpc(r, c);
    let mut md = String::new();
    writeln!(md, "# E2 — mapper correctness ({tag})\n").unwrap();
    writeln!(md, "Config: groups={groups}, seed={seed}, distractors={distractors}, MapConfig::default(). Runtime {:.1?}.\n", t0.elapsed()).unwrap();
    writeln!(md, "| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |").unwrap();
    writeln!(md, "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|").unwrap();
    let mut jsplits = Vec::new();
    for (name, keep) in &fams {
        let rs: Vec<&Row> = rows.iter().filter(|r| keep(r)).collect();
        let (fp_top, fused_top) = (top_of(&rs, &fpc), top_of(&rs, &fused));
        let col = |f: &dyn Fn(&Row) -> f64| mean(&rs.iter().map(|r| f(r)).collect::<Vec<_>>());
        let get = |r: &Row, c: VariantClass| r.norm.iter().find(|x| x.0 == c).unwrap().1;
        let w_ma = win_rate(&rs.iter().map(|r| (get(r, VariantClass::TA), get(r, VariantClass::MA))).collect::<Vec<_>>());
        let w_for = win_rate(&rs.iter().map(|r| (get(r, VariantClass::TA), get(r, VariantClass::FOR))).collect::<Vec<_>>());
        let top = win_rate(
            &rs.iter()
                .map(|r| (get(r, VariantClass::TA), get(r, VariantClass::MA).max(get(r, VariantClass::FOR)).max(get(r, VariantClass::RND))))
                .collect::<Vec<_>>(),
        );
        let gaps: Vec<f64> = rs.iter().filter_map(|r| r.optimal.map(|o| if o == 0.0 { 1.0 } else { r.greedy / o })).collect();
        let ci_rec = col(&|r| r.ci_recall as u8 as f64);
        let (t, c): (usize, usize) = rs.iter().fold((0, 0), |a, r| (a.0 + r.ci_struct_total, a.1 + r.ci_struct_correct));
        let ci_prec = if t == 0 { f64::NAN } else { c as f64 / t as f64 };
        writeln!(
            md,
            "| {name} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.4} (n={}) | {:.3} | {:.3} |",
            rs.len(), col(&|r| r.ls_p), col(&|r| r.ls_r), col(&|r| r.ta_p), col(&|r| r.ta_r), w_ma, w_for, top, fp_top, fused_top, mean(&gaps), gaps.len(), ci_rec, ci_prec
        )
        .unwrap();
        jsplits.push(json!({"split": name, "ls_p": col(&|r| r.ls_p), "ls_r": col(&|r| r.ls_r), "ta_p": col(&|r| r.ta_p), "ta_r": col(&|r| r.ta_r),
            "n": rs.len(), "fac_win_ma": w_ma, "fac_win_for": w_for, "fac_ta_top": top, "fp_ta_top": fp_top, "fused_ta_top": fused_top, "greedy_over_optimal": mean(&gaps), "n_optimal": gaps.len(),
            "ci_recall": ci_rec, "ci_struct_precision": ci_prec}));
    }
    // Per-family TA correspondence recall.
    writeln!(md, "\n| family | TA corr. R | FAC TA-top | CI recall |").unwrap();
    writeln!(md, "|---|---|---|---|").unwrap();
    for f in Family::ALL {
        let rs: Vec<&Row> = rows.iter().filter(|r| r.family == f).collect();
        let get = |r: &Row, c: VariantClass| r.norm.iter().find(|x| x.0 == c).unwrap().1;
        let top = win_rate(&rs.iter().map(|r| (get(r, VariantClass::TA), get(r, VariantClass::MA).max(get(r, VariantClass::FOR)).max(get(r, VariantClass::RND)))).collect::<Vec<_>>());
        writeln!(md, "| {} | {:.3} | {:.3} | {:.3} |", f.name(), mean(&rs.iter().map(|r| r.ta_r).collect::<Vec<_>>()), top, mean(&rs.iter().map(|r| r.ci_recall as u8 as f64).collect::<Vec<_>>())).unwrap();
    }
    // Timing by pair size.
    let mut all_t: Vec<(usize, f64)> = rows.iter().flat_map(|r| r.pair_us.iter().copied()).collect();
    all_t.sort_by_key(|x| x.0);
    writeln!(md, "\n## Time per mapping (single thread, µs)\n\n| exprs in pair | n | mean µs | p95 µs |\n|---|---|---|---|").unwrap();
    let mut jt = Vec::new();
    for (lo, hi) in [(0, 20), (20, 30), (30, 40), (40, 60), (60, 1000)] {
        let mut v: Vec<f64> = all_t.iter().filter(|x| x.0 >= lo && x.0 < hi).map(|x| x.1).collect();
        if v.is_empty() {
            continue;
        }
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95 = v[((v.len() as f64 * 0.95) as usize).min(v.len() - 1)];
        writeln!(md, "| {lo}–{hi} | {} | {:.1} | {:.1} |", v.len(), mean(&v), p95).unwrap();
        jt.push(json!({"lo": lo, "hi": hi, "n": v.len(), "mean_us": mean(&v), "p95_us": p95}));
    }
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E2-{tag}.md"), &md).map_err(|e| e.to_string())?;
    let j = json!({"experiment": "E2", "tag": tag, "gen_config": gcfg, "splits": jsplits, "timing": jt, "runtime_s": t0.elapsed().as_secs_f64()});
    std::fs::write(format!("{out_dir}/E2-{tag}.json"), serde_json::to_string_pretty(&j).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
