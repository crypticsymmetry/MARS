//! `mars-encode`: from explicit relational cases to structural fingerprints.
//!
//! Pipeline: [`FeatureExtractor::extract`] (case → per-channel weighted
//! feature multisets) → optional [`FeatureStats`] IDF weighting (a frozen
//! vocabulary epoch) → [`Sketcher::sketch`] (per-channel SimHash segments).
//!
//! The fingerprint is a lossy sketch of per-channel cosine similarity; the
//! exact cosine ([`sparse::channel_cosines`]) is its upper bound and is kept
//! as a baseline.

pub mod features;
pub mod sketch;
pub mod sparse;

pub use features::{lexical_tokens, mac_content_vector, symbol_hashes, Channel, FeatureConfig, FeatureExtractor, Features, SparseVec, N_CHANNELS};
pub use sketch::{channel_sims, Layout, Profile, Sketcher};
pub use sparse::{channel_cosines, cosine, FeatureStats};

#[cfg(test)]
mod tests {
    use super::*;
    use mars_rel::Kb;

    const SRC: &str = r#"
(defpredicate attracts :arity 2 :kind relation :parents (force))
(defpredicate e-attracts :arity 2 :kind relation :parents (force))
(defpredicate revolve-around :arity 2 :kind relation)
(defpredicate greater :arity 2 :kind relation)
(defpredicate mass :arity 1 :kind function)
(defpredicate cause :arity 2 :kind relation)
(defpredicate and :arity * :kind logical :commutative t)
(defpredicate yellow :arity 1 :kind attribute)
(defpredicate hot :arity 1 :kind attribute)
(defcase solar
  (yellow sun) (hot sun)
  (attracts sun planet)
  (greater (mass sun) (mass planet))
  (revolve-around planet sun)
  (cause (and (attracts sun planet) (greater (mass sun) (mass planet))) (revolve-around planet sun)))
(defcase atom
  (attracts nucleus electron)
  (greater (mass nucleus) (mass electron))
  (revolve-around electron nucleus)
  (cause (and (attracts nucleus electron) (greater (mass nucleus) (mass electron))) (revolve-around electron nucleus)))
(defcase atom-synonym
  (e-attracts nucleus electron)
  (greater (mass nucleus) (mass electron))
  (revolve-around electron nucleus)
  (cause (and (e-attracts nucleus electron) (greater (mass nucleus) (mass electron))) (revolve-around electron nucleus)))
(defcase mere-appearance
  (yellow sun) (hot sun)
  (revolve-around sun planet)
  (attracts planet sun)
  (cause (revolve-around sun planet) (attracts planet sun)))
"#;

    fn setup() -> (Kb, Vec<mars_rel::CaseId>) {
        let mut kb = Kb::new();
        let ids = kb.load_str(SRC).unwrap();
        (kb, ids)
    }

    #[test]
    fn analogues_identical_in_structural_channels() {
        let (kb, ids) = setup();
        let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
        let solar = fx.extract(ids[0]);
        let atom = fx.extract(ids[1]);
        let cos = channel_cosines(&solar, &atom);
        // Entity-anonymous channels see the same structure.
        assert!((cos[1] - 1.0).abs() < 1e-9, "{cos:?}");
        assert!((cos[2] - 1.0).abs() < 1e-9, "{cos:?}");
        assert!((cos[3] - 1.0).abs() < 1e-9, "{cos:?}");
        assert_eq!(cos[0], 0.0, "no shared surface");
    }

    #[test]
    fn taxonomy_gives_graded_similarity() {
        let (kb, ids) = setup();
        let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
        let solar = fx.extract(ids[0]);
        let syn = fx.extract(ids[2]);
        let cos = channel_cosines(&solar, &syn);
        assert!(cos[2] > 0.3 && cos[2] < 1.0, "{cos:?}");
        let fx0 = FeatureExtractor::new(&kb, FeatureConfig { taxonomy_alpha: 0.0, ..Default::default() });
        let cos0 = channel_cosines(&fx0.extract(ids[0]), &fx0.extract(ids[2]));
        assert!(cos[2] > cos0[2], "taxonomy should raise similarity: {cos:?} vs {cos0:?}");
    }

    #[test]
    fn mere_appearance_loses_on_structure_wins_on_surface() {
        let (kb, ids) = setup();
        let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
        let solar = fx.extract(ids[0]);
        let atom = fx.extract(ids[1]);
        let ma = fx.extract(ids[3]);
        let ta = channel_cosines(&solar, &atom);
        let m = channel_cosines(&solar, &ma);
        assert!(m[0] > ta[0]);
        assert!(ta[2] > m[2] && ta[3] > m[3], "ta={ta:?} ma={m:?}");
        let p = Profile::analogy();
        assert!(p.score(&ta) > p.score(&m));
    }

    #[test]
    fn fingerprint_tracks_exact_cosine() {
        let (kb, ids) = setup();
        let fx = FeatureExtractor::new(&kb, FeatureConfig::default());
        let sk = Sketcher::new(Layout::default(), 42);
        let f: Vec<_> = ids.iter().map(|&c| fx.extract(c)).collect();
        let fp: Vec<_> = f.iter().map(|x| sk.sketch(x)).collect();
        let s = sk.channel_sims(&fp[0], &fp[1]);
        assert!(s[2] > 0.999 && s[3] > 0.999, "{s:?}");
        let s_ma = sk.channel_sims(&fp[0], &fp[3]);
        assert!(s_ma[0] > 0.5);
        assert!(Profile::analogy().score(&s) > Profile::analogy().score(&s_ma));
    }
}
