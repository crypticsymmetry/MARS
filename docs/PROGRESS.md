# Progress Tracker

This is the working document: current status, the task board, the decision log and the experiment log. Keep it short and current. Design rationale lives in [DESIGN.md](DESIGN.md), and experiment protocols in [EXPERIMENTS.md](EXPERIMENTS.md).

## Current focus

**Consolidate and harden.** H0–H4 hold on synthetic data (E0, E2–E4, E6, E7), SDM was dropped (E5), and the first real-data test (E9, program analogy) shows structure beats lexical and MAC retrieval but exposes the representation problem. Candidate next steps: (1) representation normalization for code (P7b); (2) persistence (event log + snapshots) and a CLI/API so MARS is usable as a tool; (3) near-miss memory; (4) Mode K kernel speed-ups.

## Task board

Legend: `[x]` done · `[~]` in progress · `[ ]` todo · `[-]` dropped

### P0 Foundations
- [x] Cargo workspace, `target-cpu=native`, release profile
- [x] `mars-hv`: packed HVs, bind/permute/Hamming, seeded RNG, i32 accumulator, **bit-sliced weighted majority** (property-tested against the accumulator, including ties)
- [x] `mars-rel`: interner, vocabulary + taxonomy, hash-consed expression DAG (commutative canonicalization, relational order), cases, s-expression loader and renderer
- [x] `mars-encode`: feature channels C0–C3, taxonomy multi-resolution features, IDF stats (epochs), segmented sketcher, profiles; solar/atom sanity tests
- [x] Popcount scan microbenchmark vs memory bandwidth: single query 79% of probe (G0 pass); batched 6.6 ms/query at 10⁶ rows on 4 cores → [results/bw](../results/bw/)
- [ ] Mode K kernel: query register-blocking / transposed layout (currently about 55 cycles per row per query)

### P1 Fingerprint feasibility
- [x] `mars-gen`: vocabulary (54 FO preds in 9 categories, HO, functions, comparisons), 12 domains, random templates in 7 structural families, Gentner variants (LS/TA/MA/FOR/RND) with HO re-wiring, distractors, naming modes (canonical/synonyms/unresolved), ground-truth var→entity maps
- [x] `mars-gen`: perturbation operators (delete-fact, insert-intermediate, substitute-predicate, swap-args, add-ho) + ground-truth higher-order overlap / *discriminable* flag
- [x] `mars-bench e0`: pooled AUC + per-group win rates, channel mixes, feature ablations, D sweep, distance distributions
- [x] E0 results → `results/E0/`; **G1 passed** (per-group TA-top 0.97 on held-out families; pooled AUC borderline, see finding 7)
- [ ] `e1`: exhaustive retrieval at 10³–10⁵ with baselines (lexical, MAC, exact cosine, fingerprint)

### P2 Mapper
- [x] `mars-map`: MHs (identical, minimal ascension, non-identical functions, commutative permutations), bitset consistency, kernels, trickle-down, greedy merge + alternatives, branch-and-bound optimal merge, candidate inferences (skolems, grounding), alignable differences
- [x] Gold cases as unit tests (solar/atom, water/heat, 1:1 consistency); E2 → [results/E2](../results/E2/README.md)
- [ ] SMTB comparison (needs the CRE Python package; network permitting)

### P8 Hardening round 2
- [x] E10 scoring study → [results/E10](../results/E10/README.md): normalization/IDF ≈ no effect; fusion weight 0.3 best (new default); E3 degradation explained as an information-per-case limit (2-template cases: 0.996 at 10⁶)
- [x] `mars-gen` template composition (`compose`); `MapConfig::pred_weights` (IDF option)
- [x] E9 follow-up: syntactic normalization passes for code (negative result); fusion 0.3 also best on real code (MRR 0.359)
- [x] **Persistence**: snapshot (`kb.mars` + `meta.txt`) + frozen IDF epoch (`epoch.idf`) + append-only op log, replay on open; round-trip test reproduces results and inferences exactly
- [x] **`mars serve`**: line protocol (case/fact/unfact/retire/declare/query/watch/top/infer/explain/map/events/checkpoint/stats); `data/examples/session.txt`
- [x] E12 cross-language program analogy (Python ↔ JS; `tools/js_ast.js` + `tools/js2mars.py` into the shared vocabulary) → [results/E12](../results/E12/README.md): structure ≈ 2× lexical (MRR 0.168 vs 0.089), low absolute accuracy
- [x] E11 inference calibration → [results/E11](../results/E11/README.md): precision rises 0.07 → 0.86 with support 1 → 5; 5 analogues with support ≥ 2 keep precision and add 59% recall. Default `infer_from` = 5; `corroborated()` API
- [x] **Corroborated inferences**: inferences drawn from the top analogues, one JTMS justification per analogue; `support()` = corroboration count; E6 still exact (0/500)

