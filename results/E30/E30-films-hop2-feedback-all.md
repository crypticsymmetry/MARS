# E30: online transfer reliability in the engine (films-hop2-feedback-all)

Data: `data/kg-films → kg2mars --condition C --hop2` (wikidata side, 992 memory cases); 3000 hold-out queries (relations: wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272; up to 300 per relation), streamed in a seeded random order (seed 1). Engine: surface profile, ½FAC + ½FP over the fingerprint top-50, first-order inferences from the top-10 analogues (the entity's own full case excluded). After each query the top-1000 suggestions are checked and fed back (21579 feedback events in total).

| ranking | Hits@1 | Hits@10 | MRR |
|---|---|---|---|
| raw (Σ fused score of proposing analogues) | 0.360 | 0.486 | 0.408 |
| learned reliability × Σ fused (online feedback) | 0.372 | 0.488 | 0.417 |

**Learning curve** (Hits@1 per stream segment):

| segment | queries | raw | learned | Δ |
|---|---|---|---|---|
| 1 | 1–375 | 0.376 | 0.389 | +0.013 |
| 2 | 376–750 | 0.355 | 0.363 | +0.008 |
| 3 | 751–1125 | 0.352 | 0.363 | +0.011 |
| 4 | 1126–1500 | 0.373 | 0.387 | +0.013 |
| 5 | 1501–1875 | 0.373 | 0.389 | +0.016 |
| 6 | 1876–2250 | 0.357 | 0.363 | +0.005 |
| 7 | 2251–2625 | 0.349 | 0.360 | +0.011 |
| 8 | 2626–3000 | 0.347 | 0.363 | +0.016 |

**Rules induced from analogy by the end of the stream** (transfer types with ≥ 20 feedback outcomes and precision ≥ 0.5; 20 in total, top 20):

| rule | precision | outcomes |
|---|---|---|
| `wdt:p495(x, y) ⇐ wdt:p364(x, z) ∧ wdt:p37(y, z)` | 1.000 | 145 |
| `wdt:p495(x, y) ⇐ wdt:p272(x, z) ∧ wdt:p17(z, y)` | 0.958 | 71 |
| `wdt:p495(x, y) ⇐ wdt:p161(x, z) ∧ wdt:p27(z, y)` | 0.930 | 43 |
| `wdt:p495(x, y) ⇐ wdt:p57(x, z) ∧ wdt:p27(z, y)` | 0.926 | 284 |
| `wdt:p495(x, y) ⇐ wdt:p162(x, z) ∧ wdt:p27(z, y)` | 0.925 | 147 |
| `wdt:p495(x, y) ⇐ wdt:p58(x, z) ∧ wdt:p27(z, y)` | 0.917 | 206 |
| `wdt:p495(x, y) ⇐ wdt:p750(x, z) ∧ wdt:p495(z, y)` | 0.913 | 46 |
| `wdt:p86(x, y) ⇐ wdt:p175(x, y)` | 0.909 | 22 |
| `wdt:p495(x, y) ⇐ wdt:p86(x, z) ∧ wdt:p27(z, y)` | 0.901 | 152 |
| `wdt:p495(x, y) ⇐ wdt:p750(x, z) ∧ wdt:p17(z, y)` | 0.892 | 83 |
| `wdt:p364(x, y) ⇐ wdt:p57(x, z) ∧ wdt:p1412(z, y)` | 0.846 | 240 |
| `wdt:p364(x, y) ⇐ wdt:p161(x, z) ∧ wdt:p1412(z, y)` | 0.841 | 44 |
| `wdt:p364(x, y) ⇐ wdt:p58(x, z) ∧ wdt:p1412(z, y)` | 0.825 | 171 |
| `wdt:p364(x, y) ⇐ wdt:p162(x, z) ∧ wdt:p1412(z, y)` | 0.820 | 128 |
| `wdt:p364(x, y) ⇐ wdt:p86(x, z) ∧ wdt:p1412(z, y)` | 0.818 | 132 |
| `wdt:p364(x, y) ⇐ wdt:p840(x, z) ∧ wdt:p37(z, y)` | 0.696 | 23 |
| `wdt:p364(x, y) ⇐ wdt:p495(x, z) ∧ wdt:p37(z, y)` | 0.623 | 247 |
| `wdt:p495(x, y) ⇐ wdt:p840(x, y)` | 0.576 | 33 |
| `wdt:p58(x, y) ⇐ wdt:p57(x, y)` | 0.570 | 298 |
| `wdt:p272(x, y) ⇐ wdt:p495(x, z) ∧ wdt:p17(y, z)` | 0.511 | 174 |

**Least reliable transfer types** (≥ 20 outcomes):

| type | precision | outcomes |
|---|---|---|
| `wdt:p344<=wdt:p58` | 0.000 | 20 |
| `wdt:p344<=wdt:p364.wdt:p1412~` | 0.000 | 22 |
| `wdt:p344<=wdt:p57` | 0.000 | 25 |
| `wdt:p86<=wdt:p161` | 0.000 | 32 |
| `wdt:p344<=wdt:p495.wdt:p27~` | 0.000 | 38 |

Runtime 44.8s.
