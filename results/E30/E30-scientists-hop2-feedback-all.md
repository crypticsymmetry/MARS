# E30: online transfer reliability in the engine (scientists-hop2-feedback-all)

Data: `data/kg-scientists → kg2mars --condition C --hop2` (wikidata side, 993 memory cases); 2400 hold-out queries (relations: wdt:p69, wdt:p108, wdt:p101, wdt:p27, wdt:p106, wdt:p166, wdt:p463, wdt:p1412; up to 300 per relation), streamed in a seeded random order (seed 1). Engine: surface profile, ½FAC + ½FP over the fingerprint top-50, first-order inferences from the top-10 analogues (the entity's own full case excluded). After each query the top-1000 suggestions are checked and fed back (23016 feedback events in total).

| ranking | Hits@1 | Hits@10 | MRR |
|---|---|---|---|
| raw (Σ fused score of proposing analogues) | 0.401 | 0.627 | 0.482 |
| learned reliability × Σ fused (online feedback) | 0.410 | 0.630 | 0.489 |

**Learning curve** (Hits@1 per stream segment):

| segment | queries | raw | learned | Δ |
|---|---|---|---|---|
| 1 | 1–300 | 0.460 | 0.467 | +0.007 |
| 2 | 301–600 | 0.410 | 0.420 | +0.010 |
| 3 | 601–900 | 0.410 | 0.407 | -0.003 |
| 4 | 901–1200 | 0.383 | 0.390 | +0.007 |
| 5 | 1201–1500 | 0.400 | 0.420 | +0.020 |
| 6 | 1501–1800 | 0.373 | 0.383 | +0.010 |
| 7 | 1801–2100 | 0.377 | 0.387 | +0.010 |
| 8 | 2101–2400 | 0.393 | 0.407 | +0.013 |

**Rules induced from analogy by the end of the stream** (transfer types with ≥ 20 feedback outcomes and precision ≥ 0.5; 10 in total, top 20):

| rule | precision | outcomes |
|---|---|---|
| `wdt:p1412(x, y) ⇐ wdt:p103(x, y)` | 1.000 | 29 |
| `wdt:p1412(x, y) ⇐ wdt:p6886(x, y)` | 1.000 | 29 |
| `wdt:p27(x, y) ⇐ wdt:p19(x, z) ∧ wdt:p17(z, y)` | 0.747 | 229 |
| `wdt:p27(x, y) ⇐ wdt:p108(x, z) ∧ wdt:p17(z, y)` | 0.713 | 289 |
| `wdt:p27(x, y) ⇐ wdt:p20(x, z) ∧ wdt:p17(z, y)` | 0.707 | 116 |
| `wdt:p27(x, y) ⇐ wdt:p69(x, z) ∧ wdt:p17(z, y)` | 0.669 | 344 |
| `wdt:p27(x, y) ⇐ wdt:p184(x, z) ∧ wdt:p27(z, y)` | 0.566 | 136 |
| `wdt:p27(x, y) ⇐ wdt:p551(x, y)` | 0.550 | 20 |
| `wdt:p27(x, y) ⇐ wdt:p463(x, z) ∧ wdt:p17(z, y)` | 0.517 | 29 |
| `wdt:p69(x, y) ⇐ wdt:p184(x, z) ∧ wdt:p108(z, y)` | 0.516 | 188 |

**Least reliable transfer types** (≥ 20 outcomes):

| type | precision | outcomes |
|---|---|---|
| `wdt:p101<=wdt:p106` | 0.000 | 24 |
| `wdt:p463<=wdt:p69` | 0.000 | 40 |
| `wdt:p108<=wdt:p19` | 0.000 | 60 |
| `wdt:p69<=wdt:p20` | 0.000 | 80 |
| `wdt:p108<=wdt:p20` | 0.000 | 83 |

Runtime 32.3s.
