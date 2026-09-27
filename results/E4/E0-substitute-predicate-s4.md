# E0 — fingerprint separability (substitute-predicate-s4)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=153.7, runtime=1.6s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 177/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.504 | 0.873 | 0.000 | 0.508 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.032 | 0.502 | 0.904 | 0.000 | 0.494 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.460 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.488 | 0.504 | 0.955 | 0.455 | 0.516 | 0.308 | 0.281 | 0.345 | 0.427 |
| exact C2 relational | 0.695 | 0.703 | 0.978 | 0.784 | 0.820 | 0.721 | 0.634 | 0.836 | 0.918 |
| exact C3 WL | 0.701 | 0.711 | 0.965 | 0.810 | 0.838 | 0.716 | 0.650 | 0.804 | 0.949 |
| exact C4 topology | 0.612 | 0.611 | 0.975 | 0.639 | 0.629 | 0.500 | 0.420 | 0.609 | 0.562 |
| exact analogy profile | 0.691 | 0.701 | 0.994 | 0.794 | 0.828 | 0.730 | 0.656 | 0.829 | 0.944 |
| exact analogy profile +IDF | 0.683 | 0.697 | 0.989 | 0.781 | 0.810 | 0.699 | 0.636 | 0.783 | 0.955 |
| exact literal profile +IDF | 0.000 | 0.662 | 0.976 | 0.000 | 0.762 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.662 | 0.678 | 0.955 | 0.731 | 0.758 | 0.633 | 0.573 | 0.713 | 0.932 |
| fingerprint analogy +IDF D=2048 | 0.668 | 0.682 | 0.970 | 0.737 | 0.769 | 0.642 | 0.589 | 0.713 | 0.921 |
| fingerprint analogy +IDF D=4096 | 0.674 | 0.688 | 0.980 | 0.755 | 0.779 | 0.678 | 0.612 | 0.768 | 0.944 |
| fingerprint analogy +IDF D=8192 | 0.674 | 0.690 | 0.982 | 0.754 | 0.794 | 0.680 | 0.617 | 0.764 | 0.949 |
| fingerprint analogy +IDF D=16384 | 0.673 | 0.690 | 0.987 | 0.753 | 0.789 | 0.668 | 0.603 | 0.755 | 0.949 |
| fingerprint C1 only D=8192 | 0.486 | 0.504 | 0.939 | 0.471 | 0.514 | 0.310 | 0.299 | 0.326 | 0.480 |
| fingerprint C2 only D=8192 | 0.687 | 0.701 | 0.968 | 0.762 | 0.797 | 0.667 | 0.591 | 0.770 | 0.918 |
| fingerprint C3 only D=8192 | 0.690 | 0.701 | 0.868 | 0.726 | 0.748 | 0.569 | 0.516 | 0.639 | 0.955 |
| fingerprint C4 only D=8192 | 0.616 | 0.615 | 0.933 | 0.642 | 0.638 | 0.499 | 0.408 | 0.621 | 0.568 |
| exact mix C2 only +IDF | 0.692 | 0.705 | 0.977 | 0.761 | 0.809 | 0.677 | 0.610 | 0.766 | 0.915 |
| exact mix C3 only +IDF | 0.731 | 0.746 | 0.972 | 0.820 | 0.839 | 0.728 | 0.677 | 0.797 | 0.966 |
| exact mix C4 only +IDF | 0.620 | 0.626 | 0.970 | 0.676 | 0.688 | 0.542 | 0.476 | 0.631 | 0.605 |
| exact mix C2+C3 +IDF | 0.703 | 0.716 | 0.985 | 0.791 | 0.834 | 0.714 | 0.661 | 0.785 | 0.949 |
| exact mix C2+C3+C4 +IDF | 0.706 | 0.719 | 0.992 | 0.801 | 0.833 | 0.722 | 0.656 | 0.811 | 0.949 |
| exact mix C1+C2+C3 +IDF | 0.659 | 0.674 | 0.983 | 0.739 | 0.776 | 0.658 | 0.601 | 0.734 | 0.944 |
| ablation: no taxonomy | 0.677 | 0.693 | 0.987 | 0.758 | 0.778 | 0.669 | 0.607 | 0.752 | 0.955 |
| ablation: no systematicity (beta=0) | 0.663 | 0.677 | 0.990 | 0.770 | 0.801 | 0.686 | 0.626 | 0.766 | 0.955 |
| ablation: no parent-child | 0.547 | 0.555 | 0.984 | 0.649 | 0.683 | 0.514 | 0.493 | 0.542 | 0.887 |
| ablation: no co-entity | 0.690 | 0.704 | 0.989 | 0.785 | 0.814 | 0.703 | 0.642 | 0.785 | 0.955 |
| ablation: co-entity weight 1.0 | 0.637 | 0.652 | 0.989 | 0.741 | 0.761 | 0.627 | 0.584 | 0.685 | 0.938 |
| ablation: co-entity weight 0.5 | 0.667 | 0.681 | 0.990 | 0.769 | 0.796 | 0.683 | 0.621 | 0.766 | 0.949 |
| ablation: no co-expr | 0.685 | 0.701 | 0.986 | 0.781 | 0.808 | 0.697 | 0.633 | 0.783 | 0.955 |
| ablation: WL depth 1 | 0.683 | 0.698 | 0.990 | 0.783 | 0.812 | 0.700 | 0.636 | 0.785 | 0.955 |
| ablation: WL depth 3 | 0.680 | 0.695 | 0.989 | 0.779 | 0.805 | 0.697 | 0.635 | 0.780 | 0.955 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.678 | 0.671 |
| star | 0.000 | 0.622 | 0.615 |
| tree | 0.000 | 0.755 | 0.727 |
| loop | 0.000 | 0.490 | 0.455 |
| deep-ho (test) | 0.000 | 0.643 | 0.608 |
| comparison (test) | 0.000 | 0.923 | 0.923 |
| mixed (test) | 0.000 | 0.782 | 0.761 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.202 ± 0.054 | 0.100 ± 0.035 | 0.370 ± 0.031 | 0.402 ± 0.063 |
| TA | 0.500 ± 0.016 | 0.320 ± 0.088 | 0.345 ± 0.087 | 0.469 ± 0.026 | 0.396 ± 0.079 |
| MA | 0.107 ± 0.054 | 0.316 ± 0.089 | 0.402 ± 0.075 | 0.485 ± 0.019 | 0.420 ± 0.062 |
| FOR | 0.500 ± 0.016 | 0.321 ± 0.087 | 0.406 ± 0.072 | 0.486 ± 0.019 | 0.420 ± 0.058 |
| RND | 0.497 ± 0.022 | 0.455 ± 0.039 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.482 ± 0.022 |
