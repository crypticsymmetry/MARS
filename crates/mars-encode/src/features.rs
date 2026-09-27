//! Feature maps: case → weighted multiset of hashed features per channel.
//!
//! Channels (see docs/DESIGN.md §6.4):
//! * **C0 Surface**: entity names, attribute predicates.
//! * **C1 Content**: functor multiset (structural functors only).
//! * **C2 Relational n-grams**: parent→child functor links, entity-mediated
//!   co-argument links, expression-mediated co-argument links. Entity-anonymous.
//! * **C3 WL**: Weisfeiler–Lehman refined labels of the anonymous incidence graph.
//!
//! Taxonomy grading is implemented as *multi-resolution features*: each
//! structural feature is also emitted with predicates replaced by their
//! primary parents, at weight `taxonomy_alpha`. In the SimHash view this is
//! exactly what binding taxonomy-bundled predicate vectors would achieve.

use mars_hv::rng::{hash_str, hash_words, mix64};
use mars_rel::{CaseId, ExprId, Kb, PredKind, Sym, Term};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

pub const N_CHANNELS: usize = 4;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Channel {
    Surface = 0,
    Content = 1,
    Relational = 2,
    Wl = 3,
}

impl Channel {
    pub const ALL: [Channel; N_CHANNELS] = [Channel::Surface, Channel::Content, Channel::Relational, Channel::Wl];
    pub fn name(self) -> &'static str {
        match self {
            Channel::Surface => "C0-surface",
            Channel::Content => "C1-content",
            Channel::Relational => "C2-relational",
            Channel::Wl => "C3-wl",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// WL refinement depth h (labels for t = 1..=h are emitted).
    pub wl_depth: usize,
    /// Weight of WL labels at iteration t (index t-1); last value repeats.
    pub wl_weights: Vec<f32>,
    /// Systematicity bias: pc-feature weight = 1 + beta * (order(parent) - 1).
    pub beta: f32,
    /// Weight of parent-level (taxonomy) copies of structural features; 0 disables.
    pub taxonomy_alpha: f32,
    /// Max occurrences per entity used for co-argument pairs.
    pub co_kmax: usize,
    /// Emit entity-mediated co-argument features.
    pub co_entity: bool,
    /// Emit expression-mediated co-argument features (shared sub-expressions).
    pub co_expr: bool,
    /// Emit parent→child features.
    pub parent_child: bool,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        FeatureConfig {
            wl_depth: 2,
            wl_weights: vec![1.0, 0.7, 0.5],
            beta: 0.5,
            taxonomy_alpha: 0.5,
            co_kmax: 16,
            co_entity: true,
            co_expr: true,
            parent_child: true,
        }
    }
}

/// Sorted, deduplicated `(feature hash, weight)` list.
pub type SparseVec = Vec<(u64, f32)>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Features {
    pub channels: [SparseVec; N_CHANNELS],
}

impl Features {
    pub fn channel(&self, c: Channel) -> &SparseVec {
        &self.channels[c as usize]
    }
    pub fn len(&self) -> usize {
        self.channels.iter().map(Vec::len).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Accumulates features for one channel, merging duplicates.
#[derive(Default)]
struct Bag(FxHashMap<u64, f32>);

impl Bag {
    #[inline]
    fn add(&mut self, h: u64, w: f32) {
        if w > 0.0 {
            *self.0.entry(h).or_insert(0.0) += w;
        }
    }
    fn finish(self) -> SparseVec {
        let mut v: Vec<(u64, f32)> = self.0.into_iter().collect();
        v.sort_unstable_by_key(|x| x.0);
        v
    }
}

// Tags keep feature families in disjoint hash spaces.
const T_ENT: u64 = 0x01;
const T_ATTR: u64 = 0x02;
const T_FN: u64 = 0x03;
const T_PC: u64 = 0x04;
const T_CO: u64 = 0x05;
const T_COE: u64 = 0x06;
const T_WL0: u64 = 0x07;
const T_WL: u64 = 0x08;
const T_WL_ENT: u64 = 0x09;
const T_EDGE_ARG: u64 = 0x0A;
const T_EDGE_PAR: u64 = 0x0B;
const T_EDGE_IN: u64 = 0x0C;

/// Stateless feature extractor bound to a knowledge base's vocabulary.
pub struct FeatureExtractor<'a> {
    kb: &'a Kb,
    cfg: FeatureConfig,
    /// Stable (name-based) hash per symbol, indexed by `Sym.0`.
    sym_hash: Vec<u64>,
}

impl<'a> FeatureExtractor<'a> {
    pub fn new(kb: &'a Kb, cfg: FeatureConfig) -> Self {
        let sym_hash = (0..kb.interner.len()).map(|i| hash_str(kb.interner.name(Sym(i as u32)))).collect();
        FeatureExtractor { kb, cfg, sym_hash }
    }

