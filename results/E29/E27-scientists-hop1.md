# E27: knowledge-graph completion by analogy (scientists-hop1)

Data: `data/kg-scientists → kg2mars --condition C` (wikidata side, 993 entity cases). 2400 queries: an entity with all facts of one relation removed (and the labels of objects no longer mentioned), up to 300 per relation, seed 1. Analogues: top-10 (MARS fused = fingerprint-surface top-50 re-ranked by 0.5·FAC + 0.5·FP). Votes weighted by similarity; ties broken by popularity.

## 1 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p69, wdt:p108, wdt:p101, wdt:p27, wdt:p106, wdt:p166, wdt:p463, wdt:p1412) |
|---|---|---|---|---|---|
| popularity | 0.222 | 0.535 | 0.322 | 1.000 | 0.06 / 0.04 / 0.12 / 0.33 / 0.36 / 0.09 / 0.20 / 0.57 |
| copy · random neighbours | 0.067 | 0.073 | 0.070 | 0.073 | 0.01 / 0.00 / 0.02 / 0.11 / 0.13 / 0.02 / 0.04 / 0.21 |
| copy · lexical neighbours | 0.214 | 0.245 | 0.228 | 0.245 | 0.12 / 0.13 / 0.06 / 0.37 / 0.29 / 0.09 / 0.26 / 0.39 |
| copy · fingerprint neighbours | 0.217 | 0.260 | 0.237 | 0.260 | 0.06 / 0.06 / 0.09 / 0.33 / 0.30 / 0.10 / 0.28 / 0.51 |
| copy · MARS fused neighbours | 0.213 | 0.245 | 0.228 | 0.245 | 0.06 / 0.06 / 0.08 / 0.36 / 0.31 / 0.10 / 0.26 / 0.47 |
| analogy · lexical neighbours | 0.213 | 0.243 | 0.227 | 0.243 | 0.11 / 0.14 / 0.06 / 0.36 / 0.28 / 0.09 / 0.26 / 0.40 |
| analogy · MARS fused neighbours | 0.220 | 0.255 | 0.236 | 0.255 | 0.07 / 0.09 / 0.09 / 0.35 / 0.31 / 0.10 / 0.26 / 0.49 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.238 | 0.273 | 0.254 | 0.273 | 0.10 / 0.10 / 0.07 / 0.40 / 0.32 / 0.12 / 0.31 / 0.48 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.238 | 0.275 | 0.256 | 0.275 | 0.09 / 0.11 / 0.08 / 0.39 / 0.32 / 0.12 / 0.31 / 0.48 |
| rules (length-1, same object) | 0.109 | 0.130 | 0.118 | 0.130 | 0.22 / 0.26 / 0.07 / 0.07 / 0.00 / 0.02 / 0.04 / 0.20 |
| rules + copy · lexical (vote share + confidence) | 0.248 | 0.326 | 0.281 | 0.326 | 0.17 / 0.19 / 0.09 / 0.40 / 0.29 / 0.10 / 0.28 / 0.47 |
| rules + analogy · hybrid | 0.263 | 0.355 | 0.302 | 0.355 | 0.11 / 0.18 / 0.11 / 0.41 / 0.32 / 0.12 / 0.32 / 0.53 |
| rules ≤ 2 (length-1 and 2-path) | 0.109 | 0.130 | 0.118 | 0.130 | 0.22 / 0.26 / 0.07 / 0.07 / 0.00 / 0.02 / 0.04 / 0.20 |
| rules ≤ 2 + copy · lexical | 0.248 | 0.326 | 0.281 | 0.326 | 0.17 / 0.19 / 0.09 / 0.40 / 0.29 / 0.10 / 0.28 / 0.47 |
| rules ≤ 2 + analogy · hybrid | 0.263 | 0.355 | 0.302 | 0.355 | 0.11 / 0.18 / 0.11 / 0.41 / 0.32 / 0.12 / 0.32 / 0.53 |
| gated analogy · MARS fused (E29) | 0.225 | 0.255 | 0.238 | 0.255 | 0.08 / 0.11 / 0.09 / 0.35 / 0.31 / 0.10 / 0.26 / 0.50 |
| gated analogy · hybrid (E29) | 0.242 | 0.275 | 0.257 | 0.275 | 0.10 / 0.13 / 0.08 / 0.39 / 0.32 / 0.12 / 0.31 / 0.49 |
| rules ≤ 2 + gated analogy · hybrid (E29) | 0.263 | 0.355 | 0.303 | 0.355 | 0.11 / 0.18 / 0.11 / 0.41 / 0.32 / 0.12 / 0.32 / 0.53 |

