# E2 — mapper correctness (substitute-predicate-s4)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 364.5ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.895 | 0.762 | 0.688 | 0.700 | 0.599 | 0.686 | 0.681 | 0.9912 (n=1000) | 0.320 | 0.202 |
| dev families | 572 | 1.000 | 1.000 | 0.880 | 0.722 | 0.686 | 0.687 | 0.599 | 0.622 | 0.645 | 0.9921 (n=572) | 0.339 | 0.210 |
| test families | 428 | 1.000 | 0.985 | 0.914 | 0.815 | 0.689 | 0.716 | 0.598 | 0.771 | 0.729 | 0.9900 (n=428) | 0.294 | 0.191 |
| discriminable | 177 | 1.000 | 0.994 | 0.992 | 0.906 | 0.966 | 0.960 | 0.938 | 0.949 | 0.960 | 1.0000 (n=177) | 0.610 | 0.344 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.703 | 0.594 | 0.350 |
| star | 0.635 | 0.528 | 0.266 |
| tree | 0.702 | 0.570 | 0.329 |
| loop | 0.847 | 0.703 | 0.413 |
| deep-ho | 0.613 | 0.423 | 0.105 |
| comparison | 0.965 | 0.696 | 0.427 |
| mixed | 0.869 | 0.676 | 0.352 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 971 | 14.5 | 22.7 |
| 30–40 | 1590 | 21.9 | 31.6 |
| 40–60 | 439 | 32.5 | 48.5 |
