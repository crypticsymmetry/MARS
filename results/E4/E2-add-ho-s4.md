# E2 — mapper correctness (add-ho-s4)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 425.1ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 1.000 | 0.993 | 0.985 | 0.985 | 0.970 | 0.899 | 0.970 | 1.0000 (n=1000) | 0.271 | 1.000 |
| dev families | 572 | 1.000 | 1.000 | 0.999 | 0.999 | 0.976 | 0.978 | 0.955 | 0.858 | 0.953 | 1.0000 (n=572) | 0.266 | 1.000 |
| test families | 428 | 1.000 | 0.985 | 1.000 | 0.985 | 0.996 | 0.994 | 0.991 | 0.953 | 0.993 | 1.0000 (n=428) | 0.278 | 1.000 |
| discriminable | 953 | 1.000 | 0.993 | 0.999 | 0.993 | 0.999 | 0.996 | 0.995 | 0.919 | 0.995 | 1.0000 (n=953) | 0.278 | 1.000 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 1.000 | 0.944 | 0.217 |
| star | 0.997 | 0.955 | 0.189 |
| tree | 1.000 | 0.927 | 0.259 |
| loop | 1.000 | 0.993 | 0.399 |
| deep-ho | 1.000 | 1.000 | 0.196 |
| comparison | 1.000 | 0.986 | 0.357 |
| mixed | 0.954 | 0.986 | 0.282 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 696 | 14.4 | 19.8 |
| 30–40 | 1595 | 19.5 | 27.0 |
| 40–60 | 709 | 27.8 | 42.1 |
