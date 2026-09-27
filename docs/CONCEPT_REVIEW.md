# Concept Review: Changes, Fixes and Improvements

This document reviews the original concept ([ORIGINAL_CONCEPT.md](ORIGINAL_CONCEPT.md)) against the literature and some back-of-envelope analysis. It records every change carried into [DESIGN.md](DESIGN.md) and the reason for it. The original section numbers are cited as **§n**.

**Verdict.** The core thesis survives and is worth building:

> *cheap binary associative retrieval → exact structure mapping → provenance-tracked, incrementally maintained inferences, at millions of cases on commodity hardware.*

Five issues would have sunk the first experiment as written, though. All five are fixable (R1–R5). Several "novel" extensions turn out to have direct prior art that we should build on rather than reinvent (R11). And the justification for SDM has to change, from "speed" to capabilities that plain Hamming search lacks (R9).

Severity key: 🔴 critical (breaks the core experiment) · 🟠 major (wrong cost, claim or plan) · 🟡 minor · 🟢 keep as is.

---

## 🔴 Critical

### R1: Entity identity in the fingerprint defeats analogical retrieval (§8, §9)

`attracts(sun, earth) → PRED⊗ATTRACTS + ARG1⊗SUN + ARG2⊗EARTH`. An analogue such as `attracts(nucleus, electron)` shares only the predicate term, so one of three bundled components matches. Analogies *by definition* swap entities, so most of the fingerprint encodes exactly what we want to ignore.

