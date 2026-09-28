# E27: knowledge-graph completion by analogy (wikidata-films-literal)

Data: `data/kg-films → kg2mars --condition C` (wikidata side, 992 entity cases). 3000 queries: an entity with all facts of one relation removed (and the labels of objects no longer mentioned), up to 300 per relation, seed 1. Analogues: top-10 (MARS fused = fingerprint-literal top-50 re-ranked by 0.5·FAC + 0.5·FP). Votes weighted by similarity; ties broken by popularity.

## 1 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272) |
|---|---|---|---|---|---|
| popularity | 0.144 | 0.305 | 0.200 | 1.000 | 0.01 / 0.00 / 0.00 / 0.01 / 0.02 / 0.01 / 0.38 / 0.42 / 0.53 / 0.06 |
| copy · random neighbours | 0.063 | 0.065 | 0.064 | 0.065 | 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.18 / 0.17 / 0.27 / 0.00 |
| copy · lexical neighbours | 0.203 | 0.211 | 0.207 | 0.211 | 0.08 / 0.08 / 0.07 / 0.09 / 0.07 / 0.04 / 0.33 / 0.59 / 0.56 / 0.13 |
| copy · fingerprint neighbours | 0.096 | 0.101 | 0.098 | 0.101 | 0.01 / 0.00 / 0.01 / 0.02 / 0.03 / 0.00 / 0.22 / 0.30 / 0.35 / 0.03 |
| copy · MARS fused neighbours | 0.117 | 0.121 | 0.119 | 0.121 | 0.03 / 0.03 / 0.03 / 0.03 / 0.04 / 0.01 / 0.26 / 0.33 / 0.37 / 0.04 |
| analogy · lexical neighbours | 0.216 | 0.226 | 0.221 | 0.226 | 0.13 / 0.08 / 0.14 / 0.10 / 0.07 / 0.04 / 0.33 / 0.58 / 0.56 / 0.14 |
| analogy · MARS fused neighbours | 0.147 | 0.154 | 0.150 | 0.154 | 0.12 / 0.03 / 0.15 / 0.06 / 0.05 / 0.01 / 0.26 / 0.34 / 0.37 / 0.09 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.197 | 0.207 | 0.202 | 0.207 | 0.07 / 0.05 / 0.08 / 0.07 / 0.08 / 0.03 / 0.32 / 0.56 / 0.58 / 0.13 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.221 | 0.234 | 0.227 | 0.234 | 0.14 / 0.05 / 0.19 / 0.08 / 0.09 / 0.03 / 0.32 / 0.56 / 0.58 / 0.17 |
| rules (length-1, same object) | 0.177 | 0.211 | 0.190 | 0.211 | 0.37 / 0.09 / 0.57 / 0.22 / 0.08 / 0.04 / 0.01 / 0.07 / 0.01 / 0.32 |
| rules + copy · lexical (vote share + confidence) | 0.286 | 0.389 | 0.330 | 0.389 | 0.11 / 0.11 / 0.41 / 0.25 / 0.11 / 0.07 / 0.32 / 0.59 / 0.56 / 0.33 |
| rules + analogy · hybrid | 0.276 | 0.385 | 0.323 | 0.386 | 0.14 / 0.09 / 0.42 / 0.18 / 0.10 / 0.05 / 0.31 / 0.56 / 0.58 / 0.32 |

## 5 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272) |
|---|---|---|---|---|---|
| popularity | 0.144 | 0.305 | 0.200 | 1.000 | 0.01 / 0.00 / 0.00 / 0.01 / 0.02 / 0.01 / 0.38 / 0.42 / 0.53 / 0.06 |
| copy · random neighbours | 0.122 | 0.171 | 0.143 | 0.171 | 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.31 / 0.39 / 0.51 / 0.01 |
| copy · lexical neighbours | 0.260 | 0.344 | 0.293 | 0.346 | 0.07 / 0.08 / 0.10 / 0.14 / 0.09 / 0.04 / 0.44 / 0.69 / 0.72 / 0.22 |
| copy · fingerprint neighbours | 0.121 | 0.209 | 0.154 | 0.210 | 0.01 / 0.01 / 0.01 / 0.03 / 0.03 / 0.00 / 0.27 / 0.38 / 0.44 / 0.04 |
| copy · MARS fused neighbours | 0.146 | 0.232 | 0.178 | 0.234 | 0.03 / 0.03 / 0.03 / 0.04 / 0.04 / 0.01 / 0.32 / 0.40 / 0.50 / 0.05 |
| analogy · lexical neighbours | 0.300 | 0.402 | 0.340 | 0.404 | 0.16 / 0.08 / 0.34 / 0.15 / 0.10 / 0.04 / 0.44 / 0.69 / 0.72 / 0.27 |
| analogy · MARS fused neighbours | 0.210 | 0.324 | 0.250 | 0.326 | 0.15 / 0.04 / 0.34 / 0.10 / 0.06 / 0.02 / 0.32 / 0.41 / 0.50 / 0.17 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.234 | 0.325 | 0.270 | 0.327 | 0.07 / 0.05 / 0.09 / 0.09 / 0.08 / 0.03 / 0.41 / 0.63 / 0.68 / 0.20 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.284 | 0.402 | 0.328 | 0.403 | 0.19 / 0.07 / 0.34 / 0.11 / 0.10 / 0.03 / 0.41 / 0.64 / 0.68 / 0.27 |
| rules (length-1, same object) | 0.177 | 0.211 | 0.190 | 0.211 | 0.37 / 0.09 / 0.57 / 0.22 / 0.08 / 0.04 / 0.01 / 0.07 / 0.01 / 0.32 |
| rules + copy · lexical (vote share + confidence) | 0.330 | 0.497 | 0.392 | 0.503 | 0.37 / 0.08 / 0.33 / 0.18 / 0.13 / 0.05 / 0.44 / 0.70 / 0.72 / 0.30 |
| rules + analogy · hybrid | 0.333 | 0.484 | 0.387 | 0.488 | 0.34 / 0.09 / 0.50 / 0.17 / 0.13 / 0.03 / 0.41 / 0.65 / 0.68 / 0.33 |

