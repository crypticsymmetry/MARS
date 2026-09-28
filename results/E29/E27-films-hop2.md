# E27: knowledge-graph completion by analogy (films-hop2)

Data: `data/kg-films → kg2mars --condition C --hop2` (wikidata side, 992 entity cases). 3000 queries: an entity with all facts of one relation removed (and the labels of objects no longer mentioned), up to 300 per relation, seed 1. Analogues: top-10 (MARS fused = fingerprint-surface top-50 re-ranked by 0.5·FAC + 0.5·FP). Votes weighted by similarity; ties broken by popularity.

## 1 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272) |
|---|---|---|---|---|---|
| popularity | 0.144 | 0.308 | 0.201 | 1.000 | 0.01 / 0.00 / 0.00 / 0.01 / 0.02 / 0.01 / 0.39 / 0.41 / 0.53 / 0.06 |
| copy · random neighbours | 0.056 | 0.058 | 0.057 | 0.058 | 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.17 / 0.18 / 0.20 / 0.00 |
| copy · lexical neighbours | 0.148 | 0.153 | 0.150 | 0.153 | 0.07 / 0.07 / 0.06 / 0.07 / 0.07 / 0.03 / 0.16 / 0.46 / 0.39 / 0.10 |
| copy · fingerprint neighbours | 0.194 | 0.205 | 0.199 | 0.205 | 0.07 / 0.06 / 0.06 / 0.06 / 0.07 / 0.03 / 0.33 / 0.60 / 0.55 / 0.12 |
| copy · MARS fused neighbours | 0.214 | 0.221 | 0.217 | 0.221 | 0.07 / 0.05 / 0.06 / 0.06 / 0.06 / 0.03 / 0.31 / 0.74 / 0.64 / 0.11 |
| analogy · lexical neighbours | 0.162 | 0.168 | 0.165 | 0.168 | 0.15 / 0.07 / 0.11 / 0.07 / 0.06 / 0.03 / 0.16 / 0.48 / 0.39 / 0.10 |
| analogy · MARS fused neighbours | 0.287 | 0.297 | 0.291 | 0.297 | 0.20 / 0.06 / 0.22 / 0.09 / 0.08 / 0.03 / 0.31 / 0.92 / 0.75 / 0.20 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.222 | 0.231 | 0.226 | 0.231 | 0.08 / 0.06 / 0.08 / 0.09 / 0.07 / 0.03 / 0.29 / 0.74 / 0.63 / 0.15 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.255 | 0.266 | 0.260 | 0.266 | 0.19 / 0.06 / 0.19 / 0.09 / 0.07 / 0.03 / 0.29 / 0.79 / 0.66 / 0.19 |
| rules (length-1, same object) | 0.174 | 0.212 | 0.188 | 0.212 | 0.37 / 0.09 / 0.57 / 0.19 / 0.08 / 0.04 / 0.01 / 0.07 / 0.01 / 0.32 |
| rules + copy · lexical (vote share + confidence) | 0.264 | 0.341 | 0.296 | 0.341 | 0.24 / 0.12 / 0.46 / 0.22 / 0.12 / 0.07 / 0.17 / 0.47 / 0.39 / 0.39 |
| rules + analogy · hybrid | 0.321 | 0.411 | 0.359 | 0.412 | 0.23 / 0.11 / 0.44 / 0.19 / 0.10 / 0.06 / 0.29 / 0.79 / 0.66 / 0.34 |
| rules ≤ 2 (length-1 and 2-path) | 0.330 | 0.386 | 0.353 | 0.386 | 0.37 / 0.09 / 0.57 / 0.19 / 0.08 / 0.04 / 0.01 / 0.89 / 0.75 / 0.32 |
| rules ≤ 2 + copy · lexical | 0.343 | 0.436 | 0.382 | 0.436 | 0.24 / 0.12 / 0.46 / 0.22 / 0.12 / 0.07 / 0.17 / 0.91 / 0.74 / 0.39 |
| rules ≤ 2 + analogy · hybrid | 0.350 | 0.448 | 0.392 | 0.449 | 0.23 / 0.11 / 0.44 / 0.19 / 0.10 / 0.06 / 0.29 / 0.92 / 0.82 / 0.34 |
| gated analogy · MARS fused (E29) | 0.289 | 0.297 | 0.293 | 0.297 | 0.20 / 0.06 / 0.24 / 0.10 / 0.08 / 0.03 / 0.31 / 0.92 / 0.76 / 0.20 |
| gated analogy · hybrid (E29) | 0.257 | 0.266 | 0.261 | 0.266 | 0.19 / 0.07 / 0.19 / 0.09 / 0.07 / 0.03 / 0.29 / 0.80 / 0.66 / 0.19 |
| rules ≤ 2 + gated analogy · hybrid (E29) | 0.351 | 0.448 | 0.392 | 0.449 | 0.23 / 0.11 / 0.44 / 0.20 / 0.10 / 0.06 / 0.29 / 0.92 / 0.82 / 0.34 |

