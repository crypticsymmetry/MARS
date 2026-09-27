# E0 — fingerprint separability (delete-fact-s1)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=144.1, runtime=1.4s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 694/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.501 | 0.939 | 0.000 | 0.515 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.010 | 0.501 | 0.931 | 0.000 | 0.504 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.462 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.502 | 0.504 | 0.979 | 0.506 | 0.512 | 0.329 | 0.330 | 0.328 | 0.378 |
| exact C2 relational | 0.704 | 0.708 | 0.917 | 0.757 | 0.763 | 0.646 | 0.668 | 0.618 | 0.860 |
| exact C3 WL | 0.735 | 0.739 | 0.956 | 0.835 | 0.848 | 0.757 | 0.792 | 0.710 | 0.921 |
| exact C4 topology | 0.567 | 0.561 | 0.862 | 0.568 | 0.584 | 0.377 | 0.312 | 0.463 | 0.450 |
| exact analogy profile | 0.707 | 0.711 | 0.967 | 0.802 | 0.798 | 0.716 | 0.736 | 0.689 | 0.902 |
| exact analogy profile +IDF | 0.721 | 0.724 | 0.991 | 0.806 | 0.796 | 0.721 | 0.750 | 0.682 | 0.919 |
| exact literal profile +IDF | 0.000 | 0.706 | 0.987 | 0.000 | 0.790 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.715 | 0.717 | 0.950 | 0.774 | 0.775 | 0.675 | 0.708 | 0.631 | 0.873 |
| fingerprint analogy +IDF D=2048 | 0.716 | 0.719 | 0.963 | 0.790 | 0.786 | 0.694 | 0.720 | 0.659 | 0.888 |
| fingerprint analogy +IDF D=4096 | 0.716 | 0.719 | 0.973 | 0.780 | 0.791 | 0.701 | 0.734 | 0.657 | 0.895 |
| fingerprint analogy +IDF D=8192 | 0.714 | 0.719 | 0.982 | 0.789 | 0.789 | 0.704 | 0.726 | 0.673 | 0.896 |
| fingerprint analogy +IDF D=16384 | 0.715 | 0.719 | 0.987 | 0.794 | 0.804 | 0.720 | 0.740 | 0.694 | 0.915 |
| fingerprint C1 only D=8192 | 0.499 | 0.500 | 0.994 | 0.510 | 0.508 | 0.335 | 0.328 | 0.343 | 0.359 |
| fingerprint C2 only D=8192 | 0.717 | 0.722 | 0.944 | 0.768 | 0.779 | 0.684 | 0.704 | 0.658 | 0.872 |
| fingerprint C3 only D=8192 | 0.758 | 0.759 | 0.919 | 0.810 | 0.802 | 0.710 | 0.737 | 0.674 | 0.898 |
| fingerprint C4 only D=8192 | 0.559 | 0.563 | 0.802 | 0.595 | 0.612 | 0.382 | 0.337 | 0.443 | 0.439 |
| exact mix C2 only +IDF | 0.721 | 0.725 | 0.960 | 0.770 | 0.766 | 0.684 | 0.703 | 0.659 | 0.872 |
| exact mix C3 only +IDF | 0.771 | 0.774 | 0.957 | 0.837 | 0.846 | 0.773 | 0.822 | 0.708 | 0.955 |
| exact mix C4 only +IDF | 0.559 | 0.559 | 0.858 | 0.610 | 0.601 | 0.403 | 0.340 | 0.487 | 0.465 |
| exact mix C2+C3 +IDF | 0.737 | 0.741 | 0.974 | 0.814 | 0.811 | 0.735 | 0.760 | 0.701 | 0.929 |
| exact mix C2+C3+C4 +IDF | 0.727 | 0.729 | 0.971 | 0.809 | 0.801 | 0.722 | 0.753 | 0.680 | 0.922 |
| exact mix C1+C2+C3 +IDF | 0.715 | 0.719 | 0.994 | 0.794 | 0.795 | 0.715 | 0.736 | 0.687 | 0.914 |
| ablation: no taxonomy | 0.727 | 0.730 | 0.993 | 0.808 | 0.800 | 0.728 | 0.755 | 0.692 | 0.925 |
| ablation: no systematicity (beta=0) | 0.709 | 0.712 | 0.993 | 0.796 | 0.790 | 0.706 | 0.740 | 0.661 | 0.905 |
| ablation: no parent-child | 0.592 | 0.588 | 0.994 | 0.693 | 0.686 | 0.547 | 0.556 | 0.535 | 0.666 |
| ablation: no co-entity | 0.717 | 0.720 | 0.983 | 0.801 | 0.807 | 0.723 | 0.750 | 0.687 | 0.925 |
| ablation: co-entity weight 1.0 | 0.690 | 0.693 | 0.997 | 0.779 | 0.783 | 0.689 | 0.713 | 0.657 | 0.870 |
| ablation: co-entity weight 0.5 | 0.714 | 0.717 | 0.994 | 0.799 | 0.790 | 0.710 | 0.741 | 0.668 | 0.909 |
| ablation: no co-expr | 0.727 | 0.731 | 0.991 | 0.810 | 0.797 | 0.724 | 0.753 | 0.685 | 0.924 |
| ablation: WL depth 1 | 0.726 | 0.730 | 0.991 | 0.805 | 0.803 | 0.728 | 0.755 | 0.692 | 0.925 |
| ablation: WL depth 3 | 0.718 | 0.721 | 0.991 | 0.802 | 0.793 | 0.716 | 0.743 | 0.680 | 0.915 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.811 | 0.811 |
| star | 0.000 | 0.699 | 0.696 |
| tree | 0.000 | 0.748 | 0.741 |
| loop | 0.000 | 0.741 | 0.657 |
| deep-ho (test) | 0.000 | 0.357 | 0.357 |
| comparison (test) | 0.000 | 0.881 | 0.860 |
| mixed (test) | 0.000 | 0.810 | 0.803 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.200 ± 0.053 | 0.098 ± 0.035 | 0.369 ± 0.032 | 0.400 ± 0.064 |
| TA | 0.500 ± 0.015 | 0.255 ± 0.071 | 0.286 ± 0.130 | 0.441 ± 0.040 | 0.446 ± 0.040 |
| MA | 0.108 ± 0.054 | 0.255 ± 0.070 | 0.389 ± 0.098 | 0.476 ± 0.023 | 0.456 ± 0.032 |
| FOR | 0.500 ± 0.016 | 0.256 ± 0.069 | 0.392 ± 0.099 | 0.476 ± 0.024 | 0.456 ± 0.033 |
| RND | 0.496 ± 0.023 | 0.452 ± 0.040 | 0.495 ± 0.014 | 0.499 ± 0.012 | 0.483 ± 0.022 |