## 5 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p69, wdt:p108, wdt:p101, wdt:p27, wdt:p106, wdt:p166, wdt:p463, wdt:p1412) |
|---|---|---|---|---|---|
| popularity | 0.222 | 0.535 | 0.322 | 1.000 | 0.06 / 0.04 / 0.12 / 0.33 / 0.36 / 0.09 / 0.20 / 0.57 |
| copy · random neighbours | 0.161 | 0.263 | 0.201 | 0.264 | 0.03 / 0.02 / 0.04 / 0.28 / 0.26 / 0.04 / 0.14 / 0.48 |
| copy · lexical neighbours | 0.344 | 0.504 | 0.401 | 0.506 | 0.16 / 0.16 / 0.13 / 0.61 / 0.34 / 0.19 / 0.42 / 0.74 |
| copy · fingerprint neighbours | 0.281 | 0.482 | 0.351 | 0.492 | 0.06 / 0.08 / 0.13 / 0.45 / 0.33 / 0.15 / 0.37 / 0.68 |
| copy · MARS fused neighbours | 0.293 | 0.477 | 0.359 | 0.480 | 0.09 / 0.07 / 0.12 / 0.50 / 0.34 / 0.19 / 0.37 / 0.67 |
| analogy · lexical neighbours | 0.345 | 0.505 | 0.403 | 0.508 | 0.16 / 0.19 / 0.13 / 0.60 / 0.34 / 0.19 / 0.41 / 0.74 |
| analogy · MARS fused neighbours | 0.312 | 0.500 | 0.377 | 0.504 | 0.13 / 0.15 / 0.13 / 0.49 / 0.34 / 0.19 / 0.37 / 0.69 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.339 | 0.533 | 0.409 | 0.534 | 0.14 / 0.11 / 0.11 / 0.58 / 0.39 / 0.20 / 0.43 / 0.74 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.345 | 0.545 | 0.416 | 0.547 | 0.14 / 0.14 / 0.12 / 0.57 / 0.38 / 0.20 / 0.43 / 0.76 |
| rules (length-1, same object) | 0.109 | 0.130 | 0.118 | 0.130 | 0.22 / 0.26 / 0.07 / 0.07 / 0.00 / 0.02 / 0.04 / 0.20 |
| rules + copy · lexical (vote share + confidence) | 0.362 | 0.550 | 0.429 | 0.552 | 0.20 / 0.19 / 0.15 / 0.61 / 0.34 / 0.20 / 0.43 / 0.78 |
| rules + analogy · hybrid | 0.363 | 0.578 | 0.440 | 0.578 | 0.20 / 0.20 / 0.14 / 0.57 / 0.38 / 0.21 / 0.43 / 0.78 |
| rules ≤ 2 (length-1 and 2-path) | 0.109 | 0.130 | 0.118 | 0.130 | 0.22 / 0.26 / 0.07 / 0.07 / 0.00 / 0.02 / 0.04 / 0.20 |
| rules ≤ 2 + copy · lexical | 0.362 | 0.550 | 0.429 | 0.552 | 0.20 / 0.19 / 0.15 / 0.61 / 0.34 / 0.20 / 0.43 / 0.78 |
| rules ≤ 2 + analogy · hybrid | 0.363 | 0.578 | 0.440 | 0.578 | 0.20 / 0.20 / 0.14 / 0.57 / 0.38 / 0.21 / 0.43 / 0.78 |
| gated analogy · MARS fused (E29) | 0.319 | 0.501 | 0.385 | 0.504 | 0.17 / 0.17 / 0.13 / 0.49 / 0.35 / 0.19 / 0.37 / 0.69 |
| gated analogy · hybrid (E29) | 0.360 | 0.546 | 0.427 | 0.547 | 0.20 / 0.18 / 0.13 / 0.57 / 0.39 / 0.21 / 0.43 / 0.77 |
| rules ≤ 2 + gated analogy · hybrid (E29) | 0.364 | 0.578 | 0.441 | 0.578 | 0.20 / 0.19 / 0.14 / 0.57 / 0.39 / 0.21 / 0.43 / 0.78 |

