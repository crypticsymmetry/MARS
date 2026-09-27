//! Integer bundling accumulator.
//!
//! `sums[j] = Σ_f w_f · bipolar(v_f)[j]`, with bipolar(1) = +1, bipolar(0) = -1.
//! Thresholding gives the weighted-majority bundle. This is the simple,
//! exactly reversible path used for incremental updates and as the
//! reference implementation for [`crate::bitslice::BitSliceMajority`].

use crate::hv::{HyperVector, WORD_BITS};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Accumulator {
    sums: Vec<i32>,
}

impl Accumulator {
    pub fn new(dim: usize) -> Self {
        assert!(dim.is_multiple_of(WORD_BITS) && dim > 0);
        Accumulator { sums: vec![0; dim] }
    }

    pub fn dim(&self) -> usize {
        self.sums.len()
    }

    pub fn sums(&self) -> &[i32] {
        &self.sums
    }

    /// Add `w · bipolar(v)`. Negative weights subtract.
    pub fn add_words(&mut self, v: &[u64], w: i32) {
        debug_assert_eq!(v.len() * WORD_BITS, self.sums.len());
        for (wi, &word) in v.iter().enumerate() {
            let base = wi * WORD_BITS;
            let s = &mut self.sums[base..base + WORD_BITS];
            for (b, slot) in s.iter_mut().enumerate() {
                // (2*bit - 1) * w, branch-free.
                let bit = ((word >> b) & 1) as i32;
                *slot += (2 * bit - 1) * w;
            }
        }
    }

    pub fn add(&mut self, v: &HyperVector, w: i32) {
        self.add_words(v.words(), w);
    }

    pub fn sub(&mut self, v: &HyperVector, w: i32) {
        self.add_words(v.words(), -w);
    }

    /// Majority threshold: bit = sum > 0, ties broken by `tie`.
    pub fn threshold(&self, tie: &HyperVector) -> HyperVector {
        assert_eq!(tie.dim(), self.sums.len());
        let mut out = HyperVector::zeros(self.sums.len());
        for (wi, w) in out.words_mut().iter_mut().enumerate() {
            let t = tie.words()[wi];
            let mut word = 0u64;
            for b in 0..WORD_BITS {
                let s = self.sums[wi * WORD_BITS + b];
                let bit = s > 0 || (s == 0 && (t >> b) & 1 == 1);
                word |= (bit as u64) << b;
            }
            *w = word;
        }
        out
    }

    /// Indices whose thresholded bit would differ between `self` and `other`.
    pub fn flipped_bits(&self, other: &Accumulator, tie: &HyperVector) -> Vec<usize> {
        let a = self.threshold(tie);
        let b = other.threshold(tie);
        (0..self.sums.len()).filter(|&i| a.get(i) != b.get(i)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub_roundtrip() {
        let mut acc = Accumulator::new(256);
        let vs: Vec<_> = (0..5).map(|i| HyperVector::random(256, i)).collect();
        for (i, v) in vs.iter().enumerate() {
            acc.add(v, i as i32 + 1);
        }
        let snapshot = acc.clone();
        let extra = HyperVector::random(256, 99);
        acc.add(&extra, 3);
        acc.sub(&extra, 3);
        assert_eq!(acc, snapshot);
    }

    #[test]
    fn bundle_is_similar_to_members() {
        let dim = 8192;
        let tie = HyperVector::random(dim, 1234);
        let vs: Vec<_> = (0..7).map(|i| HyperVector::random(dim, 100 + i)).collect();
        let mut acc = Accumulator::new(dim);
        for v in &vs {
            acc.add(v, 1);
        }
        let b = acc.threshold(&tie);
        for v in &vs {
            // Expected similarity for a majority of 7 is ~0.31.
            assert!(b.similarity(v) > 0.2, "{}", b.similarity(v));
        }
        let unrelated = HyperVector::random(dim, 7777);
        assert!(b.similarity(&unrelated).abs() < 0.06);
    }
}
