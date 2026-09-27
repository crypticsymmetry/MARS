# E2 — mapper correctness (insert-intermediate-s2)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 411.6ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.806 | 0.730 | 0.822 | 0.834 | 0.776 | 0.997 | 0.982 | 0.9839 (n=999) | 0.397 | 0.252 |
| dev families | 572 | 1.000 | 1.000 | 0.768 | 0.678 | 0.832 | 0.845 | 0.788 | 0.997 | 0.974 | 0.9842 (n=572) | 0.453 | 0.280 |
| test families | 428 | 1.000 | 0.985 | 0.857 | 0.801 | 0.807 | 0.818 | 0.762 | 0.998 | 0.993 | 0.9835 (n=427) | 0.322 | 0.213 |
| discriminable | 473 | 1.000 | 0.992 | 0.874 | 0.815 | 0.989 | 0.989 | 0.985 | 0.996 | 0.998 | 0.9923 (n=472) | 0.558 | 0.346 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.647 | 0.755 | 0.462 |
| star | 0.686 | 0.752 | 0.580 |
| tree | 0.638 | 0.748 | 0.552 |
| loop | 0.740 | 0.895 | 0.217 |
| deep-ho | 0.629 | 0.657 | 0.126 |
| comparison | 1.000 | 0.794 | 0.413 |
| mixed | 0.772 | 0.835 | 0.430 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 545 | 17.3 | 26.1 |
| 30–40 | 1641 | 23.3 | 36.7 |
| 40–60 | 814 | 31.9 | 52.6 |
