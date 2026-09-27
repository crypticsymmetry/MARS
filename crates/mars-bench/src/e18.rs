//! E18: representation granularity for code — nested statements vs flat
//! blocks.
//!
//! The converters (P7a) encode control structure by *wrapping*: a statement
//! inside a loop inside a branch is one fact
//! `(in-loop i (guards t (swap …)))`. E17 found that candidate inference on
//! this representation is all-or-nothing: a whole nested statement must be
//! projected. The *flat* representation (built here from the same KB) makes
//! each distinct control context a block entity and each statement a small
//! fact placed in it:
//!
//! ```text
//! (loop-block b1 i)  (loop-block b2 j)  (within b2 b1)
//! (guard-block b3 t) (within b3 b2)     (in-block b3 (swap …))
//! ```
//!
//! Both representations are evaluated on E9 retrieval (same algorithm,
//! other package) and on E17-style inference, deleting the *same core
//! statement* in both. Recovery is scored for the core statement and for
//! the statement in its context.

use crate::e17::{entities, proj_matches};
use crate::e9::{load_corpus, Meta};
use crate::metrics::{mean, mrr, recall_at};
use crate::Args;
use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, N_CHANNELS};
use mars_hv::{HyperVector, Rng};
use mars_map::{Grounding, MapConfig, Mapper, Proj};
use mars_rel::views::{FlatView, CODE_WRAPPERS};
use mars_rel::{CaseId, CaseKind, ExprId, Kb, Term};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde_json::json;
use std::collections::HashSet;
use std::fmt::Write as _;
use std::time::Instant;

/// Core of a projected inference: strip wrappers (nested) or the block (flat).
pub(crate) fn proj_core<'a>(fl: &FlatView, p: &'a Proj) -> &'a Proj {
    match p {
        Proj::Expr { functor, args } if args.len() == 2 && (*functor == fl.in_block || fl.is_wrapper(*functor)) => proj_core(fl, &args[1]),
        _ => p,
    }
}

/// One representation of the corpus: case per function.
pub(crate) struct Rep {
    pub cases: Vec<CaseId>,
    pub fps: Vec<HyperVector>,
    pub sk: Sketcher,
    pub stats: FeatureStats,
    pub self_s: Vec<f64>,
}

pub(crate) fn build_rep(kb: &Kb, cases: Vec<CaseId>) -> Rep {
    let fx = FeatureExtractor::new(kb, FeatureConfig::default());
    let raw: Vec<Features> = cases.par_iter().map(|&c| fx.extract(c)).collect();
    let stats = FeatureStats::fit(raw.iter());
    let sk = Sketcher::new(Layout::default(), 0xF1);
    let fps = raw
        .into_par_iter()
        .map(|mut f| {
            stats.apply(&mut f, &[true; N_CHANNELS]);
            sk.sketch(&f)
        })
        .collect();
    let mapper = Mapper::new(kb, MapConfig::default());
    let self_s = cases.par_iter().map(|&c| mapper.score(c, c) as f64).collect();
    Rep { cases, fps, sk, stats, self_s }
}

impl Rep {
    pub(crate) fn fp_of(&self, kb: &Kb, c: CaseId) -> HyperVector {
        let mut f = FeatureExtractor::new(kb, FeatureConfig::default()).extract(c);
        self.stats.apply(&mut f, &[true; N_CHANNELS]);
        self.sk.sketch(&f)
    }

    /// Fused ranking (0.3·FAC + 0.7·FP-literal over the FP top-k) of all functions but `skip`.
    fn rank(&self, kb: &Kb, q: CaseId, qfp: &HyperVector, skip: usize, k: usize) -> Vec<(usize, f64, f64, f64)> {
        let literal = Profile::literal();
        let mapper = Mapper::new(kb, MapConfig::default());
        let qs = mapper.score(q, q) as f64;
        let mut v: Vec<(usize, f64)> = (0..self.cases.len()).filter(|&c| c != skip).map(|c| (c, literal.score(&self.sk.channel_sims(qfp, &self.fps[c])))).collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        v.truncate(k);
        let mut out: Vec<(usize, f64, f64, f64)> = v
            .into_iter()
            .map(|(c, fp)| {
                let s = mapper.score(self.cases[c], q) as f64;
                let fac = if s == 0.0 { 0.0 } else { (s / (qs * self.self_s[c]).sqrt()).min(1.0) };
                (c, 0.3 * fac + 0.7 * fp, fac, fp)
            })
            .collect();
        out.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        out
    }
}

