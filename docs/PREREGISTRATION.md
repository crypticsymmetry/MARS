# Pre-registration: external evaluation of MARS v0.1

Written 2026-09-28, **before** any test-set run of the evaluations below. It is committed together with the evaluation harnesses. Any deviation from this plan is reported in the results as a deviation, with its reason.

## Frozen system

- **Version:** commit `78fdc34` (tagged `v0.1` locally; this session can push only its branch, so the tag is not on GitHub yet), the commit that passed CI (run 3) before this document was added.
- **Frozen components:** the engine, encoder, index and mapper crates (`mars-hv`, `mars-rel`, `mars-encode`, `mars-index`, `mars-map`, `mars-tms`, `mars-engine`) and the Python front end `tools/py2mars.py`.
- **Allowed after the tag:** evaluation harnesses (data loading, conversion adapters, metrics) and their baselines, written before test results are seen.
- **Bugs.** If a bug in a frozen component is found during evaluation, it is *reported*, not silently fixed. Results with and without a fix are labelled.

## Selection rules

- **Settings.** Every tunable setting is chosen on validation data only, from the grids listed here, by the stated primary metric. The test split is run **once** per evaluation with the chosen settings. Nothing is tuned on test.
- **Seeds.** All random sampling is seeded (seed 1 unless stated); the seed is recorded in each result.
- **Statistics.** Paired 95% bootstrap CIs and two-sided sign-flip permutation tests (`tools/stats.py`, 10,000 resamples). A comparison is reported as significant only if the CI of the paired difference excludes 0. All pre-registered comparisons are reported, whatever the outcome.

---

## EV1: knowledge-graph link prediction (FB15k-237, WN18RR)

**Harness:** `mars-bench ev1` (`crates/mars-bench/src/ev1.rs`).
- A sanity run on 100–200 *validation* triples was made before this document was committed, to check the harness and its cost: WN18RR MRR ≈ 0.09 and FB15k-237 ≈ 0.11 on those samples. No test data was used.

**Data:**
- **FB15k-237:** 14,541 entities, 237 relations; 272,115 / 17,535 / 20,466 train / valid / test triples.
- **WN18RR:** 40,943 entities, 11 relations; 86,835 / 3,034 / 3,134 triples.
- **Source:** standard splits from `github.com/villmow/datasets_knowledge_embedding`, fetched 2026-09-28. For WN18RR the `original/` (numeric synset id) files are used; the repo's `text/` variant has 41,105 distinct names, not the standard 40,943 entities.
- **Checksums** (SHA-256 prefixes, `data/external/kgc/`): FB15k-237 test `e2e35e8e6113de22`, train `749cbe9d923bac7b`, valid `a35aca3c963b71b8`; WN18RR (`WN18RR-original/`) test `0383bceaaa1096cf`, train `038612e783c215ee`, valid `453ce7202afa5809`.

**Protocol:** the standard filtered link-prediction protocol.
- For every test triple (h, r, t), predict t from (h, r, ?) and h from (?, r, t) (tail and head prediction).
- Rank against all entities, filtering the other true answers from train ∪ valid ∪ test.
- **Metrics:** MRR, Hits@1, Hits@3, Hits@10, averaged over both directions. **Primary: MRR.**
- **Unproposed entities.** MARS ranks only the entities its inferences propose. Every entity it does not propose gets the expected rank under random ordering after the proposed ones: rank = P + (U + 1) / 2, where P is the number of proposed non-filtered candidates and U is the number of remaining non-filtered entities.

**MARS memory:**
- **Entity cases.** One case per entity, built from **training triples only**: (r e x) for each (e, r, x) and (r⁻¹ e y) for each (y, r, e).
- **Names.** Relations and entities are renamed to opaque ids. MARS sees no names, only identity; KGE baselines likewise.
- **Cap.** Cases are capped at **100 facts**, keeping the facts of the rarest relations first (ties by entity id). The cap is set in advance, for cost.
- **Queries.** A query for (h, r, ?) uses h's case as the query; h itself is excluded from its analogues. Head prediction uses the inverse relation on t's case.
- **Candidates** are the objects of the inferences (r h y) projected from the top-k analogues (`first_order` inferences): substitutions and copies.

**Ranking:**
- by learned reliability × Σ fused analogue score, as in E30;
- transfer reliability is learned by feedback on the top-3 suggestions of every **validation** query, in a seeded order, then **frozen** for test (no test feedback);
- the raw ranking (Σ fused score, no reliability) is reported too.

**Settings chosen on validation**, on a seeded sample of 2,000 validation triples per dataset, both directions, by MRR of the **raw** ranking (learned reliability needs validation feedback of its own, so it is not used for selection):

| setting | options |
|---|---|
| fingerprint profile | `analogy`, `literal` |
| identity weight λ | 0, 0.5, 0.85 |
| analogues k | 10, 30 |

FAC weight 0.5 and fingerprint shortlist 50 are fixed, as in E27–E32.

**Baselines:**
- **Computed:**
  - relation popularity: rank entities by frequency as the object of r in train;
  - MARS raw ranking (no learned reliability).
