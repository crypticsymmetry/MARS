# E2 — mapper correctness (insert-intermediate-s1)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 393.6ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.878 | 0.815 | 0.916 | 0.908 | 0.884 | 0.996 | 0.997 | 0.9886 (n=999) | 0.484 | 0.337 |
| dev families | 572 | 1.000 | 1.000 | 0.856 | 0.784 | 0.917 | 0.914 | 0.890 | 0.993 | 0.995 | 0.9898 (n=572) | 0.488 | 0.330 |
| test families | 428 | 1.000 | 0.985 | 0.908 | 0.858 | 0.916 | 0.900 | 0.876 | 1.000 | 1.000 | 0.9871 (n=427) | 0.479 | 0.347 |
| discriminable | 663 | 1.000 | 0.994 | 0.908 | 0.857 | 0.991 | 0.994 | 0.988 | 0.994 | 1.000 | 0.9932 (n=663) | 0.554 | 0.375 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.712 | 0.829 | 0.622 |
| star | 0.792 | 0.860 | 0.678 |
| tree | 0.741 | 0.885 | 0.650 |
| loop | 0.889 | 0.986 | 0.000 |
| deep-ho | 0.726 | 0.745 | 0.273 |
| comparison | 1.000 | 0.948 | 0.587 |
| mixed | 0.848 | 0.937 | 0.577 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 804 | 14.6 | 20.6 |
| 30–40 | 1585 | 20.9 | 31.6 |
| 40–60 | 611 | 29.2 | 48.2 |
