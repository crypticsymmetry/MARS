# E2 — mapper correctness (swap-args-s3)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 351.5ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.776 | 0.687 | 0.700 | 0.698 | 0.622 | 0.998 | 0.939 | 0.9696 (n=1000) | 0.299 | 0.172 |
| dev families | 572 | 1.000 | 1.000 | 0.750 | 0.641 | 0.665 | 0.667 | 0.583 | 0.998 | 0.914 | 0.9670 (n=572) | 0.308 | 0.163 |
| test families | 428 | 1.000 | 0.985 | 0.810 | 0.748 | 0.745 | 0.738 | 0.674 | 0.998 | 0.972 | 0.9729 (n=428) | 0.287 | 0.187 |
| discriminable | 484 | 1.000 | 0.994 | 0.974 | 0.889 | 0.978 | 0.981 | 0.972 | 1.000 | 0.996 | 0.9961 (n=484) | 0.579 | 0.347 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.579 | 0.531 | 0.266 |
| star | 0.622 | 0.552 | 0.266 |
| tree | 0.562 | 0.587 | 0.329 |
| loop | 0.800 | 0.661 | 0.371 |
| deep-ho | 0.552 | 0.510 | 0.000 |
| comparison | 0.965 | 0.790 | 0.476 |
| mixed | 0.727 | 0.722 | 0.387 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 965 | 15.4 | 23.0 |
| 30–40 | 1596 | 21.6 | 31.5 |
| 40–60 | 439 | 32.3 | 52.2 |
