//! `mars-engine`: the persistent, incremental analogical memory (DESIGN §10).
//!
//! * Cases live in a [`Kb`]; each has a fingerprint row in a Mode K index.
//!   The feature IDF statistics are frozen at construction (a *vocabulary
//!   epoch*), so every update is exactly incremental.
//! * **Standing queries**, two modes:
//!   - [`SqMode::Pipeline`] (default): the exact top-K by fingerprint score
//!     (the MAC candidate set) is maintained incrementally, and re-ranked by
//!     `fused(q,c) = w·FAC(q,c) + (1−w)·fp(q,c)` when it changes. Same
//!     semantics as the E3 pipeline, which matched the exhaustive bound.
//!   - [`SqMode::ExactFused`]: the exact top-k of `fused` over all live cases,
//!     via branch and bound on `fused ≤ w + (1−w)·fp` (FAC ∈ [0,1]). Exact but
//!     the bound is loose, so most updates must map the changed case (E6).
//!
//!   Both keep a buffer of `slack` extra members; every live non-member
//!   scores ≤ `floor`, so each update needs one O(1) test per standing query.
//! * Each standing query keeps candidate inferences from its best analogue
//!   in a JTMS. Every inference is justified by the mapping (a premise while
//!   current) *and* by the base facts it was projected from, so retracting a
//!   base fact withdraws exactly the dependent inferences, before any
//!   re-mapping happens.
//! * [`Work`] counts the units of work each update causes (E6).

pub mod sage;
mod store;

use mars_encode::{symbol_hashes, FeatureConfig, FeatureExtractor, FeatureStats, Features, Layout, Profile, Sketcher, N_CHANNELS};
use mars_index::{ModeK, Scorer};
use mars_map::{Grounding, MapConfig, Mapper};
use mars_rel::{CaseId, CaseKind, ExprId, Kb};
use mars_tms::{Jtms, NodeId};
use rayon::prelude::*;
use rustc_hash::FxHashMap;

#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub features: FeatureConfig,
    pub layout: Layout,
    pub seed: u64,
    pub map: MapConfig,
    pub profile: Profile,
    /// Weight of the structural (FAC) score in the fused score.
    pub fac_weight: f64,
    /// Extra results kept beyond k in each standing query (exact buffer).
    pub slack: usize,
    /// Candidate inferences are drawn from this many top analogues; each
    /// analogue is a separate justification (corroboration).
    pub infer_from: usize,
    /// Standing-query semantics.
    pub sq_mode: SqMode,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SqMode {
    /// Exact top-`mac_k` by fingerprint, re-ranked by fused score.
    Pipeline { mac_k: usize },
    /// Exact top-k by fused score over all live cases.
    ExactFused,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            features: FeatureConfig::default(),
            layout: Layout::default(),
            seed: 0xF1,
            map: MapConfig::default(),
            profile: Profile::analogy(),
            fac_weight: 0.3,
            slack: 8,
            infer_from: 5,
            sq_mode: SqMode::Pipeline { mac_k: 64 },
        }
    }
}

/// Work counters (cumulative).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Work {
    pub encodes: u64,
    pub rows_written: u64,
    pub sq_bound_checks: u64,
    pub fac_evals: u64,
    pub sq_full_recomputes: u64,
    pub sq_incremental_changes: u64,
    pub remaps: u64,
    pub tms_touched: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    ResultsChanged { sq: usize },
    InferenceIn { sq: usize, text: String },
    InferenceOut { sq: usize, text: String },
}

#[derive(Clone, Debug)]
pub struct StandingQuery {
    pub case: CaseId,
    pub k: usize,
    /// Exact top-|set| by the maintained key (fp in pipeline mode, fused in
    /// exact mode), descending (ties: ascending id).
    set: Vec<(CaseId, f64)>,
    /// Every live non-member has key ≤ floor.
    floor: f64,
    /// Final ranking by fused score (top-k).
    ranked: Vec<(CaseId, f64)>,
    /// (analogue, analogue version, query version) of the current mapping.
    /// (analogue, analogue version) list + query version of the current mappings.
    mapped: Option<(Vec<(CaseId, u64)>, u64)>,
    mapping_nodes: Vec<NodeId>,
}

impl StandingQuery {
    pub fn top(&self) -> &[(CaseId, f64)] {
        &self.ranked
    }
}

