# E2 — mapper correctness (all-s2)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 400.7ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.905 | 0.820 | 0.803 | 0.816 | 0.754 | 0.813 | 0.855 | 0.9880 (n=999) | 0.404 | 0.259 |
| dev families | 572 | 1.000 | 1.000 | 0.897 | 0.799 | 0.801 | 0.811 | 0.753 | 0.815 | 0.853 | 0.9887 (n=572) | 0.451 | 0.281 |
| test families | 428 | 1.000 | 0.985 | 0.916 | 0.848 | 0.806 | 0.821 | 0.756 | 0.811 | 0.857 | 0.9871 (n=427) | 0.341 | 0.226 |
| discriminable | 552 | 1.000 | 0.995 | 0.962 | 0.900 | 0.976 | 0.975 | 0.966 | 0.897 | 0.987 | 0.9968 (n=552) | 0.580 | 0.376 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.798 | 0.741 | 0.420 |
| star | 0.758 | 0.675 | 0.455 |
| tree | 0.763 | 0.762 | 0.441 |
| loop | 0.878 | 0.832 | 0.490 |
| deep-ho | 0.710 | 0.615 | 0.126 |
| comparison | 0.979 | 0.818 | 0.476 |
| mixed | 0.854 | 0.835 | 0.423 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 0–20 | 2 | 10.4 | 11.0 |
| 20–30 | 970 | 15.6 | 23.3 |
| 30–40 | 1536 | 23.7 | 37.7 |
| 40–60 | 492 | 34.2 | 55.0 |
