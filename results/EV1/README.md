# EV1 (pre-registered): link prediction on FB15k-237 and WN18RR

Protocol, predictions and selection rules: [docs/PREREGISTRATION.md](../../docs/PREREGISTRATION.md). Frozen components as of commit `78fdc34`. Harness: `mars-bench ev1` (`crates/mars-bench/src/ev1.rs`); run script: `scripts/ev1_run.sh`, which chooses settings from the validation grid and runs the test set once.

- Validation grids: [FB15k-237](EV1-FB15k-237-valid-grid.md), [WN18RR](EV1-WN18RR-valid-grid.md).
- Test tables: [FB15k-237](EV1-FB15k-237-test.md), [WN18RR](EV1-WN18RR-test.md).
- Per-query ranks are in the JSON files.
- Predictions scored with the pre-committed criteria: [EV_PREDICTIONS.md](../EV_PREDICTIONS.md).

**Deviations from the pre-registration:**
- WN18RR uses the repo's `original/` (numeric id) files. This was decided and recorded in the pre-registration before any run; the `text/` variant has non-standard entity counts.
- The `v0.1` tag exists locally only; this session can push only its branch.
- No other deviations.

## Settings chosen on validation

| dataset | chosen (max validation MRR) | validation MRR |
|---|---|---|
| FB15k-237 | literal profile, identity λ = 0.85, k = 30 | 0.163 |
| WN18RR | literal profile, identity λ = 0.85, k = 30 | 0.346 |

Both choices sit at the **edge of the pre-registered grid** (the largest λ and k). The optimum may lie beyond it (more analogues, more weight on identity). That is not explored here, because the test set has now been used.

## Test results (filtered, both directions)

| method | FB15k-237 MRR / H@1 / H@3 / H@10 | WN18RR MRR / H@1 / H@3 / H@10 |
|---|---|---|
| **MARS** (learned reliability) | 0.163 / 0.129 / 0.177 / 0.231 | **0.372 / 0.360 / 0.380 / 0.396** |
| MARS raw | 0.159 / 0.124 / 0.174 / 0.230 | 0.351 / 0.322 / 0.372 / 0.395 |
| relation popularity (computed) | **0.233 / 0.170 / 0.250 / 0.354** | 0.026 / 0.016 / 0.025 / 0.044 |
| DistMult (published) | .241 / .155 / — / .419 | .43 / .39 / — / .49 |
| ComplEx (published) | .247 / .158 / — / .428 | .44 / .41 / — / .51 |
| RotatE (published) | .338 / .241 / .375 / .533 | .476 / .428 / .492 / .571 |
| TuckER (published) | .358 / .266 / — / .544 | .470 / .443 / — / .526 |

**By answer location** (is the answer within 2 undirected hops of the query entity in the training graph?), MARS learned:

| dataset | within 2 hops: share / MRR / H@1 | farther: share / MRR / H@1 |
|---|---|---|
| FB15k-237 | 74% / 0.173 / 0.138 | 26% / 0.134 / 0.101 |
| WN18RR | 44% / **0.758 / 0.749** | 56% / 0.066 / 0.051 |

## Predictions

| | prediction | outcome |
|---|---|---|
| P1 | MARS below RotatE and TuckER in MRR | **confirmed** on both datasets |
| P2 | MARS above relation popularity | **not confirmed on FB15k-237** (0.163 vs 0.233); confirmed on WN18RR (0.372 vs 0.026) |
| P3 | Hits@1 much higher within 2 hops | **confirmed on WN18RR** (0.75 vs 0.05); *inconclusive* on FB15k-237 (0.14 vs 0.10: higher, but not the pre-set 2×) |
| P4 | learned ≥ raw | **confirmed** on both (MRR +0.004 FB15k-237, +0.021 WN18RR) |
| P5 | relatively closer to KGE on WN18RR | **confirmed** (MARS/RotatE MRR ratio 0.78 on WN18RR vs 0.48 on FB15k-237) |

## Findings

1. **FB15k-237 is a clear negative.** MARS is below even relation popularity, and well below every published embedding model.
   - FB15k-237's relations are dominated by high-frequency object values (gender, nationality, profession, award categories, film genres). A global frequency prior predicts these well. MARS's votes from 30 analogues are noisier than the prior, and entities MARS does not propose are ranked at random, so they get no prior at all.
   - Being "within 2 hops" is also uninformative on FB15k-237: 74% of answers are that close, because the graph is dense around hubs. Relational locality does not separate the easy cases.
   - This agrees with E31: much of link prediction is *new-value* prediction, which analogy does not do well.
2. **WN18RR splits sharply by answer location.** MARS reaches Hits@1 0.36 overall (TuckER 0.44, RotatE 0.43), but MRR and Hits@10 lag because it proposes few candidates.
   - **Answer within 2 hops:** MARS gets Hits@1 0.75. This is the relational-inference regime E31 identified, e.g. the answer is the hypernym of a sibling, or a derivationally related form of a related word.
   - **Answer farther:** Hits@1 0.05. Analogy has nothing to transfer here.
   - A split for the published models is not available, so no claim is made about how MARS compares to them *within* the relational subset.
3. **Learned reliability helps a little, as predicted** (+0.004 / +0.021 MRR). Its gain on WN18RR is concentrated in the within-2-hops subset (H@1 0.66 → 0.75 there).
4. **The identity channel matters most on both datasets** (λ = 0.85 was chosen in both). In knowledge graphs without labels, overlap of neighbour identities is the main similarity signal, and the structural fingerprint adds little on its own (validation MRR 0.10–0.19 without identity).
5. **What this means for MARS.** On standard KG completion, MARS is not competitive overall and should not be presented as a link-prediction method. Where the answer is relationally close it is strong (WN18RR Hits@1 0.75), consistent with E27–E31. Useful directions:
   - a frequency or embedding prior for unproposed and new-value candidates, i.e. hybrid scoring (not pre-registered, needs its own evaluation);
   - query-conditioned subgraph cases instead of capped stars;
   - positioning MARS as the relational-transfer and explanation layer next to an embedding model, rather than a replacement for one.
