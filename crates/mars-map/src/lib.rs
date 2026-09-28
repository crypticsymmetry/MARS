//! `mars-map`: the FAC stage — explicit structure mapping (DESIGN §8).

pub mod bitset;
pub mod mapper;
pub mod relsim;

pub use bitset::BitSet;
pub use relsim::{learn_taxonomy, LearnedTaxonomy, RelSim};
pub use mapper::{predicate_idf, CandidateInference, Grounding, Kernel, MapConfig, Mapper, Mapping, MatchSet, Mh, Proj};

#[cfg(test)]
mod tests {
    use super::*;
    use mars_rel::{Kb, Term};

    const SRC: &str = r#"
(defpredicate attracts :arity 2 :kind relation :parents (force))
(defpredicate e-attracts :arity 2 :kind relation :parents (force))
(defpredicate revolve-around :arity 2 :kind relation)
(defpredicate greater :arity 2 :kind relation)
(defpredicate mass :arity 1 :kind function)
(defpredicate charge :arity 1 :kind function)
(defpredicate temperature :arity 1 :kind function)
(defpredicate pressure :arity 1 :kind function)
(defpredicate cause :arity 2 :kind relation)
(defpredicate and :arity * :kind logical :commutative t)
(defpredicate yellow :arity 1 :kind attribute)
(defpredicate hot :arity 1 :kind attribute)
(defcase solar
  (yellow sun) (hot sun)
  (attracts sun planet)
  (greater (mass sun) (mass planet))
  (greater (temperature sun) (temperature planet))
  (revolve-around planet sun)
  (cause (and (attracts sun planet) (greater (mass sun) (mass planet))) (revolve-around planet sun)))
(defcase atom
  (attracts nucleus electron)
  (greater (mass nucleus) (mass electron))
  (revolve-around electron nucleus))
(defcase atom-charge
  (e-attracts nucleus electron)
  (greater (charge nucleus) (charge electron))
  (revolve-around electron nucleus))
(defcase water
  (flow beaker vial water pipe)
  (greater (pressure beaker) (pressure vial))
  (cause (greater (pressure beaker) (pressure vial)) (flow beaker vial water pipe)))
(defcase heat
  (flow coffee cube heat bar)
  (greater (temperature coffee) (temperature cube))
  (cause (greater (temperature coffee) (temperature cube)) (flow coffee cube heat bar)))
"#;

    fn kb() -> Kb {
        let mut kb = Kb::new();
        kb.load_str(SRC).unwrap();
        kb
    }

    fn ent_map(kb: &Kb, m: &Mapping) -> Vec<(String, String)> {
        let mut v: Vec<_> = m.entity_map().iter().map(|(a, b)| (kb.name(*a).to_string(), kb.name(*b).to_string())).collect();
        v.sort();
        v
    }

    #[test]
    fn rutherford_analogy() {
        let kb = kb();
        let mp = Mapper::new(&kb, MapConfig::default());
        let m = mp.best(kb.case_by_name("solar").unwrap(), kb.case_by_name("atom").unwrap()).unwrap();
        assert_eq!(ent_map(&kb, &m), vec![("planet".into(), "electron".into()), ("sun".into(), "nucleus".into())]);
        // The causal explanation is projected as a candidate inference.
        let infs: Vec<String> = m.inferences.iter().map(|i| mp.render_proj(&i.projected)).collect();
        assert!(
            infs.iter().any(|s| s == "(cause (and (attracts nucleus electron) (greater (mass nucleus) (mass electron))) (revolve-around electron nucleus))"),
            "{infs:?}"
        );
        // Attributes are not transferred in analogy mode.
        assert!(infs.iter().all(|s| !s.contains("yellow")));
        // Temperature comparison transfers with skolem-free projection of new function terms.
        assert!(infs.iter().any(|s| s.contains("temperature nucleus")), "{infs:?}");
    }

    #[test]
    fn ascension_and_function_mismatch() {
        let kb = kb();
        let mp = Mapper::new(&kb, MapConfig::default());
        let m = mp.best(kb.case_by_name("solar").unwrap(), kb.case_by_name("atom-charge").unwrap()).unwrap();
        assert_eq!(ent_map(&kb, &m), vec![("planet".into(), "electron".into()), ("sun".into(), "nucleus".into())]);
        let strict = Mapper::new(&kb, MapConfig { ascension: false, ..Default::default() });
        let ms = strict.best(kb.case_by_name("solar").unwrap(), kb.case_by_name("atom-charge").unwrap()).unwrap();
        assert!(ms.score < m.score);
    }

