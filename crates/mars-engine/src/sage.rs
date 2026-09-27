//! SAGE-style analogical generalization (Kuehne et al. 2000; McLure et al.
//! 2015), built on the MARS mapper and fingerprints (DESIGN §11).
//!
//! A *generalization* is a schema: facts over generalized entities, each
//! with a count of assimilated members that contain it (probability =
//! count / members). New cases are assimilated into the best-matching
//! generalization when the normalized structural score ≥ `assimilate`;
//! otherwise they become outliers, and an outlier that later matches a new
//! case seeds a new generalization. Low-probability facts wear away. Each
//! generalization is materialized as a `Schema` case whose facts are those
//! with probability ≥ `view`, so it can be fingerprinted, retrieved and
//! mapped like any other case.

use mars_encode::{FeatureConfig, FeatureExtractor, FeatureStats, Profile, Sketcher, N_CHANNELS};
use mars_map::{MapConfig, Mapper};
use mars_rel::{CaseId, CaseKind, ExprId, Kb, PredKind, Sym, Term};
use rustc_hash::FxHashMap;

#[derive(Clone, Debug)]
pub struct SageConfig {
    /// Minimum normalized structural score to assimilate.
    pub assimilate: f64,
    /// Facts with probability below this wear away (once `min_members` reached).
    pub drop_below: f64,
    pub min_members: usize,
    /// Probability threshold for facts in the materialized schema case.
    pub view: f64,
    /// Fingerprint prefilter size before FAC.
    pub prefilter: usize,
    pub map: MapConfig,
    /// Score assimilation into an established generalization (≥ 2 members)
    /// by *schema coverage* `S(g→x) / S(g→g)` — how much of the schema the
    /// new case contains — instead of the symmetric normalization, which
    /// penalizes a case for its own distractors and perturbations.
    pub coverage: bool,
    /// Prefix for schema case and entity names (several pools may share a KB).
    pub namespace: String,
    /// Also require the best match to be *significant*: its fused score's z
    /// against the fused scores of fingerprint ranks `null_k/2..null_k` of
    /// the pool (local null, E16) must reach this. Skipped while the pool
    /// has fewer than 16 items.
    pub min_z: Option<f64>,
    pub null_k: usize,
}

impl Default for SageConfig {
    fn default() -> Self {
        SageConfig { assimilate: 0.5, drop_below: 0.2, min_members: 5, view: 0.5, prefilter: 8, map: MapConfig::default(), coverage: false, namespace: "schema".into(), min_z: None, null_k: 64 }
    }
}

#[derive(Clone, Debug)]
pub struct Generalization {
    /// (fact in schema namespace, count of members containing it)
    pub facts: Vec<(ExprId, u32)>,
    pub members: Vec<CaseId>,
    /// Materialized schema case.
    pub case: CaseId,
    /// Stable id (names are derived from it; indices change on merges).
    pub id: usize,
    /// Labeled near-misses: cases that resemble the schema but are *not*
    /// instances of it (E20).
    pub near_misses: Vec<CaseId>,
    next_entity: usize,
}

/// A near-miss-trained test for membership in a generalization (E20): each
/// schema fact weighted by how much better it separates members from
/// near-misses, P(matched | member) − P(matched | near-miss).
#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub schema: CaseId,
    /// (schema fact, emphasis ≥ 0)
    pub weights: Vec<(ExprId, f64)>,
    /// Midpoint between the members' and the near-misses' mean scores.
    pub threshold: f64,
}

