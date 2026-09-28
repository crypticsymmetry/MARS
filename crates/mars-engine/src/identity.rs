//! Identity channel: TF-IDF cosine over entity and predicate names (E27, E32).
//!
//! Fingerprints are entity-anonymous by design: they find cases with the same
//! *structure*. When instances are identified by shared entities (knowledge
//! graphs: the same university, country or award), exact identity overlap finds
//! better neighbours for copying values (E27, E31). This index keeps one IDF
//! weighted token vector per case (IDF frozen at construction, like the
//! fingerprint epoch, and persisted as `identity.idf`) and an inverted index
//! for candidate generation; it is maintained incrementally on every change.

use mars_encode::{cosine, lexical_tokens, FeatureStats, Features, SparseVec};
use mars_rel::{CaseId, Kb};
use rustc_hash::FxHashMap;

pub struct IdentityIndex {
    stats: FeatureStats,
    vecs: Vec<SparseVec>,
    postings: FxHashMap<u64, Vec<u32>>,
}

impl IdentityIndex {
    /// IDF statistics over the given cases.
    pub fn fit(kb: &Kb, cases: impl IntoIterator<Item = CaseId>) -> FeatureStats {
        let raw: Vec<Features> = cases.into_iter().map(|c| Features { channels: [lexical_tokens(kb, c), vec![], vec![], vec![], vec![]] }).collect();
        FeatureStats::fit(raw.iter())
    }

    /// Index every case of `kb` under frozen statistics.
    pub fn new(kb: &Kb, stats: FeatureStats) -> Self {
        let mut ix = IdentityIndex { stats, vecs: Vec::new(), postings: FxHashMap::default() };
        for c in 0..kb.n_cases() {
            ix.set(kb, CaseId(c as u32));
        }
        ix
    }

    pub fn stats(&self) -> &FeatureStats {
        &self.stats
    }

    fn vector(&self, kb: &Kb, c: CaseId) -> SparseVec {
        let mut v = lexical_tokens(kb, c);
        self.stats.apply_sparse(0, &mut v);
        v
    }

    /// Add or re-index case `c` (ids are dense: `c` is at most one past the end).
    pub fn set(&mut self, kb: &Kb, c: CaseId) {
        let i = c.0 as usize;
        let v = self.vector(kb, c);
        if i < self.vecs.len() {
            for (h, _) in &self.vecs[i] {
                if let Some(p) = self.postings.get_mut(h) {
                    p.retain(|&x| x != c.0);
                }
            }
            self.vecs[i] = v;
        } else {
            debug_assert_eq!(i, self.vecs.len());
            self.vecs.push(v);
        }
        for (h, _) in &self.vecs[i] {
            self.postings.entry(*h).or_default().push(c.0);
        }
    }

    /// Cosine similarity of two indexed cases.
    pub fn score(&self, a: CaseId, b: CaseId) -> f64 {
        cosine(&self.vecs[a.0 as usize], &self.vecs[b.0 as usize])
    }

    /// Top-`want` live cases by cosine to `q` (excluding `q`). Candidates come
    /// from the postings of `q`'s tokens; tokens shared by more than 10% of the
    /// cases (and more than 1,000) are skipped for candidate generation only.
    pub fn top(&self, q: CaseId, want: usize, alive: &[bool]) -> Vec<(CaseId, f64)> {
        let cap = (self.vecs.len() / 10).max(1000);
        let mut seen: FxHashMap<u32, ()> = FxHashMap::default();
        for (h, _) in &self.vecs[q.0 as usize] {
            if let Some(p) = self.postings.get(h) {
                if p.len() <= cap {
                    for &c in p {
                        if c != q.0 && alive[c as usize] {
                            seen.insert(c, ());
                        }
                    }
                }
            }
        }
        let mut v: Vec<(CaseId, f64)> = seen.into_keys().map(|c| (CaseId(c), self.score(q, CaseId(c)))).collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        v.truncate(want);
        v
    }
}
