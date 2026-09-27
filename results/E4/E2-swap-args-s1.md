# E2 — mapper correctness (swap-args-s1)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 354.4ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.966 | 0.873 | 0.865 | 0.862 | 0.821 | 0.998 | 0.991 | 0.9842 (n=1000) | 0.409 | 0.251 |
| dev families | 572 | 1.000 | 1.000 | 0.968 | 0.856 | 0.864 | 0.865 | 0.830 | 0.997 | 0.986 | 0.9864 (n=572) | 0.451 | 0.257 |
| test families | 428 | 1.000 | 0.985 | 0.963 | 0.895 | 0.867 | 0.857 | 0.808 | 1.000 | 0.998 | 0.9812 (n=428) | 0.353 | 0.242 |
| discriminable | 671 | 1.000 | 0.994 | 0.993 | 0.912 | 0.997 | 1.000 | 0.997 | 0.997 | 0.999 | 0.9972 (n=671) | 0.583 | 0.348 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.829 | 0.773 | 0.469 |
| star | 0.787 | 0.755 | 0.385 |
| tree | 0.810 | 0.790 | 0.469 |
| loop | 1.000 | 1.000 | 0.483 |
| deep-ho | 0.805 | 0.601 | 0.000 |
| comparison | 1.000 | 0.958 | 0.573 |
| mixed | 0.879 | 0.866 | 0.486 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 963 | 14.5 | 20.6 |
| 30–40 | 1612 | 21.1 | 30.8 |
| 40–60 | 425 | 31.1 | 46.5 |