## 5 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272) |
|---|---|---|---|---|---|
| popularity | 0.144 | 0.308 | 0.201 | 1.000 | 0.01 / 0.00 / 0.00 / 0.01 / 0.02 / 0.01 / 0.39 / 0.41 / 0.53 / 0.06 |
| copy · random neighbours | 0.121 | 0.165 | 0.140 | 0.165 | 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.00 / 0.33 / 0.37 / 0.50 / 0.00 |
| copy · lexical neighbours | 0.222 | 0.259 | 0.238 | 0.262 | 0.08 / 0.07 / 0.07 / 0.11 / 0.08 / 0.03 / 0.23 / 0.78 / 0.61 / 0.16 |
| copy · fingerprint neighbours | 0.225 | 0.317 | 0.261 | 0.320 | 0.07 / 0.06 / 0.06 / 0.08 / 0.07 / 0.03 / 0.39 / 0.68 / 0.67 / 0.13 |
| copy · MARS fused neighbours | 0.239 | 0.325 | 0.271 | 0.326 | 0.07 / 0.05 / 0.07 / 0.08 / 0.06 / 0.03 / 0.40 / 0.76 / 0.71 / 0.15 |
| analogy · lexical neighbours | 0.258 | 0.315 | 0.282 | 0.317 | 0.22 / 0.08 / 0.23 / 0.11 / 0.09 / 0.03 / 0.23 / 0.82 / 0.60 / 0.17 |
| analogy · MARS fused neighbours | 0.348 | 0.446 | 0.385 | 0.447 | 0.26 / 0.06 / 0.40 / 0.12 / 0.09 / 0.03 / 0.40 / 0.95 / 0.86 / 0.31 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.266 | 0.331 | 0.293 | 0.333 | 0.09 / 0.06 / 0.08 / 0.12 / 0.08 / 0.04 / 0.40 / 0.83 / 0.78 / 0.18 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.340 | 0.418 | 0.372 | 0.419 | 0.28 / 0.08 / 0.36 / 0.14 / 0.09 / 0.03 / 0.40 / 0.94 / 0.82 / 0.27 |
| rules (length-1, same object) | 0.174 | 0.212 | 0.188 | 0.212 | 0.37 / 0.09 / 0.57 / 0.19 / 0.08 / 0.04 / 0.01 / 0.07 / 0.01 / 0.32 |
| rules + copy · lexical (vote share + confidence) | 0.318 | 0.434 | 0.362 | 0.437 | 0.33 / 0.10 / 0.43 / 0.18 / 0.11 / 0.06 / 0.23 / 0.78 / 0.61 / 0.36 |
| rules + analogy · hybrid | 0.376 | 0.492 | 0.420 | 0.494 | 0.32 / 0.08 / 0.54 / 0.18 / 0.11 / 0.05 / 0.40 / 0.94 / 0.82 / 0.32 |
| rules ≤ 2 (length-1 and 2-path) | 0.330 | 0.386 | 0.353 | 0.386 | 0.37 / 0.09 / 0.57 / 0.19 / 0.08 / 0.04 / 0.01 / 0.89 / 0.75 / 0.32 |
| rules ≤ 2 + copy · lexical | 0.351 | 0.475 | 0.398 | 0.478 | 0.33 / 0.10 / 0.43 / 0.18 / 0.11 / 0.06 / 0.23 / 0.92 / 0.80 / 0.36 |
| rules ≤ 2 + analogy · hybrid | 0.382 | 0.499 | 0.426 | 0.501 | 0.32 / 0.08 / 0.54 / 0.18 / 0.11 / 0.05 / 0.40 / 0.95 / 0.86 / 0.32 |
| gated analogy · MARS fused (E29) | 0.368 | 0.446 | 0.399 | 0.447 | 0.30 / 0.06 / 0.49 / 0.16 / 0.10 / 0.03 / 0.40 / 0.95 / 0.86 / 0.34 |
| gated analogy · hybrid (E29) | 0.349 | 0.418 | 0.378 | 0.419 | 0.29 / 0.07 / 0.42 / 0.15 / 0.11 / 0.03 / 0.40 / 0.94 / 0.81 / 0.27 |
| rules ≤ 2 + gated analogy · hybrid (E29) | 0.380 | 0.499 | 0.425 | 0.501 | 0.31 / 0.07 / 0.53 / 0.18 / 0.12 / 0.05 / 0.40 / 0.95 / 0.86 / 0.32 |

