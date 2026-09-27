# E2 — mapper correctness (substitute-predicate-s1)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 364.3ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.997 | 0.948 | 0.881 | 0.874 | 0.857 | 0.948 | 0.947 | 0.9988 (n=1000) | 0.716 | 0.552 |
| dev families | 572 | 1.000 | 1.000 | 0.998 | 0.943 | 0.887 | 0.872 | 0.861 | 0.914 | 0.923 | 0.9986 (n=572) | 0.738 | 0.548 |
| test families | 428 | 1.000 | 0.985 | 0.996 | 0.955 | 0.873 | 0.875 | 0.850 | 0.993 | 0.979 | 0.9992 (n=428) | 0.687 | 0.557 |
| discriminable | 668 | 1.000 | 0.994 | 0.999 | 0.960 | 0.990 | 0.987 | 0.987 | 0.970 | 0.996 | 0.9998 (n=668) | 0.819 | 0.628 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.951 | 0.867 | 0.790 |
| star | 0.901 | 0.776 | 0.664 |
| tree | 0.920 | 0.857 | 0.699 |
| loop | 1.000 | 0.944 | 0.797 |
| deep-ho | 0.931 | 0.699 | 0.503 |
| comparison | 1.000 | 0.951 | 0.797 |
| mixed | 0.934 | 0.901 | 0.761 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 971 | 13.9 | 19.3 |
| 30–40 | 1605 | 19.8 | 27.8 |
| 40–60 | 424 | 31.2 | 47.3 |