pub struct Engine {
    pub kb: Kb,
    pub cfg: EngineConfig,
    stats: FeatureStats,
    sym_hash: Vec<u64>,
    sketcher: Sketcher,
    index: ModeK,
    scorer: Scorer,
    fps: Vec<Vec<u64>>,
    alive: Vec<bool>,
    self_score: Vec<Option<(u64, f32)>>,
    fac_memo: FxHashMap<(CaseId, CaseId), (u64, u64, f64)>,
    sqs: Vec<StandingQuery>,
    tms: Jtms,
    fact_nodes: FxHashMap<(CaseId, ExprId), NodeId>,
    /// (sq, projected text) → inference node.
    inf_nodes: FxHashMap<(usize, String), NodeId>,
    /// Provenance: inference node → (analogue case, base facts).
    inf_prov: FxHashMap<NodeId, Vec<(CaseId, Vec<ExprId>, mars_tms::JustId)>>,
    pub work: Work,
    events: Vec<Event>,
    log: Option<std::io::BufWriter<std::fs::File>>,
    replaying: bool,
}

/// Local-null z at or above which a top-1 analogue is accepted as real
/// (E16: precision ≈ 0.9 at every memory size from 10³ to 10⁶ cases).
pub const SIGNIFICANT_Z: f64 = 9.0;

impl Engine {
    /// Build from an existing KB: freeze an IDF epoch over its cases and index them all.
    pub fn new(kb: Kb, cfg: EngineConfig) -> Self {
        let n = kb.n_cases();
        let stats = {
            let fx = FeatureExtractor::new(&kb, cfg.features.clone());
            let raw: Vec<Features> = (0..n).into_par_iter().map(|c| fx.extract(CaseId(c as u32))).collect();
            FeatureStats::fit(raw.iter())
        };
        Self::with_stats(kb, cfg, stats)
    }

    /// Build with a given (e.g. persisted) IDF epoch.
    pub fn with_stats(kb: Kb, cfg: EngineConfig, stats: FeatureStats) -> Self {
        let sym_hash = symbol_hashes(&kb);
        let n = kb.n_cases();
        let raw: Vec<Features> = {
            let fx = FeatureExtractor::with_hashes(&kb, cfg.features.clone(), &sym_hash);
            (0..n).into_par_iter().map(|c| fx.extract(CaseId(c as u32))).collect()
        };
        let sketcher = Sketcher::new(cfg.layout.clone(), cfg.seed);
        let fps: Vec<Vec<u64>> = raw
            .into_par_iter()
            .map(|mut f| {
                stats.apply(&mut f, &[true; N_CHANNELS]);
                sketcher.sketch(&f).into_words()
            })
            .collect();
        let mut index = ModeK::with_capacity(cfg.layout.clone(), n);
        for f in &fps {
            index.push(f);
        }
        let scorer = index.scorer(&cfg.profile.weights);
        Engine {
            kb,
            stats,
            sym_hash,
            sketcher,
            index,
            scorer,
            fps,
            alive: vec![true; n],
            self_score: vec![None; n],
            fac_memo: FxHashMap::default(),
            sqs: Vec::new(),
            tms: Jtms::new(),
            fact_nodes: FxHashMap::default(),
            inf_nodes: FxHashMap::default(),
            inf_prov: FxHashMap::default(),
            work: Work::default(),
            events: Vec::new(),
            log: None,
            replaying: false,
            cfg,
        }
    }

    pub fn n_live(&self) -> usize {
        self.alive.iter().filter(|&&a| a).count()
    }

    pub fn is_alive(&self, c: CaseId) -> bool {
        self.alive[c.0 as usize]
    }

    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn standing(&self, sq: usize) -> &StandingQuery {
        &self.sqs[sq]
    }

    pub fn n_standing(&self) -> usize {
        self.sqs.len()
    }

    // ------------------------------------------------------------ encoding

    fn encode(&mut self, c: CaseId) -> Vec<u64> {
        self.work.encodes += 1;
        let fx = FeatureExtractor::with_hashes(&self.kb, self.cfg.features.clone(), &self.sym_hash);
        let mut f = fx.extract(c);
        self.stats.apply(&mut f, &[true; N_CHANNELS]);
        self.sketcher.sketch(&f).into_words()
    }

    fn fp_score(&self, q: CaseId, c: CaseId) -> f64 {
        self.index.score(&self.scorer, &self.fps[q.0 as usize], c.0) as f64
    }

    fn self_score(&mut self, c: CaseId) -> f32 {
        let v = self.kb.case(c).version;
        if let Some((ver, s)) = self.self_score[c.0 as usize] {
            if ver == v {
                return s;
            }
        }
        let s = Mapper::new(&self.kb, self.cfg.map.clone()).score(c, c);
        self.self_score[c.0 as usize] = Some((v, s));
        s
    }