**Fix.** Split the fingerprint into channels ([DESIGN §6.4](DESIGN.md#64-feature-maps-channels)). Structural channels are **entity-anonymous**: parent→child functor links, co-argument links (the same entity at position i of p and position j of q), and WL-refined role signatures of anonymous entities. Surface identity lives in its own channel (C0), which the analogy profile weights at zero.

### R2: XOR-chained higher-order binding decays multiplicatively (§9)

`cause_hv = CAUSES ⊕ ρ(attract_hv) ⊕ ρ(orbit_hv)`. For bound products, `sim(a⊗b, a′⊗b′) ≈ sim(a,a′)·sim(b,b′)`. Partially similar sub-structures (0.33 each) yield about 0.11 at the next level and less after that. The deepest structure, the thing systematicity values most, becomes the *least* visible. §8 and §9 also mix "+ bundling" and "⊕ binding" inconsistently.

**Fix.** Encode structure as **bundled conjunctive features** (`v(p) ⊗ ρ^i(v(q))`, co-argument pairs, WL labels) and weight higher-order features *up*. The algebra is defined precisely in [DESIGN §6.1](DESIGN.md#61-algebra-binary-spatter-codes), including a deterministic tie-break for even-count majorities.

### R3: SDM does not return "top-64 candidates" (§10, §36)

`memory.recall(query_fingerprint, 64)` is not an SDM operation. Classical SDM reads return **one vector**, the thresholded sum of counters, not a ranked list of case IDs. To get candidates you need one of: iterative cleanup and then exact lookup (one attractor, not 64); a heteroassociative ID readout (capacity-limited superposition that has to be decoded); or posting lists per hard location (which is really an inverted index).

**Fix.** Four explicitly defined retrieval modes ([DESIGN §7](DESIGN.md#7-retrieval-the-mac-stage)): **K** exact batched Hamming k-NN (baseline), **B** SDM-bucket posting lists ("Kanerva-coded inverted index"), **A** autoassociative cleanup and prototypes, **H** heteroassociative transition memory. Each mode is benchmarked for what it is actually good at.

### R4: The SDM attractor range is far smaller than analogue distances (§11, §12)

Kanerva's analysis puts the critical distance at about 209 of 1000 bits (≈0.21·n) at moderate load. Cues farther away diverge. Cross-domain analogues will plausibly sit at normalized Hamming distances of 0.35–0.47, far outside any basin of attraction. A radius-based SDM read would therefore return noise or the wrong attractor for exactly the cases we care about.

At the same time, **ranking** tolerates these distances well. At D = 8192 the random-pair distance is 4096 ± 45 bits. An analogue with feature cosine 0.15 lands around 3703 bits, about 8.7σ closer than random. Ranking works where attraction does not.

**Fix.** Use top-k ranking (Mode K/B) for far-analogue retrieval. Restrict SDM attraction (Mode A) to cue cleanup, near-duplicate recall and prototype formation. Add a first experiment (E0) that *measures* the analogue-distance distribution before anything is built around assumptions.

### R5: Evaluation lacked the baselines that could embarrass the idea (§23, §29, §43)

The comparisons listed were text/BM25, embeddings, plain HDC, SDM and SDM+SME. Missing:

- **MAC content vectors** (functor counts, exact dot product), the 1995 method we claim to modernize. If fingerprints don't beat it, the HD layer adds nothing.
- **Exact sparse cosine on our own features.** It isolates the *feature map* from the *sketch*: the fingerprint can at best approach it (R6).
- **Exact Hamming k-NN on the same fingerprints.** It isolates what SDM adds.
- **WL subtree kernel and GraphHD**, standard structural graph-similarity baselines.
- **Exhaustive FAC**, the upper bound for the pipeline.
- **Circularity controls.** If one person designs the generator and the encoder, features can trivially mirror the templates. Hold out template *families* and perturbation types, and require real-data tracks.

**Fix.** The full baseline ladder B1–B12 and the protocols are in [EXPERIMENTS.md](EXPERIMENTS.md).

---

## 🟠 Major

### R6: A bundled binary fingerprint is a SimHash (§8–§10, §20)

With each feature a random ±1 vector, `sign(Σ w_f v_f)` is sign-random-projection LSH, so `P[bit differs] = arccos(cos(w_A, w_B))/π`. That reframes the project: **the research content is the structural feature map; the hypervector is a compact, SIMD-friendly, incrementally updatable sketch of it.** D controls precision (resolution ∝ 1/√D), not what can be represented. Binding adds *graded* similarity only when its inputs are bundles, for example taxonomy-aware predicate vectors. This framing also explains why HD analogical retrieval "works" in the literature (Plate 1994/2000; Rachkovskij's structure-sensitive binary codes): it inherits the quality of the implicit features.

### R7: Per-case accumulators cost 16 GiB per million cases (§20)

`HyperAccumulator { sums: Box<[i16]> }` at D = 8192 is 16 KiB per case. The idea of incremental ±1 updates with only zero-crossing bits changing is correct and kept. Keeping every accumulator resident is not viable.

**Fix.** Store fingerprint bits (1 KiB) plus the case's hashed feature multiset (≈ a few KiB). Rebuild the accumulator on update (sub-millisecond) and cache hot cases in an LRU.

### R8: SDM counter width vs reversibility (§12)

With int8 counters, each location receives on the order of hundreds of writes at 10⁶ items, and correlated data drifts counters to saturation. Saturated counters make deletion (subtracting a prior write) inexact, which conflicts with the incremental retraction story.

**Fix.** Use i16 counters (2 GiB at H = 131k, D = 8192), or accept that SDM state is approximate and rebuildable (it is derived data in the dependency graph anyway).

### R9: At 10⁶ cases, SDM is not needed for speed (§11–§13, §36)

A scan of 1M × 1 KiB = 1 GiB is memory-bound at about 15–20 ms per single query. Batched and tiled (32–128 queries per pass), it becomes compute-bound: roughly sub-millisecond amortized on a 16-core AVX-512 desktop and about 1 ms on a GPU. §13 correctly identified bandwidth as the bottleneck but did not draw the conclusion: **exhaustive Hamming is a very strong baseline at 1M**.

**Fix.** Reposition SDM. Its case must rest on cleanup, prototype emergence, heteroassociative transitions and continual-learning behaviour (H6), or on 10⁷+ scale. A keep/drop gate (G4) is added.

### R10: Incrementality is non-monotone, and the retrieval side was missing (§16–§17, §30, §34)

- Greedy mapping, top-k retrieval and schema assimilation are argmax computations. Deleting a fact can change which mapping wins, so fine-grained deltas cannot be pushed *through* them. Differential Dataflow is excellent for recursive rules under change, but it is not a drop-in for a greedy matcher.
- The concept tracked how new facts change *inferences*, but not how new cases change **old retrieval results**. A newly learned case may be the best analogue for an open problem.

**Fix.** Coarse-grained invalidation units (case pair, query, schema), memoized à la Salsa/Adapton, lazy by default and eager where beliefs depend on them. I-SME-style extension for additions, with a re-merge from cached kernels. A JTMS with well-founded support for retraction. **Standing queries**, rescored on every insert: 10⁴ standing queries × 1 KiB = a 10 MiB scan per update. See [DESIGN §10](DESIGN.md#10-provenance-truth-maintenance-and-incrementality).

### R11: Consolidation, schemas and "negative analogies" already exist (§31–§33)

- **SEQL → SAGE** (Kuehne, Forbus, Gentner; Halstead & Forbus; McLure et al.) already does incremental analogical generalization: gpools, fact probabilities, an assimilation threshold, and MAC/FAC for retrieval.
- **Near-miss** learning goes back to Winston (1970), and was combined with SAGE by McLure, Friedman & Forbus (2015).
- "Negative analogies" are **alignable differences** (Markman & Gentner 1993), a first-class concept in structure-mapping theory.

**Fix.** Adopt SAGE's design explicitly. The contributions become scale (millions of cases), an idle-time scheduler with principled pair selection, SDM prototypes compared against explicit generalizations, and *difference fingerprints* that make near-misses retrievable ("similar except for polarity at the effect").

### R12: IDF and "ubiquitous predicate" weighting conflicts with incrementality (new)

Down-weighting common predicates is necessary against structured distractors (SME 2017 lists "ubiquitous predicates" as one of its five scaling techniques). But corpus statistics drift, and changing weights changes every fingerprint.

**Fix.** Vocabulary **epochs**: weights are frozen within an epoch, rebuilt in the background, and swapped atomically.

### R13: Non-identical predicates need a principled similarity (§14)

The example maps `attracts ↔ electrostatically_attracts`, which strict SME identicality forbids and random symbol vectors make orthogonal.

**Fix.** A predicate taxonomy in the vocabulary. Predicate hypervectors are bundles of their ancestors' IDs, giving graded similarity that survives XOR binding. The mapper uses minimal ascension with a reduced score.

### R14: The representation problem is the elephant (§35)

Structure mapping is only as good as the representations fed to it. The long-standing critique (Chalmers, French & Hofstadter 1992) is that hand-built representations pre-solve the analogy. An LLM parser moves the problem rather than removing it: the same text can come out as different predicate structures.

**Fix.** A controlled vocabulary with constrained decoding, a **parse-stability metric**, a synonym-unresolved setting in the generator, and real-data tracks where representations are not hand-tuned.

### R15: "What is a case?" was undefined for knowledge-graph-scale inputs (§21, §47)

Analogy works on bounded descriptions. A 10⁷-fact graph is not a case.

**Fix.** A case policy ([DESIGN §5.4](DESIGN.md#54-what-is-a-case-in-a-large-knowledge-graph)): explicit episodes; materialized views (ego-graphs, contexts, communities) for large graphs; schemas.

---

## 🟡 Minor

### R16: Bit-slicing is aimed at the wrong place (§19)

Evaluating four predicates over 64 candidates is negligible next to the mapping itself. **Move the bit-parallelism into the mapper**: MH sets, descendant closures, nogood sets and kernel admission as word-bitsets, where it dominates runtime. Keep roaring-bitmap *prefilters* for hard constraints on the retrieval side.

### R17: Majority tie-breaking (§8, §20)

Even counts produce ties, and `sums[i] >= 0` biases toward 1. Use a fixed, seed-derived tie vector.

### R18: SME complexity claim (§5)

The MH construction in the original SME is polynomial, but the exhaustive global-mapping merge is worst-case exponential. Greedy merge (Forbus & Oblinger 1990; SME 2017, about O(n² log n)) is what makes it practical. The design uses greedy merge plus an exhaustive small-case matcher to measure the approximation gap.

### R19: Name collision (§preamble)

The ICLR 2023 *Multimodal Analogical Reasoning dataSet* is called MARS. Keep MARS as a codename and pick a public name later (candidates in [DESIGN §1.4](DESIGN.md#14-naming)).

### R20: New prior art since the original research pass

- **SMTB** (Weitekamp & MacLellan, arXiv 2609.25508, September 2026): a structure-mapping algorithm reported at 5–15× faster than SME and about 50% better on large nested domains, part of the Cognitive Rule Engine (C++ with a Python interface). It maximizes relational connectivity without privileging higher-order relations. Adopt it as a reference matcher, and treat "systematicity vs connectivity" as an open question.
- **Gentner & Forbus (2025)**: an overview of SME's four decades, in *Current Directions in Psychological Science*.
- **Kanerva (2025)**: a high-dimensional computing architecture "similar to von Neumann's", recent thinking from SDM's originator.
- **Goldowsky & Sarathy (2024)**: HDC + Conceptual Spaces for analogy.
- **ARN** (TACL 2024): a narrative analogy benchmark that separates near and far analogies. LLMs struggle with far analogies zero-shot. This is a direct real-data target for H5.

### R21: Citation corrections in the original text

- The "2023 SDM/HDC study" is **Teeters, Kleyko, Kanerva & Olshausen, "On separating long- and short-term memories in hyperdimensional computing"**, *Frontiers in Neuroscience* 16 (published January 2023).
- The Stanford SDM prototype is documented in Flynn, Kanerva & Bhadkamkar, *Sparse Distributed Memory: Principles and Operation*, Stanford CSL-TR-89-400 (1989).
- The follow-up to Emruli et al. 2013 is **Emruli & Sandin (2014)**, "Analogical mapping with sparse distributed memory: a simple model that learns to generalize from examples", *Cognitive Computation*. It is the closest prior art for state-transition memory (§22).

---

## 🟢 Keep (these were right)

- **Two representations, two jobs** (§7): approximate proposes, exact disposes.
- **The MAC/FAC framing** (§4, §39), and positioning the project as its modern, persistent, large-scale runtime.
- **Synthetic-first, no NLP in v0** (§25), and **falsifiable hypotheses with failure modes** (§41–§42), now with explicit gates.
- **Adversarial surface-similarity benchmarks** (§23–§24), now formalized with Gentner's four classes (LS, TA, MA, FOR).
- **Provenance and truth maintenance** (§17, §34) as a differentiator from neural systems.
- **Candidate inferences are hypotheses** (§21).
- **LLMs on the outside only** (§35).
- **Program analogy** (§46) as an objectively evaluable domain. Suggested ground truth: Project CodeNet or Rosetta Code, where solutions to the same task in different languages are natural analogues.
- **Rust, no deep-learning dependencies in the core** (§26, §48).
- **The "smallest falsifiable prototype first" ordering** (§49), now sharpened: the very first experiment (E0) needs no index, no SDM and no mapper, only the encoder and a generator.

---

## Summary of structural changes to the plan

| Original plan | Revised plan |
|---|---|
| Phase 1: hypervector layer, then benchmark SDM | **P1: fingerprint separability first (E0)**, SDM later (P4) behind a keep/drop gate |
| Single fingerprint encoding entities and relations | **4-channel segmented fingerprint**, entity-anonymous structural channels, query profiles |
| SDM as the retriever | **Exact batched Hamming k-NN as baseline**; SDM in explicit modes (bucket, cleanup, transition) |
| Accumulator per case | features + bits stored; accumulators rebuilt on demand |
| "Differential reasoner" end to end | coarse-grained invalidation + JTMS + standing queries; Differential Dataflow only for an optional rule layer |
| Schema consolidation and negative analogies as novel | built on SAGE, near-miss and alignable differences; novelty is scale, scheduling and difference fingerprints |
| Baselines: text, embeddings, HDC, SDM | + MAC content vectors, exact sparse cosine, exact Hamming, WL, GraphHD, exhaustive FAC, SMTB |
