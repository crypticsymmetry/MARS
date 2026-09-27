//! `mars-index`: the MAC stage (DESIGN §7).
//!
//! * [`ModeK`]: exact exhaustive weighted-segment Hamming top-k, batched and
//!   cache-tiled. The baseline every other retrieval mode must beat.
//! * [`SparseIndex`] / [`MultiSparse`]: exact sparse cosine (inverted index)
//!   for the MAC-content, exact-feature and lexical baselines.

pub mod modek;
pub mod sparse;
pub mod topk;

pub use modek::{ModeK, Scorer};
pub use sparse::{MultiSparse, SparseIndex};
pub use topk::{Hit, TopK};
