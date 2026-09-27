# E26: knowledge-graph vocabulary alignment (A-surface-all)

Data: `data/kg-scientists → kg2mars --condition A` — 1993 cases, 126 properties (21 DBpedia, 105 Wikidata); 13 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `surface-only` profile), wildcard mapping (local score 0.3, attributes included), evidence (`all` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster, re-estimated each round. Counterpart retrieval: each DBpedia entity ranks the Wikidata entities (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.000 / 0.000 | 0.007 / 0.004 |
| 1 | 1 | 0 / 1 / 0 | 0.000 | 0.000 | 0.000 / 0.000 | 0.007 / 0.003 |
| 2 | 2 | 0 / 2 / 0 | 0.000 | 0.000 | 0.000 / 0.001 | 0.007 / 0.005 |
| 3 | 3 | 0 / 3 / 0 | 0.000 | 0.000 | 0.000 / 0.002 | 0.007 / 0.007 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|
| `dbo:almamater` | `wdt:p106` | 8234.7 | **wrong** |
| `dbo:academicdiscipline` | `wdt:p108` | 4602.8 | **wrong** |
| `dbo:birthplace` | `wdt:p101` | 1331.8 | **wrong** |

Gold pairs present but not learned (13): birthplace↔p19, party↔p102, nationality↔p27, spouse↔p26, citizenship↔p27, almamater↔p69, deathdate↔p570, doctoralstudent↔p185, deathplace↔p20, award↔p166, birthdate↔p569, doctoraladvisor↔p184, child↔p40

Runtime 6.7s.
