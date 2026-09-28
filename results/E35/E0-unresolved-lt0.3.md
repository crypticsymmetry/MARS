# E0 — fingerprint separability (lt0.3)

Config: groups=300, seed=1, naming=Unresolved, distractors=2, cases=2400, mean features/case=122.0, runtime=3.7s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 300/300.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.496 | 0.794 | 0.000 | 0.410 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.000 | 0.498 | 0.809 | 0.000 | 0.472 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.456 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.503 | 0.508 | 0.997 | 0.503 | 0.507 | 0.498 | 0.497 | 0.500 | 0.498 |
| exact C2 relational | 0.577 | 0.575 | 0.998 | 0.552 | 0.572 | 0.415 | 0.297 | 0.574 | 0.415 |
| exact C3 WL | 0.613 | 0.622 | 0.989 | 0.640 | 0.625 | 0.503 | 0.430 | 0.602 | 0.503 |
| exact C4 topology | 0.608 | 0.617 | 0.975 | 0.638 | 0.603 | 0.482 | 0.398 | 0.594 | 0.482 |
| exact analogy profile | 0.618 | 0.626 | 0.997 | 0.648 | 0.622 | 0.508 | 0.439 | 0.602 | 0.508 |
| exact analogy profile +IDF | 0.635 | 0.643 | 0.999 | 0.660 | 0.702 | 0.563 | 0.477 | 0.680 | 0.563 |
| exact literal profile +IDF | 0.000 | 0.643 | 0.999 | 0.000 | 0.702 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.633 | 0.622 | 0.999 | 0.658 | 0.658 | 0.517 | 0.401 | 0.672 | 0.517 |
| fingerprint analogy +IDF D=2048 | 0.636 | 0.628 | 0.999 | 0.683 | 0.672 | 0.543 | 0.459 | 0.656 | 0.543 |
| fingerprint analogy +IDF D=4096 | 0.630 | 0.628 | 1.000 | 0.657 | 0.638 | 0.527 | 0.465 | 0.609 | 0.527 |
| fingerprint analogy +IDF D=8192 | 0.632 | 0.632 | 0.999 | 0.673 | 0.662 | 0.547 | 0.477 | 0.641 | 0.547 |
| fingerprint analogy +IDF D=16384 | 0.632 | 0.632 | 0.999 | 0.663 | 0.665 | 0.547 | 0.459 | 0.664 | 0.547 |
| fingerprint C1 only D=8192 | 0.503 | 0.505 | 0.991 | 0.503 | 0.505 | 0.498 | 0.497 | 0.500 | 0.498 |
| fingerprint C2 only D=8192 | 0.549 | 0.545 | 0.999 | 0.587 | 0.567 | 0.425 | 0.331 | 0.551 | 0.425 |
| fingerprint C3 only D=8192 | 0.621 | 0.620 | 0.995 | 0.675 | 0.672 | 0.577 | 0.529 | 0.641 | 0.577 |
| fingerprint C4 only D=8192 | 0.614 | 0.627 | 0.956 | 0.603 | 0.682 | 0.510 | 0.410 | 0.645 | 0.510 |
| exact mix C2 only +IDF | 0.548 | 0.544 | 0.999 | 0.553 | 0.568 | 0.415 | 0.297 | 0.574 | 0.415 |
| exact mix C3 only +IDF | 0.621 | 0.627 | 0.997 | 0.673 | 0.697 | 0.560 | 0.500 | 0.641 | 0.560 |
| exact mix C4 only +IDF | 0.601 | 0.609 | 0.973 | 0.633 | 0.668 | 0.507 | 0.407 | 0.641 | 0.507 |
| exact mix C2+C3 +IDF | 0.633 | 0.640 | 0.999 | 0.667 | 0.700 | 0.563 | 0.488 | 0.664 | 0.563 |
| exact mix C2+C3+C4 +IDF | 0.635 | 0.644 | 0.999 | 0.660 | 0.692 | 0.560 | 0.471 | 0.680 | 0.560 |
| exact mix C1+C2+C3 +IDF | 0.633 | 0.640 | 0.999 | 0.667 | 0.700 | 0.563 | 0.488 | 0.664 | 0.563 |
| ablation: no taxonomy | 0.634 | 0.642 | 0.999 | 0.663 | 0.715 | 0.567 | 0.494 | 0.664 | 0.567 |
| ablation: no systematicity (beta=0) | 0.630 | 0.638 | 0.999 | 0.657 | 0.702 | 0.553 | 0.477 | 0.656 | 0.553 |
| ablation: no parent-child | 0.611 | 0.619 | 0.997 | 0.637 | 0.698 | 0.533 | 0.483 | 0.602 | 0.533 |
| ablation: no co-entity | 0.637 | 0.645 | 0.999 | 0.667 | 0.708 | 0.577 | 0.500 | 0.680 | 0.577 |
| ablation: co-entity weight 1.0 | 0.625 | 0.631 | 0.998 | 0.647 | 0.688 | 0.533 | 0.453 | 0.641 | 0.533 |
| ablation: co-entity weight 0.5 | 0.632 | 0.638 | 0.999 | 0.660 | 0.698 | 0.557 | 0.477 | 0.664 | 0.557 |
| ablation: no co-expr | 0.635 | 0.643 | 0.999 | 0.657 | 0.702 | 0.563 | 0.477 | 0.680 | 0.563 |
| ablation: WL depth 1 | 0.554 | 0.565 | 0.998 | 0.573 | 0.605 | 0.437 | 0.349 | 0.555 | 0.437 |
| ablation: WL depth 3 | 0.639 | 0.649 | 0.999 | 0.660 | 0.708 | 0.570 | 0.483 | 0.688 | 0.570 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.512 | 0.512 |
| star | 0.000 | 0.349 | 0.372 |
| tree | 0.000 | 0.698 | 0.651 |
| loop | 0.000 | 0.349 | 0.372 |
| deep-ho (test) | 0.000 | 0.512 | 0.442 |
| comparison (test) | 0.000 | 0.977 | 0.977 |
| mixed (test) | 0.000 | 0.548 | 0.500 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.175 ± 0.032 | 0.000 ± 0.004 | 0.024 ± 0.020 | 0.339 ± 0.063 | 0.400 ± 0.057 |
| TA | 0.498 ± 0.016 | 0.000 ± 0.005 | 0.023 ± 0.019 | 0.338 ± 0.072 | 0.395 ± 0.068 |
| MA | 0.177 ± 0.031 | 0.000 ± 0.003 | 0.051 ± 0.077 | 0.363 ± 0.068 | 0.417 ± 0.057 |
| FOR | 0.499 ± 0.016 | 0.001 ± 0.004 | 0.051 ± 0.078 | 0.364 ± 0.064 | 0.420 ± 0.054 |
| RND | 0.497 ± 0.022 | 0.229 ± 0.125 | 0.434 ± 0.073 | 0.479 ± 0.020 | 0.482 ± 0.021 |
