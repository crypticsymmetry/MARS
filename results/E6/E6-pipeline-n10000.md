# E6: incremental maintenance (pipeline-n10000)

Mode Pipeline { mac_k: 64 }. 10200 live cases initially, 200 standing queries (k = 5), 5000 updates. Engine build 543.6ms; SQ initialization 381.8ms. **Exactness: 0 mismatches in 250 checks** against from-scratch recomputation.

Recomputing all standing queries from scratch costs about **252 ms** per update (the non-incremental alternative).

| op | n | mean µs | p50 µs | p99 µs | encodes | SQ bound checks | FAC evals | SQ full recomputes | remaps | TMS touched |
|---|---|---|---|---|---|---|---|---|---|---|
| remove-fact | 1789 | 425 | 197 | 3235 | 1.00 | 200 | 18.65 | 0.020 | 0.242 | 4.6 |
| re-add-fact | 1281 | 346 | 190 | 2681 | 0.99 | 197 | 12.09 | 0.023 | 0.231 | 4.3 |
| add-fact | 517 | 487 | 227 | 3604 | 0.99 | 198 | 21.25 | 0.023 | 0.250 | 4.8 |
| add-case | 941 | 284 | 170 | 2084 | 1.00 | 200 | 9.20 | 0.000 | 0.003 | 0.1 |
| remove-case | 472 | 207 | 49 | 2630 | 0.00 | 0 | 12.07 | 0.004 | 0.201 | 4.5 |

All updates: mean 364 µs, median 182 µs → **691× cheaper** (mean) than recomputing all standing queries per update.
