# E2 — mapper correctness (add-ho-s2)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 404.9ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 1.000 | 0.993 | 0.993 | 0.994 | 0.989 | 0.937 | 0.986 | 0.9999 (n=1000) | 0.447 | 0.996 |
| dev families | 572 | 1.000 | 1.000 | 1.000 | 1.000 | 0.988 | 0.990 | 0.980 | 0.909 | 0.976 | 0.9999 (n=572) | 0.455 | 0.996 |
| test families | 428 | 1.000 | 0.985 | 1.000 | 0.985 | 1.000 | 1.000 | 1.000 | 0.974 | 1.000 | 1.0000 (n=428) | 0.437 | 0.995 |
| discriminable | 978 | 1.000 | 0.993 | 1.000 | 0.993 | 0.998 | 1.000 | 0.998 | 0.953 | 1.000 | 0.9999 (n=978) | 0.452 | 0.995 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.998 | 0.979 | 0.385 |
| star | 1.000 | 0.969 | 0.371 |
| tree | 1.000 | 0.979 | 0.427 |
| loop | 1.000 | 0.993 | 0.636 |
| deep-ho | 1.000 | 1.000 | 0.378 |
| comparison | 1.000 | 1.000 | 0.524 |
| mixed | 0.954 | 1.000 | 0.408 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 841 | 14.8 | 22.0 |
| 30–40 | 1578 | 23.0 | 33.3 |
| 40–60 | 581 | 29.4 | 45.9 |
