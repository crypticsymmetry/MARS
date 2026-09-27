# E6: incremental maintenance (pipeline-n1000000)

Mode Pipeline { mac_k: 64 }. 1000200 live cases initially, 200 standing queries (k = 5), 5000 updates. Engine build 57.7s; SQ initialization 5.1s. **Exactness: 0 mismatches in 250 checks** against from-scratch recomputation.

Recomputing all standing queries from scratch costs about **5860 ms** per update (the non-incremental alternative).

| op | n | mean µs | p50 µs | p99 µs | encodes | SQ bound checks | FAC evals | SQ full recomputes | remaps | TMS touched |
|---|---|---|---|---|---|---|---|---|---|---|
| remove-fact | 1780 | 314 | 133 | 1750 | 1.00 | 200 | 12.15 | 0.000 | 0.060 | 0.4 |
| re-add-fact | 1277 | 237 | 132 | 1555 | 0.99 | 198 | 6.80 | 0.000 | 0.062 | 0.4 |
| add-fact | 494 | 361 | 152 | 1997 | 0.99 | 198 | 14.14 | 0.000 | 0.071 | 0.4 |
| add-case | 987 | 261 | 120 | 770 | 1.00 | 200 | 0.59 | 0.000 | 0.000 | 0.0 |
| remove-case | 462 | 187 | 11 | 1813 | 0.00 | 0 | 10.29 | 0.000 | 0.052 | 0.5 |

All updates: mean 277 µs, median 127 µs → **21182× cheaper** (mean) than recomputing all standing queries per update.