## 10 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p69, wdt:p108, wdt:p101, wdt:p27, wdt:p106, wdt:p166, wdt:p463, wdt:p1412) |
|---|---|---|---|---|---|
| popularity | 0.222 | 0.535 | 0.322 | 1.000 | 0.06 / 0.04 / 0.12 / 0.33 / 0.36 / 0.09 / 0.20 / 0.57 |
| copy · random neighbours | 0.179 | 0.348 | 0.237 | 0.353 | 0.02 / 0.02 / 0.05 / 0.30 / 0.31 / 0.06 / 0.17 / 0.51 |
| copy · lexical neighbours | 0.378 | 0.601 | 0.454 | 0.620 | 0.17 / 0.15 / 0.15 / 0.65 / 0.36 / 0.23 / 0.48 / 0.82 |
| copy · fingerprint neighbours | 0.311 | 0.542 | 0.388 | 0.586 | 0.07 / 0.09 / 0.14 / 0.51 / 0.38 / 0.20 / 0.38 / 0.71 |
| copy · MARS fused neighbours | 0.318 | 0.541 | 0.395 | 0.569 | 0.10 / 0.08 / 0.15 / 0.50 / 0.38 / 0.21 / 0.40 / 0.71 |
| analogy · lexical neighbours | 0.385 | 0.605 | 0.461 | 0.625 | 0.19 / 0.20 / 0.16 / 0.64 / 0.36 / 0.23 / 0.47 / 0.83 |
| analogy · MARS fused neighbours | 0.343 | 0.570 | 0.421 | 0.597 | 0.17 / 0.18 / 0.17 / 0.50 / 0.38 / 0.21 / 0.40 / 0.73 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.372 | 0.595 | 0.449 | 0.621 | 0.15 / 0.12 / 0.16 / 0.61 / 0.44 / 0.26 / 0.48 / 0.78 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.387 | 0.612 | 0.463 | 0.635 | 0.18 / 0.18 / 0.17 / 0.60 / 0.43 / 0.26 / 0.48 / 0.79 |
| rules (length-1, same object) | 0.109 | 0.130 | 0.118 | 0.130 | 0.22 / 0.26 / 0.07 / 0.07 / 0.00 / 0.02 / 0.04 / 0.20 |
| rules + copy · lexical (vote share + confidence) | 0.400 | 0.633 | 0.482 | 0.656 | 0.23 / 0.22 / 0.17 / 0.65 / 0.36 / 0.23 / 0.48 / 0.85 |
| rules + analogy · hybrid | 0.396 | 0.629 | 0.477 | 0.652 | 0.21 / 0.21 / 0.18 / 0.59 / 0.44 / 0.26 / 0.47 / 0.81 |
| rules ≤ 2 (length-1 and 2-path) | 0.109 | 0.130 | 0.118 | 0.130 | 0.22 / 0.26 / 0.07 / 0.07 / 0.00 / 0.02 / 0.04 / 0.20 |
| rules ≤ 2 + copy · lexical | 0.400 | 0.633 | 0.482 | 0.656 | 0.23 / 0.22 / 0.17 / 0.65 / 0.36 / 0.23 / 0.48 / 0.85 |
| rules ≤ 2 + analogy · hybrid | 0.396 | 0.629 | 0.477 | 0.652 | 0.21 / 0.21 / 0.18 / 0.59 / 0.44 / 0.26 / 0.47 / 0.81 |
| gated analogy · MARS fused (E29) | 0.350 | 0.576 | 0.429 | 0.597 | 0.19 / 0.21 / 0.18 / 0.50 / 0.37 / 0.21 / 0.40 / 0.74 |
| gated analogy · hybrid (E29) | 0.393 | 0.614 | 0.469 | 0.635 | 0.21 / 0.19 / 0.18 / 0.60 / 0.43 / 0.26 / 0.48 / 0.80 |
| rules ≤ 2 + gated analogy · hybrid (E29) | 0.395 | 0.629 | 0.477 | 0.652 | 0.21 / 0.21 / 0.18 / 0.59 / 0.43 / 0.26 / 0.48 / 0.81 |

