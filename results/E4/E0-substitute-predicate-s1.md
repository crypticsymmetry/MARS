# E0 — fingerprint separability (substitute-predicate-s1)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=153.8, runtime=1.6s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 668/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.501 | 0.944 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.006 | 0.501 | 0.971 | 0.000 | 0.483 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.462 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.497 | 0.501 | 0.997 | 0.512 | 0.507 | 0.336 | 0.332 | 0.341 | 0.344 |
| exact C2 relational | 0.872 | 0.870 | 1.000 | 0.931 | 0.929 | 0.902 | 0.842 | 0.984 | 0.908 |
| exact C3 WL | 0.882 | 0.884 | 1.000 | 0.978 | 0.980 | 0.961 | 0.951 | 0.974 | 0.973 |
| exact C4 topology | 0.606 | 0.611 | 0.976 | 0.627 | 0.629 | 0.494 | 0.377 | 0.652 | 0.507 |
| exact analogy profile | 0.875 | 0.876 | 1.000 | 0.977 | 0.979 | 0.961 | 0.939 | 0.991 | 0.973 |
| exact analogy profile +IDF | 0.883 | 0.884 | 1.000 | 0.969 | 0.976 | 0.952 | 0.923 | 0.991 | 0.975 |
| exact literal profile +IDF | 0.000 | 0.852 | 1.000 | 0.000 | 0.970 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.867 | 0.867 | 1.000 | 0.951 | 0.954 | 0.920 | 0.874 | 0.981 | 0.943 |
| fingerprint analogy +IDF D=2048 | 0.870 | 0.871 | 1.000 | 0.953 | 0.958 | 0.927 | 0.880 | 0.991 | 0.951 |
| fingerprint analogy +IDF D=4096 | 0.872 | 0.872 | 1.000 | 0.958 | 0.962 | 0.936 | 0.897 | 0.988 | 0.958 |
| fingerprint analogy +IDF D=8192 | 0.873 | 0.872 | 1.000 | 0.965 | 0.969 | 0.946 | 0.913 | 0.991 | 0.970 |
| fingerprint analogy +IDF D=16384 | 0.873 | 0.873 | 1.000 | 0.968 | 0.972 | 0.950 | 0.921 | 0.988 | 0.970 |
| fingerprint C1 only D=8192 | 0.498 | 0.498 | 0.997 | 0.511 | 0.481 | 0.323 | 0.316 | 0.331 | 0.325 |
| fingerprint C2 only D=8192 | 0.879 | 0.877 | 1.000 | 0.933 | 0.925 | 0.906 | 0.848 | 0.984 | 0.915 |
| fingerprint C3 only D=8192 | 0.904 | 0.905 | 0.993 | 0.948 | 0.945 | 0.906 | 0.879 | 0.942 | 0.964 |
| fingerprint C4 only D=8192 | 0.608 | 0.617 | 0.939 | 0.633 | 0.668 | 0.487 | 0.408 | 0.593 | 0.529 |
| exact mix C2 only +IDF | 0.885 | 0.885 | 1.000 | 0.927 | 0.926 | 0.900 | 0.834 | 0.988 | 0.910 |
| exact mix C3 only +IDF | 0.920 | 0.924 | 1.000 | 0.967 | 0.974 | 0.943 | 0.927 | 0.965 | 0.976 |
| exact mix C4 only +IDF | 0.619 | 0.626 | 0.972 | 0.680 | 0.697 | 0.561 | 0.463 | 0.693 | 0.586 |
| exact mix C2+C3 +IDF | 0.909 | 0.910 | 1.000 | 0.973 | 0.976 | 0.958 | 0.934 | 0.991 | 0.981 |
| exact mix C2+C3+C4 +IDF | 0.883 | 0.886 | 1.000 | 0.969 | 0.969 | 0.946 | 0.913 | 0.991 | 0.970 |
| exact mix C1+C2+C3 +IDF | 0.876 | 0.877 | 1.000 | 0.971 | 0.976 | 0.953 | 0.927 | 0.988 | 0.979 |
| ablation: no taxonomy | 0.885 | 0.887 | 1.000 | 0.974 | 0.975 | 0.957 | 0.932 | 0.991 | 0.979 |
| ablation: no systematicity (beta=0) | 0.857 | 0.859 | 1.000 | 0.970 | 0.972 | 0.949 | 0.918 | 0.991 | 0.972 |
| ablation: no parent-child | 0.638 | 0.640 | 0.999 | 0.888 | 0.903 | 0.836 | 0.827 | 0.848 | 0.922 |
| ablation: no co-entity | 0.892 | 0.893 | 1.000 | 0.970 | 0.977 | 0.955 | 0.928 | 0.991 | 0.975 |
| ablation: co-entity weight 1.0 | 0.821 | 0.823 | 1.000 | 0.954 | 0.965 | 0.931 | 0.899 | 0.974 | 0.960 |
| ablation: co-entity weight 0.5 | 0.862 | 0.864 | 1.000 | 0.968 | 0.969 | 0.945 | 0.916 | 0.984 | 0.969 |
| ablation: no co-expr | 0.887 | 0.889 | 1.000 | 0.969 | 0.976 | 0.952 | 0.923 | 0.991 | 0.975 |
| ablation: WL depth 1 | 0.884 | 0.886 | 1.000 | 0.970 | 0.977 | 0.954 | 0.927 | 0.991 | 0.975 |
| ablation: WL depth 3 | 0.878 | 0.880 | 1.000 | 0.967 | 0.974 | 0.950 | 0.920 | 0.991 | 0.970 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.951 | 0.951 |
| star | 0.000 | 0.909 | 0.895 |
| tree | 0.000 | 0.944 | 0.937 |
| loop | 0.000 | 0.888 | 0.867 |
| deep-ho (test) | 0.000 | 0.993 | 1.000 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 0.979 | 0.972 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.202 ± 0.054 | 0.100 ± 0.035 | 0.371 ± 0.031 | 0.402 ± 0.063 |
| TA | 0.500 ± 0.016 | 0.252 ± 0.068 | 0.225 ± 0.070 | 0.422 ± 0.032 | 0.398 ± 0.070 |
| MA | 0.107 ± 0.053 | 0.252 ± 0.066 | 0.364 ± 0.089 | 0.474 ± 0.023 | 0.420 ± 0.057 |
| FOR | 0.500 ± 0.015 | 0.251 ± 0.067 | 0.363 ± 0.090 | 0.474 ± 0.023 | 0.422 ± 0.057 |
| RND | 0.496 ± 0.022 | 0.453 ± 0.040 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.021 |
