//! E16: is this analogy real? Match significance as memory grows.
//!
//! A raw similarity threshold means different things at different memory
//! sizes: the best chance match among N items rises with N (E10, E15). Here
//! the task is to decide whether the top retrieved analogue is real (same
//! hidden template as the query) or should be rejected. Half of the queries
//! have their template in memory, half do not (open set).
//!
//! Scores for the top-1 item (after the fused re-rank of the fingerprint
//! top-k):
//! * raw: fused score, normalized FAC, fingerprint score, fused margin to #2;
//! * **global null** (fingerprint): the query's fingerprint scores against a
//!   random sample of memory give μ, σ; E = N·Q((fp₁ − μ)/σ) is the expected
//!   number of chance matches at least as good in a memory of N items;
//!   score = −log₁₀ E;
//! * **local null** (fused): z of the top-1 fused score against the fused
//!   scores of the fingerprint ranks k/2..k (the tail of the candidate list);
//!   and the same with fingerprint scores only (no FAC needed).
//!
//! A threshold is fixed at the smallest memory for precision ≥ 0.9 and
//! reused unchanged at 10× and 100× the memory.

use crate::metrics::{auc, mean, std};
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Layout, Profile, Sketcher, N_CHANNELS};
use mars_gen::{template_instances, GenConfig, Naming, PerturbOp};
use mars_hv::Rng;
use mars_index::ModeK;
use mars_map::{MapConfig, Mapper};
use mars_rel::CaseId;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde_json::json;
use std::fmt::Write as _;
use std::time::Instant;

const SCORES: [&str; 7] = ["fused score", "normalized FAC", "fingerprint score", "fused margin to #2", "fingerprint −log₁₀E (global null)", "fused z (local null)", "fingerprint z (local null, FAC-free)"];

/// ln Q(z) for the standard normal upper tail, via erfc (Numerical Recipes
/// `erfcc`, relative error < 1.2e-7 everywhere, so usable deep in the tail).
fn ln_q(z: f64) -> f64 {
    let x = z / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.5 * x.abs());
    let poly = -x * x - 1.26551223 + t * (1.00002368 + t * (0.37409196 + t * (0.09678418 + t * (-0.18628806 + t * (0.27886807 + t * (-1.13520398 + t * (1.48851587 + t * (-0.82215223 + t * 0.17087277))))))));
    if x >= 0.0 {
        (0.5 * t).ln() + poly
    } else {
        (1.0 - 0.5 * t * poly.exp()).ln()
    }
}

struct Obs {
    correct: bool,
    present: bool,
    s: [f64; 7],
}

