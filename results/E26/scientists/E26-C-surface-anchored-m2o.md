# E26: knowledge-graph vocabulary alignment (C-surface-anchored-m2o)

Data: `data/kg-scientists → kg2mars --condition C` — 1993 cases, 126 properties (21 DBpedia, 105 Wikidata); 13 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `surface-only` profile), wildcard mapping (local score 0.3, attributes included), evidence (`anchored` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster plus many-to-one joins (evidence ≥ 0.25 × the pair's), re-estimated each round. Counterpart retrieval: each DBpedia entity ranks the Wikidata entities (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.540 / 0.790 | 0.597 / 0.802 |
| 1 | 14 | 10 / 4 / 0 | 0.714 | 0.769 | 0.540 / 0.604 | 0.597 / 0.653 |
| 2 | 14 | 10 / 4 / 0 | 0.714 | 0.769 | 0.540 / 0.604 | 0.597 / 0.653 |
| 3 | 14 | 10 / 4 / 0 | 0.714 | 0.769 | 0.540 / 0.604 | 0.597 / 0.653 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|
| `dbo:almamater` | `wdt:p69` | 1113.3 | correct |
| `dbo:birthdate` | `wdt:p569` | 639.9 | correct |
| `dbo:academicdiscipline` | `wdt:p101` | 545.3 | **wrong** |
| `dbo:birthplace` | `wdt:p19` | 543.8 | correct |
| `dbo:institution` | `wdt:p108` | 516.4 | **wrong** |
| `dbo:deathdate` | `wdt:p570` | 409.1 | correct |
| `dbo:award` | `wdt:p166` | 356.4 | correct |
| `dbo:deathplace` | `wdt:p20` | 276.4 | correct |
| `dbo:birthplace` | `wdt:p27` | 227.1 | **wrong** |
| `dbo:doctoraladvisor` | `wdt:p184` | 191.7 | correct |
| `dbo:doctoralstudent` | `wdt:p185` | 129.4 | correct |
| `dbo:knownfor` | `wdt:p800` | 70.8 | **wrong** |
| `dbo:spouse` | `wdt:p26` | 28.4 | correct |
| `dbo:child` | `wdt:p40` | 26.6 | correct |

Gold pairs present but not learned (3): party↔p102, nationality↔p27, citizenship↔p27

Runtime 6.1s.