## 10 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272) |
|---|---|---|---|---|---|
| popularity | 0.144 | 0.308 | 0.201 | 1.000 | 0.01 / 0.00 / 0.00 / 0.01 / 0.02 / 0.01 / 0.39 / 0.41 / 0.53 / 0.06 |
| copy · random neighbours | 0.130 | 0.202 | 0.156 | 0.202 | 0.00 / 0.00 / 0.01 / 0.00 / 0.01 / 0.00 / 0.35 / 0.38 / 0.53 / 0.02 |
| copy · lexical neighbours | 0.254 | 0.323 | 0.280 | 0.328 | 0.08 / 0.07 / 0.08 / 0.11 / 0.08 / 0.04 / 0.28 / 0.88 / 0.74 / 0.18 |
| copy · fingerprint neighbours | 0.229 | 0.354 | 0.273 | 0.362 | 0.08 / 0.06 / 0.06 / 0.08 / 0.07 / 0.03 / 0.40 / 0.69 / 0.65 / 0.16 |
| copy · MARS fused neighbours | 0.238 | 0.357 | 0.279 | 0.363 | 0.07 / 0.05 / 0.07 / 0.08 / 0.06 / 0.03 / 0.42 / 0.74 / 0.70 / 0.16 |
| analogy · lexical neighbours | 0.309 | 0.396 | 0.343 | 0.401 | 0.26 / 0.08 / 0.32 / 0.12 / 0.09 / 0.04 / 0.28 / 0.93 / 0.74 / 0.23 |
| analogy · MARS fused neighbours | 0.363 | 0.487 | 0.409 | 0.495 | 0.31 / 0.06 / 0.45 / 0.14 / 0.07 / 0.03 / 0.42 / 0.95 / 0.85 / 0.34 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.268 | 0.368 | 0.305 | 0.373 | 0.09 / 0.06 / 0.08 / 0.12 / 0.08 / 0.04 / 0.40 / 0.84 / 0.80 / 0.17 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.360 | 0.476 | 0.403 | 0.480 | 0.31 / 0.09 / 0.43 / 0.15 / 0.09 / 0.03 / 0.40 / 0.96 / 0.84 / 0.31 |
| rules (length-1, same object) | 0.174 | 0.212 | 0.188 | 0.212 | 0.37 / 0.09 / 0.57 / 0.19 / 0.08 / 0.04 / 0.01 / 0.07 / 0.01 / 0.32 |
| rules + copy · lexical (vote share + confidence) | 0.349 | 0.488 | 0.398 | 0.495 | 0.34 / 0.10 / 0.48 / 0.16 / 0.12 / 0.06 / 0.28 / 0.89 / 0.74 / 0.32 |
| rules + analogy · hybrid | 0.383 | 0.517 | 0.431 | 0.524 | 0.32 / 0.08 / 0.55 / 0.17 / 0.10 / 0.04 / 0.40 / 0.97 / 0.84 / 0.36 |
| rules ≤ 2 (length-1 and 2-path) | 0.330 | 0.386 | 0.353 | 0.386 | 0.37 / 0.09 / 0.57 / 0.19 / 0.08 / 0.04 / 0.01 / 0.89 / 0.75 / 0.32 |
| rules ≤ 2 + copy · lexical | 0.364 | 0.502 | 0.413 | 0.509 | 0.34 / 0.10 / 0.48 / 0.16 / 0.12 / 0.06 / 0.28 / 0.93 / 0.86 / 0.32 |
| rules ≤ 2 + analogy · hybrid | 0.384 | 0.519 | 0.432 | 0.525 | 0.32 / 0.08 / 0.55 / 0.17 / 0.10 / 0.04 / 0.40 / 0.96 / 0.86 / 0.36 |
| gated analogy · MARS fused (E29) | 0.382 | 0.488 | 0.422 | 0.495 | 0.32 / 0.07 / 0.55 / 0.18 / 0.11 / 0.02 / 0.42 / 0.95 / 0.86 / 0.34 |
| gated analogy · hybrid (E29) | 0.375 | 0.476 | 0.414 | 0.480 | 0.32 / 0.07 / 0.51 / 0.18 / 0.12 / 0.03 / 0.40 / 0.97 / 0.85 / 0.31 |
| rules ≤ 2 + gated analogy · hybrid (E29) | 0.386 | 0.518 | 0.434 | 0.525 | 0.32 / 0.07 / 0.54 / 0.19 / 0.12 / 0.04 / 0.40 / 0.96 / 0.86 / 0.35 |

