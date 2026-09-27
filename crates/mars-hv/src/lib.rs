//! `mars-hv`: binary hypervector primitives for MARS.
//!
//! * [`HyperVector`]: packed bits; bind (XOR), permute (rotation), Hamming.
//! * [`Accumulator`]: exact, reversible integer bundling (incremental path).
//! * [`BitSliceMajority`] / [`sketch_seeds`]: fast weighted-majority bundling
//!   used to build fingerprints (a SimHash of a weighted feature multiset).
//! * [`rng`]: deterministic seeding and hashing.

pub mod accum;
pub mod bitslice;
pub mod hv;
pub mod rng;

pub use accum::Accumulator;
pub use bitslice::{sketch_seeds, BitSliceMajority};
pub use hv::{hamming_words, HyperVector, WORD_BITS};
pub use rng::Rng;
