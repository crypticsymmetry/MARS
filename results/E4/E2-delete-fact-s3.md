# E2 — mapper correctness (delete-fact-s3)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 308.0ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.974 | 0.769 | 0.603 | 0.599 | 0.468 | 0.375 | 0.438 | 0.9944 (n=1000) | 0.234 | 0.100 |
| dev families | 572 | 1.000 | 1.000 | 0.968 | 0.719 | 0.589 | 0.606 | 0.462 | 0.357 | 0.425 | 0.9926 (n=572) | 0.215 | 0.104 |
| test families | 428 | 1.000 | 0.985 | 0.981 | 0.835 | 0.620 | 0.590 | 0.475 | 0.400 | 0.456 | 0.9967 (n=428) | 0.259 | 0.092 |
| discriminable | 175 | 1.000 | 0.991 | 0.994 | 0.831 | 0.960 | 0.983 | 0.954 | 0.897 | 0.971 | 0.9997 (n=175) | 0.966 | 0.324 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.727 | 0.423 | 0.119 |
| star | 0.643 | 0.409 | 0.175 |
| tree | 0.676 | 0.434 | 0.084 |
| loop | 0.832 | 0.580 | 0.483 |
| deep-ho | 0.752 | 0.402 | 0.000 |
| comparison | 0.931 | 0.469 | 0.476 |
| mixed | 0.820 | 0.556 | 0.303 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 0–20 | 65 | 8.7 | 11.4 |
| 20–30 | 1567 | 13.4 | 18.9 |
| 30–40 | 1158 | 20.2 | 30.4 |
| 40–60 | 210 | 31.9 | 45.3 |
