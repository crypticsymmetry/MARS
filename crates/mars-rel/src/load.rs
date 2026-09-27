//! Loading `.mars` source: `(defpredicate ...)` and `(defcase ...)` forms.
//!
//! ```text
//! (defpredicate attracts :arity 2 :kind relation :parents (force-relation))
//! (defpredicate and :arity * :kind logical :commutative t)
//! (defcase solar-system
//!   (attracts sun planet)
//!   (cause (attracts sun planet) (revolve-around planet sun)))
//! ```

use crate::kb::{CaseId, CaseKind, ExprId, Kb, Term};
use crate::sexpr::{parse_all, SExp};
use crate::vocab::{PredInfo, PredKind};
use std::fmt;

#[derive(Debug, Clone)]
pub struct LoadError(pub String);

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for LoadError {}

fn err<T>(msg: impl Into<String>) -> Result<T, LoadError> {
    Err(LoadError(msg.into()))
}

impl Kb {
    /// Load all forms from source text. Returns the ids of cases defined.
    pub fn load_str(&mut self, src: &str) -> Result<Vec<CaseId>, LoadError> {
        let forms = parse_all(src).map_err(|e| LoadError(e.to_string()))?;
        let mut cases = Vec::new();
        for form in &forms {
            let items = form.as_list().ok_or_else(|| LoadError(format!("top-level atom: {form}")))?;
            match items.first().and_then(SExp::as_atom) {
                Some("defpredicate") => self.load_defpredicate(items)?,
                Some("defcase") | Some("defschema") | Some("defquery") => cases.push(self.load_defcase(items)?),
                _ => return err(format!("unknown top-level form: {form}")),
            }
        }
        Ok(cases)
    }

    fn load_defpredicate(&mut self, items: &[SExp]) -> Result<(), LoadError> {
        let name = items.get(1).and_then(SExp::as_atom).ok_or_else(|| LoadError("defpredicate needs a name".into()))?;
        let mut arity = None;
        let mut kind = PredKind::Relation;
        let mut commutative = false;
        let mut canonical = true;
        let mut parents = Vec::new();
        let mut i = 2;
        while i < items.len() {
            let key = items[i].as_atom().ok_or_else(|| LoadError(format!("bad option in defpredicate {name}")))?;
            let val = items.get(i + 1).ok_or_else(|| LoadError(format!("missing value for {key} in {name}")))?;
            match key {
                ":arity" => {
                    let a = val.as_atom().unwrap_or("");
                    arity = if a == "*" {
                        None
                    } else {
                        Some(a.parse::<u8>().map_err(|_| LoadError(format!("bad arity {a} for {name}")))?)
                    };
                }
                ":kind" => {
                    let k = val.as_atom().unwrap_or("");
                    kind = PredKind::parse(k).ok_or_else(|| LoadError(format!("bad kind {k} for {name}")))?;
                }
                ":commutative" | ":symmetric" => commutative = val.as_atom() == Some("t"),
                ":canonical" => canonical = val.as_atom() != Some("nil"),
                ":parents" => {
                    let list = val.as_list().ok_or_else(|| LoadError(format!(":parents must be a list in {name}")))?;
                    for p in list {
                        parents.push(p.as_atom().ok_or_else(|| LoadError("parent must be an atom".into()))?.to_string());
                    }
                }
                ":higher-order" | ":doc" => {}
                other => return err(format!("unknown option {other} in defpredicate {name}")),
            }
            i += 2;
        }
        let name_sym = self.sym(name);
        let parent_syms = parents.iter().map(|p| self.sym(p)).collect();
        self.vocab.declare(PredInfo { name: name_sym, arity, kind, commutative, parents: parent_syms, declared: true, canonical });
        Ok(())
    }

    fn load_defcase(&mut self, items: &[SExp]) -> Result<CaseId, LoadError> {
        let kind = match items[0].as_atom() {
            Some("defschema") => CaseKind::Schema,
            Some("defquery") => CaseKind::Query,
            _ => CaseKind::Episode,
        };
        let name = items.get(1).and_then(SExp::as_atom).ok_or_else(|| LoadError("defcase needs a name".into()))?;
        if self.case_by_name(name).is_some() {
            return err(format!("duplicate case {name}"));
        }
        let mut facts = Vec::new();
        for f in &items[2..] {
            match self.term_from_sexp(f)? {
                Term::Expr(e) => facts.push(e),
                Term::Ent(_) => return err(format!("fact must be a list in case {name}: {f}")),
            }
        }
        Ok(self.add_case(name, kind, facts))
    }

