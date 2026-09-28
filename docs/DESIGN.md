# MARS: Concept and System Design

**Memory-Assisted Relational System** (working codename, see [§1.4](#14-naming))

| | |
|---|---|
| Status | v0.2: design plus a working prototype. **Where the experiments changed the design, [§0](#0-what-the-experiments-changed) takes precedence over the original text.** |
| Date | 2026-09-27 |
| Companion docs | [CONCEPT_REVIEW.md](CONCEPT_REVIEW.md) (what changed from the original idea and why) · [PRIOR_ART.md](PRIOR_ART.md) (literature and novelty map) · [EXPERIMENTS.md](EXPERIMENTS.md) (benchmarks, protocols, gates) · [ORIGINAL_CONCEPT.md](ORIGINAL_CONCEPT.md) (the unedited starting idea) |

---

## 0. What the experiments changed

This summary is maintained against [PROGRESS.md](PROGRESS.md); the detailed evidence is in `results/`.

| Area | Original design | Current design | Evidence |
|---|---|---|---|
| Feature channels | C0–C3 | **C0–C4**: added C4 topology (predicate-agnostic WL). Default layout 1024/1024/3072/2048/1024 | E0 |
| Predicate identity | taxonomy-graded vectors | **canonical resolution**: structural channels use the nearest *canonical* ancestor; non-canonical names → C0; unresolved → anonymized (kind, arity). Taxonomy grading = multi-resolution features | E0 |
| Co-argument features | weight 1 | **weight 0.25** (fragile under distractors) | E0 |
| FAC ranking | structural score | **fused 0.3·normalized FAC + 0.7·fingerprint score** (MAC and FAC fail in complementary ways; weight tuned in E10) | E4, E3, E10 |
| Case size | 5–200 facts | **≥ ~10 connected facts** for identifiability at 10⁶ (tiny cases collide with chance matches) | E10 |
| SDM | modes A/B/H in the core | **dropped from the core** (G4). Learned-address buckets (= IVF) kept as the sublinear index; SDM prototypes ≈ k-NN bundling | E5 |
| Standing queries | exact top-k | **pipeline semantics** by default (exact incremental fingerprint top-64 + fused re-rank; tie-inclusive boundary). Exact-fused mode kept (≈10× costlier) | E6 |
| Consolidation | SAGE-style | assimilation θ = 0.4, merge threshold 0.6 (stricter than θ). Schemas are **added alongside** instances, never replacing them; inference confidence = corroboration, not schema fact probability. A fixed θ loses purity as memory grows (needs null calibration) | E7, E15 |
| Unresolved vocabularies | open problem (representation) | **analogical bootstrapping**: wildcard mappings between retrieved neighbours → mutual-best predicate correspondences (one per domain) → canonical clusters, re-estimated each round. On sparse real data it is a precision tool: it learns frequent correspondences, and anonymization remains the default fallback. On real KGs it needs entity anchors (labels) and counts only anchor-consistent correspondences | E13, E14, E26 |
| Accepting an analogue | score threshold | **local-null significance**: z of the top-1 fused score against the lower half of its fingerprint candidate list; accept at z ≥ 9, else abstain. Memory-size invariant, unlike raw thresholds | E16 |
| Representation granularity | one encoding per case | one view per case in the engine. Views (`mars_rel::views`) trade fingerprint vs mapper strength; score-level fusion of views does not help, but *role-specialized* views do modestly (rank/map in flat, project from nested). The fusion weight depends on the view | E18, E19 |
| Natural-language input | LLM front end (planned) | `tools/llm2mars.py`: conceptual-dependency vocabulary + canonical higher-order relations; vocabulary adherence of the front end matters (1.6% vs 44% off-vocabulary between models). MAC fuses MARS with a lexical signal; an LLM verifies the short list; analogy scores discount the surface channel. Abstraction-first prompting (pattern sentence, then facts) and several pooled conversions per story are the main front-end levers | E23, E24, E25 |
| Profiles | fixed analogy profile | **domain-dependent**: in code, identifier names are informative (literal profile + FAC fusion best). In knowledge graphs, labels identify instances (surface profile for neighbours) while structure identifies schemas and roles; structure-aware neighbours overtake identity overlap as cases get richer (2-hop) | E9, E26, E27, E28 |
| Candidate inferences | structurally grounded (SME-style) only | also **first-order** inferences (`EngineConfig::first_order_inferences`) for flat facts such as KG triples. An inference is either a *substitution* (a query entity reached through the correspondences) or a *copy* (skolem for the analogue's entity); substitutions carry relational regularities and their benefit grows with relational depth | E27, E28 |
| Inference ranking | heuristic confidence | **learned transfer reliability**: each inference is typed by how it relates to the query (paths linking its arguments, or `new`); feedback learns per-type precision online (persisted), which weights inferences; reliable types read as rules induced from analogy (`Engine::induced_rules`) | E29, E30 |

Also built since: persistence (snapshot + frozen IDF epoch + op log), the `mars serve` line protocol, and corroborated inferences (top-3 analogues as separate JTMS justifications).

Also built: Python bindings (`mars-py`, §12.4), LLM front ends (`tools/llm2mars.py`), KG front ends (`tools/kg_*.py`, `tools/kg2mars.py`), near-miss diagnostics in SAGE (E20), and transfer-reliability learning in the engine (E30).

Not yet built: cascade/MIH indexes, Mode H transition memory, and applying induced rules directly / storing them as schema cases.

---

## Contents

1. [Summary](#1-summary)
2. [Background in one page](#2-background-in-one-page)
3. [Design principles](#3-design-principles)
4. [Architecture overview](#4-architecture-overview)
5. [Representation layer](#5-representation-layer)
6. [Hyperdimensional encoding](#6-hyperdimensional-encoding)
7. [Retrieval: the MAC stage](#7-retrieval-the-mac-stage)
8. [Structure mapping: the FAC stage](#8-structure-mapping-the-fac-stage)
9. [Inference, verification and epistemic status](#9-inference-verification-and-epistemic-status)
10. [Provenance, truth maintenance and incrementality](#10-provenance-truth-maintenance-and-incrementality)
11. [Consolidation, schemas and near-misses](#11-consolidation-schemas-and-near-misses)
12. [Interfaces](#12-interfaces)
13. [Implementation plan](#13-implementation-plan)
14. [Hypotheses and success criteria](#14-hypotheses-and-success-criteria)
15. [Roadmap and decision gates](#15-roadmap-and-decision-gates)
16. [Risks and mitigations](#16-risks-and-mitigations)
17. [Open questions](#17-open-questions)
18. [Glossary](#18-glossary)

---

## 1. Summary

### 1.1 One sentence

MARS is a continuously learning analogical memory for commodity hardware. Compact binary hyperdimensional fingerprints cheaply propose structurally similar memories. An explicit structure-mapping engine then verifies them and produces correspondences and candidate inferences. A dependency and truth-maintenance layer means that learning or retracting one fact only recomputes what depends on it.

### 1.2 The research question

> **Can a cheap binary associative front end keep the right structural analogues in a small candidate set, reliably enough that exact analogical reasoning becomes practical over millions of continually changing relational memories on one workstation?**

The project is built to answer this question quickly and falsifiably, in stages. Each stage has a gate (§15). If a stage fails, we learn which link broke instead of ending up with a large, uninterpretable "cognitive architecture" demo.

### 1.3 What is borrowed and what is new

Nearly every component has prior art (see [PRIOR_ART.md](PRIOR_ART.md)):

| Borrowed (mature) | Source lineage |
|---|---|
| Two-stage cheap-retrieval → expensive-mapping | MAC/FAC (Forbus, Gentner & Law 1995) |
| Structure mapping, candidate inferences, greedy merge, incremental mapping | SME (1989), I-SME (1994), SME extensions (2017), SMTB (2026) |
| Analogical generalization into schemas, near-miss learning | SEQL/SAGE, McLure et al. 2015, Winston 1970 |
| Binary hypervectors, binding and bundling | Binary Spatter Codes (Kanerva), VSA/HDC surveys |
| Associative long-term memory | Sparse Distributed Memory (Kanerva 1988), Teeters et al. 2023 |
| SDM + VSA for analogy | Emruli, Gayler & Sandin 2013; Emruli & Sandin 2014 |
| Truth maintenance, incremental view maintenance | JTMS/ATMS, DRed, Rete, Differential Dataflow, Salsa/Adapton |

What we have not found built, and what this project contributes:

1. **A structure-sensitive, entity-anonymous binary fingerprint** designed specifically for *far* (cross-domain) analogue retrieval. It is analysed explicitly as a sketch of a structural feature histogram (§6.2), and benchmarked against MAC content vectors, exact sparse cosine, WL kernels and text embeddings.
2. **A persistent, incremental MAC/FAC runtime at 10⁵–10⁷ cases on one machine**, with standing queries, provenance on every derived object, and retraction.
3. **A clean empirical test of what SDM adds** over plain Hamming k-NN (cleanup, prototype formation, continual-learning interference) instead of assuming it.
4. **An adversarial structural-retrieval benchmark** with a generator, held-out template families and Gentner's four similarity classes, plus real-data tracks.
5. **Online consolidation at scale:** SAGE-style generalization and near-miss memory, driven by an idle-time scheduler over millions of cases.

### 1.4 Naming

"MARS" collides with at least one analogy-research artifact: the *Multimodal Analogical Reasoning dataSet* (Zhang et al., ICLR 2023). We keep **MARS as an internal codename** for now (the repo is already named). Candidates for a public name: *Homolog*, *Isomorph*, *Rhyme* ("history rhymes"), *Consonance*. Decide before the first public release.

---

## 2. Background in one page

- **Structure-mapping theory** (Gentner 1983): an analogy is an alignment of *relational structure*, not of object attributes. The alignment is constrained by one-to-one correspondence and *parallel connectivity* (if two relations correspond, so do their arguments). It prefers *systematicity*: deep, interconnected systems of relations governed by higher-order relations such as `cause`.
- **SME** computes such alignments. It generates local match hypotheses (MHs), enforces structural consistency, groups MHs into kernels, scores them by *trickle-down* systematicity, merges kernels (greedily in modern versions) into global mappings, and projects **candidate inferences** from base to target.
- **MAC/FAC** models retrieval. MAC ("many are called") takes a dot product of cheap *content vectors* (counts of predicates) to pick a few candidates. FAC ("few are chosen") runs SME on those candidates only. Humans show the same pattern: surface similarity dominates *remindings*, while structural similarity dominates *judged soundness* (Gentner, Rattermann & Forbus 1993).
- **Binary Spatter Codes / HDC**: symbols are random D-bit vectors (D ≈ 10⁴). Binding is XOR, bundling is bitwise majority, and permutation encodes order. Random vectors are quasi-orthogonal, and bundles stay similar to their constituents.
- **SDM**: many "hard locations" with random addresses in {0,1}^D. A write adds the data vector into the counters of every location near the address. A read sums the counters of locations near the cue and thresholds the result. Iterated reads converge to a stored pattern if the cue lies within the *critical distance*. In Kanerva's classic setting (n = 1000, 10⁴ stored items) that distance is about 209 bits, roughly 0.21·n.
- **Truth maintenance**: every derived belief records its justifications, so retracting a premise withdraws exactly the beliefs that lose all support.

---

## 3. Design principles

1. **Approximate proposes, exact disposes.** Fingerprints and SDM only choose *what to look at*. Every analogy that leaves the system has passed explicit structure mapping.
2. **Every approximate component must beat an exact or classical baseline** on the same features, or it gets removed. SDM in particular has to earn its place against plain Hamming k-NN.
3. **No derived object without provenance.** Retrievals, mappings, inferences and schemas all carry justifications and version stamps.
4. **Work proportional to change.** A single-fact update should cost roughly the same at 10⁴ and 10⁷ cases.
5. **Data layout first.** The hot paths are contiguous bit matrices, popcounts and bitsets. The design follows memory bandwidth, not object graphs.
6. **Falsify early.** The cheapest experiments that could kill the premise run first (§15).
7. **No deep-learning dependency in the core.** LLMs and embedding models are optional front ends, back ends and *baselines*, never the reasoning engine.
8. **Determinism.** All random vectors come from seeds, and every run can be replayed from the event log.

---

## 4. Architecture overview

```text
           natural language / code / structured data
                           │   (optional LLM or program-analysis front end)
                           ▼
 ┌──────────────────────────────────────────────────────────────────────┐
 │ INGEST       parse s-expressions → validate against Vocabulary       │
 └──────────────────────────────┬───────────────────────────────────────┘
                                ▼                        Δ events
 ┌──────────────────────────────────────────────────────────────────────┐
 │ CASE STORE   hash-consed expression DAG, cases, versions, event log  │───┐
 └───────┬──────────────────────────────────────────────┬───────────────┘   │
         ▼                                              │                   │
 ┌────────────────────────┐                             │                   │
 │ ENCODER                │ feature maps C0..C3         │                   │
 │ features → ±1 sums →   │ → segmented fingerprint     │                   │
 │ sign → fingerprint     │   (e.g. 8192 bits)          │                   │
 └───────┬────────────────┘                             │                   │
         ▼                                              │                   │
 ┌────────────────────────┐   candidates (ids, approx)  │                   │
 │ RETRIEVAL  (MAC)       │───────────────┐             │                   │
 │ exact Hamming k-NN     │               ▼             ▼                   │
 │ MIH / IVF / SDM-bucket │      ┌─────────────────────────────────┐        │
 │ SDM cleanup (optional) │      │ STRUCTURE MAPPER  (FAC)         │        │
 │ exact prefilters       │      │ MHs → kernels → greedy merge →  │        │
 └────────────────────────┘      │ mappings, candidate inferences, │        │
                                 │ alignable differences           │        │
                                 └───────────────┬─────────────────┘        │
                                                 ▼                          │
                                 ┌─────────────────────────────────┐        │
                                 │ VERIFY  type / contradiction /  │        │
                                 │ rules / corroboration           │        │
                                 └───────────────┬─────────────────┘        │
                                                 ▼                          │
 ┌──────────────────────────────────────────────────────────────────────┐   │
 │ DEPENDENCY GRAPH + JTMS   facts, fingerprints, retrievals, mappings, │◀──┘
 │ inferences, schemas; standing queries; lazy/eager recomputation      │
 └──────────────────────────────┬───────────────────────────────────────┘
                                ▼
 ┌──────────────────────────────────────────────────────────────────────┐
 │ CONSOLIDATION (idle time)  SAGE-style generalization pools, near-miss│
 │ memory, SDM prototypes → new schema cases back into the store        │
 └──────────────────────────────────────────────────────────────────────┘
```

Every arrow carries versioned objects. Every box subscribes to the **delta bus** (§10) and reacts only to changes that concern it.

---

## 5. Representation layer

### 5.1 Case language

Cases are written in a small s-expression language in the style of SME/CycL:

```lisp
(defpredicate attracts        :arity 2 :kind relation  :parents (force-relation))
(defpredicate revolve-around  :arity 2 :kind relation  :parents (motion-relation))
(defpredicate greater         :arity 2 :kind relation  :parents (comparison))
(defpredicate mass            :arity 1 :kind function  :parents (quantity))
(defpredicate cause           :arity 2 :kind relation  :higher-order t :parents (causal-relation))
(defpredicate and             :arity * :kind logical   :commutative t)
(defpredicate yellow          :arity 1 :kind attribute)

(defcase solar-system
  (yellow sun)
  (attracts sun planet)
  (greater (mass sun) (mass planet))
  (revolve-around planet sun)
  (cause (and (attracts sun planet) (greater (mass sun) (mass planet)))
         (revolve-around planet sun)))

(defcase rutherford-atom
  (attracts nucleus electron)
  (greater (mass nucleus) (mass electron))
  (revolve-around electron nucleus))
```

This is the classic SME demonstration. The mapping sun↔nucleus, planet↔electron should yield the candidate inference `(cause (and …) (revolve-around electron nucleus))`, while `(yellow sun)` is not carried over.

### 5.2 Vocabulary

Each predicate declaration carries:

| Field | Purpose |
|---|---|
| `arity` | fixed n, or variadic `*` |
| `kind` | `relation`, `attribute` (unary, object property), `function` (returns an entity or quantity), `logical` (`and`, `or`, `not`, `implies`) |
| `higher-order` | whether its arguments are expressions (inferred when arguments are expressions) |
| `commutative` / `symmetric` | argument order irrelevant for matching and encoding |
| `parents` | taxonomy (DAG) used for graded predicate similarity (§6.5) and *minimal ascension* in matching (§8.1) |
| `roles` (optional) | named argument roles (`agent`, `patient` …) for cross-predicate role alignment |
| `ubiquitous` (derived) | frequency statistics used to down-weight ubiquitous predicates |
| `incompatible-with`, `functional`, `asymmetric` | used by verification (§9) |

The vocabulary is itself versioned. Changes to taxonomy or weights are handled as *epochs* (§6.7).

### 5.3 Expression DAG and hash-consing

- Every expression is interned exactly once: `ExprId = intern(functor, [arg ids])`. Identical sub-expressions across all cases share one node. That gives O(1) structural equality, memory sharing, and cheap "which cases contain this sub-expression" reverse indexes.
- Entities have global `EntityId`s. Whether "sun" in two cases is the *same* entity is a modelling decision. Case-local entities are the default, and cross-case identity is an explicit `(same-as a b)` fact.
- A **case** is a set of root facts, a kind (`Episode | Schema | Query`), metadata and a monotonically increasing `version`.

```rust
pub struct Expr {
    pub functor: PredId,
    pub args: SmallVec<[Term; 3]>,
}

pub enum Term {
    Entity(EntityId),
    Expr(ExprId),
    Const(ConstId),        // numbers, strings: opaque to matching unless declared
}

pub struct Case {
    pub id: CaseId,
    pub kind: CaseKind,
    pub version: u64,
    pub roots: Vec<ExprId>,   // top-level facts (sorted, deduplicated)
}
```

### 5.4 What is a "case" in a large knowledge graph?

Analogy operates over bounded descriptions. Knowledge arrives in three forms, and each needs a case policy:

1. **Explicit episodes** (stories, incident reports, programs, mechanisms): one input equals one case.
2. **A large connected KG**: cases are *views*. Examples are the k-hop neighbourhood of an entity, the facts within one context or microtheory, or a community produced by graph partitioning. Views are materialised and maintained incrementally like any other derived object.
3. **Schemas**: produced by consolidation, stored as `CaseKind::Schema` with generalized entities.

Recommended case size for the first phases: 5–200 facts. SME-class matchers are comfortable there, and fingerprints do not saturate (§6.2).

---

## 6. Hyperdimensional encoding

### 6.1 Algebra (Binary Spatter Codes)

- Dimension D, stored as `D/64` machine words. The default is D = 8192 (1 KiB per vector), configurable from 1024 to 16384.
- **Atomic vectors** are generated on demand from a 64-bit seed: `hv(seed) = xoshiro256**(splitmix(seed))`. No codebook is stored, and any symbol, tuple or WL label hashes to a seed.
- **Bind** `a ⊗ b = a XOR b`. It is self-inverse, preserves distance (`d(a⊗c, b⊗c) = d(a,b)`), and produces something dissimilar to both inputs.
- **Permute** `ρᵏ(a)`: a fixed pseudo-random permutation, or a word rotation with a bit shift, applied k times. It encodes argument position without a separate role codebook.
- **Bundle** is a weighted majority. Keep an integer accumulator `s = Σ wᵢ · bipolar(vᵢ)`, then set `bit_j = s_j > 0`. Ties (`s_j = 0`) are broken by a fixed, seed-derived tie vector so that results are deterministic. This fixes the even-count ambiguity in the original sketch.
- **Similarity** is `sim(a,b) = 1 − 2·d_H(a,b)/D` in [−1, 1]. It is computed per segment with XOR + POPCNT.

### 6.2 Key insight: a bundled fingerprint is a SimHash of a feature histogram

Suppose a case is encoded as a weighted bundle of *feature* vectors, `s = Σ_f w_f · v_f`, where each v_f is an independent random ±1 vector. Each coordinate s_j is then a random ±1 projection of the weight vector **w**, and `bit_j = sign(s_j)`. That is exactly **sign-random-projection LSH (SimHash, Charikar 2002)**. It follows that, approximately (CLT, large feature counts):

$$
\Pr[\text{bit}_j(A) \neq \text{bit}_j(B)] \;=\; \frac{\theta_{AB}}{\pi},
\qquad \theta_{AB} = \arccos\big(\cos(\mathbf{w}_A,\mathbf{w}_B)\big)
$$

The bits are independent across j, so `d_H ~ Binomial(D, θ/π)`.

*Sanity check (numpy simulation, D = 8192, 200 unit-weight features per case, random ±1 feature vectors, seeded tie-break):*

| feature cosine | predicted δ = θ/π | observed δ |
|---|---|---|
| 0.00 | 0.5000 | 0.4976 |
| 0.15 | 0.4521 | 0.4525 |
| 0.30 | 0.4030 | 0.4099 |
| 0.60 | 0.2952 | 0.2955 |
| 0.90 | 0.1436 | 0.1464 |

**Consequences, which drive the whole design:**

1. **Fingerprint retrieval quality is bounded by the feature map.** The fingerprint is a lossy sketch of cosine similarity between *structural feature histograms*. The research content lives in *which features we extract* (§6.4), not in the XOR tricks. An exact sparse cosine over the same features (an inverted index) is the upper bound and a mandatory baseline.
2. **D sets resolution, not capability.** The standard deviation of the normalized Hamming estimate is `√(δ(1−δ)/D)`, about 0.0055 at D = 8192. Two candidates whose feature cosines differ by about 0.07 are separated at roughly 3σ. Doubling D improves resolution by √2 and costs 2× in bandwidth.
3. **Retrieval tolerates far analogues; SDM attraction does not.** A true analogue with feature cosine ρ = 0.15 sits at an expected normalized distance of about 0.452, which is 3,703 bits against a 4,096-bit random baseline. That is about 8.7σ inside the random-distance distribution, enough to rank it above 10⁶ *random* distractors (the expected extreme of 10⁶ normals is about 4.9σ). The same case is *far* outside SDM's critical distance (about 0.2·D), so iterative SDM recall would not converge to it (§7.5). **Ranking works where attractor dynamics do not.** The real competition is *structured* distractors that share common predicates, which is why IDF weighting and higher-order features matter.
4. **Binding creates conjunctive features.** When its inputs are atomic, `v(p) ⊗ ρ(v(q))` is just a random vector for the tuple (p, q), equivalent to hashing the tuple. Binding adds *graded* similarity only when its inputs are themselves bundles, for example taxonomy-aware predicate vectors (§6.5), because XOR preserves Hamming distance.
5. **Incremental updates are exact.** Adding or removing a feature adds or subtracts `w_f·v_f` from s. Only coordinates whose sign flips change bits.

The encoder is therefore specified as a **feature map** (case → weighted multiset of hashed features) followed by a **sketch** (D-bit sign projection per segment). The two are tested separately.

### 6.3 Why naive relational encoding fails for analogy

The original idea encoded `attracts(sun, earth)` as `PRED⊗ATTRACTS + ARG1⊗SUN + ARG2⊗EARTH`, and higher-order facts by XOR-chaining sub-vectors. Two problems follow:

- **Entity identity dominates.** `attracts(nucleus, electron)` shares only the predicate component, so one of three bundled terms matches. For analogies, which by definition swap the entities, most of the signal is noise.
- **Chained binding decays multiplicatively.** For bound products, similarities multiply: `sim(a⊗b, a'⊗b') ≈ sim(a,a')·sim(b,b')`. A higher-order fact whose arguments are each 0.33-similar ends up about 0.11-similar, and the similarity falls again at every level. Deep structure, exactly what systematicity values most, becomes the least visible.

The fix is to make the structural channels **entity-anonymous** and to encode structure as *features* (bundled), not as one deep XOR chain.

### 6.4 Feature maps (channels)

The fingerprint is split into **segments**, one per channel. Hamming distance is computed per segment, so query-time channel weights are free.

| Channel | Default bits | Captures | Entity names? |
|---|---|---|---|
| **C0 Surface** | 1024 | entity symbols, attribute predicates, constants | yes |
| **C1 Content** | 1024 | multiset of functors (MAC content vector, IDF-weighted) | no |
| **C2 Relational n-grams** | 4096 | parent→child functor links, co-argument (entity-mediated) links | no, anonymous |
| **C3 WL signatures** | 2048 | Weisfeiler–Lehman refined labels of expressions and anonymous entities, h = 1..3 | no, anonymous |

**C0 Surface** has features `sym(e)` for every entity name, `attr(p)` for attribute predicates, and `const(c)`. It detects literal similarity and mere appearance. It is weighted **zero** in the analogy profile.

**C1 Content** has features `functor(p)` with weight = count × idf(p). This reproduces MAC's content vectors in HD form and doubles as a strong baseline.

**C2 Relational n-grams** have two feature families:

- *Parent–child:* for every expression `e = p(…, aᵢ, …)` where aᵢ is an expression with functor q, emit `pc(p, i, q)`. For commutative p, use i = 0. The HD form is `v(p) ⊗ ρ^i(v(q))` with taxonomy-aware `v` (§6.5). Weight: `1 + β·depth(e)` (systematicity bias, β ≈ 0.5 initially).
- *Co-argument:* for every entity x appearing at position i of expression e (functor p) *and* position j of a different expression e′ (functor q), emit the unordered `co(p,i | q,j)`. This captures first-order relational systems *without naming the entity*:
  - `attracts(sun, planet)` + `revolve-around(planet, sun)` → `co(attracts,1 | revolve-around,2)` (via sun) and `co(attracts,2 | revolve-around,1)` (via planet).
  - `attracts(nucleus, electron)` + `revolve-around(electron, nucleus)` → **the same two features**.
  - A chain `causes(A,B), causes(B,C)` → `co(causes,2 | causes,1)`, the signature of a chain.
  - Entities with degree > k_max (default 16) contribute a deterministic sample of pairs, which bounds the O(deg²) cost.

**C3 WL signatures** use the case's bipartite incidence graph (expression nodes, entity nodes, edges labelled by argument position). Initial labels are `ℓ₀(expr) = functor` and `ℓ₀(entity) = ⊥` (anonymous; optionally its attribute set in a "semi-surface" variant). Refinement: `ℓ_{t+1}(n) = hash(ℓ_t(n), sorted multiset{(edge_label, ℓ_t(m))})`. Features are all labels for t = 1..h, weighted w_t (decreasing in t, because deeper labels are more discriminative but more brittle to perturbation). After refinement, an anonymous entity's label encodes its **relational role signature**: "arg1 of attracts, arg2 of revolve-around, arg of mass inside a greater". That is exactly what makes sun and nucleus look alike.

> The channels are a hypothesis, not a commitment. Experiment E0 ([EXPERIMENTS.md](EXPERIMENTS.md)) ablates each one. Anything that does not improve true-analogue separation gets dropped.

### 6.5 Predicate similarity: taxonomy vectors, ubiquity and weights

- **Taxonomy-aware predicate vectors.** `v(p) = majority( α₀·id(p), α₁·id(parent(p)), α₂·id(grandparent(p)), … )` with decreasing α. Siblings such as `attracts` and `electrostatically-attracts` share their parent component and are therefore partially similar. Because XOR preserves distance, that graded similarity carries through every C2 feature they participate in. This is the HD counterpart of SME's *minimal ascension*.
- **Ubiquitous predicates** (`and`, `isa`, `holds-in`, very common relations) are down-weighted by `idf(p) = log((N+1)/(df(p)+1))`, floored, or stop-listed. This matters: common structure is the main source of *structured* distractors.
- **Systematicity bias:** features rooted at higher-order expressions get extra weight (C2 β, C3 w_t).

### 6.6 Query profiles

A profile is a vector of segment weights, and the score is `Σ_c λ_c · sim_c(q, x)`.

| Profile | λ(C0, C1, C2, C3) initial | Use |
|---|---|---|
| `analogy` | 0, 0.2, 0.5, 0.3 | far/cross-domain analogues |
| `literal` | 0.4, 0.2, 0.2, 0.2 | "same kind of thing" |
| `surface-only` | 1, 0, 0, 0 | diagnostics, mere-appearance detection |
| `structure-only` | 0, 0, 0.6, 0.4 | ablations |

The weights are tuned on a development split with simple, non-deep methods (grid search or logistic regression), and reported.

### 6.7 Incremental fingerprints and weight epochs

- **Do not keep per-case accumulators resident.** At D = 8192 with i16 they cost 16 KiB per case, which is 16 GiB per million cases. Store only the **fingerprint bits** (1 KiB) and the case's **feature multiset** (typically a few hundred hashed features, a few KiB at most). On update, rebuild the accumulator from features in about 0.1–1 ms, apply the delta, and publish the flipped-bit set. Hot cases can keep accumulators in an LRU cache.
- **IDF weights drift** as the corpus grows, and a weight change touches every fingerprint, which is non-incremental. Weights are therefore frozen per **vocabulary epoch**. Within an epoch everything is exactly incremental. A new epoch is rebuilt in the background (embarrassingly parallel, about 1M cases × ≤1 ms ≈ minutes on 16 cores) and swapped atomically. Queries always report the epoch they used.

### 6.8 Encoder parameters (initial)

| Parameter | Initial value | Swept in |
|---|---|---|
| D (total) | 8192 | 1024 → 16384 |
| Segment split C0/C1/C2/C3 | 1024/1024/4096/2048 | E0 |
| WL depth h | 2 | 1..3 |
| systematicity β | 0.5 | 0..2 |
| taxonomy α | 1, 0.5, 0.25 | E0 |
| co-arg k_max | 16 | 4..64 |
| IDF floor | 0.1 | E0 |

---

## 7. Retrieval: the MAC stage

### 7.1 Contract

`retrieve(query, profile, k, filters) → [(CaseId, approx_score, per_channel_scores)]`, targeting **recall@k of true structural analogues** (not precision; FAC handles precision). Per-channel scores are returned for explainability ("retrieved for structure, not surface").

### 7.2 Mode K: exact Hamming k-NN (the baseline that must be beaten)

- The fingerprint matrix is row-major, 64-byte aligned, one 1 KiB row per case. Scoring means per-segment XOR + POPCNT, a weighted sum, and a top-k heap per query thread.
- **Batching is mandatory.** A single query over 10⁶ cases reads 1 GiB, about 15–20 ms at desktop DRAM bandwidth, while the popcount work is under 1 ms. Tile the matrix so that a block of cases stays in L2 while a batch of Q = 32–128 queries is scored against it. That turns the scan compute-bound, at an estimated sub-millisecond *amortized* cost per query at 10⁶ cases on 16 cores. The GPU version (≈1 TB/s) is about 1 ms per unbatched scan.
- **Implication:** at 10⁶ cases, exhaustive Hamming is fast enough. **Speed alone does not justify SDM or sublinear indexes at this scale.** They matter at 10⁷–10⁹ cases, for single-query latency, or for capabilities other than speed (§7.5–7.6).

### 7.3 Mode X: sublinear Hamming indexes (for 10⁷+)

- **Multi-index hashing** (Norouzi, Punjani & Fleet 2014): exact r-neighbour and k-NN search by splitting codes into m substrings. Hamming distances at far-analogue scale (δ ≈ 0.45) are large, so MIH may degrade toward a linear scan. That has to be measured.
- **Binary IVF / binary HNSW** (e.g. FAISS `IndexBinaryIVF`, `IndexBinaryHNSW`), used as external baselines.
- **Cascade:** a 1024-bit sub-sketch (the leading bits of each segment) for coarse ranking, then a full-D re-rank of the top k′ = 10k.

### 7.4 Mode B: SDM-bucket index ("Kanerva-coded inverted index")

This is the SDM geometry used for *candidate generation* rather than vector recall.

- H hard locations with random addresses (for example 2¹⁶), each holding a **posting list** of case IDs instead of counters.
- **Write:** activate the A nearest hard locations to the case fingerprint (*top-A activation*, not a fixed radius, so load stays balanced) and append the case ID to their posting lists.
- **Query:** activate the A_q nearest locations, take the union of their posting lists, and score the candidates exactly (Mode K on the subset).
- Cost at 10⁶ cases, H = 65,536, A = 8: 8M postings ≈ 32 MB. A query scans 64 MiB of addresses instead of 1 GiB.
- This is essentially IVF with random centroids and multi-assignment. The experiment compares **random addresses** (Kanerva) with **learned addresses** (k-means, i.e. IVF) and with MIH.

### 7.5 Mode A: autoassociative SDM (counters)

This is the classical SDM. It does *not* return a top-k list; it returns a vector.

- **Uses:** (1) *cue cleanup and completion*: a partial or noisy query (half its structure missing) is iterated toward the nearest stored pattern or prototype, then passed to Mode K; (2) *prototype emergence*: many similar writes superimpose into an attractor that behaves like a schema (compared with SAGE in §11); (3) the continual-learning and interference experiments.
- **Limits:** attraction only works inside the critical distance, about 0.2·D at moderate load. Far analogues (δ ≈ 0.4–0.45) are *not* reachable by iteration. Mode A helps near-duplicate and partial-cue retrieval, not far-analogy retrieval.
- **Counters:** i16 (2 bytes × D × H; H = 131,072 → 2 GiB). i8 saturates under correlated data at realistic write counts. Deletion (subtracting a write) is exact only with non-saturating counters.

### 7.6 Mode H: heteroassociative transition memory

Store `state-fingerprint → transformation-fingerprint` pairs, for example "structures like this tended to undergo *cause-propagation* / *dependency reversal*". A read returns a superposition that is decoded against the transformation codebook (small, so cleanup is reliable). This is the SDM counterpart of the state-transition learning in Emruli & Sandin (2014). It is a later-phase feature (P6+).

### 7.7 Exact prefilters

Roaring-bitmap inverted indexes over functors, and optionally over C2 features, answer hard constraints such as "must contain a higher-order `cause`" or "must contain an inhibition link". Candidate sets are ANDed *before* Hamming ranking. This is where bit-parallel set algebra pays off on the retrieval side.

### 7.8 Memory and bandwidth model

| Scale | Fingerprints (D = 8192) | Features (≈2 KiB/case) | Mode B postings (A = 8) | Mode A counters (H = 131k, i16) |
|---|---|---|---|---|
| 10⁵ cases | 100 MiB | 200 MiB | 3 MiB | 2 GiB |
| 10⁶ cases | 1 GiB | 2 GiB | 32 MiB | 2 GiB (H fixed) |
| 10⁷ cases | 10 GiB | 20 GiB | 320 MiB | 2–16 GiB |

The case store, mappings and the TMS come on top of this. A 64 GiB workstation comfortably holds 10⁶ cases with all modes, and 10⁷ with fingerprints plus features, the mapper working set and mmap'd cold storage.

---

## 8. Structure mapping: the FAC stage

### 8.1 Constraints

- **Tiered identicality:** relations match if their functors are identical. A reduced-score match is allowed if they share a taxonomy parent within a depth limit (*minimal ascension*). Functions may match non-identically when their parents match (standard SME treatment). Attributes match only in the `literal` profile or when licensed by a mapped higher-order relation.
- **One-to-one:** every base item corresponds to at most one target item, and vice versa.
- **Parallel connectivity:** if two expressions correspond, their arguments correspond.
- **Systematicity:** prefer mappings that include higher-order relational structure (trickle-down scoring).

### 8.2 Algorithm (greedy SME-class)

1. **Local match hypotheses.** Index target expressions by functor (and by taxonomy parent for ascension). For each base expression, emit MHs with compatible target expressions, recursively emitting argument MHs, including entity MHs.
2. **Structural consistency.** For each MH, compute `descendants(mh)` and `nogood(mh)`, the MHs that conflict with it through one-to-one violations, as **bitsets over MH indices**. An MH is inconsistent if its descendant closure conflicts with itself.
3. **Kernels.** Root MHs (those not an argument of another MH) plus their consistent descendant closures. Inconsistent roots are split at the conflicting children.
4. **Scoring.** Local scores (identical 1.0, ascension 0.5–0.8) plus *trickle-down*: each MH passes a fraction τ of its score to its argument MHs, iterated top-down. Kernel score is the sum over its members. This implements the systematicity preference.
5. **Greedy merge.** Sort kernels by score. The mapping starts with the best kernel, then admits each next kernel `k` iff `(nogood(mapping) & members(k)) == 0`. Up to K = 3 alternative mappings are produced by seeding from the best non-overlapping kernels.
6. **Candidate inferences.** For each base expression not in the mapping but *structurally supported* (a parent or child is mapped), substitute mapped entities. Base entities that are not mapped become **skolems** `(:skolem <base-entity>)`. Each inference records its supporting MHs.
7. **Differences.** For each corresponding entity pair, collect *alignable differences* (corresponding positions with different functors or attributes) and *non-alignable differences* (unmatched structure), following Markman & Gentner. These feed near-miss memory (§11.3).
8. **Normalized score.** `score(b,t) / √(score(b,b) · score(t,t))`, comparable across candidates, so FAC can rank retrieval candidates.

### 8.3 Bit-parallel core

This is where the original "bit-slicing" idea earns its keep: *inside the mapper*, not on 64 retrieval candidates.

- MH sets, descendant sets, nogood sets and kernel membership are all `FixedBitSet`-style word arrays. Kernel admission is one AND over a few words.
- For batched FAC (one query against 64–256 candidates), candidates are processed in parallel across cores. Within a candidate, the conflict checks are word-parallel.
- Target budget: **≤ 0.2 ms per pair** for cases of about 50 facts, so FAC over 64 candidates takes about 13 ms single-threaded and 1–2 ms on 16 cores.

### 8.4 Mapping output

```rust
pub struct Mapping {
    pub id: MappingId,
    pub base: (CaseId, u64 /*version*/),
    pub target: (CaseId, u64),
    pub correspondences: Vec<(Term, Term, f32)>,
    pub score: f32,
    pub normalized: f32,
    pub candidate_inferences: Vec<CandidateInference>,
    pub alignable_differences: Vec<Difference>,
    pub nonalignable: Vec<ExprId>,
    pub matcher: MatcherInfo,        // algorithm + parameters, for provenance
}
```

### 8.5 Reference matchers and validation

- **Hand-built gold cases**: solar/atom, water-flow/heat-flow, Karla-the-Hawk sets, encoded once and used as regression tests.
- **SMTB** (Weitekamp & MacLellan 2026, part of the Cognitive Rule Engine): reported as 5–15× faster than SME and better on large nested domains. It maximizes relational connectivity without privileging higher-order relations, which is a meaningful philosophical difference from SME's systematicity bias. We use it as an external comparison through its Python interface and may adopt its bounding ideas.
- **Exhaustive exact matcher** for small cases (≤ 12 relations): brute-force the optimal one-to-one consistent mapping to measure the greedy approximation gap.
- SME itself is used as a literature reference. Its availability and licence are not assumed.

### 8.6 Incremental mapping

Following I-SME: when facts are *added* to base or target, extend existing MHs and kernels and re-run the greedy merge from the cached kernel set. When facts are *removed*, drop the affected MHs and kernels and re-merge. Greedy merge is non-monotone (removing a fact can change which mapping wins), so the *merge* is always recomputed from the cached kernels, which is cheap. MH and kernel construction is what gets cached.

---

## 9. Inference, verification and epistemic status

Analogical inferences are **hypotheses**, never facts.

| State | Meaning | Transition |
|---|---|---|
| `Proposed` | projected by a mapping | on creation |
| `Checked` | passes type/arity, no contradiction with accepted facts | automatic checks |
| `Corroborated` | independently proposed by ≥ n mappings from distinct base cases, or supported by rules | support count ≥ n |
| `Accepted` | promoted by policy (a threshold, a user, or an external oracle) | explicit |
| `Rejected` | contradiction found, or refuted by an oracle | explicit or automatic |

**Checks:** vocabulary typing; declared `incompatible-with` pairs; `functional` predicates (at most one value); asymmetry; explicit `(not …)`; optional Datalog constraint rules (§10.7). Skolem-containing inferences stay `Proposed` until the skolem is resolved to a target entity or explicitly accepted as a new entity.

**Confidence** is a documented heuristic: normalized mapping score × structural support of the inference × corroboration. It is **not** presented as a probability. Calibration is an experiment (E7), not an assumption.

*As built (E11, E27–E30):* corroboration (the number of analogues proposing an inference) is the calibrated signal. Precision rises monotonically with support, on synthetic data and on real KGs (0.07 → 0.66–0.73).

On top of that, each inference has a **transfer type** (`mars_engine::transfer`), defined against the query rather than the analogue:
- a binary fact between query entities is typed by the relation paths (length ≤ 2) that link them in the query;
- a fact about a hypothesized entity is typed as a copy (`new`).

`Engine::feedback` records whether an inference was right. The smoothed precision of its types is its **reliability**, and inferences rank by reliability × Σ fused score of the proposing analogues. This moves inferences toward `Accepted` / `Rejected` by *kind*, not one at a time. Types with enough evidence are rules induced from analogy, reported with precision and outcome counts (`Engine::induced_rules`).

---

## 10. Provenance, truth maintenance and incrementality

### 10.1 Event-sourced core

All mutations are appended to a log: `AddFact`, `RemoveFact`, `AddCase`, `RemoveCase`, `DeclarePredicate`, `NewEpoch`, `Accept`, `Reject`. State equals a snapshot plus a log replay. This gives crash recovery, reproducible experiments, and "what did the system believe at version v" queries.

### 10.2 Dependency graph

| Node | Depends on |
|---|---|
| `Fact` (asserted) | premise |
| `CaseVersion` | its facts |
| `Features(case, epoch)` | CaseVersion, vocabulary epoch |
| `Fingerprint(case, epoch)` | Features |
| `IndexEntry` (Mode K/B/A) | Fingerprint |
| `Retrieval(standing query)` | query fingerprint, index entries in its result or near its threshold |
| `Mapping(b@v, t@v′)` | two CaseVersions, matcher config |
| `Inference` | Mapping + supporting MHs, plus verification inputs |
| `Schema` | the mappings and cases assimilated into it |
| `NearMiss` | Mapping |

### 10.3 Propagation

```text
Δfact ─▶ CaseVersion++ ─▶ ΔFeatures ─▶ flipped bits ─▶ index update
                     │                                   │
                     ├─▶ Mappings(case as base/target) marked STALE
                     │        │
                     │        └─▶ (eager if they support IN inferences or standing queries)
                     │             re-map (I-SME extend or re-merge) ─▶ Δinferences ─▶ JTMS
                     │
                     └─▶ standing queries: rescore this case vs each standing query
```

- **Lazy by default:** stale mappings are recomputed on demand (memoized queries in the style of Salsa/Adapton).
- **Eager** for mappings that currently justify `Checked`/`Corroborated`/`Accepted` inferences, and for standing queries, so that beliefs are never silently stale.

### 10.4 What cannot be delta-patched, and why that is fine

Greedy merge, top-k retrieval and schema assimilation are non-monotone argmax computations. We do not try to derive fine-grained deltas *through* them. We invalidate at coarse granularity (a case pair, a query, a schema) and recompute that unit, whose cost is bounded by case size. Incrementality comes from **limiting which units are touched**, which is what H3 measures.

### 10.5 Truth maintenance (JTMS)

- Nodes are IN or OUT. A node is IN if it is a premise or has a justification whose antecedents are all IN.
- Each IN node keeps a **well-founded supporting justification**, which prevents circular self-support.
- Retracting a premise marks the transitive dependents that relied on it OUT, then tries to re-support them through alternative justifications (the classical JTMS algorithm; DRed is the equivalent over-delete-and-rederive formulation for rule-derived facts).
- ATMS-style multiple contexts ("what follows if we assume this analogy holds?") are a later option.

### 10.6 Standing queries (retrieval-side incrementality)

The original concept covered incremental *reasoning*, but not the fact that **new memories change old retrieval results**.

- A standing query keeps its top-k and its k-th score τ.
- On every fingerprint insert or update, score the changed case against all standing queries. That is a brute-force Hamming scan: 10⁴ standing queries × 1 KiB = 10 MiB, which is trivial. Mode B buckets can cut it further.
- If the score beats τ, run FAC, update the result and emit a `RetrievalChanged` event. This is how the system can *notice* that a newly learned case is analogous to an open problem.

### 10.7 Technology choices

| Need | Choice | Why |
|---|---|---|
| Memoized, invalidatable derived objects | custom, Salsa-style (revision counters, dependency edges) | coarse-grained units, full control of eviction |
| Rule layer (optional Datalog constraints and derivations) | evaluate **differential-dataflow** (Rust) vs a simple semi-naive engine with DRed | DD shines for recursive rules under change; overkill if rules are few |
| Incremental pattern matching over facts | Rete-style network (optional) | classic answer for many rules over changing facts |
| TMS | custom JTMS | small, well understood, needs tight integration |

---

## 11. Consolidation, schemas and near-misses

### 11.1 Analogical generalization (SAGE-style)

This is prior art (SEQL → SAGE) that we adopt deliberately:

- **Generalization pools** (gpools) hold generalizations plus outlier exemplars.
- A new case retrieves the best generalization or exemplar (MAC/FAC). If `normalized score ≥ assimilation threshold`, the two are merged: corresponding entities become generalized entities, and each fact carries a **probability** (the fraction of assimilated cases containing it). Low-probability facts decay out.
- Generalizations are stored as `CaseKind::Schema` cases, fingerprinted and indexed like everything else. Retrieval can therefore return schemas, and a query can map to a schema instead of an arbitrary exemplar.

### 11.2 SDM prototypes vs explicit schemas

Mode A SDM forms prototypes *implicitly* by superimposing similar writes. Experiment E6 compares (a) SAGE generalizations, (b) SDM attractors decoded back to their nearest cases and features, and (c) both, on template recovery (purity) and on whether schema-mediated retrieval improves few-shot far-analogue recall.

### 11.3 Near-miss memory

Close structural matches with localized, high-order differences are valuable (Winston 1970; alignable differences, Markman & Gentner 1993; near-miss generalization, McLure, Friedman & Forbus 2015).

- A mapping with normalized score ≥ θ_near and ≥ 1 alignable difference at a higher-order or polarity-bearing position is stored as `NearMiss { mapping, differences }`.
- **Difference fingerprints** are a small HD vector (1024 bits) bundling features `diff(p_base, p_target, role-path)`. That enables queries like "cases similar to X *except for causal polarity at the effect*".
- Near-misses refine schemas with *must-not* conditions and give the verification layer counterexamples.

### 11.4 Idle-time consolidation ("sleep")

A budgeted scheduler runs when the machine is idle:

1. **Pair selection** (never all O(N²) pairs): recent cases × their top-k retrieved neighbours not yet compared; co-bucketed cases from Mode B; schema × fresh cases.
2. **Map** each pair and persist the result, *including negative results*, so pairs are not recompared until a version changes.
3. **Assimilate** into gpools, extract near-misses, and update SDM Mode A.
4. **Report** consolidation metrics (new schemas, schema purity drift, near-miss counts).

---

## 12. Interfaces

### 12.1 Rust API sketch

```rust
let mut engine = Engine::open("data/")?;               // snapshot + log replay

engine.declare(Predicate::relation("attracts", 2).parent("force-relation"))?;
let solar = engine.insert_case_sexpr(SOLAR_SYSTEM)?;   // -> CaseId
let atom  = engine.insert_case_sexpr(RUTHERFORD)?;

// MAC
let hits = engine.retrieve(atom, Profile::Analogy, 64)?;
// FAC
let maps = engine.map_many(atom, hits.ids(), MapOptions::default())?;
for inf in &maps[0].candidate_inferences {
    println!("{} (support {:.2})", engine.render(inf), inf.support);
}

// standing query: notify when something analogous arrives
let sq = engine.watch(atom, Profile::Analogy, 16)?;

// retraction propagates through mappings and inferences
engine.remove_fact(solar, "(attracts sun planet)")?;
for ev in engine.drain_events() { /* RetrievalChanged, InferenceOut, ... */ }

// explanation / provenance
let why = engine.explain(inference_id)?;               // justification tree
```

### 12.2 CLI

```text
mars init data/
mars load data/ cases/*.mars
mars query data/ --case rutherford-atom --profile analogy -k 16 --map --explain
mars bench retrieval --corpus gen:1e6 --baselines mac,wl,exact-cosine,hamming
mars consolidate data/ --budget 10m
```

### 12.3 Language-model front and back ends (outside the core)

- **Parser:** an LLM emits s-expressions against the *controlled vocabulary*, using schema- or grammar-constrained decoding. Unknown predicates are mapped to vocabulary entries by nearest neighbour plus taxonomy, or proposed as new declarations for review.
- **Representation stability** is measured, not assumed: parse the same text N times and report the Jaccard similarity of the resulting facts. Analogy quality is only as good as representation consistency. This is the classic "tailorability" critique of SME-style systems (Chalmers, French & Hofstadter 1992).
- **Explainer:** renders mappings, inferences and justification trees into prose.
- **Program-analysis front end:** dataflow and control-flow extraction for the program-analogy track, with no LLM needed.

### 12.4 Python bindings

A `mars-py` crate (PyO3 + maturin) exposes the engine to the evaluation harness. The harness hosts the baselines (BM25, sentence embeddings, FAISS binary, WL kernels, GraphHD, SMTB via CRE) and the plots. The Rust core never depends on Python.

*As built:* `import mars` gives `Engine` (from text, files or a store; `first_order`, `profile`, `fac_weight`), with these calls:
- retrieval and mapping: `query` (with significance z), `map`, `fac`;
- standing queries: `watch` / `top` / `infer` / `explain` / `events`;
- inference with feedback: `suggest`, `ranked`, `feedback`, `induced_rules`;
- mutations and `checkpoint` / `open`.

See [crates/mars-py/README.md](../crates/mars-py/README.md).

---

## 13. Implementation plan

### 13.1 Workspace layout

```text
MARS/
├── Cargo.toml                  (workspace)
├── crates/
│   ├── mars-hv/                packed hypervectors, bind/permute, accumulators, SIMD popcount kernels
│   ├── mars-rel/               vocabulary, s-expression parser, hash-consed expression DAG, cases
│   ├── mars-encode/            feature maps C0..C3, IDF epochs, sketching, fingerprint deltas
│   ├── mars-index/             Mode K (batched exhaustive), cascade, MIH, Mode B, Mode A/H SDM, prefilters
│   ├── mars-map/               match hypotheses, kernels, trickle-down, greedy merge, inferences, differences
│   ├── mars-tms/               dependency graph, JTMS, invalidation, event bus
│   ├── mars-engine/            orchestration, persistence (log + snapshot), standing queries, consolidation
│   ├── mars-gen/               synthetic relational-analogy generator (see EXPERIMENTS.md)
│   ├── mars-bench/             benchmark runners, metrics, report output (JSON/CSV)
│   ├── mars-cli/
│   └── mars-py/                PyO3 bindings (optional feature)
├── eval/                       Python harness, baselines, notebooks, plots
├── data/                       small checked-in gold cases (large datasets fetched by scripts)
└── docs/
```

### 13.2 Dependencies (candidates)

`rayon` (parallelism), `smallvec`, `roaring` (bitmaps), `fixedbitset` or hand-rolled word bitsets, `rand_xoshiro` + `splitmix`/`wyhash` (seeded vectors), `ahash`/`foldhash`, `memmap2`, `rkyv` or `bincode` + `serde` (snapshots), `thiserror`, `tracing`, `clap`, `criterion` or `divan` (benchmarks), `proptest` (property tests), `pyo3` (bindings). Optional later: `differential-dataflow`, `cudarc`/`wgpu` (GPU scan).

### 13.3 SIMD strategy

- Portable baseline: `u64::count_ones()` compiled with `-C target-cpu=native`. LLVM emits `POPCNT`, and `VPOPCNTQ` on AVX-512 VPOPCNTDQ machines.
- Hand-tuned kernels behind feature flags: Harley–Seal / pshufb popcount for AVX2, VPOPCNTQ for AVX-512, NEON `cnt` for ARM (Apple Silicon).
- Batched tile kernel: Q queries × B cases, with per-segment popcount accumulation in registers.
- Every kernel is property-tested against the scalar reference.

### 13.4 Engineering conventions

- Determinism: global seed in config; all random vectors derive from `(seed, symbol-hash)`.
- Property tests for algebraic laws: bind self-inverse, distance preservation, the bundle-majority tie policy, and the accumulator add/remove round trip.
- Benchmarks record hardware (CPU model, cores, memory bandwidth measured by a STREAM-like probe), git SHA and config, emitted as JSON.
- Every experiment is a config file in the repo, so every figure is reproducible.

### 13.5 Persistence

- Append-only event log (length-prefixed, checksummed records).
- Periodic snapshots: the fingerprint matrix as a raw aligned file (mmap-able), the case store and DAG via rkyv, index structures rebuilt or loaded.
- Recovery = load the latest snapshot and replay the log tail.

---

## 14. Hypotheses and success criteria

The numeric thresholds are **initial targets**, to be revised after E0 with the reasons recorded.

| ID | Hypothesis | Primary measure | Initial target |
|---|---|---|---|
| **H0 Separability** | With entity-anonymous structural features, true analogues (TA) are separable from mere-appearance (MA) and random distractors in fingerprint space | pairwise AUC (TA vs MA ∪ random), analogy profile | ≥ 0.95 on held-out template families |
| **H1 Retrieval** | Binary fingerprints keep TAs in the top-k despite surface dissimilarity | recall@64 of TA, 10⁶-case corpus with MA distractors | ≥ 0.90; ≥ MAC content vectors + 0.15; within 0.03 of exact sparse cosine on the same features |
| **H2 Scale** | MAC→FAC matches exhaustive FAC accuracy while mapping a tiny fraction | top-1 structural accuracy vs exhaustive; fraction mapped; latency | ≥ 95% of exhaustive; ≤ 10⁻⁴ of the corpus mapped; p50 ≤ 100 ms at 10⁶ on 16 cores |
| **H3 Incrementality** | Update cost tracks the affected region, not corpus size | work units and wall time per single-fact update vs N | flat from 10⁴ to 10⁶; ≥ 100× cheaper than recomputing affected results from scratch |
| **H4 Abstraction** | Consolidation recovers hidden templates and helps retrieval | schema purity; recall@k with vs without schemas | purity ≥ 0.9; measurable recall gain for 1–3-shot families |
| **H5 Robustness** | Structural reranking suppresses surface-driven false analogies | MA-intrusion rate@k vs embedding retrieval, including ARN far analogies | ≥ 50% relative reduction |
| **H6 SDM value** | SDM modes add something plain Hamming k-NN does not | partial-cue recall, prototype quality, interference over time | significant gain on ≥ 1 of the three, or SDM is dropped from the core |

---

## 15. Roadmap and decision gates

Phases are ordered so that the cheapest experiment able to kill the premise runs first. Details are in [EXPERIMENTS.md](EXPERIMENTS.md).

| Phase | Build | Experiments | Gate |
|---|---|---|---|
| **P0 Foundations** | `mars-hv`, `mars-rel` (parser, DAG), `mars-gen` v0, eval harness skeleton | unit and property tests, popcount microbenchmarks | algebra laws pass; scan throughput ≥ 70% of measured memory bandwidth |
| **P1 Fingerprint feasibility** | `mars-encode` C0–C3, exact sparse-cosine baseline, MAC baseline | **E0** separability and ablations, **E1** retrieval at 10³–10⁵ | **G1 (H0):** TA/MA AUC ≥ 0.95 on held-out families. If this fails after feature iteration, the HD layer is demoted to a loose filter and the premise is re-scoped |
| **P2 Mapper** | `mars-map` greedy SME-class, gold cases, exhaustive small-case matcher | **E2** mapping correctness vs gold, exhaustive and SMTB | correspondence F1 ≥ 0.95 on gold; greedy gap ≤ 5% |
| **P3 MAC/FAC at scale** | Mode K batched, cascade, MIH, FAISS baselines | **E3** end-to-end 10¹–10⁶, **E4** perturbation robustness | **G3 (H1, H2)** |
| **P4 SDM** | Modes B, A (and H prototype) | **E5** SDM value-add | **G4 (H6):** keep, restrict or drop SDM |
| **P5 Incremental** | `mars-tms`, event log, standing queries, I-SME-style updates | **E6** update streams | **G5 (H3)** |
| **P6 Consolidation** | gpools, near-miss memory, idle scheduler | **E7** template recovery, schema-assisted retrieval, inference calibration | H4 |
| **P7 Real data and front ends** | LLM parser, program-analysis front end, Python bindings | **E8** Karla/ARN/StoryAnalogy, **E9** program analogy (CodeNet/Rosetta), mechanism search pilot | H5 |

**Stop rule:** if G1 fails *and* exact sparse cosine on the same features also fails, the problem is the feature map or the representations, not HD. Pivot to representation research before building more machinery.

---

## 16. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| **Benchmark circularity**: the generator and the encoder are designed by the same people, so the features may trivially match the templates | inflated H0/H1 | template *families* held out; perturbation types held out; an independent generator author where possible; mandatory real-data tracks (E8, E9) |
| **Representation dependence** (tailorability): analogies found only because inputs were hand-coded to align | results don't transfer | controlled vocabulary; parse-stability metric; synonym-unresolved setting in the generator; LLM-parsed real data |
| **Structured distractors**: common predicates make everything look alike | TA recall collapses at scale | IDF epochs, higher-order weighting, prefilters, WL channel; measure recall vs corpus size |
| **Fingerprint saturation** for large cases (hundreds of features) | similarity washes out | case-size policy (§5.4); per-channel feature caps; larger D for the C2 segment |
| **Co-argument feature explosion** (O(deg²)) | encode time and noise | k_max sampling; hub down-weighting |
| **SDM adds nothing** | wasted effort | isolated as P4 with an explicit keep/drop gate; the core never depends on it |
| **Mapper cost dominates** after filtering | latency | bitset kernels, parallel FAC, SMTB-style bounds, early termination when the upper bound < current k-th score |
| **Weight or taxonomy changes are non-incremental** | periodic full rebuilds | vocabulary epochs, background rebuild, atomic swap |
| **Scope creep** into a "cognitive architecture" | nothing finishes | gates; LLM and GUI work stays outside the core until P7 |
| **Dataset licences** (stories, pathways, code corpora) | cannot publish | check each licence before use; the synthetic generator is fully owned |

---

## 17. Open questions

1. **Random vs learned hard-location addresses** (Mode B): is k-means (IVF) strictly better, or does random coverage help on novel structure?
2. **Channel weighting**: fixed profiles vs per-query adaptive weights (for example, raise C0 if the query has few relations).
3. **Continuous quantities**: qualitative encodings (`greater`, ordinal bins) vs level hypervectors; how these interact with analogy.
4. **Temporal and sequential structure**: event order as higher-order relations (`before`) vs positional permutation encodings.
5. **Negation and modality** inside matching and inference.
6. **Cross-case entity identity**: when to merge entities across cases, and the effect on C0 and literal similarity.
7. **Case segmentation** for large KGs (§5.4): ego-graphs vs communities vs contexts.
8. **Mapper philosophy**: SME-style systematicity vs SMTB-style connectivity maximization. Which agrees better with human judgements (Karla sets) *and* gives better candidate inferences?
9. **Resonator networks** (Frady et al. 2020) for *unbinding* fingerprints, i.e. reading structure back out of them. Is there a use beyond diagnostics?
10. **GPU**: is a GPU scan worth its complexity at 10⁶, or only at 10⁷+?

---

## 18. Glossary

| Term | Meaning |
|---|---|
| **Base / target** | the remembered (source) and current (target) descriptions in an analogy |
| **Candidate inference** | base structure projected into the target through a mapping; a hypothesis |
| **Critical distance** | the largest cue-to-pattern distance from which iterated SDM reads converge |
| **FAC** | "few are chosen": the expensive structural matching stage |
| **Gpool** | generalization pool (SAGE): generalizations plus outlier exemplars for one concept |
| **Hard location** | an SDM storage location with a fixed random address |
| **Kernel** | a structurally consistent MH subtree rooted at a root MH (SME) |
| **MAC** | "many are called": the cheap retrieval stage |
| **MH** | match hypothesis: a proposed correspondence between two items |
| **Minimal ascension** | matching non-identical predicates through a close common ancestor |
| **Mere appearance (MA)** | a pair sharing surface features but not relational structure |
| **Near-miss** | a close match differing in a small number of significant places |
| **Parallel connectivity** | corresponding expressions must have corresponding arguments |
| **Skolem** | a placeholder entity created for an unmapped base entity in an inference |
| **Systematicity** | preference for deep, connected relational systems |
| **True analogy (TA)** | shared relational structure, different surface |
| **WL** | Weisfeiler–Lehman iterative label refinement |
