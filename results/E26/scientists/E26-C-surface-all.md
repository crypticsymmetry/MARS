# E26: knowledge-graph vocabulary alignment (C-surface-all)

Data: `data/kg-scientists → kg2mars --condition C` — 1993 cases, 126 properties (21 DBpedia, 105 Wikidata); 13 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `surface-only` profile), wildcard mapping (local score 0.3, attributes included), evidence (`all` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster, re-estimated each round. Counterpart retrieval: each DBpedia entity ranks the Wikidata entities (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.540 / 0.790 | 0.597 / 0.802 |
| 1 | 5 | 4 / 1 / 0 | 0.800 | 0.308 | 0.540 / 0.646 | 0.597 / 0.691 |
| 2 | 10 | 8 / 2 / 0 | 0.800 | 0.615 | 0.540 / 0.572 | 0.597 / 0.630 |
| 3 | 11 | 8 / 3 / 0 | 0.727 | 0.615 | 0.540 / 0.524 | 0.597 / 0.587 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|
| `dbo:almamater` | `wdt:p106` | 6757.5 | **wrong** |
| `dbo:birthdate` | `wdt:p569` | 3323.2 | correct |
| `dbo:academicdiscipline` | `wdt:p101` | 2856.5 | **wrong** |
| `dbo:award` | `wdt:p166` | 1785.4 | correct |
| `dbo:deathdate` | `wdt:p570` | 1383.8 | correct |
| `dbo:birthplace` | `wdt:p108` | 1236.4 | **wrong** |
| `dbo:deathplace` | `wdt:p20` | 1128.6 | correct |
| `dbo:doctoraladvisor` | `wdt:p184` | 716.0 | correct |
| `dbo:doctoralstudent` | `wdt:p185` | 569.6 | correct |
| `dbo:spouse` | `wdt:p26` | 46.5 | correct |
| `dbo:child` | `wdt:p40` | 39.1 | correct |

Gold pairs present but not learned (5): birthplace↔p19, party↔p102, nationality↔p27, citizenship↔p27, almamater↔p69

Runtime 6.9s.
