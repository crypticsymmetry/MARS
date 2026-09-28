# E30: online transfer reliability in the engine (scientists-identity-0.3)

Data: `data/kg-scientists → kg2mars --condition C --hop2` (wikidata side, 993 memory cases); 2400 hold-out queries (relations: wdt:p69, wdt:p108, wdt:p101, wdt:p27, wdt:p106, wdt:p166, wdt:p463, wdt:p1412; up to 300 per relation), streamed in a seeded random order (seed 1). Engine: surface profile, ½FAC + ½FP over the fingerprint top-50 (identity channel weight 0.3), first-order inferences from the top-10 analogues (the entity's own full case excluded). After each query the top-3 suggestions are checked and fed back (6790 feedback events in total).

| ranking | Hits@1 | Hits@10 | MRR |
|---|---|---|---|
| raw (Σ fused score of proposing analogues) | 0.413 | 0.632 | 0.494 |
| learned reliability × Σ fused (online feedback) | 0.420 | 0.633 | 0.498 |
| learned + rules induced so far (E31) | 0.418 | 0.635 | 0.497 |

Induced rules (≥ 20 outcomes, precision ≥ 0.5) fired on 402 queries; 1 queries were answered correctly at rank 1 only thanks to them (no analogue proposed a correct object). Feedback shown from the learned ranking.

**Where the answer is** (Hits@1 raw / learned / learned + rules):

| queries | n | raw | learned | learned + rules |
|---|---|---|---|---|
| a correct object is already an entity of the query (substitution / rule reachable) | 706 | 0.714 | 0.744 | 0.739 |
| no correct object in the query (only copying from analogues can reach it) | 1694 | 0.288 | 0.285 | 0.284 |

**Learning curve** (Hits@1 per stream segment):

| segment | queries | raw | learned | Δ | learned + rules |
|---|---|---|---|---|---|
| 1 | 1–300 | 0.433 | 0.447 | +0.013 | 0.447 |
| 2 | 301–600 | 0.430 | 0.437 | +0.007 | 0.427 |
| 3 | 601–900 | 0.420 | 0.413 | -0.007 | 0.413 |
| 4 | 901–1200 | 0.410 | 0.410 | +0.000 | 0.407 |
| 5 | 1201–1500 | 0.410 | 0.430 | +0.020 | 0.430 |
| 6 | 1501–1800 | 0.380 | 0.383 | +0.003 | 0.387 |
| 7 | 1801–2100 | 0.400 | 0.407 | +0.007 | 0.410 |
| 8 | 2101–2400 | 0.423 | 0.430 | +0.007 | 0.423 |

**Rules induced from analogy by the end of the stream** (transfer types with ≥ 20 feedback outcomes and precision ≥ 0.5; 10 in total, top 20):

| rule | precision | outcomes |
|---|---|---|
| `wdt:p1412(x, y) ⇐ wdt:p103(x, y)` | 1.000 | 29 |
| `wdt:p1412(x, y) ⇐ wdt:p6886(x, y)` | 1.000 | 28 |
| `wdt:p27(x, y) ⇐ wdt:p19(x, z) ∧ wdt:p17(z, y)` | 0.766 | 222 |
| `wdt:p27(x, y) ⇐ wdt:p108(x, z) ∧ wdt:p17(z, y)` | 0.716 | 285 |
| `wdt:p27(x, y) ⇐ wdt:p20(x, z) ∧ wdt:p17(z, y)` | 0.701 | 117 |
| `wdt:p27(x, y) ⇐ wdt:p69(x, z) ∧ wdt:p17(z, y)` | 0.677 | 334 |
| `wdt:p27(x, y) ⇐ wdt:p184(x, z) ∧ wdt:p27(z, y)` | 0.591 | 127 |
| `wdt:p69(x, y) ⇐ wdt:p184(x, z) ∧ wdt:p108(z, y)` | 0.571 | 163 |
| `wdt:p27(x, y) ⇐ wdt:p551(x, y)` | 0.550 | 20 |
| `wdt:p27(x, y) ⇐ wdt:p463(x, z) ∧ wdt:p17(z, y)` | 0.536 | 28 |

**Least reliable transfer types** (≥ 20 outcomes):

| type | precision | outcomes |
|---|---|---|
| `wdt:p69<=wdt:p20` | 0.000 | 49 |
| `wdt:p108<=wdt:p19` | 0.000 | 49 |
| `wdt:p69<=wdt:p19` | 0.000 | 54 |
| `wdt:p108<=wdt:p20` | 0.000 | 58 |
| `wdt:p69<=new:1` | 0.034 | 237 |

Runtime 78.4s.
