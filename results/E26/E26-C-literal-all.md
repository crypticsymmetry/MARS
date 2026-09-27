# E26: knowledge-graph vocabulary alignment (C-literal-all)

Data: `/tmp/claude-0/-home-user-MARS/e9702898-0fe9-58b9-9954-8861366f0496/scratchpad/kgc-C` — 1992 film cases, 170 properties (27 DBpedia, 143 Wikidata); 11 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `literal` profile), wildcard mapping (local score 0.3, attributes included), evidence (`all` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster, re-estimated each round. Counterpart retrieval: each DBpedia film ranks the Wikidata films (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.021 / 0.136 | 0.051 / 0.192 |
| 1 | 3 | 0 / 2 / 1 | 0.000 | 0.000 | 0.024 / 0.087 | 0.070 / 0.150 |
| 2 | 4 | 0 / 3 / 1 | 0.000 | 0.000 | 0.033 / 0.059 | 0.078 / 0.115 |
| 3 | 5 | 1 / 3 / 1 | 0.250 | 0.091 | 0.037 / 0.080 | 0.084 / 0.131 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|
| `dbo:runtime` | `wdt:p161` | 9191.0 | **wrong** |
| `dbo:starring` | `wdt:p577` | 6622.4 | **wrong** |
| `dbo:writer` | `wdt:p58` | 2359.8 | not in gold |
| `dbo:cinematography` | `wdt:p136` | 2332.2 | **wrong** |
| `dbo:director` | `wdt:p57` | 992.4 | correct |

Gold pairs present but not learned (10): composer↔p86, cinematography↔p344, editing↔p1040, author↔p50, budget↔p2769, genre↔p136, runtime↔p2047, distributor↔p750, releasedate↔p577, starring↔p161

Runtime 6.9s.
