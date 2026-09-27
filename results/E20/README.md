# E20: near-miss learning — what matters in a concept?

Tables: [E20-s1.md](E20-s1.md), [E20-s2.md](E20-s2.md). Reproduce with `mars-bench e20 [--severity 1|2] [--seed s]` (< 1 s). Data: `mars_gen::concept_instances`.

**Question:** analogical generalization (SAGE, E7/E15) learns from positives only. A schema-similarity threshold then cannot reject a *near-miss*: a case with the same content but the wrong structure. Can MARS learn what matters from a few labeled near-misses, as in Winston's arch learning and analogy-based concept learning with near-misses?

**Setup.**
- ~100 concepts (hidden templates; severity 1, 2 distractors).
- Positives are noisy instances. Near-misses are noisy instances of a *fresh random re-wiring* of the template: identical first-order facts, different causal (higher-order) structure. Every near-miss is re-wired differently.
- Per concept: a SAGE schema from 10 training positives (mean 9.6 assimilated), k ∈ {0, 1, 2, 5} training near-misses, and 5 + 5 fresh test cases.
- Metric: balanced accuracy.

Classifiers:
- **A, schema threshold:** accept iff normalized FAC(schema → x) ≥ that of the lowest training positive.
- **B, hard critical facts (Winston "must-have"):** schema facts missing from some near-miss's mapping but present in ≥ 80% of positives must be matched, as well as every training positive matches them.
- **C, 1-NN:** nearest neighbour by normalized FAC over all labeled training cases.
- **D, emphasis weights (soft Winston):** each schema fact is weighted by P(matched | positive) − P(matched | near-miss). A case scores its weighted matched fraction; the threshold sits midway between the training classes' mean scores.

## Results

Balanced accuracy (true-positive rate / true-negative rate), severity 1, seed 1:

| k near-misses | A: threshold | B: hard critical facts | C: 1-NN | **D: emphasis weights** |
|---|---|---|---|---|
| 0 | 0.713 (0.90 / 0.53) | 0.713 | 0.500 | 0.713 |
| 1 | 0.713 | 0.710 | 0.604 (0.99 / 0.22) | **0.712** (0.83 / 0.59) |
| 2 | 0.713 | 0.705 | 0.679 (0.97 / 0.39) | **0.758** (0.78 / 0.74) |
| 5 | 0.713 | 0.707 | **0.781** (0.95 / 0.61) | 0.774 (0.76 / 0.79) |

Robustness (balanced accuracy, C vs D):

| setting | k = 1 | k = 2 | k = 5 |
|---|---|---|---|
| seed 2 | 0.613 vs **0.705** | 0.674 vs **0.726** | **0.764** vs 0.755 |
| seed 3 | 0.628 vs **0.718** | 0.690 vs **0.754** | **0.792** vs 0.776 |
| severity 2 | 0.563 vs **0.596** | 0.612 vs **0.639** | **0.696** vs 0.658 |

## Findings

1. **Positives alone cannot reject structural near-misses.** A schema threshold accepts about half of them (true-negative rate 0.50–0.53; 0.24 at severity 2). Same content with the wrong causal structure still scores high, because noisy positives also lose some higher-order structure.
2. **Hard "must-have" conditions fail under noise** (B ≈ A at every k). Each near-miss breaks different links, and the rule must accept every noisy positive, so the learned conditions end up too lenient. Winston's clean-data formulation does not survive noisy instances.
3. **Soft near-miss emphasis is sample-efficient.** Weighting schema facts by how much better they discriminate positives from near-misses gives +0.04–0.09 balanced accuracy over 1-NN with 1–2 near-misses, across seeds and severities. With 5 near-misses, 1-NN catches up or edges ahead (+0.01–0.04).
   - It needs *one* mapping per test case (against the schema), while 1-NN needs one per labeled example.
   - It is interpretable: the weights name the facts that matter. Critical facts are mostly higher-order when learned from one near-miss (71%), as expected for re-wired near-misses; the share falls as more near-misses add noise-driven first-order facts.
4. **Design consequence.** Concept (schema) memory should store a per-fact *emphasis* learned from near-misses, next to SAGE's per-fact frequency. Frequency says what is typical; emphasis says what is diagnostic. Not yet built into `mars-engine::sage`; the evaluation is in `mars-bench e20`.
