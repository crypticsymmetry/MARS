# E0 — fingerprint separability (all-s4)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=155.8, runtime=1.5s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 316/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.507 | 0.924 | 0.000 | 0.518 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.020 | 0.503 | 0.916 | 0.005 | 0.520 | 0.004 | 0.005 | 0.002 | 0.009 |
| exact C0 surface | 0.000 | 0.500 | 0.463 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.509 | 0.508 | 0.962 | 0.500 | 0.527 | 0.343 | 0.345 | 0.340 | 0.457 |
| exact C2 relational | 0.719 | 0.707 | 0.918 | 0.740 | 0.739 | 0.650 | 0.642 | 0.661 | 0.829 |
| exact C3 WL | 0.737 | 0.727 | 0.946 | 0.782 | 0.774 | 0.684 | 0.668 | 0.706 | 0.848 |
| exact C4 topology | 0.554 | 0.542 | 0.858 | 0.561 | 0.542 | 0.374 | 0.300 | 0.474 | 0.462 |
| exact analogy profile | 0.715 | 0.703 | 0.964 | 0.753 | 0.740 | 0.665 | 0.654 | 0.680 | 0.854 |
| exact analogy profile +IDF | 0.726 | 0.714 | 0.980 | 0.757 | 0.759 | 0.678 | 0.675 | 0.682 | 0.873 |
| exact literal profile +IDF | 0.002 | 0.692 | 0.977 | 0.000 | 0.743 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.723 | 0.706 | 0.946 | 0.749 | 0.740 | 0.649 | 0.642 | 0.659 | 0.829 |
| fingerprint analogy +IDF D=2048 | 0.721 | 0.709 | 0.950 | 0.750 | 0.747 | 0.665 | 0.661 | 0.671 | 0.858 |
| fingerprint analogy +IDF D=4096 | 0.724 | 0.711 | 0.958 | 0.758 | 0.752 | 0.675 | 0.675 | 0.675 | 0.861 |
| fingerprint analogy +IDF D=8192 | 0.723 | 0.709 | 0.968 | 0.754 | 0.749 | 0.672 | 0.671 | 0.673 | 0.867 |
| fingerprint analogy +IDF D=16384 | 0.722 | 0.708 | 0.973 | 0.755 | 0.750 | 0.675 | 0.675 | 0.675 | 0.867 |
| fingerprint C1 only D=8192 | 0.512 | 0.509 | 0.981 | 0.502 | 0.513 | 0.335 | 0.319 | 0.355 | 0.454 |
| fingerprint C2 only D=8192 | 0.734 | 0.719 | 0.928 | 0.757 | 0.747 | 0.663 | 0.654 | 0.675 | 0.845 |
| fingerprint C3 only D=8192 | 0.751 | 0.737 | 0.890 | 0.770 | 0.759 | 0.667 | 0.675 | 0.657 | 0.867 |
| fingerprint C4 only D=8192 | 0.544 | 0.536 | 0.783 | 0.561 | 0.552 | 0.342 | 0.267 | 0.442 | 0.392 |
| exact mix C2 only +IDF | 0.731 | 0.718 | 0.933 | 0.757 | 0.745 | 0.664 | 0.657 | 0.673 | 0.848 |
| exact mix C3 only +IDF | 0.770 | 0.760 | 0.949 | 0.792 | 0.788 | 0.715 | 0.710 | 0.722 | 0.896 |
| exact mix C4 only +IDF | 0.540 | 0.525 | 0.858 | 0.558 | 0.534 | 0.358 | 0.296 | 0.442 | 0.415 |
| exact mix C2+C3 +IDF | 0.744 | 0.731 | 0.953 | 0.767 | 0.766 | 0.689 | 0.689 | 0.689 | 0.889 |
| exact mix C2+C3+C4 +IDF | 0.734 | 0.721 | 0.960 | 0.763 | 0.760 | 0.685 | 0.684 | 0.687 | 0.870 |
| exact mix C1+C2+C3 +IDF | 0.718 | 0.707 | 0.983 | 0.749 | 0.750 | 0.669 | 0.671 | 0.666 | 0.877 |
| ablation: no taxonomy | 0.729 | 0.717 | 0.983 | 0.760 | 0.759 | 0.679 | 0.685 | 0.671 | 0.877 |
| ablation: no systematicity (beta=0) | 0.708 | 0.697 | 0.981 | 0.746 | 0.744 | 0.662 | 0.657 | 0.668 | 0.854 |
| ablation: no parent-child | 0.579 | 0.579 | 0.980 | 0.628 | 0.634 | 0.502 | 0.463 | 0.554 | 0.693 |
| ablation: no co-entity | 0.730 | 0.718 | 0.978 | 0.756 | 0.756 | 0.681 | 0.678 | 0.685 | 0.873 |
| ablation: co-entity weight 1.0 | 0.685 | 0.678 | 0.984 | 0.734 | 0.730 | 0.639 | 0.638 | 0.640 | 0.867 |
| ablation: co-entity weight 0.5 | 0.714 | 0.703 | 0.982 | 0.751 | 0.752 | 0.671 | 0.671 | 0.671 | 0.877 |
| ablation: no co-expr | 0.734 | 0.722 | 0.980 | 0.768 | 0.769 | 0.691 | 0.692 | 0.689 | 0.889 |
| ablation: WL depth 1 | 0.731 | 0.719 | 0.980 | 0.766 | 0.763 | 0.691 | 0.694 | 0.687 | 0.889 |
| ablation: WL depth 3 | 0.724 | 0.712 | 0.980 | 0.755 | 0.756 | 0.677 | 0.673 | 0.682 | 0.867 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.720 | 0.699 |
| star | 0.000 | 0.734 | 0.727 |
| tree | 0.000 | 0.734 | 0.748 |
| loop | 0.021 | 0.510 | 0.510 |
| deep-ho (test) | 0.000 | 0.538 | 0.545 |
| comparison (test) | 0.007 | 0.727 | 0.720 |
| mixed (test) | 0.000 | 0.782 | 0.754 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.201 ± 0.054 | 0.100 ± 0.035 | 0.369 ± 0.032 | 0.403 ± 0.062 |
| TA | 0.501 ± 0.016 | 0.279 ± 0.077 | 0.310 ± 0.115 | 0.456 ± 0.035 | 0.453 ± 0.036 |
| MA | 0.136 ± 0.049 | 0.283 ± 0.079 | 0.403 ± 0.085 | 0.484 ± 0.019 | 0.459 ± 0.033 |
| FOR | 0.500 ± 0.015 | 0.281 ± 0.078 | 0.397 ± 0.086 | 0.482 ± 0.020 | 0.458 ± 0.031 |
| RND | 0.496 ± 0.022 | 0.454 ± 0.039 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.022 |
