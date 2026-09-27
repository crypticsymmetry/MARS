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
}

impl Default for SageConfig {
    fn default() -> Self {
        SageConfig { assimilate: 0.5, drop_below: 0.2, min_members: 5, view: 0.5, prefilter: 8, map: MapConfig::default() }
    }
}

#[derive(Clone, Debug)]
pub struct Generalization {
    /// (fact in schema namespace, count of members containing it)
    pub facts: Vec<(ExprId, u32)>,
    pub members: Vec<CaseId>,
    /// Materialized schema case.
    pub case: CaseId,
    next_entity: usize,
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
}

impl Sage {
    pub fn new(cfg: SageConfig, stats: FeatureStats, sketcher: Sketcher, fcfg: FeatureConfig) -> Self {
        Sage { cfg, gens: Vec::new(), outliers: Vec::new(), stats, sketcher, profile: Profile::analogy(), fcfg, fp_cache: FxHashMap::default(), self_cache: FxHashMap::default() }
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
        cands.truncate(self.cfg.prefilter);
        let mut best: Option<(bool, usize, f64, f64)> = None;
        for (g, i, c, fp) in cands {
            let fac = self.fac(kb, c, x);
            let fused = 0.5 * fac + 0.5 * fp;
            if best.map(|b| fused > b.2).unwrap_or(true) {
                best = Some((g, i, fused, fac));
            }
        }
        best
    }

    /// Add a case to the pool: assimilate, seed a generalization, or keep as outlier.
    pub fn add(&mut self, kb: &mut Kb, x: CaseId) {
        match self.best_match(kb, x) {
            Some((true, gi, _, fac)) if fac >= self.cfg.assimilate => self.assimilate(kb, gi, x),
            Some((false, oi, _, fac)) if fac >= self.cfg.assimilate => {
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
        let mut ents: FxHashMap<Sym, Sym> = FxHashMap::default();
        let mut next = 0usize;
        let mut facts = Vec::new();
        for f in Self::structural_facts(kb, a) {
            let g = rewrite(kb, f, &mut |kb: &mut Kb, s: Sym| {
                *ents.entry(s).or_insert_with(|| {
                    next += 1;
                    kb.sym(&format!("?g{gi}e{next}"))
                })
            });
            facts.push((g, 1u32));
        }
        let case = kb.add_case(&format!("schema{gi}"), CaseKind::Schema, facts.iter().map(|x| x.0));
        self.gens.push(Generalization { facts, members: vec![a], case, next_entity: next });
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
                    kb.sym(&format!("?g{gi}e{next}"))
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

    /// Merge generalization `from` into `into` (then remove `from`).
    fn merge(&mut self, kb: &mut Kb, from: usize, into: usize) {
        let (fcase, icase) = (self.gens[from].case, self.gens[into].case);
        let Some(m) = Mapper::new(kb, self.cfg.map.clone()).best(icase, fcase) else { return };
        let from_to_into: FxHashMap<Term, Term> = m.correspondences.iter().map(|&(i, f)| (f, i)).collect();
        let ent_map: FxHashMap<Sym, Sym> = m.entity_map().into_iter().map(|(i, f)| (f, i)).collect();
        let from_facts = self.gens[from].facts.clone();
        let mut next = self.gens[into].next_entity;
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
                        kb.sym(&format!("?g{into}e{next}m"))
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
}