/// Task A retrieval (E9): (R@1, R@10, MRR) for FP-literal, FAC and fused
/// with FAC weight 0.3 / 0.5 / 0.7, over the full ranking.
fn retrieval(kb: &Kb, rep: &Rep, metas: &[Meta]) -> Vec<(f64, f64, f64)> {
    let n = metas.len();
    let queries: Vec<usize> = (0..n).filter(|&i| metas[i].is_main && (0..n).any(|j| j != i && metas[j].is_main && metas[j].norm == metas[i].norm && metas[j].pkg != metas[i].pkg)).collect();
    let literal = Profile::literal();
    let mapper = Mapper::new(kb, MapConfig::default());
    let ranks: Vec<[usize; 5]> = queries
        .par_iter()
        .map(|&q| {
            let sc: Vec<[f64; 5]> = (0..n)
                .map(|c| {
                    let fp = literal.score(&rep.sk.channel_sims(&rep.fps[q], &rep.fps[c]));
                    let s = mapper.score(rep.cases[c], rep.cases[q]) as f64;
                    let fac = if s == 0.0 { 0.0 } else { (s / (rep.self_s[q] * rep.self_s[c]).sqrt()).min(1.0) };
                    [fp, fac, 0.3 * fac + 0.7 * fp, 0.5 * fac + 0.5 * fp, 0.7 * fac + 0.3 * fp]
                })
                .collect();
            let rel = |c: usize| metas[c].is_main && metas[c].norm == metas[q].norm && metas[c].pkg != metas[q].pkg;
            std::array::from_fn(|m| {
                let mut o: Vec<usize> = (0..n).filter(|&c| c != q).collect();
                o.sort_by(|&a, &b| sc[b][m].total_cmp(&sc[a][m]).then(a.cmp(&b)));
                o.iter().position(|&c| rel(c)).unwrap_or(usize::MAX / 2)
            })
        })
        .collect();
    (0..5)
        .map(|m| {
            let r: Vec<usize> = ranks.iter().map(|x| x[m]).collect();
            (recall_at(&r, 1), recall_at(&r, 10), mrr(&r))
        })
        .collect()
}

/// Per query and analogue rank: proposals (text, skolem, core exact, full up to skolems, in original).
type Prop = (String, bool, bool, bool, bool);
type Props = Vec<Vec<Vec<Prop>>>;