## Analogy vs copy (MARS analogues, m = 10)

- Top-1 predictions that are pure substitutions (a query entity reached through the mapping, not an analogue's own object): 257 of 2391 (10.7%), precision 0.214; other top-1s precision 0.359.
- Substitutions per relation (count, precision): wdt:p69 109 (0.21), wdt:p108 136 (0.19), wdt:p101 7 (0.57), wdt:p27 3 (0.00), wdt:p106 0 (NaN), wdt:p166 0 (NaN), wdt:p463 0 (NaN), wdt:p1412 2 (1.00).
- Where analogy's and copy's top-1 differ (347 queries): analogy right 89, copy right 29.

**Calibration** (analogy · MARS, top-1 precision by support = analogues proposing it):

| support | queries | precision |
|---|---|---|
| 1 | 449 | 0.076 |
| 2 | 580 | 0.184 |
| 3 | 484 | 0.341 |
| 4 | 322 | 0.453 |
| 5+ | 556 | 0.665 |

## Transfers learned from analogy (E29)

Precision of analogical votes by (relation, transfer type), MARS analogues, all queries (vote-level precision overall 0.181); transfer types with ≥ 30 votes, top 4 per relation. `mined` is the confidence of the same pattern as an explicitly mined rule.

| relation | transfer | votes | precision | mined rule confidence |
|---|---|---|---|---|
| wdt:p69 | wdt:p108 | 751 | 0.230 | 0.182 |
| wdt:p69 | copy (analogue's own object) | 3907 | 0.029 | — |
| wdt:p108 | wdt:p69 | 809 | 0.225 | 0.163 |
| wdt:p108 | wdt:p463 | 38 | 0.211 | 0.015 |
| wdt:p108 | copy (analogue's own object) | 3179 | 0.031 | — |
| wdt:p101 | wdt:p2650 | 43 | 0.581 | 0.571 |
| wdt:p101 | copy (analogue's own object) | 3208 | 0.065 | — |
| wdt:p101 | wdt:p106 | 30 | 0.000 | 0.004 |
| wdt:p27 | wdt:p551 | 61 | 0.541 | 0.219 |
| wdt:p27 | copy (analogue's own object) | 3068 | 0.289 | — |
| wdt:p27 | wdt:p19 | 67 | 0.119 | 0.039 |
| wdt:p106 | copy (analogue's own object) | 6292 | 0.223 | — |
| wdt:p166 | copy (analogue's own object) | 4471 | 0.091 | — |
| wdt:p463 | wdt:p108 | 76 | 0.487 | 0.010 |
| wdt:p463 | copy (analogue's own object) | 4738 | 0.212 | — |
| wdt:p1412 | wdt:p6886 | 154 | 1.000 | 1.000 |
| wdt:p1412 | wdt:p103 | 152 | 0.993 | 0.950 |
| wdt:p1412 | copy (analogue's own object) | 2322 | 0.496 | — |

Runtime 9.3s.
