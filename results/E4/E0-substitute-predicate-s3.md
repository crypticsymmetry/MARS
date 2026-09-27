# E0 — fingerprint separability (substitute-predicate-s3)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=153.5, runtime=1.5s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 285/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.494 | 0.897 | 0.000 | 0.486 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.026 | 0.493 | 0.919 | 0.000 | 0.484 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.464 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.505 | 0.494 | 0.968 | 0.507 | 0.489 | 0.325 | 0.336 | 0.311 | 0.411 |
| exact C2 relational | 0.739 | 0.734 | 0.987 | 0.839 | 0.838 | 0.773 | 0.708 | 0.860 | 0.898 |
| exact C3 WL | 0.748 | 0.744 | 0.986 | 0.855 | 0.855 | 0.769 | 0.711 | 0.847 | 0.954 |
| exact C4 topology | 0.616 | 0.608 | 0.976 | 0.641 | 0.633 | 0.496 | 0.378 | 0.654 | 0.542 |
| exact analogy profile | 0.738 | 0.733 | 0.997 | 0.860 | 0.851 | 0.788 | 0.738 | 0.855 | 0.933 |
| exact analogy profile +IDF | 0.731 | 0.725 | 0.995 | 0.846 | 0.832 | 0.758 | 0.719 | 0.811 | 0.937 |
| exact literal profile +IDF | 0.000 | 0.687 | 0.987 | 0.000 | 0.791 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.724 | 0.711 | 0.975 | 0.806 | 0.791 | 0.705 | 0.654 | 0.772 | 0.891 |
| fingerprint analogy +IDF D=2048 | 0.726 | 0.718 | 0.982 | 0.825 | 0.810 | 0.735 | 0.684 | 0.801 | 0.909 |
| fingerprint analogy +IDF D=4096 | 0.726 | 0.719 | 0.988 | 0.832 | 0.815 | 0.737 | 0.689 | 0.801 | 0.919 |
| fingerprint analogy +IDF D=8192 | 0.724 | 0.717 | 0.992 | 0.841 | 0.821 | 0.744 | 0.698 | 0.804 | 0.921 |
| fingerprint analogy +IDF D=16384 | 0.724 | 0.717 | 0.993 | 0.842 | 0.817 | 0.742 | 0.685 | 0.818 | 0.916 |
| fingerprint C1 only D=8192 | 0.507 | 0.495 | 0.961 | 0.509 | 0.499 | 0.336 | 0.344 | 0.326 | 0.444 |
| fingerprint C2 only D=8192 | 0.735 | 0.728 | 0.980 | 0.829 | 0.822 | 0.732 | 0.683 | 0.798 | 0.895 |
| fingerprint C3 only D=8192 | 0.751 | 0.746 | 0.921 | 0.801 | 0.775 | 0.668 | 0.615 | 0.738 | 0.946 |
| fingerprint C4 only D=8192 | 0.621 | 0.611 | 0.936 | 0.683 | 0.648 | 0.513 | 0.423 | 0.634 | 0.567 |
| exact mix C2 only +IDF | 0.738 | 0.732 | 0.986 | 0.833 | 0.817 | 0.746 | 0.701 | 0.806 | 0.895 |
| exact mix C3 only +IDF | 0.781 | 0.779 | 0.990 | 0.858 | 0.850 | 0.771 | 0.724 | 0.834 | 0.968 |
| exact mix C4 only +IDF | 0.627 | 0.618 | 0.972 | 0.723 | 0.678 | 0.566 | 0.458 | 0.710 | 0.607 |
| exact mix C2+C3 +IDF | 0.753 | 0.748 | 0.993 | 0.853 | 0.843 | 0.770 | 0.734 | 0.818 | 0.951 |
| exact mix C2+C3+C4 +IDF | 0.750 | 0.745 | 0.996 | 0.865 | 0.853 | 0.793 | 0.745 | 0.857 | 0.940 |
| exact mix C1+C2+C3 +IDF | 0.709 | 0.704 | 0.991 | 0.820 | 0.798 | 0.716 | 0.684 | 0.759 | 0.930 |
| ablation: no taxonomy | 0.726 | 0.719 | 0.993 | 0.825 | 0.807 | 0.730 | 0.706 | 0.762 | 0.940 |
| ablation: no systematicity (beta=0) | 0.709 | 0.702 | 0.995 | 0.833 | 0.827 | 0.741 | 0.710 | 0.783 | 0.933 |
| ablation: no parent-child | 0.570 | 0.563 | 0.989 | 0.700 | 0.691 | 0.558 | 0.524 | 0.603 | 0.863 |
| ablation: no co-entity | 0.739 | 0.733 | 0.995 | 0.852 | 0.839 | 0.769 | 0.731 | 0.820 | 0.937 |
| ablation: co-entity weight 1.0 | 0.680 | 0.672 | 0.995 | 0.790 | 0.776 | 0.678 | 0.638 | 0.731 | 0.909 |
| ablation: co-entity weight 0.5 | 0.714 | 0.707 | 0.995 | 0.835 | 0.820 | 0.741 | 0.703 | 0.792 | 0.930 |
| ablation: no co-expr | 0.735 | 0.729 | 0.994 | 0.846 | 0.831 | 0.755 | 0.713 | 0.811 | 0.937 |
| ablation: WL depth 1 | 0.732 | 0.726 | 0.995 | 0.845 | 0.828 | 0.757 | 0.720 | 0.806 | 0.940 |
| ablation: WL depth 3 | 0.728 | 0.722 | 0.995 | 0.844 | 0.829 | 0.752 | 0.715 | 0.801 | 0.930 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.811 | 0.818 |
| star | 0.000 | 0.713 | 0.657 |
| tree | 0.000 | 0.776 | 0.790 |
| loop | 0.000 | 0.573 | 0.528 |
| deep-ho (test) | 0.000 | 0.706 | 0.671 |
| comparison (test) | 0.000 | 0.888 | 0.902 |
| mixed (test) | 0.000 | 0.838 | 0.838 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.109 ± 0.054 | 0.202 ± 0.054 | 0.100 ± 0.035 | 0.370 ± 0.031 | 0.403 ± 0.063 |
| TA | 0.500 ± 0.016 | 0.302 ± 0.084 | 0.321 ± 0.087 | 0.460 ± 0.029 | 0.398 ± 0.071 |
| MA | 0.106 ± 0.055 | 0.304 ± 0.084 | 0.395 ± 0.078 | 0.483 ± 0.020 | 0.423 ± 0.056 |
| FOR | 0.500 ± 0.016 | 0.300 ± 0.082 | 0.393 ± 0.080 | 0.483 ± 0.020 | 0.421 ± 0.059 |
| RND | 0.496 ± 0.022 | 0.454 ± 0.039 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.482 ± 0.022 |
