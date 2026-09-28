# E30: online transfer reliability in the engine (films-identity-1)

Data: `data/kg-films → kg2mars --condition C --hop2` (wikidata side, 992 memory cases); 3000 hold-out queries (relations: wdt:p57, wdt:p161, wdt:p58, wdt:p162, wdt:p86, wdt:p344, wdt:p136, wdt:p495, wdt:p364, wdt:p272; up to 300 per relation), streamed in a seeded random order (seed 1). Engine: surface profile, ½FAC + ½FP over the fingerprint top-50 (identity channel weight 1), first-order inferences from the top-10 analogues (the entity's own full case excluded). After each query the top-3 suggestions are checked and fed back (6507 feedback events in total).

| ranking | Hits@1 | Hits@10 | MRR |
|---|---|---|---|
| raw (Σ fused score of proposing analogues) | 0.329 | 0.445 | 0.373 |
| learned reliability × Σ fused (online feedback) | 0.341 | 0.445 | 0.382 |
| learned + rules induced so far (E31) | 0.361 | 0.473 | 0.404 |

Induced rules (≥ 20 outcomes, precision ≥ 0.5) fired on 1063 queries; 50 queries were answered correctly at rank 1 only thanks to them (no analogue proposed a correct object). Feedback shown from the learned ranking.

**Where the answer is** (Hits@1 raw / learned / learned + rules):

| queries | n | raw | learned | learned + rules |
|---|---|---|---|---|
| a correct object is already an entity of the query (substitution / rule reachable) | 1160 | 0.680 | 0.728 | 0.789 |
| no correct object in the query (only copying from analogues can reach it) | 1840 | 0.108 | 0.097 | 0.091 |

**Learning curve** (Hits@1 per stream segment):

| segment | queries | raw | learned | Δ | learned + rules |
|---|---|---|---|---|---|
| 1 | 1–375 | 0.349 | 0.373 | +0.024 | 0.376 |
| 2 | 376–750 | 0.339 | 0.333 | -0.005 | 0.363 |
| 3 | 751–1125 | 0.317 | 0.333 | +0.016 | 0.341 |
| 4 | 1126–1500 | 0.357 | 0.357 | +0.000 | 0.371 |
| 5 | 1501–1875 | 0.315 | 0.357 | +0.043 | 0.387 |
| 6 | 1876–2250 | 0.309 | 0.333 | +0.024 | 0.357 |
| 7 | 2251–2625 | 0.312 | 0.317 | +0.005 | 0.333 |
| 8 | 2626–3000 | 0.333 | 0.325 | -0.008 | 0.357 |

**Rules induced from analogy by the end of the stream** (transfer types with ≥ 20 feedback outcomes and precision ≥ 0.5; 18 in total, top 20):

| rule | precision | outcomes |
|---|---|---|
| `wdt:p495(x, y) ⇐ wdt:p364(x, z) ∧ wdt:p37(y, z)` | 1.000 | 145 |
| `wdt:p495(x, y) ⇐ wdt:p272(x, z) ∧ wdt:p17(z, y)` | 0.971 | 69 |
| `wdt:p495(x, y) ⇐ wdt:p161(x, z) ∧ wdt:p27(z, y)` | 0.951 | 41 |
| `wdt:p495(x, y) ⇐ wdt:p58(x, z) ∧ wdt:p27(z, y)` | 0.940 | 199 |
| `wdt:p495(x, y) ⇐ wdt:p57(x, z) ∧ wdt:p27(z, y)` | 0.938 | 275 |
| `wdt:p495(x, y) ⇐ wdt:p162(x, z) ∧ wdt:p27(z, y)` | 0.936 | 140 |
| `wdt:p364(x, y) ⇐ wdt:p58(x, z) ∧ wdt:p1412(z, y)` | 0.914 | 151 |
| `wdt:p495(x, y) ⇐ wdt:p86(x, z) ∧ wdt:p27(z, y)` | 0.912 | 148 |
| `wdt:p495(x, y) ⇐ wdt:p750(x, z) ∧ wdt:p495(z, y)` | 0.905 | 42 |
| `wdt:p364(x, y) ⇐ wdt:p57(x, z) ∧ wdt:p1412(z, y)` | 0.900 | 220 |
| `wdt:p495(x, y) ⇐ wdt:p750(x, z) ∧ wdt:p17(z, y)` | 0.897 | 78 |
| `wdt:p364(x, y) ⇐ wdt:p86(x, z) ∧ wdt:p1412(z, y)` | 0.869 | 122 |
| `wdt:p364(x, y) ⇐ wdt:p162(x, z) ∧ wdt:p1412(z, y)` | 0.863 | 117 |
| `wdt:p364(x, y) ⇐ wdt:p161(x, z) ∧ wdt:p1412(z, y)` | 0.800 | 45 |
| `wdt:p364(x, y) ⇐ wdt:p495(x, z) ∧ wdt:p37(z, y)` | 0.673 | 223 |
| `wdt:p58(x, y) ⇐ wdt:p57(x, y)` | 0.639 | 191 |
| `wdt:p272(x, y) ⇐ wdt:p495(x, z) ∧ wdt:p17(y, z)` | 0.621 | 95 |
| `wdt:p495(x, y) ⇐ wdt:p840(x, y)` | 0.606 | 33 |

**Least reliable transfer types** (≥ 20 outcomes):

| type | precision | outcomes |
|---|---|---|
| `wdt:p161<=wdt:p86` | 0.000 | 28 |
| `wdt:p162<=wdt:p86` | 0.000 | 80 |
| `wdt:p57<=wdt:p86` | 0.000 | 90 |
| `wdt:p86<=wdt:p58` | 0.009 | 112 |
| `wdt:p86<=wdt:p162` | 0.009 | 107 |

Runtime 59.8s.
