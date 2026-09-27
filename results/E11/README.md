# E11: inference quality and calibration with corroboration

Full tables: `E11-s1-m10.md`, `E11-s2-m10.md`. Reproduce with `mars-bench e11 --templates 100 --per 10 --severity {1,2}`.

**Setup:** memory holds 10 noisy instances of each of 100 hidden templates (1,000 cases; mixed perturbations; 2 distractors). There are 500 queries: fresh instances with one root higher-order fact deleted. `Engine::query` retrieves the top-m analogues (fused score), and candidate inferences are projected from each. An inference's **support** is the number of analogues proposing it. It is **true** if it is a fact of the query's undeleted instance.

## Results (severity 1; severity 2 in brackets)

| analogues m | accept if support ≥ | deleted-fact recall | precision | inferences / query |
|---|---|---|---|---|
| 1 | 1 | 0.407 [0.267] | 0.621 [0.390] | 0.66 |
| 3 | 1 | 0.726 [0.524] | 0.388 [0.259] | 1.87 |
| 3 | 2 | 0.479 [0.238] | 0.686 [0.673] | 0.70 |
| **5** | **2** | **0.646 [0.400]** | **0.621 [0.537]** | 1.04 |
| 5 | 3 | 0.487 [0.236] | 0.688 [0.697] | 0.71 |

Calibration at m = 5 (severity 1): precision by support.

| support | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|
| inferences | 833 | 157 | 162 | 115 | 59 |
| precision | 0.073 | 0.478 | 0.586 | 0.739 | 0.864 |

## Findings

1. **Corroboration is a well-calibrated confidence signal.** Precision rises monotonically with the number of supporting analogues, from 0.07 (a single analogue) to 0.86 (all five). Severity 2 shows the same trend (0.07 → 0.90). A lone analogue's inference is usually wrong; agreement across independently retrieved analogues is strong evidence.
2. **Corroboration is a Pareto improvement over the single-best-analogue policy.** Five analogues with support ≥ 2 keep the single-analogue precision (0.62) while recovering **59% more** deleted facts (0.646 vs 0.407). At severity 2 it improves both precision (0.54 vs 0.39) and recall (0.40 vs 0.27).
3. **Engine defaults updated:** inferences are drawn from the top-5 analogues, each as a separate JTMS justification. `Engine::corroborated(sq, min_support)` and `mars serve`'s `infer SQ [MIN_SUPPORT]` expose support as the confidence. This operationalizes the design's `Proposed` → `Corroborated` states (DESIGN §9) with measured semantics.
