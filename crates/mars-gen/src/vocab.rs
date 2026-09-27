//! Canonical predicate vocabulary and surface domains for the generator.

use mars_rel::{Kb, PredKind};

/// First-order relation categories (taxonomy parents) and their members.
pub const FO_CATEGORIES: &[(&str, &[&str])] = &[
    ("influence", &["activates", "inhibits", "promotes", "suppresses", "regulates", "triggers"]),
    ("control", &["controls", "owns", "manages", "commands", "governs", "directs"]),
    ("dependency", &["depends-on", "requires", "relies-on", "consumes", "needs", "uses"]),
    ("transfer", &["supplies", "sends", "transfers", "gives", "feeds", "delivers"]),
    ("spatial", &["contains", "surrounds", "adjacent-to", "above", "inside", "connects"]),
    ("social", &["trusts", "opposes", "allies-with", "competes-with", "follows", "imitates"]),
    ("motion", &["orbits", "chases", "pushes", "pulls", "attracts", "repels"]),
    ("information", &["informs", "observes", "signals", "predicts", "reports", "monitors"]),
    ("production", &["produces", "creates", "destroys", "transforms", "builds", "repairs"]),
];

/// Higher-order relations over expressions.
pub const HO_PREDS: &[&str] = &["cause", "enable", "prevent", "implies"];

/// Functions (quantities of entities).
pub const FUNCTIONS: &[&str] = &["mass", "temperature", "pressure", "amount", "size", "speed", "strength", "value"];

/// Comparisons over function expressions.
pub const COMPARISONS: &[&str] = &["greater", "less"];

pub const DOMAINS: &[&str] = &[
    "bio", "biz", "mech", "soc", "net", "logi", "eco", "law", "elec", "cook", "astro", "sport",
];

/// Nouns used to build domain-specific entity names (`<domain>_<noun>`).
pub const NOUNS: &[&str] = &[
    "alpha", "anchor", "arch", "atlas", "badge", "banner", "basin", "beacon", "birch", "bolt", "border", "bridge",
    "cabin", "canal", "castle", "cedar", "chain", "cliff", "comet", "coral", "crane", "crest", "dune", "echo",
    "ember", "falcon", "fern", "field", "flint", "forge", "fox", "garnet", "glade", "grove", "harbor", "hawk",
    "haven", "heron", "hill", "island", "ivy", "jade", "keel", "kite", "lake", "lantern", "ledge", "lotus",
    "maple", "marsh", "meadow", "mesa", "mill", "moss", "north", "oak", "orbit", "otter", "peak", "pine",
    "pond", "prism", "quartz", "quill", "raven", "reef", "ridge", "river", "rook", "sage", "shore", "sparrow",
    "spire", "spruce", "stone", "summit", "thorn", "tide", "timber", "vale", "willow", "wren", "zenith", "zephyr",
];

/// Adjectives used to build domain-specific attribute predicates (`<domain>.<adj>`).
pub const ADJECTIVES: &[&str] = &[
    "red", "blue", "green", "large", "small", "old", "young", "fast", "slow", "bright", "dark", "heavy",
    "light", "hot", "cold", "rough", "smooth", "loud", "quiet", "rare", "common", "sharp", "soft", "tall",
];

pub const AND: &str = "and";

/// How first-order predicates are named in generated cases.
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Naming {
    /// Canonical names shared by all domains.
    Canonical,
    /// Domain-specific synonyms (`bio:activates`) declared as children of the
    /// canonical predicate: analogies are visible only via the taxonomy.
    Synonyms,
    /// Domain-specific synonyms with no taxonomy link.
    Unresolved,
}

pub fn fo_predicates() -> Vec<&'static str> {
    FO_CATEGORIES.iter().flat_map(|(_, ps)| ps.iter().copied()).collect()
}

pub fn fo_category(pred: &str) -> Option<&'static str> {
    FO_CATEGORIES.iter().find(|(_, ps)| ps.contains(&pred)).map(|(c, _)| *c)
}

/// Surface name of a first-order predicate in a domain.
pub fn surface_pred(naming: Naming, domain: &str, canonical: &str) -> String {
    match naming {
        Naming::Canonical => canonical.to_string(),
        Naming::Synonyms | Naming::Unresolved => format!("{domain}:{canonical}"),
    }
}

pub fn entity_name(domain: &str, noun_idx: usize) -> String {
    format!("{domain}_{}", NOUNS[noun_idx % NOUNS.len()])
}

pub fn attribute_name(domain: &str, adj_idx: usize) -> String {
    format!("{domain}.{}", ADJECTIVES[adj_idx % ADJECTIVES.len()])
}

/// Declare the full vocabulary in a knowledge base.
pub fn declare_vocabulary(kb: &mut Kb, naming: Naming) {
    for (cat, preds) in FO_CATEGORIES {
        kb.declare(cat, Some(2), PredKind::Relation, false, &["relation"]);
        for p in *preds {
            kb.declare(p, Some(2), PredKind::Relation, false, &[cat]);
            if naming != Naming::Canonical {
                for d in DOMAINS {
                    let name = surface_pred(naming, d, p);
                    let parents: &[&str] = if naming == Naming::Synonyms { &[p] } else { &[] };
                    let s = kb.declare(&name, Some(2), PredKind::Relation, false, parents);
                    kb.vocab.set_canonical(s, false);
                }
            }
        }
    }
    for p in HO_PREDS {
        kb.declare(p, Some(2), PredKind::Relation, false, &["higher-order"]);
    }
    for f in FUNCTIONS {
        kb.declare(f, Some(1), PredKind::Function, false, &["quantity"]);
    }
    for c in COMPARISONS {
        kb.declare(c, Some(2), PredKind::Relation, false, &["comparison"]);
    }
    kb.declare(AND, None, PredKind::Logical, true, &[]);
    for d in DOMAINS {
        for a in 0..ADJECTIVES.len() {
            kb.declare(&attribute_name(d, a), Some(1), PredKind::Attribute, false, &[]);
        }
    }
}