fn run_size(args: &Args, n_mem_t: usize, per: usize, n_q: usize, k: usize) -> (Vec<Obs>, usize, f64) {
    let seed = args.u64("seed", 1);
    let t0 = Instant::now();
    let n_neg_t = n_q / 2;
    let cfg = GenConfig { seed, naming: Naming::Canonical, distractors: 2, perturb_ops: PerturbOp::ALL.to_vec(), severity: args.usize("severity", 1), ..Default::default() };
    let inst = template_instances(&cfg, n_mem_t + n_neg_t, per + 1);
    let kb = inst.kb;
    let mut by_t: FxHashMap<usize, Vec<CaseId>> = FxHashMap::default();
    for &(c, t, clean) in &inst.cases {
        if !clean {
            by_t.entry(t).or_default().push(c);
        }
    }
    let mut memory: Vec<(CaseId, usize)> = Vec::new();
    for t in 0..n_mem_t {
        memory.extend(by_t[&t].iter().take(per).map(|&c| (c, t)));
    }
    let mut rng = Rng::new(seed ^ 0xE16);
    let mut queries: Vec<(CaseId, usize, bool)> = Vec::new();
    for _ in 0..n_q - n_neg_t {
        let t = rng.below(n_mem_t as u64) as usize;
        queries.push((by_t[&t][per], t, true));
    }
    for t in n_mem_t..n_mem_t + n_neg_t {
        queries.push((by_t[&t][0], t, false));
    }
    let n = memory.len();
    let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
    let mut feats: Vec<_> = memory.par_iter().map(|&(c, _)| fx.extract(c)).collect();
    let stats = FeatureStats::fit(feats.iter());
    feats.par_iter_mut().for_each(|f| stats.apply(f, &[true; N_CHANNELS]));
    let sk = Sketcher::new(Layout::default(), seed ^ 0xF1);
    let fps: Vec<Vec<u64>> = feats.par_iter().map(|f| sk.sketch(f).into_words()).collect();
    drop(feats);
    let qfps: Vec<Vec<u64>> = queries
        .par_iter()
        .map(|&(c, _, _)| {
            let mut f = fx.extract(c);
            stats.apply(&mut f, &[true; N_CHANNELS]);
            sk.sketch(&f).into_words()
        })
        .collect();
    let mut idx = ModeK::with_capacity(sk.layout.clone(), n);
    for f in &fps {
        idx.push(f);
    }
    let scorer = idx.scorer(&Profile::analogy().weights);
    let lists = idx.search_batch(&qfps.iter().map(|v| v.as_slice()).collect::<Vec<_>>(), &scorer, k);
    let sample: Vec<u32> = (0..2048.min(n)).map(|_| rng.below(n as u64) as u32).collect();
    let mapper = Mapper::new(&kb, MapConfig::default());
    let obs: Vec<Obs> = queries
        .par_iter()
        .enumerate()
        .map(|(qi, &(q, t, present))| {
            let null: Vec<f64> = sample.iter().map(|&r| idx.score(&scorer, &qfps[qi], r) as f64).collect();
            let (mu, sd) = (mean(&null), std(&null).max(1e-9));
            let qs = mapper.score(q, q) as f64;
            let mut cands: Vec<(usize, f64, f64, f64)> = lists[qi]
                .iter()
                .map(|h| {
                    let c = memory[h.id as usize].0;
                    let raw = mapper.score(c, q) as f64;
                    let cs = mapper.score(c, c) as f64;
                    let fac = if raw == 0.0 { 0.0 } else { (raw / (qs * cs).sqrt()).min(1.0) };
                    (h.id as usize, 0.3 * fac + 0.7 * h.score as f64, fac, h.score as f64)
                })
                .collect();
            let tail: Vec<f64> = cands[cands.len() / 2..].iter().map(|x| x.1).collect();
            let (tm, ts) = (mean(&tail), std(&tail).max(1e-9));
            let ftail: Vec<f64> = cands[cands.len() / 2..].iter().map(|x| x.3).collect();
            let (fm, fs) = (mean(&ftail), std(&ftail).max(1e-9));
            cands.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let top = cands[0];
            let second = cands.get(1).map(|x| x.1).unwrap_or(0.0);
            let z = (top.3 - mu) / sd;
            let neg_log10_e = -((n as f64).ln() + ln_q(z)) / std::f64::consts::LN_10;
            Obs { correct: memory[top.0].1 == t, present, s: [top.1, top.2, top.3, top.1 - second, neg_log10_e, (top.1 - tm) / ts, (top.3 - fm) / fs] }
        })
        .collect();
    (obs, n, t0.elapsed().as_secs_f64())
}

/// Lowest threshold with precision ≥ `target` among accepted (score ≥ τ).
fn threshold_for(obs: &[Obs], si: usize, target: f64) -> f64 {
    let mut v: Vec<(f64, bool)> = obs.iter().map(|o| (o.s[si], o.correct)).collect();
    v.sort_by(|a, b| b.0.total_cmp(&a.0));
    let (mut tp, mut best) = (0usize, f64::INFINITY);
    for (i, &(s, c)) in v.iter().enumerate() {
        tp += c as usize;
        let last_of_tie = v.get(i + 1).map(|x| x.0 != s).unwrap_or(true);
        if last_of_tie && tp as f64 / (i + 1) as f64 >= target {
            best = s;
        }
    }
    best
}

fn at(obs: &[Obs], si: usize, tau: f64) -> (f64, f64, f64) {
    let acc: Vec<&Obs> = obs.iter().filter(|o| o.s[si] >= tau).collect();
    let prec = acc.iter().filter(|o| o.correct).count() as f64 / acc.len().max(1) as f64;
    let n_correct = obs.iter().filter(|o| o.correct).count().max(1) as f64;
    let rec = acc.iter().filter(|o| o.correct).count() as f64 / n_correct;
    let absent = obs.iter().filter(|o| !o.present).count().max(1) as f64;
    let false_absent = acc.iter().filter(|o| !o.present).count() as f64 / absent;
    (prec, rec, false_absent)
}

