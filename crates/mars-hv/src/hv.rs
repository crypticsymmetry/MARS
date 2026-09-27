//! Packed binary hypervectors (Binary Spatter Codes).
//!
//! Algebra: bind = XOR, permute = cyclic bit rotation, similarity via Hamming
//! distance. Bundling (majority) lives in [`crate::accum`] and
//! [`crate::bitslice`].

use crate::rng::Rng;

pub const WORD_BITS: usize = 64;

/// A binary hypervector of `64 * words.len()` dimensions.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct HyperVector {
    words: Vec<u64>,
}

impl HyperVector {
    /// All-zero vector. `dim` must be a multiple of 64.
    pub fn zeros(dim: usize) -> Self {
        assert!(dim.is_multiple_of(WORD_BITS) && dim > 0, "dim must be a positive multiple of 64, got {dim}");
        HyperVector { words: vec![0; dim / WORD_BITS] }
    }

    /// Uniformly random vector derived from `seed`.
    pub fn random(dim: usize, seed: u64) -> Self {
        let mut v = Self::zeros(dim);
        Rng::new(seed).fill(&mut v.words);
        v
    }

    pub fn from_words(words: Vec<u64>) -> Self {
        assert!(!words.is_empty());
        HyperVector { words }
    }

    #[inline]
    pub fn dim(&self) -> usize {
        self.words.len() * WORD_BITS
    }

    #[inline]
    pub fn words(&self) -> &[u64] {
        &self.words
    }

    #[inline]
    pub fn words_mut(&mut self) -> &mut [u64] {
        &mut self.words
    }

    pub fn into_words(self) -> Vec<u64> {
        self.words
    }

    #[inline]
    pub fn get(&self, i: usize) -> bool {
        (self.words[i / WORD_BITS] >> (i % WORD_BITS)) & 1 == 1
    }

    #[inline]
    pub fn set(&mut self, i: usize, b: bool) {
        let m = 1u64 << (i % WORD_BITS);
        if b {
            self.words[i / WORD_BITS] |= m;
        } else {
            self.words[i / WORD_BITS] &= !m;
        }
    }

    pub fn count_ones(&self) -> u32 {
        self.words.iter().map(|w| w.count_ones()).sum()
    }

    #[inline]
    pub fn hamming(&self, other: &Self) -> u32 {
        debug_assert_eq!(self.words.len(), other.words.len());
        hamming_words(&self.words, &other.words)
    }

    /// Normalized similarity in [-1, 1]: `1 - 2 d_H / D`.
    #[inline]
    pub fn similarity(&self, other: &Self) -> f64 {
        1.0 - 2.0 * self.hamming(other) as f64 / self.dim() as f64
    }

    /// Bind (XOR). Self-inverse and distance-preserving.
    pub fn bind(&self, other: &Self) -> Self {
        let mut out = self.clone();
        out.bind_assign(other);
        out
    }

    #[inline]
    pub fn bind_assign(&mut self, other: &Self) {
        debug_assert_eq!(self.words.len(), other.words.len());
        for (a, b) in self.words.iter_mut().zip(&other.words) {
            *a ^= *b;
        }
    }

    /// Cyclic rotation by `k` bit positions (toward higher indices).
    /// `permute(k)` composed with `permute(dim - k)` is the identity.
    pub fn permute(&self, k: usize) -> Self {
        let n = self.words.len();
        let dim = self.dim();
        let k = k % dim;
        if k == 0 {
            return self.clone();
        }
        let ws = k / WORD_BITS;
        let bs = k % WORD_BITS;
        let mut out = vec![0u64; n];
        for i in 0..n {
            // Destination word i receives bits from source words i-ws and i-ws-1.
            let src = (i + n - ws) % n;
            let lo = self.words[src];
            if bs == 0 {
                out[i] = lo;
            } else {
                let prev = self.words[(src + n - 1) % n];
                out[i] = (lo << bs) | (prev >> (WORD_BITS - bs));
            }
        }
        HyperVector { words: out }
    }
}

/// Hamming distance between two equal-length word slices.
#[inline]
pub fn hamming_words(a: &[u64], b: &[u64]) -> u32 {
    debug_assert_eq!(a.len(), b.len());
    // Four independent accumulators let the compiler keep several vector
    // popcounts in flight; with target-cpu=native this lowers to VPOPCNTQ.
    let mut acc = [0u32; 4];
    let chunks_a = a.chunks_exact(4);
    let chunks_b = b.chunks_exact(4);
    let (ra, rb) = (chunks_a.remainder(), chunks_b.remainder());
    for (ca, cb) in chunks_a.zip(chunks_b) {
        acc[0] += (ca[0] ^ cb[0]).count_ones();
        acc[1] += (ca[1] ^ cb[1]).count_ones();
        acc[2] += (ca[2] ^ cb[2]).count_ones();
        acc[3] += (ca[3] ^ cb[3]).count_ones();
    }
    let mut tail = 0;
    for (x, y) in ra.iter().zip(rb) {
        tail += (x ^ y).count_ones();
    }
    acc[0] + acc[1] + acc[2] + acc[3] + tail
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_vectors_are_quasi_orthogonal() {
        let a = HyperVector::random(8192, 1);
        let b = HyperVector::random(8192, 2);
        let d = a.hamming(&b) as i64;
        // mean 4096, sd ~45; 6 sd bound.
        assert!((d - 4096).abs() < 6 * 46, "d = {d}");
    }

    #[test]
    fn bind_is_self_inverse_and_distance_preserving() {
        let a = HyperVector::random(1024, 1);
        let b = HyperVector::random(1024, 2);
        let c = HyperVector::random(1024, 3);
        assert_eq!(a.bind(&b).bind(&b), a);
        assert_eq!(a.bind(&c).hamming(&b.bind(&c)), a.hamming(&b));
    }

    #[test]
    fn permute_roundtrip_and_bit_semantics() {
        let a = HyperVector::random(512, 9);
        for k in [0, 1, 7, 63, 64, 65, 200, 511] {
            let p = a.permute(k);
            assert_eq!(p.permute(512 - k), a, "k={k}");
            for i in 0..512 {
                assert_eq!(p.get((i + k) % 512), a.get(i), "k={k} i={i}");
            }
        }
        // A permuted vector is unrelated to the original.
        let d = a.permute(1).hamming(&a);
        assert!(d > 150, "d={d}");
    }

    #[test]
    fn hamming_words_matches_naive() {
        for words in [1usize, 3, 4, 5, 128, 131] {
            let a = HyperVector::random(words * 64, 11);
            let b = HyperVector::random(words * 64, 12);
            let naive: u32 = (0..a.dim()).filter(|&i| a.get(i) != b.get(i)).count() as u32;
            assert_eq!(a.hamming(&b), naive);
        }
    }
}