- **Published, same filtered protocol** (quoted, not rerun):

  | method | FB15k-237 MRR / H@1 / H@10 | WN18RR MRR / H@1 / H@10 |
  |---|---|---|
  | DistMult (Dettmers et al. 2018) | .241 / .155 / .419 | .43 / .39 / .49 |
  | ComplEx (Dettmers et al. 2018) | .247 / .158 / .428 | .44 / .41 / .51 |
  | RotatE (Sun et al. 2019) | .338 / .241 / .533 | .476 / .428 / .571 |
  | TuckER (Balažević et al. 2019) | .358 / .266 / .544 | .470 / .443 / .526 |

  Note: published numbers come from the cited papers and may differ in minor protocol details, e.g. how ties are broken.

**Pre-registered analyses:**
- **A1:** overall filtered MRR / Hits@k on test, for MARS learned and raw.
- **A2, split by answer location:** is the answer entity within 2 hops (undirected) of the query entity in the training graph? Metrics per subset.
- **A3:** learned vs raw ranking (paired, per query, on reciprocal rank).
- **A4:** MARS vs relation popularity (paired).

**Predictions** (made before running):
- **P1.** MARS is **below** RotatE and TuckER in overall MRR on both datasets. MARS does not do new-value prediction well (E31), and much of link prediction is exactly that.
- **P2.** MARS is **above** relation popularity on both datasets (A4).
- **P3.** On the within-2-hops subset, MARS's Hits@1 is much higher than on the rest (A2), as in E31.
- **P4.** Learned ≥ raw (A3), with a small effect (E30: +0.008 to +0.015 Hits@1).
- **P5** (weak). MARS is relatively closer to the KGE baselines on WN18RR than on FB15k-237: WN18RR's relations are few and structural (hypernymy, derivation), and relational transfer suits them.

---

## EV2: code retrieval by algorithm (CodeNet Python800)

**Data:**
- Project CodeNet Python800: 800 problems × 300 accepted Python solutions each (IBM, 2021).
- Fetched 2026-09-28 from the Hugging Face mirror `qiankunmu/Project_CodeNet_Python800_and_Java250`, `Project_CodeNet_Python800.tar.gz`, SHA-256 prefix `39297d11df8030ce`, into `data/external/codenet/`.
- The original IBM CDN link returns 502.

**Sample** (seed 1):
- 200 problems, drawn at random from the 800;
- 30 solutions per problem, drawn at random among those that parse and encode to at least 5 facts. If fewer qualify, take all of them.

**Front end.** `tools/py2mars.py` is frozen. CodeNet solutions are scripts, so an adapter wraps each file's top-level statements, except function and class definitions, into one synthetic function. Module functions are inlined one level, as py2mars does for helpers. This adapter is written before any results.

**Task:** every program is a query; the pool is all other programs in the sample. Relevant = same problem.
- **Metrics:** MAP@R, where R is the number of other solutions to the same problem, typically 29 (**primary**); also Hits@1 and MRR.

**Methods** (no tuning: settings are the E9 defaults, frozen):
- **MARS fused:** fingerprint-literal top-100, re-ranked by ½FAC + ½FP. This is E9's best configuration, except that it re-ranks a shortlist instead of exhaustively. That is necessary at this pool size, and fixed in advance.
- **MARS fingerprint-literal only.**
- **Lexical TF-IDF** over code tokens (identifiers, keywords, operators; Python `tokenize`).
- **Code embedding:** `jinaai/jina-embeddings-v2-base-code` via fastembed, if available. Otherwise `BAAI/bge-small-en-v1.5`, with the choice recorded.
- **RRF(MARS fused, lexical)** (secondary).

**Pre-registered analyses:**
- **B1:** MAP@R, Hits@1 and MRR for each method.
- **B2:** paired MARS fused vs lexical.
- **B3:** paired MARS fused vs code embedding.

**Predictions:**
- **P6.** MARS fused > lexical TF-IDF on MAP@R (the E9 direction).
- **P7.** Code embedding ≥ MARS fused on MAP@R. MARS's representation is syntactic, and different strategies for the same problem are common (E9).
- **P8.** RRF(MARS, lexical) ≥ both components.

---

## EV3 (secondary): E33 with stronger baselines

This reruns the E33 incident-response stream (seeds 1–5, noise 2) with:
- an **embedding-RAG** memory: episodes retrieved by bge-small cosine over their facts rendered as text, with the same mapper and feedback as MARS;
- the **LLM agent** of E33 given episodes from MARS, from embedding RAG and from name recall, with per-incident outcomes saved. It runs on 150 incidents per memory, GLM-5.3-Flash, reasoning low, temperature 0.

E33's task is synthetic and was designed by us, so EV3 checks *baseline strength*, not external validity.

**Prediction P9.** MARS > embedding RAG on fix@1; an LLM given MARS-retrieved episodes > an LLM given embedding-retrieved ones.

---

## Reporting

- **Where:** results go in `results/EV1`, `results/EV2` and `results/EV3`, with configs, seeds, checksums, chosen settings and the validation grid.
- **Scoring:** each prediction P1–P9 is marked **confirmed / not confirmed / inconclusive**.
- **Deviations** are listed at the top of each results README.

