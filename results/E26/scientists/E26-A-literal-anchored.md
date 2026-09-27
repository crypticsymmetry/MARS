# E26: knowledge-graph vocabulary alignment (A-literal-anchored)

Data: `data/kg-scientists → kg2mars --condition A` — 1993 cases, 126 properties (21 DBpedia, 105 Wikidata); 13 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `literal` profile), wildcard mapping (local score 0.3, attributes included), evidence (`anchored` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster, re-estimated each round. Counterpart retrieval: each DBpedia entity ranks the Wikidata entities (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.000 / 0.000 | 0.006 / 0.003 |
| 1 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.000 / 0.000 | 0.006 / 0.003 |
| 2 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.000 / 0.000 | 0.006 / 0.003 |
| 3 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.000 / 0.000 | 0.006 / 0.003 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|

Gold pairs present but not learned (13): birthplace↔p19, party↔p102, nationality↔p27, spouse↔p26, citizenship↔p27, almamater↔p69, deathdate↔p570, doctoralstudent↔p185, deathplace↔p20, award↔p166, birthdate↔p569, doctoraladvisor↔p184, child↔p40

Runtime 6.8s.
