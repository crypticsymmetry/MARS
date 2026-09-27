# E0 — fingerprint separability (unresolved-d2)

Config: groups=1000, seed=1, naming=Unresolved, distractors=2, cases=8000, mean features/case=121.3, runtime=1.1s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) |
|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.499 | 0.802 | 0.000 | 0.486 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.000 | 0.500 | 0.815 | 0.000 | 0.495 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.459 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.500 | 0.501 | 0.997 | 0.502 | 0.501 | 0.496 | 0.497 | 0.496 |
| exact C2 relational | 0.572 | 0.574 | 0.999 | 0.534 | 0.555 | 0.400 | 0.297 | 0.537 |
| exact C3 WL | 0.600 | 0.602 | 0.989 | 0.615 | 0.602 | 0.476 | 0.381 | 0.603 |
| exact C4 topology | 0.599 | 0.601 | 0.972 | 0.618 | 0.588 | 0.475 | 0.384 | 0.598 |
| exact analogy profile | 0.608 | 0.610 | 0.997 | 0.616 | 0.607 | 0.484 | 0.393 | 0.605 |
| exact analogy profile +IDF | 0.631 | 0.639 | 0.999 | 0.666 | 0.707 | 0.569 | 0.502 | 0.659 |
| exact literal profile +IDF | 0.000 | 0.639 | 0.999 | 0.000 | 0.707 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.617 | 0.618 | 0.999 | 0.646 | 0.635 | 0.510 | 0.420 | 0.632 |
| fingerprint analogy +IDF D=2048 | 0.623 | 0.629 | 0.999 | 0.644 | 0.664 | 0.531 | 0.451 | 0.638 |
| fingerprint analogy +IDF D=4096 | 0.624 | 0.631 | 0.999 | 0.636 | 0.661 | 0.521 | 0.430 | 0.643 |
| fingerprint analogy +IDF D=8192 | 0.626 | 0.633 | 0.999 | 0.659 | 0.690 | 0.554 | 0.472 | 0.664 |
| fingerprint analogy +IDF D=16384 | 0.624 | 0.632 | 0.999 | 0.648 | 0.679 | 0.539 | 0.455 | 0.652 |
| fingerprint C1 only D=8192 | 0.502 | 0.500 | 0.990 | 0.502 | 0.500 | 0.497 | 0.497 | 0.496 |
| fingerprint C2 only D=8192 | 0.541 | 0.544 | 0.999 | 0.549 | 0.561 | 0.410 | 0.310 | 0.543 |
| fingerprint C3 only D=8192 | 0.613 | 0.624 | 0.995 | 0.666 | 0.687 | 0.551 | 0.509 | 0.609 |
| fingerprint C4 only D=8192 | 0.601 | 0.613 | 0.936 | 0.624 | 0.654 | 0.489 | 0.413 | 0.592 |
| exact mix C2 only +IDF | 0.542 | 0.544 | 0.999 | 0.538 | 0.552 | 0.401 | 0.302 | 0.533 |
| exact mix C3 only +IDF | 0.613 | 0.623 | 0.997 | 0.665 | 0.696 | 0.552 | 0.498 | 0.624 |
| exact mix C4 only +IDF | 0.599 | 0.608 | 0.970 | 0.655 | 0.680 | 0.524 | 0.439 | 0.638 |
| exact mix C2+C3 +IDF | 0.628 | 0.636 | 0.999 | 0.666 | 0.707 | 0.569 | 0.500 | 0.661 |
| exact mix C2+C3+C4 +IDF | 0.631 | 0.640 | 0.999 | 0.664 | 0.704 | 0.564 | 0.497 | 0.654 |
| exact mix C1+C2+C3 +IDF | 0.628 | 0.636 | 0.999 | 0.666 | 0.707 | 0.569 | 0.500 | 0.661 |
| ablation: no taxonomy | 0.630 | 0.638 | 0.999 | 0.674 | 0.712 | 0.576 | 0.512 | 0.661 |
| ablation: no systematicity (beta=0) | 0.624 | 0.633 | 0.999 | 0.661 | 0.703 | 0.562 | 0.500 | 0.645 |
| ablation: no parent-child | 0.600 | 0.607 | 0.997 | 0.651 | 0.684 | 0.532 | 0.493 | 0.584 |
| ablation: no co-entity | 0.633 | 0.642 | 0.999 | 0.669 | 0.704 | 0.571 | 0.503 | 0.661 |
| ablation: co-entity weight 1.0 | 0.617 | 0.626 | 0.999 | 0.664 | 0.689 | 0.542 | 0.476 | 0.631 |
| ablation: co-entity weight 0.5 | 0.625 | 0.634 | 0.999 | 0.661 | 0.697 | 0.561 | 0.497 | 0.647 |
| ablation: no co-expr | 0.631 | 0.639 | 0.999 | 0.665 | 0.707 | 0.569 | 0.500 | 0.661 |
| ablation: WL depth 1 | 0.551 | 0.558 | 0.998 | 0.582 | 0.590 | 0.445 | 0.364 | 0.554 |
| ablation: WL depth 3 | 0.637 | 0.646 | 0.999 | 0.670 | 0.716 | 0.576 | 0.510 | 0.664 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.608 | 0.559 |
| star | 0.000 | 0.371 | 0.364 |
| tree | 0.000 | 0.643 | 0.594 |
| loop | 0.000 | 0.385 | 0.371 |
| deep-ho (test) | 0.000 | 0.448 | 0.434 |
| comparison (test) | 0.000 | 0.951 | 0.993 |
| mixed (test) | 0.000 | 0.577 | 0.563 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.174 ± 0.031 | 0.000 ± 0.004 | 0.025 ± 0.022 | 0.340 ± 0.069 | 0.403 ± 0.063 |
| TA | 0.500 ± 0.016 | 0.001 ± 0.007 | 0.024 ± 0.019 | 0.339 ± 0.075 | 0.400 ± 0.071 |
| MA | 0.174 ± 0.032 | 0.001 ± 0.007 | 0.052 ± 0.079 | 0.364 ± 0.070 | 0.421 ± 0.057 |
| FOR | 0.500 ± 0.016 | 0.000 ± 0.004 | 0.052 ± 0.079 | 0.366 ± 0.067 | 0.423 ± 0.056 |
| RND | 0.497 ± 0.021 | 0.224 ± 0.122 | 0.439 ± 0.073 | 0.481 ± 0.019 | 0.483 ± 0.021 |
