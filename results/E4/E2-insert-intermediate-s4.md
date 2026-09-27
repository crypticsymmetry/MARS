# E2 — mapper correctness (insert-intermediate-s4)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 487.8ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.710 | 0.640 | 0.761 | 0.767 | 0.695 | 0.996 | 0.977 | 0.9758 (n=992) | 0.306 | 0.181 |
| dev families | 572 | 1.000 | 1.000 | 0.653 | 0.571 | 0.800 | 0.786 | 0.720 | 0.993 | 0.967 | 0.9758 (n=572) | 0.392 | 0.222 |
| test families | 428 | 1.000 | 0.985 | 0.787 | 0.733 | 0.709 | 0.742 | 0.660 | 1.000 | 0.991 | 0.9759 (n=420) | 0.192 | 0.120 |
| discriminable | 294 | 1.000 | 0.994 | 0.838 | 0.787 | 0.997 | 0.991 | 0.988 | 0.997 | 1.000 | 0.9931 (n=290) | 0.524 | 0.294 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.576 | 0.696 | 0.329 |
| star | 0.563 | 0.713 | 0.490 |
| tree | 0.511 | 0.682 | 0.483 |
| loop | 0.633 | 0.790 | 0.266 |
| deep-ho | 0.529 | 0.605 | 0.056 |
| comparison | 1.000 | 0.566 | 0.245 |
| mixed | 0.670 | 0.810 | 0.275 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 302 | 17.1 | 27.3 |
| 30–40 | 1211 | 21.5 | 34.4 |
| 40–60 | 1484 | 29.4 | 47.2 |
| 60–1000 | 3 | 39.1 | 42.3 |
