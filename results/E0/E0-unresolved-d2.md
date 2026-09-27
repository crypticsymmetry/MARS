# E0 — fingerprint separability (unresolved-d2)

Config: groups=1000, seed=1, naming=Unresolved, distractors=2, cases=8000, mean features/case=121.3, runtime=1.2s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) |
|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.499 | 0.802 | 0.000 | 0.490 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.000 | 0.499 | 0.815 | 0.000 | 0.491 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.459 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.500 | 0.501 | 0.997 | 0.502 | 0.501 | 0.496 | 0.497 | 0.496 |
| exact C2 relational | 0.572 | 0.573 | 0.999 | 0.538 | 0.556 | 0.403 | 0.297 | 0.543 |
| exact C3 WL | 0.602 | 0.601 | 0.989 | 0.620 | 0.602 | 0.477 | 0.381 | 0.605 |
| exact C4 topology | 0.601 | 0.600 | 0.973 | 0.624 | 0.589 | 0.477 | 0.384 | 0.603 |
| exact analogy profile | 0.610 | 0.609 | 0.997 | 0.620 | 0.607 | 0.484 | 0.393 | 0.605 |
| exact analogy profile +IDF | 0.632 | 0.638 | 0.999 | 0.672 | 0.705 | 0.569 | 0.503 | 0.657 |
| exact literal profile +IDF | 0.000 | 0.638 | 0.999 | 0.000 | 0.705 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.619 | 0.616 | 0.999 | 0.653 | 0.632 | 0.515 | 0.427 | 0.633 |
| fingerprint analogy +IDF D=2048 | 0.624 | 0.627 | 0.999 | 0.644 | 0.664 | 0.528 | 0.448 | 0.636 |
| fingerprint analogy +IDF D=4096 | 0.626 | 0.629 | 0.999 | 0.637 | 0.661 | 0.524 | 0.435 | 0.644 |
| fingerprint analogy +IDF D=8192 | 0.627 | 0.631 | 0.999 | 0.667 | 0.681 | 0.557 | 0.479 | 0.661 |
| fingerprint analogy +IDF D=16384 | 0.625 | 0.630 | 0.999 | 0.659 | 0.675 | 0.543 | 0.462 | 0.652 |
| fingerprint C1 only D=8192 | 0.502 | 0.500 | 0.990 | 0.502 | 0.500 | 0.497 | 0.497 | 0.496 |
| fingerprint C2 only D=8192 | 0.540 | 0.541 | 0.999 | 0.547 | 0.552 | 0.406 | 0.307 | 0.539 |
| fingerprint C3 only D=8192 | 0.615 | 0.622 | 0.995 | 0.664 | 0.684 | 0.550 | 0.506 | 0.610 |
| fingerprint C4 only D=8192 | 0.603 | 0.612 | 0.936 | 0.623 | 0.648 | 0.491 | 0.406 | 0.604 |
| exact mix C2 only +IDF | 0.542 | 0.543 | 0.999 | 0.541 | 0.553 | 0.402 | 0.302 | 0.536 |
| exact mix C3 only +IDF | 0.615 | 0.621 | 0.997 | 0.668 | 0.695 | 0.551 | 0.498 | 0.621 |
| exact mix C4 only +IDF | 0.600 | 0.607 | 0.970 | 0.660 | 0.679 | 0.523 | 0.439 | 0.636 |
| exact mix C2+C3 +IDF | 0.629 | 0.635 | 0.999 | 0.670 | 0.705 | 0.568 | 0.500 | 0.659 |
| exact mix C2+C3+C4 +IDF | 0.633 | 0.639 | 0.999 | 0.666 | 0.704 | 0.564 | 0.498 | 0.652 |
| exact mix C1+C2+C3 +IDF | 0.629 | 0.635 | 0.999 | 0.670 | 0.705 | 0.568 | 0.500 | 0.659 |
| ablation: no taxonomy | 0.631 | 0.637 | 0.999 | 0.677 | 0.710 | 0.574 | 0.512 | 0.657 |
| ablation: no systematicity (beta=0) | 0.626 | 0.632 | 0.999 | 0.667 | 0.701 | 0.561 | 0.500 | 0.643 |
| ablation: no parent-child | 0.601 | 0.605 | 0.997 | 0.650 | 0.683 | 0.532 | 0.493 | 0.584 |
| ablation: no co-entity | 0.634 | 0.640 | 0.999 | 0.674 | 0.703 | 0.569 | 0.502 | 0.659 |
| ablation: co-entity weight 1.0 | 0.618 | 0.624 | 0.999 | 0.664 | 0.689 | 0.543 | 0.474 | 0.636 |
| ablation: co-entity weight 0.5 | 0.627 | 0.633 | 0.999 | 0.667 | 0.695 | 0.560 | 0.497 | 0.645 |
| ablation: no co-expr | 0.632 | 0.638 | 0.999 | 0.669 | 0.705 | 0.568 | 0.500 | 0.659 |
| ablation: WL depth 1 | 0.553 | 0.558 | 0.998 | 0.586 | 0.596 | 0.447 | 0.364 | 0.558 |
| ablation: WL depth 3 | 0.639 | 0.645 | 0.999 | 0.674 | 0.714 | 0.574 | 0.510 | 0.659 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.608 | 0.559 |
| star | 0.000 | 0.378 | 0.371 |
| tree | 0.000 | 0.643 | 0.601 |
| loop | 0.000 | 0.385 | 0.385 |
| deep-ho (test) | 0.000 | 0.441 | 0.427 |
| comparison (test) | 0.000 | 0.951 | 0.993 |
| mixed (test) | 0.000 | 0.577 | 0.563 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.175 ± 0.031 | 0.000 ± 0.004 | 0.024 ± 0.022 | 0.340 ± 0.070 | 0.402 ± 0.064 |
| TA | 0.500 ± 0.015 | 0.001 ± 0.007 | 0.024 ± 0.020 | 0.339 ± 0.075 | 0.400 ± 0.071 |
| MA | 0.174 ± 0.032 | 0.001 ± 0.007 | 0.052 ± 0.079 | 0.364 ± 0.070 | 0.421 ± 0.057 |
| FOR | 0.500 ± 0.016 | 0.000 ± 0.004 | 0.052 ± 0.079 | 0.365 ± 0.068 | 0.422 ± 0.057 |
| RND | 0.497 ± 0.021 | 0.224 ± 0.122 | 0.439 ± 0.073 | 0.481 ± 0.019 | 0.483 ± 0.021 |
