# E26: knowledge-graph vocabulary alignment (A-surface-anchored)

Data: `/tmp/claude-0/-home-user-MARS/e9702898-0fe9-58b9-9954-8861366f0496/scratchpad/kgc-A` — 1992 film cases, 170 properties (27 DBpedia, 143 Wikidata); 11 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `surface-only` profile), wildcard mapping (local score 0.3, attributes included), evidence (`anchored` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster, re-estimated each round. Counterpart retrieval: each DBpedia film ranks the Wikidata films (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.001 / 0.001 | 0.007 / 0.003 |
| 1 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.001 / 0.001 | 0.007 / 0.003 |
| 2 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.001 / 0.001 | 0.007 / 0.003 |
| 3 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.001 / 0.001 | 0.007 / 0.003 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|

Gold pairs present but not learned (11): composer↔p86, cinematography↔p344, editing↔p1040, author↔p50, budget↔p2769, genre↔p136, director↔p57, runtime↔p2047, distributor↔p750, releasedate↔p577, starring↔p161

Runtime 7.0s.
