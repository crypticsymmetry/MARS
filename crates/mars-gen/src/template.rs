//! Structural templates: small predicate-calculus DAGs over variables.
//!
//! Templates are drawn at random from seven structural *families*. They are
//! random draws, not hand-designed to suit the encoder (circularity
//! control), and whole families are held out for testing.

use crate::vocab::{fo_predicates, COMPARISONS, FUNCTIONS, HO_PREDS};
use mars_hv::Rng;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Family {
    Chain,
    Star,
    Tree,
    Loop,
    DeepHo,
    Comparison,
    Mixed,
}

impl Family {
    pub const ALL: [Family; 7] =
        [Family::Chain, Family::Star, Family::Tree, Family::Loop, Family::DeepHo, Family::Comparison, Family::Mixed];
    /// Families available while designing/tuning features.
    pub const DEV: [Family; 4] = [Family::Chain, Family::Star, Family::Tree, Family::Loop];
    /// Held-out families for reporting.
    pub const TEST: [Family; 3] = [Family::DeepHo, Family::Comparison, Family::Mixed];

    pub fn name(self) -> &'static str {
        match self {
            Family::Chain => "chain",
            Family::Star => "star",
            Family::Tree => "tree",
            Family::Loop => "loop",
            Family::DeepHo => "deep-ho",
            Family::Comparison => "comparison",
            Family::Mixed => "mixed",
        }
    }

    pub fn is_test(self) -> bool {
        Family::TEST.contains(&self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TArg {
    Var(usize),
    Node(usize),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PredRef {
    /// First-order relation (canonical name; surfaced per domain).
    Fo(&'static str),
    /// Higher-order relation over expressions.
    Ho(&'static str),
    /// Function term, e.g. `(mass x)`; not a fact by itself.
    Func(&'static str),
    /// Comparison of function terms (first-order level).
    Cmp(&'static str),
    /// Logical conjunction; only used as an argument.
    And,
}

impl PredRef {
    /// First-order level: facts over entities (or function terms).
    pub fn is_fo_level(self) -> bool {
        matches!(self, PredRef::Fo(_) | PredRef::Cmp(_))
    }
    pub fn is_fact(self) -> bool {
        !matches!(self, PredRef::Func(_) | PredRef::And)
    }
    pub fn canonical(self) -> &'static str {
        match self {
            PredRef::Fo(s) | PredRef::Ho(s) | PredRef::Func(s) | PredRef::Cmp(s) => s,
            PredRef::And => crate::vocab::AND,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TNode {
    pub pred: PredRef,
    pub args: Vec<TArg>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Template {
    pub family: Family,
    pub n_vars: usize,
    /// Topologically ordered: node args only reference earlier nodes.
    pub nodes: Vec<TNode>,
}

impl Template {
    pub fn n_fo_level(&self) -> usize {
        self.nodes.iter().filter(|n| n.pred.is_fo_level()).count()
    }
    pub fn n_ho(&self) -> usize {
        self.nodes.iter().filter(|n| matches!(n.pred, PredRef::Ho(_))).count()
    }
}

struct B {
    nodes: Vec<TNode>,
}

impl B {
    fn push(&mut self, pred: PredRef, args: Vec<TArg>) -> usize {
        self.nodes.push(TNode { pred, args });
        self.nodes.len() - 1
    }
}

fn fo(rng: &mut Rng, fos: &[&'static str]) -> PredRef {
    PredRef::Fo(fos[rng.index(fos.len())])
}

fn ho(rng: &mut Rng) -> PredRef {
    PredRef::Ho(HO_PREDS[rng.index(HO_PREDS.len())])
}

fn oriented(rng: &mut Rng, a: usize, b: usize) -> Vec<TArg> {
    if rng.bernoulli(0.5) {
        vec![TArg::Var(a), TArg::Var(b)]
    } else {
        vec![TArg::Var(b), TArg::Var(a)]
    }
}

/// Draw a random template from a family.
pub fn generate(family: Family, rng: &mut Rng) -> Template {
    let fos = fo_predicates();
    let mut b = B { nodes: Vec::new() };
    let n_vars;
    match family {
        Family::Chain => {
            let len = rng.range_inclusive(3, 5);
            n_vars = len + 1;
            let f: Vec<usize> = (0..len).map(|i| b.push(fo(rng, &fos), vec![TArg::Var(i), TArg::Var(i + 1)])).collect();
            let mut any = false;
            for i in 0..len - 1 {
                if rng.bernoulli(0.7) || (!any && i == len - 2) {
                    b.push(ho(rng), vec![TArg::Node(f[i]), TArg::Node(f[i + 1])]);
                    any = true;
                }
            }
        }
        Family::Star => {
            let k = rng.range_inclusive(3, 5);
            n_vars = k + 1;
            let f: Vec<usize> = (1..=k).map(|v| { let a = oriented(rng, 0, v); b.push(fo(rng, &fos), a) }).collect();
            let mut any = false;
            for j in 1..k {
                if rng.bernoulli(0.7) || (!any && j == k - 1) {
                    b.push(ho(rng), vec![TArg::Node(f[0]), TArg::Node(f[j])]);
                    any = true;
                }
            }
        }
        Family::Tree => {
            n_vars = rng.range_inclusive(4, 6);
            let mut edges = Vec::new(); // (node, u, v)
            for v in 1..n_vars {
                let u = rng.index(v);
                let args = oriented(rng, u, v);
                let n = b.push(fo(rng, &fos), args);
                edges.push((n, u, v));
            }
            let mut pairs = Vec::new();
            for i in 0..edges.len() {
                for j in 0..edges.len() {
                    let (a, b2) = (edges[i], edges[j]);
                    let share = a.1 == b2.1 || a.1 == b2.2 || a.2 == b2.1 || a.2 == b2.2;
                    if i != j && share {
                        pairs.push((a.0, b2.0));
                    }
                }
            }
            rng.shuffle(&mut pairs);
            let n_ho = rng.range_inclusive(1, 3).min(pairs.len());
            let mut used = Vec::new();
            for &(x, y) in &pairs {
                if used.len() >= n_ho {
                    break;
                }
                if used.iter().any(|&(a, c)| (a == x && c == y) || (a == y && c == x)) {
                    continue;
                }
                used.push((x, y));
                b.push(ho(rng), vec![TArg::Node(x), TArg::Node(y)]);
            }
        }
        Family::Loop => {
            let n = rng.range_inclusive(3, 5);
            n_vars = n;
            let f: Vec<usize> = (0..n).map(|i| b.push(fo(rng, &fos), vec![TArg::Var(i), TArg::Var((i + 1) % n)])).collect();
            let neg = rng.bernoulli(0.5).then(|| rng.index(n));
            for i in 0..n {
                let p = if Some(i) == neg { PredRef::Ho("prevent") } else { PredRef::Ho("cause") };
                b.push(p, vec![TArg::Node(f[i]), TArg::Node(f[(i + 1) % n])]);
            }
        }
        Family::DeepHo => {
            let m = rng.range_inclusive(3, 4);
            n_vars = m + 1;
            let f: Vec<usize> = (0..m).map(|i| { let a = oriented(rng, i, i + 1); b.push(fo(rng, &fos), a) }).collect();
            let a = b.push(PredRef::And, vec![TArg::Node(f[0]), TArg::Node(f[1])]);
            let h1 = b.push(ho(rng), vec![TArg::Node(a), TArg::Node(f[2])]);
            if m == 4 {
                b.push(ho(rng), vec![TArg::Node(h1), TArg::Node(f[3])]);
            }
        }
        Family::Comparison => {
            n_vars = rng.range_inclusive(3, 4);
            let mut pairs = Vec::new();
            for i in 0..n_vars - 1 {
                let func = PredRef::Func(FUNCTIONS[rng.index(FUNCTIONS.len())]);
                let fa = b.push(func, vec![TArg::Var(i)]);
                let fb = b.push(func, vec![TArg::Var(i + 1)]);
                let cmp = PredRef::Cmp(COMPARISONS[if rng.bernoulli(0.8) { 0 } else { 1 }]);
                let c = b.push(cmp, vec![TArg::Node(fa), TArg::Node(fb)]);
                let r = { let a = oriented(rng, i, i + 1); b.push(fo(rng, &fos), a) };
                pairs.push((c, r));
            }
            for (c, r) in pairs {
                b.push(PredRef::Ho("cause"), vec![TArg::Node(c), TArg::Node(r)]);
            }
        }
        Family::Mixed => {
            n_vars = rng.range_inclusive(4, 6);
            let m = rng.range_inclusive(4, 7);
            let mut f = Vec::new();
            let mut seen = Vec::new();
            while f.len() < m {
                let (u, v) = (rng.index(n_vars), rng.index(n_vars));
                if u == v || seen.contains(&(u, v)) {
                    continue;
                }
                seen.push((u, v));
                f.push(b.push(fo(rng, &fos), vec![TArg::Var(u), TArg::Var(v)]));
            }
            let n_ho = rng.range_inclusive(1, 3);
            let mut hos = Vec::new();
            for _ in 0..n_ho {
                let idx = rng.sample_indices(f.len(), 3);
                let lhs = if rng.bernoulli(0.3) {
                    b.push(PredRef::And, vec![TArg::Node(f[idx[0]]), TArg::Node(f[idx[2]])])
                } else {
                    f[idx[0]]
                };
                hos.push(b.push(ho(rng), vec![TArg::Node(lhs), TArg::Node(f[idx[1]])]));
            }
            if rng.bernoulli(0.3) {
                let h = hos[rng.index(hos.len())];
                let x = f[rng.index(f.len())];
                b.push(ho(rng), vec![TArg::Node(h), TArg::Node(x)]);
            }
        }
    }
    Template { family, n_vars, nodes: b.nodes }
}

/// Structural signature of the higher-order layer, independent of variable
/// names: multiset of (HO predicate, argument predicates, argument sharing).
fn ho_signature(t: &Template) -> Vec<String> {
    let desc = |i: usize| -> String { t.nodes[i].pred.canonical().to_string() };
    let vars_of = |i: usize| -> Vec<usize> {
        let mut out = Vec::new();
        let mut stack = vec![i];
        while let Some(x) = stack.pop() {
            for a in &t.nodes[x].args {
                match *a {
                    TArg::Var(v) => out.push(v),
                    TArg::Node(c) => stack.push(c),
                }
            }
        }
        out
    };
    let mut sig: Vec<String> = t
        .nodes
        .iter()
        .filter(|n| matches!(n.pred, PredRef::Ho(_) | PredRef::And))
        .map(|n| {
            let mut args: Vec<String> = n
                .args
                .iter()
                .map(|a| match *a {
                    TArg::Node(c) => desc(c),
                    TArg::Var(v) => format!("v{v}"),
                })
                .collect();
            if n.pred == PredRef::And {
                // Commutative: argument order carries no structure.
                args.sort();
            }
            let share = if n.args.len() == 2 {
                match (n.args[0], n.args[1]) {
                    (TArg::Node(a), TArg::Node(c)) => {
                        let (va, vc) = (vars_of(a), vars_of(c));
                        va.iter().any(|x| vc.contains(x))
                    }
                    _ => false,
                }
            } else {
                false
            };
            format!("{}({})/{share}", n.pred.canonical(), args.join(","))
        })
        .collect();
    sig.sort();
    sig
}

/// Karla-style re-wiring: keep all first-order-level facts, keep the
/// higher-order predicates, but permute which first-order facts they connect.
/// Returns `None` if no structurally different re-wiring was found.
pub fn rewire(t: &Template, rng: &mut Rng) -> Option<Template> {
    let fo_idx: Vec<usize> = (0..t.nodes.len()).filter(|&i| t.nodes[i].pred.is_fo_level()).collect();
    let first_ho = t.nodes.iter().position(|n| matches!(n.pred, PredRef::Ho(_) | PredRef::And))?;
    // All FO-level nodes must precede the HO layer so permuted references stay topological.
    if fo_idx.iter().any(|&i| i > first_ho) || fo_idx.len() < 2 {
        return None;
    }
    let orig = ho_signature(t);
    for _ in 0..64 {
        let mut perm = fo_idx.clone();
        rng.shuffle(&mut perm);
        let map = |i: usize| -> usize { fo_idx.iter().position(|&x| x == i).map(|p| perm[p]).unwrap_or(i) };
        let mut nt = t.clone();
        for n in nt.nodes.iter_mut().skip(first_ho) {
            for a in n.args.iter_mut() {
                if let TArg::Node(c) = a {
                    *c = map(*c);
                }
            }
        }
        if ho_signature(&nt) != orig {
            return Some(nt);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_are_topological_and_nonempty() {
        let mut rng = Rng::new(1);
        for fam in Family::ALL {
            for _ in 0..50 {
                let t = generate(fam, &mut rng);
                assert!(t.n_fo_level() >= 2, "{fam:?}");
                assert!(t.n_ho() >= 1, "{fam:?} {t:?}");
                for (i, n) in t.nodes.iter().enumerate() {
                    for a in &n.args {
                        match *a {
                            TArg::Node(c) => assert!(c < i),
                            TArg::Var(v) => assert!(v < t.n_vars),
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn rewire_changes_ho_structure_only() {
        let mut rng = Rng::new(2);
        let mut ok = 0;
        for fam in Family::ALL {
            for _ in 0..30 {
                let t = generate(fam, &mut rng);
                if let Some(r) = rewire(&t, &mut rng) {
                    ok += 1;
                    let fo_a: Vec<_> = t.nodes.iter().filter(|n| n.pred.is_fo_level()).collect();
                    let fo_b: Vec<_> = r.nodes.iter().filter(|n| n.pred.is_fo_level()).collect();
                    assert_eq!(fo_a, fo_b);
                    assert_ne!(t.nodes, r.nodes);
                }
            }
        }
        assert!(ok > 180, "rewire succeeded only {ok}/210");
    }
}
