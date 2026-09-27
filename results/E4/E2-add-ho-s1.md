# E2 — mapper correctness (add-ho-s1)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 373.6ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.999 | 0.993 | 0.998 | 0.996 | 0.994 | 0.954 | 0.992 | 1.0000 (n=1000) | 0.632 | 0.997 |
| dev families | 572 | 1.000 | 1.000 | 0.999 | 0.999 | 0.997 | 0.993 | 0.990 | 0.923 | 0.986 | 1.0000 (n=572) | 0.656 | 0.997 |
| test families | 428 | 1.000 | 0.985 | 0.999 | 0.985 | 1.000 | 1.000 | 1.000 | 0.995 | 1.000 | 1.0000 (n=428) | 0.600 | 0.996 |
| discriminable | 986 | 1.000 | 0.993 | 0.999 | 0.993 | 1.000 | 1.000 | 1.000 | 0.962 | 1.000 | 1.0000 (n=986) | 0.635 | 0.997 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 1.000 | 0.979 | 0.587 |
| star | 0.998 | 0.997 | 0.601 |
| tree | 1.000 | 0.983 | 0.629 |
| loop | 1.000 | 1.000 | 0.804 |
| deep-ho | 1.000 | 1.000 | 0.538 |
| comparison | 1.000 | 1.000 | 0.685 |
| mixed | 0.954 | 1.000 | 0.577 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 903 | 13.8 | 20.0 |
| 30–40 | 1602 | 20.0 | 28.5 |
| 40–60 | 495 | 29.4 | 42.7 |
