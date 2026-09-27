//! Greedy SME-class structure mapping.
//!
//! 1. **Local match hypotheses (MHs).** Base × target expressions with the
//!    same structural functor (or a shared taxonomy parent: *minimal
//!    ascension*, reduced score). Argument alignment creates child MHs:
//!    entities always, functions may match non-identically, other
//!    expressions only if they are MHs themselves (parallel connectivity).
//! 2. **Structural consistency** as bitsets: `desc(m)` (m and its
//!    descendants) and `nogood(m)` (MHs violating one-to-one with anything in
//!    `desc(m)`). m is consistent iff `desc ∩ nogood = ∅`.
//! 3. **Kernels**: consistent expression MHs with no consistent parent.
//! 4. **Trickle-down scoring** (systematicity): `score(m) = local(m) + τ Σ score(parents)`.
//! 5. **Greedy merge** of kernels, plus up to K alternatives; an exact
//!    branch-and-bound merge for measuring the greedy gap.
//! 6. **Candidate inferences** (with skolems) and **alignable differences**.

use crate::bitset::BitSet;
use mars_rel::{CaseId, ExprId, Kb, PredKind, Sym, Term};
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

#[derive(Clone, Debug)]
pub struct MapConfig {
    /// Trickle-down factor τ.
    pub trickle: f32,
    /// Allow relations with a shared taxonomy parent to match.
    pub ascension: bool,
    pub ascension_score: f32,
    /// Local score of a non-identical function match (licensed by its parent).
    pub function_mismatch_score: f32,
    /// Include attribute (surface) expressions in matching.
    pub include_attributes: bool,
    /// Number of alternative global mappings to return.
    pub max_mappings: usize,
    /// Safety cap on the number of MHs per pair.
    pub max_mhs: usize,
    /// Optional informativeness weight per structural functor, multiplied
    /// into the local score of relation MHs (e.g. normalized IDF). `None` = 1.
    pub pred_weights: Option<std::sync::Arc<FxHashMap<Sym, f32>>>,
}

impl Default for MapConfig {
    fn default() -> Self {
        MapConfig {
            trickle: 0.8,
            ascension: true,
            ascension_score: 0.5,
            function_mismatch_score: 0.3,
            include_attributes: false,
            max_mappings: 3,
            max_mhs: 50_000,
            pred_weights: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mh {
    pub base: Term,
    pub target: Term,
    pub children: SmallVec<[u32; 4]>,
    pub parents: SmallVec<[u32; 2]>,
    pub local: f32,
    pub score: f32,
    pub consistent: bool,
}

impl Mh {
    pub fn is_entity(&self) -> bool {
        matches!(self.base, Term::Ent(_))
    }
}

#[derive(Clone, Debug)]
pub struct Kernel {
    pub root: u32,
    pub members: BitSet,
    pub nogood: BitSet,
    pub score: f32,
}

/// All match hypotheses and kernels for one base/target pair.
#[derive(Clone, Debug)]
pub struct MatchSet {
    pub base: CaseId,
    pub target: CaseId,
    pub mhs: Vec<Mh>,
    pub kernels: Vec<Kernel>,
    pub truncated: bool,
}

/// Projected term of a candidate inference.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Proj {
    /// An existing target item.
    Target(Term),
    /// A new entity hypothesized in the target, standing for this base entity.
    Skolem(Sym),
    /// A new expression.
    Expr { functor: Sym, args: Vec<Proj> },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Grounding {
    /// Some sub-expression of the base fact is in the mapping (SME-style).
    Structural,
    /// Only entity arguments are mapped.
    EntityOnly,
}

#[derive(Clone, Debug)]
pub struct CandidateInference {
    pub base_fact: ExprId,
    pub projected: Proj,
    /// Fraction of the fact's sub-items (expressions and entities) that are mapped.
    pub support: f32,
    pub grounding: Grounding,
    pub has_skolem: bool,
}

#[derive(Clone, Debug)]
pub struct Mapping {
    pub base: CaseId,
    pub target: CaseId,
    pub members: BitSet,
    pub correspondences: Vec<(Term, Term)>,
    pub score: f32,
    pub inferences: Vec<CandidateInference>,
    /// Same (mapped) arguments, different functors.
    pub alignable_differences: Vec<(ExprId, ExprId)>,
    /// Base facts with no counterpart in the mapping.
    pub unmatched_base_facts: Vec<ExprId>,
}

impl Mapping {
    pub fn entity_map(&self) -> FxHashMap<Sym, Sym> {
        self.correspondences
            .iter()
            .filter_map(|&(b, t)| match (b, t) {
                (Term::Ent(x), Term::Ent(y)) => Some((x, y)),
                _ => None,
            })
            .collect()
    }

    pub fn n_expr_correspondences(&self) -> usize {
        self.correspondences.iter().filter(|(b, _)| matches!(b, Term::Expr(_))).count()
    }
}

pub struct Mapper<'a> {
    kb: &'a Kb,
    pub cfg: MapConfig,
}

struct Builder<'a> {
    kb: &'a Kb,
    cfg: &'a MapConfig,
    mhs: Vec<Mh>,
    index: FxHashMap<(Term, Term), u32>,
    truncated: bool,
}

impl Builder<'_> {
    fn sfun(&self, e: ExprId) -> Sym {
        self.kb.vocab.structural(self.kb.expr(e).functor)
    }

