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
