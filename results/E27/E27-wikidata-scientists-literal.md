# E27: knowledge-graph completion by analogy (wikidata-scientists-literal)

Data: `data/kg-scientists → kg2mars --condition C` (wikidata side, 993 entity cases). 2400 queries: an entity with all facts of one relation removed (and the labels of objects no longer mentioned), up to 300 per relation, seed 1. Analogues: top-10 (MARS fused = fingerprint-literal top-50 re-ranked by 0.5·FAC + 0.5·FP). Votes weighted by similarity; ties broken by popularity.

## 1 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p69, wdt:p108, wdt:p101, wdt:p27, wdt:p106, wdt:p166, wdt:p463, wdt:p1412) |
|---|---|---|---|---|---|
| popularity | 0.222 | 0.535 | 0.322 | 1.000 | 0.06 / 0.04 / 0.12 / 0.33 / 0.36 / 0.09 / 0.20 / 0.57 |
| copy · random neighbours | 0.067 | 0.073 | 0.070 | 0.073 | 0.01 / 0.00 / 0.02 / 0.11 / 0.13 / 0.02 / 0.04 / 0.21 |
| copy · lexical neighbours | 0.214 | 0.245 | 0.228 | 0.245 | 0.12 / 0.13 / 0.06 / 0.37 / 0.29 / 0.09 / 0.26 / 0.39 |
| copy · fingerprint neighbours | 0.121 | 0.137 | 0.129 | 0.137 | 0.02 / 0.02 / 0.02 / 0.18 / 0.26 / 0.05 / 0.11 / 0.30 |
| copy · MARS fused neighbours | 0.139 | 0.154 | 0.146 | 0.154 | 0.04 / 0.02 / 0.03 / 0.24 / 0.26 / 0.05 / 0.14 / 0.32 |
| analogy · lexical neighbours | 0.213 | 0.243 | 0.227 | 0.243 | 0.11 / 0.14 / 0.06 / 0.36 / 0.28 / 0.09 / 0.26 / 0.40 |
| analogy · MARS fused neighbours | 0.148 | 0.166 | 0.157 | 0.166 | 0.07 / 0.04 / 0.04 / 0.24 / 0.26 / 0.05 / 0.14 / 0.34 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.222 | 0.254 | 0.237 | 0.254 | 0.07 / 0.09 / 0.08 / 0.39 / 0.29 / 0.09 / 0.28 / 0.48 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.224 | 0.259 | 0.240 | 0.259 | 0.08 / 0.10 / 0.08 / 0.38 / 0.29 / 0.09 / 0.28 / 0.50 |
| rules (length-1, same object) | 0.109 | 0.130 | 0.118 | 0.130 | 0.22 / 0.26 / 0.07 / 0.07 / 0.00 / 0.02 / 0.04 / 0.20 |
| rules + copy · lexical (vote share + confidence) | 0.248 | 0.326 | 0.281 | 0.326 | 0.17 / 0.19 / 0.09 / 0.40 / 0.29 / 0.10 / 0.28 / 0.47 |
| rules + analogy · hybrid | 0.249 | 0.340 | 0.288 | 0.340 | 0.11 / 0.16 / 0.10 / 0.38 / 0.29 / 0.09 / 0.28 / 0.58 |

## 5 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p69, wdt:p108, wdt:p101, wdt:p27, wdt:p106, wdt:p166, wdt:p463, wdt:p1412) |
|---|---|---|---|---|---|
| popularity | 0.222 | 0.535 | 0.322 | 1.000 | 0.06 / 0.04 / 0.12 / 0.33 / 0.36 / 0.09 / 0.20 / 0.57 |
| copy · random neighbours | 0.161 | 0.263 | 0.201 | 0.264 | 0.03 / 0.02 / 0.04 / 0.28 / 0.26 / 0.04 / 0.14 / 0.48 |
| copy · lexical neighbours | 0.344 | 0.504 | 0.401 | 0.506 | 0.16 / 0.16 / 0.13 / 0.61 / 0.34 / 0.19 / 0.42 / 0.74 |
| copy · fingerprint neighbours | 0.179 | 0.341 | 0.234 | 0.348 | 0.02 / 0.03 / 0.04 / 0.24 / 0.29 / 0.06 / 0.22 / 0.53 |
| copy · MARS fused neighbours | 0.210 | 0.357 | 0.260 | 0.360 | 0.04 / 0.03 / 0.09 / 0.33 / 0.30 / 0.07 / 0.26 / 0.56 |
| analogy · lexical neighbours | 0.345 | 0.505 | 0.403 | 0.508 | 0.16 / 0.19 / 0.13 / 0.60 / 0.34 / 0.19 / 0.41 / 0.74 |
| analogy · MARS fused neighbours | 0.227 | 0.389 | 0.282 | 0.391 | 0.09 / 0.09 / 0.10 / 0.33 / 0.30 / 0.07 / 0.26 / 0.59 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.329 | 0.509 | 0.392 | 0.511 | 0.12 / 0.11 / 0.12 / 0.55 / 0.36 / 0.19 / 0.45 / 0.74 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.338 | 0.523 | 0.402 | 0.525 | 0.16 / 0.13 / 0.13 / 0.53 / 0.36 / 0.19 / 0.44 / 0.76 |
| rules (length-1, same object) | 0.109 | 0.130 | 0.118 | 0.130 | 0.22 / 0.26 / 0.07 / 0.07 / 0.00 / 0.02 / 0.04 / 0.20 |
| rules + copy · lexical (vote share + confidence) | 0.362 | 0.550 | 0.429 | 0.552 | 0.20 / 0.19 / 0.15 / 0.61 / 0.34 / 0.20 / 0.43 / 0.78 |
| rules + analogy · hybrid | 0.354 | 0.556 | 0.425 | 0.557 | 0.20 / 0.20 / 0.14 / 0.53 / 0.36 / 0.19 / 0.44 / 0.78 |

