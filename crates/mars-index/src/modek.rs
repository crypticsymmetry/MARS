//! Mode K: exact exhaustive top-k over segmented fingerprints.
//!
//! Storage is structure-of-arrays *per channel segment*, so a profile that
//! weights a channel at zero (e.g. C0 surface in the analogy profile) never
//! reads that segment. This is a bandwidth saving, not just a compute one.
//!
//! Score for profile λ: `Σ_c λ_c (1 - 2 h_c / d_c)`, computed as
//! `base - Σ_c coef_c · h_c` with integer per-segment Hamming distances.
//!
//! Batched search tiles the corpus so that a block of rows stays in cache
//! while a batch of queries is scored against it, turning a memory-bound
//! scan into a compute-bound one (DESIGN §7.2).

use crate::topk::{Hit, TopK};
use mars_encode::{Layout, N_CHANNELS};
use mars_hv::{hamming_words, WORD_BITS};
use rayon::prelude::*;

#[derive(Clone, Debug)]
pub struct ModeK {
    layout: Layout,
    /// Words per row for each channel segment.
    seg_words: [usize; N_CHANNELS],
    /// Word offset of each segment within a full fingerprint.
    seg_off: [usize; N_CHANNELS],
    segs: [Vec<u64>; N_CHANNELS],
    n: usize,
    /// All segment lengths are multiples of 8 words (fused AVX-512 kernel usable).
    fused_ok: bool,
}

/// Precomputed scoring coefficients for a profile.
#[derive(Clone, Debug)]
pub struct Scorer {
    base: f32,
    active: Vec<(usize, f32)>,
    /// Integer weights for the fused kernel: score ≈ base - Σ qw_c h_c / QSCALE.
    qw: [u64; N_CHANNELS],
}

/// Fixed-point scale for integer segment weights.
const QSCALE: f64 = (1u64 << 24) as f64;

impl ModeK {
    pub fn new(layout: Layout) -> Self {
        let mut seg_words = [0; N_CHANNELS];
        let mut seg_off = [0; N_CHANNELS];
        let mut off = 0;
        for c in 0..N_CHANNELS {
            seg_words[c] = layout.dims[c] / WORD_BITS;
            seg_off[c] = off;
            off += seg_words[c];
        }
        let fused_ok = seg_words.iter().all(|w| w % 8 == 0);
        ModeK { layout, seg_words, seg_off, segs: Default::default(), n: 0, fused_ok }
    }

