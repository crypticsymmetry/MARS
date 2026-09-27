# E2 — mapper correctness (all-s3)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 398.4ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.835 | 0.731 | 0.730 | 0.717 | 0.632 | 0.725 | 0.764 | 0.9833 (n=1000) | 0.291 | 0.173 |
| dev families | 572 | 1.000 | 1.000 | 0.817 | 0.694 | 0.714 | 0.710 | 0.632 | 0.713 | 0.753 | 0.9851 (n=572) | 0.334 | 0.198 |
| test families | 428 | 1.000 | 0.985 | 0.859 | 0.780 | 0.751 | 0.727 | 0.633 | 0.739 | 0.778 | 0.9809 (n=428) | 0.234 | 0.137 |
| discriminable | 410 | 1.000 | 0.994 | 0.947 | 0.864 | 0.973 | 0.968 | 0.949 | 0.846 | 0.980 | 0.9963 (n=410) | 0.522 | 0.310 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.696 | 0.650 | 0.315 |
| star | 0.637 | 0.577 | 0.329 |
| tree | 0.671 | 0.608 | 0.308 |
| loop | 0.772 | 0.692 | 0.385 |
| deep-ho | 0.599 | 0.497 | 0.049 |
| comparison | 0.945 | 0.650 | 0.378 |
| mixed | 0.796 | 0.754 | 0.275 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 0–20 | 5 | 8.3 | 10.9 |
| 20–30 | 928 | 15.0 | 21.7 |
| 30–40 | 1534 | 20.9 | 31.1 |
| 40–60 | 533 | 30.4 | 47.5 |
