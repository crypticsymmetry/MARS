# E19: multi-view cases — fusing the nested and flat views of code

Full table: [E19.md](E19.md). Reproduce with `mars-bench e19 [--seed 1]` (~6 s; needs `tools/fetch_e9_corpus.sh`). The flat view is now a library transform: `mars_rel::views::FlatView`.

**Question:** E18 found that the nested view of code (wrapped statements) has the stronger fingerprint and restores more statements, while the flat-block view has the stronger mapper and places statements better. Does keeping *both* views of every case, and combining them, beat either one?

**Setup.** Every function (1,308: Python + JS) has both views. FP (literal profile) and FAC are computed within each view.

Retrieval compares:
- each view at its best fusion weight (nested 0.3, flat 0.7);
- a-priori cross-view fusion ½·FAC(flat) + ½·FP(nested), i.e. each view's stronger signal;
- the average of the two views' fused scores;
- reciprocal-rank fusion (RRF).

It runs on two tasks: same algorithm, other package (33 Python queries, E9) and same algorithm, other language (52 queries, E12).

Inference uses E17's 700 deleted statements, scored at the core-statement level. It compares proposals from each view, their union, *cross-view agreement*, and top-5 corroboration.

## Results

Retrieval, MRR (R@1 / R@10):

| method | other package | other language |
|---|---|---|
| nested fused 0.3 (E9) | 0.359 (0.27 / 0.55) | 0.141 (0.00 / 0.38) |
| **flat fused 0.7** | **0.375** (0.27 / 0.58) | **0.198** (0.08 / 0.38) |
| cross-view ½·FAC(flat) + ½·FP(nested) | 0.324 (0.18 / 0.64) | 0.176 (0.04 / 0.35) |
| view average | 0.331 (0.21 / 0.64) | **0.202** (0.08 / 0.37) |
| RRF(nested, flat) | 0.339 (0.24 / **0.67**) | 0.171 (0.06 / 0.37) |

Inference, core statement:

| analogue ranked by | proposals from | recall | precision | proposals / q |
|---|---|---|---|---|
| nested fused 0.3 | nested view (E17 setting) | 0.189 | 0.555 | 2.11 |
| nested fused 0.3 | both views agree | 0.104 | 0.577 | 0.71 |
| **flat fused 0.7** | **nested view** | **0.193** | **0.637** | 2.18 |
| flat fused 0.7 | both views agree | 0.109 | 0.597 | 0.78 |
| nested fused 0.3, top-5 | nested, support ≥ 2 / ≥ 3 | 0.116 / 0.071 | 0.752 / 0.878 | 1.65 / 0.57 |
| **flat fused 0.7, top-5** | **nested, support ≥ 2 / ≥ 3** | **0.124 / 0.089** | **0.768 / 0.870** | 1.97 / 0.80 |

## Findings

1. **Score-level fusion of views does not beat the best single view.** The a-priori cross-view score and RRF lose MRR against the flat view alone. They only gain R@10 on the small other-package task (0.64–0.67 vs 0.58). Signals from two views of the same case are too correlated to add much, and each fusion dilutes the stronger ranking.
2. **The flat view is the better retrieval view across languages:** MRR 0.198 vs 0.141 (+40%), and above E12's best (0.168). Block structure abstracts away language-specific nesting idioms.
3. **Views have roles.** Rank and map in the flat view, project statements from the nested view. This beats the single-view pipeline at every operating point:
   - top-1 precision 0.637 vs 0.555 at equal recall;
   - with top-5 corroboration, recall 0.124 vs 0.116 at support ≥ 2 and 0.089 vs 0.071 at support ≥ 3, with equal precision.
4. **Corroboration needs independent evidence.** Agreement between two views of *the same analogue* is a weak confidence signal (precision 0.58–0.60, about the level of a single view). Agreement between *different analogues* is strong (0.75–0.88). This sharpens E11: support counts must be over independent sources, and the JTMS justification per analogue (engine) is the right unit, not per view.

## Design consequence

Multi-view cases are worth keeping only with *specialized roles*: a mapping/ranking view and a projection view. This is not built into the engine (it holds one view per case). The measured gain on code is modest (+15% precision at top-1, +25% recall at support ≥ 3), so it stays an application-level pattern for now, available via `mars_rel::views`.