impl Diagnostic {
    /// Train on labeled positives and near-misses of `schema`.
    pub fn train(kb: &Kb, map: &MapConfig, schema: CaseId, positives: &[CaseId], near_misses: &[CaseId]) -> Diagnostic {
        use std::collections::HashSet;
        let pm: Vec<HashSet<ExprId>> = positives.iter().map(|&x| matched_facts(kb, map, schema, x)).collect();
        let nm: Vec<HashSet<ExprId>> = near_misses.iter().map(|&x| matched_facts(kb, map, schema, x)).collect();
        let frac = |ms: &[HashSet<ExprId>], f: &ExprId| ms.iter().filter(|m| m.contains(f)).count() as f64 / ms.len().max(1) as f64;
        let weights: Vec<(ExprId, f64)> = kb.case(schema).facts.iter().map(|f| (*f, (frac(&pm, f) - frac(&nm, f)).max(0.0))).collect();
        let total: f64 = weights.iter().map(|w| w.1).sum();
        let score = |m: &HashSet<ExprId>| if total == 0.0 { 0.0 } else { weights.iter().filter(|w| m.contains(&w.0)).map(|w| w.1).sum::<f64>() / total };
        let mean = |v: &[HashSet<ExprId>]| v.iter().map(score).sum::<f64>() / v.len().max(1) as f64;
        let threshold = (mean(&pm) + mean(&nm)) / 2.0;
        Diagnostic { schema, weights, threshold }
    }

    /// Weighted fraction of emphasized schema facts that `x` matches.
    pub fn score(&self, kb: &Kb, map: &MapConfig, x: CaseId) -> f64 {
        let matched = matched_facts(kb, map, self.schema, x);
        let total: f64 = self.weights.iter().map(|w| w.1).sum();
        if total == 0.0 {
            return 0.0;
        }
        self.weights.iter().filter(|w| matched.contains(&w.0)).map(|w| w.1).sum::<f64>() / total
    }

    pub fn accepts(&self, kb: &Kb, map: &MapConfig, x: CaseId) -> bool {
        self.score(kb, map, x) >= self.threshold
    }
}

/// Facts of `base` that the best mapping base → target places in correspondence.
fn matched_facts(kb: &Kb, map: &MapConfig, base: CaseId, target: CaseId) -> std::collections::HashSet<ExprId> {
    let facts: std::collections::HashSet<ExprId> = kb.case(base).facts.iter().copied().collect();
    Mapper::new(kb, map.clone())
        .best(base, target)
        .map(|m| m.correspondences.iter().filter_map(|(b, _)| if let Term::Expr(e) = b { facts.contains(e).then_some(*e) } else { None }).collect())
        .unwrap_or_default()
}

impl Generalization {
    pub fn probability(&self, f: ExprId) -> Option<f64> {
        self.facts.iter().find(|x| x.0 == f).map(|x| x.1 as f64 / self.members.len() as f64)
    }
}

/// Rewrite an expression, mapping entities through `f` (hash-consed).
pub fn rewrite(kb: &mut Kb, e: ExprId, f: &mut dyn FnMut(&mut Kb, Sym) -> Sym) -> ExprId {
    let ex = kb.expr(e).clone();
    let mut args = Vec::with_capacity(ex.args.len());
    for a in ex.args {
        args.push(match a {
            Term::Ent(s) => Term::Ent(f(kb, s)),
            Term::Expr(c) => Term::Expr(rewrite(kb, c, f)),
        });
    }
    kb.intern_expr(ex.functor, args)
}

pub struct Sage {
    pub cfg: SageConfig,
    pub gens: Vec<Generalization>,
    pub outliers: Vec<CaseId>,
    stats: FeatureStats,
    sketcher: Sketcher,
    profile: Profile,
    fcfg: FeatureConfig,
    fp_cache: FxHashMap<(CaseId, u64), Vec<u64>>,
    self_cache: FxHashMap<(CaseId, u64), f32>,
    next_id: usize,
    /// Local-null z of the last `best_match` (when `min_z` is set).
    pub last_z: Option<f64>,
}

impl Sage {
    pub fn new(cfg: SageConfig, stats: FeatureStats, sketcher: Sketcher, fcfg: FeatureConfig) -> Self {
        Sage { cfg, gens: Vec::new(), outliers: Vec::new(), stats, sketcher, profile: Profile::analogy(), fcfg, fp_cache: FxHashMap::default(), self_cache: FxHashMap::default(), next_id: 0, last_z: None }
    }