pub fn run(args: &Args) -> Result<(), String> {
    let per = args.usize("per", 5);
    let n_q = args.usize("queries", 500);
    let k = args.usize("k", 64);
    let target = args.f64("precision", 0.9);
    let sizes: Vec<usize> = args.str("templates", "200,2000,20000").split(',').map(|x| x.parse().unwrap()).collect();
    let out_dir = args.str("out", "results/E16");
    let mut md = String::new();
    writeln!(md, "# E16: is this analogy real? Match significance vs memory size\n").unwrap();
    writeln!(md, "Memory: {per} noisy instances per template (severity {}). {n_q} queries: half are fresh instances of stored templates, half of templates absent from memory. Retrieval: fingerprint top-{k}, fused re-rank. The top-1 is *correct* if it shares the query's template. Each score is used to accept or reject the top-1.\n", args.usize("severity", 1)).unwrap();
    let mut all: Vec<(usize, Vec<Obs>)> = Vec::new();
    for &t in &sizes {
        let (obs, n, secs) = run_size(args, t, per, n_q, k);
        eprintln!("[e16] N={n}: top-1 correct {:.3} ({secs:.1}s)", obs.iter().filter(|o| o.correct).count() as f64 / obs.len() as f64);
        all.push((n, obs));
    }
    writeln!(md, "## Separation (AUC: correct vs wrong top-1)\n").unwrap();
    write!(md, "| score |").unwrap();
    for (n, obs) in &all {
        write!(md, " N = {n} (top-1 correct {:.2}) |", obs.iter().filter(|o| o.correct).count() as f64 / obs.len() as f64).unwrap();
    }
    writeln!(md, "\n|---|{}", "---|".repeat(all.len())).unwrap();
    let mut rows = Vec::new();
    for (si, name) in SCORES.iter().enumerate() {
        write!(md, "| {name} |").unwrap();
        let mut aucs = Vec::new();
        for (_, obs) in &all {
            let pos: Vec<f64> = obs.iter().filter(|o| o.correct).map(|o| o.s[si]).collect();
            let neg: Vec<f64> = obs.iter().filter(|o| !o.correct).map(|o| o.s[si]).collect();
            let a = auc(&pos, &neg);
            aucs.push(a);
            write!(md, " {a:.3} |").unwrap();
        }
        writeln!(md).unwrap();
        rows.push(json!({"score": name, "auc": aucs}));
    }
    writeln!(md, "\n## Threshold transfer: τ set at N = {} for precision ≥ {target}, reused unchanged\n", all[0].0).unwrap();
    writeln!(md, "Cells: precision / recall of correct top-1s / fraction of absent-template queries wrongly accepted.\n").unwrap();
    write!(md, "| score | τ |").unwrap();
    for (n, _) in &all {
        write!(md, " N = {n} |").unwrap();
    }
    writeln!(md, "\n|---|---|{}", "---|".repeat(all.len())).unwrap();
    for (si, name) in SCORES.iter().enumerate() {
        let tau = threshold_for(&all[0].1, si, target);
        write!(md, "| {name} | {tau:.3} |").unwrap();
        let mut cells = Vec::new();
        for (_, obs) in &all {
            let (p, r, fa) = at(obs, si, tau);
            write!(md, " {p:.3} / {r:.3} / {fa:.3} |").unwrap();
            cells.push(json!({"precision": p, "recall": r, "false_accept_absent": fa}));
        }
        writeln!(md).unwrap();
        rows[si]["transfer"] = json!({"tau": tau, "cells": cells});
    }
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let tag = args.str("tag", &format!("p{per}-s{}", args.usize("severity", 1)));
    std::fs::write(format!("{out_dir}/E16-{tag}.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E16-{tag}.json"), serde_json::to_string_pretty(&json!({"sizes": all.iter().map(|x| x.0).collect::<Vec<_>>(), "scores": rows})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ln_q;

    #[test]
    fn normal_tail() {
        assert!((ln_q(0.0).exp() - 0.5).abs() < 1e-6);
        assert!((ln_q(1.959964).exp() - 0.025).abs() < 1e-6);
        assert!((ln_q(-1.959964).exp() - 0.975).abs() < 1e-6);
        // Deep tail: Q(10) ≈ 7.62e-24.
        assert!((ln_q(10.0) - (7.6198530e-24f64).ln()).abs() < 1e-4);
    }
}
