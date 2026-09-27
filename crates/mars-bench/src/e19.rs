//! E19: multi-view cases — fusing the nested and flat views of code.
//!
//! E18 found that the nested view (wrapped statements) has the stronger
//! fingerprint and restores more statements, while the flat view (block
//! entities) has the stronger mapper and places statements better. Here
//! both views of every function are kept and combined:
//!
//! * retrieval: single views at their best weights (nested 0.3, flat 0.7),
//!   the a-priori cross-view score ½·FAC(flat) + ½·FP(nested) (each view's
//!   stronger signal), the average of the two views' fused scores, and
//!   reciprocal-rank fusion (RRF, k = 60) of the two view rankings —
//!   on E9 task A (Python, other package) and E12 (other language);
//! * inference (E17 queries): candidate statements from each view's top-1
//!   analogue, their union, and *cross-view agreement* (a core statement
//!   proposed by both views) as a corroboration signal, compared with
//!   within-view support over the top-5.

use crate::e17::entities;
use crate::e18::{build_rep, proj_core, Rep};
use crate::e9::{load_corpus, Meta};
use crate::metrics::{mrr, recall_at};
use crate::Args;
use mars_encode::Profile;
use mars_hv::Rng;
use mars_map::{Grounding, MapConfig, Mapper};
use mars_rel::views::{FlatView, CODE_WRAPPERS};
use mars_rel::{CaseId, CaseKind, ExprId, Kb};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde_json::json;
use std::collections::HashSet;
use std::fmt::Write as _;
use std::time::Instant;

const METHODS: [&str; 7] = [
    "nested fused 0.3 (E9)",
    "flat fused 0.7 (E18)",
    "FP nested only",
    "FAC flat only",
    "**cross-view ½·FAC(flat) + ½·FP(nested)**",
    "view average ½·nested + ½·flat",
    "RRF(nested, flat)",
];

/// Signals of candidate c for query q: [FP nested, FAC nested, FP flat, FAC flat].
fn signals(kb: &Kb, n: &Rep, f: &Rep, mapper: &Mapper, q: usize, c: usize) -> [f64; 4] {
    let lit = Profile::literal();
    let fac = |r: &Rep| {
        let s = mapper.score(r.cases[c], r.cases[q]) as f64;
        if s == 0.0 {
            0.0
        } else {
            (s / (r.self_s[q] * r.self_s[c]).sqrt()).min(1.0)
        }
    };
    let _ = kb;
    [lit.score(&n.sk.channel_sims(&n.fps[q], &n.fps[c])), fac(n), lit.score(&f.sk.channel_sims(&f.fps[q], &f.fps[c])), fac(f)]
}

/// Scores of every method for a list of candidates' signals (RRF needs the whole list).
fn method_scores(sig: &[[f64; 4]]) -> Vec<[f64; 7]> {
    let nf: Vec<f64> = sig.iter().map(|s| 0.3 * s[1] + 0.7 * s[0]).collect();
    let ff: Vec<f64> = sig.iter().map(|s| 0.7 * s[3] + 0.3 * s[2]).collect();
    let rank_of = |v: &[f64]| {
        let mut o: Vec<usize> = (0..v.len()).collect();
        o.sort_by(|&a, &b| v[b].total_cmp(&v[a]).then(a.cmp(&b)));
        let mut r = vec![0usize; v.len()];
        for (i, &x) in o.iter().enumerate() {
            r[x] = i;
        }
        r
    };
    let (rn, rf) = (rank_of(&nf), rank_of(&ff));
    (0..sig.len())
        .map(|i| {
            let s = sig[i];
            [nf[i], ff[i], s[0], s[3], 0.5 * s[3] + 0.5 * s[0], 0.5 * nf[i] + 0.5 * ff[i], 1.0 / (60.0 + rn[i] as f64) + 1.0 / (60.0 + rf[i] as f64)]
        })
        .collect()
}

