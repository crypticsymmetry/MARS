# E6: incremental maintenance (pipeline-n10000)

Mode Pipeline { mac_k: 64 }. 10200 live cases initially, 200 standing queries (k = 5), 5000 updates. Engine build 452.3ms; SQ initialization 424.5ms. **Exactness: 0 mismatches in 250 checks** against from-scratch recomputation.

Recomputing all standing queries from scratch costs about **238 ms** per update (the non-incremental alternative).

| op | n | mean µs | p50 µs | p99 µs | encodes | SQ bound checks | FAC evals | SQ full recomputes | remaps | TMS touched |
|---|---|---|---|---|---|---|---|---|---|---|
| remove-fact | 1790 | 413 | 189 | 3131 | 1.00 | 200 | 18.76 | 0.018 | 0.088 | 0.5 |
| re-add-fact | 1281 | 328 | 180 | 2557 | 0.98 | 197 | 12.04 | 0.022 | 0.095 | 0.6 |
| add-fact | 516 | 473 | 208 | 3548 | 0.99 | 198 | 21.23 | 0.017 | 0.093 | 0.5 |
| add-case | 940 | 270 | 161 | 1891 | 1.00 | 200 | 9.08 | 0.000 | 0.000 | 0.0 |
| remove-case | 473 | 191 | 43 | 2557 | 0.00 | 0 | 11.74 | 0.004 | 0.066 | 0.6 |

All updates: mean 350 µs, median 174 µs → **680× cheaper** (mean) than recomputing all standing queries per update.
