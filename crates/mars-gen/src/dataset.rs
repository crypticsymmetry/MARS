//! Dataset generation: groups of a base case plus Gentner-style variants.
//!
//! | class | first-order facts | higher-order structure | surface (domain, entities, attributes) |
//! |-------|-------------------|------------------------|----------------------------------------|
//! | LS    | same              | same                   | same                                   |
//! | TA    | same pattern      | same                   | different domain                       |
//! | MA    | same              | **re-wired**           | same                                   |
//! | FOR   | same pattern      | **re-wired**           | different domain                       |
//! | RND   | other template (other family)              | random domain                          |
//!
//! MA and FOR keep the higher-order predicate multiset, so TA vs FOR differs
//! only in *which* facts the higher-order relations connect.

use crate::template::{self, Family, PerturbOp, PredRef, TArg, Template};
use crate::vocab::{self, attribute_name, entity_name, fo_predicates, Naming, ADJECTIVES, DOMAINS, NOUNS};
use mars_hv::Rng;
use mars_rel::{CaseId, CaseKind, ExprId, Kb, Sym, Term};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VariantClass {
    Base,
    LS,
    TA,
    MA,
    FOR,
    RND,
}

impl VariantClass {
    pub fn name(self) -> &'static str {
        match self {
            VariantClass::Base => "base",
            VariantClass::LS => "LS",
            VariantClass::TA => "TA",
            VariantClass::MA => "MA",
            VariantClass::FOR => "FOR",
            VariantClass::RND => "RND",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenConfig {
    pub seed: u64,
    pub n_groups: usize,
    /// Families cycled over groups.
    pub families: Vec<Family>,
    pub naming: Naming,
    /// Random first-order distractor facts added to every case.
    pub distractors: usize,
    /// Attribute facts per entity (inclusive range).
    pub attrs_per_entity: (usize, usize),
    /// Random-template variants per group.
    pub n_rnd: usize,
    /// Perturbation operators applied (independently) to TA, MA and FOR.
    pub perturb_ops: Vec<PerturbOp>,
    /// Number of perturbation operators applied per variant.
    pub severity: usize,
}

impl Default for GenConfig {
    fn default() -> Self {
        GenConfig {
            seed: 1,
            n_groups: 1000,
            families: Family::ALL.to_vec(),
            naming: Naming::Canonical,
            distractors: 2,
            attrs_per_entity: (1, 2),
            n_rnd: 3,
            perturb_ops: Vec::new(),
            severity: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub case: CaseId,
    pub group: usize,
    pub class: VariantClass,
    pub family: Family,
    pub domain: usize,
    /// Template variable → entity. Ground-truth correspondences between
    /// members of a group are given by equal variable indices (for LS/TA).
    pub entities: Vec<Sym>,
    /// Number of the base's higher-order facts preserved (under ground-truth
    /// variable correspondence) in this variant's template.
    pub ho_overlap: usize,
}

#[derive(Clone, Debug)]
pub struct Group {
    pub family: Family,
    pub base: usize,
    pub ls: usize,
    pub ta: usize,
    pub ma: usize,
    pub for_: usize,
    pub rnd: Vec<usize>,
}

impl Group {
    /// True if TA preserves strictly more of the base's higher-order facts
    /// than MA and FOR do, i.e. the analogy is still identifiable in principle.
    pub fn discriminable(&self, ds: &Dataset) -> bool {
        let o = |i: usize| ds.items[i].ho_overlap;
        o(self.ta) > o(self.ma).max(o(self.for_))
    }

    pub fn members(&self) -> Vec<(VariantClass, usize)> {
        let mut v = vec![
            (VariantClass::LS, self.ls),
            (VariantClass::TA, self.ta),
            (VariantClass::MA, self.ma),
            (VariantClass::FOR, self.for_),
        ];
        v.extend(self.rnd.iter().map(|&r| (VariantClass::RND, r)));
        v
    }
}

pub struct Dataset {
    pub kb: Kb,
    pub items: Vec<Item>,
    pub groups: Vec<Group>,
    pub templates: Vec<Template>,
    pub config: GenConfig,
}

impl Dataset {
    pub fn item(&self, i: usize) -> &Item {
        &self.items[i]
    }
    pub fn case(&self, i: usize) -> CaseId {
        self.items[i].case
    }
}

/// Surface profile of an instantiation: domain, entity per variable, attributes.
#[derive(Clone)]
struct Surface {
    domain: usize,
    entities: Vec<Sym>,
    /// (attribute predicate, variable)
    attrs: Vec<(Sym, usize)>,
}

struct Builder<'a> {
    kb: Kb,
    cfg: &'a GenConfig,
    fos: Vec<&'static str>,
}

impl Builder<'_> {
    fn fresh_surface(&mut self, rng: &mut Rng, domain: usize, n_vars: usize) -> Surface {
        let d = DOMAINS[domain];
        let nouns = rng.sample_indices(NOUNS.len(), n_vars);
        let entities: Vec<Sym> = nouns.iter().map(|&n| self.kb.sym(&entity_name(d, n))).collect();
        let mut attrs = Vec::new();
        for v in 0..n_vars {
            let k = rng.range_inclusive(self.cfg.attrs_per_entity.0, self.cfg.attrs_per_entity.1);
            for a in rng.sample_indices(ADJECTIVES.len(), k) {
                attrs.push((self.kb.sym(&attribute_name(d, a)), v));
            }
        }
        Surface { domain, entities, attrs }
    }

    /// Add fresh entities for variables introduced by perturbation.
    fn extend_surface(&mut self, rng: &mut Rng, s: &Surface, n_vars: usize) -> Surface {
        let mut s = s.clone();
        let d = DOMAINS[s.domain];
        while s.entities.len() < n_vars {
            let e = self.kb.sym(&entity_name(d, rng.index(NOUNS.len())));
            if !s.entities.contains(&e) {
                s.entities.push(e);
            }
        }
        s
    }

    fn instantiate(&mut self, t: &Template, s: &Surface) -> Vec<ExprId> {
        let naming = self.cfg.naming;
        let d = DOMAINS[s.domain];
        let mut ids: Vec<ExprId> = Vec::with_capacity(t.nodes.len());
        let mut facts = Vec::new();
        for n in &t.nodes {
            let name = match n.pred {
                PredRef::Fo(p) => vocab::surface_pred(naming, d, p),
                other => other.canonical().to_string(),
            };
            let f = self.kb.sym(&name);
            let args: Vec<Term> = n
                .args
                .iter()
                .map(|a| match *a {
                    TArg::Var(v) => Term::Ent(s.entities[v]),
                    TArg::Node(c) => Term::Expr(ids[c]),
                })
                .collect();
            let id = self.kb.intern_expr(f, args);
            ids.push(id);
            if n.pred.is_fact() {
                facts.push(id);
            }
        }
        facts
    }

    fn make_case(&mut self, name: &str, t: &Template, s: &Surface, rng: &mut Rng) -> CaseId {
        self.make_case_d(name, t, s, rng, self.cfg.distractors)
    }

    fn make_case_d(&mut self, name: &str, t: &Template, s: &Surface, rng: &mut Rng, distractors: usize) -> CaseId {
        let mut facts = self.instantiate(t, s);
        for &(a, v) in &s.attrs {
            facts.push(self.kb.intern_expr(a, [Term::Ent(s.entities[v])]));
        }
        // Distractors: random first-order facts among the case's entities plus one extra.
        let d = DOMAINS[s.domain];
        let mut pool = s.entities.clone();
        loop {
            let extra = self.kb.sym(&entity_name(d, rng.index(NOUNS.len())));
            if !pool.contains(&extra) {
                pool.push(extra);
                break;
            }
        }
        for _ in 0..distractors {
            let p = self.fos[rng.index(self.fos.len())];
            let f = self.kb.sym(&vocab::surface_pred(self.cfg.naming, d, p));
            let ij = rng.sample_indices(pool.len(), 2);
            facts.push(self.kb.intern_expr(f, [Term::Ent(pool[ij[0]]), Term::Ent(pool[ij[1]])]));
        }
        self.kb.add_case(name, CaseKind::Episode, facts)
    }
}

fn other_domain(rng: &mut Rng, d: usize) -> usize {
    (d + 1 + rng.index(DOMAINS.len() - 1)) % DOMAINS.len()
}

pub fn generate(cfg: &GenConfig) -> Dataset {
    let mut kb = Kb::new();
    vocab::declare_vocabulary(&mut kb, cfg.naming);
    let mut b = Builder { kb, cfg, fos: fo_predicates() };
    let mut items = Vec::new();
    let mut groups = Vec::new();
    let mut templates = Vec::new();

    for g in 0..cfg.n_groups {
        let mut rng = Rng::derive(cfg.seed, g as u64);
        let family = cfg.families[g % cfg.families.len()];
        let (t, r) = loop {
            let t = template::generate(family, &mut rng);
            if let Some(r) = template::rewire(&t, &mut rng) {
                break (t, r);
            }
        };
        let d = rng.index(DOMAINS.len());
        let base_s = b.fresh_surface(&mut rng, d, t.n_vars);
        let base_keys = template::ho_keys(&t);
        let mut push = |b: &mut Builder, rng: &mut Rng, class: VariantClass, tmpl: &Template, s: &Surface, tag: &str| {
            let case = b.make_case(&format!("g{g}-{tag}"), tmpl, s, rng);
            let ho_overlap = template::overlap(&base_keys, &template::ho_keys(tmpl));
            items.push(Item { case, group: g, class, family: tmpl.family, domain: s.domain, entities: s.entities.clone(), ho_overlap });
            items.len() - 1
        };
        let base = push(&mut b, &mut rng, VariantClass::Base, &t, &base_s, "base");
        let ls = push(&mut b, &mut rng, VariantClass::LS, &t, &base_s, "LS");
        let (ops, sev) = (&cfg.perturb_ops, cfg.severity);
        let t_ta = template::perturb(&t, ops, sev, &mut rng);
        let r_ma = template::perturb(&r, ops, sev, &mut rng);
        let r_for = template::perturb(&r, ops, sev, &mut rng);
        let d_ta = other_domain(&mut rng, d);
        let ta_s = b.fresh_surface(&mut rng, d_ta, t_ta.n_vars);
        let ta = push(&mut b, &mut rng, VariantClass::TA, &t_ta, &ta_s, "TA");
        let ma_s = b.extend_surface(&mut rng, &base_s, r_ma.n_vars);
        let ma = push(&mut b, &mut rng, VariantClass::MA, &r_ma, &ma_s, "MA");
        let d_for = other_domain(&mut rng, d);
        let for_s = b.fresh_surface(&mut rng, d_for, r_for.n_vars);
        let for_ = push(&mut b, &mut rng, VariantClass::FOR, &r_for, &for_s, "FOR");
        let mut rnd = Vec::new();
        for i in 0..cfg.n_rnd {
            let others: Vec<Family> = Family::ALL.iter().copied().filter(|&f| f != family).collect();
            let f2 = others[rng.index(others.len())];
            let t2 = template::generate(f2, &mut rng);
            let d2 = rng.index(DOMAINS.len());
            let s2 = b.fresh_surface(&mut rng, d2, t2.n_vars);
            rnd.push(push(&mut b, &mut rng, VariantClass::RND, &t2, &s2, &format!("RND{i}")));
        }
        groups.push(Group { family, base, ls, ta, ma, for_, rnd });
        templates.push(t);
    }
    Dataset { kb: b.kb, items, groups, templates, config: cfg.clone() }
}

/// Many noisy instances of a few hidden templates (for prototype / schema
/// experiments). Each template also gets one *clean* instance (no
/// distractors, no perturbation) that serves as its reference prototype.
pub struct InstanceSet {
    pub kb: Kb,
    /// (case, template index, is_clean)
    pub cases: Vec<(CaseId, usize, bool)>,
    pub templates: Vec<Template>,
}

pub fn template_instances(cfg: &GenConfig, n_templates: usize, per_template: usize) -> InstanceSet {
    let mut kb = Kb::new();
    vocab::declare_vocabulary(&mut kb, cfg.naming);
    let mut b = Builder { kb, cfg, fos: fo_predicates() };
    let mut cases = Vec::new();
    let mut templates = Vec::new();
    for ti in 0..n_templates {
        let mut rng = Rng::derive(cfg.seed ^ 0x1257, ti as u64);
        let t = template::generate(cfg.families[ti % cfg.families.len()], &mut rng);
        let d = rng.index(DOMAINS.len());
        let s = b.fresh_surface(&mut rng, d, t.n_vars);
        let clean = b.make_case_d(&format!("t{ti}-clean"), &t, &s, &mut rng, 0);
        cases.push((clean, ti, true));
        for j in 0..per_template {
            let tp = template::perturb(&t, &cfg.perturb_ops, cfg.severity, &mut rng);
            let d = rng.index(DOMAINS.len());
            let s = b.fresh_surface(&mut rng, d, tp.n_vars);
            let c = b.make_case(&format!("t{ti}-i{j}"), &tp, &s, &mut rng);
            cases.push((c, ti, false));
        }
        templates.push(t);
    }
    InstanceSet { kb: b.kb, cases, templates }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_well_formed() {
        let cfg = GenConfig { n_groups: 30, ..Default::default() };
        let a = generate(&cfg);
        let b = generate(&cfg);
        assert_eq!(a.items.len(), 30 * 8);
        for (x, y) in a.items.iter().zip(&b.items) {
            assert_eq!(a.kb.render_case(x.case), b.kb.render_case(y.case));
        }
        for g in &a.groups {
            let base = a.item(g.base);
            let ls = a.item(g.ls);
            let ta = a.item(g.ta);
            let ma = a.item(g.ma);
            assert_eq!(base.entities, ls.entities);
            assert_eq!(base.entities, ma.entities);
            assert_ne!(base.domain, ta.domain);
            assert!(base.entities.iter().all(|e| !ta.entities.contains(e)));
        }
    }

    #[test]
    fn ma_shares_first_order_facts_with_base() {
        let cfg = GenConfig { n_groups: 20, distractors: 0, ..Default::default() };
        let ds = generate(&cfg);
        for g in &ds.groups {
            let base = ds.kb.case(ds.case(g.base));
            let ma = ds.kb.case(ds.case(g.ma));
            let fo_of = |facts: &[ExprId]| -> Vec<ExprId> {
                let mut v: Vec<ExprId> = facts.iter().copied().filter(|&f| ds.kb.order(f) <= 2 && !matches!(ds.kb.name(ds.kb.expr(f).functor), "cause" | "enable" | "prevent" | "implies")).collect();
                v.sort();
                v
            };
            assert_eq!(fo_of(&base.facts), fo_of(&ma.facts));
            assert_ne!(base.facts.iter().collect::<std::collections::BTreeSet<_>>(), ma.facts.iter().collect());
        }
    }
}