    /// Normalized structural score in [0, 1], memoized by case versions.
    pub fn fac(&mut self, q: CaseId, c: CaseId) -> f64 {
        let (vq, vc) = (self.kb.case(q).version, self.kb.case(c).version);
        if let Some(&(a, b, s)) = self.fac_memo.get(&(q, c)) {
            if a == vq && b == vc {
                return s;
            }
        }
        self.work.fac_evals += 1;
        let raw = Mapper::new(&self.kb, self.cfg.map.clone()).score(c, q) as f64;
        let (sq, sc) = (self.self_score(q) as f64, self.self_score(c) as f64);
        let s = if raw == 0.0 || sq == 0.0 || sc == 0.0 { 0.0 } else { (raw / (sq * sc).sqrt()).min(1.0) };
        self.fac_memo.insert((q, c), (vq, vc, s));
        s
    }

    pub fn fused(&mut self, q: CaseId, c: CaseId) -> f64 {
        let w = self.cfg.fac_weight;
        w * self.fac(q, c) + (1.0 - w) * self.fp_score(q, c)
    }

    fn bound(&self, fp: f64) -> f64 {
        self.cfg.fac_weight + (1.0 - self.cfg.fac_weight) * fp
    }

    /// Minimum size of a standing query's maintained set.
    fn set_min(&self, k: usize) -> usize {
        match self.cfg.sq_mode {
            SqMode::Pipeline { mac_k } => mac_k.max(k),
            SqMode::ExactFused => k,
        }
    }

    /// One-off retrieval (not registered): top-k analogues of `q` by fused
    /// score over the fingerprint top-`mac_k` candidates.
    pub fn query(&mut self, q: CaseId, k: usize) -> Vec<(CaseId, f64)> {
        self.query_significance(q, k).0
    }

    /// `query` plus the significance of the top-1 analogue: the z-score of
    /// its fused score against the fused scores of the lower half of the
    /// fingerprint candidate list (a *local null*: the best chance matches
    /// in this memory). Unlike a raw score threshold, a z threshold keeps
    /// its precision as memory grows (E16: z ≥ 9 gives precision ≈ 0.9 from
    /// 10³ to 10⁶ cases). `None` with fewer than 8 candidates.
    pub fn query_significance(&mut self, q: CaseId, k: usize) -> (Vec<(CaseId, f64)>, Option<f64>) {
        let mac_k = match self.cfg.sq_mode {
            SqMode::Pipeline { mac_k } => mac_k,
            SqMode::ExactFused => 64,
        };
        let (top, _) = self.fp_top(q, mac_k + self.cfg.slack);
        let cands = Self::pipeline_candidates(&top, mac_k);
        let live: Vec<CaseId> = cands.into_iter().filter(|c| self.alive[c.0 as usize] && *c != q).collect();
        let mut r: Vec<(CaseId, f64)> = live.into_iter().map(|c| (c, self.fused(q, c))).collect();
        let z = (r.len() >= 8).then(|| {
            let tail: Vec<f64> = r[r.len() / 2..].iter().map(|x| x.1).collect();
            let m = tail.iter().sum::<f64>() / tail.len() as f64;
            let sd = (tail.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (tail.len() - 1) as f64).sqrt().max(1e-9);
            let best = r.iter().map(|x| x.1).fold(f64::MIN, f64::max);
            (best - m) / sd
        });
        sort_set(&mut r);
        r.truncate(k);
        (r, z)
    }

    /// Pipeline candidates: the first `mac_k` of a fingerprint-sorted set,
    /// plus anything tied with the `mac_k`-th (tie-inclusive, so the result
    /// does not depend on how ties at the boundary were broken).
    fn pipeline_candidates(set: &[(CaseId, f64)], mac_k: usize) -> Vec<CaseId> {
        if set.is_empty() {
            return Vec::new();
        }
        let t = set[mac_k.min(set.len()) - 1].1 - 1e-6;
        set.iter().take_while(|x| x.1 >= t).map(|x| x.0).collect()
    }

    /// Recompute the final fused ranking from the maintained set.
    fn rerank(&mut self, sq: usize) {
        let (q, k) = (self.sqs[sq].case, self.sqs[sq].k);
        let ranked = match self.cfg.sq_mode {
            SqMode::ExactFused => self.sqs[sq].set.iter().take(k).copied().collect(),
            SqMode::Pipeline { mac_k } => {
                let cands = Self::pipeline_candidates(&self.sqs[sq].set, mac_k);
                let mut r: Vec<(CaseId, f64)> = cands.into_iter().map(|c| (c, self.fused(q, c))).collect();
                sort_set(&mut r);
                r.truncate(k);
                r
            }
        };
        self.sqs[sq].ranked = ranked;
    }

