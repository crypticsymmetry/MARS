# E30: online transfer reliability in the engine (scientists-identity-0.85)

Data: `data/kg-scientists → kg2mars --condition C --hop2` (wikidata side, 993 memory cases); 2400 hold-out queries (relations: wdt:p69, wdt:p108, wdt:p101, wdt:p27, wdt:p106, wdt:p166, wdt:p463, wdt:p1412; up to 300 per relation), streamed in a seeded random order (seed 1). Engine: surface profile, ½FAC + ½FP over the fingerprint top-50 (identity channel weight 0.85), first-order inferences from the top-10 analogues (the entity's own full case excluded). After each query the top-3 suggestions are checked and fed back (6707 feedback events in total).

| ranking | Hits@1 | Hits@10 | MRR |
|---|---|---|---|
| raw (Σ fused score of proposing analogues) | 0.439 | 0.645 | 0.514 |
| learned reliability × Σ fused (online feedback) | 0.445 | 0.645 | 0.518 |
| learned + rules induced so far (E31) | 0.442 | 0.647 | 0.517 |

Induced rules (≥ 20 outcomes, precision ≥ 0.5) fired on 404 queries; 0 queries were answered correctly at rank 1 only thanks to them (no analogue proposed a correct object). Feedback shown from the learned ranking.

**Where the answer is** (Hits@1 raw / learned / learned + rules):

| queries | n | raw | learned | learned + rules |
|---|---|---|---|---|
| a correct object is already an entity of the query (substitution / rule reachable) | 706 | 0.727 | 0.762 | 0.758 |
| no correct object in the query (only copying from analogues can reach it) | 1694 | 0.319 | 0.312 | 0.311 |

**Learning curve** (Hits@1 per stream segment):

| segment | queries | raw | learned | Δ | learned + rules |
|---|---|---|---|---|---|
| 1 | 1–300 | 0.450 | 0.463 | +0.013 | 0.463 |
| 2 | 301–600 | 0.447 | 0.440 | -0.007 | 0.437 |
| 3 | 601–900 | 0.427 | 0.430 | +0.003 | 0.427 |
| 4 | 901–1200 | 0.463 | 0.457 | -0.007 | 0.457 |
| 5 | 1201–1500 | 0.433 | 0.450 | +0.017 | 0.447 |
| 6 | 1501–1800 | 0.417 | 0.413 | -0.003 | 0.407 |
| 7 | 1801–2100 | 0.447 | 0.457 | +0.010 | 0.453 |
| 8 | 2101–2400 | 0.427 | 0.447 | +0.020 | 0.447 |

**Rules induced from analogy by the end of the stream** (transfer types with ≥ 20 feedback outcomes and precision ≥ 0.5; 11 in total, top 20):

| rule | precision | outcomes |
|---|---|---|
| `wdt:p1412(x, y) ⇐ wdt:p6886(x, y)` | 1.000 | 27 |
| `wdt:p1412(x, y) ⇐ wdt:p103(x, y)` | 0.968 | 31 |
| `wdt:p27(x, y) ⇐ wdt:p19(x, z) ∧ wdt:p17(z, y)` | 0.810 | 211 |
| `wdt:p27(x, y) ⇐ wdt:p20(x, z) ∧ wdt:p17(z, y)` | 0.731 | 108 |
| `wdt:p27(x, y) ⇐ wdt:p108(x, z) ∧ wdt:p17(z, y)` | 0.719 | 278 |
| `wdt:p27(x, y) ⇐ wdt:p69(x, z) ∧ wdt:p17(z, y)` | 0.685 | 336 |
| `wdt:p69(x, y) ⇐ wdt:p184(x, z) ∧ wdt:p108(z, y)` | 0.615 | 156 |
| `wdt:p27(x, y) ⇐ wdt:p463(x, z) ∧ wdt:p17(z, y)` | 0.609 | 23 |
| `wdt:p27(x, y) ⇐ wdt:p184(x, z) ∧ wdt:p27(z, y)` | 0.603 | 126 |
| `wdt:p69(x, y) ⇐ wdt:p184(x, z) ∧ wdt:p69(z, y)` | 0.525 | 118 |
| `wdt:p27(x, y) ⇐ wdt:p551(x, y)` | 0.524 | 21 |

**Least reliable transfer types** (≥ 20 outcomes):

| type | precision | outcomes |
|---|---|---|
| `wdt:p108<=wdt:p19` | 0.000 | 45 |
| `wdt:p69<=wdt:p20` | 0.000 | 46 |
| `wdt:p108<=wdt:p20` | 0.000 | 55 |
| `wdt:p69<=wdt:p19` | 0.000 | 59 |
| `wdt:p101<=new:1` | 0.041 | 656 |

Runtime 46.6s.
