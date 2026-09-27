# E6: incremental maintenance (pipeline-n100000)

Mode Pipeline { mac_k: 64 }. 100200 live cases initially, 200 standing queries (k = 5), 5000 updates. Engine build 4.7s; SQ initialization 714.8ms. **Exactness: 0 mismatches in 250 checks** against from-scratch recomputation.

Recomputing all standing queries from scratch costs about **453 ms** per update (the non-incremental alternative).

| op | n | mean µs | p50 µs | p99 µs | encodes | SQ bound checks | FAC evals | SQ full recomputes | remaps | TMS touched |
|---|---|---|---|---|---|---|---|---|---|---|
| remove-fact | 1775 | 379 | 143 | 2585 | 1.00 | 200 | 15.57 | 0.002 | 0.199 | 3.7 |
| re-add-fact | 1261 | 264 | 144 | 1824 | 0.99 | 198 | 7.24 | 0.003 | 0.201 | 3.7 |
| add-fact | 509 | 395 | 163 | 2329 | 0.99 | 199 | 15.95 | 0.000 | 0.194 | 3.6 |
| add-case | 988 | 199 | 130 | 1473 | 1.00 | 200 | 3.05 | 0.000 | 0.001 | 0.0 |
| remove-case | 467 | 221 | 12 | 1995 | 0.00 | 0 | 11.49 | 0.000 | 0.176 | 3.8 |

All updates: mean 301 µs, median 136 µs → **1505× cheaper** (mean) than recomputing all standing queries per update.
