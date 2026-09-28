# E27: knowledge-graph completion by analogy (scientists-hop2)

Data: `data/kg-scientists → kg2mars --condition C --hop2` (wikidata side, 993 entity cases). 2400 queries: an entity with all facts of one relation removed (and the labels of objects no longer mentioned), up to 300 per relation, seed 1. Analogues: top-10 (MARS fused = fingerprint-surface top-50 re-ranked by 0.5·FAC + 0.5·FP). Votes weighted by similarity; ties broken by popularity.

## 1 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p69, wdt:p108, wdt:p101, wdt:p27, wdt:p106, wdt:p166, wdt:p463, wdt:p1412) |
|---|---|---|---|---|---|
| popularity | 0.223 | 0.535 | 0.324 | 1.000 | 0.06 / 0.05 / 0.12 / 0.34 / 0.36 / 0.09 / 0.20 / 0.57 |
| copy · random neighbours | 0.068 | 0.075 | 0.071 | 0.075 | 0.01 / 0.01 / 0.00 / 0.10 / 0.16 / 0.02 / 0.04 / 0.21 |
| copy · lexical neighbours | 0.231 | 0.263 | 0.245 | 0.263 | 0.19 / 0.14 / 0.07 / 0.54 / 0.26 / 0.07 / 0.15 / 0.44 |
| copy · fingerprint neighbours | 0.231 | 0.280 | 0.254 | 0.280 | 0.06 / 0.09 / 0.10 / 0.41 / 0.34 / 0.09 / 0.27 / 0.50 |
| copy · MARS fused neighbours | 0.224 | 0.258 | 0.240 | 0.258 | 0.08 / 0.09 / 0.09 / 0.46 / 0.27 / 0.07 / 0.27 / 0.47 |
| analogy · lexical neighbours | 0.235 | 0.265 | 0.249 | 0.265 | 0.25 / 0.16 / 0.07 / 0.50 / 0.25 / 0.07 / 0.15 / 0.44 |
| analogy · MARS fused neighbours | 0.270 | 0.313 | 0.290 | 0.313 | 0.23 / 0.11 / 0.13 / 0.60 / 0.26 / 0.07 / 0.26 / 0.49 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.265 | 0.303 | 0.282 | 0.303 | 0.14 / 0.12 / 0.07 / 0.56 / 0.29 / 0.11 / 0.28 / 0.55 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.292 | 0.333 | 0.311 | 0.333 | 0.27 / 0.16 / 0.08 / 0.60 / 0.29 / 0.11 / 0.28 / 0.56 |
| rules (length-1, same object) | 0.109 | 0.130 | 0.118 | 0.130 | 0.22 / 0.26 / 0.07 / 0.07 / 0.00 / 0.02 / 0.04 / 0.20 |
| rules + copy · lexical (vote share + confidence) | 0.265 | 0.341 | 0.297 | 0.341 | 0.22 / 0.20 / 0.10 / 0.55 / 0.26 / 0.08 / 0.17 / 0.53 |
| rules + analogy · hybrid | 0.312 | 0.396 | 0.348 | 0.396 | 0.25 / 0.23 / 0.12 / 0.61 / 0.29 / 0.11 / 0.28 / 0.62 |
| rules ≤ 2 (length-1 and 2-path) | 0.220 | 0.290 | 0.250 | 0.292 | 0.35 / 0.27 / 0.20 / 0.69 / 0.00 / 0.02 / 0.03 / 0.20 |
| rules ≤ 2 + copy · lexical | 0.299 | 0.421 | 0.348 | 0.422 | 0.23 / 0.20 / 0.17 / 0.75 / 0.26 / 0.08 / 0.17 / 0.53 |
| rules ≤ 2 + analogy · hybrid | 0.345 | 0.457 | 0.390 | 0.458 | 0.31 / 0.24 / 0.18 / 0.73 / 0.29 / 0.11 / 0.27 / 0.62 |
| gated analogy · MARS fused (E29) | 0.282 | 0.312 | 0.296 | 0.313 | 0.27 / 0.12 / 0.14 / 0.63 / 0.26 / 0.08 / 0.26 / 0.50 |
| gated analogy · hybrid (E29) | 0.303 | 0.333 | 0.317 | 0.333 | 0.29 / 0.18 / 0.08 / 0.63 / 0.29 / 0.11 / 0.27 / 0.56 |
| rules ≤ 2 + gated analogy · hybrid (E29) | 0.344 | 0.457 | 0.392 | 0.458 | 0.31 / 0.24 / 0.18 / 0.73 / 0.29 / 0.11 / 0.27 / 0.62 |

