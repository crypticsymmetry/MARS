//! `mars-gen`: synthetic relational-analogy data with known ground truth.
//!
//! See docs/EXPERIMENTS.md §1. Each *group* holds a base case and its
//! variants (LS, TA, MA, FOR, RND). Templates are random draws from seven
//! structural families; DEV/TEST family splits support held-out evaluation.

pub mod dataset;
pub mod template;
pub mod vocab;

pub use dataset::{generate, Dataset, GenConfig, Group, Item, VariantClass};
pub use template::{Family, Template};
pub use vocab::Naming;
