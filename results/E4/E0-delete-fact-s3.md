# E0 — fingerprint separability (delete-fact-s3)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=130.9, runtime=1.3s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 175/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.505 | 0.870 | 0.000 | 0.501 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.007 | 0.506 | 0.786 | 0.008 | 0.513 | 0.008 | 0.003 | 0.014 | 0.023 |
| exact C0 surface | 0.000 | 0.500 | 0.461 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.496 | 0.508 | 0.859 | 0.490 | 0.514 | 0.275 | 0.259 | 0.297 | 0.534 |
| exact C2 relational | 0.518 | 0.516 | 0.670 | 0.529 | 0.531 | 0.270 | 0.235 | 0.315 | 0.857 |
| exact C3 WL | 0.548 | 0.560 | 0.835 | 0.568 | 0.597 | 0.396 | 0.390 | 0.403 | 0.771 |
| exact C4 topology | 0.514 | 0.519 | 0.665 | 0.507 | 0.516 | 0.204 | 0.141 | 0.290 | 0.400 |
| exact analogy profile | 0.525 | 0.531 | 0.824 | 0.522 | 0.553 | 0.333 | 0.308 | 0.367 | 0.886 |
| exact analogy profile +IDF | 0.521 | 0.528 | 0.927 | 0.521 | 0.553 | 0.366 | 0.348 | 0.390 | 0.897 |
| exact literal profile +IDF | 0.000 | 0.526 | 0.918 | 0.000 | 0.547 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.524 | 0.530 | 0.782 | 0.538 | 0.565 | 0.349 | 0.339 | 0.363 | 0.857 |
| fingerprint analogy +IDF D=2048 | 0.520 | 0.540 | 0.832 | 0.534 | 0.580 | 0.379 | 0.357 | 0.409 | 0.897 |
| fingerprint analogy +IDF D=4096 | 0.521 | 0.535 | 0.860 | 0.551 | 0.572 | 0.387 | 0.377 | 0.400 | 0.897 |
| fingerprint analogy +IDF D=8192 | 0.521 | 0.533 | 0.886 | 0.543 | 0.569 | 0.383 | 0.360 | 0.414 | 0.909 |
| fingerprint analogy +IDF D=16384 | 0.520 | 0.529 | 0.903 | 0.539 | 0.557 | 0.376 | 0.369 | 0.386 | 0.897 |
| fingerprint C1 only D=8192 | 0.493 | 0.508 | 0.945 | 0.484 | 0.496 | 0.307 | 0.295 | 0.324 | 0.449 |
| fingerprint C2 only D=8192 | 0.525 | 0.540 | 0.750 | 0.534 | 0.567 | 0.339 | 0.325 | 0.356 | 0.863 |
| fingerprint C3 only D=8192 | 0.543 | 0.544 | 0.709 | 0.548 | 0.572 | 0.349 | 0.337 | 0.364 | 0.826 |
| fingerprint C4 only D=8192 | 0.515 | 0.513 | 0.649 | 0.528 | 0.540 | 0.236 | 0.197 | 0.290 | 0.386 |
| exact mix C2 only +IDF | 0.524 | 0.523 | 0.772 | 0.538 | 0.538 | 0.348 | 0.313 | 0.395 | 0.880 |
| exact mix C3 only +IDF | 0.557 | 0.569 | 0.827 | 0.564 | 0.613 | 0.410 | 0.411 | 0.409 | 0.931 |
| exact mix C4 only +IDF | 0.507 | 0.513 | 0.723 | 0.498 | 0.523 | 0.241 | 0.197 | 0.301 | 0.451 |
| exact mix C2+C3 +IDF | 0.526 | 0.532 | 0.837 | 0.550 | 0.558 | 0.375 | 0.357 | 0.400 | 0.914 |
| exact mix C2+C3+C4 +IDF | 0.526 | 0.530 | 0.842 | 0.550 | 0.557 | 0.369 | 0.341 | 0.407 | 0.903 |
| exact mix C1+C2+C3 +IDF | 0.519 | 0.527 | 0.943 | 0.515 | 0.547 | 0.364 | 0.353 | 0.379 | 0.880 |
| ablation: no taxonomy | 0.523 | 0.530 | 0.948 | 0.528 | 0.557 | 0.377 | 0.367 | 0.390 | 0.897 |
| ablation: no systematicity (beta=0) | 0.519 | 0.525 | 0.935 | 0.513 | 0.549 | 0.360 | 0.341 | 0.386 | 0.874 |
| ablation: no parent-child | 0.507 | 0.513 | 0.949 | 0.489 | 0.510 | 0.324 | 0.306 | 0.348 | 0.640 |
| ablation: no co-entity | 0.521 | 0.531 | 0.901 | 0.534 | 0.575 | 0.365 | 0.336 | 0.404 | 0.903 |
| ablation: co-entity weight 1.0 | 0.516 | 0.522 | 0.953 | 0.506 | 0.541 | 0.368 | 0.355 | 0.386 | 0.851 |
| ablation: co-entity weight 0.5 | 0.520 | 0.525 | 0.940 | 0.518 | 0.547 | 0.369 | 0.350 | 0.395 | 0.897 |
| ablation: no co-expr | 0.522 | 0.529 | 0.928 | 0.523 | 0.557 | 0.370 | 0.355 | 0.390 | 0.914 |
| ablation: WL depth 1 | 0.524 | 0.531 | 0.927 | 0.530 | 0.560 | 0.374 | 0.364 | 0.388 | 0.903 |
| ablation: WL depth 3 | 0.520 | 0.527 | 0.926 | 0.519 | 0.553 | 0.363 | 0.344 | 0.388 | 0.897 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.308 | 0.350 |
| star | 0.000 | 0.392 | 0.364 |
| tree | 0.000 | 0.357 | 0.343 |
| loop | 0.014 | 0.336 | 0.385 |
| deep-ho (test) | 0.000 | 0.273 | 0.329 |
| comparison (test) | 0.042 | 0.441 | 0.455 |
| mixed (test) | 0.000 | 0.458 | 0.458 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.196 ± 0.053 | 0.096 ± 0.034 | 0.366 ± 0.032 | 0.394 ± 0.066 |
| TA | 0.500 ± 0.016 | 0.330 ± 0.069 | 0.440 ± 0.082 | 0.484 ± 0.023 | 0.469 ± 0.026 |
| MA | 0.108 ± 0.053 | 0.328 ± 0.070 | 0.455 ± 0.065 | 0.488 ± 0.017 | 0.470 ± 0.024 |
| FOR | 0.501 ± 0.015 | 0.332 ± 0.068 | 0.457 ± 0.062 | 0.489 ± 0.017 | 0.471 ± 0.024 |
| RND | 0.496 ± 0.023 | 0.450 ± 0.041 | 0.495 ± 0.016 | 0.499 ± 0.011 | 0.481 ± 0.023 |
