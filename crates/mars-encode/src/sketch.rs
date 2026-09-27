//! Segmented binary fingerprints: one SimHash segment per channel.

use crate::features::{Features, N_CHANNELS};
use mars_hv::rng::hash_combine;
use mars_hv::{hamming_words, sketch_seeds, HyperVector, WORD_BITS};
use serde::{Deserialize, Serialize};

/// Bit allocation per channel. Each must be a multiple of 64 (0 disables).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub dims: [usize; N_CHANNELS],
}

impl Default for Layout {
    fn default() -> Self {
        Layout { dims: [1024, 1024, 3072, 2048, 1024] }
    }
}

impl Layout {
    pub fn new(dims: [usize; N_CHANNELS]) -> Self {
        for d in dims {
            assert!(d % WORD_BITS == 0, "segment size {d} not a multiple of 64");
        }
        Layout { dims }
    }

    /// Scale the default split to a total dimension (proportional, 64-aligned).
    pub fn scaled(total: usize) -> Self {
        let base = Layout::default();
        let base_total: usize = base.dims.iter().sum();
        let mut dims = [0; N_CHANNELS];
        for c in 0..N_CHANNELS {
            dims[c] = ((base.dims[c] * total / base_total) / WORD_BITS).max(1) * WORD_BITS;
        }
        Layout { dims }
    }

    pub fn total_bits(&self) -> usize {
        self.dims.iter().sum()
    }

    pub fn total_words(&self) -> usize {
        self.total_bits() / WORD_BITS
    }

    /// Word range of channel `c` inside a fingerprint.
    pub fn word_range(&self, c: usize) -> std::ops::Range<usize> {
        let start: usize = self.dims[..c].iter().sum::<usize>() / WORD_BITS;
        start..start + self.dims[c] / WORD_BITS
    }
}

/// Weight quantization for the integer majority: w → round(w · Q), min 1.
pub const WEIGHT_Q: f32 = 16.0;

#[inline]
pub fn quantize(w: f32) -> u32 {
    ((w * WEIGHT_Q).round() as u32).max(1)
}

#[derive(Clone, Debug)]
pub struct Sketcher {
    pub layout: Layout,
    pub seed: u64,
    ties: [Option<HyperVector>; N_CHANNELS],
}

impl Sketcher {
    pub fn new(layout: Layout, seed: u64) -> Self {
        let ties = std::array::from_fn(|c| {
            let d = layout.dims[c];
            (d > 0).then(|| HyperVector::random(d, hash_combine(seed, 0x7E_0000 + c as u64)))
        });
        Sketcher { layout, seed, ties }
    }

    /// Fingerprint = concatenation of per-channel weighted-majority sketches.
    pub fn sketch(&self, f: &Features) -> HyperVector {
        let mut words = Vec::with_capacity(self.layout.total_words());
        for c in 0..N_CHANNELS {
            let d = self.layout.dims[c];
            if d == 0 {
                continue;
            }
            let tie = self.ties[c].as_ref().unwrap();
            let chan_seed = hash_combine(self.seed, c as u64);
            let seg = sketch_seeds(d, tie, f.channels[c].iter().map(|&(h, w)| (hash_combine(chan_seed, h), quantize(w))));
            words.extend_from_slice(seg.words());
        }
        HyperVector::from_words(words)
    }

    /// Per-channel similarity `1 - 2δ` (0 for disabled channels).
    pub fn channel_sims(&self, a: &HyperVector, b: &HyperVector) -> [f64; N_CHANNELS] {
        channel_sims(&self.layout, a.words(), b.words())
    }
}

pub fn channel_sims(layout: &Layout, a: &[u64], b: &[u64]) -> [f64; N_CHANNELS] {
    let mut out = [0.0; N_CHANNELS];
    for c in 0..N_CHANNELS {
        let d = layout.dims[c];
        if d == 0 {
            continue;
        }
        let r = layout.word_range(c);
        let h = hamming_words(&a[r.clone()], &b[r]);
        out[c] = 1.0 - 2.0 * h as f64 / d as f64;
    }
    out
}

/// Per-channel mixing weights λ.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub weights: [f64; N_CHANNELS],
}

impl Profile {
    pub fn new(name: &str, weights: [f64; N_CHANNELS]) -> Self {
        Profile { name: name.to_string(), weights }
    }
    /// Far/cross-domain analogues: surface ignored.
    pub fn analogy() -> Self {
        Profile::new("analogy", [0.0, 0.1, 0.4, 0.4, 0.1])
    }
    pub fn literal() -> Self {
        Profile::new("literal", [0.4, 0.15, 0.2, 0.2, 0.05])
    }
    pub fn surface_only() -> Self {
        Profile::new("surface-only", [1.0, 0.0, 0.0, 0.0, 0.0])
    }
    pub fn structure_only() -> Self {
        Profile::new("structure-only", [0.0, 0.0, 0.5, 0.4, 0.1])
    }
    pub fn score(&self, sims: &[f64; N_CHANNELS]) -> f64 {
        self.weights.iter().zip(sims).map(|(w, s)| w * s).sum()
    }
}
