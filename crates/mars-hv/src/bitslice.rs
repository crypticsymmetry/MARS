//! Bit-sliced weighted-majority bundling.
//!
//! Adding a hypervector to an [`crate::accum::Accumulator`] touches `D`
//! integer lanes. Here counts are kept *vertically*: plane `k` holds bit `k` of
//! the per-dimension count `S_j = Σ_f w_f · v_f[j]` for all 64 dimensions of a
//! word at once. Adding a unit-weight vector is a ripple-carry add across
//! planes (about two planes on average), i.e. O(D/64) word operations
//! instead of O(D).
//!
//! The weighted majority bit is `2·S_j > W` (W = total weight), which is
//! equivalent to the bipolar sum `Σ w_f (2v_f[j] - 1) > 0`. Comparison
//! against the constant `W/2` is done with a bit-sliced comparator.

use crate::hv::{HyperVector, WORD_BITS};
use crate::rng::Rng;

#[derive(Clone, Debug)]
pub struct BitSliceMajority {
    words: usize,
    /// planes[k * words + i] = bit k of the counts for word i.
    planes: Vec<u64>,
    n_planes: usize,
    total_weight: u64,
    carry: Vec<u64>,
}

impl BitSliceMajority {
    pub fn new(dim: usize) -> Self {
        assert!(dim.is_multiple_of(WORD_BITS) && dim > 0);
        let words = dim / WORD_BITS;
        BitSliceMajority { words, planes: Vec::new(), n_planes: 0, total_weight: 0, carry: vec![0; words] }
    }

    pub fn dim(&self) -> usize {
        self.words * WORD_BITS
    }

    pub fn total_weight(&self) -> u64 {
        self.total_weight
    }

    pub fn clear(&mut self) {
        self.planes.iter_mut().for_each(|w| *w = 0);
        self.total_weight = 0;
    }

    fn ensure_planes(&mut self, n: usize) {
        if n > self.n_planes {
            self.planes.resize(n * self.words, 0);
            self.n_planes = n;
        }
    }

    /// Add `weight · v` (per-dimension, v ∈ {0,1}).
    pub fn add_words(&mut self, v: &[u64], weight: u32) {
        debug_assert_eq!(v.len(), self.words);
        if weight == 0 {
            return;
        }
        self.total_weight += weight as u64;
        let needed = (64 - self.total_weight.leading_zeros()) as usize;
        self.ensure_planes(needed);
        let mut w = weight;
        let mut start = 0usize;
        while w != 0 {
            let tz = w.trailing_zeros() as usize;
            start += tz;
            w >>= tz;
            self.ripple_add(v, start);
            w >>= 1;
            start += 1;
        }
    }

    pub fn add(&mut self, v: &HyperVector, weight: u32) {
        self.add_words(v.words(), weight);
    }

    /// Add `v` shifted to plane `k` (i.e. v · 2^k) with carry propagation.
    fn ripple_add(&mut self, v: &[u64], k: usize) {
        let words = self.words;
        self.carry.copy_from_slice(v);
        let mut plane = k;
        loop {
            if plane >= self.n_planes {
                self.ensure_planes(plane + 1);
            }
            let p = &mut self.planes[plane * words..(plane + 1) * words];
            let mut any = 0u64;
            for (pw, cw) in p.iter_mut().zip(self.carry.iter_mut()) {
                let t = *pw & *cw;
                *pw ^= *cw;
                *cw = t;
                any |= t;
            }
            if any == 0 {
                break;
            }
            plane += 1;
        }
    }

    /// Per-dimension count (for testing/diagnostics).
    pub fn count(&self, j: usize) -> u64 {
        let (wi, b) = (j / WORD_BITS, j % WORD_BITS);
        (0..self.n_planes).map(|k| ((self.planes[k * self.words + wi] >> b) & 1) << k).sum()
    }