---

## Addendum A (2026-09-28, after EV1–EV3): E34, a dataflow front end for code

Written before any E34 run.

**Motivation.** EV2 found MARS's syntactic code representation (`tools/py2mars.py`) far below a pretrained code embedding: MAP@R 0.211 vs 0.473. E34 tests the first representation upgrade: a **dataflow front end**, `tools/py2pdg.py`.
- Expressions are flattened into single-operation facts over value entities: each operation's result is a fresh value.
- Variables resolve to their reaching definition. Loop-carried values get explicit `(phi new before in-loop)` facts, so accumulators and swaps become visible as dataflow.
- Control structure is kept as loop and guard context.
- Structure is thereby independent of variable names, expression nesting and statement grouping.

**Development and test separation.**
- **Development:** all front-end changes and settings use a *development sample* of 200 CodeNet Python800 problems drawn with seed 2 from the 600 problems *not* in EV2's sample, with the same sampling and skip rules as EV2.
- **Test:** EV2's sample (seed 1), run once with the final front end.
- The retrieval pipeline is EV2's (frozen v0.1 components, ½FAC + ½FP-literal over the FP top-100); only the front end changes.

**Arms** (test, MAP@R primary, paired over queries):
- **A1:** MARS with py2pdg vs MARS with py2mars (EV2's number, recomputed on the same programs).
- **A2 (secondary, division of labour):** the code embedding's top-100 re-ranked by ½ embedding cosine + ½ MARS FAC (the best front end on development), vs the code embedding alone. The weights are fixed here, not tuned.

**Predictions:**
- **P10:** py2pdg > py2mars on MAP@R.
- **P11:** MARS with py2pdg is still below the code embedding.
- **P12** (weak): embedding + MARS re-rank ≥ embedding alone.

Scored with `tools/ev_report.py`'s criteria ("A > B": CI of the difference above 0; "A ≥ B": point estimate ≥ 0 and CI not entirely below 0).

**Outcome (added after the test run):** P10, P11 and P12 confirmed; deviations (per-value context facts dropped on development; names kept as attributes; a harness bug fixed before any test metric) are listed in [results/E34](../results/E34/README.md).

## Addendum B (2026-09-28, after E34 and E35): E36, learned fusion and conformal abstention for code retrieval

Written before any E36 run on either sample.

**Motivation.** E34 found that re-ranking the code embedding's top-100 with MARS by a fixed ½ cosine + ½ FAC improves MAP@R (0.473 → 0.498). The gain is concentrated where the embedding is least confident (an exploratory split). E36 tests two upgrades:
1. a *learned*, query-dependent fusion;
2. *conformal* abstention: answer with a top-1 program only when a calibrated confidence guarantees precision.

**Data.** Candidates: the code embedding's top-100 per query, as in E34. Features: MARS FAC from E34's final py2pdg front end.
- **Development** (fitting and calibration): E34's development sample (seed 2, 200 problems outside EV2's; 5,970 programs).
- **Test:** EV2's sample (5,982 programs), run once.
- Nothing is fit or tuned on test.

**Arms.**
- **A3, learned fusion.** A pointwise logistic-regression ranker over (query, candidate) pairs of the development sample, predicting "same problem". Features:
  - candidate: cosine, FAC, cosine rank, FAC rank within the list;
  - query-level: top-1 cosine, top-1 minus top-2 cosine margin, maximum FAC over the list;
  - the interactions of cosine and FAC with top-1 cosine.

  Features are standardized on development; L2 regularization C = 1; no other tuning. Candidates are ranked by predicted probability. Compared with E34's fixed ½ cosine + ½ FAC (B) and with the embedding alone (E).
- **A4, conformal selective top-1.** Confidence = the A3 model's probability for its top-1 candidate.
  - Calibration: Learn-then-Test with fixed-sequence testing over thresholds (from high confidence down, a grid of 200 quantiles of the development confidences) and exact binomial p-values.
  - Target: precision among answered queries ≥ 1 − α with probability ≥ 1 − δ; α = 0.05, δ = 0.1.
  - Calibration uses all development queries. The chosen threshold is applied unchanged to test.
  - Baseline: the same procedure with the embedding's top-1 cosine as the confidence (embedding top-1 answered).

**Predictions** (test; criteria of `tools/ev_report.py`):
- **P13:** A3 > B on MAP@R (paired over queries).
- **P14:** A3's test precision among answered queries is ≥ 0.95. The guarantee assumes exchangeable queries. Problems are drawn at random from the same pool for both samples, but queries within a problem are correlated, so this is an empirical check of the guarantee. Scored "confirmed" if the point estimate is ≥ 0.95, and "inconclusive" if the point estimate is below 0.95 but its 95% CI includes 0.95.
- **P15:** at the same guarantee, coverage (the fraction of queries answered) with A3's confidence > coverage with the embedding's cosine. Paired bootstrap over queries of the difference in answered indicators; "A > B".

Scored after the test run with the same "A > B" / "A ≥ B" rules as P1–P12.
