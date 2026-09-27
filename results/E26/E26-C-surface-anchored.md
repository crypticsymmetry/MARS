# E26: knowledge-graph vocabulary alignment (C-surface-anchored)

Data: `/tmp/claude-0/-home-user-MARS/e9702898-0fe9-58b9-9954-8861366f0496/scratchpad/kgc-C` — 1992 film cases, 170 properties (27 DBpedia, 143 Wikidata); 11 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `surface-only` profile), wildcard mapping (local score 0.3, attributes included), evidence (`anchored` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster, re-estimated each round. Counterpart retrieval: each DBpedia film ranks the Wikidata films (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.557 / 0.819 | 0.626 / 0.842 |
| 1 | 15 | 6 / 5 / 4 | 0.545 | 0.545 | 0.557 / 0.610 | 0.626 / 0.652 |
| 2 | 16 | 6 / 5 / 5 | 0.545 | 0.545 | 0.557 / 0.610 | 0.626 / 0.652 |
| 3 | 16 | 6 / 5 / 5 | 0.545 | 0.545 | 0.557 / 0.610 | 0.626 / 0.652 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|
| `dbo:starring` | `wdt:p161` | 3099.1 | correct |
| `dbo:director` | `wdt:p57` | 952.3 | correct |
| `dbo:runtime` | `wdt:p2047` | 859.8 | correct |
| `dbo:writer` | `wdt:p58` | 666.9 | not in gold |
| `dbo:producer` | `wdt:p162` | 532.2 | not in gold |
| `dbo:musiccomposer` | `wdt:p86` | 465.4 | **wrong** |
| `dbo:distributor` | `wdt:p750` | 431.1 | correct |
| `dbo:cinematography` | `wdt:p344` | 325.7 | correct |
| `dbo:productioncompany` | `wdt:p272` | 213.1 | not in gold |
| `dbo:editing` | `wdt:p1040` | 179.6 | correct |
| `dbo:country` | `wdt:p495` | 164.3 | **wrong** |
| `dbo:language` | `wdt:p136` | 130.0 | **wrong** |
| `dbo:budget` | `wdt:p2130` | 55.0 | **wrong** |
| `dbo:gross` | `wdt:p2142` | 19.4 | **wrong** |
| `dbo:narrator` | `wdt:p2438` | 7.7 | not in gold |
| `dbo:network` | `wdt:p449` | 4.2 | not in gold |

Gold pairs present but not learned (5): composer↔p86, author↔p50, budget↔p2769, genre↔p136, releasedate↔p577

Runtime 8.2s.