    fn fp(&mut self, kb: &Kb, c: CaseId) -> Vec<u64> {
        let key = (c, kb.case(c).version);
        if let Some(v) = self.fp_cache.get(&key) {
            return v.clone();
        }
        let fx = FeatureExtractor::new_light(kb, self.fcfg.clone());
        let mut f = fx.extract(c);
        self.stats.apply(&mut f, &[true; N_CHANNELS]);
        let v = self.sketcher.sketch(&f).into_words();
        self.fp_cache.insert(key, v.clone());
        v
    }

    fn self_score(&mut self, kb: &Kb, c: CaseId) -> f32 {
        let key = (c, kb.case(c).version);
        if let Some(&s) = self.self_cache.get(&key) {
            return s;
        }
        let s = Mapper::new(kb, self.cfg.map.clone()).score(c, c);
        self.self_cache.insert(key, s);
        s
    }

    /// Normalized structural score of mapping `base` onto `target`.
    pub fn fac(&mut self, kb: &Kb, base: CaseId, target: CaseId) -> f64 {
        let raw = Mapper::new(kb, self.cfg.map.clone()).score(base, target) as f64;
        let (a, b) = (self.self_score(kb, base) as f64, self.self_score(kb, target) as f64);
        if raw == 0.0 || a == 0.0 || b == 0.0 {
            0.0
        } else {
            (raw / (a * b).sqrt()).min(1.0)
        }
    }

    /// Fraction of `base`'s structure found in `target`: `S(b→t) / S(b→b)`.
    pub fn coverage(&mut self, kb: &Kb, base: CaseId, target: CaseId) -> f64 {
        let raw = Mapper::new(kb, self.cfg.map.clone()).score(base, target) as f64;
        let a = self.self_score(kb, base) as f64;
        if raw == 0.0 || a == 0.0 {
            0.0
        } else {
            (raw / a).min(1.0)
        }
    }

    /// Best pool item (generalization case or outlier) for `x` by fused score.
    /// Returns (is_generalization, index, fused, fac).
    pub fn best_match(&mut self, kb: &Kb, x: CaseId) -> Option<(bool, usize, f64, f64)> {
        let fx = self.fp(kb, x);
        let layout = self.sketcher.layout.clone();
        let mut cands: Vec<(bool, usize, CaseId, f64)> = Vec::new();
        for i in 0..self.gens.len() {
            let c = self.gens[i].case;
            let fc = self.fp(kb, c);
            cands.push((true, i, c, self.profile.score(&mars_encode::channel_sims(&layout, &fx, &fc))));
        }
        for i in 0..self.outliers.len() {
            let c = self.outliers[i];
            let fc = self.fp(kb, c);
            cands.push((false, i, c, self.profile.score(&mars_encode::channel_sims(&layout, &fx, &fc))));
        }
        cands.sort_by(|a, b| b.3.total_cmp(&a.3));
        self.last_z = None;
        let tail: Vec<(CaseId, f64)> = if self.cfg.min_z.is_some() && cands.len() >= 16 {
            let k = self.cfg.null_k.min(cands.len());
            cands[k / 2..k].iter().map(|c| (c.2, c.3)).collect()
        } else {
            Vec::new()
        };
        let null: Vec<f64> = tail.into_iter().map(|(c, fp)| 0.5 * self.fac(kb, c, x) + 0.5 * fp).collect();
        cands.truncate(self.cfg.prefilter);
        let mut best: Option<(bool, usize, f64, f64)> = None;
        for (g, i, c, fp) in cands {
            let fac = if g && self.cfg.coverage && self.gens[i].members.len() >= 2 { self.coverage(kb, c, x) } else { self.fac(kb, c, x) };
            let fused = 0.5 * fac + 0.5 * fp;
            if best.map(|b| fused > b.2).unwrap_or(true) {
                best = Some((g, i, fused, fac));
            }
        }
        if let (Some(b), false) = (best, null.is_empty()) {
            let m = null.iter().sum::<f64>() / null.len() as f64;
            let sd = (null.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (null.len() - 1) as f64).sqrt().max(1e-9);
            self.last_z = Some((b.2 - m) / sd);
        }
        best
    }