### Tooling
- [x] `mars-cli`: `mars analogies | map | stats` over `.mars` files; `data/examples/classic.mars` (Rutherford, water/heat flow, supply-chain/chokepoint, mere-appearance foil)

### Later phases
- [x] P3 `mars-index` Mode K (SoA per segment, fused AVX-512 weighted kernel, L1 tiling, integer admission threshold)
- [x] E4 perturbation robustness → [results/E4](../results/E4/README.md)
- [x] E3 end-to-end MAC→FAC (fused re-rank) at 10⁴–10⁶ with sparse baselines (B2/B4/B5) and exhaustive upper bound → [results/E3](../results/E3/README.md)
- [ ] Cascade (1024-bit pre-scan), MIH / IVF comparisons (needed only at ≥10⁷)
- [x] P4 SDM: Mode B (buckets; random / data / k-means addresses), Mode A (autoassociative i16 counters, parallel batch writes); E5 → [results/E5](../results/E5/README.md). **G4: SDM dropped from core** (learned-address buckets kept as IVF)
- [x] P5 `mars-tms` (JTMS, well-founded support, circular-support test), `mars-engine` (live Mode K with soft delete, frozen IDF epoch, memoized FAC by case versions, standing queries in pipeline and exact-fused modes, TMS-maintained inferences with provenance, events, work counters); E6 → [results/E6](../results/E6/README.md). **G5 passed**
- [ ] Reverse index over standing queries (for ≥ 10³ SQs)
- [x] P6 `mars-engine::sage`: SAGE-style generalization (assimilation threshold, fact probabilities, wear-away, schema materialization), sleep consolidation (outlier re-offer + merge); E7 → [results/E7](../results/E7/README.md). **H4 supported** for few-shot inference
- [ ] Near-miss memory + difference fingerprints (E7c)
- [ ] Hierarchical generalization to reduce fragmentation; schema-level retrieval at 10⁶ (E3 weakness)
- [x] P7a program analogy on real code: `tools/py2mars.py` (Python AST → relational cases, one-level helper inlining), `tools/fetch_e9_corpus.sh` (3 MIT PyPI packages), E9 → [results/E9](../results/E9/README.md)
- [ ] P7b representation normalization for code (loop canonicalization, idioms, def-use dataflow); per-domain profile tuning
- [ ] P7c narrative analogies (ARN / Karla stories): blocked by network (datasets not reachable from this environment)

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
| 2026-09-27 | Inferences from the **top-5 analogues**; support ≥ 2 = "corroborated" | E11: support is calibrated (precision 0.07 → 0.86); 5 analogues with support ≥ 2 is a Pareto improvement over the single analogue |
| 2026-09-27 | Inferences from the **top-3 analogues**, one justification each | Demo showed a single-analogue policy drops a well-supported inference when the top analogue changes; the TMS naturally models corroboration |
| 2026-09-27 | **Fusion weight w = 0.3** (structural) / 0.7 (fingerprint) | E10: best at every severity (e.g. 0.692 vs 0.645 at severity 2) |
| 2026-09-27 | Case-size guidance: ≥ ~10 connected facts per case at 10⁶ scale | E10: single-template cases degrade with N; 2-template cases stay ≥ 0.996 up to 10⁶ |
| 2026-09-27 | Profiles are **domain-dependent**: for code, the literal profile (with identifier names) + FAC fusion is best | E9: identifier conventions are informative across authors; the pure-analogy profile under-uses them |
| 2026-09-27 | SAGE assimilation θ = 0.4; sleep merge threshold 0.6 (stricter than θ) | E7: merging at θ merges small denoised schemas across templates (purity 0.98 → 0.87) |
| 2026-09-27 | Standing queries default to **pipeline semantics** (exact incremental fp top-64 + fused re-rank), with a tie-inclusive candidate boundary | E6: exact-fused semantics costs ~10× more per update (loose FAC ≤ 1 bound); boundary ties caused 1 mismatch at 10⁶ before the tie-inclusive rule |
| 2026-09-27 | **G4: SDM Mode A and random-address SDM dropped from the core**; learned-address buckets (= IVF) kept as the sublinear index | E5: random addresses fail (R@64 ≤ 0.8 at 19% scanned); Mode A cleanup destroys partial cues at 10⁵ load; SDM prototypes ≈ kNN-bundle(50) |
| 2026-09-27 | **FAC re-ranks with a fused score** ½·normalized structural score + ½·fingerprint profile score | E4: MAC and FAC fail in complementary ways (role noise vs spurious additions); fusion ≥ both, 0.96–1.00 on discriminable groups |
| 2026-09-27 | Mode K integer-weighted fused kernel, TILE = 32 rows | 1.7–2× over the per-segment kernel; TILE 32 best among 32/64/256 |
| 2026-09-27 | Generator re-wiring check sorts commutative (`and`) arguments | E2 found ~7% of deep-ho FOR/MA variants were isomorphic to the base (label noise), which explained the deep-ho "ceiling" in E0 and E2 |

