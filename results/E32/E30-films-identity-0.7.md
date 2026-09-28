# E30: online transfer reliability in the engine (films-identity-0.7)

Data: `data/kg-films → kg2mars --condition C --hop2` (wikidata side, 992 memory cases); 3000 hold-out queries (relations: wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272; up to 300 per relation), streamed in a seeded random order (seed 1). Engine: surface profile, ½FAC + ½FP over the fingerprint top-50 (identity channel weight 0.7), first-order inferences from the top-10 analogues (the entity's own full case excluded). After each query the top-3 suggestions are checked and fed back (7441 feedback events in total).

| ranking | Hits@1 | Hits@10 | MRR |
|---|---|---|---|
| raw (Σ fused score of proposing analogues) | 0.366 | 0.490 | 0.412 |
| learned reliability × Σ fused (online feedback) | 0.375 | 0.492 | 0.419 |
| learned + rules induced so far (E31) | 0.381 | 0.499 | 0.424 |

Induced rules (≥ 20 outcomes, precision ≥ 0.5) fired on 1044 queries; 17 queries were answered correctly at rank 1 only thanks to them (no analogue proposed a correct object). Feedback shown from the learned ranking.

**Where the answer is** (Hits@1 raw / learned / learned + rules):

| queries | n | raw | learned | learned + rules |
|---|---|---|---|---|
| a correct object is already an entity of the query (substitution / rule reachable) | 1160 | 0.776 | 0.820 | 0.838 |
| no correct object in the query (only copying from analogues can reach it) | 1840 | 0.108 | 0.095 | 0.093 |

**Learning curve** (Hits@1 per stream segment):

| segment | queries | raw | learned | Δ | learned + rules |
|---|---|---|---|---|---|
| 1 | 1–375 | 0.397 | 0.397 | +0.000 | 0.395 |
| 2 | 376–750 | 0.347 | 0.365 | +0.019 | 0.371 |
| 3 | 751–1125 | 0.352 | 0.357 | +0.005 | 0.357 |
| 4 | 1126–1500 | 0.395 | 0.397 | +0.003 | 0.400 |
| 5 | 1501–1875 | 0.379 | 0.395 | +0.016 | 0.408 |
| 6 | 1876–2250 | 0.331 | 0.357 | +0.027 | 0.368 |
| 7 | 2251–2625 | 0.371 | 0.368 | -0.003 | 0.376 |
| 8 | 2626–3000 | 0.360 | 0.365 | +0.005 | 0.376 |

**Rules induced from analogy by the end of the stream** (transfer types with ≥ 20 feedback outcomes and precision ≥ 0.5; 20 in total, top 20):

| rule | precision | outcomes |
|---|---|---|
| `wdt:p495(x, y) ⇐ wdt:p364(x, z) ∧ wdt:p37(y, z)` | 1.000 | 144 |
| `wdt:p495(x, y) ⇐ wdt:p272(x, z) ∧ wdt:p17(z, y)` | 0.971 | 70 |
| `wdt:p495(x, y) ⇐ wdt:p162(x, z) ∧ wdt:p27(z, y)` | 0.937 | 143 |
| `wdt:p495(x, y) ⇐ wdt:p58(x, z) ∧ wdt:p27(z, y)` | 0.935 | 201 |
| `wdt:p495(x, y) ⇐ wdt:p57(x, z) ∧ wdt:p27(z, y)` | 0.932 | 281 |
| `wdt:p495(x, y) ⇐ wdt:p161(x, z) ∧ wdt:p27(z, y)` | 0.929 | 42 |
| `wdt:p495(x, y) ⇐ wdt:p86(x, z) ∧ wdt:p27(z, y)` | 0.926 | 148 |
| `wdt:p495(x, y) ⇐ wdt:p750(x, z) ∧ wdt:p17(z, y)` | 0.914 | 81 |
| `wdt:p86(x, y) ⇐ wdt:p175(x, y)` | 0.905 | 21 |
| `wdt:p495(x, y) ⇐ wdt:p750(x, z) ∧ wdt:p495(z, y)` | 0.894 | 47 |
| `wdt:p364(x, y) ⇐ wdt:p57(x, z) ∧ wdt:p1412(z, y)` | 0.874 | 230 |
| `wdt:p364(x, y) ⇐ wdt:p162(x, z) ∧ wdt:p1412(z, y)` | 0.867 | 120 |
| `wdt:p364(x, y) ⇐ wdt:p58(x, z) ∧ wdt:p1412(z, y)` | 0.864 | 162 |
| `wdt:p364(x, y) ⇐ wdt:p86(x, z) ∧ wdt:p1412(z, y)` | 0.862 | 123 |
| `wdt:p364(x, y) ⇐ wdt:p161(x, z) ∧ wdt:p1412(z, y)` | 0.822 | 45 |
| `wdt:p364(x, y) ⇐ wdt:p840(x, z) ∧ wdt:p37(z, y)` | 0.727 | 22 |
| `wdt:p364(x, y) ⇐ wdt:p495(x, z) ∧ wdt:p37(z, y)` | 0.645 | 234 |
| `wdt:p495(x, y) ⇐ wdt:p840(x, y)` | 0.625 | 32 |
| `wdt:p58(x, y) ⇐ wdt:p57(x, y)` | 0.579 | 273 |
| `wdt:p272(x, y) ⇐ wdt:p495(x, z) ∧ wdt:p17(y, z)` | 0.525 | 162 |

**Least reliable transfer types** (≥ 20 outcomes):

| type | precision | outcomes |
|---|---|---|
| `wdt:p344<=wdt:p57` | 0.000 | 20 |
| `wdt:p344<=wdt:p364.wdt:p1412~` | 0.000 | 21 |
| `wdt:p344<=wdt:p495.wdt:p27~` | 0.000 | 33 |
| `wdt:p86<=wdt:p58` | 0.006 | 160 |
| `wdt:p86<=wdt:p162` | 0.007 | 143 |

Runtime 70.2s.
