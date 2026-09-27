# E26: knowledge-graph vocabulary alignment (C-surface-anchored)

Data: `data/kg-scientists → kg2mars --condition C` — 1993 cases, 126 properties (21 DBpedia, 105 Wikidata); 13 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `surface-only` profile), wildcard mapping (local score 0.3, attributes included), evidence (`anchored` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster, re-estimated each round. Counterpart retrieval: each DBpedia entity ranks the Wikidata entities (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.540 / 0.790 | 0.597 / 0.802 |
| 1 | 13 | 10 / 3 / 0 | 0.769 | 0.769 | 0.540 / 0.609 | 0.597 / 0.659 |
| 2 | 13 | 10 / 3 / 0 | 0.769 | 0.769 | 0.540 / 0.609 | 0.597 / 0.659 |
| 3 | 13 | 10 / 3 / 0 | 0.769 | 0.769 | 0.540 / 0.609 | 0.597 / 0.659 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|
| `dbo:almamater` | `wdt:p69` | 1100.1 | correct |
| `dbo:birthdate` | `wdt:p569` | 631.0 | correct |
| `dbo:academicdiscipline` | `wdt:p101` | 538.7 | **wrong** |
| `dbo:birthplace` | `wdt:p19` | 534.1 | correct |
| `dbo:institution` | `wdt:p108` | 511.8 | **wrong** |
| `dbo:deathdate` | `wdt:p570` | 403.7 | correct |
| `dbo:award` | `wdt:p166` | 353.7 | correct |
| `dbo:deathplace` | `wdt:p20` | 271.9 | correct |
| `dbo:doctoraladvisor` | `wdt:p184` | 189.7 | correct |
| `dbo:doctoralstudent` | `wdt:p185` | 128.6 | correct |
| `dbo:knownfor` | `wdt:p800` | 70.2 | **wrong** |
| `dbo:spouse` | `wdt:p26` | 28.2 | correct |
| `dbo:child` | `wdt:p40` | 26.3 | correct |

Gold pairs present but not learned (3): party↔p102, nationality↔p27, citizenship↔p27

Runtime 6.7s.
