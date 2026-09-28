//! Transfer reliability: learning which analogical inferences to trust (E29/E30).
//!
//! Every candidate inference has a *transfer type* that describes how it
//! relates to the query, independently of the analogue it came from:
//! * a binary fact between two query entities is typed by the relation paths
//!   (length ≤ 2, `~` = traversed backwards) linking them in the query, e.g.
//!   `educated-at<=advisor.employer` ("studied where the advisor worked");
//! * a fact about a hypothesized new entity is typed `f<=new:<positions>`
//!   (the analogue's own entity is carried over: a copy);
//! * anything else by its argument kinds, `f<=[TSX]` (Target entity,
//!   Skolem, eXpression).
//!
//! Feedback on inferences (confirmed / refuted) updates per-type counts; the
//! reliability of an inference is the best smoothed precision among its types.
//! Types with enough evidence read as rules induced from analogy.

use mars_map::Proj;
use mars_rel::{CaseId, Kb, PredKind, Sym, Term};
use rustc_hash::{FxHashMap, FxHashSet};

/// Pseudo-votes of the prior (the global precision) in each type's estimate.
pub const PRIOR_WEIGHT: f64 = 5.0;

/// Support is bucketed as 0 (rule, no analogue), 1, 2, 3, 4, 5+.
pub const MAX_SUPPORT_BUCKET: usize = 5;

/// A type's key conditioned on support (`key#s`).
pub fn support_key(key: &str, support: usize) -> String {
    format!("{key}#{}", support.min(MAX_SUPPORT_BUCKET))
}

/// The keys recorded for one outcome: the types and their support-conditioned
/// versions (E32: corroboration is calibrated, so a copy proposed by 5
/// analogues is far more reliable than one proposed by 1).
pub fn outcome_keys(keys: &[String], support: usize) -> Vec<String> {
    keys.iter().cloned().chain(keys.iter().map(|k| support_key(k, support))).collect()
}

/// Transfer types of a projected inference against query case `q`.
pub fn transfer_keys(kb: &Kb, q: CaseId, proj: &Proj) -> Vec<String> {
    let Proj::Expr { functor, args } = proj else { return vec!["other".into()] };
    let f = kb.name(*functor);
    let new: Vec<String> = args.iter().enumerate().filter(|(_, a)| matches!(a, Proj::Skolem(_))).map(|(i, _)| i.to_string()).collect();
    if !new.is_empty() {
        return vec![format!("{f}<=new:{}", new.join(","))];
    }
    if let [Proj::Target(Term::Ent(a)), Proj::Target(Term::Ent(b))] = args.as_slice() {
        return pair_keys(kb, q, f, *a, *b);
    }
    let kinds: String = args
        .iter()
        .map(|a| match a {
            Proj::Target(Term::Ent(_)) => 'T',
            Proj::Skolem(_) => 'S',
            _ => 'X',
        })
        .collect();
    vec![format!("{f}<=[{kinds}]")]
}

/// Transfer types of a binary fact `f(a, b)` between query entities.
pub fn pair_keys(kb: &Kb, q: CaseId, f: &str, a: Sym, b: Sym) -> Vec<String> {
    let p = paths(kb, q, a, b);
    if p.is_empty() {
        vec![format!("{f}<=unlinked")]
    } else {
        p.into_iter().map(|p| format!("{f}<={p}")).collect()
    }
}

/// Entity pairs (x, y) of case `c` linked by a rule body (`g`, `g~`, `g.h`, …).
pub fn rule_pairs(kb: &Kb, c: CaseId, body: &str) -> Vec<(Sym, Sym)> {
    let st = steps(kb, c);
    let parts: Vec<&str> = body.split('.').collect();
    let mut out: FxHashSet<(Sym, Sym)> = FxHashSet::default();
    match parts.as_slice() {
        [g] => out.extend(st.iter().filter(|s| s.0 == *g).map(|s| (s.1, s.2))),
        [g, h] => {
            for (_, x, z) in st.iter().filter(|s| s.0 == *g) {
                for (_, _, y) in st.iter().filter(|s| s.0 == *h && s.1 == *z) {
                    if y != x {
                        out.insert((*x, *y));
                    }
                }
            }
        }
        _ => {}
    }
    let mut v: Vec<(Sym, Sym)> = out.into_iter().collect();
    v.sort();
    v
}

/// Binary relation steps of case `c` as (name, from, to), each fact in both
/// directions (`name~` backwards).
fn steps(kb: &Kb, c: CaseId) -> Vec<(String, Sym, Sym)> {
    let mut steps: Vec<(String, Sym, Sym)> = Vec::new();
    for &f in &kb.case(c).facts {
        let e = kb.expr(f);
        if kb.vocab.kind(e.functor) != PredKind::Relation {
            continue;
        }
        if let [Term::Ent(x), Term::Ent(y)] = e.args.as_slice() {
            let n = kb.name(e.functor);
            steps.push((n.to_string(), *x, *y));
            steps.push((format!("{n}~"), *y, *x));
        }
    }
    steps
}

/// Relation paths of length ≤ 2 from `a` to `b` among the binary relation facts of case `c`.
fn paths(kb: &Kb, c: CaseId, a: Sym, b: Sym) -> Vec<String> {
    let steps = steps(kb, c);
    let mut out: FxHashSet<String> = FxHashSet::default();
    for (n1, x, z) in &steps {
        if *x != a {
            continue;
        }
        if *z == b {
            out.insert(n1.clone());
            continue;
        }
        for (n2, z2, y) in &steps {
            if z2 == z && *y == b && *z != a {
                out.insert(format!("{n1}.{n2}"));
            }
        }
    }
    let mut v: Vec<String> = out.into_iter().collect();
    v.sort();
    v
}

