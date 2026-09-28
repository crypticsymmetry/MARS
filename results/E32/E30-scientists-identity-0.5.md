# E30: online transfer reliability in the engine (scientists-identity-0.5)

Data: `data/kg-scientists → kg2mars --condition C --hop2` (wikidata side, 993 memory cases); 2400 hold-out queries (relations: wdt:p69, wdt:p108, wdt:p101, wdt:p27, wdt:p106, wdt:p166, wdt:p463, wdt:p1412; up to 300 per relation), streamed in a seeded random order (seed 1). Engine: surface profile, ½FAC + ½FP over the fingerprint top-50 (identity channel weight 0.5), first-order inferences from the top-10 analogues (the entity's own full case excluded). After each query the top-3 suggestions are checked and fed back (6757 feedback events in total).

| ranking | Hits@1 | Hits@10 | MRR |
|---|---|---|---|
| raw (Σ fused score of proposing analogues) | 0.420 | 0.635 | 0.499 |
| learned reliability × Σ fused (online feedback) | 0.424 | 0.637 | 0.503 |
| learned + rules induced so far (E31) | 0.422 | 0.638 | 0.502 |

Induced rules (≥ 20 outcomes, precision ≥ 0.5) fired on 401 queries; 0 queries were answered correctly at rank 1 only thanks to them (no analogue proposed a correct object). Feedback shown from the learned ranking.

**Where the answer is** (Hits@1 raw / learned / learned + rules):

| queries | n | raw | learned | learned + rules |
|---|---|---|---|---|
| a correct object is already an entity of the query (substitution / rule reachable) | 706 | 0.725 | 0.751 | 0.745 |
| no correct object in the query (only copying from analogues can reach it) | 1694 | 0.292 | 0.287 | 0.287 |

**Learning curve** (Hits@1 per stream segment):

| segment | queries | raw | learned | Δ | learned + rules |
|---|---|---|---|---|---|
| 1 | 1–300 | 0.440 | 0.450 | +0.010 | 0.450 |
| 2 | 301–600 | 0.437 | 0.443 | +0.007 | 0.440 |
| 3 | 601–900 | 0.427 | 0.423 | -0.003 | 0.420 |
| 4 | 901–1200 | 0.433 | 0.423 | -0.010 | 0.420 |
| 5 | 1201–1500 | 0.413 | 0.423 | +0.010 | 0.427 |
| 6 | 1501–1800 | 0.380 | 0.387 | +0.007 | 0.387 |
| 7 | 1801–2100 | 0.417 | 0.423 | +0.007 | 0.420 |
| 8 | 2101–2400 | 0.410 | 0.417 | +0.007 | 0.410 |

**Rules induced from analogy by the end of the stream** (transfer types with ≥ 20 feedback outcomes and precision ≥ 0.5; 10 in total, top 20):

| rule | precision | outcomes |
|---|---|---|
| `wdt:p1412(x, y) ⇐ wdt:p103(x, y)` | 1.000 | 29 |
| `wdt:p1412(x, y) ⇐ wdt:p6886(x, y)` | 1.000 | 28 |
| `wdt:p27(x, y) ⇐ wdt:p19(x, z) ∧ wdt:p17(z, y)` | 0.768 | 220 |
| `wdt:p27(x, y) ⇐ wdt:p108(x, z) ∧ wdt:p17(z, y)` | 0.713 | 286 |
| `wdt:p27(x, y) ⇐ wdt:p20(x, z) ∧ wdt:p17(z, y)` | 0.701 | 117 |
| `wdt:p27(x, y) ⇐ wdt:p69(x, z) ∧ wdt:p17(z, y)` | 0.675 | 335 |
| `wdt:p27(x, y) ⇐ wdt:p184(x, z) ∧ wdt:p27(z, y)` | 0.600 | 125 |
| `wdt:p69(x, y) ⇐ wdt:p184(x, z) ∧ wdt:p108(z, y)` | 0.570 | 165 |
| `wdt:p27(x, y) ⇐ wdt:p463(x, z) ∧ wdt:p17(z, y)` | 0.556 | 27 |
| `wdt:p27(x, y) ⇐ wdt:p551(x, y)` | 0.524 | 21 |

**Least reliable transfer types** (≥ 20 outcomes):

| type | precision | outcomes |
|---|---|---|
| `wdt:p108<=wdt:p19` | 0.000 | 42 |
| `wdt:p69<=wdt:p20` | 0.000 | 47 |
| `wdt:p108<=wdt:p20` | 0.000 | 52 |
| `wdt:p69<=wdt:p19` | 0.000 | 56 |
| `wdt:p69<=new:1` | 0.038 | 237 |

Runtime 78.7s.
