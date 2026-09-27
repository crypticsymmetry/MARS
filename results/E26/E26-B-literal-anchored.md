# E26: knowledge-graph vocabulary alignment (B-literal-anchored)

Data: `/tmp/claude-0/-home-user-MARS/e9702898-0fe9-58b9-9954-8861366f0496/scratchpad/kgc-B` — 1992 film cases, 170 properties (27 DBpedia, 143 Wikidata); 11 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `literal` profile), wildcard mapping (local score 0.3, attributes included), evidence (`anchored` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster, re-estimated each round. Counterpart retrieval: each DBpedia film ranks the Wikidata films (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.005 / 0.009 | 0.015 / 0.016 |
| 1 | 3 | 1 / 2 / 0 | 0.333 | 0.091 | 0.010 / 0.036 | 0.025 / 0.047 |
| 2 | 3 | 1 / 2 / 0 | 0.333 | 0.091 | 0.010 / 0.036 | 0.025 / 0.047 |
| 3 | 3 | 1 / 2 / 0 | 0.333 | 0.091 | 0.010 / 0.036 | 0.025 / 0.047 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|
| `dbo:runtime` | `wdt:p2047` | 199.9 | correct |
| `dbo:budget` | `wdt:p2130` | 13.6 | **wrong** |
| `dbo:gross` | `wdt:p2142` | 2.0 | **wrong** |

Gold pairs present but not learned (10): composer↔p86, cinematography↔p344, editing↔p1040, author↔p50, budget↔p2769, genre↔p136, director↔p57, distributor↔p750, releasedate↔p577, starring↔p161

Runtime 6.3s.