    fn significant(&self) -> bool {
        match (self.cfg.min_z, self.last_z) {
            (Some(z0), Some(z)) => z >= z0,
            _ => true,
        }
    }

    /// Add a case to the pool: assimilate, seed a generalization, or keep as outlier.
    pub fn add(&mut self, kb: &mut Kb, x: CaseId) {
        let m = self.best_match(kb, x);
        let sig = self.significant();
        match m {
            Some((true, gi, _, fac)) if fac >= self.cfg.assimilate && sig => self.assimilate(kb, gi, x),
            Some((false, oi, _, fac)) if fac >= self.cfg.assimilate && sig => {
                let o = self.outliers.swap_remove(oi);
                let gi = self.seed(kb, o);
                self.assimilate(kb, gi, x);
            }
            _ => self.outliers.push(x),
        }
    }

    fn structural_facts(kb: &Kb, c: CaseId) -> Vec<ExprId> {
        kb.case(c).facts.iter().copied().filter(|&f| kb.vocab.kind(kb.expr(f).functor) != PredKind::Attribute).collect()
    }

    /// Start a generalization from a single case (entities renamed into the schema namespace).
    fn seed(&mut self, kb: &mut Kb, a: CaseId) -> usize {
        let gi = self.gens.len();
        let id = self.next_id;
        self.next_id += 1;
        let ns = self.cfg.namespace.clone();
        let mut ents: FxHashMap<Sym, Sym> = FxHashMap::default();
        let mut next = 0usize;
        let mut facts = Vec::new();
        for f in Self::structural_facts(kb, a) {
            let g = rewrite(kb, f, &mut |kb: &mut Kb, s: Sym| {
                *ents.entry(s).or_insert_with(|| {
                    next += 1;
                    kb.sym(&format!("?{ns}{id}e{next}"))
                })
            });
            facts.push((g, 1u32));
        }
        let case = kb.add_case(&format!("{ns}{id}"), CaseKind::Schema, facts.iter().map(|x| x.0));
        self.gens.push(Generalization { facts, members: vec![a], case, id, near_misses: Vec::new(), next_entity: next });
        gi
    }

    fn assimilate(&mut self, kb: &mut Kb, gi: usize, x: CaseId) {
        let gcase = self.gens[gi].case;
        let m = Mapper::new(kb, self.cfg.map.clone()).best(gcase, x);
        let Some(m) = m else {
            self.outliers.push(x);
            return;
        };
        let matched_base: FxHashMap<Term, Term> = m.correspondences.iter().copied().collect();
        let x_to_g: FxHashMap<Sym, Sym> = m.entity_map().into_iter().map(|(g, xe)| (xe, g)).collect();
        let matched_target: std::collections::HashSet<Term> = m.correspondences.iter().map(|x| x.1).collect();
        // Count schema facts present (matched) in x.
        for (f, cnt) in self.gens[gi].facts.iter_mut() {
            if matched_base.contains_key(&Term::Expr(*f)) {
                *cnt += 1;
            }
        }
        // Add x's unmatched facts, rewritten into the schema namespace.
        let mut next = self.gens[gi].next_entity;
        let (ns, id) = (self.cfg.namespace.clone(), self.gens[gi].id);
        let mut fresh: FxHashMap<Sym, Sym> = FxHashMap::default();
        for f in Self::structural_facts(kb, x) {
            if matched_target.contains(&Term::Expr(f)) {
                continue;
            }
            let g = rewrite(kb, f, &mut |kb: &mut Kb, s: Sym| {
                if let Some(&g) = x_to_g.get(&s) {
                    return g;
                }
                *fresh.entry(s).or_insert_with(|| {
                    next += 1;
                    kb.sym(&format!("?{ns}{id}e{next}"))
                })
            });
            match self.gens[gi].facts.iter_mut().find(|y| y.0 == g) {
                Some(y) => y.1 += 1,
                None => self.gens[gi].facts.push((g, 1)),
            }
        }
        self.gens[gi].next_entity = next;
        self.gens[gi].members.push(x);
        let n = self.gens[gi].members.len() as f64;
        if self.gens[gi].members.len() >= self.cfg.min_members {
            let drop = self.cfg.drop_below;
            self.gens[gi].facts.retain(|&(_, c)| c as f64 / n >= drop);
        }
        self.materialize(kb, gi);
    }