    // ------------------------------------------------------------ mutation

    pub fn add_case(&mut self, name: &str, facts: Vec<ExprId>) -> CaseId {
        let c = self.kb.add_case(name, CaseKind::Episode, facts);
        let rendered = self.kb.render_case(c);
        self.log_line(format!("add-case {rendered}"));
        let fp = self.encode(c);
        let id = self.index.push(&fp);
        debug_assert_eq!(id, c.0);
        self.fps.push(fp);
        self.alive.push(true);
        self.self_score.push(None);
        self.work.rows_written += 1;
        self.case_changed(c);
        c
    }

    pub fn remove_case(&mut self, c: CaseId) {
        if !self.alive[c.0 as usize] {
            return;
        }
        let name = self.kb.name(self.kb.case(c).name).to_string();
        self.log_line(format!("remove-case {name}"));
        self.alive[c.0 as usize] = false;
        self.index.remove(c.0);
        let facts = self.kb.case(c).facts.clone();
        for f in facts {
            if let Some(&n) = self.fact_nodes.get(&(c, f)) {
                self.retract_premise(n);
            }
        }
        for sq in 0..self.sqs.len() {
            if self.sqs[sq].case == c {
                continue; // a removed query simply stops updating
            }
            if let Some(pos) = self.sqs[sq].set.iter().position(|x| x.0 == c) {
                self.sqs[sq].set.remove(pos);
                self.after_set_change(sq);
            }
        }
    }

    pub fn add_fact(&mut self, c: CaseId, fact: ExprId) -> bool {
        if !self.kb.add_fact(c, fact) {
            return false;
        }
        let line = format!("add-fact {} {}", self.kb.name(self.kb.case(c).name), self.kb.render_expr(fact));
        self.log_line(line);
        if let Some(&n) = self.fact_nodes.get(&(c, fact)) {
            let before = self.tms.touched;
            self.tms.assume(n);
            self.work.tms_touched += self.tms.touched - before;
        }
        self.refresh(c);
        true
    }

    pub fn remove_fact(&mut self, c: CaseId, fact: ExprId) -> bool {
        if !self.kb.remove_fact(c, fact) {
            return false;
        }
        let line = format!("remove-fact {} {}", self.kb.name(self.kb.case(c).name), self.kb.render_expr(fact));
        self.log_line(line);
        if let Some(&n) = self.fact_nodes.get(&(c, fact)) {
            self.retract_premise(n);
        }
        self.refresh(c);
        true
    }

    fn retract_premise(&mut self, n: NodeId) {
        let before = self.tms.touched;
        let out = self.tms.retract(n);
        self.work.tms_touched += self.tms.touched - before;
        self.emit_outs(&out);
    }

    fn emit_outs(&mut self, out: &[NodeId]) {
        let names: Vec<(usize, String)> = self.inf_nodes.iter().filter(|(_, n)| out.contains(n)).map(|(k, _)| k.clone()).collect();
        for (sq, text) in names {
            self.events.push(Event::InferenceOut { sq, text });
        }
    }

    /// Re-encode a changed case and propagate.
    fn refresh(&mut self, c: CaseId) {
        let fp = self.encode(c);
        self.index.update(c.0, &fp);
        self.fps[c.0 as usize] = fp;
        self.work.rows_written += 1;
        self.case_changed(c);
    }

