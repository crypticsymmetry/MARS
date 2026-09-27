//! E17: candidate inference on real code — can an analogue restore a
//! deleted statement?
//!
//! Corpus: the E9 Python corpus (three packages). Query: a main function
//! with one top-level control statement (relational order ≥ 2) deleted,
//! chosen so that every entity it mentions still occurs elsewhere in the
//! function (restorable in principle). Analogues are retrieved from all
//! other functions (fused 0.3·FAC + 0.7·fingerprint-literal, the E9/E10
//! setting for code); candidate inferences are projected from each.
//!
//! Recovery is scored exactly (rendered text) and *up to skolems* (a
//! hypothesized new entity may stand for any query entity, consistently).
//! Conditions: top-1 analogue; top-5 with corroboration support; lexical
//! retrieval baseline; the oracle counterpart (same algorithm, other
//! package) as the analogue.

use crate::e9::load_corpus;
use crate::metrics::mean;
use crate::Args;
use mars_encode::{cosine, lexical_tokens, FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, SparseVec, N_CHANNELS};
use mars_hv::Rng;
use mars_map::{Grounding, MapConfig, Mapper, Proj};
use mars_rel::{CaseId, CaseKind, ExprId, Kb, Sym, Term};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde_json::json;
use std::collections::HashSet;
use std::fmt::Write as _;
use std::time::Instant;

/// Does projection `p` match term `t`, letting skolems stand for entities
/// (consistently: one skolem, one entity)?
pub(crate) fn proj_matches(kb: &Kb, p: &Proj, t: Term, bind: &mut FxHashMap<Sym, Sym>) -> bool {
    match (p, t) {
        (Proj::Target(x), _) => *x == t,
        (Proj::Skolem(s), Term::Ent(e)) => *bind.entry(*s).or_insert(e) == e,
        (Proj::Skolem(_), Term::Expr(_)) => false,
        (Proj::Expr { functor, args }, Term::Expr(e)) => {
            let ex = kb.expr(e);
            ex.functor == *functor && ex.args.len() == args.len() && args.iter().zip(ex.args.iter()).all(|(a, b)| proj_matches(kb, a, *b, bind))
        }
        _ => false,
    }
}

/// Multiset of sub-expression *shapes* (entities abstracted to `_`).
fn shapes_proj(kb: &Kb, p: &Proj, out: &mut Vec<String>) -> String {
    match p {
        Proj::Target(Term::Expr(e)) => shapes_expr(kb, *e, out),
        Proj::Target(Term::Ent(_)) | Proj::Skolem(_) => "_".into(),
        Proj::Expr { functor, args } => {
            let inner: Vec<String> = args.iter().map(|a| shapes_proj(kb, a, out)).collect();
            let sh = format!("({} {})", kb.name(*functor), inner.join(" "));
            out.push(sh.clone());
            sh
        }
    }
}

fn shapes_expr(kb: &Kb, e: ExprId, out: &mut Vec<String>) -> String {
    let ex = kb.expr(e);
    let inner: Vec<String> = ex.args.iter().map(|a| match *a {
        Term::Ent(_) => "_".to_string(),
        Term::Expr(x) => shapes_expr(kb, x, out),
    }).collect();
    let sh = format!("({} {})", kb.name(ex.functor), inner.join(" "));
    out.push(sh.clone());
    sh
}

/// Dice overlap of two shape multisets.
fn dice(a: &[String], b: &[String]) -> f64 {
    let mut cnt: FxHashMap<&str, i64> = FxHashMap::default();
    for x in a {
        *cnt.entry(x).or_insert(0) += 1;
    }
    let mut common = 0;
    for x in b {
        if let Some(c) = cnt.get_mut(x.as_str()) {
            if *c > 0 {
                *c -= 1;
                common += 1;
            }
        }
    }
    if a.is_empty() && b.is_empty() { 0.0 } else { 2.0 * common as f64 / (a.len() + b.len()) as f64 }
}

