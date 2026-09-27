//! Mode K: exact exhaustive top-k over segmented fingerprints.
//!
//! Storage is structure-of-arrays *per channel segment*, so a profile that
//! weights a channel at zero (e.g. C0 surface in the analogy profile) never
//! reads that segment. This is a bandwidth saving, not just a compute one.
//!
//! Within a segment, rows are stored in **blocks of 8, word-interleaved**:
//! word `i` of rows `8b..8b+8` is one contiguous 512-bit vector. The kernel
//! XORs a broadcast query word against it, so each SIMD lane accumulates
//! one row's Hamming distance: no horizontal reductions, and a cheap 32-bit
//! multiply per channel for the profile weight. Several queries share each
//! loaded row vector (register blocking).
//!
//! Score for profile λ: `Σ_c λ_c (1 - 2 h_c / d_c)`, computed as
//! `base - Σ_c coef_c · h_c` with integer per-segment Hamming distances.
//!
//! Batched search tiles the corpus so that a block of rows stays in cache
//! while a batch of queries is scored against it, turning a memory-bound
//! scan into a compute-bound one (DESIGN §7.2).

use crate::topk::{Hit, TopK};
use mars_encode::{Layout, N_CHANNELS};
use mars_hv::WORD_BITS;
use rayon::prelude::*;

/// Rows per interleaved block (one 512-bit vector of 64-bit words).
const LANES: usize = 8;
/// Queries sharing each loaded row vector.
const QBLOCK: usize = 4;

#[derive(Clone, Debug)]
pub struct ModeK {
    layout: Layout,
    /// Words per row for each channel segment.
    seg_words: [usize; N_CHANNELS],
    /// Word offset of each segment within a full fingerprint.
    seg_off: [usize; N_CHANNELS],
    /// Per segment: blocks of `LANES` rows, word-interleaved (see module docs).
    segs: [Vec<u64>; N_CHANNELS],
    n: usize,
    /// Soft-deleted rows (skipped by searches).
    dead: Vec<bool>,
    n_dead: usize,
}

