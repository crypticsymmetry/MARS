# E6: incremental maintenance (pipeline-n100000)

Mode Pipeline { mac_k: 64 }. 100200 live cases initially, 200 standing queries (k = 5), 5000 updates. Engine build 4.4s; SQ initialization 674.1ms. **Exactness: 0 mismatches in 250 checks** against from-scratch recomputation.

Recomputing all standing queries from scratch costs about **444 ms** per update (the non-incremental alternative).

| op | n | mean µs | p50 µs | p99 µs | encodes | SQ bound checks | FAC evals | SQ full recomputes | remaps | TMS touched |
|---|---|---|---|---|---|---|---|---|---|---|
| remove-fact | 1775 | 377 | 139 | 2828 | 1.00 | 200 | 15.47 | 0.001 | 0.060 | 0.4 |
| re-add-fact | 1261 | 261 | 138 | 2185 | 0.99 | 198 | 7.34 | 0.002 | 0.065 | 0.4 |
| add-fact | 509 | 387 | 154 | 2737 | 0.99 | 199 | 15.46 | 0.000 | 0.075 | 0.4 |
| add-case | 988 | 197 | 118 | 1489 | 1.00 | 200 | 3.05 | 0.000 | 0.000 | 0.0 |
| remove-case | 467 | 235 | 11 | 3067 | 0.00 | 0 | 11.91 | 0.000 | 0.051 | 0.5 |

All updates: mean 300 µs, median 129 µs → **1482× cheaper** (mean) than recomputing all standing queries per update.
