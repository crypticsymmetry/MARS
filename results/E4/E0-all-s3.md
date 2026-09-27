# E0 — fingerprint separability (all-s3)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=155.4, runtime=1.6s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 410/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.499 | 0.936 | 0.000 | 0.504 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.016 | 0.497 | 0.937 | 0.003 | 0.486 | 0.002 | 0.003 | 0.000 | 0.002 |
| exact C0 surface | 0.000 | 0.500 | 0.463 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.506 | 0.497 | 0.975 | 0.521 | 0.495 | 0.352 | 0.337 | 0.374 | 0.424 |
| exact C2 relational | 0.761 | 0.754 | 0.939 | 0.788 | 0.769 | 0.715 | 0.692 | 0.745 | 0.829 |
| exact C3 WL | 0.766 | 0.762 | 0.968 | 0.823 | 0.816 | 0.728 | 0.715 | 0.745 | 0.839 |
| exact C4 topology | 0.564 | 0.542 | 0.891 | 0.578 | 0.554 | 0.389 | 0.288 | 0.523 | 0.422 |
| exact analogy profile | 0.755 | 0.749 | 0.976 | 0.795 | 0.784 | 0.723 | 0.705 | 0.748 | 0.846 |
| exact analogy profile +IDF | 0.770 | 0.763 | 0.987 | 0.796 | 0.794 | 0.737 | 0.729 | 0.748 | 0.863 |
| exact literal profile +IDF | 0.001 | 0.738 | 0.985 | 0.001 | 0.780 | 0.001 | 0.000 | 0.002 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.764 | 0.755 | 0.965 | 0.793 | 0.786 | 0.716 | 0.708 | 0.727 | 0.839 |
| fingerprint analogy +IDF D=2048 | 0.764 | 0.757 | 0.971 | 0.793 | 0.786 | 0.718 | 0.703 | 0.738 | 0.837 |
| fingerprint analogy +IDF D=4096 | 0.767 | 0.759 | 0.977 | 0.798 | 0.785 | 0.725 | 0.710 | 0.745 | 0.844 |
| fingerprint analogy +IDF D=8192 | 0.766 | 0.759 | 0.981 | 0.799 | 0.787 | 0.724 | 0.712 | 0.741 | 0.846 |
| fingerprint analogy +IDF D=16384 | 0.765 | 0.758 | 0.985 | 0.791 | 0.784 | 0.723 | 0.717 | 0.731 | 0.849 |
| fingerprint C1 only D=8192 | 0.507 | 0.496 | 0.987 | 0.514 | 0.495 | 0.335 | 0.315 | 0.362 | 0.407 |
| fingerprint C2 only D=8192 | 0.774 | 0.768 | 0.950 | 0.799 | 0.779 | 0.729 | 0.714 | 0.749 | 0.839 |
| fingerprint C3 only D=8192 | 0.795 | 0.792 | 0.930 | 0.813 | 0.806 | 0.713 | 0.707 | 0.720 | 0.856 |
| fingerprint C4 only D=8192 | 0.557 | 0.538 | 0.812 | 0.571 | 0.558 | 0.377 | 0.313 | 0.463 | 0.427 |
| exact mix C2 only +IDF | 0.774 | 0.767 | 0.953 | 0.801 | 0.783 | 0.727 | 0.715 | 0.743 | 0.841 |
| exact mix C3 only +IDF | 0.809 | 0.806 | 0.971 | 0.835 | 0.824 | 0.754 | 0.752 | 0.757 | 0.893 |
| exact mix C4 only +IDF | 0.554 | 0.537 | 0.886 | 0.573 | 0.564 | 0.386 | 0.288 | 0.516 | 0.432 |
| exact mix C2+C3 +IDF | 0.788 | 0.782 | 0.970 | 0.806 | 0.798 | 0.745 | 0.743 | 0.748 | 0.868 |
| exact mix C2+C3+C4 +IDF | 0.778 | 0.770 | 0.974 | 0.809 | 0.802 | 0.745 | 0.738 | 0.755 | 0.868 |
| exact mix C1+C2+C3 +IDF | 0.762 | 0.755 | 0.989 | 0.783 | 0.788 | 0.720 | 0.710 | 0.734 | 0.854 |
| ablation: no taxonomy | 0.773 | 0.766 | 0.989 | 0.796 | 0.795 | 0.733 | 0.733 | 0.734 | 0.863 |
| ablation: no systematicity (beta=0) | 0.750 | 0.743 | 0.989 | 0.790 | 0.780 | 0.722 | 0.708 | 0.741 | 0.856 |
| ablation: no parent-child | 0.595 | 0.590 | 0.988 | 0.673 | 0.641 | 0.532 | 0.479 | 0.603 | 0.646 |
| ablation: no co-entity | 0.774 | 0.767 | 0.985 | 0.803 | 0.792 | 0.740 | 0.731 | 0.752 | 0.866 |
| ablation: co-entity weight 1.0 | 0.725 | 0.716 | 0.991 | 0.780 | 0.764 | 0.696 | 0.678 | 0.720 | 0.854 |
| ablation: co-entity weight 0.5 | 0.757 | 0.749 | 0.989 | 0.793 | 0.791 | 0.728 | 0.717 | 0.743 | 0.863 |
| ablation: no co-expr | 0.778 | 0.770 | 0.988 | 0.799 | 0.802 | 0.742 | 0.740 | 0.745 | 0.873 |
| ablation: WL depth 1 | 0.775 | 0.767 | 0.988 | 0.797 | 0.803 | 0.740 | 0.745 | 0.734 | 0.871 |
| ablation: WL depth 3 | 0.768 | 0.760 | 0.987 | 0.798 | 0.791 | 0.735 | 0.726 | 0.748 | 0.863 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.818 | 0.818 |
| star | 0.000 | 0.762 | 0.727 |
| tree | 0.000 | 0.776 | 0.776 |
| loop | 0.014 | 0.559 | 0.524 |
| deep-ho (test) | 0.000 | 0.657 | 0.650 |
| comparison (test) | 0.000 | 0.797 | 0.783 |
| mixed (test) | 0.000 | 0.789 | 0.789 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.202 ± 0.054 | 0.100 ± 0.035 | 0.370 ± 0.032 | 0.403 ± 0.062 |
| TA | 0.501 ± 0.016 | 0.266 ± 0.073 | 0.279 ± 0.114 | 0.446 ± 0.036 | 0.447 ± 0.040 |
| MA | 0.129 ± 0.050 | 0.267 ± 0.076 | 0.392 ± 0.089 | 0.481 ± 0.020 | 0.455 ± 0.034 |
| FOR | 0.501 ± 0.015 | 0.265 ± 0.074 | 0.389 ± 0.093 | 0.481 ± 0.021 | 0.453 ± 0.034 |
| RND | 0.497 ± 0.022 | 0.454 ± 0.039 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.484 ± 0.021 |
