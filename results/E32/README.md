# E32: instance retrieval for new-value prediction — an identity channel in the engine

Tables: `E30-{scientists,films}-identity-<λ>.md` / `.json` (config and seed included). The exploratory support-conditioned runs are in [support-conditioned/](support-conditioned/); that code is in git history (commit "Engine identity channel; support-conditioned reliability (exploratory, E32)").

Reproduce (as E30, plus the identity weight):
```
mars-bench e30 --data OUT/sci2 --tag scientists-identity-0.85 --identity 0.85
mars-bench e30 --data OUT/films2 --tag films-identity-0.5 --identity 0.5 --relations wdt:p57,wdt:p161,wdt:p58,wdt:p162,wdt:p86,wdt:p344,wdt:p136,wdt:p495,wdt:p364,wdt:p272
```

**Question:** E31 found that most KG completion errors are *new-value* predictions: the answer is an entity the query doesn't mention, so only copying from similar entities can reach it. E27 found exact identity overlap (shared universities, countries, awards) to be a better neighbour signal for copying than entity-anonymous fingerprints. Does adding identity to the engine's retrieval fix the weak half? And a second question: does conditioning learned reliability on support help rank copies against substitutions?

**What was built.**
- **`mars_engine::identity`:** a TF-IDF index over entity and predicate names.
  - One vector per case, with an inverted index for candidate generation.
  - Maintained incrementally on every add, remove and fact change.
  - IDF frozen at construction like the fingerprint epoch, persisted as `identity.idf` and reloaded by `Engine::open`.
- **`EngineConfig::identity_weight` λ:** one-off retrieval (`query`, `infer`) scores (1 − λ)·fused + λ·identity cosine over the union of fingerprint and identity candidates. Standing queries stay fingerprint-based, because their incremental bound relies on the fingerprint score.
- Also `Engine::with_epochs` and `Engine::identity_score`, and Python `Engine(..., identity_weight=λ)`.

**Protocol.** The E30 stream, identical except for λ ∈ {0, 0.3, 0.5, 0.7, 0.85, 1}. The identity IDF is fitted on the memory only.

## Results (Hits@1)

| λ | scientists: raw / learned | answer in query (learned) | new value (raw) | films: raw / learned | answer in query (learned) | new value (raw) |
|---|---|---|---|---|---|---|
| 0 (E30) | 0.401 / 0.409 | 0.746 | 0.272 | 0.360 / 0.375 | 0.826 | 0.102 |
| 0.3 | 0.413 / 0.420 | 0.744 | 0.288 | **0.369 / 0.378** | 0.826 | 0.110 |
| 0.5 | 0.420 / 0.424 | 0.751 | 0.292 | 0.367 / **0.379** | 0.828 | 0.111 |
| 0.7 | 0.429 / 0.435 | 0.758 | 0.308 | 0.366 / 0.375 | 0.820 | 0.108 |
| **0.85** | **0.439 / 0.445** | **0.762** | **0.319** | 0.361 / 0.372 | 0.809 | 0.112 |
| 1 (identity only) | 0.425 / 0.424 | 0.744 | 0.297 | 0.329 / 0.341 | 0.728 | 0.108 |

With induced rules added (E31 ranking), the best settings give 0.442 (scientists, λ = 0.85) and 0.382 (films, λ = 0.5).

**Support-conditioned reliability (exploratory, reverted).** Reliability estimated per (transfer type, support bucket), backing off to the type, with a calibrated noisy-or ranking:
- learned ranking: 0.409 → 0.407 (scientists), 0.375 → 0.373 (films);
- calibrated ranking: 0.400 / 0.375;
- no gain on new-value queries.

## Findings

1. **Identity fixes part of the weak half, and the gain depends on the domain.**
   - On scientists, fusing identity (λ = 0.85) raises Hits@1 from 0.409 to 0.445 (+0.036). New-value prediction goes from 0.272 to 0.319, and relational inference also benefits (0.746 → 0.762), because better neighbours bring better substitutions too.
   - On films the gain is small (+0.004 learned at λ = 0.3–0.5). Film neighbours share few specific entities that predict the missing ones: cast and crew are mostly unique.
2. **Fusion, not replacement.** Identity alone (λ = 1) is worse than the fusion in both domains, and much worse on films (0.341 vs 0.375). Structure (fingerprint + FAC) still carries role information that names do not. As with the FAC weight (E9, E10), the best mix depends on the domain: λ ≈ 0.85 for scientists, ≈ 0.3–0.5 for films.
3. **Calibration is not the bottleneck for new values.** From the query alone, a new-value query looks exactly like one whose answer is present. So any global calibration, even conditioned on support, still ranks a plausible substitution above copies, and support conditioning mostly fragments the evidence. Improving new-value prediction needs better candidates, which the identity channel gives on scientists, not better weighting.
4. **What remains.** New-value prediction on films stays near 0.11: which cast member or genre a film has is essentially recommendation, beyond neighbour copying. The next step for that half would be a learned recommender or embeddings as an external signal, fused like the text signals in E24/E25. MARS's own contribution remains relational inference with calibrated, explained transfers.