    /// Idle-time consolidation ("sleep"): re-offer outliers to the current
    /// generalizations, then merge generalizations that map onto each other.
    /// Returns (outliers assimilated, generalizations merged).
    pub fn consolidate(&mut self, kb: &mut Kb, merge_threshold: f64) -> (usize, usize) {
        // 1. Outliers seen before a matching generalization existed.
        let outliers = std::mem::take(&mut self.outliers);
        let mut absorbed = 0;
        for o in outliers {
            let before = self.outliers.len();
            let gens_before: usize = self.gens.iter().map(|g| g.members.len()).sum();
            self.add(kb, o);
            let gens_after: usize = self.gens.iter().map(|g| g.members.len()).sum();
            if self.outliers.len() == before && gens_after > gens_before {
                absorbed += 1;
            }
        }
        // 2. Merge generalizations, smallest first, into their best match
        //    among the fingerprint-nearest candidates (one pass).
        let mut merged = 0;
        let mut order: Vec<(usize, CaseId)> = self.gens.iter().map(|g| (g.members.len(), g.case)).collect();
        order.sort();
        let layout = self.sketcher.layout.clone();
        for (_, from_case) in order {
            let Some(gi) = self.gens.iter().position(|g| g.case == from_case) else { continue };
            let ffp = self.fp(kb, from_case);
            let mut cands: Vec<(usize, f64)> = Vec::new();
            for hi in 0..self.gens.len() {
                if hi != gi {
                    let hfp = self.fp(kb, self.gens[hi].case);
                    cands.push((hi, self.profile.score(&mars_encode::channel_sims(&layout, &ffp, &hfp))));
                }
            }
            cands.sort_by(|a, b| b.1.total_cmp(&a.1));
            cands.truncate(self.cfg.prefilter);
            let mut best: Option<(usize, f64)> = None;
            for (hi, _) in cands {
                let f = self.fac(kb, self.gens[hi].case, from_case);
                if f >= merge_threshold && best.map(|b| f > b.1).unwrap_or(true) {
                    best = Some((hi, f));
                }
            }
            if let Some((hi, _)) = best {
                self.merge(kb, gi, hi);
                merged += 1;
            }
        }
        (absorbed, merged)
    }

    /// Record `x` as a near-miss of generalization `gi`.
    pub fn add_near_miss(&mut self, gi: usize, x: CaseId) {
        self.gens[gi].near_misses.push(x);
    }

    /// Membership test for generalization `gi` trained on its members and
    /// near-misses (`positives` overrides the member list, e.g. to include
    /// labeled positives that were not assimilated). `None` without near-misses.
    pub fn diagnostic(&self, kb: &Kb, gi: usize, positives: Option<&[CaseId]>) -> Option<Diagnostic> {
        let g = &self.gens[gi];
        if g.near_misses.is_empty() {
            return None;
        }
        Some(Diagnostic::train(kb, &self.cfg.map, g.case, positives.unwrap_or(&g.members), &g.near_misses))
    }

