# E2 — mapper correctness (substitute-predicate-s3)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 539.6ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.949 | 0.835 | 0.758 | 0.734 | 0.677 | 0.738 | 0.754 | 0.9934 (n=1000) | 0.430 | 0.280 |
| dev families | 572 | 1.000 | 1.000 | 0.942 | 0.807 | 0.773 | 0.733 | 0.678 | 0.689 | 0.734 | 0.9925 (n=572) | 0.449 | 0.282 |
| test families | 428 | 1.000 | 0.985 | 0.959 | 0.873 | 0.738 | 0.735 | 0.676 | 0.804 | 0.780 | 0.9947 (n=428) | 0.404 | 0.278 |
| discriminable | 285 | 1.000 | 0.993 | 0.996 | 0.921 | 0.986 | 0.975 | 0.965 | 0.923 | 0.982 | 0.9997 (n=285) | 0.684 | 0.408 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.823 | 0.682 | 0.483 |
| star | 0.715 | 0.577 | 0.357 |
| tree | 0.775 | 0.643 | 0.420 |
| loop | 0.915 | 0.811 | 0.538 |
| deep-ho | 0.741 | 0.486 | 0.182 |
| comparison | 0.979 | 0.780 | 0.517 |
| mixed | 0.900 | 0.764 | 0.514 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 967 | 16.0 | 24.8 |
| 30–40 | 1601 | 23.9 | 35.3 |
| 40–60 | 432 | 35.5 | 51.1 |