/// Retrieval of the same algorithm (other package or other language), ranked among all functions.
fn retrieval(kb: &Kb, metas: &[Meta], n: &Rep, f: &Rep, cross_lang: bool) -> (usize, Vec<(f64, f64, f64)>) {
    let rel = |q: usize, c: usize| metas[c].is_main && metas[c].norm == metas[q].norm && if cross_lang { metas[c].lang != metas[q].lang } else { metas[c].pkg != metas[q].pkg };
    let queries: Vec<usize> = (0..metas.len()).filter(|&q| metas[q].is_main && (0..metas.len()).any(|c| c != q && rel(q, c))).collect();
    let mapper = Mapper::new(kb, MapConfig::default());
    let ranks: Vec<[usize; 7]> = queries
        .par_iter()
        .map(|&q| {
            let cands: Vec<usize> = (0..metas.len()).filter(|&c| c != q).collect();
            let sig: Vec<[f64; 4]> = cands.iter().map(|&c| signals(kb, n, f, &mapper, q, c)).collect();
            let sc = method_scores(&sig);
            std::array::from_fn(|m| {
                let mut o: Vec<usize> = (0..cands.len()).collect();
                o.sort_by(|&a, &b| sc[b][m].total_cmp(&sc[a][m]).then(a.cmp(&b)));
                o.iter().position(|&i| rel(q, cands[i])).unwrap_or(usize::MAX / 2)
            })
        })
        .collect();
    let res = (0..7)
        .map(|m| {
            let r: Vec<usize> = ranks.iter().map(|x| x[m]).collect();
            (recall_at(&r, 1), recall_at(&r, 10), mrr(&r))
        })
        .collect();
    (queries.len(), res)
}

/// Flat view of every function in `metas` (as new cases); and per function the original→flat fact map.
fn flatten_all(kb: &mut Kb, fl: &FlatView, metas: &[Meta], tag: &str) -> (Vec<CaseId>, Vec<FxHashMap<ExprId, ExprId>>) {
    let mut cases = Vec::new();
    let mut maps = Vec::new();
    for (i, m) in metas.iter().enumerate() {
        let (facts, cm) = fl.flatten(kb, m.case, &format!("{tag}{i}"));
        let name = format!("{tag}:{}", kb.name(kb.case(m.case).name));
        cases.push(kb.add_case(&name, CaseKind::Episode, facts));
        maps.push(cm);
    }
    (cases, maps)
}

