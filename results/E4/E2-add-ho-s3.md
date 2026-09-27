# E2 — mapper correctness (add-ho-s3)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 437.9ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 1.000 | 0.994 | 0.991 | 0.991 | 0.982 | 0.936 | 0.981 | 1.0000 (n=1000) | 0.342 | 0.997 |
| dev families | 572 | 1.000 | 1.000 | 1.000 | 1.000 | 0.983 | 0.986 | 0.970 | 0.916 | 0.972 | 1.0000 (n=572) | 0.346 | 0.995 |
| test families | 428 | 1.000 | 0.985 | 1.000 | 0.985 | 1.000 | 0.996 | 0.996 | 0.963 | 0.993 | 1.0000 (n=428) | 0.336 | 1.000 |
| discriminable | 968 | 1.000 | 0.993 | 1.000 | 0.994 | 0.999 | 1.000 | 0.999 | 0.951 | 0.999 | 1.0000 (n=968) | 0.345 | 0.997 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 1.000 | 0.937 | 0.287 |
| star | 1.000 | 0.965 | 0.273 |
| tree | 1.000 | 0.979 | 0.322 |
| loop | 1.000 | 1.000 | 0.503 |
| deep-ho | 1.000 | 1.000 | 0.259 |
| comparison | 1.000 | 0.993 | 0.399 |
| mixed | 0.956 | 0.996 | 0.352 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 782 | 15.4 | 22.8 |
| 30–40 | 1579 | 22.0 | 34.9 |
| 40–60 | 639 | 30.7 | 51.6 |
