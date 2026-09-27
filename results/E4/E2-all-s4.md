# E2 — mapper correctness (all-s4)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 382.6ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.777 | 0.664 | 0.705 | 0.682 | 0.595 | 0.670 | 0.698 | 0.9782 (n=999) | 0.204 | 0.121 |
| dev families | 572 | 1.000 | 1.000 | 0.751 | 0.614 | 0.687 | 0.647 | 0.563 | 0.666 | 0.685 | 0.9768 (n=572) | 0.226 | 0.131 |
| test families | 428 | 1.000 | 0.985 | 0.812 | 0.730 | 0.728 | 0.729 | 0.637 | 0.675 | 0.715 | 0.9801 (n=427) | 0.175 | 0.106 |
| discriminable | 316 | 1.000 | 0.995 | 0.935 | 0.849 | 0.975 | 0.978 | 0.953 | 0.858 | 0.981 | 0.9965 (n=316) | 0.459 | 0.272 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.588 | 0.566 | 0.182 |
| star | 0.574 | 0.503 | 0.252 |
| tree | 0.593 | 0.573 | 0.189 |
| loop | 0.700 | 0.608 | 0.280 |
| deep-ho | 0.536 | 0.514 | 0.028 |
| comparison | 0.923 | 0.727 | 0.329 |
| mixed | 0.732 | 0.669 | 0.169 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 0–20 | 4 | 6.5 | 8.1 |
| 20–30 | 892 | 13.8 | 19.5 |
| 30–40 | 1541 | 20.6 | 29.0 |
| 40–60 | 563 | 30.4 | 46.7 |