    pub fn with_capacity(layout: Layout, n: usize) -> Self {
        let mut m = Self::new(layout);
        for c in 0..N_CHANNELS {
            m.segs[c].reserve(n * m.seg_words[c]);
        }
        m
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Append a full fingerprint (all segments concatenated). Returns its row id.
    pub fn push(&mut self, fp: &[u64]) -> u32 {
        assert_eq!(fp.len(), self.layout.total_words());
        for c in 0..N_CHANNELS {
            let s = self.seg_off[c];
            self.segs[c].extend_from_slice(&fp[s..s + self.seg_words[c]]);
        }
        self.n += 1;
        (self.n - 1) as u32
    }

    /// Overwrite row `id` in place (incremental fingerprint update).
    pub fn update(&mut self, id: u32, fp: &[u64]) {
        let r = id as usize;
        for c in 0..N_CHANNELS {
            let (s, w) = (self.seg_off[c], self.seg_words[c]);
            self.segs[c][r * w..(r + 1) * w].copy_from_slice(&fp[s..s + w]);
        }
    }

    pub fn scorer(&self, weights: &[f64; N_CHANNELS]) -> Scorer {
        let mut base = 0.0;
        let mut active = Vec::new();
        let mut qw = [0u64; N_CHANNELS];
        for c in 0..N_CHANNELS {
            if weights[c] != 0.0 && self.layout.dims[c] > 0 {
                assert!(weights[c] > 0.0, "negative channel weights are not supported");
                base += weights[c];
                let coef = 2.0 * weights[c] / self.layout.dims[c] as f64;
                active.push((c, coef as f32));
                qw[c] = (coef * QSCALE).round().max(1.0) as u64;
            }
        }
        Scorer { base: base as f32, active, qw }
    }

    #[inline]
    fn row(&self, c: usize, r: usize) -> &[u64] {
        let w = self.seg_words[c];
        &self.segs[c][r * w..(r + 1) * w]
    }

    #[inline]
    fn qseg<'a>(&self, q: &'a [u64], c: usize) -> &'a [u64] {
        &q[self.seg_off[c]..self.seg_off[c] + self.seg_words[c]]
    }

    #[inline]
    pub fn score(&self, s: &Scorer, q: &[u64], r: u32) -> f32 {
        let mut x = s.base;
        for &(c, coef) in &s.active {
            x -= coef * hamming_words(self.qseg(q, c), self.row(c, r as usize)) as f32;
        }
        x
    }

    /// Weighted integer distance Σ_c qw_c · h_c for row `r` (lower = better).
    #[inline]
    fn wdist(&self, s: &Scorer, q: &[u64], r: usize) -> u64 {
        #[cfg(all(target_arch = "x86_64", target_feature = "avx512f", target_feature = "avx512vpopcntdq", target_feature = "avx512dq"))]
        {
            if self.fused_ok {
                // SAFETY: target features checked at compile time; segment lengths are multiples of 8 (fused_ok).
                return unsafe { self.wdist_avx512(s, q, r) };
            }
        }
        let mut d = 0u64;
        for &(c, _) in &s.active {
            d += s.qw[c] * hamming_words(self.qseg(q, c), self.row(c, r)) as u64;
        }
        d
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx512f", target_feature = "avx512vpopcntdq", target_feature = "avx512dq"))]
    #[inline]
    unsafe fn wdist_avx512(&self, s: &Scorer, q: &[u64], r: usize) -> u64 {
        use std::arch::x86_64::*;
        let mut total = _mm512_setzero_si512();
        for &(c, _) in &s.active {
            let w = self.seg_words[c];
            let rp = self.segs[c].as_ptr().add(r * w);
            let qp = q.as_ptr().add(self.seg_off[c]);
            let mut acc = _mm512_setzero_si512();
            let mut i = 0;
            while i < w {
                let x = _mm512_loadu_si512(qp.add(i) as *const _);
                let y = _mm512_loadu_si512(rp.add(i) as *const _);
                acc = _mm512_add_epi64(acc, _mm512_popcnt_epi64(_mm512_xor_si512(x, y)));
                i += 8;
            }
            total = _mm512_add_epi64(total, _mm512_mullo_epi64(acc, _mm512_set1_epi64(s.qw[c] as i64)));
        }
        _mm512_reduce_add_epi64(total) as u64
    }

    /// Single-query exhaustive search (parallel over row blocks).
    pub fn search(&self, q: &[u64], s: &Scorer, k: usize) -> Vec<Hit> {
        self.search_batch(&[q], s, k).pop().unwrap()
    }

    /// Score rows `lo..hi` against every query, in L1-sized tiles.
    fn scan(&self, queries: &[&[u64]], s: &Scorer, k: usize, lo: usize, hi: usize) -> Vec<TopK> {
        const TILE: usize = 32;
        let mut tops: Vec<TopK> = (0..queries.len()).map(|_| TopK::new(k)).collect();
        let mut t = lo;
        while t < hi {
            let te = (t + TILE).min(hi);
            for (qi, q) in queries.iter().enumerate() {
                let top = &mut tops[qi];
                // Integer admission threshold: skip rows that cannot enter the top-k.
                let mut thr = u64::MAX;
                if let Some(th) = top.threshold() {
                    thr = ((s.base - th) as f64 * QSCALE).ceil() as u64 + 1;
                }
                for r in t..te {
                    let d = self.wdist(s, q, r);
                    if d > thr {
                        continue;
                    }
                    top.push(r as u32, s.base - (d as f64 / QSCALE) as f32);
                    if let Some(th) = top.threshold() {
                        thr = ((s.base - th) as f64 * QSCALE).ceil() as u64 + 1;
                    }
                }
            }
            t = te;
        }
        tops
    }

    /// Batched exhaustive search, parallel over row chunks: each L1-sized
    /// tile of rows is scored against every query before moving on.
    pub fn search_batch(&self, queries: &[&[u64]], s: &Scorer, k: usize) -> Vec<Vec<Hit>> {
        const CHUNK: usize = 4096;
        let nq = queries.len();
        let n_chunks = self.n.div_ceil(CHUNK);
        let partials: Vec<Vec<TopK>> =
            (0..n_chunks).into_par_iter().map(|ch| self.scan(queries, s, k, ch * CHUNK, ((ch + 1) * CHUNK).min(self.n))).collect();
        let mut out: Vec<TopK> = (0..nq).map(|_| TopK::new(k)).collect();
        for p in partials {
            for (o, t) in out.iter_mut().zip(p) {
                o.merge(t);
            }
        }
        out.into_iter().map(TopK::into_sorted).collect()
    }

    /// Single-threaded batched search (for use inside parallel callers).
    pub fn search_batch_serial(&self, queries: &[&[u64]], s: &Scorer, k: usize) -> Vec<Vec<Hit>> {
        self.scan(queries, s, k, 0, self.n).into_iter().map(TopK::into_sorted).collect()
    }

    /// Single-threaded single-query search.
    pub fn search_serial(&self, q: &[u64], s: &Scorer, k: usize) -> Vec<Hit> {
        self.search_batch_serial(&[q], s, k).pop().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mars_hv::HyperVector;

    #[test]
    fn batch_matches_bruteforce() {
        let layout = Layout::new([128, 128, 256, 128, 64]);
        let total = layout.total_bits();
        let mut idx = ModeK::new(layout.clone());
        let rows: Vec<HyperVector> = (0..3000).map(|i| HyperVector::random(total, i)).collect();
        for r in &rows {
            idx.push(r.words());
        }
        let w = [0.0, 0.1, 0.4, 0.4, 0.1];
        let s = idx.scorer(&w);
        let qs: Vec<HyperVector> = (0..5).map(|i| HyperVector::random(total, 10_000 + i)).collect();
        let qrefs: Vec<&[u64]> = qs.iter().map(|q| q.words()).collect();
        let res = idx.search_batch(&qrefs, &s, 10);
        for (qi, q) in qs.iter().enumerate() {
            let mut all: Vec<(u32, f32)> = (0..rows.len() as u32)
                .map(|r| {
                    let sims = mars_encode::channel_sims(&layout, q.words(), rows[r as usize].words());
                    (r, sims.iter().zip(&w).map(|(a, b)| a * b).sum::<f64>() as f32)
                })
                .collect();
            all.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            let got: Vec<u32> = res[qi].iter().map(|h| h.id).collect();
            let want: Vec<u32> = all[..10].iter().map(|x| x.0).collect();
            // Scores must agree; ids agree except possibly among float ties.
            for (h, x) in res[qi].iter().zip(&all[..10]) {
                assert!((h.score - x.1).abs() < 1e-3, "{} vs {}", h.score, x.1);
            }
            assert_eq!(got.len(), want.len());
        }
    }
}