    fn entity_mh(&mut self, b: Sym, t: Sym, created: &mut Vec<(Term, Term)>) -> u32 {
        let key = (Term::Ent(b), Term::Ent(t));
        if let Some(&i) = self.index.get(&key) {
            return i;
        }
        let i = self.mhs.len() as u32;
        self.mhs.push(Mh { base: key.0, target: key.1, children: SmallVec::new(), parents: SmallVec::new(), local: 0.0, score: 0.0, consistent: true });
        self.index.insert(key, i);
        created.push(key);
        i
    }

    /// Try to align argument lists; returns child MH ids or None.
    fn align_args(&mut self, bargs: &[Term], targs: &[Term], created: &mut Vec<(Term, Term)>) -> Option<SmallVec<[u32; 4]>> {
        let mut kids = SmallVec::new();
        for (ba, ta) in bargs.iter().zip(targs) {
            match (*ba, *ta) {
                (Term::Ent(x), Term::Ent(y)) => kids.push(self.entity_mh(x, y, created)),
                (Term::Expr(bx), Term::Expr(tx)) => {
                    if let Some(&i) = self.index.get(&(*ba, *ta)) {
                        kids.push(i);
                        continue;
                    }
                    // Functions may match non-identically when licensed by the parent.
                    let (kb_, kt_) = (self.kb.vocab.kind(self.kb.expr(bx).functor), self.kb.vocab.kind(self.kb.expr(tx).functor));
                    if kb_ == PredKind::Function && kt_ == PredKind::Function && self.kb.expr(bx).args.len() == self.kb.expr(tx).args.len() {
                        let local = if self.sfun(bx) == self.sfun(tx) { 1.0 } else { self.cfg.function_mismatch_score };
                        kids.push(self.expr_mh(bx, tx, local, created)?);
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        Some(kids)
    }

    fn expr_mh(&mut self, b: ExprId, t: ExprId, local: f32, created: &mut Vec<(Term, Term)>) -> Option<u32> {
        let key = (Term::Expr(b), Term::Expr(t));
        if let Some(&i) = self.index.get(&key) {
            return Some(i);
        }
        if self.mhs.len() >= self.cfg.max_mhs {
            self.truncated = true;
            return None;
        }
        let (be, te) = (self.kb.expr(b), self.kb.expr(t));
        if be.args.len() != te.args.len() {
            return None;
        }
        let bargs: Vec<Term> = be.args.to_vec();
        let targs: Vec<Term> = te.args.to_vec();
        let kids = if self.kb.vocab.is_commutative(be.functor) && bargs.len() > 1 && bargs.len() <= 5 {
            // Try target permutations; keep the first full alignment
            // (preferring ones whose children already exist).
            let mut best: Option<SmallVec<[u32; 4]>> = None;
            for perm in permutations(targs.len()) {
                let permuted: Vec<Term> = perm.iter().map(|&i| targs[i]).collect();
                let mark = created.len();
                let n_before = self.mhs.len();
                if let Some(k) = self.align_args(&bargs, &permuted, created) {
                    best = Some(k);
                    break;
                }
                self.rollback(created, mark, n_before);
            }
            best?
        } else {
            self.align_args(&bargs, &targs, created)?
        };
        let i = self.mhs.len() as u32;
        for &k in &kids {
            self.mhs[k as usize].parents.push(i);
        }
        self.mhs.push(Mh { base: key.0, target: key.1, children: kids, parents: SmallVec::new(), local, score: 0.0, consistent: true });
        self.index.insert(key, i);
        created.push(key);
        Some(i)
    }

    fn rollback(&mut self, created: &mut Vec<(Term, Term)>, mark: usize, n_before: usize) {
        for key in created.drain(mark..) {
            self.index.remove(&key);
        }
        // Remove parent back-links pointing at truncated MHs.
        self.mhs.truncate(n_before);
        let n = n_before as u32;
        for m in self.mhs.iter_mut() {
            m.parents.retain(|p| *p < n);
        }
    }

    fn try_pair(&mut self, b: ExprId, t: ExprId, local: f32) {
        let mut created = Vec::new();
        let n_before = self.mhs.len();
        if self.expr_mh(b, t, local, &mut created).is_none() {
            self.rollback(&mut created, 0, n_before);
        }
    }
}

fn permutations(n: usize) -> Vec<Vec<usize>> {
    fn rec(cur: &mut Vec<usize>, used: &mut Vec<bool>, out: &mut Vec<Vec<usize>>) {
        if cur.len() == used.len() {
            out.push(cur.clone());
            return;
        }
        for i in 0..used.len() {
            if !used[i] {
                used[i] = true;
                cur.push(i);
                rec(cur, used, out);
                cur.pop();
                used[i] = false;
            }
        }
    }
    let mut out = Vec::new();
    rec(&mut Vec::new(), &mut vec![false; n], &mut out);
    out
}

impl<'a> Mapper<'a> {
    pub fn new(kb: &'a Kb, cfg: MapConfig) -> Self {
        Mapper { kb, cfg }
    }

    fn is_matchable(&self, e: ExprId) -> bool {
        self.cfg.include_attributes || self.kb.vocab.kind(self.kb.expr(e).functor) != PredKind::Attribute
    }

    /// Build match hypotheses, consistency, scores and kernels.
    pub fn match_set(&self, base: CaseId, target: CaseId) -> MatchSet {
        let kb = self.kb;
        let bexprs: Vec<ExprId> = kb.case_exprs(base).into_iter().filter(|&e| self.is_matchable(e)).collect();
        let texprs: Vec<ExprId> = kb.case_exprs(target).into_iter().filter(|&e| self.is_matchable(e)).collect();
        let mut by_fun: FxHashMap<Sym, Vec<ExprId>> = FxHashMap::default();
        let mut by_parent: FxHashMap<Sym, Vec<ExprId>> = FxHashMap::default();
        for &t in &texprs {
            let f = kb.vocab.structural(kb.expr(t).functor);
            by_fun.entry(f).or_default().push(t);
            if let Some(p) = kb.vocab.primary_parent(f) {
                by_parent.entry(p).or_default().push(t);
            }
        }
        let mut b = Builder { kb, cfg: &self.cfg, mhs: Vec::new(), index: FxHashMap::default(), truncated: false };
        let weight = |f: Sym| -> f32 { self.cfg.pred_weights.as_ref().and_then(|w| w.get(&f).copied()).unwrap_or(1.0) };
        // Post-order: children before parents, so child MHs exist when needed.
        for &be in &bexprs {
            let f = kb.vocab.structural(kb.expr(be).functor);
            let wf = weight(f);
            if let Some(ts) = by_fun.get(&f) {
                for &te in ts {
                    b.try_pair(be, te, wf);
                }
            }
            if self.cfg.ascension && kb.vocab.kind(f) == PredKind::Relation {
                if let Some(p) = kb.vocab.primary_parent(f) {
                    if let Some(ts) = by_parent.get(&p) {
                        for &te in ts {
                            if kb.vocab.structural(kb.expr(te).functor) != f {
                                b.try_pair(be, te, self.cfg.ascension_score * wf);
                            }
                        }
                    }
                }
            }
        }
        let truncated = b.truncated;
        let mut mhs = b.mhs;
        let n = mhs.len();

        // Descendants (children always have smaller ids).
        let mut desc: Vec<BitSet> = Vec::with_capacity(n);
        for i in 0..n {
            let mut d = BitSet::new(n);
            d.insert(i);
            for &c in &mhs[i].children {
                d.union_with(&desc[c as usize]);
            }
            desc.push(d);
        }
        // Direct one-to-one conflicts.
        let mut conflicts: Vec<BitSet> = (0..n).map(|_| BitSet::new(n)).collect();
        let mut by_base: FxHashMap<Term, Vec<u32>> = FxHashMap::default();
        let mut by_target: FxHashMap<Term, Vec<u32>> = FxHashMap::default();
        for (i, m) in mhs.iter().enumerate() {
            by_base.entry(m.base).or_default().push(i as u32);
            by_target.entry(m.target).or_default().push(i as u32);
        }
        for group in by_base.values().chain(by_target.values()) {
            for (x, &a) in group.iter().enumerate() {
                for &c in &group[x + 1..] {
                    conflicts[a as usize].insert(c as usize);
                    conflicts[c as usize].insert(a as usize);
                }
            }
        }
        let mut nogood: Vec<BitSet> = Vec::with_capacity(n);
        for i in 0..n {
            let mut g = conflicts[i].clone();
            for &c in &mhs[i].children {
                g.union_with(&nogood[c as usize]);
            }
            nogood.push(g);
        }
        for i in 0..n {
            let kids_ok = mhs[i].children.iter().all(|&c| mhs[c as usize].consistent);
            mhs[i].consistent = kids_ok && !desc[i].intersects(&nogood[i]);
        }
        // Trickle-down scores, parents first (parents have larger ids).
        for i in (0..n).rev() {
            let from_parents: f32 = mhs[i].parents.iter().filter(|&&p| mhs[p as usize].consistent).map(|&p| mhs[p as usize].score).sum();
            mhs[i].score = mhs[i].local + self.cfg.trickle * from_parents;
        }
        // Kernels: consistent, non-entity, non-function MHs without a consistent parent.
        let mut kernels = Vec::new();
        for i in 0..n {
            let m = &mhs[i];
            if !m.consistent || m.is_entity() {
                continue;
            }
            if let Term::Expr(be) = m.base {
                if kb.vocab.kind(kb.expr(be).functor) == PredKind::Function {
                    continue;
                }
            }
            if m.parents.iter().any(|&p| mhs[p as usize].consistent) {
                continue;
            }
            let score = desc[i].iter().map(|j| mhs[j].score).sum();
            kernels.push(Kernel { root: i as u32, members: desc[i].clone(), nogood: nogood[i].clone(), score });
        }
        kernels.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap().then(a.root.cmp(&b.root)));
        MatchSet { base, target, mhs, kernels, truncated }
    }

    fn union_score(ms: &MatchSet, members: &BitSet) -> f32 {
        members.iter().map(|j| ms.mhs[j].score).sum()
    }

    /// Greedy merge starting from kernel `seed` (index into `ms.kernels`).
    fn greedy_from(ms: &MatchSet, seed: usize) -> (BitSet, Vec<usize>) {
        let n = ms.mhs.len();
        let mut members = BitSet::new(n);
        let mut nogood = BitSet::new(n);
        let mut used = Vec::new();
        let order = std::iter::once(seed).chain((0..ms.kernels.len()).filter(|&k| k != seed));
        for k in order {
            let kern = &ms.kernels[k];
            if kern.members.intersects(&nogood) {
                continue;
            }
            members.union_with(&kern.members);
            nogood.union_with(&kern.nogood);
            used.push(k);
        }
        (members, used)
    }

    /// Best global mappings (greedy), best first.
    pub fn map(&self, base: CaseId, target: CaseId) -> Vec<Mapping> {
        let ms = self.match_set(base, target);
        self.mappings_from(&ms)
    }

    pub fn mappings_from(&self, ms: &MatchSet) -> Vec<Mapping> {
        let mut out: Vec<Mapping> = Vec::new();
        let mut covered = BitSet::new(ms.mhs.len());
        for seed in 0..ms.kernels.len() {
            if out.len() >= self.cfg.max_mappings {
                break;
            }
            if ms.kernels[seed].members.is_subset(&covered) {
                continue;
            }
            let (members, _) = Self::greedy_from(ms, seed);
            if out.iter().any(|m| m.members == members) {
                continue;
            }
            covered.union_with(&members);
            out.push(self.finish(ms, members));
        }
        out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        out
    }

    pub fn best(&self, base: CaseId, target: CaseId) -> Option<Mapping> {
        let ms = self.match_set(base, target);
        if ms.kernels.is_empty() {
            return None;
        }
        let (members, _) = Self::greedy_from(&ms, 0);
        Some(self.finish(&ms, members))
    }

    /// Structural evaluation score of the best greedy mapping (0 if none).
    pub fn score(&self, base: CaseId, target: CaseId) -> f32 {
        let ms = self.match_set(base, target);
        if ms.kernels.is_empty() {
            return 0.0;
        }
        let (members, _) = Self::greedy_from(&ms, 0);
        Self::union_score(&ms, &members)
    }

    /// Optimal merge over kernels by branch and bound (exponential; small
    /// kernel counts only). Returns the best achievable union score.
    pub fn optimal_score(&self, ms: &MatchSet, max_kernels: usize) -> Option<f32> {
        if ms.kernels.len() > max_kernels {
            return None;
        }
        let n = ms.mhs.len();
        let ks = &ms.kernels;
        let suffix: Vec<f32> = {
            let mut s = vec![0.0; ks.len() + 1];
            for i in (0..ks.len()).rev() {
                s[i] = s[i + 1] + ks[i].score;
            }
            s
        };
        let mut best = 0.0f32;
        fn rec(i: usize, ks: &[Kernel], ms: &MatchSet, members: &BitSet, nogood: &BitSet, suffix: &[f32], best: &mut f32) {
            let cur = Mapper::union_score(ms, members);
            if cur > *best {
                *best = cur;
            }
            if i == ks.len() || cur + suffix[i] <= *best + 1e-6 {
                return;
            }
            let k = &ks[i];
            if !k.members.intersects(nogood) {
                let mut m2 = members.clone();
                m2.union_with(&k.members);
                let mut g2 = nogood.clone();
                g2.union_with(&k.nogood);
                rec(i + 1, ks, ms, &m2, &g2, suffix, best);
            }
            rec(i + 1, ks, ms, members, nogood, suffix, best);
        }
        rec(0, ks, ms, &BitSet::new(n), &BitSet::new(n), &suffix, &mut best);
        Some(best)
    }

    fn finish(&self, ms: &MatchSet, members: BitSet) -> Mapping {
        let kb = self.kb;
        let score = Self::union_score(ms, &members);
        let mut correspondences: Vec<(Term, Term)> = members.iter().map(|j| (ms.mhs[j].base, ms.mhs[j].target)).collect();
        correspondences.sort();
        let fwd: FxHashMap<Term, Term> = correspondences.iter().copied().collect();
        let tgt_mapped: FxHashSet<Term> = correspondences.iter().map(|x| x.1).collect();

        // Candidate inferences from unmapped base facts.
        let mut inferences = Vec::new();
        let mut unmatched = Vec::new();
        for &f in &kb.case(ms.base).facts {
            if !self.is_matchable(f) || fwd.contains_key(&Term::Expr(f)) {
                continue;
            }
            unmatched.push(f);
            let (mut n_items, mut n_mapped, mut expr_mapped) = (0usize, 0usize, false);
            count_support(kb, f, &fwd, &mut n_items, &mut n_mapped, &mut expr_mapped);
            if n_mapped == 0 {
                continue;
            }
            let mut has_skolem = false;
            let projected = project(kb, Term::Expr(f), &fwd, &mut has_skolem);
            inferences.push(CandidateInference {
                base_fact: f,
                projected,
                support: n_mapped as f32 / n_items.max(1) as f32,
                grounding: if expr_mapped { Grounding::Structural } else { Grounding::EntityOnly },
                has_skolem,
            });
        }

        // Alignable differences: same mapped arguments, different functor.
        let mut by_args: FxHashMap<Vec<Term>, Vec<ExprId>> = FxHashMap::default();
        for te in kb.case_exprs(ms.target) {
            if self.is_matchable(te) && !tgt_mapped.contains(&Term::Expr(te)) {
                by_args.entry(kb.expr(te).args.to_vec()).or_default().push(te);
            }
        }
        let mut alignable = Vec::new();
        for be in kb.case_exprs(ms.base) {
            if !self.is_matchable(be) || fwd.contains_key(&Term::Expr(be)) {
                continue;
            }
            let img: Option<Vec<Term>> = kb.expr(be).args.iter().map(|a| fwd.get(a).copied()).collect();
            if let Some(img) = img {
                for &te in by_args.get(&img).into_iter().flatten() {
                    if kb.expr(te).functor != kb.expr(be).functor {
                        alignable.push((be, te));
                    }
                }
            }
        }
        Mapping {
            base: ms.base,
            target: ms.target,
            members,
            correspondences,
            score,
            inferences,
            alignable_differences: alignable,
            unmatched_base_facts: unmatched,
        }
    }

    /// Render a projected inference as an s-expression.
    pub fn render_proj(&self, p: &Proj) -> String {
        match p {
            Proj::Target(t) => self.kb.render_term(*t),
            Proj::Skolem(s) => format!("(:skolem {})", self.kb.name(*s)),
            Proj::Expr { functor, args } => {
                let mut s = format!("({}", self.kb.name(*functor));
                for a in args {
                    s.push(' ');
                    s.push_str(&self.render_proj(a));
                }
                s.push(')');
                s
            }
        }
    }
}

fn count_support(kb: &Kb, e: ExprId, fwd: &FxHashMap<Term, Term>, n: &mut usize, mapped: &mut usize, expr_mapped: &mut bool) {
    for a in &kb.expr(e).args {
        *n += 1;
        if fwd.contains_key(a) {
            *mapped += 1;
            if matches!(a, Term::Expr(_)) {
                *expr_mapped = true;
            }
        } else if let Term::Expr(c) = *a {
            count_support(kb, c, fwd, n, mapped, expr_mapped);
        }
    }
}

fn project(kb: &Kb, t: Term, fwd: &FxHashMap<Term, Term>, has_skolem: &mut bool) -> Proj {
    if let Some(&x) = fwd.get(&t) {
        return Proj::Target(x);
    }
    match t {
        Term::Ent(s) => {
            *has_skolem = true;
            Proj::Skolem(s)
        }
        Term::Expr(e) => {
            let ex = kb.expr(e);
            Proj::Expr { functor: ex.functor, args: ex.args.iter().map(|a| project(kb, *a, fwd, has_skolem)).collect() }
        }
    }
}

/// Normalized IDF weights over structural functors: `idf(p) / mean idf`,
/// where df(p) = number of cases containing p (structural name).
pub fn predicate_idf(kb: &Kb, cases: &[CaseId]) -> FxHashMap<Sym, f32> {
    let mut df: FxHashMap<Sym, u32> = FxHashMap::default();
    for &c in cases {
        let mut seen = FxHashSet::default();
        for e in kb.case_exprs(c) {
            let f = kb.vocab.structural(kb.expr(e).functor);
            if seen.insert(f) {
                *df.entry(f).or_insert(0) += 1;
            }
        }
    }
    let n = cases.len() as f64;
    let idf: FxHashMap<Sym, f64> = df.iter().map(|(&p, &d)| (p, ((n + 1.0) / (d as f64 + 1.0)).ln() + 1.0)).collect();
    let mean = idf.values().sum::<f64>() / idf.len().max(1) as f64;
    idf.into_iter().map(|(p, v)| (p, (v / mean) as f32)).collect()
}
