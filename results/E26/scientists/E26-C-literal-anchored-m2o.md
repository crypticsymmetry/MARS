# E26: knowledge-graph vocabulary alignment (C-literal-anchored-m2o)

Data: `data/kg-scientists → kg2mars --condition C` — 1993 cases, 126 properties (21 DBpedia, 105 Wikidata); 13 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `literal` profile), wildcard mapping (local score 0.3, attributes included), evidence (`anchored` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster plus many-to-one joins (evidence ≥ 0.25 × the pair's), re-estimated each round. Counterpart retrieval: each DBpedia entity ranks the Wikidata entities (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.008 / 0.133 | 0.059 / 0.210 |
| 1 | 14 | 10 / 4 / 0 | 0.714 | 0.769 | 0.147 / 0.335 | 0.215 / 0.385 |
| 2 | 14 | 11 / 3 / 0 | 0.786 | 0.846 | 0.156 / 0.363 | 0.234 / 0.411 |
| 3 | 15 | 12 / 3 / 0 | 0.800 | 0.923 | 0.157 / 0.368 | 0.237 / 0.417 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|
| `dbo:almamater` | `wdt:p69` | 374.9 | correct |
| `dbo:academicdiscipline` | `wdt:p101` | 270.5 | **wrong** |
| `dbo:birthdate` | `wdt:p569` | 195.2 | correct |
| `dbo:institution` | `wdt:p108` | 188.1 | **wrong** |
| `dbo:birthplace` | `wdt:p19` | 163.9 | correct |
| `dbo:award` | `wdt:p166` | 154.5 | correct |
| `dbo:deathdate` | `wdt:p570` | 136.0 | correct |
| `dbo:deathplace` | `wdt:p20` | 81.9 | correct |
| `dbo:nationality` | `wdt:p27` | 81.4 | correct |
| `dbo:doctoraladvisor` | `wdt:p184` | 70.4 | correct |
| `dbo:doctoralstudent` | `wdt:p185` | 64.6 | correct |
| `dbo:knownfor` | `wdt:p800` | 33.5 | **wrong** |
| `dbo:citizenship` | `wdt:p27` | 29.3 | correct |
| `dbo:child` | `wdt:p40` | 17.8 | correct |
| `dbo:spouse` | `wdt:p26` | 14.7 | correct |

Gold pairs present but not learned (1): party↔p102

Runtime 6.5s.