/// Precomputed scoring coefficients for a profile.
#[derive(Clone, Debug)]
pub struct Scorer {
    base: f32,
    active: Vec<(usize, f32)>,
    /// Integer weights: score ≈ base - Σ qw_c h_c / QSCALE (each < 2³²).
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
        ModeK { layout, seg_words, seg_off, segs: Default::default(), n: 0, dead: Vec::new(), n_dead: 0 }
    }

    pub fn with_capacity(layout: Layout, n: usize) -> Self {
        let mut m = Self::new(layout);
        let padded = n.div_ceil(LANES) * LANES;
        for c in 0..N_CHANNELS {
            m.segs[c].reserve(padded * m.seg_words[c]);
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

    /// Position of word `i` of row `r` in segment `c`.
    #[inline]
    fn pos(&self, c: usize, r: usize, i: usize) -> usize {
        ((r / LANES) * self.seg_words[c] + i) * LANES + r % LANES
    }

    fn write_row(&mut self, r: usize, fp: &[u64]) {
        for c in 0..N_CHANNELS {
            let s = self.seg_off[c];
            for i in 0..self.seg_words[c] {
                let p = self.pos(c, r, i);
                self.segs[c][p] = fp[s + i];
            }
        }
    }

    /// Append a full fingerprint (all segments concatenated). Returns its row id.
    pub fn push(&mut self, fp: &[u64]) -> u32 {
        assert_eq!(fp.len(), self.layout.total_words());
        if self.n.is_multiple_of(LANES) {
            for c in 0..N_CHANNELS {
                let len = self.segs[c].len() + self.seg_words[c] * LANES;
                self.segs[c].resize(len, 0);
            }
        }
        self.n += 1;
        self.dead.push(false);
        self.write_row(self.n - 1, fp);
        (self.n - 1) as u32
    }

    /// Soft-delete a row: it is skipped by all searches.
    pub fn remove(&mut self, id: u32) {
        if !self.dead[id as usize] {
            self.dead[id as usize] = true;
            self.n_dead += 1;
        }
    }

    pub fn is_alive(&self, id: u32) -> bool {
        !self.dead[id as usize]
    }

    /// Overwrite row `id` in place (incremental fingerprint update).
    pub fn update(&mut self, id: u32, fp: &[u64]) {
        assert_eq!(fp.len(), self.layout.total_words());
        self.write_row(id as usize, fp);
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
                assert!(qw[c] < 1 << 32, "channel weight too large for the integer kernel");
            }
        }
        Scorer { base: base as f32, active, qw }
    }

    #[inline]
    fn qseg<'a>(&self, q: &'a [u64], c: usize) -> &'a [u64] {
        &q[self.seg_off[c]..self.seg_off[c] + self.seg_words[c]]
    }

    fn hamming_row(&self, c: usize, q: &[u64], r: usize) -> u32 {
        self.qseg(q, c).iter().enumerate().map(|(i, &x)| (x ^ self.segs[c][self.pos(c, r, i)]).count_ones()).sum()
    }

    /// Score two full fingerprints (neither need be indexed); equals
    /// `score(s, a, r)` when row `r` holds `b`.
    #[inline]
    pub fn score_pair(&self, s: &Scorer, a: &[u64], b: &[u64]) -> f32 {
        let mut x = s.base;
        for &(c, coef) in &s.active {
            x -= coef * mars_hv::hamming_words(self.qseg(a, c), self.qseg(b, c)) as f32;
        }
        x
    }

    #[inline]
    pub fn score(&self, s: &Scorer, q: &[u64], r: u32) -> f32 {
        let mut x = s.base;
        for &(c, coef) in &s.active {
            x -= coef * self.hamming_row(c, q, r as usize) as f32;
        }
        x
    }

    /// Weighted integer distances Σ_c qw_c · h_c of the `LANES` rows of block
    /// `b` to each of `G` queries (lower = better).
    #[inline]
    fn block_dists<const G: usize>(&self, s: &Scorer, qs: &[&[u64]; G], b: usize) -> [[u64; LANES]; G] {
        #[cfg(all(target_arch = "x86_64", target_feature = "avx512f", target_feature = "avx512vpopcntdq"))]
        {
            // SAFETY: target features checked at compile time; block `b` exists
            // (segments are padded to whole blocks) and queries have full length.
            return unsafe { self.block_dists_avx512(s, qs, b) };
        }
        #[allow(unreachable_code)]
        {
            let mut out = [[0u64; LANES]; G];
            for &(c, _) in &s.active {
                let w = self.seg_words[c];
                let blk = &self.segs[c][b * w * LANES..(b + 1) * w * LANES];
                for g in 0..G {
                    let q = self.qseg(qs[g], c);
                    let mut acc = [0u64; LANES];
                    for i in 0..w {
                        for l in 0..LANES {
                            acc[l] += (q[i] ^ blk[i * LANES + l]).count_ones() as u64;
                        }
                    }
                    for l in 0..LANES {
                        out[g][l] += s.qw[c] * acc[l];
                    }
                }
            }
            out
        }
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx512f", target_feature = "avx512vpopcntdq"))]
    #[inline]
    unsafe fn block_dists_avx512<const G: usize>(&self, s: &Scorer, qs: &[&[u64]; G], b: usize) -> [[u64; LANES]; G] {
        use std::arch::x86_64::*;
        let mut tot = [_mm512_setzero_si512(); G];
        for &(c, _) in &s.active {
            let w = self.seg_words[c];
            let off = self.seg_off[c];
            let bp = self.segs[c].as_ptr().add(b * w * LANES);
            let mut acc = [_mm512_setzero_si512(); G];
            for i in 0..w {
                let y = _mm512_loadu_si512(bp.add(i * LANES) as *const _);
                for g in 0..G {
                    let x = _mm512_set1_epi64(*qs[g].get_unchecked(off + i) as i64);
                    acc[g] = _mm512_add_epi64(acc[g], _mm512_popcnt_epi64(_mm512_xor_si512(x, y)));
                }
            }
            // Per-lane counts and weights both fit in 32 bits: one-uop multiply.
            let wv = _mm512_set1_epi64(s.qw[c] as i64);
            for g in 0..G {
                tot[g] = _mm512_add_epi64(tot[g], _mm512_mul_epu32(acc[g], wv));
            }
        }
        let mut out = [[0u64; LANES]; G];
        for g in 0..G {
            _mm512_storeu_si512(out[g].as_mut_ptr() as *mut _, tot[g]);
        }
        out
    }

    /// Single-query exhaustive search (parallel over row blocks).
    pub fn search(&self, q: &[u64], s: &Scorer, k: usize) -> Vec<Hit> {
        self.search_batch(&[q], s, k).pop().unwrap()
    }

    #[inline]
    fn admit(&self, s: &Scorer, top: &mut TopK, thr: &mut u64, r: usize, d: u64) {
        if d > *thr || (self.n_dead > 0 && self.dead[r]) {
            return;
        }
        top.push(r as u32, s.base - (d as f64 / QSCALE) as f32);
        if let Some(th) = top.threshold() {
            // Integer admission threshold: rows with d > thr cannot enter the top-k.
            *thr = ((s.base - th) as f64 * QSCALE).ceil() as u64 + 1;
        }
    }

    fn scan_group<const G: usize>(&self, s: &Scorer, qs: &[&[u64]; G], tops: &mut [TopK], thrs: &mut [u64], blocks: std::ops::Range<usize>, rows: &std::ops::Range<usize>) {
        for b in blocks {
            let d = self.block_dists(s, qs, b);
            for g in 0..G {
                for l in 0..LANES {
                    let r = b * LANES + l;
                    if rows.contains(&r) {
                        self.admit(s, &mut tops[g], &mut thrs[g], r, d[g][l]);
                    }
                }
            }
        }
    }

    /// Score rows `lo..hi` against every query, in L1-sized tiles of blocks.
    fn scan(&self, queries: &[&[u64]], s: &Scorer, k: usize, lo: usize, hi: usize) -> Vec<TopK> {
        const TILE_BLOCKS: usize = 4;
        let mut tops: Vec<TopK> = (0..queries.len()).map(|_| TopK::new(k)).collect();
        let mut thrs = vec![u64::MAX; queries.len()];
        if lo >= hi {
            return tops;
        }
        let (b_lo, b_hi) = (lo / LANES, hi.div_ceil(LANES));
        let mut t = b_lo;
        while t < b_hi {
            let te = (t + TILE_BLOCKS).min(b_hi);
            let mut qi = 0;
            while qi + QBLOCK <= queries.len() {
                let qs: [&[u64]; QBLOCK] = std::array::from_fn(|g| queries[qi + g]);
                self.scan_group(s, &qs, &mut tops[qi..qi + QBLOCK], &mut thrs[qi..qi + QBLOCK], t..te, &(lo..hi));
                qi += QBLOCK;
            }
            while qi < queries.len() {
                self.scan_group(s, &[queries[qi]], &mut tops[qi..qi + 1], &mut thrs[qi..qi + 1], t..te, &(lo..hi));
                qi += 1;
            }
            t = te;
        }
        tops
    }

    /// Batched exhaustive search, parallel over row chunks: each L1-sized
    /// tile of rows is scored against every query before moving on.
    pub fn search_batch(&self, queries: &[&[u64]], s: &Scorer, k: usize) -> Vec<Vec<Hit>> {
        const CHUNK: usize = 4096; // a multiple of LANES
        for q in queries {
            assert_eq!(q.len(), self.layout.total_words());
        }
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
        for q in queries {
            assert_eq!(q.len(), self.layout.total_words());
        }
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

    /// Reference: score every live row with `score()` and keep the top-k by
    /// the kernel's integer ordering (no admission threshold, no tiling).
    fn reference(idx: &ModeK, q: &[u64], s: &Scorer, k: usize) -> Vec<Hit> {
        let mut t = TopK::new(k);
        for r in 0..idx.len() {
            if idx.is_alive(r as u32) {
                let d: u64 = s.active.iter().map(|&(c, _)| s.qw[c] * idx.hamming_row(c, q, r) as u64).sum();
                t.push(r as u32, s.base - (d as f64 / QSCALE) as f32);
            }
        }
        t.into_sorted()
    }

    #[test]
    fn exact_ids_with_odd_sizes_deletes_and_updates() {
        let layout = Layout::new([64, 128, 256, 128, 64]);
        let total = layout.total_bits();
        for &n in &[1usize, 7, 8, 9, 1003, 5000] {
            let mut idx = ModeK::new(layout.clone());
            for i in 0..n {
                idx.push(HyperVector::random(total, i as u64).words());
            }
            // Some deletes and in-place updates.
            for r in (0..n).step_by(17) {
                idx.remove(r as u32);
            }
            for r in (3..n).step_by(29) {
                idx.update(r as u32, HyperVector::random(total, 77_000 + r as u64).words());
            }
            // Near-duplicate queries so that top-k and ties are non-trivial.
            let qs: Vec<HyperVector> = (0..7).map(|i| if i % 2 == 0 { HyperVector::random(total, 5 + i) } else { HyperVector::random(total, (i * 31) % n as u64) }).collect();
            let qrefs: Vec<&[u64]> = qs.iter().map(|q| q.words()).collect();
            for w in [[0.0, 0.1, 0.4, 0.4, 0.1], [0.2, 0.2, 0.2, 0.2, 0.2]] {
                let s = idx.scorer(&w);
                for k in [1usize, 10, 64] {
                    let par = idx.search_batch(&qrefs, &s, k);
                    let ser = idx.search_batch_serial(&qrefs, &s, k);
                    for (qi, q) in qrefs.iter().enumerate() {
                        let want = reference(&idx, q, &s, k);
                        assert_eq!(par[qi], want, "parallel n={n} k={k} q={qi}");
                        assert_eq!(ser[qi], want, "serial n={n} k={k} q={qi}");
                        for h in &want {
                            let row = if (3..n).contains(&(h.id as usize)) && (h.id as usize - 3).is_multiple_of(29) { HyperVector::random(total, 77_000 + h.id as u64) } else { HyperVector::random(total, h.id as u64) };
                            assert_eq!(idx.score_pair(&s, q, row.words()), idx.score(&s, q, h.id));
                        }
                    }
                }
            }
        }
    }

}
