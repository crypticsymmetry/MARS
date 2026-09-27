# Progress Tracker

This is the working document: current status, the task board, the decision log and the experiment log. Keep it short and current. Design rationale lives in [DESIGN.md](DESIGN.md), and experiment protocols in [EXPERIMENTS.md](EXPERIMENTS.md).

## Current focus

**P1→P2.** E0 passed its gate (see [results/E0](../results/E0/README.md)). Next: the structure mapper (P2), then retrieval at scale (E1/E3) and perturbation robustness (E4).

## Task board

Legend: `[x]` done · `[~]` in progress · `[ ]` todo · `[-]` dropped

### P0 Foundations
- [x] Cargo workspace, `target-cpu=native`, release profile
- [x] `mars-hv`: packed HVs, bind/permute/Hamming, seeded RNG, i32 accumulator, **bit-sliced weighted majority** (property-tested against the accumulator, including ties)
- [x] `mars-rel`: interner, vocabulary + taxonomy, hash-consed expression DAG (commutative canonicalization, relational order), cases, s-expression loader and renderer
- [x] `mars-encode`: feature channels C0–C3, taxonomy multi-resolution features, IDF stats (epochs), segmented sketcher, profiles; solar/atom sanity tests
- [ ] Popcount scan microbenchmark vs memory bandwidth (G0 criterion: ≥ 70% of measured bandwidth)

### P1 Fingerprint feasibility
- [x] `mars-gen`: vocabulary (54 FO preds in 9 categories, HO, functions, comparisons), 12 domains, random templates in 7 structural families, Gentner variants (LS/TA/MA/FOR/RND) with HO re-wiring, distractors, naming modes (canonical/synonyms/unresolved), ground-truth var→entity maps
- [ ] `mars-gen`: perturbation operators for E4 (delete-edge, insert-intermediate, predicate-substitute, swap-args, ...)
- [x] `mars-bench e0`: pooled AUC + per-group win rates, channel mixes, feature ablations, D sweep, distance distributions
- [x] E0 results → `results/E0/`; **G1 passed** (per-group TA-top 0.97 on held-out families; pooled AUC borderline, see finding 7)
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
| 2026-09-27 | Entity co-argument feature weight 1 → **0.25** | E0: weight 1 was fragile under distractors (TA-top 0.888 → 0.984 at 10 distractors) |
| 2026-09-27 | **Canonical predicate resolution**: structural channels use the nearest canonical ancestor; non-canonical identity → C0; unresolved → anonymized (kind, arity) | E0: domain synonyms made MA beat TA (leaf identity acted as surface). The fix makes resolved synonyms identical to canonical |
| 2026-09-27 | Added **C4 topology** channel (predicate-agnostic WL); layout 1024/1024/3072/2048/1024 | Signal for unresolved vocabularies; weak alone (≈0.5), kept at weight 0.1 |
| 2026-09-27 | Primary E0 metric = **per-group ranking** (TA-top); pooled AUC reported but secondary | Retrieval is per-query; pooled AUC mixes similarity scales across cases |

## Experiment log

| Exp | Date | Config | Headline | Link |
|---|---|---|---|---|
| E0 | 2026-09-27 | 1000 groups, 6 configs (naming × distractors) | fingerprint TA-top 0.984 (test fam. 0.972) at 2–10 distractors; MAC/lexical 0.000; unresolved vocab 0.56 | [results/E0](../results/E0/README.md) |
