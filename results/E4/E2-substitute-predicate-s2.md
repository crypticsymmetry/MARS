# E2 — mapper correctness (substitute-predicate-s2)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 376.3ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.982 | 0.893 | 0.823 | 0.823 | 0.769 | 0.832 | 0.850 | 0.9970 (n=1000) | 0.541 | 0.373 |
| dev families | 572 | 1.000 | 1.000 | 0.981 | 0.874 | 0.819 | 0.819 | 0.767 | 0.787 | 0.827 | 0.9974 (n=572) | 0.582 | 0.387 |
| test families | 428 | 1.000 | 0.985 | 0.983 | 0.919 | 0.827 | 0.827 | 0.772 | 0.893 | 0.881 | 0.9965 (n=428) | 0.486 | 0.352 |
| discriminable | 427 | 1.000 | 0.994 | 0.998 | 0.937 | 0.979 | 0.988 | 0.970 | 0.948 | 0.986 | 0.9993 (n=427) | 0.717 | 0.476 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.892 | 0.790 | 0.657 |
| star | 0.808 | 0.685 | 0.503 |
| tree | 0.845 | 0.731 | 0.510 |
| loop | 0.951 | 0.860 | 0.657 |
| deep-ho | 0.849 | 0.598 | 0.273 |
| comparison | 0.986 | 0.881 | 0.594 |
| mixed | 0.923 | 0.838 | 0.592 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 955 | 14.6 | 21.4 |
| 30–40 | 1620 | 22.2 | 31.3 |
| 40–60 | 425 | 33.6 | 51.1 |
