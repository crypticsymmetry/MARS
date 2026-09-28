# E30: online transfer reliability in the engine (films-0)

Data: `data/kg-films → kg2mars --condition C --hop2` (wikidata side, 992 memory cases); 3000 hold-out queries (relations: wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272; up to 300 per relation), streamed in a seeded random order (seed 1). Engine: surface profile, ½FAC + ½FP over the fingerprint top-50 (identity channel weight 0), first-order inferences from the top-10 analogues (the entity's own full case excluded). After each query the top-3 suggestions are checked and fed back (7897 feedback events in total).

| ranking | Hits@1 | Hits@10 | MRR |
|---|---|---|---|
| raw (Σ fused score of proposing analogues) | 0.360 | 0.486 | 0.408 |
| learned reliability × Σ fused (online feedback) | 0.373 | 0.488 | 0.418 |
| learned + rules induced so far (E31) | 0.379 | 0.490 | 0.421 |
| calibrated: noisy-or of support-conditioned reliabilities (E32) | 0.375 | 0.488 | 0.418 |

Induced rules (≥ 20 outcomes, precision ≥ 0.5) fired on 983 queries; 5 queries were answered correctly at rank 1 only thanks to them (no analogue proposed a correct object). Feedback shown from the learned ranking.

**Where the answer is** (Hits@1 raw / learned / learned + rules / calibrated):

| queries | n | raw | learned | learned + rules | calibrated |
|---|---|---|---|---|---|
| a correct object is already an entity of the query (substitution / rule reachable) | 1160 | 0.771 | 0.816 | 0.833 | 0.827 |
| no correct object in the query (only copying from analogues can reach it) | 1840 | 0.102 | 0.095 | 0.093 | 0.090 |

**Learning curve** (Hits@1 per stream segment):

| segment | queries | raw | learned | Δ | learned + rules |
|---|---|---|---|---|---|
| 1 | 1–375 | 0.376 | 0.395 | +0.019 | 0.397 |
| 2 | 376–750 | 0.355 | 0.363 | +0.008 | 0.365 |
| 3 | 751–1125 | 0.352 | 0.368 | +0.016 | 0.368 |
| 4 | 1126–1500 | 0.373 | 0.395 | +0.021 | 0.405 |
| 5 | 1501–1875 | 0.373 | 0.389 | +0.016 | 0.395 |
| 6 | 1876–2250 | 0.357 | 0.360 | +0.003 | 0.368 |
| 7 | 2251–2625 | 0.349 | 0.360 | +0.011 | 0.365 |
| 8 | 2626–3000 | 0.347 | 0.357 | +0.011 | 0.368 |

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
| `wdt:p364(x, y) ⇐ wdt:p57(x, z) ∧ wdt:p1412(z, y)` | 0.843 | 236 |
| `wdt:p364(x, y) ⇐ wdt:p162(x, z) ∧ wdt:p1412(z, y)` | 0.840 | 125 |
| `wdt:p364(x, y) ⇐ wdt:p161(x, z) ∧ wdt:p1412(z, y)` | 0.837 | 43 |
| `wdt:p364(x, y) ⇐ wdt:p86(x, z) ∧ wdt:p1412(z, y)` | 0.822 | 129 |
| `wdt:p364(x, y) ⇐ wdt:p58(x, z) ∧ wdt:p1412(z, y)` | 0.821 | 168 |
| `wdt:p364(x, y) ⇐ wdt:p840(x, z) ∧ wdt:p37(z, y)` | 0.714 | 21 |
| `wdt:p364(x, y) ⇐ wdt:p495(x, z) ∧ wdt:p37(z, y)` | 0.627 | 236 |
| `wdt:p495(x, y) ⇐ wdt:p840(x, y)` | 0.576 | 33 |
| `wdt:p58(x, y) ⇐ wdt:p57(x, y)` | 0.572 | 297 |
| `wdt:p272(x, y) ⇐ wdt:p495(x, z) ∧ wdt:p17(y, z)` | 0.517 | 172 |

**Least reliable transfer types** (≥ 20 outcomes):

| type | precision | outcomes |
|---|---|---|
| `wdt:p344<=wdt:p57` | 0.000 | 22 |
| `wdt:p344<=wdt:p364.wdt:p1412~` | 0.000 | 22 |
| `wdt:p344<=wdt:p495.wdt:p27~` | 0.000 | 35 |
| `wdt:p86<=wdt:p162` | 0.000 | 119 |
| `wdt:p86<=wdt:p58` | 0.008 | 127 |

Runtime 45.2s.
