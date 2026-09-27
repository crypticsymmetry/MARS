# E10: why do partial analogues degrade with corpus size?

E3 found that fused accuracy on perturbed (partial) analogues falls with corpus size (severity 1: 0.912 at 10⁴ → 0.859 at 10⁵ → 0.745 at 10⁶), while the pipeline still matches the exhaustive-FAC bound. So the question is whether the fault lies in the *scoring function* or in the *data*. Reproduce with `mars-bench e10 --groups G --ops all --severity s [--compose c]`. Full tables are in `E10-*.md`.

## 1. Scoring variants barely matter (10⁵ cases, mixed perturbations, severity 1)

Candidates are the Mode K top-64 (TA among them for 93.4% of queries), re-ranked by `w·norm(S) + (1−w)·fp`.

| mapper | normalization | w=0 | **w=0.3** | w=0.5 | w=0.7 | w=1 |
|---|---|---|---|---|---|---|
| plain | geometric √(Sqq·Scc) | 0.769 | **0.874** | 0.859 | 0.798 | 0.731 |
| plain | Dice | 0.769 | 0.874 | 0.859 | 0.799 | 0.731 |
| plain | query coverage | 0.769 | 0.862 | 0.841 | 0.767 | 0.696 |
| plain | candidate coverage | 0.769 | 0.874 | 0.846 | 0.781 | 0.681 |
| IDF-weighted local scores | geometric | 0.769 | 0.869 | 0.858 | 0.809 | 0.730 |

- The normalization choice and IDF (informativeness) weighting of match hypotheses change accuracy by ≤ 1.5 points.
- **The fusion weight matters: w = 0.3 is best at every severity** (severity 2: 0.692 vs 0.645 at w = 0.5; clean: tied at 0.999). **The default changes from 0.5 to 0.3.**

## 2. Failures come from chance matches in the background, not from the foils

Of the 141 failures of the default (w = 0.5), ≈ 120 are won by cases from *other groups*: RND 67, other groups' LS/base/TA 37, and FOR/MA 18. Only 19 are the query's own MA/FOR foils. In 66 failures, TA was not among the 64 candidates at all. A small structure (3–8 facts), damaged by a perturbation, is simply **not unique** among 10⁵ structures drawn from the same generator.

## 3. Information per case is the lever: compose templates

`--compose c` builds each case from c templates sharing one entity. Severity scales with c, so the *relative* damage is constant.

| templates per case | severity | 10⁴ | 10⁵ | 10⁶ | MAC ceiling at 10⁵ |
|---|---|---|---|---|---|
| 1 | 1 | 0.919 | 0.874 | (E3: 0.745 at w=0.5) | 0.934 |
| **2** | **2** | **0.999** | **0.999** | **0.996** | 1.000 |
| 3 | 3 | 1.000 | 1.000 | — | 1.000 |
| 2 | 4 (double relative damage) | — | 0.961 | — | 0.991 |

(Values are for w = 0.3, plain geometric normalization.)

## Findings

1. **The E3 degradation is an information limit of tiny cases, not a flaw in retrieval or scoring.** With two templates' worth of structure per case (≈ 10–16 facts), partial analogues are retrieved with ≥ 0.996 accuracy up to 10⁶ cases, *flat* in corpus size. Single-template cases (≈ 3–8 facts) are small enough that chance structural matches accumulate as the corpus grows.
2. **Design implication:** a MARS *case* should carry enough relational structure, roughly ≥ 10 structurally connected facts, to be identifiable at 10⁶ scale. This informs the case-segmentation policy for large knowledge graphs (DESIGN §5.4): prefer larger neighbourhoods, or contexts, over minimal episodes.
3. Normalization and IDF weighting are not worth their complexity here. The IDF option is kept in `MapConfig::pred_weights` (off by default) for real vocabularies, where predicate frequencies are far more skewed than in the generator.