    /// Propagate a changed/added case to every standing query.
    fn case_changed(&mut self, c: CaseId) {
        for sq in 0..self.sqs.len() {
            let q = self.sqs[sq].case;
            if !self.alive[q.0 as usize] {
                continue;
            }
            if q == c {
                self.full_recompute(sq);
                continue;
            }
            self.work.sq_bound_checks += 1;
            let member = self.sqs[sq].set.iter().position(|x| x.0 == c);
            let floor = self.sqs[sq].floor;
            let pipeline = matches!(self.cfg.sq_mode, SqMode::Pipeline { .. });
            match member {
                Some(pos) => {
                    let s = if pipeline { self.fp_score(q, c) } else { self.fused(q, c) };
                    if s >= floor {
                        self.sqs[sq].set[pos].1 = s;
                        sort_set(&mut self.sqs[sq].set);
                    } else {
                        self.sqs[sq].set.remove(pos);
                    }
                    self.work.sq_incremental_changes += 1;
                    self.after_set_change(sq);
                }
                None => {
                    let fp = self.fp_score(q, c);
                    let ub = if pipeline { fp } else { self.bound(fp) };
                    if ub <= floor {
                        continue;
                    }
                    let s = if pipeline { fp } else { self.fused(q, c) };
                    if s > floor {
                        let cap = self.set_min(self.sqs[sq].k) + self.cfg.slack;
                        let set = &mut self.sqs[sq].set;
                        set.push((c, s));
                        sort_set(set);
                        if set.len() > cap {
                            let (_, dropped) = set.pop().unwrap();
                            self.sqs[sq].floor = self.sqs[sq].floor.max(dropped);
                        }
                        self.work.sq_incremental_changes += 1;
                        self.after_set_change(sq);
                    }
                }
            }
        }
    }

    fn after_set_change(&mut self, sq: usize) {
        if self.sqs[sq].set.len() < self.set_min(self.sqs[sq].k) && self.index_live() > self.sqs[sq].set.len() + 1 {
            self.full_recompute(sq);
        } else {
            let old = self.sqs[sq].ranked.clone();
            self.rerank(sq);
            if self.sqs[sq].ranked != old {
                self.events.push(Event::ResultsChanged { sq });
            }
            self.update_inferences(sq);
        }
    }

    fn index_live(&self) -> usize {
        self.alive.iter().filter(|&&a| a).count()
    }

    // ------------------------------------------------------------ standing queries

    pub fn add_standing_query(&mut self, case: CaseId, k: usize) -> usize {
        let line = format!("standing {} {k}", self.kb.name(self.kb.case(case).name));
        self.log_line(line);
        self.sqs.push(StandingQuery { case, k, set: Vec::new(), floor: f64::NEG_INFINITY, ranked: Vec::new(), mapped: None, mapping_nodes: Vec::new() });
        let sq = self.sqs.len() - 1;
        self.full_recompute(sq);
        sq
    }

    /// Exact top-(k+slack) by fused score via branch and bound over the
    /// fingerprint ranking (fused ≤ w + (1−w)·fp).
    pub fn exact_top(&mut self, q: CaseId, want: usize) -> (Vec<(CaseId, f64)>, f64) {
        let mut m = 64usize;
        loop {
            let hits = self.index.search(&self.fps[q.0 as usize], &self.scorer, m + 1);
            let exhausted = hits.len() < m + 1;
            let mut set: Vec<(CaseId, f64)> = Vec::new();
            let mut stopped = false;
            for h in &hits {
                let c = CaseId(h.id);
                if c == q {
                    continue;
                }
                let kth = if set.len() >= want { set[want - 1].1 } else { f64::NEG_INFINITY };
                // The index ranks with fixed-point weights; keep a margin so the
                // bound stays valid against the exact float score.
                if set.len() >= want && self.bound(h.score as f64) + 1e-5 < kth {
                    stopped = true;
                    break;
                }
                let s = self.fused(q, c);
                set.push((c, s));
                sort_set(&mut set);
                set.truncate(want);
            }
            if stopped || exhausted {
                // Non-members: either below the stopping bound or not reached.
                let floor = if set.len() >= want { set[want - 1].1 } else { f64::NEG_INFINITY };
                return (set, floor);
            }
            m *= 4;
        }
    }

    /// Exact top-`want` by fingerprint score (excluding the query itself).
    fn fp_top(&self, q: CaseId, want: usize) -> (Vec<(CaseId, f64)>, f64) {
        let hits = self.index.search(&self.fps[q.0 as usize], &self.scorer, want + 1);
        let mut set: Vec<(CaseId, f64)> = hits.iter().filter(|h| h.id != q.0).map(|h| (CaseId(h.id), self.fp_score(q, CaseId(h.id)))).collect();
        sort_set(&mut set);
        set.truncate(want);
        // Margin covers fixed-point ranking vs float scores.
        let floor = if set.len() >= want { set[want - 1].1 + 1e-5 } else { f64::NEG_INFINITY };
        (set, floor)
    }

    fn full_recompute(&mut self, sq: usize) {
        self.work.sq_full_recomputes += 1;
        let (q, k) = (self.sqs[sq].case, self.sqs[sq].k);
        let want = self.set_min(k) + self.cfg.slack;
        let (set, floor) = match self.cfg.sq_mode {
            SqMode::Pipeline { .. } => self.fp_top(q, want),
            SqMode::ExactFused => self.exact_top(q, want),
        };
        self.sqs[sq].set = set;
        self.sqs[sq].floor = floor;
        self.rerank(sq);
        self.events.push(Event::ResultsChanged { sq });
        self.update_inferences(sq);
    }