    /// Convert an s-expression into a term, interning expressions.
    pub fn term_from_sexp(&mut self, s: &SExp) -> Result<Term, LoadError> {
        match s {
            SExp::Atom(a) => Ok(Term::Ent(self.sym(a))),
            SExp::List(items) => {
                let head = items.first().and_then(SExp::as_atom).ok_or_else(|| LoadError(format!("expression needs an atom functor: {s}")))?;
                let functor = self.sym(head);
                let mut args = Vec::with_capacity(items.len() - 1);
                for a in &items[1..] {
                    args.push(self.term_from_sexp(a)?);
                }
                if let Some(info) = self.vocab.get(functor) {
                    if let Some(ar) = info.arity {
                        if ar as usize != args.len() {
                            return err(format!("arity mismatch for {head}: expected {ar}, got {} in {s}", args.len()));
                        }
                    }
                }
                Ok(Term::Expr(self.intern_expr(functor, args)))
            }
        }
    }

    /// Parse and intern a single expression, e.g. `"(attracts sun planet)"`.
    pub fn parse_expr(&mut self, src: &str) -> Result<ExprId, LoadError> {
        let s = crate::sexpr::parse_one(src).map_err(|e| LoadError(e.to_string()))?;
        match self.term_from_sexp(&s)? {
            Term::Expr(e) => Ok(e),
            Term::Ent(_) => err("expected an expression"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const SOLAR: &str = r#"
(defpredicate attracts :arity 2 :kind relation :parents (force-relation))
(defpredicate revolve-around :arity 2 :kind relation)
(defpredicate greater :arity 2 :kind relation)
(defpredicate mass :arity 1 :kind function)
(defpredicate cause :arity 2 :kind relation)
(defpredicate and :arity * :kind logical :commutative t)
(defpredicate yellow :arity 1 :kind attribute)
(defcase solar-system
  (yellow sun)
  (attracts sun planet)
  (greater (mass sun) (mass planet))
  (revolve-around planet sun)
  (cause (and (attracts sun planet) (greater (mass sun) (mass planet)))
         (revolve-around planet sun)))
(defcase rutherford-atom
  (attracts nucleus electron)
  (greater (mass nucleus) (mass electron))
  (revolve-around electron nucleus))
"#;

    #[test]
    fn loads_classic_example() {
        let mut kb = Kb::new();
        let ids = kb.load_str(SOLAR).unwrap();
        assert_eq!(ids.len(), 2);
        let solar = kb.case(ids[0]);
        assert_eq!(solar.facts.len(), 5);
        // Shared sub-expressions are interned once.
        let exprs = kb.case_exprs(ids[0]);
        let n_unique = exprs.len();
        assert_eq!(n_unique, 8, "{:?}", exprs.iter().map(|e| kb.render_expr(*e)).collect::<Vec<_>>());
        // Order: cause(and(...), ...) is third order.
        let cause = *solar.facts.last().unwrap();
        assert_eq!(kb.order(cause), 4);
        let ents: Vec<_> = kb.case_entities(ids[0]).iter().map(|s| kb.name(*s).to_string()).collect();
        assert_eq!(ents, vec!["sun", "planet"]);
    }

    #[test]
    fn commutative_args_canonical() {
        let mut kb = Kb::new();
        kb.load_str(SOLAR).unwrap();
        let a = kb.parse_expr("(and (attracts a b) (attracts c d))").unwrap();
        let b = kb.parse_expr("(and (attracts c d) (attracts a b))").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn arity_checked() {
        let mut kb = Kb::new();
        kb.load_str(SOLAR).unwrap();
        assert!(kb.parse_expr("(attracts a)").is_err());
    }

    #[test]
    fn render_roundtrip() {
        let mut kb = Kb::new();
        let ids = kb.load_str(SOLAR).unwrap();
        let text = format!("{}\n{}", kb.render_vocab(), kb.render_case(ids[1]));
        let mut kb2 = Kb::new();
        let ids2 = kb2.load_str(&text).unwrap();
        assert_eq!(kb2.render_case(ids2[0]), kb.render_case(ids[1]));
    }
}