    #[test]
    fn water_heat_flow() {
        let kb = kb();
        let mp = Mapper::new(&kb, MapConfig::default());
        let m = mp.best(kb.case_by_name("water").unwrap(), kb.case_by_name("heat").unwrap()).unwrap();
        let em = ent_map(&kb, &m);
        for pair in [("beaker", "coffee"), ("vial", "cube"), ("water", "heat"), ("pipe", "bar")] {
            assert!(em.contains(&(pair.0.to_string(), pair.1.to_string())), "{em:?}");
        }
        // pressure ↔ temperature is a non-identical function match inside greater.
        assert!(m.correspondences.iter().any(|(b, t)| matches!((b, t), (Term::Expr(_), Term::Expr(_)))
            && kb.render_term(*b) == "(pressure beaker)" && kb.render_term(*t) == "(temperature coffee)"));
    }

    #[test]
    fn one_to_one_and_consistency() {
        // Two ways to map; the mapping must be one-to-one.
        let mut kb = Kb::new();
        kb.load_str(
            r#"(defcase b (r a1 a2) (r a2 a3) (cause (r a1 a2) (r a2 a3)))
               (defcase t (r x1 x2) (r x2 x3) (r x3 x1) (cause (r x1 x2) (r x2 x3)))"#,
        )
        .unwrap();
        let mp = Mapper::new(&kb, MapConfig::default());
        let m = mp.best(kb.case_by_name("b").unwrap(), kb.case_by_name("t").unwrap()).unwrap();
        let em = m.entity_map();
        let mut targets: Vec<_> = em.values().collect();
        targets.sort();
        targets.dedup();
        assert_eq!(targets.len(), em.len(), "not one-to-one");
        assert_eq!(ent_map(&kb, &m), vec![("a1".into(), "x1".into()), ("a2".into(), "x2".into()), ("a3".into(), "x3".into())]);
        let ms = mp.match_set(kb.case_by_name("b").unwrap(), kb.case_by_name("t").unwrap());
        let opt = mp.optimal_score(&ms, 20).unwrap();
        assert!((opt - m.score).abs() < 1e-4, "greedy {} vs optimal {}", m.score, opt);
    }

    #[test]
    fn alignable_difference_detected() {
        let mut kb = Kb::new();
        kb.load_str(
            r#"(defcase b (causes a b) (causes b c))
               (defcase t (causes x y) (prevents y z))"#,
        )
        .unwrap();
        let mp = Mapper::new(&kb, MapConfig::default());
        let m = mp.best(kb.case_by_name("b").unwrap(), kb.case_by_name("t").unwrap()).unwrap();
        // Only (causes a b) ↔ (causes x y) aligns; b's second fact projects with a skolem.
        assert!(m.inferences.iter().any(|i| i.has_skolem));
    }

    #[test]
    fn relsim_finds_role_synonyms_and_soft_matching_uses_them() {
        // `pulls` is used exactly like `attracts` (same roles under `cause`), `sings` is not.
        let mut kb = Kb::new();
        kb.load_str(
            r#"(defcase a1 (cause (attracts s p) (orbits p s)))
               (defcase a2 (cause (attracts n e) (orbits e n)))
               (defcase b1 (cause (pulls m q) (orbits q m)))
               (defcase b2 (cause (pulls x y) (orbits y x)))
               (defcase c1 (sings u v) (likes v u))
               (defcase c2 (sings g h) (likes h g))"#,
        )
        .unwrap();
        let cases: Vec<_> = (0..kb.n_cases()).map(|i| mars_rel::CaseId(i as u32)).collect();
        let rs = RelSim::fit(&kb, &cases, 3, 0.1, 1.0);
        let sym = |s: &str| kb.interner.get(s).unwrap();
        let top = rs.of(sym("attracts"));
        assert_eq!(top.first().map(|x| x.0), Some(sym("pulls")), "{top:?}");
        assert!(!top.iter().any(|x| x.0 == sym("sings")));
        // Without soft matching, attracts/pulls do not align; with it they do.
        let (b, t) = (kb.case_by_name("a1").unwrap(), kb.case_by_name("b1").unwrap());
        let hard = Mapper::new(&kb, MapConfig::default()).best(b, t).unwrap();
        let soft = Mapper::new(&kb, MapConfig { soft: Some(std::sync::Arc::new(rs)), ..Default::default() }).best(b, t).unwrap();
        assert!(soft.score > hard.score, "soft {} vs hard {}", soft.score, hard.score);
        assert!(ent_map(&kb, &soft).contains(&("s".into(), "m".into())));
    }
}
