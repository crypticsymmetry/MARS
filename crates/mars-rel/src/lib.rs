//! `mars-rel`: the explicit relational representation.
//!
//! * [`Kb`]: hash-consed expression DAG, cases, rendering.
//! * [`Vocabulary`] / [`PredInfo`]: predicate declarations and taxonomy.
//! * [`load`]: the `.mars` s-expression format.
//! * [`views`]: alternative granularities of a case (flat blocks).

pub mod kb;
pub mod load;
pub mod sexpr;
pub mod views;
pub mod vocab;

pub use kb::{Args, Case, CaseId, CaseKind, Expr, ExprId, Kb, Term};
pub use load::LoadError;
pub use vocab::{Interner, PredInfo, PredKind, Sym, Vocabulary};