## 5 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p69, wdt:p108, wdt:p101, wdt:p27, wdt:p106, wdt:p166, wdt:p463, wdt:p1412) |
|---|---|---|---|---|---|
| popularity | 0.223 | 0.535 | 0.324 | 1.000 | 0.06 / 0.05 / 0.12 / 0.34 / 0.36 / 0.09 / 0.20 / 0.57 |
| copy · random neighbours | 0.162 | 0.258 | 0.199 | 0.258 | 0.03 / 0.02 / 0.03 / 0.28 / 0.27 / 0.04 / 0.14 / 0.48 |
| copy · lexical neighbours | 0.371 | 0.525 | 0.426 | 0.525 | 0.23 / 0.18 / 0.13 / 0.76 / 0.30 / 0.16 / 0.38 / 0.82 |
| copy · fingerprint neighbours | 0.301 | 0.494 | 0.367 | 0.504 | 0.08 / 0.12 / 0.13 / 0.53 / 0.37 / 0.13 / 0.37 / 0.68 |
| copy · MARS fused neighbours | 0.309 | 0.478 | 0.368 | 0.481 | 0.09 / 0.11 / 0.15 / 0.54 / 0.33 / 0.18 / 0.39 / 0.70 |
| analogy · lexical neighbours | 0.386 | 0.537 | 0.442 | 0.538 | 0.31 / 0.22 / 0.14 / 0.76 / 0.31 / 0.17 / 0.37 / 0.82 |
| analogy · MARS fused neighbours | 0.377 | 0.552 | 0.443 | 0.554 | 0.32 / 0.18 / 0.20 / 0.74 / 0.33 / 0.18 / 0.37 / 0.71 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.367 | 0.543 | 0.429 | 0.545 | 0.17 / 0.13 / 0.13 / 0.73 / 0.34 / 0.21 / 0.46 / 0.78 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.398 | 0.576 | 0.465 | 0.579 | 0.32 / 0.19 / 0.15 / 0.77 / 0.33 / 0.21 / 0.43 / 0.79 |
| rules (length-1, same object) | 0.109 | 0.130 | 0.118 | 0.130 | 0.22 / 0.26 / 0.07 / 0.07 / 0.00 / 0.02 / 0.04 / 0.20 |
| rules + copy · lexical (vote share + confidence) | 0.388 | 0.563 | 0.451 | 0.565 | 0.27 / 0.22 / 0.15 / 0.76 / 0.30 / 0.17 / 0.39 / 0.85 |
| rules + analogy · hybrid | 0.406 | 0.597 | 0.477 | 0.599 | 0.31 / 0.22 / 0.17 / 0.77 / 0.33 / 0.21 / 0.42 / 0.81 |
| rules ≤ 2 (length-1 and 2-path) | 0.220 | 0.290 | 0.250 | 0.292 | 0.35 / 0.27 / 0.20 / 0.69 / 0.00 / 0.02 / 0.03 / 0.20 |
| rules ≤ 2 + copy · lexical | 0.410 | 0.593 | 0.475 | 0.595 | 0.35 / 0.22 / 0.19 / 0.81 / 0.30 / 0.17 / 0.39 / 0.85 |
| rules ≤ 2 + analogy · hybrid | 0.422 | 0.607 | 0.492 | 0.610 | 0.40 / 0.24 / 0.19 / 0.77 / 0.33 / 0.21 / 0.42 / 0.81 |
| gated analogy · MARS fused (E29) | 0.395 | 0.552 | 0.455 | 0.554 | 0.38 / 0.23 / 0.21 / 0.75 / 0.32 / 0.17 / 0.37 / 0.72 |
| gated analogy · hybrid (E29) | 0.419 | 0.577 | 0.480 | 0.579 | 0.41 / 0.24 / 0.18 / 0.76 / 0.34 / 0.21 / 0.42 / 0.79 |
| rules ≤ 2 + gated analogy · hybrid (E29) | 0.423 | 0.607 | 0.492 | 0.610 | 0.41 / 0.24 / 0.19 / 0.77 / 0.34 / 0.21 / 0.42 / 0.81 |

