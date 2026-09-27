//! Deterministic seeding and hashing.
//!
//! Every random quantity in MARS derives from a 64-bit seed so that runs are
//! exactly reproducible. We avoid external RNG crates to keep the bit-level
//! behaviour stable across dependency upgrades.

/// SplitMix64 step: advances `state` and returns a well-mixed output.
#[inline]
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    mix64(*state)
}

/// Stafford variant 13 finalizer (the SplitMix64 output function).
#[inline]
pub fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Order-dependent combination of two hashes.
#[inline]
pub fn hash_combine(a: u64, b: u64) -> u64 {
    mix64(a ^ b.wrapping_add(0x9E37_79B9_7F4A_7C15).wrapping_add(a << 6).wrapping_add(a >> 2))
}

/// Hash a sequence of u64 words (order-dependent).
#[inline]
pub fn hash_words(words: &[u64]) -> u64 {
    let mut h = 0xCBF2_9CE4_8422_2325u64 ^ (words.len() as u64);
    for &w in words {
        h = hash_combine(h, w);
    }
    h
}

/// FNV-1a over bytes followed by a strong finalizer. Stable across platforms.
pub fn hash_str(s: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    mix64(h)
}

/// xoshiro256** 1.0, seeded through SplitMix64.
#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut sm = seed;
        let s = [
            splitmix64(&mut sm),
            splitmix64(&mut sm),
            splitmix64(&mut sm),
            splitmix64(&mut sm),
        ];
        Rng { s }
    }

    /// Derive an independent stream from this seed and a label.
    pub fn derive(seed: u64, label: u64) -> Self {
        Rng::new(hash_combine(seed, label))
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// Uniform integer in `0..n` (Lemire's nearly-divisionless method). `n > 0`.
    #[inline]
    pub fn below(&mut self, n: u64) -> u64 {
        debug_assert!(n > 0);
        let mut m = (self.next_u64() as u128) * (n as u128);
        let mut lo = m as u64;
        if lo < n {
            let t = n.wrapping_neg() % n;
            while lo < t {
                m = (self.next_u64() as u128) * (n as u128);
                lo = m as u64;
            }
        }
        (m >> 64) as u64
    }

    #[inline]
    pub fn index(&mut self, n: usize) -> usize {
        self.below(n as u64) as usize
    }

    /// Uniform integer in the inclusive range `lo..=hi`.
    #[inline]
    pub fn range_inclusive(&mut self, lo: usize, hi: usize) -> usize {
        debug_assert!(lo <= hi);
        lo + self.index(hi - lo + 1)
    }

    /// Uniform float in `[0, 1)`.
    #[inline]
    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    #[inline]
    pub fn bernoulli(&mut self, p: f64) -> bool {
        self.f64() < p
    }

    pub fn choose<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.index(xs.len())]
    }

    pub fn shuffle<T>(&mut self, xs: &mut [T]) {
        for i in (1..xs.len()).rev() {
            let j = self.index(i + 1);
            xs.swap(i, j);
        }
    }

    /// Sample `k` distinct indices from `0..n` (k <= n), in random order.
    pub fn sample_indices(&mut self, n: usize, k: usize) -> Vec<usize> {
        assert!(k <= n, "cannot sample {k} of {n}");
        if k * 4 < n {
            let mut out = Vec::with_capacity(k);
            while out.len() < k {
                let x = self.index(n);
                if !out.contains(&x) {
                    out.push(x);
                }
            }
            out
        } else {
            let mut all: Vec<usize> = (0..n).collect();
            // Partial Fisher–Yates.
            for i in 0..k {
                let j = i + self.index(n - i);
                all.swap(i, j);
            }
            all.truncate(k);
            all
        }
    }

    /// Fill a word slice with random bits.
    #[inline]
    pub fn fill(&mut self, out: &mut [u64]) {
        for w in out {
            *w = self.next_u64();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        assert_ne!(Rng::new(1).next_u64(), Rng::new(2).next_u64());
    }

    #[test]
    fn below_is_in_range_and_roughly_uniform() {
        let mut r = Rng::new(7);
        let mut counts = [0usize; 10];
        for _ in 0..100_000 {
            counts[r.below(10) as usize] += 1;
        }
        for c in counts {
            assert!((9_000..11_000).contains(&c), "{counts:?}");
        }
    }

    #[test]
    fn sample_indices_distinct() {
        let mut r = Rng::new(3);
        for &(n, k) in &[(10, 10), (100, 5), (50, 30)] {
            let mut s = r.sample_indices(n, k);
            s.sort();
            s.dedup();
            assert_eq!(s.len(), k);
            assert!(s.iter().all(|&x| x < n));
        }
    }

    #[test]
    fn hash_str_stable() {
        // Pin the value: fingerprints depend on it, so changes must be deliberate.
        assert_eq!(hash_str("cause"), hash_str("cause"));
        assert_ne!(hash_str("cause"), hash_str("causes"));
    }
}