/// Per-type outcome counts.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransferStats {
    counts: FxHashMap<String, (f64, f64)>,
    hits: f64,
    n: f64,
}

impl TransferStats {
    /// Record the outcome of one inference with these types.
    pub fn record(&mut self, keys: &[String], correct: bool) {
        let h = correct as u8 as f64;
        self.hits += h;
        self.n += 1.0;
        for k in keys {
            let e = self.counts.entry(k.clone()).or_insert((0.0, 0.0));
            e.0 += h;
            e.1 += 1.0;
        }
    }

    /// Global precision of all recorded inferences (Laplace-smoothed; 0.5 when empty).
    pub fn prior(&self) -> f64 {
        (self.hits + 1.0) / (self.n + 2.0)
    }

    /// Smoothed precision of one type.
    pub fn precision(&self, key: &str) -> f64 {
        let p0 = self.prior();
        self.counts.get(key).map(|&(h, n)| (h + PRIOR_WEIGHT * p0) / (n + PRIOR_WEIGHT)).unwrap_or(p0)
    }

    /// Reliability of an inference with this support: for each type, the
    /// precision at this support level, smoothed towards the type's precision
    /// (hierarchical back-off: support level → type → global); the best over types.
    pub fn reliability_at(&self, keys: &[String], support: usize) -> f64 {
        if keys.is_empty() {
            return self.prior();
        }
        keys.iter()
            .map(|k| {
                let pt = self.precision(k);
                self.counts.get(&support_key(k, support)).map(|&(h, n)| (h + PRIOR_WEIGHT * pt) / (n + PRIOR_WEIGHT)).unwrap_or(pt)
            })
            .fold(0.0, f64::max)
    }

    /// Reliability of an inference: the best precision among its types.
    pub fn reliability(&self, keys: &[String]) -> f64 {
        if keys.is_empty() {
            return self.prior();
        }
        keys.iter().map(|k| self.precision(k)).fold(0.0, f64::max)
    }

    /// (type, raw precision, count) for types with at least `min_n` outcomes, best first.
    pub fn table(&self, min_n: f64) -> Vec<(String, f64, f64)> {
        let mut v: Vec<(String, f64, f64)> = self.counts.iter().filter(|(k, c)| c.1 >= min_n && !k.contains('#')).map(|(k, &(h, n))| (k.clone(), h / n, n)).collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1).then(b.2.total_cmp(&a.2)).then(a.0.cmp(&b.0)));
        v
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0.0
    }

    /// Snapshot lines (`transfer-total H N`, `transfer-counts KEY H N`).
    pub(crate) fn to_meta(&self) -> String {
        let mut s = format!("transfer-total {} {}\n", self.hits, self.n);
        let mut keys: Vec<&String> = self.counts.keys().collect();
        keys.sort();
        for k in keys {
            let (h, n) = self.counts[k];
            s.push_str(&format!("transfer-counts {k} {h} {n}\n"));
        }
        s
    }

    pub(crate) fn set_total(&mut self, hits: f64, n: f64) {
        self.hits = hits;
        self.n = n;
    }

    pub(crate) fn set_counts(&mut self, key: &str, hits: f64, n: f64) {
        self.counts.insert(key.to_string(), (hits, n));
    }
}

/// Render a transfer type as a rule, e.g. `educated-at<=advisor.employer` →
/// `educated-at(x, y) ⇐ advisor(x, z) ∧ employer(z, y)`.
pub fn render_rule(key: &str) -> String {
    let Some((head, body)) = key.split_once("<=") else { return key.to_string() };
    if body.starts_with("new:") || body.starts_with('[') || body == "unlinked" {
        return format!("{head}: {body}");
    }
    let steps: Vec<&str> = body.split('.').collect();
    let vars = ["x", "z", "y"];
    let names: Vec<&str> = if steps.len() == 1 { vec!["x", "y"] } else { vars.to_vec() };
    let atoms: Vec<String> = steps
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let (from, to) = (names[i], names[i + 1]);
            match s.strip_suffix('~') {
                Some(r) => format!("{r}({to}, {from})"),
                None => format!("{s}({from}, {to})"),
            }
        })
        .collect();
    format!("{head}(x, y) ⇐ {}", atoms.join(" ∧ "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_render_paths_with_directions() {
        assert_eq!(render_rule("edu<=adv.emp"), "edu(x, y) ⇐ adv(x, z) ∧ emp(z, y)");
        assert_eq!(render_rule("writer<=director"), "writer(x, y) ⇐ director(x, y)");
        assert_eq!(render_rule("a<=b~.c"), "a(x, y) ⇐ b(z, x) ∧ c(z, y)");
        assert_eq!(render_rule("f<=new:1"), "f: new:1");
    }

    #[test]
    fn reliability_is_smoothed_towards_the_prior() {
        let mut s = TransferStats::default();
        assert_eq!(s.reliability(&["a".into()]), 0.5);
        for _ in 0..20 {
            s.record(&["good".into()], true);
            s.record(&["bad".into()], false);
        }
        let (g, b) = (s.precision("good"), s.precision("bad"));
        assert!(g > 0.85 && b < 0.15, "{g} {b}");
        assert_eq!(s.reliability(&["good".into(), "bad".into()]), g);
        assert!((s.precision("unseen") - s.prior()).abs() < 1e-12);
    }
}
