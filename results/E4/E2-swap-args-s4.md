# E2 — mapper correctness (swap-args-s4)

Config: groups=1000, seed=1, distractors=2, MapConfig::default(). Runtime 377.2ms.

| split | n | LS corr. P | LS corr. R | TA corr. P | TA corr. R | FAC win TA>MA | FAC win TA>FOR | FAC TA-top | FP TA-top | fused TA-top | greedy/optimal | CI recall (TA⁻) | structural CI precision |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| all | 1000 | 1.000 | 0.994 | 0.699 | 0.611 | 0.637 | 0.648 | 0.525 | 0.995 | 0.903 | 0.9670 (n=1000) | 0.283 | 0.167 |
| dev families | 572 | 1.000 | 1.000 | 0.678 | 0.556 | 0.605 | 0.615 | 0.476 | 0.991 | 0.855 | 0.9627 (n=572) | 0.267 | 0.147 |
| test families | 428 | 1.000 | 0.985 | 0.728 | 0.683 | 0.680 | 0.692 | 0.591 | 1.000 | 0.967 | 0.9726 (n=428) | 0.304 | 0.198 |
| discriminable | 409 | 1.000 | 0.994 | 0.962 | 0.878 | 0.968 | 0.971 | 0.949 | 1.000 | 0.990 | 0.9961 (n=409) | 0.645 | 0.408 |

| family | TA corr. R | FAC TA-top | CI recall |
|---|---|---|---|
| chain | 0.527 | 0.500 | 0.238 |
| star | 0.526 | 0.462 | 0.266 |
| tree | 0.507 | 0.451 | 0.238 |
| loop | 0.664 | 0.493 | 0.329 |
| deep-ho | 0.440 | 0.364 | 0.203 |
| comparison | 0.979 | 0.703 | 0.469 |
| mixed | 0.631 | 0.708 | 0.239 |

## Time per mapping (single thread, µs)

| exprs in pair | n | mean µs | p95 µs |
|---|---|---|---|
| 20–30 | 956 | 14.9 | 22.7 |
| 30–40 | 1619 | 22.0 | 32.0 |
| 40–60 | 425 | 33.3 | 55.8 |
