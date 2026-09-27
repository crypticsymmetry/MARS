# E2 — mapper correctness (swap-args-s2)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 416.3ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.802 | 0.717 | 0.686 | 0.695 | 0.593 | 0.991 | 0.915 | 0.9733 (n=1000) | 0.416 | 0.265 |
| dev families | 572 | 1.000 | 1.000 | 0.772 | 0.658 | 0.655 | 0.661 | 0.555 | 0.984 | 0.871 | 0.9671 (n=572) | 0.395 | 0.236 |
| test families | 428 | 1.000 | 0.985 | 0.843 | 0.796 | 0.729 | 0.741 | 0.644 | 1.000 | 0.974 | 0.9816 (n=428) | 0.444 | 0.309 |
| discriminable | 542 | 1.000 | 0.993 | 0.965 | 0.897 | 0.969 | 0.976 | 0.952 | 0.996 | 0.998 | 0.9978 (n=542) | 0.745 | 0.532 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.659 | 0.598 | 0.406 |
| star | 0.636 | 0.563 | 0.399 |
| tree | 0.581 | 0.503 | 0.392 |
| loop | 0.755 | 0.556 | 0.385 |
| deep-ho | 0.604 | 0.462 | 0.350 |
| comparison | 0.958 | 0.699 | 0.490 |
| mixed | 0.825 | 0.771 | 0.493 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 970 | 15.5 | 23.2 |
| 30–40 | 1606 | 23.7 | 34.3 |
| 40–60 | 424 | 35.1 | 56.1 |