    /// Merge generalization `from` into `into` (then remove `from`).
    fn merge(&mut self, kb: &mut Kb, from: usize, into: usize) {
        let (fcase, icase) = (self.gens[from].case, self.gens[into].case);
        let Some(m) = Mapper::new(kb, self.cfg.map.clone()).best(icase, fcase) else { return };
        let from_to_into: FxHashMap<Term, Term> = m.correspondences.iter().map(|&(i, f)| (f, i)).collect();
        let ent_map: FxHashMap<Sym, Sym> = m.entity_map().into_iter().map(|(i, f)| (f, i)).collect();
        let from_facts = self.gens[from].facts.clone();
        let mut next = self.gens[into].next_entity;
        let (ns, id) = (self.cfg.namespace.clone(), self.gens[into].id);
        let mut fresh: FxHashMap<Sym, Sym> = FxHashMap::default();
        for (f, cnt) in from_facts {
            let target = match from_to_into.get(&Term::Expr(f)) {
                Some(Term::Expr(x)) => *x,
                _ => rewrite(kb, f, &mut |kb: &mut Kb, s: Sym| {
                    if let Some(&x) = ent_map.get(&s) {
                        return x;
                    }
                    *fresh.entry(s).or_insert_with(|| {
                        next += 1;
                        kb.sym(&format!("?{ns}{id}e{next}m"))
                    })
                }),
            };
            match self.gens[into].facts.iter_mut().find(|y| y.0 == target) {
                Some(y) => y.1 += cnt,
                None => self.gens[into].facts.push((target, cnt)),
            }
        }
        self.gens[into].next_entity = next;
        let members = std::mem::take(&mut self.gens[from].members);
        self.gens[into].members.extend(members);
        let nms = std::mem::take(&mut self.gens[from].near_misses);
        self.gens[into].near_misses.extend(nms);
        let n = self.gens[into].members.len() as f64;
        if self.gens[into].members.len() >= self.cfg.min_members {
            let drop = self.cfg.drop_below;
            self.gens[into].facts.retain(|&(_, c)| c as f64 / n >= drop);
        }
        self.materialize(kb, into);
        // Retire `from`: empty its schema case and remove it from the pool.
        for f in kb.case(fcase).facts.clone() {
            kb.remove_fact(fcase, f);
        }
        self.gens.swap_remove(from);
    }

    /// Sync the schema case's facts with probability ≥ view.
    fn materialize(&mut self, kb: &mut Kb, gi: usize) {
        let g = &self.gens[gi];
        let n = g.members.len() as f64;
        let view = if g.members.len() < 2 { 0.0 } else { self.cfg.view };
        let want: Vec<ExprId> = g.facts.iter().filter(|&&(_, c)| c as f64 / n >= view).map(|x| x.0).collect();
        let case = g.case;
        let have = kb.case(case).facts.clone();
        for f in &have {
            if !want.contains(f) {
                kb.remove_fact(case, *f);
            }
        }
        for f in want {
            kb.add_fact(case, f);
        }
    }
}

impl<'a> FeatureExtractorLight<'a> for FeatureExtractor<'a> {}

