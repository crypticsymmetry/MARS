# E0 — fingerprint separability (delete-fact-s4)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=128.1, runtime=1.2s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 65/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.519 | 0.842 | 0.000 | 0.540 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.005 | 0.519 | 0.730 | 0.011 | 0.530 | 0.007 | 0.002 | 0.015 | 0.038 |
| exact C0 surface | 0.000 | 0.500 | 0.463 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.504 | 0.517 | 0.803 | 0.499 | 0.523 | 0.274 | 0.247 | 0.311 | 0.662 |
| exact C2 relational | 0.496 | 0.506 | 0.584 | 0.493 | 0.531 | 0.195 | 0.147 | 0.259 | 0.908 |
| exact C3 WL | 0.531 | 0.522 | 0.783 | 0.530 | 0.534 | 0.340 | 0.341 | 0.339 | 0.800 |
| exact C4 topology | 0.507 | 0.512 | 0.627 | 0.524 | 0.514 | 0.197 | 0.136 | 0.277 | 0.446 |
| exact analogy profile | 0.512 | 0.515 | 0.765 | 0.503 | 0.535 | 0.280 | 0.236 | 0.339 | 0.938 |
| exact analogy profile +IDF | 0.504 | 0.514 | 0.897 | 0.496 | 0.530 | 0.338 | 0.323 | 0.357 | 0.954 |
| exact literal profile +IDF | 0.000 | 0.518 | 0.888 | 0.000 | 0.538 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.528 | 0.521 | 0.738 | 0.519 | 0.548 | 0.295 | 0.281 | 0.314 | 0.877 |
| fingerprint analogy +IDF D=2048 | 0.532 | 0.526 | 0.786 | 0.536 | 0.535 | 0.332 | 0.316 | 0.354 | 0.908 |
| fingerprint analogy +IDF D=4096 | 0.522 | 0.524 | 0.824 | 0.548 | 0.562 | 0.343 | 0.325 | 0.367 | 0.923 |
| fingerprint analogy +IDF D=8192 | 0.516 | 0.525 | 0.856 | 0.537 | 0.558 | 0.350 | 0.336 | 0.369 | 0.969 |
| fingerprint analogy +IDF D=16384 | 0.512 | 0.525 | 0.871 | 0.513 | 0.555 | 0.335 | 0.309 | 0.369 | 0.954 |
| fingerprint C1 only D=8192 | 0.509 | 0.523 | 0.921 | 0.494 | 0.532 | 0.326 | 0.313 | 0.343 | 0.631 |
| fingerprint C2 only D=8192 | 0.514 | 0.521 | 0.692 | 0.525 | 0.545 | 0.281 | 0.265 | 0.303 | 0.938 |
| fingerprint C3 only D=8192 | 0.533 | 0.530 | 0.690 | 0.546 | 0.531 | 0.288 | 0.290 | 0.285 | 0.738 |
| fingerprint C4 only D=8192 | 0.504 | 0.515 | 0.623 | 0.513 | 0.516 | 0.218 | 0.178 | 0.272 | 0.408 |
| exact mix C2 only +IDF | 0.500 | 0.509 | 0.707 | 0.493 | 0.536 | 0.295 | 0.274 | 0.324 | 0.862 |
| exact mix C3 only +IDF | 0.534 | 0.527 | 0.773 | 0.530 | 0.540 | 0.335 | 0.344 | 0.321 | 0.923 |
| exact mix C4 only +IDF | 0.512 | 0.515 | 0.699 | 0.535 | 0.539 | 0.258 | 0.207 | 0.325 | 0.508 |
| exact mix C2+C3 +IDF | 0.507 | 0.511 | 0.790 | 0.503 | 0.531 | 0.314 | 0.306 | 0.325 | 0.985 |
| exact mix C2+C3+C4 +IDF | 0.508 | 0.513 | 0.803 | 0.517 | 0.529 | 0.307 | 0.274 | 0.350 | 0.985 |
| exact mix C1+C2+C3 +IDF | 0.505 | 0.517 | 0.918 | 0.484 | 0.525 | 0.334 | 0.322 | 0.350 | 0.892 |
| ablation: no taxonomy | 0.505 | 0.516 | 0.929 | 0.505 | 0.535 | 0.343 | 0.332 | 0.357 | 0.954 |
| ablation: no systematicity (beta=0) | 0.503 | 0.513 | 0.908 | 0.494 | 0.529 | 0.339 | 0.327 | 0.355 | 0.923 |
| ablation: no parent-child | 0.501 | 0.514 | 0.928 | 0.479 | 0.521 | 0.317 | 0.316 | 0.318 | 0.800 |
| ablation: no co-entity | 0.510 | 0.518 | 0.864 | 0.506 | 0.547 | 0.306 | 0.287 | 0.332 | 0.954 |
| ablation: co-entity weight 1.0 | 0.503 | 0.512 | 0.933 | 0.482 | 0.526 | 0.343 | 0.341 | 0.346 | 0.969 |
| ablation: co-entity weight 0.5 | 0.503 | 0.513 | 0.915 | 0.493 | 0.528 | 0.345 | 0.334 | 0.360 | 0.985 |
| ablation: no co-expr | 0.504 | 0.514 | 0.900 | 0.497 | 0.527 | 0.341 | 0.327 | 0.360 | 0.969 |
| ablation: WL depth 1 | 0.508 | 0.515 | 0.898 | 0.500 | 0.535 | 0.340 | 0.329 | 0.355 | 0.954 |
| ablation: WL depth 3 | 0.503 | 0.514 | 0.896 | 0.497 | 0.528 | 0.333 | 0.315 | 0.357 | 0.938 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.259 | 0.315 |
| star | 0.000 | 0.364 | 0.343 |
| tree | 0.000 | 0.329 | 0.336 |
| loop | 0.007 | 0.343 | 0.350 |
| deep-ho (test) | 0.000 | 0.280 | 0.266 |
| comparison (test) | 0.045 | 0.392 | 0.441 |
| mixed (test) | 0.000 | 0.401 | 0.401 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.195 ± 0.053 | 0.095 ± 0.034 | 0.365 ± 0.032 | 0.392 ± 0.066 |
| TA | 0.501 ± 0.016 | 0.347 ± 0.064 | 0.464 ± 0.059 | 0.488 ± 0.017 | 0.472 ± 0.023 |
| MA | 0.106 ± 0.054 | 0.349 ± 0.063 | 0.469 ± 0.052 | 0.491 ± 0.015 | 0.472 ± 0.022 |
| FOR | 0.501 ± 0.015 | 0.352 ± 0.062 | 0.470 ± 0.050 | 0.491 ± 0.015 | 0.473 ± 0.023 |
| RND | 0.496 ± 0.022 | 0.448 ± 0.042 | 0.495 ± 0.016 | 0.499 ± 0.011 | 0.481 ± 0.024 |
