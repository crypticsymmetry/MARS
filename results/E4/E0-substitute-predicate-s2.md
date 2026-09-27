# E0 — fingerprint separability (substitute-predicate-s2)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=154.0, runtime=1.5s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 427/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.498 | 0.930 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.018 | 0.497 | 0.944 | 0.000 | 0.485 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.466 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.500 | 0.495 | 0.984 | 0.495 | 0.470 | 0.310 | 0.323 | 0.293 | 0.398 |
| exact C2 relational | 0.797 | 0.797 | 0.995 | 0.881 | 0.886 | 0.838 | 0.762 | 0.939 | 0.906 |
| exact C3 WL | 0.805 | 0.804 | 0.997 | 0.905 | 0.916 | 0.851 | 0.816 | 0.896 | 0.961 |
| exact C4 topology | 0.602 | 0.605 | 0.976 | 0.609 | 0.618 | 0.479 | 0.377 | 0.616 | 0.506 |
| exact analogy profile | 0.797 | 0.797 | 0.999 | 0.905 | 0.911 | 0.855 | 0.799 | 0.930 | 0.951 |
| exact analogy profile +IDF | 0.795 | 0.795 | 0.999 | 0.892 | 0.893 | 0.836 | 0.790 | 0.897 | 0.948 |
| exact literal profile +IDF | 0.000 | 0.755 | 0.997 | 0.000 | 0.869 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.778 | 0.771 | 0.991 | 0.861 | 0.849 | 0.775 | 0.724 | 0.843 | 0.909 |
| fingerprint analogy +IDF D=2048 | 0.783 | 0.779 | 0.995 | 0.866 | 0.880 | 0.805 | 0.750 | 0.879 | 0.916 |
| fingerprint analogy +IDF D=4096 | 0.787 | 0.784 | 0.996 | 0.873 | 0.882 | 0.811 | 0.753 | 0.888 | 0.937 |
| fingerprint analogy +IDF D=8192 | 0.787 | 0.786 | 0.997 | 0.885 | 0.885 | 0.827 | 0.785 | 0.883 | 0.946 |
| fingerprint analogy +IDF D=16384 | 0.786 | 0.786 | 0.998 | 0.882 | 0.884 | 0.823 | 0.776 | 0.886 | 0.939 |
| fingerprint C1 only D=8192 | 0.500 | 0.494 | 0.983 | 0.496 | 0.493 | 0.320 | 0.319 | 0.320 | 0.402 |
| fingerprint C2 only D=8192 | 0.798 | 0.798 | 0.992 | 0.868 | 0.874 | 0.810 | 0.746 | 0.896 | 0.911 |
| fingerprint C3 only D=8192 | 0.818 | 0.812 | 0.959 | 0.844 | 0.847 | 0.752 | 0.711 | 0.807 | 0.963 |
| fingerprint C4 only D=8192 | 0.616 | 0.611 | 0.939 | 0.669 | 0.641 | 0.497 | 0.411 | 0.613 | 0.532 |
| exact mix C2 only +IDF | 0.802 | 0.801 | 0.994 | 0.864 | 0.878 | 0.816 | 0.760 | 0.890 | 0.913 |
| exact mix C3 only +IDF | 0.842 | 0.843 | 0.999 | 0.895 | 0.900 | 0.832 | 0.802 | 0.871 | 0.965 |
| exact mix C4 only +IDF | 0.614 | 0.611 | 0.971 | 0.668 | 0.660 | 0.516 | 0.439 | 0.619 | 0.557 |
| exact mix C2+C3 +IDF | 0.821 | 0.821 | 0.998 | 0.898 | 0.900 | 0.843 | 0.799 | 0.902 | 0.946 |
| exact mix C2+C3+C4 +IDF | 0.810 | 0.809 | 0.998 | 0.905 | 0.898 | 0.849 | 0.799 | 0.916 | 0.948 |
| exact mix C1+C2+C3 +IDF | 0.777 | 0.777 | 0.998 | 0.874 | 0.881 | 0.811 | 0.773 | 0.862 | 0.937 |
| ablation: no taxonomy | 0.793 | 0.792 | 0.998 | 0.880 | 0.883 | 0.813 | 0.787 | 0.848 | 0.944 |
| ablation: no systematicity (beta=0) | 0.770 | 0.769 | 0.999 | 0.885 | 0.883 | 0.817 | 0.790 | 0.853 | 0.951 |
| ablation: no parent-child | 0.590 | 0.591 | 0.994 | 0.758 | 0.766 | 0.641 | 0.659 | 0.617 | 0.862 |
| ablation: no co-entity | 0.805 | 0.804 | 0.998 | 0.894 | 0.895 | 0.840 | 0.794 | 0.902 | 0.946 |
| ablation: co-entity weight 1.0 | 0.731 | 0.730 | 0.999 | 0.851 | 0.852 | 0.768 | 0.747 | 0.797 | 0.916 |
| ablation: co-entity weight 0.5 | 0.774 | 0.773 | 0.999 | 0.884 | 0.890 | 0.823 | 0.788 | 0.869 | 0.941 |
| ablation: no co-expr | 0.800 | 0.799 | 0.998 | 0.892 | 0.892 | 0.837 | 0.792 | 0.897 | 0.951 |
| ablation: WL depth 1 | 0.797 | 0.796 | 0.999 | 0.890 | 0.892 | 0.831 | 0.788 | 0.888 | 0.939 |
| ablation: WL depth 3 | 0.792 | 0.791 | 0.998 | 0.894 | 0.894 | 0.838 | 0.790 | 0.902 | 0.948 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.874 | 0.874 |
| star | 0.000 | 0.825 | 0.825 |
| tree | 0.000 | 0.853 | 0.846 |
| loop | 0.000 | 0.608 | 0.594 |
| deep-ho (test) | 0.000 | 0.867 | 0.867 |
| comparison (test) | 0.000 | 0.930 | 0.902 |
| mixed (test) | 0.000 | 0.894 | 0.880 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.109 ± 0.053 | 0.202 ± 0.054 | 0.100 ± 0.035 | 0.371 ± 0.031 | 0.402 ± 0.064 |
| TA | 0.500 ± 0.015 | 0.284 ± 0.077 | 0.282 ± 0.083 | 0.446 ± 0.032 | 0.398 ± 0.070 |
| MA | 0.107 ± 0.055 | 0.283 ± 0.079 | 0.382 ± 0.083 | 0.480 ± 0.021 | 0.421 ± 0.058 |
| FOR | 0.501 ± 0.016 | 0.282 ± 0.077 | 0.382 ± 0.084 | 0.480 ± 0.022 | 0.420 ± 0.059 |
| RND | 0.496 ± 0.022 | 0.453 ± 0.040 | 0.494 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.022 |
