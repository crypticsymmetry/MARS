//! Knowledge base: hash-consed expression DAG plus cases.
//!
//! Every expression is interned once (`ExprId`), so structurally identical
//! sub-expressions are shared across all cases and equality is O(1).
//! Arguments of commutative predicates are stored in canonical (sorted) order.

use crate::vocab::{Interner, PredInfo, PredKind, Sym, Vocabulary};
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExprId(pub u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CaseId(pub u32);

/// An argument: an entity symbol or a nested expression.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Term {
    Ent(Sym),
    Expr(ExprId),
}

pub type Args = SmallVec<[Term; 3]>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Expr {
    pub functor: Sym,
    pub args: Args,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CaseKind {
    Episode,
    Schema,
    Query,
}

#[derive(Clone, Debug)]
pub struct Case {
    pub id: CaseId,
    pub name: Sym,
    pub kind: CaseKind,
    pub version: u64,
    /// Top-level facts, deduplicated, in insertion order.
    pub facts: Vec<ExprId>,
}

#[derive(Default, Clone, Debug)]
pub struct Kb {
    pub interner: Interner,
    pub vocab: Vocabulary,
    exprs: Vec<Expr>,
    /// Relational order: 1 for expressions over entities only, else 1 + max child order.
    order: Vec<u8>,
    index: FxHashMap<Expr, ExprId>,
    cases: Vec<Case>,
    case_by_name: FxHashMap<Sym, CaseId>,
}

impl Kb {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sym(&mut self, s: &str) -> Sym {
        self.interner.intern(s)
    }

    pub fn name(&self, s: Sym) -> &str {
        self.interner.name(s)
    }

    /// Declare a predicate by name.
    pub fn declare(&mut self, name: &str, arity: Option<u8>, kind: PredKind, commutative: bool, parents: &[&str]) -> Sym {
        let s = self.sym(name);
        let parents = parents.iter().map(|p| self.interner.intern(p)).collect();
        self.vocab.declare(PredInfo { name: s, arity, kind, commutative, parents, declared: true, canonical: true });
        s
    }

    // ---------------------------------------------------------------- exprs

    pub fn intern_expr(&mut self, functor: Sym, args: impl IntoIterator<Item = Term>) -> ExprId {
        let mut args: Args = args.into_iter().collect();
        self.vocab.ensure(functor, args.len());
        if self.vocab.is_commutative(functor) {
            args.sort_unstable();
        }
        let e = Expr { functor, args };
        if let Some(&id) = self.index.get(&e) {
            return id;
        }
        let ord = 1 + e
            .args
            .iter()
            .map(|t| match t {
                Term::Ent(_) => 0,
                Term::Expr(x) => self.order[x.0 as usize],
            })
            .max()
            .unwrap_or(0);
        let id = ExprId(self.exprs.len() as u32);
        self.exprs.push(e.clone());
        self.order.push(ord);
        self.index.insert(e, id);
        id
    }

    /// Lookup without interning.
    pub fn find_expr(&self, functor: Sym, args: &[Term]) -> Option<ExprId> {
        let mut args: Args = args.iter().copied().collect();
        if self.vocab.is_commutative(functor) {
            args.sort_unstable();
        }
        self.index.get(&Expr { functor, args }).copied()
    }

    #[inline]
    pub fn expr(&self, id: ExprId) -> &Expr {
        &self.exprs[id.0 as usize]
    }

    #[inline]
    pub fn order(&self, id: ExprId) -> u8 {
        self.order[id.0 as usize]
    }

    pub fn n_exprs(&self) -> usize {
        self.exprs.len()
    }

    // ---------------------------------------------------------------- cases

    pub fn add_case(&mut self, name: &str, kind: CaseKind, facts: impl IntoIterator<Item = ExprId>) -> CaseId {
        let name_sym = self.sym(name);
        assert!(!self.case_by_name.contains_key(&name_sym), "duplicate case name {name}");
        let mut seen = FxHashSet::default();
        let facts: Vec<ExprId> = facts.into_iter().filter(|f| seen.insert(*f)).collect();
        let id = CaseId(self.cases.len() as u32);
        self.cases.push(Case { id, name: name_sym, kind, version: 0, facts });
        self.case_by_name.insert(name_sym, id);
        id
    }

    #[inline]
    pub fn case(&self, id: CaseId) -> &Case {
        &self.cases[id.0 as usize]
    }

    pub fn case_by_name(&self, name: &str) -> Option<CaseId> {
        self.interner.get(name).and_then(|s| self.case_by_name.get(&s).copied())
    }

    pub fn cases(&self) -> &[Case] {
        &self.cases
    }

    pub fn n_cases(&self) -> usize {
        self.cases.len()
    }

    /// Add a fact to a case (no-op if present). Bumps the version.
    pub fn add_fact(&mut self, case: CaseId, fact: ExprId) -> bool {
        let c = &mut self.cases[case.0 as usize];
        if c.facts.contains(&fact) {
            return false;
        }
        c.facts.push(fact);
        c.version += 1;
        true
    }

    /// Remove a fact from a case. Bumps the version.
    pub fn remove_fact(&mut self, case: CaseId, fact: ExprId) -> bool {
        let c = &mut self.cases[case.0 as usize];
        if let Some(pos) = c.facts.iter().position(|&f| f == fact) {
            c.facts.remove(pos);
            c.version += 1;
            true
        } else {
            false
        }
    }

    /// All expressions reachable from the case's facts, children before
    /// parents (post-order), each exactly once.
    pub fn case_exprs(&self, case: CaseId) -> Vec<ExprId> {
        let mut out = Vec::new();
        let mut seen = FxHashSet::default();
        for &f in &self.case(case).facts {
            self.postorder(f, &mut seen, &mut out);
        }
        out
    }

    fn postorder(&self, e: ExprId, seen: &mut FxHashSet<ExprId>, out: &mut Vec<ExprId>) {
        if !seen.insert(e) {
            return;
        }
        for t in &self.expr(e).args {
            if let Term::Expr(c) = *t {
                self.postorder(c, seen, out);
            }
        }
        out.push(e);
    }

    /// Distinct entities mentioned by the case, in first-seen order.
    pub fn case_entities(&self, case: CaseId) -> Vec<Sym> {
        let mut out = Vec::new();
        let mut seen = FxHashSet::default();
        for e in self.case_exprs(case) {
            for t in &self.expr(e).args {
                if let Term::Ent(s) = *t {
                    if seen.insert(s) {
                        out.push(s);
                    }
                }
            }
        }
        out
    }

    // ---------------------------------------------------------------- render

    pub fn render_term(&self, t: Term) -> String {
        match t {
            Term::Ent(s) => self.name(s).to_string(),
            Term::Expr(e) => self.render_expr(e),
        }
    }

    pub fn render_expr(&self, id: ExprId) -> String {
        let e = self.expr(id);
        let mut s = String::from("(");
        s.push_str(self.name(e.functor));
        for a in &e.args {
            s.push(' ');
            s.push_str(&self.render_term(*a));
        }
        s.push(')');
        s
    }

    pub fn render_case(&self, id: CaseId) -> String {
        let c = self.case(id);
        let mut s = format!("(defcase {}", self.name(c.name));
        for &f in &c.facts {
            s.push_str("\n  ");
            s.push_str(&self.render_expr(f));
        }
        s.push(')');
        s
    }

    pub fn render_vocab(&self) -> String {
        let mut preds: Vec<&PredInfo> = self.vocab.iter().filter(|p| p.declared).collect();
        preds.sort_by_key(|p| self.name(p.name).to_string());
        let mut s = String::new();
        for p in preds {
            s.push_str(&format!("(defpredicate {}", self.name(p.name)));
            match p.arity {
                Some(a) => s.push_str(&format!(" :arity {a}")),
                None => s.push_str(" :arity *"),
            }
            s.push_str(&format!(" :kind {}", p.kind.as_str()));
            if p.commutative {
                s.push_str(" :commutative t");
            }
            if !p.canonical {
                s.push_str(" :canonical nil");
            }
            if !p.parents.is_empty() {
                let ps: Vec<&str> = p.parents.iter().map(|x| self.name(*x)).collect();
                s.push_str(&format!(" :parents ({})", ps.join(" ")));
            }
            s.push_str(")\n");
        }
        s
    }
}
