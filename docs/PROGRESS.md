# Progress Tracker

This is the working document: current status, the task board, the decision log and the experiment log. Keep it short and current. Design rationale lives in [DESIGN.md](DESIGN.md), and experiment protocols in [EXPERIMENTS.md](EXPERIMENTS.md).

## Current focus

**P1: fingerprint feasibility (E0).** Build the generator and the E0 runner, then find out whether true analogues separate from mere-appearance and first-order-only distractors.

## Task board

Legend: `[x]` done · `[~]` in progress · `[ ]` todo · `[-]` dropped

### P0 Foundations
- [x] Cargo workspace, `target-cpu=native`, release profile
- [x] `mars-hv`: packed HVs, bind/permute/Hamming, seeded RNG, i32 accumulator, **bit-sliced weighted majority** (property-tested against the accumulator, including ties)
- [x] `mars-rel`: interner, vocabulary + taxonomy, hash-consed expression DAG (commutative canonicalization, relational order), cases, s-expression loader and renderer
- [x] `mars-encode`: feature channels C0–C3, taxonomy multi-resolution features, IDF stats (epochs), segmented sketcher, profiles; solar/atom sanity tests
- [ ] Popcount scan microbenchmark vs memory bandwidth (G0 criterion: ≥ 70% of measured bandwidth)

### P1 Fingerprint feasibility
- [~] `mars-gen`: vocabulary, domains, random templates in 7 structural families, Gentner variants (LS/TA/MA/FOR/RND), perturbations, ground-truth maps
- [ ] `mars-bench e0`: pairwise separability (AUC, per-base win rates), channel ablations, D sweep, TA distance distribution
- [ ] E0 results → `results/E0.md`; gate G1 decision
- [ ] `e1`: exhaustive retrieval at 10³–10⁵ with baselines (lexical, MAC, exact cosine, fingerprint)

### P2 Mapper
- [ ] `mars-map`: MHs, structural consistency (bitsets), kernels, trickle-down, greedy merge, candidate inferences, alignable differences
- [ ] Exhaustive small-case matcher; gold cases; E2

### Later phases
- [ ] P3 Mode K batched index, cascade, MIH; E3/E4
- [ ] P4 SDM modes; E5 (keep/drop gate)
- [ ] P5 TMS, event log, standing queries; E6
- [ ] P6 consolidation (SAGE-style), near-miss memory; E7
- [ ] P7 real data (Karla, ARN), program analogy; E8/E9

## Decision log

| Date | Decision | Why |
|---|---|---|
| 2026-09-27 | Structural features are **hashed conjunctive tuples**, not XOR-bound atomic vectors | Identical in the SimHash view (binding atomic vectors = a random vector per tuple), cheaper, and uniform across feature families |
| 2026-09-27 | Taxonomy grading = **multi-resolution features** (each structural feature also emitted at the parent-predicate level, weight α) instead of taxonomy-bundled predicate vectors | Same effect on similarity; works for co-argument and WL features where XOR-commutativity would otherwise need canonical ordering tricks |
| 2026-09-27 | Attributes (unary surface predicates) are excluded from C1–C3 | Keeps structural channels purely relational; attributes live in C0 |
| 2026-09-27 | Variant classes follow the Karla-the-Hawk design: LS = FO+HO+attributes, TA = FO+HO, MA = FO+attributes, FOR = FO only (with HO rewired, not deleted) | TA vs FOR then differs *only* in higher-order structure with equal predicate counts, so MAC content vectors cannot separate them. This is the sharpest test of structural encoding |
| 2026-09-27 | Bit-sliced counters for sketching | O(D/64) word operations per feature instead of O(D); needed for 10⁵–10⁶-case corpora |

## Experiment log

| Exp | Date | Config | Headline | Link |
|---|---|---|---|---|
| — | | | | |