## 10 analogues

| method | Hits@1 | Hits@10 | MRR | coverage | per relation Hits@1 (wdt:p69, wdt:p108, wdt:p101, wdt:p27, wdt:p106, wdt:p166, wdt:p463, wdt:p1412) |
|---|---|---|---|---|---|
| popularity | 0.223 | 0.535 | 0.324 | 1.000 | 0.06 / 0.05 / 0.12 / 0.34 / 0.36 / 0.09 / 0.20 / 0.57 |
| copy · random neighbours | 0.178 | 0.352 | 0.238 | 0.356 | 0.02 / 0.03 / 0.06 / 0.30 / 0.29 / 0.06 / 0.17 / 0.51 |
| copy · lexical neighbours | 0.406 | 0.600 | 0.471 | 0.623 | 0.27 / 0.20 / 0.16 / 0.79 / 0.33 / 0.20 / 0.44 / 0.86 |
| copy · fingerprint neighbours | 0.320 | 0.550 | 0.396 | 0.592 | 0.09 / 0.12 / 0.16 / 0.54 / 0.38 / 0.16 / 0.40 / 0.71 |
| copy · MARS fused neighbours | 0.319 | 0.545 | 0.397 | 0.578 | 0.10 / 0.09 / 0.16 / 0.53 / 0.36 / 0.21 / 0.40 / 0.71 |
| analogy · lexical neighbours | 0.425 | 0.616 | 0.491 | 0.632 | 0.36 / 0.23 / 0.19 / 0.81 / 0.33 / 0.21 / 0.41 / 0.87 |
| analogy · MARS fused neighbours | 0.401 | 0.625 | 0.481 | 0.647 | 0.35 / 0.22 / 0.20 / 0.76 / 0.36 / 0.21 / 0.39 / 0.72 |
| copy · hybrid neighbours (RRF MARS + lexical) | 0.390 | 0.617 | 0.470 | 0.645 | 0.18 / 0.14 / 0.15 / 0.73 / 0.37 / 0.24 / 0.50 / 0.81 |
| analogy · hybrid neighbours (RRF MARS + lexical) | 0.434 | 0.653 | 0.513 | 0.673 | 0.36 / 0.22 / 0.20 / 0.79 / 0.36 / 0.24 / 0.47 / 0.83 |
| rules (length-1, same object) | 0.109 | 0.130 | 0.118 | 0.130 | 0.22 / 0.26 / 0.07 / 0.07 / 0.00 / 0.02 / 0.04 / 0.20 |
| rules + copy · lexical (vote share + confidence) | 0.420 | 0.625 | 0.490 | 0.650 | 0.28 / 0.24 / 0.18 / 0.79 / 0.33 / 0.21 / 0.44 / 0.89 |
| rules + analogy · hybrid | 0.433 | 0.662 | 0.516 | 0.684 | 0.33 / 0.23 / 0.21 / 0.79 / 0.37 / 0.24 / 0.47 / 0.84 |
| rules ≤ 2 (length-1 and 2-path) | 0.220 | 0.290 | 0.250 | 0.292 | 0.35 / 0.27 / 0.20 / 0.69 / 0.00 / 0.02 / 0.03 / 0.20 |
| rules ≤ 2 + copy · lexical | 0.433 | 0.648 | 0.509 | 0.671 | 0.35 / 0.22 / 0.22 / 0.80 / 0.33 / 0.21 / 0.44 / 0.89 |
| rules ≤ 2 + analogy · hybrid | 0.445 | 0.666 | 0.526 | 0.689 | 0.42 / 0.24 / 0.21 / 0.78 / 0.37 / 0.24 / 0.47 / 0.84 |
| gated analogy · MARS fused (E29) | 0.415 | 0.627 | 0.491 | 0.647 | 0.40 / 0.26 / 0.22 / 0.76 / 0.36 / 0.21 / 0.39 / 0.73 |
| gated analogy · hybrid (E29) | 0.441 | 0.654 | 0.520 | 0.673 | 0.41 / 0.23 / 0.19 / 0.77 / 0.36 / 0.25 / 0.47 / 0.84 |
| rules ≤ 2 + gated analogy · hybrid (E29) | 0.442 | 0.666 | 0.524 | 0.689 | 0.41 / 0.24 / 0.19 / 0.77 / 0.36 / 0.24 / 0.47 / 0.84 |

