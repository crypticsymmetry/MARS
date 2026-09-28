# E27: knowledge-graph completion by analogy (wikidata-films)

Data: `data/kg-films → kg2mars --condition C` (wikidata side, 992 entity cases). 3000 queries: an entity with all facts of one relation removed (and the labels of objects no longer mentioned), up to 300 per relation, seed 1. Analogues: top-10 (MARS fused = fingerprint-surface top-50 re-ranked by 0.5·FAC + 0.5·FP). Votes weighted by similarity; ties broken by popularity.

## 1 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272) |
|---|---|---|---|---|---|
| popularity | 0.144 | 0.305 | 0.200 | 1.000 | 0.01 / 0.00 / 0.00 / 0.01 / 0.02 / 0.01 / 0.38 / 0.42 / 0.53 / 0.06 |
| copy · random neighbours | 0.063 | 0.065 | 0.064 | 0.065 | 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.18 / 0.17 / 0.27 / 0.00 |
| copy · lexical neighbours | 0.203 | 0.211 | 0.207 | 0.211 | 0.08 / 0.08 / 0.07 / 0.09 / 0.07 / 0.04 / 0.33 / 0.59 / 0.56 / 0.13 |
| copy · fingerprint neighbours | 0.173 | 0.185 | 0.179 | 0.185 | 0.06 / 0.05 / 0.06 / 0.06 / 0.07 / 0.03 / 0.31 / 0.48 / 0.50 / 0.10 |
| copy · MARS fused neighbours | 0.176 | 0.182 | 0.179 | 0.182 | 0.06 / 0.05 / 0.06 / 0.06 / 0.07 / 0.03 / 0.30 / 0.51 / 0.52 / 0.11 |
| analogy · lexical neighbours | 0.216 | 0.226 | 0.221 | 0.226 | 0.13 / 0.08 / 0.14 / 0.10 / 0.07 / 0.04 / 0.33 / 0.58 / 0.56 / 0.14 |
| analogy · MARS fused neighbours | 0.207 | 0.218 | 0.212 | 0.218 | 0.14 / 0.06 / 0.19 / 0.07 / 0.08 / 0.03 / 0.30 / 0.51 / 0.52 / 0.17 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.212 | 0.223 | 0.217 | 0.223 | 0.07 / 0.06 / 0.08 / 0.08 / 0.08 / 0.03 / 0.36 / 0.62 / 0.60 / 0.16 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.234 | 0.248 | 0.240 | 0.248 | 0.14 / 0.06 / 0.18 / 0.08 / 0.09 / 0.03 / 0.36 / 0.61 / 0.60 / 0.19 |
| rules (length-1, same object) | 0.177 | 0.211 | 0.190 | 0.211 | 0.37 / 0.09 / 0.57 / 0.22 / 0.08 / 0.04 / 0.01 / 0.07 / 0.01 / 0.32 |
| rules + copy · lexical (vote share + confidence) | 0.286 | 0.389 | 0.330 | 0.389 | 0.11 / 0.11 / 0.41 / 0.25 / 0.11 / 0.07 / 0.32 / 0.59 / 0.56 / 0.33 |
| rules + analogy · hybrid | 0.286 | 0.397 | 0.334 | 0.398 | 0.15 / 0.08 / 0.40 / 0.19 / 0.10 / 0.04 / 0.35 / 0.61 / 0.60 / 0.33 |

## 5 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272) |
|---|---|---|---|---|---|
| popularity | 0.144 | 0.305 | 0.200 | 1.000 | 0.01 / 0.00 / 0.00 / 0.01 / 0.02 / 0.01 / 0.38 / 0.42 / 0.53 / 0.06 |
| copy · random neighbours | 0.122 | 0.171 | 0.143 | 0.171 | 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.31 / 0.39 / 0.51 / 0.01 |
| copy · lexical neighbours | 0.260 | 0.344 | 0.293 | 0.346 | 0.07 / 0.08 / 0.10 / 0.14 / 0.09 / 0.04 / 0.44 / 0.69 / 0.72 / 0.22 |
| copy · fingerprint neighbours | 0.199 | 0.298 | 0.236 | 0.301 | 0.06 / 0.05 / 0.06 / 0.09 / 0.07 / 0.04 / 0.37 / 0.53 / 0.59 / 0.13 |
| copy · MARS fused neighbours | 0.205 | 0.290 | 0.238 | 0.292 | 0.06 / 0.05 / 0.07 / 0.08 / 0.07 / 0.03 / 0.39 / 0.56 / 0.60 / 0.15 |
| analogy · lexical neighbours | 0.300 | 0.402 | 0.340 | 0.404 | 0.16 / 0.08 / 0.34 / 0.15 / 0.10 / 0.04 / 0.44 / 0.69 / 0.72 / 0.27 |
| analogy · MARS fused neighbours | 0.270 | 0.381 | 0.312 | 0.383 | 0.21 / 0.06 / 0.39 / 0.11 / 0.09 / 0.03 / 0.39 / 0.56 / 0.60 / 0.25 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.240 | 0.333 | 0.276 | 0.335 | 0.08 / 0.06 / 0.09 / 0.09 / 0.08 / 0.03 / 0.41 / 0.66 / 0.68 / 0.21 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.295 | 0.409 | 0.337 | 0.410 | 0.21 / 0.08 / 0.39 / 0.10 / 0.10 / 0.03 / 0.41 / 0.67 / 0.68 / 0.28 |
| rules (length-1, same object) | 0.177 | 0.211 | 0.190 | 0.211 | 0.37 / 0.09 / 0.57 / 0.22 / 0.08 / 0.04 / 0.01 / 0.07 / 0.01 / 0.32 |
| rules + copy · lexical (vote share + confidence) | 0.330 | 0.497 | 0.392 | 0.503 | 0.37 / 0.08 / 0.33 / 0.18 / 0.13 / 0.05 / 0.44 / 0.70 / 0.72 / 0.30 |
| rules + analogy · hybrid | 0.334 | 0.488 | 0.388 | 0.492 | 0.32 / 0.08 / 0.52 / 0.17 / 0.13 / 0.04 / 0.41 / 0.66 / 0.68 / 0.33 |