    pub fn config(&self) -> &FeatureConfig {
        &self.cfg
    }

    #[inline]
    fn h(&self, s: Sym) -> u64 {
        self.sym_hash.get(s.0 as usize).copied().unwrap_or_else(|| hash_str(self.kb.name(s)))
    }

    /// Predicate hash at taxonomy level 0 (itself) or 1 (primary parent, or itself if none).
    #[inline]
    fn ph(&self, p: Sym, level: usize) -> u64 {
        if level == 0 {
            self.h(p)
        } else {
            self.h(self.kb.vocab.primary_parent(p).unwrap_or(p))
        }
    }

    #[inline]
    fn is_attr(&self, e: ExprId) -> bool {
        self.kb.vocab.kind(self.kb.expr(e).functor) == PredKind::Attribute
    }

    #[inline]
    fn pos(&self, p: Sym, i: usize) -> u64 {
        if self.kb.vocab.is_commutative(p) {
            0
        } else {
            i as u64 + 1
        }
    }

    fn levels(&self) -> Vec<(usize, f32)> {
        if self.cfg.taxonomy_alpha > 0.0 {
            vec![(0, 1.0), (1, self.cfg.taxonomy_alpha)]
        } else {
            vec![(0, 1.0)]
        }
    }

    pub fn extract(&self, case: CaseId) -> Features {
        let kb = self.kb;
        let exprs = kb.case_exprs(case);
        let structural: Vec<ExprId> = exprs.iter().copied().filter(|&e| !self.is_attr(e)).collect();
        let levels = self.levels();

        // ---------------- C0 surface
        let mut c0 = Bag::default();
        for ent in kb.case_entities(case) {
            c0.add(hash_words(&[T_ENT, self.h(ent)]), 1.0);
        }
        for &e in &exprs {
            if self.is_attr(e) {
                c0.add(hash_words(&[T_ATTR, self.h(kb.expr(e).functor)]), 1.0);
            }
        }

        // ---------------- C1 content
        let mut c1 = Bag::default();
        for &e in &structural {
            let p = kb.expr(e).functor;
            for &(lvl, w) in &levels {
                c1.add(hash_words(&[T_FN, lvl as u64, self.ph(p, lvl)]), w);
            }
        }

        // ---------------- C2 relational n-grams
        let mut c2 = Bag::default();
        // Occurrence lists: entity -> [(functor, pos)], expr -> [(parent functor, pos)].
        let mut ent_occ: FxHashMap<Sym, Vec<(Sym, u64)>> = FxHashMap::default();
        let mut expr_occ: FxHashMap<ExprId, Vec<(Sym, u64)>> = FxHashMap::default();
        for &e in &structural {
            let ex = kb.expr(e);
            let p = ex.functor;
            let w_sys = 1.0 + self.cfg.beta * (kb.order(e) as f32 - 1.0);
            for (i, a) in ex.args.iter().enumerate() {
                let pos = self.pos(p, i);
                match *a {
                    Term::Ent(s) => ent_occ.entry(s).or_default().push((p, pos)),
                    Term::Expr(c) => {
                        if self.is_attr(c) {
                            continue;
                        }
                        expr_occ.entry(c).or_default().push((p, pos));
                        if self.cfg.parent_child {
                            let q = kb.expr(c).functor;
                            for &(lvl, w) in &levels {
                                c2.add(hash_words(&[T_PC, lvl as u64, self.ph(p, lvl), pos, self.ph(q, lvl)]), w * w_sys);
                            }
                        }
                    }
                }
            }
        }
        if self.cfg.co_entity {
            for occ in ent_occ.values_mut() {
                self.co_pairs(occ, T_CO, &levels, &mut c2);
            }
        }
        if self.cfg.co_expr {
            for occ in expr_occ.values_mut() {
                self.co_pairs(occ, T_COE, &levels, &mut c2);
            }
        }

        // ---------------- C3 WL
        let mut c3 = Bag::default();
        if self.cfg.wl_depth > 0 {
            for &(lvl, w) in &levels {
                self.wl(&structural, lvl, w, &mut c3);
            }
        }

        Features { channels: [c0.finish(), c1.finish(), c2.finish(), c3.finish()] }
    }