/// Helper trait to build a feature extractor without precomputing symbol
/// hashes (hashes are computed on the fly): cheap for one-off extraction.
pub trait FeatureExtractorLight<'a> {
    fn new_light(kb: &'a Kb, cfg: FeatureConfig) -> FeatureExtractor<'a> {
        FeatureExtractor::with_hashes(kb, cfg, &[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mars_encode::Layout;

    #[test]
    fn three_isomorphs_form_one_generalization() {
        let mut kb = Kb::new();
        kb.load_str(
            r#"(defpredicate cause :arity 2 :kind relation)
               (defcase a (pushes a1 a2) (moves a2 a3) (cause (pushes a1 a2) (moves a2 a3)) (noise-a a3 a1))
               (defcase b (pushes b1 b2) (moves b2 b3) (cause (pushes b1 b2) (moves b2 b3)) (noise-b b1 b2))
               (defcase c (pushes c1 c2) (moves c2 c3) (cause (pushes c1 c2) (moves c2 c3)))"#,
        )
        .unwrap();
        let cases: Vec<CaseId> = ["a", "b", "c"].iter().map(|n| kb.case_by_name(n).unwrap()).collect();
        let feats: Vec<_> = {
            let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
            cases.iter().map(|&c| fx.extract(c)).collect()
        };
        let stats = FeatureStats::fit(feats.iter());
        let mut sage = Sage::new(SageConfig { min_members: 10, ..Default::default() }, stats, Sketcher::new(Layout::default(), 1), FeatureConfig::default());
        for &c in &cases {
            sage.add(&mut kb, c);
        }
        assert_eq!(sage.gens.len(), 1, "outliers: {:?}", sage.outliers);
        let g = &sage.gens[0];
        assert_eq!(g.members.len(), 3);
        // Core facts in all 3 members; each noise fact in 1 of 3.
        let mut probs: Vec<f64> = g.facts.iter().map(|x| x.1 as f64 / 3.0).collect();
        probs.sort_by(|a, b| b.partial_cmp(a).unwrap());
        assert_eq!(&probs[..3], &[1.0, 1.0, 1.0]);
        assert!(probs[3..].iter().all(|&p| p < 0.5), "{probs:?}");
        // Schema view (p ≥ 0.5) contains exactly the core.
        assert_eq!(kb.case(g.case).facts.len(), 3, "{}", kb.render_case(g.case));
    }

    #[test]
    fn near_miss_emphasis_rejects_rewired_structure() {
        // Members share causal structure A→B→C; the near-miss has the same
        // first-order facts but the causal link C→A instead.
        let mut kb = Kb::new();
        kb.load_str(
            r#"(defpredicate cause :arity 2 :kind relation)
               (defcase a (push a1 a2) (move a2 a3) (heat a3 a1) (cause (push a1 a2) (move a2 a3)) (cause (move a2 a3) (heat a3 a1)))
               (defcase b (push b1 b2) (move b2 b3) (heat b3 b1) (cause (push b1 b2) (move b2 b3)) (cause (move b2 b3) (heat b3 b1)))
               (defcase c (push c1 c2) (move c2 c3) (heat c3 c1) (cause (push c1 c2) (move c2 c3)) (cause (move c2 c3) (heat c3 c1)))
               (defcase nm (push n1 n2) (move n2 n3) (heat n3 n1) (cause (heat n3 n1) (push n1 n2)) (cause (move n2 n3) (heat n3 n1)))
               (defcase nm2 (push m1 m2) (move m2 m3) (heat m3 m1) (cause (heat m3 m1) (push m1 m2)) (cause (move m2 m3) (heat m3 m1)))
               (defcase d (push d1 d2) (move d2 d3) (heat d3 d1) (cause (push d1 d2) (move d2 d3)) (cause (move d2 d3) (heat d3 d1)))"#,
        )
        .unwrap();
        let id = |kb: &Kb, n: &str| kb.case_by_name(n).unwrap();
        let cases: Vec<CaseId> = ["a", "b", "c"].iter().map(|n| id(&kb, n)).collect();
        let stats = {
            let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
            FeatureStats::fit(cases.iter().map(|&c| fx.extract(c)).collect::<Vec<_>>().iter())
        };
        let mut sage = Sage::new(SageConfig::default(), stats, Sketcher::new(Layout::default(), 1), FeatureConfig::default());
        for &c in &cases {
            sage.add(&mut kb, c);
        }
        assert_eq!(sage.gens.len(), 1);
        assert!(sage.diagnostic(&kb, 0, None).is_none());
        sage.add_near_miss(0, id(&kb, "nm"));
        let d = sage.diagnostic(&kb, 0, None).unwrap();
        // The emphasized fact is the causal link the near-miss lacks.
        let top = d.weights.iter().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
        assert!(kb.render_expr(top.0).starts_with("(cause (push"), "{}", kb.render_expr(top.0));
        let map = MapConfig::default();
        assert!(d.accepts(&kb, &map, id(&kb, "d")));
        assert!(!d.accepts(&kb, &map, id(&kb, "nm2")));
    }

}
