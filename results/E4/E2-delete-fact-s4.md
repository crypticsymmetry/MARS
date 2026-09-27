# E2 — mapper correctness (delete-fact-s4)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 311.5ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.960 | 0.725 | 0.520 | 0.535 | 0.375 | 0.342 | 0.382 | 0.9930 (n=1000) | 0.113 | 0.039 |
| dev families | 572 | 1.000 | 1.000 | 0.952 | 0.675 | 0.525 | 0.519 | 0.367 | 0.333 | 0.379 | 0.9942 (n=572) | 0.079 | 0.035 |
| test families | 428 | 1.000 | 0.985 | 0.971 | 0.792 | 0.513 | 0.556 | 0.386 | 0.353 | 0.386 | 0.9912 (n=428) | 0.159 | 0.045 |
| discriminable | 65 | 1.000 | 0.992 | 0.984 | 0.808 | 0.969 | 1.000 | 0.969 | 0.954 | 0.969 | 1.0000 (n=65) | 0.954 | 0.303 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.690 | 0.357 | 0.035 |
| star | 0.583 | 0.346 | 0.056 |
| tree | 0.642 | 0.336 | 0.000 |
| loop | 0.784 | 0.430 | 0.224 |
| deep-ho | 0.744 | 0.332 | 0.000 |
| comparison | 0.882 | 0.381 | 0.322 |
| mixed | 0.749 | 0.444 | 0.155 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 0–20 | 65 | 7.8 | 11.6 |
| 20–30 | 1634 | 11.6 | 17.2 |
| 30–40 | 1118 | 18.2 | 27.5 |
| 40–60 | 183 | 29.9 | 45.2 |
