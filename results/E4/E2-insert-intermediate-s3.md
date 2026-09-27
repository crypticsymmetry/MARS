# E2 — mapper correctness (insert-intermediate-s3)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 434.1ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.750 | 0.678 | 0.788 | 0.793 | 0.727 | 0.996 | 0.977 | 0.9815 (n=996) | 0.333 | 0.201 |
| dev families | 572 | 1.000 | 1.000 | 0.705 | 0.618 | 0.804 | 0.825 | 0.753 | 0.997 | 0.967 | 0.9808 (n=572) | 0.409 | 0.239 |
| test families | 428 | 1.000 | 0.985 | 0.811 | 0.759 | 0.768 | 0.750 | 0.690 | 0.995 | 0.991 | 0.9823 (n=424) | 0.231 | 0.147 |
| discriminable | 364 | 1.000 | 0.994 | 0.854 | 0.797 | 0.992 | 0.992 | 0.986 | 0.992 | 0.997 | 0.9933 (n=362) | 0.519 | 0.308 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.601 | 0.734 | 0.378 |
| star | 0.622 | 0.734 | 0.510 |
| tree | 0.576 | 0.720 | 0.517 |
| loop | 0.672 | 0.825 | 0.231 |
| deep-ho | 0.564 | 0.601 | 0.091 |
| comparison | 1.000 | 0.643 | 0.301 |
| mixed | 0.713 | 0.827 | 0.303 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 355 | 15.5 | 21.2 |
| 30–40 | 1466 | 20.0 | 28.5 |
| 40–60 | 1179 | 27.2 | 42.9 |
