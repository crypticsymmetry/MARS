//! Soft relation similarity: graded matches between *different* predicates.
//!
//! The mapper normally pairs only identical predicates, plus taxonomy siblings
//! (ascension) or, optionally, any same-arity pair (wildcard). `RelSim` learns
//! which predicates are used alike from the memory itself, without labels. It
//! follows the distributional hypothesis, applied to relational roles:
//!
//! * The context of predicate `p` is the multiset of role features of its
//!   occurrences:
//!   - `(i, q, j)`: argument `i` of `p` is an entity that is also argument
//!     `j` of a `q` fact in the same case;
//!   - `(child, i, q)`: argument `i` is a nested `q` expression;
//!   - `(parent, q, j)`: the expression is argument `j` of a `q` expression.
//! * Counts are weighted by positive pointwise mutual information (PPMI), and
//!   similarity is the cosine of the PPMI vectors.
//! * Each predicate keeps its `k` most similar predicates with similarity
//!   ≥ `min_sim`. The mapper uses them as extra match hypotheses with local score
//!   `scale × similarity` ([`crate::MapConfig::soft`]).
//!
//! Attributes are excluded both as targets and as context: they are surface
//! features, not relations.

use mars_rel::{CaseId, Kb, PredKind, Sym, Term};
use rustc_hash::FxHashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Ctx {
    Shared(u8, Sym, u8),
    Child(u8, Sym),
    Parent(Sym, u8),
}

/// Learned neighbours per structural predicate: `(other, local score)`, best first.
#[derive(Clone, Debug, Default)]
pub struct RelSim {
    pub neighbours: FxHashMap<Sym, Vec<(Sym, f32)>>,
}

fn pos(i: usize) -> u8 {
    i.min(7) as u8
}

impl RelSim {
    /// Fit on `cases`, keeping the top `k` neighbours with cosine ≥ `min_sim`,
    /// scaled by `scale` (the local score of a perfect soft match).
    pub fn fit(kb: &Kb, cases: &[CaseId], k: usize, min_sim: f32, scale: f32) -> RelSim {
        Self::fit_with(kb, cases, k, min_sim, scale, false)
    }

    /// As [`RelSim::fit`]; with `resolved_context`, context features mention only
    /// predicates whose structural name is canonical (so that unresolved,
    /// domain-specific names do not make every context unique).
    pub fn fit_with(kb: &Kb, cases: &[CaseId], k: usize, min_sim: f32, scale: f32, resolved_context: bool) -> RelSim {
        let v = &kb.vocab;
        let ctx_ok = |q: Sym| !resolved_context || v.is_canonical(q);
        let sf = |e: mars_rel::ExprId| v.structural(kb.expr(e).functor);
        let relational = |p: Sym| v.kind(p) != PredKind::Attribute;
        let mut counts: FxHashMap<Sym, FxHashMap<Ctx, f64>> = FxHashMap::default();
        for &c in cases {
            let exprs = kb.case_exprs(c);
            // Entity -> (predicate, position) occurrences in this case.
            let mut occ: FxHashMap<Sym, Vec<(Sym, u8)>> = FxHashMap::default();
            let mut parent_of: FxHashMap<mars_rel::ExprId, Vec<(Sym, u8)>> = FxHashMap::default();
            for &e in &exprs {
                let p = sf(e);
                if !relational(p) {
                    continue;
                }
                for (i, a) in kb.expr(e).args.iter().enumerate() {
                    match *a {
                        Term::Ent(x) => occ.entry(x).or_default().push((p, pos(i))),
                        Term::Expr(ch) => parent_of.entry(ch).or_default().push((p, pos(i))),
                    }
                }
            }
            for &e in &exprs {
                let p = sf(e);
                if !relational(p) {
                    continue;
                }
                let row = counts.entry(p).or_default();
                for (i, a) in kb.expr(e).args.iter().enumerate() {
                    match *a {
                        Term::Ent(x) => {
                            let others = &occ[&x];
                            // Each other occurrence of x (this fact's own slot excluded once).
                            let mut skipped = false;
                            for &(q, j) in others {
                                if !skipped && q == p && j == pos(i) {
                                    skipped = true;
                                    continue;
                                }
                                if !ctx_ok(q) {
                                    continue;
                                }
                                *row.entry(Ctx::Shared(pos(i), q, j)).or_insert(0.0) += 1.0;
                            }
                        }
                        Term::Expr(ch) => {
                            let q = sf(ch);
                            if relational(q) && ctx_ok(q) {
                                *row.entry(Ctx::Child(pos(i), q)).or_insert(0.0) += 1.0;
                            }
                        }
                    }
                }
                if let Some(ps) = parent_of.get(&e) {
                    for &(q, j) in ps.iter().filter(|(q, _)| ctx_ok(*q)) {
                        *row.entry(Ctx::Parent(q, j)).or_insert(0.0) += 1.0;
                    }
                }
            }
        }
        // PPMI weighting.
        let mut col: FxHashMap<Ctx, f64> = FxHashMap::default();
        let mut total = 0.0;
        for row in counts.values() {
            for (&f, &n) in row {
                *col.entry(f).or_insert(0.0) += n;
                total += n;
            }
        }
        let mut vecs: Vec<(Sym, FxHashMap<Ctx, f64>, f64)> = Vec::new();
        for (&p, row) in &counts {
            let rs: f64 = row.values().sum();
            let w: FxHashMap<Ctx, f64> = row
                .iter()
                .filter_map(|(&f, &n)| {
                    let pmi = (n * total / (rs * col[&f])).ln();
                    (pmi > 0.0).then_some((f, pmi))
                })
                .collect();
            let norm = w.values().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 0.0 {
                vecs.push((p, w, norm));
            }
        }
        vecs.sort_by_key(|(p, _, _)| *p);
        let mut neighbours: FxHashMap<Sym, Vec<(Sym, f32)>> = FxHashMap::default();
        for (a, (pa, wa, na)) in vecs.iter().enumerate() {
            let mut sims: Vec<(Sym, f32)> = Vec::new();
            for (b, (pb, wb, nb)) in vecs.iter().enumerate() {
                if a == b {
                    continue;
                }
                let (small, large) = if wa.len() <= wb.len() { (wa, wb) } else { (wb, wa) };
                let dot: f64 = small.iter().filter_map(|(f, x)| large.get(f).map(|y| x * y)).sum();
                let s = (dot / (na * nb)) as f32;
                if s >= min_sim {
                    sims.push((*pb, s));
                }
            }
            sims.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
            sims.truncate(k);
            if !sims.is_empty() {
                neighbours.insert(*pa, sims.into_iter().map(|(q, s)| (q, scale * s)).collect());
            }
        }
        RelSim { neighbours }
    }

