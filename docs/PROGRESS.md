# Progress Tracker

This is the working document: current status, the task board, the decision log and the experiment log. Keep it short and current. Design rationale lives in [DESIGN.md](DESIGN.md), and experiment protocols in [EXPERIMENTS.md](EXPERIMENTS.md).

## Current focus

**Reliability of analogical judgments at scale.**
- H0–H4 hold on synthetic data. The engine is persistent and served (`mars serve`).
- Real code (E9, E12, E14) shows structure beats lexical retrieval, and vocabulary alignment is not the bottleneck there.
- Recent results are about *trusting* an analogy as memory grows:
  - corroboration is the calibrated confidence (E11, E15);
  - schemas complement instances but don't replace them (E15);
  - a local-null significance makes accept/abstain decisions memory-size invariant (E16).

Latest:
- Mode K kernel v2: 1.8× batched throughput, bit-identical results.
- Real-code inference works well above chance, though exact restoration is rare (E17).
- Representation granularity trades fingerprint strength against mapper strength, and different views restore different inferences (E18).

Candidate next steps:

## Task board

Legend: `[x]` done · `[~]` in progress · `[ ]` todo · `[-]` dropped

### P0 Foundations
- [x] Cargo workspace, `target-cpu=native`, release profile
- [x] `mars-hv`: packed HVs, bind/permute/Hamming, seeded RNG, i32 accumulator, **bit-sliced weighted majority** (property-tested against the accumulator, including ties)
- [x] `mars-rel`: interner, vocabulary + taxonomy, hash-consed expression DAG (commutative canonicalization, relational order), cases, s-expression loader and renderer
- [x] `mars-encode`: feature channels C0–C3, taxonomy multi-resolution features, IDF stats (epochs), segmented sketcher, profiles; solar/atom sanity tests
- [x] Popcount scan microbenchmark vs memory bandwidth: single query 79% of probe (G0 pass); batched 6.6 ms/query at 10⁶ rows on 4 cores → [results/bw](../results/bw/)
- [x] Mode K kernel: rows stored in word-interleaved blocks of 8 (one SIMD lane per row: no horizontal reductions, 1-uop `vpmuludq` weights), 4 queries share each row vector → batched scan 6.66 → 3.68 ms/query at 10⁶ (55 → ~30 cycles/row/query; ALU-port floor ≈ 21–28); E3 MAC stage 10.4 → 6.35 ms/query, identical accuracy; exact-id equivalence test vs a threshold-free reference (odd sizes, deletes, updates), portable path tested without AVX-512. [results/bw](../results/bw/bw-1000000.md)

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
- [x] **E13 analogical bootstrapping of vocabulary alignment** (`MapConfig::wildcard`, `Vocabulary::set_parents`) → [results/E13](../results/E13/README.md): unresolved vocabularies recovered without labels (pair precision 1.000; retrieval 0.15 → 0.99, oracle 1.00)
- [x] **E14 bootstrapped alignment on real code** (`MARS_RAW=1` converter mode; `mars-bench e14`) → [results/E14](../results/E14/README.md): frequent pairs learned (len↔length, append↔push, pop, abs; 0 wrong among hand-map-judgeable), recall 4/18 (Zipfian, small JS side); even oracle alignment barely moves retrieval (fused MRR 0.152 → 0.155): anonymous shapes already carry cross-language structure
- [ ] Alignment on knowledge graphs with independently developed schemas (dense parallel structure; where FAC is blocked by names)
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
- [x] **E20 near-miss learning** (`mars_gen::concept_instances`) → [results/E20](../results/E20/README.md): schema thresholds accept ~half of structural near-misses; hard Winston must-have rules fail under noise; soft emphasis weights (P(match|pos) − P(match|near-miss)) beat 1-NN with 1–2 near-misses (+0.04–0.09 balanced accuracy, 3 seeds, 2 severities), tie at 5
- [x] Near-miss emphasis in SAGE: `Generalization::near_misses`, `Sage::add_near_miss`, `Sage::diagnostic`, `sage::Diagnostic::{train, score, accepts}` (E20's classifier D now runs through it; identical results; unit test)
- [x] **E15 schema-level retrieval vs corroboration** → [results/E15](../results/E15/README.md): schemas *complement* instances (instances + schemas best at 10³ and 10⁴; R@P≥0.6 0.45–0.48 vs 0.39 at 10⁴); schema-only pools collapse at scale (purity 0.96 → 0.87 at fixed θ); schema fact probability is uncalibrated, corroboration is; coverage assimilation negative
- [x] **E16 match significance** → [results/E16](../results/E16/README.md): a local-null z (top-1 vs the tail of its own candidate list) keeps precision 0.90–0.95 from 10³ to 10⁶ cases with a fixed threshold; raw scores fall to 0.44 and accept 79% of absent-template queries. `Engine::query_significance`, `SIGNIFICANT_Z`, `serve` abstains
- [x] E16 addendum → [results/E16](../results/E16/README.md#addendum-can-recall-at-10-be-recovered): recall at 10⁶ is information-limited (local z ≈ E-value over the whole memory); sibling pooling (Stouffer) recovers +3 points (0.492 → 0.523 at precision 0.945); sibling count alone is uninformative
- [x] Local-null assimilation gate for SAGE (`SageConfig::min_z`) → [E15 addendum](../results/E15/README.md#addendum-significance-gated-assimilation-after-e16): purity invariant with memory size (0.967 → 0.978 vs 0.964 → 0.868), completeness falls; small gain in the combined pool at 10⁴
- [x] Indexed SAGE candidate scan: Mode K index over the pool (generalizations + outliers) maintained on every pool mutation, exact re-scoring of candidates with the old tie-break; E7/E15 reproduce exactly; build 0.74 → 0.56 ms/case at 10⁴ (coverage 1.17 → 0.76); 10⁵-case pool built in 171 s. `Sage::push_outlier` replaces direct writes to `outliers`
- [ ] Hierarchical generalization to reduce fragmentation
- [x] P7a program analogy on real code: `tools/py2mars.py` (Python AST → relational cases, one-level helper inlining), `tools/fetch_e9_corpus.sh` (3 MIT PyPI packages), E9 → [results/E9](../results/E9/README.md)
- [x] **E17 candidate inference on real code** → [results/E17](../results/E17/README.md): deleted control statements restored exactly for 7.6% (lexical 4.6%, random 0.1%); shape overlap 0.243 vs 0.180 lexical vs 0.026 chance; the structurally nearest function beats the same algorithm by another author as an inference source; support calibrated (0.02 → 0.09 → 0.33)
- [x] **E18 representation granularity (nested vs flat blocks)** → [results/E18](../results/E18/README.md): flattening improves FAC retrieval +25% but weakens the fingerprint (best fusion weight 0.3 → 0.7; best-vs-best equal); nested restores more statements (0.189 vs 0.106), flat more placements; union restores 0.136 in context (1.7×)
- [x] **E19 multi-view cases** (`mars_rel::views::FlatView`) → [results/E19](../results/E19/README.md): score-level fusion of views does not beat the best single view; the flat view is the better cross-language retrieval view (MRR 0.198 vs 0.141); views have *roles* (rank/map in flat, project from nested: top-1 precision 0.637 vs 0.555; support ≥ 3 recall 0.089 vs 0.071); cross-view agreement is weak corroboration (views of one analogue are not independent)
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
| 2026-09-27 | Concept memory: per-fact *emphasis* from near-misses (soft), not hard must-have conditions | E20: hard rules ≈ no gain under noise; soft emphasis beats 1-NN at 1–2 near-misses |
| 2026-09-27 | Corroboration counts independent analogues only (not views of one analogue); multi-view kept as an application pattern (rank/map in one view, project from another), not an engine feature | E19: cross-view agreement precision ≈ single view (0.58–0.60) vs 0.75–0.88 across analogues; role split gains are modest |
| 2026-09-27 | Fusion weight is a property of the representation (code nested: 0.3; flat: 0.7), not a global constant | E18 |
| 2026-09-27 | Mode K storage: word-interleaved 8-row blocks; `score_pair` for scoring stored fingerprints | 1.8× batched throughput, bit-identical results; single-row `score()` now strided, so the engine scores its contiguous copies |
| 2026-09-27 | Accept/abstain on analogues by local-null z (top-1 vs lower half of the fingerprint candidate list), threshold 9, never by raw score | E16: raw-score thresholds lose precision 0.90 → 0.44 from 10³ to 10⁶ cases; local z holds 0.90–0.95; Gaussian global E-value fails |
| 2026-09-27 | Consolidation adds schema cases alongside instances (never replaces them); confidence = corroboration over the combined pool; schema fact probabilities reported, not used as confidence | E15: combined pool best at every scale; schema-only pools lose at 10⁴ (purity drop); probability buckets flat |
| 2026-09-27 | SAGE names from stable ids + per-pool namespace | E15 exposed name collisions (vector index reused after merges; two pools in one KB) |
| 2026-09-27 | Keep anonymization of unresolved predicates as the default fallback; treat bootstrapped alignment as a precision tool for mapper-bound settings | E14: oracle alignment adds only +0.003 fused MRR on real code and lowers fingerprint-analogy MRR; learned alignment reaches the same FAC gain |
| 2026-09-27 | Vocabulary alignment by analogical bootstrapping: wildcard mapping + mutual-best + one-per-domain constraint + EM re-estimation | E13: the constraint prevents transitive collapse; re-estimation prevents entrenchment; together they reach oracle-level retrieval |
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
| E20 | 2026-09-27 | ~100 concepts, 10 positives, k ∈ {0,1,2,5} re-wired near-misses, 5+5 test; severities 1–2, seeds 1–3 | balanced acc. at k = 2: threshold 0.713, hard rules 0.705, 1-NN 0.679, emphasis 0.758; k = 5: 1-NN 0.781, emphasis 0.774 | [results/E20](../results/E20/README.md) |
| E19 | 2026-09-27 | both views of 1,308 functions; retrieval (33 other-package, 52 other-language queries); 700 inference queries | best single view (flat 0.7) beats all view fusions (0.375 / 0.198 MRR); role split: top-1 precision 0.637 vs 0.555, support ≥ 2 recall 0.124 vs 0.116 at 0.77 precision; cross-view agreement precision 0.58–0.60 | [results/E19](../results/E19/README.md) |
| E18 | 2026-09-27 | E9 corpus in nested vs flat-block form; E9 task A retrieval; E17 inference with the same deleted core | retrieval best-vs-best 0.359 (nested, w=0.3) vs 0.370 (flat, w=0.7); FAC 0.283 → 0.354; inference top-1 core 0.189 vs 0.106, in context 0.079 vs 0.091, union 0.136 | [results/E18](../results/E18/README.md) |
| E17 | 2026-09-27 | 700 deleted-statement queries over 1,113 Python functions; fused/lexical/random/oracle analogues; top-5 corroboration | fused top-1: exact 0.076, shape overlap 0.243 (lexical 0.046/0.180, random 0.001/0.026, other-author twin 0.015/0.206); precision by support 0.022/0.092/0.333 | [results/E17](../results/E17/README.md) |
| E16 | 2026-09-27 | 5 instances/template, 10³–10⁶ cases, 500 queries (half with absent templates); thresholds fixed at 10³ | fused local-null z: precision 0.90 / 0.91 / 0.91 / 0.95, absent false-accept 0.10 → 0.01 (recall 0.93 → 0.49); fused raw score: precision 0.90 → 0.44, false-accept 0.79 | [results/E16](../results/E16/README.md) |
| E15 | 2026-09-27 | 10 instances × 100 / 1,000 templates, severity 1–2; pools: instances, schemas (symmetric / coverage), instances + schemas | instances + schemas best (10⁴: R@P≥0.6 0.448 vs 0.392, m=1 precision 0.60 vs 0.55); schema-only wins only at 10³ m=1 (recall 0.574 vs 0.374) and collapses at 10⁴ (purity 0.87) | [results/E15](../results/E15/README.md) |
| E14 | 2026-09-27 | E12 corpus in raw mode (359 language-specific call predicates, 18 gold py↔js pairs), 52 queries | learned 14 pairs: 4 hand-map-correct, 2 same identifier, 0 judgeable-wrong (≈0.6 precision by inspection); fused MRR 0.152 → 0.153 (oracle 0.155, hand-mapped corpus 0.168) | [results/E14](../results/E14/README.md) |
| E13 | 2026-09-27 | unresolved vocab (12 domains × 54 relations), 250–1000 groups, 2–10 distractors | 1000 groups: alignment precision 1.000, fused TA-top 0.151 → 0.993 (oracle 1.000); ≥ 0.938 in all settings | [results/E13](../results/E13/README.md) |
| E12 | 2026-09-27 | 1308 functions (Python + npm JS); 52 cross-language same-algorithm queries | fused ½FAC+½FP-literal MRR 0.168 vs lexical 0.089 / MAC 0.114; R@5 0.35 vs 0.10; strategy-level differences unsolved | [results/E12](../results/E12/README.md) |
| E11 | 2026-09-27 | 100 templates × 10 instances; 500 queries with a deleted fact | precision by support 0.07/0.48/0.59/0.74/0.86; m=5, support≥2: recall 0.646 vs 0.407 at equal precision 0.62 | [results/E11](../results/E11/README.md) |
| E10 | 2026-09-27 | scoring variants × fusion weight; case size (compose 1–3) × N (10⁴–10⁶) | normalization/IDF ≤1.5 pts; w=0.3 best; composed cases: 0.999 (10⁴), 0.999 (10⁵), 0.996 (10⁶) → degradation was an information limit | [results/E10](../results/E10/README.md) |
| E9 | 2026-09-27 | 1113 real Python functions from 3 packages; 33 cross-author same-algorithm queries; 365 category queries | fused FAC + literal FP: MRR 0.334 vs lexical 0.147 / MAC 0.188; category P@1 0.636 vs 0.578; syntax-level representation misses algorithm-level identity (quick/merge sort) | [results/E9](../results/E9/README.md) |
| E7 | 2026-09-27 | 100 templates × 60 noisy instances; severity 1/2 | schemas: deleted-fact recall 0.59 vs 0.42 (single best instance, M=10); schema precision 0.85–0.90 vs 0.54; purity ≥0.94 but fragmented (completeness 0.80) | [results/E7](../results/E7/README.md) |
| E6 | 2026-09-27 | 10⁴/10⁵/10⁶ live cases, 200 SQs, 5000 updates | 0 mismatches / 1150 checks; mean update 350 → 300 → 277 µs (flat in N); 680× → 21,000× cheaper than recompute | [results/E6](../results/E6/README.md) |
| E5 | 2026-09-27 | 100K corpus; 100 templates × 100 instances | k-means buckets R@64 1.000 at 0.47% scanned (random addresses ≤ 0.8); Mode A cleanup 0.02 vs 0.95 raw; SDM prototype 0.713 ≈ kNN-bundle 0.704 → SDM dropped | [results/E5](../results/E5/README.md) |
| E3 | 2026-09-27 | 10⁴/10⁵/10⁶ cases, 1000 queries, severity 0–2 | clean: fused acc@1 0.994 at 10⁶, TA in top-64 100%; MAC content vectors 0.35 at 10⁵; pipeline = exhaustive-FAC bound at 1/1100 the cost; partial analogues degrade with N (0.91→0.75) | [results/E3](../results/E3/README.md) |
| E2 | 2026-09-27 | 1000 groups, distractors 0/2/5/10 | entity corr. P≈1.0 R≈0.99; FAC TA-top 1.000; greedy = optimal; deleted-fact re-inference 0.99 (0.92 at d=10); 16–38 µs/pair | [results/E2](../results/E2/README.md) |