pub(crate) fn entities(kb: &Kb, e: ExprId, out: &mut HashSet<Sym>) {
    for a in kb.expr(e).args.iter() {
        match *a {
            Term::Ent(s) => {
                out.insert(s);
            }
            Term::Expr(x) => entities(kb, x, out),
        }
    }
}

/// One proposal: (text, has skolem, equals deleted exactly, matches deleted up to skolems, in the original function, shape overlap with the deleted statement).
type Prop = (String, bool, bool, bool, bool, f64);

struct Q {
    case: CaseId,
    orig: usize,
    deleted: ExprId,
    has_counterpart: bool,
}

pub fn run(args: &Args) -> Result<(), String> {
    let dir = args.str("data", "data/e9");
    let seed = args.u64("seed", 1);
    let per_func = args.usize("per-func", 2);
    let m_max = 5usize;
    let k = args.usize("k", 64);
    let out_dir = args.str("out", "results/E17");
    let t0 = Instant::now();
    let (mut kb, metas) = load_corpus(&dir)?;
    let metas: Vec<_> = metas.into_iter().filter(|m| m.lang == "py").collect();
    let n = metas.len();

    // Queries.
    let mut rng = Rng::new(seed ^ 0xE17);
    let mut queries: Vec<Q> = Vec::new();
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
                let mut mine = HashSet::new();
                entities(&kb, f, &mut mine);
                let mut rest = HashSet::new();
                for &g in &facts {
                    if g != f {
                        entities(&kb, g, &mut rest);
                    }
                }
                mine.is_subset(&rest)
            })
            .collect();
        rng.shuffle(&mut cands);
        let has_counterpart = metas.iter().any(|o| o.is_main && o.norm == m.norm && o.pkg != m.pkg);
        for &del in cands.iter().take(per_func) {
            let kept: Vec<ExprId> = facts.iter().copied().filter(|&f| f != del).collect();
            let name = format!("{}-del{}", kb.name(kb.case(m.case).name), queries.len());
            let q = kb.add_case(&name, CaseKind::Query, kept);
            queries.push(Q { case: q, orig: i, deleted: del, has_counterpart });
        }
    }
    let nq = queries.len();
    eprintln!("[e17] {n} functions, {nq} queries ({:.1?})", t0.elapsed());

    // Representations (statistics from the corpus only).
    let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
    let raw: Vec<Features> = metas.par_iter().map(|m| fx.extract(m.case)).collect();
    let stats = FeatureStats::fit(raw.iter());
    let sk = Sketcher::new(Layout::default(), 0xF1);
    let sketch = |mut f: Features| {
        stats.apply(&mut f, &[true; N_CHANNELS]);
        sk.sketch(&f)
    };
    let fps: Vec<_> = raw.into_par_iter().map(sketch).collect();
    let qfps: Vec<_> = queries.par_iter().map(|q| sketch(fx.extract(q.case))).collect();
    let lex_raw: Vec<SparseVec> = metas.iter().map(|m| lexical_tokens(&kb, m.case)).collect();
    let lex_stats = FeatureStats::fit(lex_raw.iter().map(|v| Features { channels: [v.clone(), vec![], vec![], vec![], vec![]] }).collect::<Vec<_>>().iter());
    let idf = |mut v: SparseVec| {
        lex_stats.apply_sparse(0, &mut v);
        v
    };
    let lex: Vec<SparseVec> = lex_raw.into_iter().map(idf).collect();
    let qlex: Vec<SparseVec> = queries.iter().map(|q| idf(lexical_tokens(&kb, q.case))).collect();
    let mapper = Mapper::new(&kb, MapConfig::default());
    let self_s: Vec<f64> = metas.par_iter().map(|m| mapper.score(m.case, m.case) as f64).collect();
    let literal = Profile::literal();

    let proposals = |base: CaseId, q: &Q| -> Vec<Prop> {
        let full: HashSet<String> = kb.case(metas[q.orig].case).facts.iter().map(|&f| kb.render_expr(f)).collect();
        let del_text = kb.render_expr(q.deleted);
        let mut del_shapes = Vec::new();
        shapes_expr(&kb, q.deleted, &mut del_shapes);
        let mut v: Vec<Prop> = mapper
            .best(base, q.case)
            .map(|m| {
                m.inferences
                    .iter()
                    .filter(|i| i.grounding == Grounding::Structural)
                    .map(|i| {
                        let text = mapper.render_proj(&i.projected);
                        let tol = proj_matches(&kb, &i.projected, Term::Expr(q.deleted), &mut FxHashMap::default());
                        let exact = text == del_text;
                        let in_full = full.contains(&text);
                        let mut sh = Vec::new();
                        shapes_proj(&kb, &i.projected, &mut sh);
                        (text, i.has_skolem, exact, tol, in_full, dice(&sh, &del_shapes))
                    })
                    .collect()
            })
            .unwrap_or_default();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v.dedup_by(|a, b| a.0 == b.0);
        v
    };

    struct R {
        fused: Vec<Vec<Prop>>,
        lexical: Vec<Prop>,
        random: Vec<Prop>,
        oracle: Option<Vec<Prop>>,
        top_same_pkg: bool,
    }
    let res: Vec<R> = queries
        .par_iter()
        .enumerate()
        .map(|(qi, q)| {
            let qs = mapper.score(q.case, q.case) as f64;
            let mut by_fp: Vec<(usize, f64)> = (0..n).filter(|&c| c != q.orig).map(|c| (c, literal.score(&sk.channel_sims(&qfps[qi], &fps[c])))).collect();
            by_fp.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            by_fp.truncate(k);
            let mut fused: Vec<(usize, f64)> = by_fp
                .into_iter()
                .map(|(c, fp)| {
                    let s = mapper.score(metas[c].case, q.case) as f64;
                    let fac = if s == 0.0 { 0.0 } else { (s / (qs * self_s[c]).sqrt()).min(1.0) };
                    (c, 0.3 * fac + 0.7 * fp)
                })
                .collect();
            fused.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let lex_top = (0..n).filter(|&c| c != q.orig).max_by(|&a, &b| cosine(&qlex[qi], &lex[a]).total_cmp(&cosine(&qlex[qi], &lex[b])).then(b.cmp(&a))).unwrap();
            let oracle = q.has_counterpart.then(|| {
                let m = &metas[q.orig];
                let cp: Vec<usize> = (0..n).filter(|&c| metas[c].is_main && metas[c].norm == m.norm && metas[c].pkg != m.pkg).collect();
                // Best-mapping counterpart (by normalized FAC).
                let best = *cp.iter().max_by(|&&a, &&b| (mapper.score(metas[a].case, q.case) as f64 / self_s[a].sqrt()).total_cmp(&(mapper.score(metas[b].case, q.case) as f64 / self_s[b].sqrt())).then(b.cmp(&a))).unwrap();
                proposals(metas[best].case, q)
            });
            let mut rr = Rng::derive(seed ^ 0x5A, qi as u64);
            let rand_c = loop {
                let c = rr.below(n as u64) as usize;
                if c != q.orig {
                    break c;
                }
            };
            R { fused: fused.iter().take(m_max).map(|&(c, _)| proposals(metas[c].case, q)).collect(), lexical: proposals(metas[lex_top].case, q), random: proposals(metas[rand_c].case, q), oracle, top_same_pkg: fused.first().map(|x| metas[x.0].pkg == metas[q.orig].pkg).unwrap_or(false) }
        })
        .collect();
    eprintln!("[e17] mapped ({:.1?})", t0.elapsed());

    // Aggregation helpers.
    let single = |sel: &dyn Fn(&R) -> Option<&Vec<Prop>>, subset: &dyn Fn(&Q) -> bool| -> (usize, f64, f64, f64, f64, f64, f64) {
        let (mut nq2, mut ex, mut tol, mut num, mut den, mut per) = (0usize, 0.0, 0.0, 0usize, 0usize, 0.0);
        let mut best_shape = Vec::new();
        for (q, r) in queries.iter().zip(&res) {
            if !subset(q) {
                continue;
            }
            let Some(p) = sel(r) else { continue };
            nq2 += 1;
            ex += p.iter().any(|x| x.2) as u8 as f64;
            tol += p.iter().any(|x| x.3) as u8 as f64;
            let grounded: Vec<&Prop> = p.iter().filter(|x| !x.1).collect();
            den += grounded.len();
            num += grounded.iter().filter(|x| x.4).count();
            per += p.len() as f64;
            best_shape.push(p.iter().map(|x| x.5).fold(0.0, f64::max));
        }
        let d = nq2.max(1) as f64;
        let half = best_shape.iter().filter(|&&x| x >= 0.5).count() as f64 / d;
        (nq2, ex / d, tol / d, num as f64 / den.max(1) as f64, per / d, mean(&best_shape), half)
    };
    let mut md = String::new();
    writeln!(md, "# E17: candidate inference on real code\n").unwrap();
    writeln!(md, "Corpus: {n} Python functions (E9). {nq} queries: a main function with one top-level control statement (order ≥ 2) deleted, all of whose entities occur elsewhere in the function (≤ {per_func} per function, seed {seed}). Analogues: all other functions, fingerprint-literal top-{k} re-ranked by 0.3·FAC + 0.7·FP. *exact* = the deleted statement is proposed verbatim; *up to skolems* = proposed with hypothesized entities standing for query entities; *precision* = fraction of skolem-free proposals that are statements of the original function. Runtime {:.1?}.\n", t0.elapsed()).unwrap();
    writeln!(md, "| analogue | subset | queries | recall exact | recall up to skolems | precision | proposals / query | best shape overlap | shape overlap ≥ 0.5 |").unwrap();
    writeln!(md, "|---|---|---|---|---|---|---|---|---|").unwrap();
    let mut rows = Vec::new();
    type Sel<'a> = Box<dyn Fn(&R) -> Option<&Vec<Prop>> + 'a>;
    let conds: Vec<(&str, Sel)> = vec![
        ("fused top-1", Box::new(|r: &R| r.fused.first())),
        ("lexical top-1 (baseline)", Box::new(|r: &R| Some(&r.lexical))),
        ("random analogue (chance)", Box::new(|r: &R| Some(&r.random))),
        ("oracle counterpart (same algorithm, other package)", Box::new(|r: &R| r.oracle.as_ref())),
    ];
    type Subset = Box<dyn Fn(&Q) -> bool>;
    let subsets: Vec<(&str, Subset)> = vec![("all", Box::new(|_: &Q| true)), ("with counterpart", Box::new(|q: &Q| q.has_counterpart)), ("without counterpart", Box::new(|q: &Q| !q.has_counterpart))];
    for (cn, sel) in &conds {
        for (sn, sub) in &subsets {
            let (c, ex, tol, prec, per, shape, half) = single(sel.as_ref(), sub.as_ref());
            if c == 0 {
                continue;
            }
            writeln!(md, "| {cn} | {sn} | {c} | {ex:.3} | {tol:.3} | {prec:.3} | {per:.2} | {shape:.3} | {half:.3} |").unwrap();
            rows.push(json!({"analogue": cn, "subset": sn, "queries": c, "recall_exact": ex, "recall_tolerant": tol, "precision": prec, "per_query": per, "best_shape_overlap": shape, "shape_overlap_ge_half": half}));
        }
    }

    // Same-author vs other-author top-1 analogues.
    writeln!(md, "\n## Fused top-1 analogue: same package (author) vs other package\n").unwrap();
    writeln!(md, "| top-1 analogue from | queries | recall exact | best shape overlap |\n|---|---|---|---|").unwrap();
    let mut split = Vec::new();
    for same in [true, false] {
        let sel: Vec<&R> = res.iter().filter(|r| r.top_same_pkg == same).collect();
        let ex = mean(&sel.iter().map(|r| r.fused[0].iter().any(|x| x.2) as u8 as f64).collect::<Vec<_>>());
        let sh = mean(&sel.iter().map(|r| r.fused[0].iter().map(|x| x.5).fold(0.0, f64::max)).collect::<Vec<_>>());
        writeln!(md, "| {} | {} | {ex:.3} | {sh:.3} |", if same { "same package" } else { "other package" }, sel.len()).unwrap();
        split.push(json!({"same_package": same, "queries": sel.len(), "recall_exact": ex, "best_shape_overlap": sh}));
    }

    // Corroboration across the top-5 fused analogues.
    writeln!(md, "\n## Corroboration across the top-{m_max} fused analogues\n").unwrap();
    writeln!(md, "| accept if support ≥ | recall exact | recall up to skolems | precision (skolem-free) | proposals / query |").unwrap();
    writeln!(md, "|---|---|---|---|---|").unwrap();
    let mut by_support: FxHashMap<usize, (usize, usize)> = FxHashMap::default();
    /// (support, has skolem, exact, up to skolems, in the original function)
    type Agg = (usize, bool, bool, bool, bool);
    let mut agg: Vec<Vec<Agg>> = Vec::new();
    for r in &res {
        let mut m: FxHashMap<&str, Agg> = FxHashMap::default();
        for p in &r.fused {
            for x in p {
                let e = m.entry(x.0.as_str()).or_insert((0, x.1, x.2, x.3, x.4));
                let _ = x.5;
                e.0 += 1;
            }
        }
        for v in m.values() {
            if !v.1 {
                let e = by_support.entry(v.0).or_insert((0, 0));
                e.0 += 1;
                e.1 += v.4 as usize;
            }
        }
        agg.push(m.into_values().collect());
    }
    let mut corr = Vec::new();
    for min in 1..=m_max {
        let rec_ex = mean(&agg.iter().map(|a| a.iter().any(|x| x.0 >= min && x.2) as u8 as f64).collect::<Vec<_>>());
        let rec_tol = mean(&agg.iter().map(|a| a.iter().any(|x| x.0 >= min && x.3) as u8 as f64).collect::<Vec<_>>());
        let acc: Vec<&Agg> = agg.iter().flatten().filter(|x| x.0 >= min && !x.1).collect();
        let prec = acc.iter().filter(|x| x.4).count() as f64 / acc.len().max(1) as f64;
        let per = agg.iter().flatten().filter(|x| x.0 >= min).count() as f64 / nq.max(1) as f64;
        writeln!(md, "| {min} | {rec_ex:.3} | {rec_tol:.3} | {prec:.3} | {per:.2} |").unwrap();
        corr.push(json!({"min_support": min, "recall_exact": rec_ex, "recall_tolerant": rec_tol, "precision": prec, "per_query": per}));
    }
    writeln!(md, "\nPrecision by exact support (skolem-free proposals): {}", (1..=m_max).filter_map(|s| by_support.get(&s).map(|&(t, ok)| format!("{s}: {:.3} (n={t})", ok as f64 / t as f64))).collect::<Vec<_>>().join(" · ")).unwrap();

    // Examples: recovered statements.
    writeln!(md, "\n## Examples (fused top-1, recovered exactly)\n").unwrap();
    for (q, r) in queries.iter().zip(&res).filter(|(_, r)| r.fused.first().map(|p| p.iter().any(|x| x.2)).unwrap_or(false)).take(6) {
        writeln!(md, "- `{}`: restored `{}`", kb.name(kb.case(metas[q.orig].case).name), kb.render_expr(q.deleted)).unwrap();
        let _ = r;
    }
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E17.md"), &md).map_err(|e| e.to_string())?;
    std::fs::write(format!("{out_dir}/E17.json"), serde_json::to_string_pretty(&json!({"seed": seed, "per_func": per_func, "k": k, "queries": nq, "rows": rows, "corroboration": corr, "author_split": split})).unwrap()).map_err(|e| e.to_string())?;
    println!("{md}");
    Ok(())
}
