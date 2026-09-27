//! Sparse Distributed Memory modes (DESIGN §7.4–7.5).
//!
//! Both modes address hard locations with the *same* weighted segment
//! distance as Mode K (a profile's scorer), using top-A activation (the A
//! nearest hard locations) instead of a fixed radius, so load stays
//! balanced (Bricken et al. 2023 use the same Top-K activation).
//!
//! * [`SdmBucket`] (Mode B): hard locations hold posting lists of case ids.
//!   A query activates its A_q nearest locations and exactly re-scores the
//!   union of their postings. With learned (k-means) addresses and A = 1
//!   this is IVF; with random addresses it is Kanerva's geometry.
//! * [`SdmAuto`] (Mode A): classical autoassociative SDM with i16 counters.
//!   Writes add the bipolar pattern to the counters of the activated
//!   locations; reads sum those counters and threshold, optionally iterated.

use crate::modek::{ModeK, Scorer};
use crate::topk::{Hit, TopK};
use mars_encode::Layout;
use mars_hv::{Accumulator, HyperVector, Rng, WORD_BITS};
use rayon::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AddressMode {
    /// Uniformly random addresses (Kanerva).
    Random,
    /// Addresses copied from a random sample of the data.
    DataSample,
    /// Data-sampled addresses refined by `iters` rounds of binary k-means
    /// (assign to nearest, recompute bitwise majority).
    KMeans { iters: usize },
}

fn random_words(n_words: usize, rng: &mut Rng) -> Vec<u64> {
    let mut v = vec![0u64; n_words];
    rng.fill(&mut v);
    v
}

/// Build hard-location addresses.
pub fn build_addresses(layout: &Layout, data: &[Vec<u64>], h: usize, mode: AddressMode, scorer_weights: &[f64; mars_encode::N_CHANNELS], seed: u64) -> Vec<Vec<u64>> {
    let words = layout.total_words();
    let mut rng = Rng::new(seed);
    let mut addrs: Vec<Vec<u64>> = match mode {
        AddressMode::Random => (0..h).map(|_| random_words(words, &mut rng)).collect(),
        AddressMode::DataSample | AddressMode::KMeans { .. } => rng.sample_indices(data.len(), h.min(data.len())).into_iter().map(|i| data[i].clone()).collect(),
    };
    if let AddressMode::KMeans { iters } = mode {
        let dim = layout.total_bits();
        for it in 0..iters {
            let mut idx = ModeK::with_capacity(layout.clone(), addrs.len());
            for a in &addrs {
                idx.push(a);
            }
            let s = idx.scorer(scorer_weights);
            let assign: Vec<u32> = data.par_iter().map(|x| idx.search(x, &s, 1)[0].id).collect();
            let mut accs: Vec<Option<Accumulator>> = (0..addrs.len()).map(|_| None).collect();
            for (x, &c) in data.iter().zip(&assign) {
                let acc = accs[c as usize].get_or_insert_with(|| Accumulator::new(dim));
                acc.add_words(x, 1);
            }
            let tie = HyperVector::random(dim, seed ^ (it as u64 + 1));
            for (a, acc) in addrs.iter_mut().zip(accs) {
                if let Some(acc) = acc {
                    *a = acc.threshold(&tie).into_words();
                }
            }
        }
    }
    addrs
}

pub struct SdmBucket {
    addrs: ModeK,
    postings: Vec<Vec<u32>>,
}

impl SdmBucket {
    /// Index `data` (row i = case id i). Each case is written to its `a_write` nearest locations.
    pub fn build(layout: &Layout, addrs: Vec<Vec<u64>>, data: &[Vec<u64>], scorer_weights: &[f64; mars_encode::N_CHANNELS], a_write: usize) -> Self {
        let mut idx = ModeK::with_capacity(layout.clone(), addrs.len());
        for a in &addrs {
            idx.push(a);
        }
        let s = idx.scorer(scorer_weights);
        let assign: Vec<Vec<u32>> = data.par_chunks(64).flat_map_iter(|chunk| {
            let qs: Vec<&[u64]> = chunk.iter().map(|x| x.as_slice()).collect();
            idx.search_batch_serial(&qs, &s, a_write).into_iter().map(|hits| hits.into_iter().map(|h| h.id).collect::<Vec<_>>())
        }).collect();
        let mut postings = vec![Vec::new(); addrs.len()];
        for (i, locs) in assign.iter().enumerate() {
            for &l in locs {
                postings[l as usize].push(i as u32);
            }
        }
        SdmBucket { addrs: idx, postings }
    }

