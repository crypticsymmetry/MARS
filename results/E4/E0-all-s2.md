# E0 — fingerprint separability (all-s2)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=155.3, runtime=1.5s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 552/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.499 | 0.942 | 0.000 | 0.484 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.009 | 0.495 | 0.955 | 0.002 | 0.478 | 0.001 | 0.000 | 0.002 | 0.002 |
| exact C0 surface | 0.000 | 0.500 | 0.460 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.497 | 0.495 | 0.988 | 0.517 | 0.484 | 0.337 | 0.317 | 0.362 | 0.378 |
| exact C2 relational | 0.802 | 0.810 | 0.965 | 0.834 | 0.845 | 0.791 | 0.773 | 0.815 | 0.870 |
| exact C3 WL | 0.817 | 0.820 | 0.986 | 0.894 | 0.882 | 0.841 | 0.834 | 0.849 | 0.902 |
| exact C4 topology | 0.563 | 0.581 | 0.907 | 0.577 | 0.597 | 0.440 | 0.361 | 0.544 | 0.455 |
| exact analogy profile | 0.800 | 0.807 | 0.988 | 0.849 | 0.862 | 0.813 | 0.804 | 0.825 | 0.899 |
| exact analogy profile +IDF | 0.818 | 0.826 | 0.995 | 0.857 | 0.861 | 0.816 | 0.818 | 0.813 | 0.899 |
| exact literal profile +IDF | 0.001 | 0.800 | 0.993 | 0.000 | 0.845 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.812 | 0.815 | 0.982 | 0.856 | 0.840 | 0.795 | 0.787 | 0.806 | 0.882 |
| fingerprint analogy +IDF D=2048 | 0.812 | 0.817 | 0.987 | 0.851 | 0.844 | 0.795 | 0.791 | 0.801 | 0.886 |
| fingerprint analogy +IDF D=4096 | 0.813 | 0.819 | 0.989 | 0.854 | 0.848 | 0.806 | 0.801 | 0.813 | 0.886 |
| fingerprint analogy +IDF D=8192 | 0.812 | 0.819 | 0.991 | 0.857 | 0.858 | 0.813 | 0.816 | 0.808 | 0.895 |
| fingerprint analogy +IDF D=16384 | 0.811 | 0.819 | 0.993 | 0.856 | 0.859 | 0.812 | 0.809 | 0.815 | 0.895 |
| fingerprint C1 only D=8192 | 0.498 | 0.492 | 0.995 | 0.502 | 0.498 | 0.338 | 0.330 | 0.349 | 0.364 |
| fingerprint C2 only D=8192 | 0.816 | 0.824 | 0.971 | 0.843 | 0.854 | 0.801 | 0.795 | 0.811 | 0.880 |
| fingerprint C3 only D=8192 | 0.849 | 0.854 | 0.962 | 0.866 | 0.876 | 0.807 | 0.802 | 0.813 | 0.911 |
| fingerprint C4 only D=8192 | 0.553 | 0.574 | 0.835 | 0.577 | 0.593 | 0.377 | 0.302 | 0.478 | 0.395 |
| exact mix C2 only +IDF | 0.818 | 0.826 | 0.977 | 0.841 | 0.846 | 0.799 | 0.794 | 0.806 | 0.877 |
| exact mix C3 only +IDF | 0.863 | 0.866 | 0.987 | 0.894 | 0.903 | 0.851 | 0.853 | 0.848 | 0.938 |
| exact mix C4 only +IDF | 0.559 | 0.571 | 0.903 | 0.598 | 0.597 | 0.432 | 0.355 | 0.535 | 0.447 |
| exact mix C2+C3 +IDF | 0.838 | 0.844 | 0.986 | 0.869 | 0.870 | 0.829 | 0.832 | 0.825 | 0.911 |
| exact mix C2+C3+C4 +IDF | 0.823 | 0.831 | 0.988 | 0.861 | 0.864 | 0.819 | 0.820 | 0.818 | 0.897 |
| exact mix C1+C2+C3 +IDF | 0.812 | 0.818 | 0.996 | 0.854 | 0.853 | 0.811 | 0.811 | 0.811 | 0.899 |
| ablation: no taxonomy | 0.822 | 0.829 | 0.996 | 0.866 | 0.860 | 0.819 | 0.818 | 0.820 | 0.902 |
| ablation: no systematicity (beta=0) | 0.797 | 0.804 | 0.996 | 0.853 | 0.851 | 0.806 | 0.802 | 0.811 | 0.897 |
| ablation: no parent-child | 0.608 | 0.619 | 0.995 | 0.710 | 0.723 | 0.621 | 0.570 | 0.689 | 0.676 |
| ablation: no co-entity | 0.822 | 0.829 | 0.993 | 0.859 | 0.863 | 0.817 | 0.816 | 0.818 | 0.900 |
| ablation: co-entity weight 1.0 | 0.771 | 0.778 | 0.997 | 0.841 | 0.838 | 0.786 | 0.780 | 0.794 | 0.893 |
| ablation: co-entity weight 0.5 | 0.805 | 0.812 | 0.996 | 0.854 | 0.861 | 0.814 | 0.809 | 0.820 | 0.904 |
| ablation: no co-expr | 0.826 | 0.833 | 0.995 | 0.863 | 0.863 | 0.825 | 0.829 | 0.820 | 0.908 |
| ablation: WL depth 1 | 0.824 | 0.831 | 0.995 | 0.866 | 0.866 | 0.821 | 0.818 | 0.825 | 0.909 |
| ablation: WL depth 3 | 0.815 | 0.823 | 0.995 | 0.858 | 0.859 | 0.816 | 0.816 | 0.815 | 0.899 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.888 | 0.902 |
| star | 0.000 | 0.846 | 0.860 |
| tree | 0.000 | 0.846 | 0.853 |
| loop | 0.000 | 0.692 | 0.650 |
| deep-ho (test) | 0.000 | 0.762 | 0.762 |
| comparison (test) | 0.007 | 0.853 | 0.832 |
| mixed (test) | 0.000 | 0.824 | 0.831 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.202 ± 0.054 | 0.100 ± 0.035 | 0.370 ± 0.031 | 0.403 ± 0.062 |
| TA | 0.500 ± 0.016 | 0.250 ± 0.068 | 0.241 ± 0.109 | 0.432 ± 0.038 | 0.439 ± 0.047 |
| MA | 0.121 ± 0.055 | 0.249 ± 0.069 | 0.374 ± 0.097 | 0.478 ± 0.022 | 0.447 ± 0.045 |
| FOR | 0.500 ± 0.015 | 0.249 ± 0.070 | 0.376 ± 0.093 | 0.478 ± 0.022 | 0.451 ± 0.038 |
| RND | 0.496 ± 0.023 | 0.454 ± 0.040 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.022 |