pub fn run(args: &Args) -> Result<(), String> {
    let dir = args.str("data", "data/e9");
    let seed = args.u64("seed", 1);
    let per_func = args.usize("per-func", 2);
    let k = args.usize("k", 64);
    let out_dir = args.str("out", "results/E19");
    let t0 = Instant::now();
    let (mut kb, all) = load_corpus(&dir)?;
    let fl = FlatView::declare(&mut kb, &CODE_WRAPPERS);
    let (flat_all, _) = flatten_all(&mut kb, &fl, &all, "fa");
    let py: Vec<Meta> = all.iter().filter(|m| m.lang == "py").cloned().collect();
    let (flat_py, core_maps) = flatten_all(&mut kb, &fl, &py, "fp");

    // ------------------------------------------------------------ inference queries (E17/E18 protocol)
    let mut rng = Rng::new(seed ^ 0xE17);
    let mut qs: Vec<(CaseId, CaseId, usize, ExprId)> = Vec::new(); // (nested query, flat query, function, core)
    for (i, m) in py.iter().enumerate() {
        if !m.is_main {
            continue;
        }
        let facts = kb.case(m.case).facts.clone();
        if facts.len() < 4 {
            continue;
        }
        let mut cands: Vec<ExprId> = facts
            .iter()
            .copied()
            .filter(|&f| {
                if kb.order(f) < 2 {
                    return false;
                }
                let (mut mine, mut rest) = (HashSet::new(), HashSet::new());
                entities(&kb, f, &mut mine);
                for &g in &facts {
                    if g != f {
                        entities(&kb, g, &mut rest);
                    }
                }
                mine.is_subset(&rest)
            })
            .collect();
        rng.shuffle(&mut cands);
        for &del in cands.iter().take(per_func) {
            let j = qs.len();
            let kept: Vec<ExprId> = facts.iter().copied().filter(|&f| f != del).collect();
            let qn = kb.add_case(&format!("mvqn{j}"), CaseKind::Query, kept);
            let fdel = core_maps[i][&del];
            let shared = facts.iter().any(|&g| g != del && core_maps[i].get(&g) == Some(&fdel));
            let mut fkept: Vec<ExprId> = kb.case(flat_py[i]).facts.iter().copied().filter(|&f| f != fdel || shared).collect();
            fl.prune(&kb, &mut fkept);
            let qf = kb.add_case(&format!("mvqf{j}"), CaseKind::Query, fkept);
            qs.push((qn, qf, i, fl.peel(&kb, del).1));
        }
    }
    let nq = qs.len();

    // ------------------------------------------------------------ representations
    let rn_all = build_rep(&kb, all.iter().map(|m| m.case).collect());
    let rf_all = build_rep(&kb, flat_all.clone());
    let rn_py = build_rep(&kb, py.iter().map(|m| m.case).collect());
    let rf_py = build_rep(&kb, flat_py.clone());
    eprintln!("[e19] {} functions ({} py), {nq} inference queries ({:.1?})", all.len(), py.len(), t0.elapsed());

    let (nq_a, ret_a) = retrieval(&kb, &py, &rn_py, &rf_py, false);
    let (nq_x, ret_x) = retrieval(&kb, &all, &rn_all, &rf_all, true);
    eprintln!("[e19] retrieval done ({:.1?})", t0.elapsed());

    // ------------------------------------------------------------ inference
    // Query fingerprints in each view need the view's statistics: extend each Rep by the query case.
    let mapper = Mapper::new(&kb, MapConfig::default());
    let lit = Profile::literal();
    let core_text_of = |f: ExprId| kb.render_expr(fl.peel(&kb, f).1);
    let orig_cores: Vec<HashSet<String>> = py
        .iter()
        .map(|m| kb.case(m.case).facts.iter().map(|&f| core_text_of(f)).collect())
        .collect();
    struct QRes {
        /// Per method, per view (0 nested, 1 flat): top-1 proposals (core text, skolem-free) and top-5 core support (nested, method 0 only).
        cores: Vec<[Vec<(String, bool)>; 2]>,
        top5_support: FxHashMap<String, (usize, bool)>,
        /// Top-5 ranked by the flat view (fused 0.7), proposals from the nested view.
        top5_flat_rank: FxHashMap<String, (usize, bool)>,
    }
    let results: Vec<QRes> = qs
        .par_iter()
        .map(|&(qn, qf, orig, _core)| {
            let fpn = rn_py.fp_of(&kb, qn);
            let fpf = rf_py.fp_of(&kb, qf);
            let (sn, sf) = (mapper.score(qn, qn) as f64, mapper.score(qf, qf) as f64);
            // Candidates: union of each view's fingerprint top-k.
            let top = |rep: &Rep, qfp: &mars_hv::HyperVector| {
                let mut v: Vec<(usize, f64)> = (0..rep.cases.len()).filter(|&c| c != orig).map(|c| (c, lit.score(&rep.sk.channel_sims(qfp, &rep.fps[c])))).collect();
                v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                v.into_iter().take(k).map(|x| x.0).collect::<Vec<_>>()
            };
            let mut cands = top(&rn_py, &fpn);
            cands.extend(top(&rf_py, &fpf));
            cands.sort_unstable();
            cands.dedup();
            let sig: Vec<[f64; 4]> = cands
                .iter()
                .map(|&c| {
                    let fac = |rep: &Rep, q: CaseId, qs: f64| {
                        let s = mapper.score(rep.cases[c], q) as f64;
                        if s == 0.0 { 0.0 } else { (s / (qs * rep.self_s[c]).sqrt()).min(1.0) }
                    };
                    [lit.score(&rn_py.sk.channel_sims(&fpn, &rn_py.fps[c])), fac(&rn_py, qn, sn), lit.score(&rf_py.sk.channel_sims(&fpf, &rf_py.fps[c])), fac(&rf_py, qf, sf)]
                })
                .collect();
            let sc = method_scores(&sig);
            let proposals = |rep: &Rep, c: usize, q: CaseId| -> Vec<(String, bool)> {
                let mut v: Vec<(String, bool)> = mapper
                    .best(rep.cases[c], q)
                    .map(|m| m.inferences.iter().filter(|i| i.grounding == Grounding::Structural).map(|i| (mapper.render_proj(proj_core(&fl, &i.projected)), !i.has_skolem)).collect())
                    .unwrap_or_default();
                v.sort();
                v.dedup();
                v
            };
            let order = |m: usize| {
                let mut o: Vec<usize> = (0..cands.len()).collect();
                o.sort_by(|&a, &b| sc[b][m].total_cmp(&sc[a][m]).then(a.cmp(&b)));
                o.into_iter().map(|i| cands[i]).collect::<Vec<_>>()
            };
            let cores = (0..METHODS.len())
                .map(|m| {
                    let best = order(m)[0];
                    [proposals(&rn_py, best, qn), proposals(&rf_py, best, qf)]
                })
                .collect();
            let mut top5_support: FxHashMap<String, (usize, bool)> = FxHashMap::default();
            for c in order(0).into_iter().take(5) {
                for (t, ok) in proposals(&rn_py, c, qn) {
                    top5_support.entry(t).or_insert((0, ok)).0 += 1;
                }
            }
            let mut top5_flat_rank: FxHashMap<String, (usize, bool)> = FxHashMap::default();
            for c in order(1).into_iter().take(5) {
                for (t, ok) in proposals(&rn_py, c, qn) {
                    top5_flat_rank.entry(t).or_insert((0, ok)).0 += 1;
                }
            }
            QRes { cores, top5_support, top5_flat_rank }
        })
        .collect();
    eprintln!("[e19] inference done ({:.1?})", t0.elapsed());

    // Aggregate: (core recall, core precision over skolem-free proposals, proposals / query).
    let agg = |sel: &dyn Fn(&QRes) -> Vec<(String, bool)>| -> (f64, f64, f64) {
        let (mut rec, mut num, mut den, mut per) = (0.0, 0usize, 0usize, 0.0);
        for (r, &(_, _, orig, core)) in results.iter().zip(&qs) {
            let p = sel(r);
            let ct = kb.render_expr(core);
            rec += p.iter().any(|x| x.0 == ct) as u8 as f64;
            for (t, ok) in &p {
                if *ok {
                    den += 1;
                    num += orig_cores[orig].contains(t) as usize;
                }
            }
            per += p.len() as f64;
        }
        let d = nq.max(1) as f64;
        (rec / d, num as f64 / den.max(1) as f64, per / d)
    };
    let union = |a: &[(String, bool)], b: &[(String, bool)]| {
        let mut v: Vec<(String, bool)> = a.iter().chain(b).cloned().collect();
        v.sort();
        v.dedup_by(|x, y| x.0 == y.0);
        v
    };
    let inter = |a: &[(String, bool)], b: &[(String, bool)]| a.iter().filter(|x| b.iter().any(|y| y.0 == x.0)).cloned().collect::<Vec<_>>();

    let mut md = String::new();
    writeln!(md, "# E19: multi-view cases — fusing nested and flat views of code\n").unwrap();
    writeln!(md, "Corpus: {} functions ({} Python). Every function has a nested view (P7a) and a flat-block view (E18). FP = fingerprint (literal profile), FAC = normalized structural score, each computed within its view. Seed {seed}; runtime {:.1?}.\n", all.len(), py.len(), t0.elapsed()).unwrap();
    writeln!(md, "## Retrieval — MRR (R@1 / R@10)\n").unwrap();
    writeln!(md, "| method | same algorithm, other package ({nq_a} Python queries) | same algorithm, other language ({nq_x} queries) |\n|---|---|---|").unwrap();
    let mut jr = Vec::new();
    for (m, name) in METHODS.iter().enumerate() {
        let (a, x) = (ret_a[m], ret_x[m]);
        writeln!(md, "| {name} | {:.3} ({:.2} / {:.2}) | {:.3} ({:.2} / {:.2}) |", a.2, a.0, a.1, x.2, x.0, x.1).unwrap();
        jr.push(json!({"method": name.replace("**", ""), "cross_pkg": {"r1": a.0, "r10": a.1, "mrr": a.2}, "cross_lang": {"r1": x.0, "r10": x.1, "mrr": x.2}}));
    }
    writeln!(md, "\n## Inference — restoring a deleted statement ({nq} queries; core statement level)\n").unwrap();
    writeln!(md, "*recall* = the deleted core statement is proposed; *precision* = skolem-free proposed cores that are statements of the original function.\n").unwrap();
    writeln!(md, "| analogue chosen by | proposals from | recall | precision | proposals / query |\n|---|---|---|---|---|").unwrap();
    let mut ji = Vec::new();
    for (m, label) in [(0usize, "nested fused 0.3"), (1, "flat fused 0.7"), (4, "cross-view ½·FAC(flat) + ½·FP(nested)")] {
        type Select<'a> = Box<dyn Fn(&QRes) -> Vec<(String, bool)> + 'a>;
        let rows: [(&str, Select); 4] = [
            ("nested view", Box::new(move |r: &QRes| r.cores[m][0].clone())),
            ("flat view", Box::new(move |r: &QRes| r.cores[m][1].clone())),
            ("either view (union)", Box::new(move |r: &QRes| union(&r.cores[m][0], &r.cores[m][1]))),
            ("**both views agree**", Box::new(move |r: &QRes| inter(&r.cores[m][0], &r.cores[m][1]))),
        ];
        for (vn, sel) in rows.iter() {
            let (rec, prec, per) = agg(sel.as_ref());
            writeln!(md, "| {label} | {vn} | {rec:.3} | {prec:.3} | {per:.2} |").unwrap();
            ji.push(json!({"analogue": label, "view": vn.replace("**", ""), "recall": rec, "precision": prec, "per_query": per}));
        }
    }
    // Reference: within-view corroboration over the nested top-5.
    for min in [2usize, 3] {
        let (rec, prec, per) = agg(&|r: &QRes| r.top5_support.iter().filter(|x| x.1 .0 >= min).map(|(t, v)| (t.clone(), v.1)).collect());
        writeln!(md, "| nested fused 0.3, top-5 | nested view, support ≥ {min} | {rec:.3} | {prec:.3} | {per:.2} |").unwrap();
        ji.push(json!({"analogue": "nested top-5", "view": format!("support>={min}"), "recall": rec, "precision": prec, "per_query": per}));
    }
    for min in [1usize, 2, 3] {
        let (rec, prec, per) = agg(&|r: &QRes| r.top5_flat_rank.iter().filter(|x| x.1 .0 >= min).map(|(t, v)| (t.clone(), v.1)).collect());
        writeln!(md, "| **flat fused 0.7, top-5** | **nested view, support ≥ {min}** | {rec:.3} | {prec:.3} | {per:.2} |").unwrap();
        ji.push(json!({"analogue": "flat-ranked top-5", "view": format!("nested, support>={min}"), "recall": rec, "precision": prec, "per_query": per}));
    }
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E19.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E19.json"), serde_json::to_string_pretty(&json!({"seed": seed, "per_func": per_func, "k": k, "retrieval": jr, "inference": ji, "queries_inference": nq, "queries_cross_pkg": nq_a, "queries_cross_lang": nq_x})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