## Analogy vs copy (MARS analogues, m = 10)

- Top-1 predictions that are pure substitutions (a query entity reached through the mapping, not an analogue's own object): 465 of 2396 (19.4%), precision 0.275; other top-1s precision 0.432.
- Substitutions per relation (count, precision): wdt:p69 185 (0.30), wdt:p108 180 (0.19), wdt:p101 46 (0.24), wdt:p27 50 (0.56), wdt:p106 0 (NaN), wdt:p166 0 (NaN), wdt:p463 4 (0.00), wdt:p1412 0 (NaN).
- Where analogy's and copy's top-1 differ (694 queries): analogy right 258, copy right 61.

**Calibration** (analogy · MARS, top-1 precision by support = analogues proposing it):

| support | queries | precision |
|---|---|---|
| 1 | 407 | 0.103 |
| 2 | 367 | 0.188 |
| 3 | 372 | 0.320 |
| 4 | 352 | 0.375 |
| 5+ | 898 | 0.668 |

## Transfers learned from analogy (E29)

Precision of analogical votes by (relation, transfer type), MARS analogues, all queries (vote-level precision overall 0.226); transfer types with ≥ 30 votes, top 4 per relation. `mined` is the confidence of the same pattern as an explicitly mined rule.

| relation | transfer | votes | precision | mined rule confidence |
|---|---|---|---|---|
| wdt:p69 | wdt:p1066 · wdt:p69 | 36 | 0.833 | 0.222 |
| wdt:p69 | wdt:p1066 · wdt:p108 | 44 | 0.795 | 0.170 |
| wdt:p69 | wdt:p184 · wdt:p108 | 698 | 0.716 | 0.309 |
| wdt:p69 | wdt:p184 · wdt:p69 | 578 | 0.621 | 0.215 |
| wdt:p108 | wdt:p1066 · wdt:p69 | 39 | 0.462 | 0.099 |
| wdt:p108 | wdt:p1066 · wdt:p108 | 67 | 0.403 | 0.125 |
| wdt:p108 | wdt:p184 · wdt:p108 | 524 | 0.372 | 0.129 |
| wdt:p108 | wdt:p184 · wdt:p69 | 247 | 0.296 | 0.081 |
| wdt:p101 | wdt:p2650 | 52 | 0.577 | 0.571 |
| wdt:p101 | wdt:p1066 · wdt:p101 | 85 | 0.412 | 0.261 |
| wdt:p101 | wdt:p184 · wdt:p101 | 725 | 0.360 | 0.184 |
| wdt:p101 | copy (analogue's own object) | 2789 | 0.039 | — |
| wdt:p27 | wdt:p19 · wdt:p17 | 1430 | 0.852 | 0.627 |
| wdt:p27 | wdt:p551 · wdt:p17 | 231 | 0.848 | 0.632 |
| wdt:p27 | wdt:p69 · wdt:p27 | 32 | 0.812 | 0.750 |
| wdt:p27 | wdt:p108 · wdt:p17 | 1740 | 0.799 | 0.551 |
| wdt:p106 | copy (analogue's own object) | 6385 | 0.214 | — |
| wdt:p166 | copy (analogue's own object) | 4715 | 0.091 | — |
| wdt:p463 | wdt:p184 · wdt:p108 | 31 | 0.355 | 0.003 |
| wdt:p463 | copy (analogue's own object) | 4504 | 0.198 | — |
| wdt:p463 | wdt:p108 | 148 | 0.162 | 0.010 |
| wdt:p463 | wdt:p69 | 49 | 0.000 | 0.000 |
| wdt:p1412 | wdt:p103 | 145 | 1.000 | 0.950 |
| wdt:p1412 | wdt:p6886 | 160 | 1.000 | 1.000 |
| wdt:p1412 | copy (analogue's own object) | 2265 | 0.501 | — |

Runtime 53.8s.