    fn fact_node(&mut self, c: CaseId, f: ExprId) -> NodeId {
        if let Some(&n) = self.fact_nodes.get(&(c, f)) {
            return n;
        }
        let n = self.tms.add_node();
        if self.kb.case(c).facts.contains(&f) && self.alive[c.0 as usize] {
            self.tms.assume(n);
        }
        self.fact_nodes.insert((c, f), n);
        n
    }

    /// Candidate inferences from the top-`infer_from` analogues, one mapping
    /// per analogue. Returns (analogue, [(inference text, base facts)]).
    fn analogue_inferences(&self, q: CaseId, analogues: &[CaseId]) -> Vec<AnalogueInferences> {
        let m = Mapper::new(&self.kb, self.cfg.map.clone());
        analogues
            .iter()
            .map(|&a| {
                let infs = m
                    .best(a, q)
                    .map(|mp| {
                        mp.inferences
                            .iter()
                            .filter(|i| i.grounding == Grounding::Structural)
                            .map(|i| (m.render_proj(&i.projected), vec![i.base_fact]))
                            .collect()
                    })
                    .unwrap_or_default();
                (a, infs)
            })
            .collect()
    }

    fn top_analogues(&self, sq: usize) -> Vec<CaseId> {
        self.sqs[sq].ranked.iter().take(self.cfg.infer_from).map(|x| x.0).collect()
    }

    /// Keep candidate inferences in sync with the top analogues (via the TMS).
    fn update_inferences(&mut self, sq: usize) {
        let q = self.sqs[sq].case;
        let analogues = self.top_analogues(sq);
        let key = (analogues.iter().map(|&a| (a, self.kb.case(a).version)).collect::<Vec<_>>(), self.kb.case(q).version);
        if Some(&key) == self.sqs[sq].mapped.as_ref() {
            return;
        }
        self.work.remaps += 1;
        let before = self.tms.touched;
        let in_before: Vec<String> = self.inferences(sq);
        // Retire the old mapping premises (dependent inferences go OUT unless
        // re-justified below).
        for old in std::mem::take(&mut self.sqs[sq].mapping_nodes) {
            self.tms.retract(old);
        }
        self.sqs[sq].mapped = Some(key);
        let sq_nodes: Vec<NodeId> = self.inf_nodes.iter().filter(|((s, _), _)| *s == sq).map(|(_, &n)| n).collect();
        for n in sq_nodes {
            self.inf_prov.remove(&n);
        }
        for (a, infs) in self.analogue_inferences(q, &analogues) {
            let mapping_node = self.tms.add_node();
            self.tms.assume(mapping_node);
            self.sqs[sq].mapping_nodes.push(mapping_node);
            for (text, base_facts) in infs {
                let key = (sq, text.clone());
                let node = match self.inf_nodes.get(&key) {
                    Some(&n) => n,
                    None => {
                        let n = self.tms.add_node();
                        self.inf_nodes.insert(key, n);
                        n
                    }
                };
                let mut ants = vec![mapping_node];
                for &bf in &base_facts {
                    ants.push(self.fact_node(a, bf));
                }
                let j = self.tms.justify(node, &ants);
                self.inf_prov.entry(node).or_default().push((a, base_facts, j));
            }
        }
        // Events: net change in the believed set.
        let in_after = self.inferences(sq);
        for t in in_before.iter().filter(|t| !in_after.contains(t)) {
            self.events.push(Event::InferenceOut { sq, text: t.clone() });
        }
        for t in in_after.iter().filter(|t| !in_before.contains(t)) {
            self.events.push(Event::InferenceIn { sq, text: t.clone() });
        }
        self.work.tms_touched += self.tms.touched - before;
    }

    /// Believed inferences with support ≥ `min_support` (E11: support is a
    /// calibrated confidence; ≥ 2 of 5 analogues keeps single-analogue
    /// precision at ~1.6× the recall).
    pub fn corroborated(&self, sq: usize, min_support: usize) -> Vec<(String, usize)> {
        let mut v: Vec<(String, usize)> = self.inferences(sq).into_iter().map(|t| { let s = self.support(sq, &t); (t, s) }).filter(|x| x.1 >= min_support).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }

    /// Number of analogues currently supporting an inference.
    pub fn support(&self, sq: usize, text: &str) -> usize {
        self.inf_nodes.get(&(sq, text.to_string())).map(|&n| self.tms.support_count(n)).unwrap_or(0)
    }

    /// Currently believed (IN) inferences of a standing query, sorted.
    pub fn inferences(&self, sq: usize) -> Vec<String> {
        let mut v: Vec<String> = self.inf_nodes.iter().filter(|((s, _), n)| *s == sq && self.tms.is_in(**n)).map(|((_, t), _)| t.clone()).collect();
        v.sort();
        v
    }

    /// Provenance of an inference: every analogue currently supporting it
    /// and the base facts it was projected from.
    pub fn explain(&self, sq: usize, text: &str) -> Option<String> {
        let n = *self.inf_nodes.get(&(sq, text.to_string()))?;
        let provs = self.inf_prov.get(&n)?;
        let mut s = format!("{text}\n  believed: {} (supported by {} analogue(s))", self.tms.is_in(n), self.tms.support_count(n));
        for (a, facts, j) in provs {
            if !self.tms.just_holds(*j) {
                continue;
            }
            s.push_str(&format!("\n  by analogy with case {}", self.kb.name(self.kb.case(*a).name)));
            for f in facts {
                s.push_str(&format!("\n    from base fact {}", self.kb.render_expr(*f)));
            }
        }
        Some(s)
    }

    /// From-scratch reference: exact top-k and inference texts for a standing query.
    pub fn scratch(&mut self, sq: usize) -> (Vec<(CaseId, f64)>, Vec<String>) {
        let (q, k) = (self.sqs[sq].case, self.sqs[sq].k);
        self.fac_memo.clear();
        let set = match self.cfg.sq_mode {
            SqMode::ExactFused => self.exact_top(q, k).0,
            SqMode::Pipeline { mac_k } => {
                let (top, _) = self.fp_top(q, mac_k + self.cfg.slack);
                let cands = Self::pipeline_candidates(&top, mac_k);
                let mut r: Vec<(CaseId, f64)> = cands.into_iter().map(|c| (c, self.fused(q, c))).collect();
                sort_set(&mut r);
                r.truncate(k);
                r
            }
        };
        let analogues: Vec<CaseId> = set.iter().take(self.cfg.infer_from).map(|x| x.0).collect();
        let mut infs: Vec<String> = self
            .analogue_inferences(q, &analogues)
            .into_iter()
            .flat_map(|(_, v)| v.into_iter().map(|(t, _)| t))
            .collect();
        infs.sort();
        infs.dedup();
        (set, infs)
    }
}

/// (analogue, [(inference text, base facts it was projected from)])
type AnalogueInferences = (CaseId, Vec<(String, Vec<ExprId>)>);