## 10 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272) |
|---|---|---|---|---|---|
| popularity | 0.144 | 0.305 | 0.200 | 1.000 | 0.01 / 0.00 / 0.00 / 0.01 / 0.02 / 0.01 / 0.38 / 0.42 / 0.53 / 0.06 |
| copy · random neighbours | 0.128 | 0.207 | 0.156 | 0.208 | 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.33 / 0.40 / 0.53 / 0.02 |
| copy · lexical neighbours | 0.266 | 0.381 | 0.309 | 0.388 | 0.08 / 0.07 / 0.10 / 0.14 / 0.08 / 0.04 / 0.45 / 0.70 / 0.75 / 0.25 |
| copy · fingerprint neighbours | 0.133 | 0.242 | 0.170 | 0.248 | 0.01 / 0.01 / 0.01 / 0.03 / 0.03 / 0.00 / 0.31 / 0.38 / 0.52 / 0.04 |
| copy · MARS fused neighbours | 0.160 | 0.269 | 0.197 | 0.276 | 0.04 / 0.03 / 0.03 / 0.05 / 0.04 / 0.02 / 0.35 / 0.45 / 0.54 / 0.06 |
| analogy · lexical neighbours | 0.325 | 0.461 | 0.375 | 0.469 | 0.25 / 0.08 / 0.41 / 0.17 / 0.10 / 0.04 / 0.45 / 0.69 / 0.75 / 0.30 |
| analogy · MARS fused neighbours | 0.252 | 0.385 | 0.296 | 0.394 | 0.24 / 0.05 / 0.46 / 0.12 / 0.06 / 0.02 / 0.35 / 0.46 / 0.54 / 0.23 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.235 | 0.363 | 0.282 | 0.371 | 0.06 / 0.05 / 0.08 / 0.09 / 0.08 / 0.03 / 0.42 / 0.66 / 0.68 / 0.19 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.311 | 0.461 | 0.364 | 0.469 | 0.26 / 0.07 / 0.45 / 0.13 / 0.10 / 0.03 / 0.42 / 0.66 / 0.68 / 0.30 |
| rules (length-1, same object) | 0.177 | 0.211 | 0.190 | 0.211 | 0.37 / 0.09 / 0.57 / 0.22 / 0.08 / 0.04 / 0.01 / 0.07 / 0.01 / 0.32 |
| rules + copy · lexical (vote share + confidence) | 0.342 | 0.525 | 0.405 | 0.536 | 0.39 / 0.09 / 0.39 / 0.16 / 0.13 / 0.05 / 0.45 / 0.70 / 0.75 / 0.31 |
| rules + analogy · hybrid | 0.340 | 0.513 | 0.401 | 0.525 | 0.33 / 0.07 / 0.55 / 0.18 / 0.13 / 0.04 / 0.42 / 0.67 / 0.68 / 0.33 |

## Analogy vs copy (MARS analogues, m = 10)

- Top-1 predictions that are pure substitutions (a query entity reached through the mapping, not an analogue's own object): 647 of 2943 (22.0%), precision 0.405; other top-1s precision 0.215.
- Substitutions per relation (count, precision): wdt:p57 145 (0.43), wdt:p161 41 (0.07), wdt:p58 240 (0.52), wdt:p162 91 (0.24), wdt:p86 10 (0.40), wdt:p344 4 (0.25), wdt:p136 0 (NaN), wdt:p495 1 (0.00), wdt:p364 0 (NaN), wdt:p272 115 (0.38).
- Where analogy's and copy's top-1 differ (688 queries): analogy right 287, copy right 13.

**Calibration** (analogy · MARS, top-1 precision by support = analogues proposing it):

| support | queries | precision |
|---|---|---|
| 1 | 1370 | 0.048 |
| 2 | 427 | 0.283 |
| 3 | 362 | 0.420 |
| 4 | 330 | 0.467 |
| 5+ | 454 | 0.577 |

Runtime 14.0s.