    /// Unordered pairs of occurrences sharing an argument.
    fn co_pairs(&self, occ: &mut Vec<(Sym, u64)>, tag: u64, levels: &[(usize, f32)], bag: &mut Bag) {
        if occ.len() < 2 {
            return;
        }
        if occ.len() > self.cfg.co_kmax {
            // Deterministic subsample: keep the k smallest by hash.
            occ.sort_unstable_by_key(|&(p, pos)| mix64(self.h(p) ^ pos));
            occ.truncate(self.cfg.co_kmax);
        }
        for &(lvl, w) in levels {
            for i in 0..occ.len() {
                for j in (i + 1)..occ.len() {
                    let a = hash_words(&[self.ph(occ[i].0, lvl), occ[i].1]);
                    let b = hash_words(&[self.ph(occ[j].0, lvl), occ[j].1]);
                    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                    bag.add(hash_words(&[tag, lvl as u64, lo, hi]), w);
                }
            }
        }
    }

    /// WL refinement over the bipartite incidence graph of structural
    /// expressions and (anonymous) entities.
    fn wl(&self, structural: &[ExprId], lvl: usize, w_level: f32, bag: &mut Bag) {
        let kb = self.kb;
        let n_e = structural.len();
        let idx: FxHashMap<ExprId, usize> = structural.iter().enumerate().map(|(i, &e)| (e, i)).collect();
        let mut ents: FxHashMap<Sym, usize> = FxHashMap::default();
        // Adjacency: node -> [(edge label, neighbour node)].
        let mut adj: Vec<Vec<(u64, usize)>> = vec![Vec::new(); n_e];
        let mut labels: Vec<u64> = Vec::with_capacity(n_e);
        for &e in structural {
            labels.push(hash_words(&[T_WL0, self.ph(kb.expr(e).functor, lvl)]));
        }
        for (ei, &e) in structural.iter().enumerate() {
            let ex = kb.expr(e);
            for (i, a) in ex.args.iter().enumerate() {
                let pos = self.pos(ex.functor, i);
                let child = match *a {
                    Term::Expr(c) => match idx.get(&c) {
                        Some(&ci) => ci,
                        None => continue, // attribute
                    },
                    Term::Ent(s) => {
                        let next = n_e + ents.len();
                        let id = *ents.entry(s).or_insert(next);
                        if id == adj.len() {
                            adj.push(Vec::new());
                            labels.push(T_WL_ENT);
                        }
                        id
                    }
                };
                adj[ei].push((hash_words(&[T_EDGE_ARG, pos]), child));
                let back = if child < n_e { T_EDGE_PAR } else { T_EDGE_IN };
                adj[child].push((hash_words(&[back, pos]), ei));
            }
        }
        let mut buf: Vec<u64> = Vec::new();
        for t in 1..=self.cfg.wl_depth {
            let mut next = Vec::with_capacity(labels.len());
            for (v, nbrs) in adj.iter().enumerate() {
                buf.clear();
                buf.extend(nbrs.iter().map(|&(edge, u)| hash_words(&[edge, labels[u]])));
                buf.sort_unstable();
                buf.push(labels[v]);
                next.push(hash_words(&buf));
            }
            labels = next;
            let wt = *self.cfg.wl_weights.get(t - 1).or(self.cfg.wl_weights.last()).unwrap_or(&1.0);
            for &l in &labels {
                bag.add(hash_words(&[T_WL, lvl as u64, t as u64, l]), wt * w_level);
            }
        }
    }
}

/// Plain MAC content vector (Forbus, Gentner & Law 1995): counts of every
/// functor occurrence, including attributes. Baseline B4.
pub fn mac_content_vector(kb: &Kb, case: CaseId) -> SparseVec {
    let mut bag = Bag::default();
    for e in kb.case_exprs(case) {
        bag.add(hash_str(kb.name(kb.expr(e).functor)), 1.0);
    }
    bag.finish()
}

/// Bag of tokens (entity names and functor names). Lexical baseline B2.
pub fn lexical_tokens(kb: &Kb, case: CaseId) -> SparseVec {
    let mut bag = Bag::default();
    for e in kb.case_exprs(case) {
        let ex = kb.expr(e);
        bag.add(hash_str(kb.name(ex.functor)), 1.0);
        for a in &ex.args {
            if let Term::Ent(s) = *a {
                bag.add(hash_str(kb.name(s)), 1.0);
            }
        }
    }
    bag.finish()
}
