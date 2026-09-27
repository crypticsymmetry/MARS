# E26: knowledge-graph vocabulary alignment (C-literal-all)

Data: `data/kg-scientists → kg2mars --condition C` — 1993 cases, 126 properties (21 DBpedia, 105 Wikidata); 13 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `literal` profile), wildcard mapping (local score 0.3, attributes included), evidence (`all` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster, re-estimated each round. Counterpart retrieval: each DBpedia entity ranks the Wikidata entities (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.008 / 0.133 | 0.059 / 0.210 |
| 1 | 4 | 1 / 2 / 1 | 0.333 | 0.077 | 0.057 / 0.162 | 0.105 / 0.212 |
| 2 | 6 | 1 / 4 / 1 | 0.200 | 0.077 | 0.039 / 0.082 | 0.082 / 0.125 |
| 3 | 7 | 1 / 5 / 1 | 0.167 | 0.077 | 0.042 / 0.057 | 0.085 / 0.102 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|
| `dbo:almamater` | `wdt:p108` | 5899.4 | **wrong** |
| `dbo:academicdiscipline` | `wdt:p106` | 5852.6 | **wrong** |
| `dbo:birthplace` | `wdt:p31` | 4410.7 | **wrong** |
| `dbo:institution` | `wdt:p69` | 4248.3 | **wrong** |
| `dbo:knownfor` | `wdt:p735` | 2718.9 | not in gold |
| `dbo:award` | `wdt:p166` | 2540.9 | correct |
| `dbo:birthdate` | `wdt:p101` | 1003.4 | **wrong** |

Gold pairs present but not learned (12): birthplace↔p19, party↔p102, nationality↔p27, spouse↔p26, citizenship↔p27, almamater↔p69, deathdate↔p570, doctoralstudent↔p185, deathplace↔p20, birthdate↔p569, doctoraladvisor↔p184, child↔p40

Runtime 6.6s.
