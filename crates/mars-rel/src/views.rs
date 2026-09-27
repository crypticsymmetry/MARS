//! Alternative *views* of a case (E18): the same content at a different
//! granularity.
//!
//! [`FlatView`] turns *wrapping* encodings of context — a statement inside
//! a loop inside a branch as one fact `(in-loop i (guards t (swap …)))` —
//! into explicit context entities: each distinct context path becomes a
//! block entity, and each wrapped core becomes a small fact placed in it:
//!
//! ```text
//! (loop-block b1 i) (guard-block b2 t) (within b2 b1) (in-block b2 (swap …))
//! ```
//!
//! Wrappers are binary relations whose second argument is the wrapped fact;
//! the default set is the code converters' control relations.

use crate::{CaseId, ExprId, Kb, PredKind, Sym, Term};
use rustc_hash::FxHashMap;
use std::collections::HashSet;

/// Code converters' control wrappers and the block predicate each maps to.
pub const CODE_WRAPPERS: [(&str, &str); 4] = [("in-loop", "loop-block"), ("while-loop", "while-block"), ("guards", "guard-block"), ("inlined", "inlined-block")];

pub struct FlatView {
    /// (wrapper predicate, block predicate)
    pub wrap: Vec<(Sym, Sym)>,
    pub within: Sym,
    pub in_block: Sym,
}

impl FlatView {
    /// Declare the block predicates for `wrappers` in `kb`.
    pub fn declare(kb: &mut Kb, wrappers: &[(&str, &str)]) -> FlatView {
        let mut wrap = Vec::new();
        for (w, b) in wrappers {
            let ws = kb.sym(w);
            wrap.push((ws, kb.declare(b, Some(2), PredKind::Relation, false, &[])));
        }
        FlatView { wrap, within: kb.declare("within", Some(2), PredKind::Relation, false, &[]), in_block: kb.declare("in-block", Some(2), PredKind::Relation, false, &[]) }
    }

    pub fn is_wrapper(&self, f: Sym) -> bool {
        self.wrap.iter().any(|w| w.0 == f)
    }

    fn is_block_pred(&self, f: Sym) -> bool {
        self.wrap.iter().any(|w| w.1 == f)
    }

    /// Peel wrappers: (context path of (wrapper, head term), core).
    pub fn peel(&self, kb: &Kb, f: ExprId) -> (Vec<(Sym, Term)>, ExprId) {
        let mut path = Vec::new();
        let mut cur = f;
        loop {
            let ex = kb.expr(cur);
            match (self.is_wrapper(ex.functor), ex.args.len(), ex.args.get(1)) {
                (true, 2, Some(&Term::Expr(inner))) => {
                    path.push((ex.functor, ex.args[0]));
                    cur = inner;
                }
                _ => return (path, cur),
            }
        }
    }

    /// The flat facts of `case` (block entities named `{prefix}b{n}`), and
    /// for each original fact the flat fact holding its core.
    pub fn flatten(&self, kb: &mut Kb, case: CaseId, prefix: &str) -> (Vec<ExprId>, FxHashMap<ExprId, ExprId>) {
        let facts = kb.case(case).facts.clone();
        let mut blocks: FxHashMap<Vec<(Sym, Term)>, Sym> = FxHashMap::default();
        let mut out: Vec<ExprId> = Vec::new();
        let mut core_fact = FxHashMap::default();
        for f in facts {
            let (path, core) = self.peel(kb, f);
            if path.is_empty() {
                out.push(f);
                core_fact.insert(f, f);
                continue;
            }
            let mut parent: Option<Sym> = None;
            for d in 1..=path.len() {
                let key = path[..d].to_vec();
                let b = match blocks.get(&key) {
                    Some(&b) => b,
                    None => {
                        let b = kb.sym(&format!("{prefix}b{}", blocks.len() + 1));
                        blocks.insert(key, b);
                        let (w, head) = path[d - 1];
                        let bp = self.wrap.iter().find(|x| x.0 == w).unwrap().1;
                        out.push(kb.intern_expr(bp, [Term::Ent(b), head]));
                        if let Some(p) = parent {
                            out.push(kb.intern_expr(self.within, [Term::Ent(b), Term::Ent(p)]));
                        }
                        b
                    }
                };
                parent = Some(b);
            }
            let fact = kb.intern_expr(self.in_block, [Term::Ent(parent.unwrap()), Term::Expr(core)]);
            out.push(fact);
            core_fact.insert(f, fact);
        }
        out.sort_unstable();
        out.dedup();
        (out, core_fact)
    }

    /// Remove block facts left without content (after deleting statements).
    pub fn prune(&self, kb: &Kb, facts: &mut Vec<ExprId>) {
        loop {
            let used: HashSet<Sym> = facts
                .iter()
                .filter_map(|&f| {
                    let ex = kb.expr(f);
                    let slot = if ex.functor == self.within { 1 } else if ex.functor == self.in_block { 0 } else { return None };
                    match ex.args[slot] {
                        Term::Ent(b) => Some(b),
                        _ => None,
                    }
                })
                .collect();
            let before = facts.len();
            facts.retain(|&f| {
                let ex = kb.expr(f);
                match (self.is_block_pred(ex.functor) || ex.functor == self.within, ex.args[0]) {
                    (true, Term::Ent(b)) => used.contains(&b),
                    _ => true,
                }
            });
            if facts.len() == before {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CaseKind;

    #[test]
    fn flattens_shared_contexts_into_blocks() {
        let mut kb = Kb::new();
        kb.load_str(
            "(defcase f (param a) (in-loop i (in-loop j (guards (lt x y) (swap x y)))) (in-loop i (assign k i)))",
        )
        .unwrap();
        let fv = FlatView::declare(&mut kb, &CODE_WRAPPERS);
        let c = kb.case_by_name("f").unwrap();
        let (facts, core) = fv.flatten(&mut kb, c, "f");
        let text: HashSet<String> = facts.iter().map(|&f| kb.render_expr(f)).collect();
        for want in ["(param a)", "(loop-block fb1 i)", "(loop-block fb2 j)", "(within fb2 fb1)", "(guard-block fb3 (lt x y))", "(within fb3 fb2)", "(in-block fb3 (swap x y))", "(in-block fb1 (assign k i))"] {
            assert!(text.contains(want), "missing {want} in {text:?}");
        }
        assert_eq!(facts.len(), 8);
        assert_eq!(core.len(), 3);
        // Deleting the swap leaves blocks fb2/fb3 empty: they are pruned, fb1 stays.
        let swap = *core.values().find(|&&f| kb.render_expr(f).contains("swap")).unwrap();
        let mut kept: Vec<ExprId> = facts.iter().copied().filter(|&f| f != swap).collect();
        fv.prune(&kb, &mut kept);
        let kept_text: Vec<String> = kept.iter().map(|&f| kb.render_expr(f)).collect();
        assert_eq!(kept.len(), 3, "{kept_text:?}");
        let _ = kb.add_case("g", CaseKind::Episode, kept);
    }
}
