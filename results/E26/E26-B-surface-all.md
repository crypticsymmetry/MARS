# E26: knowledge-graph vocabulary alignment (B-surface-all)

Data: `/tmp/claude-0/-home-user-MARS/e9702898-0fe9-58b9-9954-8861366f0496/scratchpad/kgc-B` — 1992 film cases, 170 properties (27 DBpedia, 143 Wikidata); 11 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `surface-only` profile), wildcard mapping (local score 0.3, attributes included), evidence (`all` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster, re-estimated each round. Counterpart retrieval: each DBpedia film ranks the Wikidata films (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.027 / 0.052 | 0.048 / 0.082 |
| 1 | 2 | 1 / 1 / 0 | 0.500 | 0.091 | 0.027 / 0.016 | 0.048 / 0.035 |
| 2 | 2 | 1 / 1 / 0 | 0.500 | 0.091 | 0.027 / 0.016 | 0.048 / 0.035 |
| 3 | 2 | 1 / 1 / 0 | 0.500 | 0.091 | 0.027 / 0.016 | 0.048 / 0.035 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|
| `dbo:starring` | `wdt:p161` | 15322.2 | correct |
| `dbo:director` | `wdt:p136` | 6102.2 | **wrong** |

Gold pairs present but not learned (10): composer↔p86, cinematography↔p344, editing↔p1040, author↔p50, budget↔p2769, genre↔p136, director↔p57, runtime↔p2047, distributor↔p750, releasedate↔p577

Runtime 7.7s.
