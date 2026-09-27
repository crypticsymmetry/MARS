# E6: incremental maintenance (exact-n10000)

Mode ExactFused. 10200 live cases initially, 200 standing queries (k = 5), 3000 updates. Engine build 502.7ms; SQ initialization 18.8s. **Exactness: 0 mismatches in 150 checks** against from-scratch recomputation.

Recomputing all standing queries from scratch costs about **16759 ms** per update (the non-incremental alternative).

| op | n | mean µs | p50 µs | p99 µs | encodes | SQ bound checks | FAC evals | SQ full recomputes | remaps | TMS touched |
|---|---|---|---|---|---|---|---|---|---|---|
| remove-fact | 1059 | 3545 | 1806 | 88940 | 1.00 | 200 | 385.58 | 0.018 | 0.098 | 0.6 |
| re-add-fact | 765 | 4205 | 1839 | 104251 | 0.98 | 196 | 440.28 | 0.024 | 0.099 | 0.6 |
| add-fact | 314 | 3681 | 2014 | 102838 | 0.99 | 198 | 362.11 | 0.016 | 0.096 | 0.6 |
| add-case | 573 | 1924 | 1850 | 3448 | 1.00 | 200 | 200.00 | 0.000 | 0.000 | 0.0 |
| remove-case | 289 | 7 | 4 | 57 | 0.00 | 0 | 0.00 | 0.000 | 0.080 | 0.7 |

All updates: mean 3077 µs, median 1806 µs → **5447× cheaper** (mean) than recomputing all standing queries per update.