pub fn run(args: &Args) -> Result<(), String> {
    let dir = args.str("data", "data/e9");
    let seed = args.u64("seed", 1);
    let per_func = args.usize("per-func", 2);
    let out_dir = args.str("out", "results/E18");
    let t0 = Instant::now();
    let (mut kb, metas) = load_corpus(&dir)?;
    let metas: Vec<Meta> = metas.into_iter().filter(|m| m.lang == "py").collect();
    let n = metas.len();
    let fl = FlatView::declare(&mut kb, &CODE_WRAPPERS);
    let mut flat_cases = Vec::new();
    let mut core_maps = Vec::new();
    let mut size = (0usize, 0usize, 0usize, 0usize); // facts and expressions, nested / flat
    for (i, m) in metas.iter().enumerate() {
        let (facts, cm) = fl.flatten(&mut kb, m.case, &format!("f{i}"));
        let name = format!("flat:{}", kb.name(kb.case(m.case).name));
        size.0 += kb.case(m.case).facts.len();
        size.1 += kb.case_exprs(m.case).len();
        let c = kb.add_case(&name, CaseKind::Episode, facts);
        size.2 += kb.case(c).facts.len();
        size.3 += kb.case_exprs(c).len();
        flat_cases.push(c);
        core_maps.push(cm);
    }
    eprintln!("[e18] {n} functions flattened ({:.1?})", t0.elapsed());

    // Queries: the same deleted statement in both representations.
    let mut rng = Rng::new(seed ^ 0xE17);
    let mut qn: Vec<(CaseId, usize, ExprId)> = Vec::new(); // (query case, function, deleted fact)
    let mut qf: Vec<(CaseId, usize, ExprId)> = Vec::new();
    let mut cores: Vec<ExprId> = Vec::new();
    for (i, m) in metas.iter().enumerate() {
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
            let k = qn.len();
            let kept: Vec<ExprId> = facts.iter().copied().filter(|&f| f != del).collect();
            let q = kb.add_case(&format!("qn{k}"), CaseKind::Query, kept);
            qn.push((q, i, del));
            let fdel = core_maps[i][&del];
            // The flat fact may be shared by an identical statement elsewhere: then nothing is deleted from the flat case.
            let shared = facts.iter().any(|&g| g != del && core_maps[i].get(&g) == Some(&fdel));
            let mut fkept: Vec<ExprId> = kb.case(flat_cases[i]).facts.iter().copied().filter(|&f| f != fdel || shared).collect();
            fl.prune(&kb, &mut fkept);
            let q = kb.add_case(&format!("qf{k}"), CaseKind::Query, fkept);
            qf.push((q, i, fdel));
            cores.push(fl.peel(&kb, del).1);
        }
    }
    let nq = qn.len();
    let nested = build_rep(&kb, metas.iter().map(|m| m.case).collect());
    let flat = build_rep(&kb, flat_cases.clone());
    eprintln!("[e18] {nq} queries, representations built ({:.1?})", t0.elapsed());

    let infer = |rep: &Rep, qs: &[(CaseId, usize, ExprId)]| -> Props {
        let mapper = Mapper::new(&kb, MapConfig::default());
        qs.par_iter()
            .zip(cores.par_iter())
            .map(|(&(q, orig, del), &core)| {
                let qfp = rep.fp_of(&kb, q);
                let full: HashSet<String> = kb.case(rep.cases[orig]).facts.iter().map(|&f| kb.render_expr(f)).collect();
                let core_text = kb.render_expr(core);
                rep.rank(&kb, q, &qfp, orig, 64)
                    .iter()
                    .take(5)
                    .map(|&(c, _, _, _)| {
                        let mut v: Vec<(String, bool, bool, bool, bool)> = mapper
                            .best(rep.cases[c], q)
                            .map(|m| {
                                m.inferences
                                    .iter()
                                    .filter(|i| i.grounding == Grounding::Structural)
                                    .map(|i| {
                                        let text = mapper.render_proj(&i.projected);
                                        let core_ok = mapper.render_proj(proj_core(&fl, &i.projected)) == core_text;
                                        let full_ok = proj_matches(&kb, &i.projected, Term::Expr(del), &mut FxHashMap::default());
                                        let in_full = full.contains(&text);
                                        (text, i.has_skolem, core_ok, full_ok, in_full)
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        v.sort_by(|a, b| a.0.cmp(&b.0));
                        v.dedup_by(|a, b| a.0 == b.0);
                        v
                    })
                    .collect()
            })
            .collect()
    };
    let pn = infer(&nested, &qn);
    let pf = infer(&flat, &qf);
    eprintln!("[e18] inference done ({:.1?})", t0.elapsed());

    let summarize = |p: &Props| -> serde_json::Value {
        let top1_core = mean(&p.iter().map(|x| x.first().map(|v| v.iter().any(|y| y.2)).unwrap_or(false) as u8 as f64).collect::<Vec<_>>());
        let top1_full = mean(&p.iter().map(|x| x.first().map(|v| v.iter().any(|y| y.3)).unwrap_or(false) as u8 as f64).collect::<Vec<_>>());
        let top1_props: Vec<&(String, bool, bool, bool, bool)> = p.iter().filter_map(|x| x.first()).flatten().filter(|y| !y.1).collect();
        let top1_prec = top1_props.iter().filter(|y| y.4).count() as f64 / top1_props.len().max(1) as f64;
        let per_q = mean(&p.iter().map(|x| x.first().map(|v| v.len()).unwrap_or(0) as f64).collect::<Vec<_>>());
        // Support over the top-5 (by text).
        /// (support, skolem, core, in context, in original)
        type Sup = (usize, bool, bool, bool, bool);
        let mut sup: Vec<FxHashMap<&str, Sup>> = Vec::new();
        for x in p {
            let mut m: FxHashMap<&str, Sup> = FxHashMap::default();
            for v in x {
                for y in v {
                    m.entry(y.0.as_str()).or_insert((0, y.1, y.2, y.3, y.4)).0 += 1;
                }
            }
            sup.push(m);
        }
        let at = |min: usize| {
            let core = mean(&sup.iter().map(|m| m.values().any(|y| y.0 >= min && y.2) as u8 as f64).collect::<Vec<_>>());
            let acc: Vec<&Sup> = sup.iter().flat_map(|m| m.values()).filter(|y| y.0 >= min && !y.1).collect();
            let prec = acc.iter().filter(|y| y.4).count() as f64 / acc.len().max(1) as f64;
            (core, prec, acc.len() as f64 / sup.len().max(1) as f64)
        };
        let (c1, p1, n1) = at(1);
        let (c2, p2, n2) = at(2);
        let (c3, p3, n3) = at(3);
        json!({"top1_core": top1_core, "top1_full": top1_full, "top1_precision": top1_prec, "top1_per_query": per_q,
               "sup1": [c1, p1, n1], "sup2": [c2, p2, n2], "sup3": [c3, p3, n3]})
    };
    let sn = summarize(&pn);
    let sf = summarize(&pf);
    let rn = retrieval(&kb, &nested, &metas);
    let rf = retrieval(&kb, &flat, &metas);

    let mut md = String::new();
    writeln!(md, "# E18: representation granularity for code — nested vs flat\n").unwrap();
    writeln!(md, "Corpus: {n} Python functions (E9). Nested: {} facts / {} expressions in total; flat: {} facts / {} expressions. Seed {seed}; runtime {:.1?}.\n", size.0, size.1, size.2, size.3, t0.elapsed()).unwrap();
    writeln!(md, "## Retrieval (E9 task A: same algorithm, other package) — MRR (R@1 / R@10)\n").unwrap();
    writeln!(md, "| representation | fingerprint literal | FAC | fused w=0.3 | fused w=0.5 | fused w=0.7 |\n|---|---|---|---|---|---|").unwrap();
    for (name, r) in [("nested (P7a)", &rn), ("flat blocks", &rf)] {
        writeln!(md, "| {name} | {} |", r.iter().map(|(a, b, c)| format!("{c:.3} ({a:.2} / {b:.2})")).collect::<Vec<_>>().join(" | ")).unwrap();
    }
    writeln!(md, "\n## Inference: restoring a deleted statement ({nq} queries, same core statement deleted in both)\n").unwrap();
    writeln!(md, "*core* = the deleted statement itself is proposed (without its control context); *in context* = proposed together with the right context (nested: the whole wrapped fact; flat: the right block, up to skolems). Precision = skolem-free proposals that are facts of the original function.\n").unwrap();
    writeln!(md, "| representation | top-1: core | top-1: in context | top-1 precision | proposals / query | top-5 support ≥ 2: core / precision / proposals | support ≥ 3: core / precision / proposals |").unwrap();
    writeln!(md, "|---|---|---|---|---|---|---|").unwrap();
    for (name, s) in [("nested (P7a)", &sn), ("flat blocks", &sf)] {
        let f = |k: &str| s[k].as_f64().unwrap();
        let t = |k: &str| {
            let a = s[k].as_array().unwrap();
            format!("{:.3} / {:.3} / {:.2}", a[0].as_f64().unwrap(), a[1].as_f64().unwrap(), a[2].as_f64().unwrap())
        };
        writeln!(md, "| {name} | {:.3} | {:.3} | {:.3} | {:.2} | {} | {} |", f("top1_core"), f("top1_full"), f("top1_precision"), f("top1_per_query"), t("sup2"), t("sup3")).unwrap();
    }
    let hit = |p: &Props, i: usize, k: usize| p[i].first().map(|v| v.iter().any(|y| if k == 2 { y.2 } else { y.3 })).unwrap_or(false);
    let (mut both_core, mut either_core, mut either_ctx) = (0usize, 0usize, 0usize);
    for i in 0..nq {
        both_core += (hit(&pn, i, 2) && hit(&pf, i, 2)) as usize;
        either_core += (hit(&pn, i, 2) || hit(&pf, i, 2)) as usize;
        either_ctx += (hit(&pn, i, 3) || hit(&pf, i, 3)) as usize;
    }
    let d = nq.max(1) as f64;
    writeln!(md, "| either representation (top-1 of each) | {:.3} | {:.3} | | | | |", either_core as f64 / d, either_ctx as f64 / d).unwrap();
    writeln!(md, "\nCore restored by both representations: {:.3}; by nested only: {:.3}; by flat only: {:.3}.", both_core as f64 / d, (0..nq).filter(|&i| hit(&pn, i, 2) && !hit(&pf, i, 2)).count() as f64 / d, (0..nq).filter(|&i| !hit(&pn, i, 2) && hit(&pf, i, 2)).count() as f64 / d).unwrap();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E18.md"), &md).map_err(|e| e.to_string())?;
    let rj = |r: &Vec<(f64, f64, f64)>| r.iter().zip(["fp-literal", "fac", "fused-0.3", "fused-0.5", "fused-0.7"]).map(|((a, b, c), m)| json!({"method": m, "r1": a, "r10": b, "mrr": c})).collect::<Vec<_>>();
    std::fs::write(format!("{out_dir}/E18.json"), serde_json::to_string_pretty(&json!({"seed": seed, "per_func": per_func, "queries": nq, "retrieval": {"nested": rj(&rn), "flat": rj(&rf)}, "inference": {"nested": sn, "flat": sf}})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