## 10 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p69, wdt:p108, wdt:p101, wdt:p27, wdt:p106, wdt:p166, wdt:p463, wdt:p1412) |
|---|---|---|---|---|---|
| popularity | 0.222 | 0.535 | 0.322 | 1.000 | 0.06 / 0.04 / 0.12 / 0.33 / 0.36 / 0.09 / 0.20 / 0.57 |
| copy · random neighbours | 0.179 | 0.348 | 0.237 | 0.353 | 0.02 / 0.02 / 0.05 / 0.30 / 0.31 / 0.06 / 0.17 / 0.51 |
| copy · lexical neighbours | 0.378 | 0.601 | 0.454 | 0.620 | 0.17 / 0.15 / 0.15 / 0.65 / 0.36 / 0.23 / 0.48 / 0.82 |
| copy · fingerprint neighbours | 0.208 | 0.394 | 0.270 | 0.429 | 0.03 / 0.04 / 0.08 / 0.30 / 0.34 / 0.07 / 0.24 / 0.58 |
| copy · MARS fused neighbours | 0.245 | 0.431 | 0.307 | 0.463 | 0.06 / 0.04 / 0.12 / 0.36 / 0.33 / 0.10 / 0.31 / 0.65 |
| analogy · lexical neighbours | 0.385 | 0.605 | 0.461 | 0.625 | 0.19 / 0.20 / 0.16 / 0.64 / 0.36 / 0.23 / 0.47 / 0.83 |
| analogy · MARS fused neighbours | 0.270 | 0.471 | 0.337 | 0.503 | 0.14 / 0.14 / 0.13 / 0.36 / 0.32 / 0.10 / 0.31 / 0.67 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.356 | 0.579 | 0.431 | 0.605 | 0.13 / 0.10 / 0.16 / 0.57 / 0.43 / 0.21 / 0.48 / 0.77 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.378 | 0.595 | 0.449 | 0.618 | 0.19 / 0.19 / 0.18 / 0.57 / 0.43 / 0.21 / 0.48 / 0.78 |
| rules (length-1, same object) | 0.109 | 0.130 | 0.118 | 0.130 | 0.22 / 0.26 / 0.07 / 0.07 / 0.00 / 0.02 / 0.04 / 0.20 |
| rules + copy · lexical (vote share + confidence) | 0.400 | 0.633 | 0.482 | 0.656 | 0.23 / 0.22 / 0.17 / 0.65 / 0.36 / 0.23 / 0.48 / 0.85 |
| rules + analogy · hybrid | 0.386 | 0.616 | 0.462 | 0.636 | 0.21 / 0.22 / 0.19 / 0.55 / 0.43 / 0.21 / 0.48 / 0.80 |

## Analogy vs copy (MARS analogues, m = 10)

- Top-1 predictions that are pure substitutions (a query entity reached through the mapping, not an analogue's own object): 247 of 2374 (10.4%), precision 0.219; other top-1s precision 0.280.
- Substitutions per relation (count, precision): wdt:p69 108 (0.22), wdt:p108 129 (0.20), wdt:p101 6 (0.50), wdt:p27 3 (0.00), wdt:p106 0 (NaN), wdt:p166 0 (NaN), wdt:p463 0 (NaN), wdt:p1412 1 (1.00).
- Where analogy's and copy's top-1 differ (318 queries): analogy right 74, copy right 13.

**Calibration** (analogy · MARS, top-1 precision by support = analogues proposing it):

| support | queries | precision |
|---|---|---|
| 1 | 688 | 0.062 |
| 2 | 531 | 0.171 |
| 3 | 439 | 0.285 |
| 4 | 278 | 0.424 |
| 5+ | 438 | 0.621 |

Runtime 9.1s.
