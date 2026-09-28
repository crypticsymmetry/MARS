# E30: online transfer reliability in the engine (films-identity-0.85)

Data: `data/kg-films → kg2mars --condition C --hop2` (wikidata side, 992 memory cases); 3000 hold-out queries (relations: wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272; up to 300 per relation), streamed in a seeded random order (seed 1). Engine: surface profile, ½FAC + ½FP over the fingerprint top-50 (identity channel weight 0.85), first-order inferences from the top-10 analogues (the entity's own full case excluded). After each query the top-3 suggestions are checked and fed back (7198 feedback events in total).

| ranking | Hits@1 | Hits@10 | MRR |
|---|---|---|---|
| raw (Σ fused score of proposing analogues) | 0.361 | 0.486 | 0.408 |
| learned reliability × Σ fused (online feedback) | 0.372 | 0.487 | 0.415 |
| learned + rules induced so far (E31) | 0.378 | 0.497 | 0.421 |

Induced rules (≥ 20 outcomes, precision ≥ 0.5) fired on 1034 queries; 20 queries were answered correctly at rank 1 only thanks to them (no analogue proposed a correct object). Feedback shown from the learned ranking.

**Where the answer is** (Hits@1 raw / learned / learned + rules):

| queries | n | raw | learned | learned + rules |
|---|---|---|---|---|
| a correct object is already an entity of the query (substitution / rule reachable) | 1160 | 0.757 | 0.809 | 0.828 |
| no correct object in the query (only copying from analogues can reach it) | 1840 | 0.112 | 0.097 | 0.093 |

**Learning curve** (Hits@1 per stream segment):

| segment | queries | raw | learned | Δ | learned + rules |
|---|---|---|---|---|---|
| 1 | 1–375 | 0.387 | 0.400 | +0.013 | 0.397 |
| 2 | 376–750 | 0.349 | 0.357 | +0.008 | 0.365 |
| 3 | 751–1125 | 0.344 | 0.349 | +0.005 | 0.355 |
| 4 | 1126–1500 | 0.395 | 0.392 | -0.003 | 0.392 |
| 5 | 1501–1875 | 0.365 | 0.379 | +0.013 | 0.389 |
| 6 | 1876–2250 | 0.333 | 0.357 | +0.024 | 0.368 |
| 7 | 2251–2625 | 0.360 | 0.379 | +0.019 | 0.379 |
| 8 | 2626–3000 | 0.357 | 0.363 | +0.005 | 0.376 |

**Rules induced from analogy by the end of the stream** (transfer types with ≥ 20 feedback outcomes and precision ≥ 0.5; 20 in total, top 20):

| rule | precision | outcomes |
|---|---|---|
| `wdt:p495(x, y) ⇐ wdt:p364(x, z) ∧ wdt:p37(y, z)` | 1.000 | 145 |
| `wdt:p495(x, y) ⇐ wdt:p272(x, z) ∧ wdt:p17(z, y)` | 0.971 | 70 |
| `wdt:p495(x, y) ⇐ wdt:p162(x, z) ∧ wdt:p27(z, y)` | 0.957 | 141 |
| `wdt:p495(x, y) ⇐ wdt:p58(x, z) ∧ wdt:p27(z, y)` | 0.950 | 199 |
| `wdt:p495(x, y) ⇐ wdt:p57(x, z) ∧ wdt:p27(z, y)` | 0.936 | 281 |
| `wdt:p495(x, y) ⇐ wdt:p161(x, z) ∧ wdt:p27(z, y)` | 0.930 | 43 |
| `wdt:p495(x, y) ⇐ wdt:p86(x, z) ∧ wdt:p27(z, y)` | 0.926 | 148 |
| `wdt:p495(x, y) ⇐ wdt:p750(x, z) ∧ wdt:p17(z, y)` | 0.914 | 81 |
| `wdt:p495(x, y) ⇐ wdt:p750(x, z) ∧ wdt:p495(z, y)` | 0.913 | 46 |
| `wdt:p86(x, y) ⇐ wdt:p175(x, y)` | 0.900 | 20 |
| `wdt:p364(x, y) ⇐ wdt:p57(x, z) ∧ wdt:p1412(z, y)` | 0.881 | 226 |
| `wdt:p364(x, y) ⇐ wdt:p58(x, z) ∧ wdt:p1412(z, y)` | 0.880 | 158 |
| `wdt:p364(x, y) ⇐ wdt:p161(x, z) ∧ wdt:p1412(z, y)` | 0.860 | 43 |
| `wdt:p364(x, y) ⇐ wdt:p86(x, z) ∧ wdt:p1412(z, y)` | 0.848 | 125 |
| `wdt:p364(x, y) ⇐ wdt:p162(x, z) ∧ wdt:p1412(z, y)` | 0.837 | 123 |
| `wdt:p364(x, y) ⇐ wdt:p840(x, z) ∧ wdt:p37(z, y)` | 0.800 | 20 |
| `wdt:p364(x, y) ⇐ wdt:p495(x, z) ∧ wdt:p37(z, y)` | 0.676 | 222 |
| `wdt:p495(x, y) ⇐ wdt:p840(x, y)` | 0.625 | 32 |
| `wdt:p58(x, y) ⇐ wdt:p57(x, y)` | 0.585 | 253 |
| `wdt:p272(x, y) ⇐ wdt:p495(x, z) ∧ wdt:p17(y, z)` | 0.546 | 152 |

**Least reliable transfer types** (≥ 20 outcomes):

| type | precision | outcomes |
|---|---|---|
| `wdt:p344<=wdt:p57` | 0.000 | 23 |
| `wdt:p344<=wdt:p364.wdt:p1412~` | 0.000 | 23 |
| `wdt:p344<=wdt:p495.wdt:p27~` | 0.000 | 35 |
| `wdt:p57<=wdt:p86` | 0.000 | 78 |
| `wdt:p86<=wdt:p58` | 0.007 | 148 |

Runtime 65.7s.
