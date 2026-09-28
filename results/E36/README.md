# E36 (pre-registered, addendum B): learned fusion and conformal abstention for code retrieval

Protocol and predictions: [docs/PREREGISTRATION.md, addendum B](../../docs/PREREGISTRATION.md) and amendments B.1–B.2. Tool: `tools/e36_fusion.py`. Tables: [E36.md](E36.md) (test) and [E36-dev.md](E36-dev.md) (development); per-query outcomes are in `E36.json`.

**Setup.** Candidates are the code embedding's top-100 per query (jina-embeddings-v2-base-code, as in EV2 and E34). MARS contributes the mapper's FAC score for each candidate, computed with E34's dataflow front end.
- **A3, learned fusion:** a pointwise logistic ranker (L2, C = 1) fit on the development sample's 597k (query, candidate) pairs. Features:
  - candidate: cosine, FAC, cosine rank, FAC rank;
  - query: top-1 cosine, top-1 − top-2 margin, max FAC;
  - interactions: cosine × top-1 cosine, FAC × top-1 cosine.
- **A4, conformal selective top-1:** answer with the top-1 program only when its confidence clears a threshold calibrated by Learn-then-Test on development: precision ≥ 1 − α = 0.95 with probability ≥ 1 − δ = 0.9. The threshold is applied unchanged to test.

Reproduce:
```
python3 tools/e34_eval.py cands W_PDG_DEV W_MARS_DEV && mars-bench ev2 --data W_PDG_DEV --cands emb_cands.json   # likewise for test
python3 tools/e36_fusion.py W_MARS_DEV W_PDG_DEV W_MARS_TEST W_PDG_TEST results/E36
```

## Test (EV2's 5,982 programs, run once)

| method | MAP@R [95% CI] | Hits@1 | MRR |
|---|---|---|---|
| code embedding | 0.473 [0.466, 0.480] | 0.856 | 0.892 |
| fixed ½ cosine + ½ FAC (E34) | 0.498 [0.491, 0.504] | **0.892** | **0.919** |
| **learned fusion (A3)** | **0.503** [0.496, 0.510] | 0.891 | 0.917 |

| selective top-1 (α = 0.05, δ = 0.1) | threshold (from development) | answered | precision among answered [95% CI] |
|---|---|---|---|
| **A3 confidence** (MARS + embedding) | 0.957 | **988 / 5,982 (16.5%)** | **0.968** [0.957, 0.978] |
| embedding top-1 cosine | none certifiable | 0 | — |

**Predictions** (scored by `tools/ev_report.py`; see [EV_PREDICTIONS.md](../EV_PREDICTIONS.md)):

| | prediction | result | verdict |
|---|---|---|---|
| P13 | learned fusion > fixed ½/½ | +0.0055 [+0.0038, +0.0073] MAP@R | **confirmed** (small) |
| P14 | test precision among answered ≥ 0.95 | 0.968 [0.957, 0.978] | **confirmed** |
| P15 | coverage with learned confidence > with embedding cosine | +0.165 [+0.156, +0.175] | **confirmed** |

**Deviations from addendum B** (both recorded in the pre-registration before the test run):
- **B.1.** Fixed-sequence testing cannot reject below 45 answered queries, so it would start there. Found on a 100-program smoke sample, before any development run.
- **B.2.** The fixed-sequence variant still answered nothing on development: it stopped at a local dip, with 4 errors in the 100 most confident queries. A4 switched to Learn-then-Test with a Bonferroni correction over the same grid. This was chosen after seeing development behaviour, which is why P14 checks the guarantee empirically on test. It held.

## Findings

1. **The learned ranker encodes "trust MARS more when the embedding is unsure".** Its weights on standardized features: FAC +1.95, FAC × top-1 cosine −1.71, cosine −2.51, cosine × top-1 cosine +3.82. FAC's weight grows as the embedding's top-1 cosine falls, and cosine's weight grows as it rises. This is E34's exploratory finding, now learned and tested.
2. **Learning adds little over the fixed ½/½ weights on MAP@R** (+0.006, significant). Hits@1 and MRR are marginally lower than with the fixed weights. Most of the benefit of adding MARS was already captured by E34's fixed fusion.
3. **Calibrated abstention is where MARS adds a capability.** With MARS and embedding features, 16.5% of queries can be answered with a certified ≥ 95% precision, and test precision was 0.968. The embedding's own cosine cannot certify any answer at this level: its most confident matches include byte-identical programs submitted to *different* problems, so high cosine is not reliable evidence of the same task. The structural mapping score, conditioned on the embedding, is.
4. **Caveat on the guarantee.** Learn-then-Test assumes exchangeable queries. Queries are clustered by problem, and development and test are different random problem draws from the same pool. The guarantee held on test, but it is an empirical check, not a theorem, for this data.
