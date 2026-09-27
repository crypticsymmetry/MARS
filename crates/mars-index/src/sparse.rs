//! Exact sparse cosine retrieval via inverted indexes (baselines B2/B4/B5).

use crate::topk::{Hit, TopK};
use mars_encode::SparseVec;
use rayon::prelude::*;
use rustc_hash::FxHashMap;

/// Inverted index over L2-normalized sparse vectors.
pub struct SparseIndex {
    postings: FxHashMap<u64, Vec<(u32, f32)>>,
    n: usize,
}

fn normalized(v: &SparseVec) -> Vec<(u64, f32)> {
    let norm = v.iter().map(|x| (x.1 as f64).powi(2)).sum::<f64>().sqrt();
    if norm == 0.0 {
        return Vec::new();
    }
    v.iter().map(|&(h, w)| (h, (w as f64 / norm) as f32)).collect()
}

impl SparseIndex {
    pub fn build(docs: &[SparseVec]) -> Self {
        let mut postings: FxHashMap<u64, Vec<(u32, f32)>> = FxHashMap::default();
        for (d, v) in docs.iter().enumerate() {
            for (h, w) in normalized(v) {
                postings.entry(h).or_default().push((d as u32, w));
            }
        }
        SparseIndex { postings, n: docs.len() }
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// acc[d] += weight · cos(q, doc_d)
    pub fn accumulate(&self, q: &SparseVec, weight: f32, acc: &mut [f32]) {
        for (h, qw) in normalized(q) {
            if let Some(list) = self.postings.get(&h) {
                let s = qw * weight;
                for &(d, w) in list {
                    acc[d as usize] += s * w;
                }
            }
        }
    }
}

/// Weighted sum of per-channel cosines, `Σ_c λ_c cos_c(q, d)`.
pub struct MultiSparse {
    pub channels: Vec<(f32, SparseIndex)>,
}

impl MultiSparse {
    pub fn n(&self) -> usize {
        self.channels.first().map(|c| c.1.len()).unwrap_or(0)
    }

    /// Batched exact search; `queries[i][c]` is query i's vector for channel c.
    pub fn search_batch(&self, queries: &[Vec<&SparseVec>], k: usize) -> Vec<Vec<Hit>> {
        let n = self.n();
        queries
            .par_iter()
            .map_init(
                || vec![0f32; n],
                |acc, q| {
                    acc.iter_mut().for_each(|x| *x = 0.0);
                    for ((lambda, idx), qv) in self.channels.iter().zip(q) {
                        if *lambda != 0.0 {
                            idx.accumulate(qv, *lambda, acc);
                        }
                    }
                    let mut top = TopK::new(k);
                    for (d, &s) in acc.iter().enumerate() {
                        top.push(d as u32, s);
                    }
                    top.into_sorted()
                },
            )
            .collect()
    }
}
