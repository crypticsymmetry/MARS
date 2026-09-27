# E2 — mapper correctness (delete-fact-s2)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 308.8ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.982 | 0.832 | 0.686 | 0.699 | 0.589 | 0.515 | 0.576 | 0.9965 (n=1000) | 0.433 | 0.196 |
| dev families | 572 | 1.000 | 1.000 | 0.975 | 0.792 | 0.696 | 0.712 | 0.601 | 0.512 | 0.573 | 0.9952 (n=572) | 0.453 | 0.212 |
| test families | 428 | 1.000 | 0.985 | 0.991 | 0.885 | 0.673 | 0.682 | 0.571 | 0.519 | 0.579 | 0.9982 (n=428) | 0.407 | 0.171 |
| discriminable | 388 | 1.000 | 0.994 | 0.997 | 0.897 | 0.992 | 0.979 | 0.974 | 0.902 | 0.982 | 0.9995 (n=388) | 0.961 | 0.362 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.792 | 0.577 | 0.385 |
| star | 0.714 | 0.549 | 0.378 |
| tree | 0.739 | 0.535 | 0.294 |
| loop | 0.923 | 0.745 | 0.755 |
| deep-ho | 0.800 | 0.315 | 0.007 |
| comparison | 0.968 | 0.703 | 0.671 |
| mixed | 0.888 | 0.697 | 0.542 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 0–20 | 52 | 7.7 | 10.6 |
| 20–30 | 1455 | 11.9 | 18.0 |
| 30–40 | 1232 | 19.0 | 28.6 |
| 40–60 | 261 | 29.0 | 40.9 |