## 10 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272) |
|---|---|---|---|---|---|
| popularity | 0.144 | 0.305 | 0.200 | 1.000 | 0.01 / 0.00 / 0.00 / 0.01 / 0.02 / 0.01 / 0.38 / 0.42 / 0.53 / 0.06 |
| copy · random neighbours | 0.128 | 0.207 | 0.156 | 0.208 | 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.33 / 0.40 / 0.53 / 0.02 |
| copy · lexical neighbours | 0.266 | 0.381 | 0.309 | 0.388 | 0.08 / 0.07 / 0.10 / 0.14 / 0.08 / 0.04 / 0.45 / 0.70 / 0.75 / 0.25 |
| copy · fingerprint neighbours | 0.206 | 0.329 | 0.248 | 0.339 | 0.07 / 0.05 / 0.06 / 0.08 / 0.07 / 0.04 / 0.38 / 0.55 / 0.60 / 0.15 |
| copy · MARS fused neighbours | 0.207 | 0.323 | 0.249 | 0.330 | 0.06 / 0.04 / 0.06 / 0.08 / 0.07 / 0.03 / 0.39 / 0.57 / 0.61 / 0.14 |
| analogy · lexical neighbours | 0.325 | 0.461 | 0.375 | 0.469 | 0.25 / 0.08 / 0.41 / 0.17 / 0.10 / 0.04 / 0.45 / 0.69 / 0.75 / 0.30 |
| analogy · MARS fused neighbours | 0.294 | 0.431 | 0.344 | 0.440 | 0.28 / 0.07 / 0.48 / 0.15 / 0.09 / 0.03 / 0.39 / 0.58 / 0.61 / 0.26 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.243 | 0.365 | 0.288 | 0.374 | 0.08 / 0.05 / 0.09 / 0.09 / 0.08 / 0.03 / 0.42 / 0.67 / 0.71 / 0.21 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.318 | 0.460 | 0.369 | 0.470 | 0.28 / 0.08 / 0.47 / 0.12 / 0.11 / 0.03 / 0.42 / 0.68 / 0.71 / 0.29 |
| rules (length-1, same object) | 0.177 | 0.211 | 0.190 | 0.211 | 0.37 / 0.09 / 0.57 / 0.22 / 0.08 / 0.04 / 0.01 / 0.07 / 0.01 / 0.32 |
| rules + copy · lexical (vote share + confidence) | 0.342 | 0.525 | 0.405 | 0.536 | 0.39 / 0.09 / 0.39 / 0.16 / 0.13 / 0.05 / 0.45 / 0.70 / 0.75 / 0.31 |
| rules + analogy · hybrid | 0.344 | 0.510 | 0.402 | 0.522 | 0.32 / 0.09 / 0.55 / 0.19 / 0.13 / 0.03 / 0.42 / 0.68 / 0.71 / 0.34 |

## Analogy vs copy (MARS analogues, m = 10)

- Top-1 predictions that are pure substitutions (a query entity reached through the mapping, not an analogue's own object): 630 of 2970 (21.2%), precision 0.394; other top-1s precision 0.271.
- Substitutions per relation (count, precision): wdt:p57 151 (0.46), wdt:p161 57 (0.09), wdt:p58 227 (0.54), wdt:p162 83 (0.27), wdt:p86 11 (0.45), wdt:p344 8 (0.00), wdt:p136 0 (NaN), wdt:p495 1 (0.00), wdt:p364 0 (NaN), wdt:p272 92 (0.26).
- Where analogy's and copy's top-1 differ (703 queries): analogy right 292, copy right 31.

**Calibration** (analogy · MARS, top-1 precision by support = analogues proposing it):

| support | queries | precision |
|---|---|---|
| 1 | 1263 | 0.063 |
| 2 | 452 | 0.274 |
| 3 | 326 | 0.414 |
| 4 | 328 | 0.442 |
| 5+ | 601 | 0.661 |

Runtime 15.6s.
