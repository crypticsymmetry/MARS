//! `mars-index`: the MAC stage (DESIGN §7).
//!
//! * [`ModeK`]: exact exhaustive weighted-segment Hamming top-k, batched and
//!   cache-tiled. The baseline every other retrieval mode must beat.

pub mod modek;
pub mod topk;

pub use modek::{ModeK, Scorer};
pub use topk::{Hit, TopK};
