# EV2 (pre-registered): code retrieval by algorithm on CodeNet Python800

Protocol, predictions and selection rules: [docs/PREREGISTRATION.md](../../docs/PREREGISTRATION.md). Frozen components as of commit `78fdc34`. Tables: [EV2.md](EV2.md); per-query outcomes: `EV2.json`.

Reproduce:
```
tar -xzf data/external/codenet/Python800.tar.gz -C data/external/codenet     # HF mirror, sha256 39297d11…
python3 tools/ev2_codenet.py prepare WORK
mars-bench ev2 --data WORK
python3 tools/ev2_codenet.py evaluate WORK results/EV2
```

**Deviations from the pre-registration:**
1. Code-embedding inputs are capped at 2,000 characters and embedded in batches of 4. The first attempt, at 8,000 / 32, was killed out of memory (12 GB) before any metric was computed. The cap affects 45 of 5,982 programs (median length 253 characters).
2. Sample size: 5,982 programs rather than 6,000. Two of the 200 problems had fewer than 30 qualifying solutions (16 and 26), and the rule "take all" applied. 4,496 files were skipped: under 5 facts, or Python 2 / syntax errors.

## Results (5,982 queries; relevant = other solutions of the same problem)

| method | MAP@R [95% CI] | Hits@1 | MRR |
|---|---|---|---|
| code embedding (jina-embeddings-v2-base-code) | **0.473** [0.466, 0.480] | **0.856** | **0.892** |
| MARS fused (½FAC + ½FP-literal over the FP top-100) | 0.211 [0.206, 0.216] | 0.699 | 0.761 |
| MARS fingerprint-literal only | 0.166 [0.162, 0.171] | 0.609 | 0.689 |
| RRF(MARS fused, lexical) | 0.164 [0.160, 0.169] | 0.605 | 0.694 |
| lexical TF-IDF (Python tokens) | 0.129 [0.125, 0.133] | 0.494 | 0.594 |

**Predictions** (paired over queries, MAP@R):

| prediction | result | verdict |
|---|---|---|
| P6: MARS fused > lexical TF-IDF | +0.082 [+0.078, +0.087] | **confirmed** |
| P7: code embedding ≥ MARS fused | embedding − MARS = +0.262 [+0.256, +0.269] | **confirmed** |
| P8: RRF(MARS, lexical) ≥ both | vs MARS −0.047 [−0.050, −0.044]; vs lexical +0.036 | **not confirmed** |

## Findings

1. **E9's direction replicates on independent data at scale.** Structure beats lexical overlap, and the gap is large and tight at 5,982 queries: MAP@R 0.211 vs 0.129, Hits@1 0.70 vs 0.49. The structural mapping step adds substantially to the fingerprint alone (0.211 vs 0.166).
2. **A pretrained code embedding is far better:** MAP@R 0.47 vs 0.21, Hits@1 0.86 vs 0.70. This is the external benchmark the review asked for, and it answers the question plainly. For "same problem" retrieval on real code, MARS's syntactic representation is well behind a learned code model. It is the bottleneck E9 identified (different strategies for the same problem produce different syntax), now measured against a strong baseline. The embedding also carries information MARS never sees: identifier names, string constants, input-parsing idioms and problem-specific vocabulary.
3. **Rank fusion with lexical hurts MARS here.** RRF(MARS, lexical) is below MARS alone. The lexical ranking is much weaker, and equal-weight RRF lets it dilute the stronger list. In E24 the two were closer in strength. Fusion helps only between signals of comparable quality, or with learned weights.
4. **What this means for MARS.** On whole-program retrieval by problem, MARS is a clear improvement over lexical search but not competitive with learned code embeddings. The pre-registered prediction said so (P7), and the margin is large. Two directions follow:
   - **Representation.** Program-dependence graphs, e-graph normalization and learned abstractions (library learning) are what could close part of the gap.
   - **Division of labour.** Use embeddings for recall. Use MARS where it adds what embeddings lack: explicit correspondences, candidate inferences with provenance, and structural verification of a shortlist, e.g. re-ranking the embedding's top-k by structure. That combination was *not* pre-registered and would need its own evaluation.
