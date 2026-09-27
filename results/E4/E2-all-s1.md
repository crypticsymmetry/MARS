# E2 — mapper correctness (all-s1)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 419.6ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.962 | 0.906 | 0.902 | 0.904 | 0.868 | 0.889 | 0.951 | 0.9934 (n=999) | 0.588 | 0.431 |
| dev families | 572 | 1.000 | 1.000 | 0.962 | 0.900 | 0.899 | 0.903 | 0.865 | 0.867 | 0.944 | 0.9941 (n=572) | 0.615 | 0.439 |
| test families | 428 | 1.000 | 0.985 | 0.962 | 0.913 | 0.905 | 0.907 | 0.871 | 0.918 | 0.960 | 0.9925 (n=427) | 0.551 | 0.420 |
| discriminable | 731 | 1.000 | 0.994 | 0.981 | 0.937 | 0.993 | 0.997 | 0.992 | 0.925 | 0.999 | 0.9973 (n=731) | 0.702 | 0.520 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.886 | 0.846 | 0.636 |
| star | 0.867 | 0.825 | 0.608 |
| tree | 0.877 | 0.839 | 0.608 |
| loop | 0.971 | 0.951 | 0.608 |
| deep-ho | 0.842 | 0.755 | 0.294 |
| comparison | 1.000 | 0.951 | 0.762 |
| mixed | 0.896 | 0.908 | 0.599 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 0–20 | 1 | 8.5 | 8.5 |
| 20–30 | 961 | 16.3 | 26.6 |
| 30–40 | 1577 | 24.7 | 38.7 |
| 40–60 | 461 | 35.0 | 54.1 |
