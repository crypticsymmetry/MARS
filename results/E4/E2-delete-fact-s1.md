# E2 — mapper correctness (delete-fact-s1)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 326.4ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.996 | 0.921 | 0.879 | 0.872 | 0.813 | 0.706 | 0.782 | 0.9992 (n=1000) | 0.719 | 0.388 |
| dev families | 572 | 1.000 | 1.000 | 0.997 | 0.906 | 0.903 | 0.905 | 0.855 | 0.724 | 0.830 | 0.9991 (n=572) | 0.795 | 0.423 |
| test families | 428 | 1.000 | 0.985 | 0.995 | 0.941 | 0.847 | 0.828 | 0.757 | 0.682 | 0.717 | 0.9993 (n=428) | 0.617 | 0.334 |
| discriminable | 694 | 1.000 | 0.993 | 0.997 | 0.932 | 0.996 | 0.997 | 0.994 | 0.903 | 0.994 | 0.9996 (n=694) | 0.970 | 0.495 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.903 | 0.874 | 0.797 |
| star | 0.836 | 0.783 | 0.650 |
| tree | 0.886 | 0.801 | 0.748 |
| loop | 1.000 | 0.962 | 0.986 |
| deep-ho | 0.890 | 0.455 | 0.140 |
| comparison | 1.000 | 0.941 | 0.888 |
| mixed | 0.934 | 0.877 | 0.824 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 0–20 | 10 | 9.1 | 12.0 |
| 20–30 | 1244 | 14.3 | 21.6 |
| 30–40 | 1413 | 21.2 | 30.3 |
| 40–60 | 333 | 32.2 | 46.9 |