## Analogy vs copy (MARS analogues, m = 10)

- Top-1 predictions that are pure substitutions (a query entity reached through the mapping, not an analogue's own object): 1081 of 2954 (36.6%), precision 0.291; other top-1s precision 0.413.
- Substitutions per relation (count, precision): wdt:p57 230 (0.35), wdt:p161 59 (0.05), wdt:p58 257 (0.46), wdt:p162 168 (0.16), wdt:p86 193 (0.04), wdt:p344 13 (0.00), wdt:p136 0 (NaN), wdt:p495 29 (0.90), wdt:p364 21 (0.57), wdt:p272 111 (0.36).
- Where analogy's and copy's top-1 differ (1249 queries): analogy right 445, copy right 71.

**Calibration** (analogy · MARS, top-1 precision by support = analogues proposing it):

| support | queries | precision |
|---|---|---|
| 1 | 904 | 0.069 |
| 2 | 421 | 0.173 |
| 3 | 327 | 0.300 |
| 4 | 316 | 0.427 |
| 5+ | 986 | 0.731 |

## Transfers learned from analogy (E29)

Precision of analogical votes by (relation, transfer type), MARS analogues, all queries (vote-level precision overall 0.250); transfer types with ≥ 30 votes, top 4 per relation. `mined` is the confidence of the same pattern as an explicitly mined rule.

| relation | transfer | votes | precision | mined rule confidence |
|---|---|---|---|---|
| wdt:p57 | wdt:p1040 | 37 | 0.676 | 0.085 |
| wdt:p57 | wdt:p344 | 54 | 0.481 | 0.037 |
| wdt:p57 | wdt:p58 | 955 | 0.468 | 0.400 |
| wdt:p57 | wdt:p161 | 175 | 0.331 | 0.010 |
| wdt:p161 | wdt:p162 | 127 | 0.134 | 0.070 |
| wdt:p161 | wdt:p58 | 131 | 0.107 | 0.079 |
| wdt:p161 | wdt:p57 | 154 | 0.091 | 0.068 |
| wdt:p161 | wdt:p86 | 41 | 0.049 | 0.025 |
| wdt:p58 | wdt:p1040 | 45 | 0.667 | 0.070 |
| wdt:p58 | wdt:p57 | 999 | 0.592 | 0.337 |
| wdt:p58 | wdt:p161 | 252 | 0.381 | 0.010 |
| wdt:p58 | wdt:p162 | 557 | 0.219 | 0.125 |
| wdt:p162 | wdt:p57 | 337 | 0.258 | 0.099 |
| wdt:p162 | wdt:p161 | 150 | 0.247 | 0.007 |
| wdt:p162 | wdt:p58 | 519 | 0.179 | 0.099 |
| wdt:p162 | wdt:p1431 | 44 | 0.136 | 0.143 |
| wdt:p86 | wdt:p175 | 92 | 0.924 | 0.756 |
| wdt:p86 | copy (analogue's own object) | 1116 | 0.034 | — |
| wdt:p86 | wdt:p57 | 131 | 0.008 | 0.005 |
| wdt:p86 | wdt:p58 | 406 | 0.002 | 0.007 |
| wdt:p344 | copy (analogue's own object) | 1830 | 0.011 | — |
| wdt:p136 | copy (analogue's own object) | 4458 | 0.222 | — |
| wdt:p495 | wdt:p1040 · wdt:p27 | 81 | 1.000 | 0.824 |
| wdt:p495 | wdt:p1431 · wdt:p27 | 30 | 1.000 | 0.667 |
| wdt:p495 | wdt:p170 · wdt:p27 | 32 | 1.000 | 1.000 |
| wdt:p495 | wdt:p175 · wdt:p27 | 100 | 1.000 | 0.933 |
| wdt:p364 | wdt:p1040 · wdt:p1412 | 37 | 1.000 | 0.724 |
| wdt:p364 | wdt:p344 · wdt:p1412 | 56 | 1.000 | 0.600 |
| wdt:p364 | wdt:p57 · wdt:p1412 | 1571 | 0.935 | 0.716 |
| wdt:p364 | wdt:p162 · wdt:p1412 | 880 | 0.933 | 0.674 |
| wdt:p272 | wdt:p750 | 728 | 0.485 | 0.140 |
| wdt:p272 | copy (analogue's own object) | 1387 | 0.056 | — |

Runtime 17.3s.