    /// Similarity-ordered neighbours of `p` (empty if none).
    pub fn of(&self, p: Sym) -> &[(Sym, f32)] {
        self.neighbours.get(&p).map_or(&[], |v| v.as_slice())
    }
}

/// Result of [`learn_taxonomy`]: learned clusters (members, then the new parent).
#[derive(Clone, Debug, Default)]
pub struct LearnedTaxonomy {
    pub clusters: Vec<(Sym, Vec<Sym>)>,
}

/// Re-representation by a *learned taxonomy*: clusters the relations that have
/// no canonical ancestor (unresolved vocabulary) by role-context similarity and
/// declares a canonical parent `~rel-N` per cluster. Structural channels and the
/// mapper resolve predicates to their nearest canonical ancestor, so members of a
/// cluster then match as one relation in every stage.
///
/// Average-linkage agglomeration among same-arity unresolved relations, merging
/// while the best pair's similarity is ≥ `min_sim`; repeated `iters` times,
/// each round re-fitting similarities with the previous round's clusters as the
/// vocabulary (the learned names make contexts comparable across domains).
pub fn learn_taxonomy(kb: &mut Kb, cases: &[CaseId], min_sim: f32, iters: usize) -> LearnedTaxonomy {
    let unresolved: Vec<Sym> = {
        let v = &kb.vocab;
        let mut u: Vec<Sym> = v.iter().filter(|i| i.kind == PredKind::Relation && !v.is_canonical(v.structural(i.name))).map(|i| i.name).collect();
        u.sort();
        u
    };
    let ar_of: Vec<Option<u8>> = unresolved.iter().map(|&p| kb.vocab.get(p).and_then(|i| i.arity)).collect();
    let mut out = LearnedTaxonomy::default();
    for round in 0..iters.max(1) {
        // Similarities among unresolved relations under the current vocabulary.
        let rs = RelSim::fit_with(kb, cases, usize::MAX, 0.0, 1.0, true);
        let n = unresolved.len();
        let mut sim = vec![vec![0.0f32; n]; n];
        // Similarity of *current structural names*: members of an earlier cluster
        // share their parent's row (the parent is what the contexts now mention).
        let cur = |p: Sym| kb.vocab.structural(p);
        for (i, &p) in unresolved.iter().enumerate() {
            for &(q, s) in rs.of(cur(p)) {
                for (j, &r) in unresolved.iter().enumerate() {
                    if j != i && cur(r) == q {
                        sim[i][j] = sim[i][j].max(s);
                    }
                }
            }
            for (j, &r) in unresolved.iter().enumerate() {
                if j != i && cur(r) == cur(p) && round > 0 {
                    sim[i][j] = 1.0; // already merged: stays merged unless re-split below
                }
            }
        }
        // Average-linkage agglomeration (O(n^3), n = unresolved relations).
        let mut members: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
        let mut alive: Vec<bool> = vec![true; n];
        loop {
            let mut best = (min_sim, usize::MAX, usize::MAX);
            for a in 0..n {
                if !alive[a] {
                    continue;
                }
                for b in a + 1..n {
                    if !alive[b] || ar_of[members[a][0]] != ar_of[members[b][0]] {
                        continue;
                    }
                    let tot: f32 = members[a].iter().flat_map(|&x| members[b].iter().map(move |&y| (x, y))).map(|(x, y)| sim[x][y]).sum();
                    let avg = tot / (members[a].len() * members[b].len()) as f32;
                    if avg >= best.0 {
                        best = (avg, a, b);
                    }
                }
            }
            if best.1 == usize::MAX {
                break;
            }
            let moved = std::mem::take(&mut members[best.2]);
            members[best.1].extend(moved);
            alive[best.2] = false;
        }
        // Re-declare the learned taxonomy for this round.
        for &p in &unresolved {
            kb.vocab.set_parents(p, Vec::new());
        }
        out.clusters.clear();
        let mut k = 0;
        for (a, m) in members.iter().enumerate() {
            if !alive[a] || m.len() < 2 {
                continue;
            }
            let ps: Vec<Sym> = m.iter().map(|&i| unresolved[i]).collect();
            let ar = ar_of[m[0]];
            let parent = kb.declare(&format!("~rel-{k}"), ar, PredKind::Relation, false, &[]);
            k += 1;
            for &p in &ps {
                kb.vocab.set_parents(p, vec![parent]);
            }
            out.clusters.push((parent, ps));
        }
    }
    out
}