    /// Weighted-majority vector: bit = 2·S > W, ties (2·S == W) from `tie`.
    pub fn threshold_into(&self, tie: &[u64], out: &mut [u64]) {
        assert_eq!(tie.len(), self.words);
        assert_eq!(out.len(), self.words);
        let w = self.total_weight;
        // Compare S against c = floor(W/2).
        let c = w / 2;
        let even = w.is_multiple_of(2);
        let c_bits = (64 - c.leading_zeros()) as usize;
        let top = self.n_planes.max(c_bits);
        for i in 0..self.words {
            let mut gt = 0u64;
            let mut eq = !0u64;
            for k in (0..top).rev() {
                let x = if k < self.n_planes { self.planes[k * self.words + i] } else { 0 };
                if (c >> k) & 1 == 1 {
                    eq &= x;
                } else {
                    gt |= eq & x;
                    eq &= !x;
                }
            }
            out[i] = if even { gt | (eq & tie[i]) } else { gt };
        }
    }

    pub fn threshold(&self, tie: &HyperVector) -> HyperVector {
        let mut out = HyperVector::zeros(self.dim());
        self.threshold_into(tie.words(), out.words_mut());
        out
    }
}

/// Weighted majority of seed-derived random vectors: the core sketch
/// operation. `features` yields `(seed, weight)`; each seed expands to a
/// uniformly random `dim`-bit vector. Equivalent to SimHash of the weighted
/// feature vector with Rademacher projections.
pub fn sketch_seeds<I>(dim: usize, tie: &HyperVector, features: I) -> HyperVector
where
    I: IntoIterator<Item = (u64, u32)>,
{
    let mut acc = BitSliceMajority::new(dim);
    let mut buf = vec![0u64; dim / WORD_BITS];
    for (seed, w) in features {
        if w == 0 {
            continue;
        }
        Rng::new(seed).fill(&mut buf);
        acc.add_words(&buf, w);
    }
    acc.threshold(tie)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accum::Accumulator;

    #[test]
    fn counts_match_naive() {
        let dim = 256;
        let mut bs = BitSliceMajority::new(dim);
        let mut naive = vec![0u64; dim];
        let mut rng = Rng::new(5);
        for f in 0..40 {
            let v = HyperVector::random(dim, 1000 + f);
            let w = rng.range_inclusive(1, 37) as u32;
            bs.add(&v, w);
            for (j, n) in naive.iter_mut().enumerate() {
                if v.get(j) {
                    *n += w as u64;
                }
            }
        }
        for (j, n) in naive.iter().enumerate() {
            assert_eq!(bs.count(j), *n, "dim {j}");
        }
    }

    #[test]
    fn threshold_matches_integer_accumulator_including_ties() {
        let dim = 1024;
        let tie = HyperVector::random(dim, 77);
        let mut rng = Rng::new(9);
        for trial in 0..50 {
            let n = rng.range_inclusive(1, 12);
            let mut bs = BitSliceMajority::new(dim);
            let mut acc = Accumulator::new(dim);
            for f in 0..n {
                let v = HyperVector::random(dim, (trial * 100 + f) as u64);
                // Small weights, many even totals, so ties are frequent.
                let w = rng.range_inclusive(1, 3) as u32;
                bs.add(&v, w);
                acc.add(&v, w as i32);
            }
            assert_eq!(bs.threshold(&tie), acc.threshold(&tie), "trial {trial}");
        }
    }

    #[test]
    fn sketch_similarity_follows_simhash_law() {
        // Two feature sets with cosine 0.5 -> expected normalized Hamming 1/3.
        let dim = 8192;
        let tie = HyperVector::random(dim, 1);
        let a: Vec<(u64, u32)> = (0..200).map(|i| (i, 1)).collect();
        let b: Vec<(u64, u32)> = (0..100).chain(1000..1100).map(|i| (i, 1)).collect();
        let fa = sketch_seeds(dim, &tie, a);
        let fb = sketch_seeds(dim, &tie, b);
        let delta = fa.hamming(&fb) as f64 / dim as f64;
        assert!((delta - 1.0 / 3.0).abs() < 0.02, "delta={delta}");
    }
}