    pub fn n_locations(&self) -> usize {
        self.postings.len()
    }

    pub fn load_stats(&self) -> (f64, usize) {
        let n: usize = self.postings.iter().map(Vec::len).sum();
        (n as f64 / self.postings.len() as f64, self.postings.iter().map(Vec::len).max().unwrap_or(0))
    }

    /// Candidate ids from the `a_query` nearest locations (deduplicated).
    pub fn candidates(&self, q: &[u64], s_addr: &Scorer, a_query: usize) -> Vec<u32> {
        let locs = self.addrs.search_serial(q, s_addr, a_query);
        let mut c: Vec<u32> = locs.iter().flat_map(|h| self.postings[h.id as usize].iter().copied()).collect();
        c.sort_unstable();
        c.dedup();
        c
    }

    pub fn address_scorer(&self, weights: &[f64; mars_encode::N_CHANNELS]) -> Scorer {
        self.addrs.scorer(weights)
    }

    /// Candidate generation + exact re-scoring against `data_index`.
    pub fn search(&self, q: &[u64], data_index: &ModeK, s_data: &Scorer, s_addr: &Scorer, a_query: usize, k: usize) -> (Vec<Hit>, usize) {
        let cands = self.candidates(q, s_addr, a_query);
        let mut top = TopK::new(k);
        for &c in &cands {
            top.push(c, data_index.score(s_data, q, c));
        }
        (top.into_sorted(), cands.len())
    }
}

/// Classical autoassociative SDM with i16 counters over the full fingerprint.
pub struct SdmAuto {
    addrs: ModeK,
    counters: Vec<i16>,
    dim: usize,
    pub a: usize,
    tie: HyperVector,
    pub writes: usize,
}

impl SdmAuto {
    pub fn new(layout: &Layout, addrs: Vec<Vec<u64>>, a: usize, seed: u64) -> Self {
        let dim = layout.total_bits();
        let mut idx = ModeK::with_capacity(layout.clone(), addrs.len());
        for x in &addrs {
            idx.push(x);
        }
        let h = addrs.len();
        SdmAuto { addrs: idx, counters: vec![0; h * dim], dim, a, tie: HyperVector::random(dim, seed ^ 0x5D3), writes: 0 }
    }

    pub fn memory_bytes(&self) -> usize {
        self.counters.len() * 2
    }

    pub fn activate(&self, x: &[u64], s: &Scorer) -> Vec<u32> {
        self.addrs.search_serial(x, s, self.a).into_iter().map(|h| h.id).collect()
    }

    pub fn scorer(&self, weights: &[f64; mars_encode::N_CHANNELS]) -> Scorer {
        self.addrs.scorer(weights)
    }

    /// Write pattern `x` at the locations activated by `addr` (autoassociative: addr = x).
    pub fn write(&mut self, addr: &[u64], x: &[u64], s: &Scorer) {
        let locs = self.activate(addr, s);
        self.write_at(&locs, x);
    }

    pub fn write_at(&mut self, locs: &[u32], x: &[u64]) {
        for &l in locs {
            let row = &mut self.counters[l as usize * self.dim..(l as usize + 1) * self.dim];
            for (wi, &w) in x.iter().enumerate() {
                let base = wi * WORD_BITS;
                for b in 0..WORD_BITS {
                    let delta = if (w >> b) & 1 == 1 { 1 } else { -1 };
                    let c = &mut row[base + b];
                    *c = c.saturating_add(delta);
                }
            }
        }
        self.writes += 1;
    }

