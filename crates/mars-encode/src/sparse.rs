//! Exact sparse similarity and corpus feature statistics (IDF epochs).

use crate::features::{Features, SparseVec, N_CHANNELS};
use rustc_hash::FxHashMap;

/// Cosine similarity of two sorted sparse vectors. 0 if either is empty.
pub fn cosine(a: &[(u64, f32)], b: &[(u64, f32)]) -> f64 {
    let (mut i, mut j) = (0, 0);
    let mut dot = 0.0f64;
    while i < a.len() && j < b.len() {
        match a[i].0.cmp(&b[j].0) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                dot += a[i].1 as f64 * b[j].1 as f64;
                i += 1;
                j += 1;
            }
        }
    }
    let na = norm(a);
    let nb = norm(b);
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

pub fn norm(a: &[(u64, f32)]) -> f64 {
    a.iter().map(|x| (x.1 as f64) * (x.1 as f64)).sum::<f64>().sqrt()
}

/// Per-channel exact cosines.
pub fn channel_cosines(a: &Features, b: &Features) -> [f64; N_CHANNELS] {
    let mut out = [0.0; N_CHANNELS];
    for (c, o) in out.iter_mut().enumerate() {
        *o = cosine(&a.channels[c], &b.channels[c]);
    }
    out
}

/// Document frequencies of features over a corpus, per channel. A frozen
/// snapshot of this is a *vocabulary epoch* (DESIGN §6.7).
#[derive(Clone, Debug, Default)]
pub struct FeatureStats {
    pub n_docs: u32,
    df: [FxHashMap<u64, u32>; N_CHANNELS],
}

impl FeatureStats {
    pub fn fit<'a>(corpus: impl IntoIterator<Item = &'a Features>) -> Self {
        let mut s = FeatureStats::default();
        for f in corpus {
            s.observe(f);
        }
        s
    }

    pub fn observe(&mut self, f: &Features) {
        self.n_docs += 1;
        for c in 0..N_CHANNELS {
            for &(h, _) in &f.channels[c] {
                *self.df[c].entry(h).or_insert(0) += 1;
            }
        }
    }

    /// Smoothed IDF: `ln((N + 1) / (df + 1)) + 1`, always ≥ 1 for seen features
    /// relative to unseen ones' maximum. Features in every document get ~1.
    pub fn idf(&self, channel: usize, h: u64) -> f32 {
        let df = self.df[channel].get(&h).copied().unwrap_or(0);
        (((self.n_docs as f64 + 1.0) / (df as f64 + 1.0)).ln() + 1.0) as f32
    }

    /// Multiply weights by IDF on the selected channels.
    pub fn apply(&self, f: &mut Features, channels: &[bool; N_CHANNELS]) {
        for c in 0..N_CHANNELS {
            if channels[c] {
                for (h, w) in f.channels[c].iter_mut() {
                    *w *= self.idf(c, *h);
                }
            }
        }
    }

    pub fn apply_sparse(&self, channel: usize, v: &mut SparseVec) {
        for (h, w) in v.iter_mut() {
            *w *= self.idf(channel, *h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_basic() {
        let a = vec![(1, 1.0), (2, 1.0)];
        let b = vec![(2, 1.0), (3, 1.0)];
        assert!((cosine(&a, &b) - 0.5).abs() < 1e-9);
        assert!((cosine(&a, &a) - 1.0).abs() < 1e-9);
        assert_eq!(cosine(&a, &[]), 0.0);
    }
}
