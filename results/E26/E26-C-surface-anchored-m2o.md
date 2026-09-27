# E26: knowledge-graph vocabulary alignment (C-surface-anchored-m2o)

Data: `data/kg-films → kg2mars --condition C` — 1992 film cases, 170 properties (27 DBpedia, 143 Wikidata); 11 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `surface-only` profile), wildcard mapping (local score 0.3, attributes included), evidence (`anchored` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster plus many-to-one joins (evidence ≥ 0.25 × the pair's), re-estimated each round. Counterpart retrieval: each DBpedia film ranks the Wikidata films (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.557 / 0.819 | 0.626 / 0.842 |
| 1 | 16 | 6 / 6 / 4 | 0.500 | 0.545 | 0.557 / 0.617 | 0.626 / 0.656 |
| 2 | 17 | 6 / 6 / 5 | 0.500 | 0.545 | 0.557 / 0.617 | 0.626 / 0.656 |
| 3 | 17 | 6 / 6 / 5 | 0.500 | 0.545 | 0.557 / 0.617 | 0.626 / 0.656 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|
| `dbo:starring` | `wdt:p161` | 3094.2 | correct |
| `dbo:director` | `wdt:p57` | 950.9 | correct |
| `dbo:runtime` | `wdt:p2047` | 857.1 | correct |
| `dbo:writer` | `wdt:p58` | 666.1 | not in gold |
| `dbo:producer` | `wdt:p162` | 531.4 | not in gold |
| `dbo:musiccomposer` | `wdt:p86` | 464.9 | **wrong** |
| `dbo:distributor` | `wdt:p750` | 430.1 | correct |
| `dbo:cinematography` | `wdt:p344` | 325.4 | correct |
| `dbo:productioncompany` | `wdt:p272` | 212.7 | not in gold |
| `dbo:editing` | `wdt:p1040` | 179.3 | correct |
| `dbo:country` | `wdt:p495` | 164.9 | **wrong** |
| `dbo:language` | `wdt:p136` | 130.3 | **wrong** |
| `dbo:language` | `wdt:p364` | 96.2 | **wrong** |
| `dbo:budget` | `wdt:p2130` | 54.9 | **wrong** |
| `dbo:gross` | `wdt:p2142` | 19.4 | **wrong** |
| `dbo:narrator` | `wdt:p2438` | 7.7 | not in gold |
| `dbo:network` | `wdt:p449` | 4.2 | not in gold |

Gold pairs present but not learned (5): composer↔p86, author↔p50, budget↔p2769, genre↔p136, releasedate↔p577

Runtime 9.6s.