    /// Parallel batch write (autoassociative): activations are computed in
    /// parallel, then each location's counters are updated independently.
    pub fn write_batch(&mut self, patterns: &[&[u64]], s: &Scorer) {
        let locs: Vec<Vec<u32>> = patterns.par_iter().map(|x| self.activate(x, s)).collect();
        let h = self.counters.len() / self.dim;
        let mut per_loc: Vec<Vec<u32>> = vec![Vec::new(); h];
        for (i, ls) in locs.iter().enumerate() {
            for &l in ls {
                per_loc[l as usize].push(i as u32);
            }
        }
        let dim = self.dim;
        self.counters.par_chunks_mut(dim).zip(per_loc.par_iter()).for_each(|(row, items)| {
            for &i in items {
                for (wi, &w) in patterns[i as usize].iter().enumerate() {
                    let base = wi * WORD_BITS;
                    for b in 0..WORD_BITS {
                        let delta: i16 = if (w >> b) & 1 == 1 { 1 } else { -1 };
                        row[base + b] = row[base + b].saturating_add(delta);
                    }
                }
            }
        });
        self.writes += patterns.len();
    }

    /// One read: sum the counters of the locations activated by `cue`, threshold.
    pub fn read(&self, cue: &[u64], s: &Scorer) -> Vec<u64> {
        let locs = self.activate(cue, s);
        let mut sums = vec![0i32; self.dim];
        for &l in &locs {
            let row = &self.counters[l as usize * self.dim..(l as usize + 1) * self.dim];
            for (acc, &c) in sums.iter_mut().zip(row) {
                *acc += c as i32;
            }
        }
        let mut out = vec![0u64; self.dim / WORD_BITS];
        for (wi, o) in out.iter_mut().enumerate() {
            let t = self.tie.words()[wi];
            let mut word = 0u64;
            for b in 0..WORD_BITS {
                let v = sums[wi * WORD_BITS + b];
                if v > 0 || (v == 0 && (t >> b) & 1 == 1) {
                    word |= 1 << b;
                }
            }
            *o = word;
        }
        out
    }

    /// Iterated read (fixed point search), `iters` ≥ 1.
    pub fn recall(&self, cue: &[u64], s: &Scorer, iters: usize) -> Vec<u64> {
        let mut x = self.read(cue, s);
        for _ in 1..iters {
            x = self.read(&x, s);
        }
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flip(x: &[u64], frac: f64, rng: &mut Rng) -> Vec<u64> {
        let mut y = x.to_vec();
        let bits = x.len() * 64;
        for i in rng.sample_indices(bits, (bits as f64 * frac) as usize) {
            y[i / 64] ^= 1 << (i % 64);
        }
        y
    }

    #[test]
    fn autoassociative_cleanup_of_noisy_cues() {
        let layout = Layout::new([128, 128, 256, 256, 256]);
        let w = [0.2, 0.2, 0.2, 0.2, 0.2];
        let mut rng = Rng::new(3);
        let words = layout.total_words();
        let pats: Vec<Vec<u64>> = (0..40).map(|_| random_words(words, &mut rng)).collect();
        let addrs = build_addresses(&layout, &pats, 2000, AddressMode::Random, &w, 7);
        let mut sdm = SdmAuto::new(&layout, addrs, 40, 1);
        let s = sdm.scorer(&w);
        for p in &pats {
            sdm.write(p, p, &s);
        }
        let dist = |a: &[u64], b: &[u64]| mars_hv::hamming_words(a, b);
        let mut improved = 0;
        for p in &pats {
            let cue = flip(p, 0.15, &mut rng);
            let out = sdm.recall(&cue, &s, 3);
            if dist(&out, p) < dist(&cue, p) {
                improved += 1;
            }
        }
        assert!(improved >= 36, "cleanup improved only {improved}/40 cues");
    }

    #[test]
    fn bucket_with_all_locations_is_exhaustive() {
        let layout = Layout::new([64, 64, 128, 128, 128]);
        let w = [0.0, 0.1, 0.4, 0.4, 0.1];
        let mut rng = Rng::new(5);
        let data: Vec<Vec<u64>> = (0..500).map(|_| random_words(layout.total_words(), &mut rng)).collect();
        let addrs = build_addresses(&layout, &data, 16, AddressMode::KMeans { iters: 2 }, &w, 9);
        let b = SdmBucket::build(&layout, addrs, &data, &w, 1);
        let mut idx = ModeK::new(layout.clone());
        for x in &data {
            idx.push(x);
        }
        let s = idx.scorer(&w);
        let sa = b.address_scorer(&w);
        let (hits, n) = b.search(&data[3], &idx, &s, &sa, 16, 5);
        assert_eq!(n, 500);
        assert_eq!(hits[0].id, 3);
    }
}
