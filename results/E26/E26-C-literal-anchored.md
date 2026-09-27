# E26: knowledge-graph vocabulary alignment (C-literal-anchored)

Data: `/tmp/claude-0/-home-user-MARS/e9702898-0fe9-58b9-9954-8861366f0496/scratchpad/kgc-C` — 1992 film cases, 170 properties (27 DBpedia, 143 Wikidata); 11 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `literal` profile), wildcard mapping (local score 0.3, attributes included), evidence (`anchored` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster, re-estimated each round. Counterpart retrieval: each DBpedia film ranks the Wikidata films (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.021 / 0.136 | 0.051 / 0.192 |
| 1 | 14 | 6 / 5 / 3 | 0.545 | 0.545 | 0.123 / 0.322 | 0.185 / 0.355 |
| 2 | 16 | 6 / 5 / 5 | 0.545 | 0.545 | 0.119 / 0.324 | 0.184 / 0.358 |
| 3 | 16 | 6 / 5 / 5 | 0.545 | 0.545 | 0.119 / 0.324 | 0.184 / 0.358 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|
| `dbo:starring` | `wdt:p161` | 1165.4 | correct |
| `dbo:director` | `wdt:p57` | 339.7 | correct |
| `dbo:runtime` | `wdt:p2047` | 304.9 | correct |
| `dbo:writer` | `wdt:p58` | 223.3 | not in gold |
| `dbo:distributor` | `wdt:p750` | 176.4 | correct |
| `dbo:producer` | `wdt:p162` | 175.3 | not in gold |
| `dbo:musiccomposer` | `wdt:p86` | 163.5 | **wrong** |
| `dbo:language` | `wdt:p136` | 143.3 | **wrong** |
| `dbo:cinematography` | `wdt:p344` | 108.5 | correct |
| `dbo:productioncompany` | `wdt:p272` | 83.0 | not in gold |
| `dbo:country` | `wdt:p495` | 80.9 | **wrong** |
| `dbo:editing` | `wdt:p1040` | 60.2 | correct |
| `dbo:budget` | `wdt:p2130` | 12.2 | **wrong** |
| `dbo:gross` | `wdt:p2142` | 9.1 | **wrong** |
| `dbo:narrator` | `wdt:p2438` | 6.1 | not in gold |
| `dbo:network` | `wdt:p449` | 4.6 | not in gold |

Gold pairs present but not learned (5): composer↔p86, author↔p50, budget↔p2769, genre↔p136, releasedate↔p577

Runtime 9.1s.
