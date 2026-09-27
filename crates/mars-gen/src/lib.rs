//! `mars-gen`: synthetic relational-analogy data with known ground truth.
//!
//! See docs/EXPERIMENTS.md §1. Each *group* holds a base case and its
//! variants (LS, TA, MA, FOR, RND). Templates are random draws from seven
//! structural families; DEV/TEST family splits support held-out evaluation.

pub mod dataset;
pub mod template;
pub mod vocab;

pub use dataset::{concept_instances, generate, template_instances, ConceptSet, Dataset, GenConfig, Group, InstanceSet, Item, VariantClass};
pub use template::{Family, PerturbOp, Template};
pub use vocab::Naming;