## Experiment log

| Exp | Date | Config | Headline | Link |
|---|---|---|---|---|
| E0 | 2026-09-27 | 1000 groups, 6 configs (naming × distractors) | fingerprint TA-top ≥ 0.995 at 0–10 distractors (after label-noise fix); MAC/lexical 0.000; unresolved vocab 0.55 | [results/E0](../results/E0/README.md) |
| bw | 2026-09-27 | 10⁶ random fingerprints, analogy profile | 1 query 22 ms (79% of 52 GB/s probe); batched 6.6 ms/query (4 cores) | [results/bw](../results/bw/) |
| E4 | 2026-09-27 | 5 operators + mixed × severity 1–4 | fused MAC+FAC ≥ both; 0.96–1.00 on discriminable groups; delete-fact collapse is intrinsic ambiguity | [results/E4](../results/E4/README.md) |
| E12 | 2026-09-27 | 1308 functions (Python + npm JS); 52 cross-language same-algorithm queries | fused ½FAC+½FP-literal MRR 0.168 vs lexical 0.089 / MAC 0.114; R@5 0.35 vs 0.10; strategy-level differences unsolved | [results/E12](../results/E12/README.md) |
| E11 | 2026-09-27 | 100 templates × 10 instances; 500 queries with a deleted fact | precision by support 0.07/0.48/0.59/0.74/0.86; m=5, support≥2: recall 0.646 vs 0.407 at equal precision 0.62 | [results/E11](../results/E11/README.md) |
| E10 | 2026-09-27 | scoring variants × fusion weight; case size (compose 1–3) × N (10⁴–10⁶) | normalization/IDF ≤1.5 pts; w=0.3 best; composed cases: 0.999 (10⁴), 0.999 (10⁵), 0.996 (10⁶) → degradation was an information limit | [results/E10](../results/E10/README.md) |
| E9 | 2026-09-27 | 1113 real Python functions from 3 packages; 33 cross-author same-algorithm queries; 365 category queries | fused FAC + literal FP: MRR 0.334 vs lexical 0.147 / MAC 0.188; category P@1 0.636 vs 0.578; syntax-level representation misses algorithm-level identity (quick/merge sort) | [results/E9](../results/E9/README.md) |
| E7 | 2026-09-27 | 100 templates × 60 noisy instances; severity 1/2 | schemas: deleted-fact recall 0.59 vs 0.42 (single best instance, M=10); schema precision 0.85–0.90 vs 0.54; purity ≥0.94 but fragmented (completeness 0.80) | [results/E7](../results/E7/README.md) |
| E6 | 2026-09-27 | 10⁴/10⁵/10⁶ live cases, 200 SQs, 5000 updates | 0 mismatches / 1150 checks; mean update 350 → 300 → 277 µs (flat in N); 680× → 21,000× cheaper than recompute | [results/E6](../results/E6/README.md) |
| E5 | 2026-09-27 | 100K corpus; 100 templates × 100 instances | k-means buckets R@64 1.000 at 0.47% scanned (random addresses ≤ 0.8); Mode A cleanup 0.02 vs 0.95 raw; SDM prototype 0.713 ≈ kNN-bundle 0.704 → SDM dropped | [results/E5](../results/E5/README.md) |
| E3 | 2026-09-27 | 10⁴/10⁵/10⁶ cases, 1000 queries, severity 0–2 | clean: fused acc@1 0.994 at 10⁶, TA in top-64 100%; MAC content vectors 0.35 at 10⁵; pipeline = exhaustive-FAC bound at 1/1100 the cost; partial analogues degrade with N (0.91→0.75) | [results/E3](../results/E3/README.md) |
| E2 | 2026-09-27 | 1000 groups, distractors 0/2/5/10 | entity corr. P≈1.0 R≈0.99; FAC TA-top 1.000; greedy = optimal; deleted-fact re-inference 0.99 (0.92 at d=10); 16–38 µs/pair | [results/E2](../results/E2/README.md) |
