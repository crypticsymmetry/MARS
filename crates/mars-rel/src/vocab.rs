//! Symbols and predicate vocabulary.

use rustc_hash::FxHashMap;

/// Interned symbol (entity name, predicate name, ...).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Sym(pub u32);

#[derive(Default, Clone, Debug)]
pub struct Interner {
    map: FxHashMap<String, Sym>,
    names: Vec<String>,
}

impl Interner {
    pub fn intern(&mut self, s: &str) -> Sym {
        if let Some(&x) = self.map.get(s) {
            return x;
        }
        let id = Sym(self.names.len() as u32);
        self.names.push(s.to_string());
        self.map.insert(s.to_string(), id);
        id
    }

    pub fn get(&self, s: &str) -> Option<Sym> {
        self.map.get(s).copied()
    }

    pub fn name(&self, s: Sym) -> &str {
        &self.names[s.0 as usize]
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PredKind {
    /// n-ary relation between entities and/or expressions.
    Relation,
    /// Unary object property (surface feature), e.g. `(yellow sun)`.
    Attribute,
    /// Maps arguments to a quantity/entity, e.g. `(mass sun)`.
    Function,
    /// `and`, `or`, `not`, `implies` ...
    Logical,
}

impl PredKind {
    pub fn parse(s: &str) -> Option<PredKind> {
        Some(match s {
            "relation" => PredKind::Relation,
            "attribute" => PredKind::Attribute,
            "function" => PredKind::Function,
            "logical" => PredKind::Logical,
            _ => return None,
        })
    }
    pub fn as_str(self) -> &'static str {
        match self {
            PredKind::Relation => "relation",
            PredKind::Attribute => "attribute",
            PredKind::Function => "function",
            PredKind::Logical => "logical",
        }
    }
}

#[derive(Clone, Debug)]
pub struct PredInfo {
    pub name: Sym,
    /// `None` = variadic.
    pub arity: Option<u8>,
    pub kind: PredKind,
    pub commutative: bool,
    /// Taxonomy parents (first is primary).
    pub parents: Vec<Sym>,
    /// False when auto-declared from use.
    pub declared: bool,
}

#[derive(Default, Clone, Debug)]
pub struct Vocabulary {
    preds: FxHashMap<Sym, PredInfo>,
}

impl Vocabulary {
    pub fn declare(&mut self, info: PredInfo) {
        self.preds.insert(info.name, info);
    }

    pub fn get(&self, p: Sym) -> Option<&PredInfo> {
        self.preds.get(&p)
    }

    pub fn contains(&self, p: Sym) -> bool {
        self.preds.contains_key(&p)
    }

    /// Auto-declare an unknown functor as a relation of the observed arity.
    pub fn ensure(&mut self, p: Sym, arity: usize) -> &PredInfo {
        self.preds.entry(p).or_insert(PredInfo {
            name: p,
            arity: Some(arity as u8),
            kind: PredKind::Relation,
            commutative: false,
            parents: Vec::new(),
            declared: false,
        })
    }

    pub fn kind(&self, p: Sym) -> PredKind {
        self.preds.get(&p).map(|i| i.kind).unwrap_or(PredKind::Relation)
    }

    pub fn is_commutative(&self, p: Sym) -> bool {
        self.preds.get(&p).map(|i| i.commutative).unwrap_or(false)
    }

    pub fn primary_parent(&self, p: Sym) -> Option<Sym> {
        self.preds.get(&p).and_then(|i| i.parents.first().copied())
    }

    /// Ancestors along primary parents, nearest first (cycle-safe).
    pub fn ancestors(&self, p: Sym) -> Vec<Sym> {
        let mut out = Vec::new();
        let mut cur = self.primary_parent(p);
        while let Some(x) = cur {
            if out.contains(&x) || x == p {
                break;
            }
            out.push(x);
            cur = self.primary_parent(x);
        }
        out
    }

    pub fn iter(&self) -> impl Iterator<Item = &PredInfo> {
        self.preds.values()
    }

    pub fn len(&self) -> usize {
        self.preds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.preds.is_empty()
    }
}