fn sort_set(set: &mut [(CaseId, f64)]) {
    set.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = r#"
(defpredicate attracts :arity 2 :kind relation)
(defpredicate revolve-around :arity 2 :kind relation)
(defpredicate greater :arity 2 :kind relation)
(defpredicate mass :arity 1 :kind function)
(defpredicate cause :arity 2 :kind relation)
(defpredicate and :arity * :kind logical :commutative t)
(defcase solar
  (attracts sun planet)
  (greater (mass sun) (mass planet))
  (revolve-around planet sun)
  (cause (and (attracts sun planet) (greater (mass sun) (mass planet))) (revolve-around planet sun)))
(defcase unrelated
  (attracts x y) (attracts y z))
(defquery atom
  (attracts nucleus electron)
  (greater (mass nucleus) (mass electron))
  (revolve-around electron nucleus))
"#;

    fn setup() -> (Engine, CaseId, CaseId, usize) {
        setup_mode(SqMode::ExactFused)
    }

    fn setup_mode(mode: SqMode) -> (Engine, CaseId, CaseId, usize) {
        let mut kb = Kb::new();
        kb.load_str(SRC).unwrap();
        let solar = kb.case_by_name("solar").unwrap();
        let atom = kb.case_by_name("atom").unwrap();
        let mut e = Engine::new(kb, EngineConfig { slack: 2, sq_mode: mode, ..Default::default() });
        let sq = e.add_standing_query(atom, 1);
        (e, solar, atom, sq)
    }

    #[test]
    fn standing_query_infers_and_retracts_with_provenance() {
        let (mut e, solar, _atom, sq) = setup();
        assert_eq!(e.standing(sq).top()[0].0, solar);
        let infs = e.inferences(sq);
        let causal = "(cause (and (attracts nucleus electron) (greater (mass nucleus) (mass electron))) (revolve-around electron nucleus))";
        assert!(infs.iter().any(|s| s == causal), "{infs:?}");
        assert!(e.explain(sq, causal).unwrap().contains("by analogy with case solar"));
        // Retract the causal fact in the base: the inference goes OUT via the TMS.
        let cause_fact = *e.kb.case(solar).facts.last().unwrap();
        e.drain_events();
        e.remove_fact(solar, cause_fact);
        assert!(!e.inferences(sq).iter().any(|s| s == causal));
        let ev = e.drain_events();
        assert!(ev.iter().any(|x| matches!(x, Event::InferenceOut { text, .. } if text == causal)), "{ev:?}");
        // Re-adding restores it.
        e.add_fact(solar, cause_fact);
        assert!(e.inferences(sq).iter().any(|s| s == causal));
    }

    #[test]
    fn incremental_matches_scratch_after_new_case() {
        let (mut e, _solar, _atom, sq) = setup();
        let mut kb_facts = Vec::new();
        for s in ["(attracts star moon)", "(greater (mass star) (mass moon))", "(revolve-around moon star)", "(cause (attracts star moon) (revolve-around moon star))"] {
            kb_facts.push(e.kb.parse_expr(s).unwrap());
        }
        e.add_case("star-moon", kb_facts);
        let inc: Vec<CaseId> = e.standing(sq).top().iter().map(|x| x.0).collect();
        let (scratch, _) = e.scratch(sq);
        let sc: Vec<CaseId> = scratch.iter().map(|x| x.0).collect();
        assert_eq!(inc, sc);
    }

    #[test]
    fn pipeline_mode_infers_and_matches_scratch() {
        let (mut e, solar, _atom, sq) = setup_mode(SqMode::Pipeline { mac_k: 2 });
        assert_eq!(e.standing(sq).top()[0].0, solar);
        assert!(!e.inferences(sq).is_empty());
        let cause_fact = *e.kb.case(solar).facts.last().unwrap();
        e.remove_fact(solar, cause_fact);
        let inc = e.standing(sq).top().to_vec();
        let (sc, infs) = e.scratch(sq);
        assert_eq!(inc.iter().map(|x| x.0).collect::<Vec<_>>(), sc.iter().map(|x| x.0).collect::<Vec<_>>());
        assert_eq!(e.inferences(sq), infs);
    }

    #[test]
    fn significance_separates_real_from_chance_analogues() {
        // 40 random 3-chain cases over 12 predicates, one deep causal case,
        // and two queries: an isomorph of the deep case and a fresh random chain.
        let mut src = String::from("(defpredicate cause :arity 2 :kind relation)\n");
        let mut rng = mars_hv::Rng::new(7);
        let chain = |rng: &mut mars_hv::Rng, e: &str| {
            (0..3).map(|i| format!("(p{} {e}{i} {e}{})", rng.below(12), i + 1)).collect::<Vec<_>>().join(" ")
        };
        for c in 0..40 {
            src.push_str(&format!("(defcase r{c} {})\n", chain(&mut rng, &format!("r{c}e"))));
        }
        let deep = |e: &str| format!("(p1 {e}0 {e}1) (p2 {e}1 {e}2) (p3 {e}2 {e}3) (cause (p1 {e}0 {e}1) (p2 {e}1 {e}2)) (cause (p2 {e}1 {e}2) (p3 {e}2 {e}3)) (cause (cause (p1 {e}0 {e}1) (p2 {e}1 {e}2)) (p3 {e}2 {e}3))");
        src.push_str(&format!("(defcase deep {})\n(defquery q-real {})\n(defquery q-chance {})\n", deep("d"), deep("x"), chain(&mut rng, "y")));
        let mut kb = Kb::new();
        kb.load_str(&src).unwrap();
        let (qr, qc) = (kb.case_by_name("q-real").unwrap(), kb.case_by_name("q-chance").unwrap());
        let deep_id = kb.case_by_name("deep").unwrap();
        let mut e = Engine::new(kb, EngineConfig::default());
        e.remove_case(qr);
        e.remove_case(qc);
        let (hits, zr) = e.query_significance(qr, 3);
        assert_eq!(hits[0].0, deep_id);
        let (_, zc) = e.query_significance(qc, 3);
        let (zr, zc) = (zr.unwrap(), zc.unwrap());
        assert!(zr >= SIGNIFICANT_Z, "real analogue z = {zr}");
        assert!(zc < SIGNIFICANT_Z && zc < zr, "chance {zc} vs real {zr}");
    }

}
