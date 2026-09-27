# E26: knowledge-graph vocabulary alignment (B-surface-anchored)

Data: `data/kg-scientists → kg2mars --condition B` — 1993 cases, 126 properties (21 DBpedia, 105 Wikidata); 13 gold property pairs occur in the data. Loop: 5 other-KG neighbours per case (fingerprint, `surface-only` profile), wildcard mapping (local score 0.3, attributes included), evidence (`anchored` correspondences) from mappings with normalized score ≥ 0.2, mutual-best pairs (evidence ≥ 1), one property per KG per cluster, re-estimated each round. Counterpart retrieval: each DBpedia entity ranks the Wikidata entities (FP = fingerprint, fused = ½FAC + ½FP over the FP top-50).

| round | aligned pairs | correct / wrong / unjudged | precision (judged) | gold recall | counterpart R@1 FP / fused | MRR FP / fused |
|---|---|---|---|---|---|---|
| 0 | 0 | 0 / 0 / 0 | NaN | 0.000 | 0.327 / 0.449 | 0.364 / 0.466 |
| 1 | 2 | 2 / 0 / 0 | 1.000 | 0.154 | 0.327 / 0.350 | 0.364 / 0.390 |
| 2 | 2 | 2 / 0 / 0 | 1.000 | 0.154 | 0.327 / 0.350 | 0.364 / 0.390 |
| 3 | 2 | 2 / 0 / 0 | 1.000 | 0.154 | 0.327 / 0.350 | 0.364 / 0.390 |

## Final alignment (top 40 by evidence)

| DBpedia | Wikidata | evidence | verdict |
|---|---|---|---|
| `dbo:birthdate` | `wdt:p569` | 472.8 | correct |
| `dbo:deathdate` | `wdt:p570` | 323.8 | correct |

Gold pairs present but not learned (11): birthplace↔p19, party↔p102, nationality↔p27, spouse↔p26, citizenship↔p27, almamater↔p69, doctoralstudent↔p185, deathplace↔p20, award↔p166, doctoraladvisor↔p184, child↔p40

Runtime 5.5s.
