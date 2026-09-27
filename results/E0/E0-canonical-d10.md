# E0 — fingerprint separability (canonical-d10)

Config: groups=1000, seed=1, naming=Canonical, distractors=10, cases=8000, mean features/case=298.3, runtime=3.1s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) |
|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.501 | 0.917 | 0.000 | 0.516 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.059 | 0.501 | 0.962 | 0.001 | 0.502 | 0.001 | 0.002 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.464 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.500 | 0.502 | 0.976 | 0.511 | 0.500 | 0.340 | 0.339 | 0.341 |
| exact C2 relational | 0.866 | 0.866 | 1.000 | 0.942 | 0.949 | 0.926 | 0.897 | 0.965 |
| exact C3 WL | 0.926 | 0.925 | 0.999 | 0.973 | 0.979 | 0.958 | 0.951 | 0.967 |
| exact C4 topology | 0.551 | 0.558 | 0.939 | 0.557 | 0.576 | 0.416 | 0.321 | 0.544 |
| exact analogy profile | 0.892 | 0.894 | 1.000 | 0.981 | 0.984 | 0.974 | 0.983 | 0.963 |
| exact analogy profile +IDF | 0.924 | 0.924 | 1.000 | 0.990 | 0.992 | 0.988 | 1.000 | 0.972 |
| exact literal profile +IDF | 0.000 | 0.888 | 1.000 | 0.000 | 0.990 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.913 | 0.911 | 1.000 | 0.971 | 0.974 | 0.958 | 0.949 | 0.970 |
| fingerprint analogy +IDF D=2048 | 0.918 | 0.919 | 1.000 | 0.978 | 0.985 | 0.968 | 0.967 | 0.970 |
| fingerprint analogy +IDF D=4096 | 0.922 | 0.923 | 1.000 | 0.985 | 0.987 | 0.981 | 0.984 | 0.977 |
| fingerprint analogy +IDF D=8192 | 0.922 | 0.922 | 1.000 | 0.989 | 0.988 | 0.984 | 0.993 | 0.972 |
| fingerprint analogy +IDF D=16384 | 0.920 | 0.921 | 1.000 | 0.987 | 0.988 | 0.985 | 0.998 | 0.967 |
| fingerprint C1 only D=8192 | 0.496 | 0.503 | 0.976 | 0.469 | 0.513 | 0.327 | 0.315 | 0.343 |
| fingerprint C2 only D=8192 | 0.914 | 0.914 | 1.000 | 0.942 | 0.946 | 0.929 | 0.903 | 0.965 |
| fingerprint C3 only D=8192 | 0.962 | 0.963 | 0.999 | 0.985 | 0.986 | 0.976 | 0.987 | 0.960 |
| fingerprint C4 only D=8192 | 0.551 | 0.570 | 0.805 | 0.561 | 0.591 | 0.351 | 0.316 | 0.398 |
| exact mix C2 only +IDF | 0.909 | 0.910 | 1.000 | 0.945 | 0.947 | 0.926 | 0.900 | 0.960 |
| exact mix C3 only +IDF | 0.977 | 0.977 | 1.000 | 0.990 | 0.992 | 0.989 | 1.000 | 0.974 |
| exact mix C4 only +IDF | 0.558 | 0.567 | 0.923 | 0.582 | 0.601 | 0.418 | 0.343 | 0.519 |
| exact mix C2+C3 +IDF | 0.946 | 0.946 | 1.000 | 0.988 | 0.992 | 0.986 | 1.000 | 0.967 |
| exact mix C2+C3+C4 +IDF | 0.935 | 0.935 | 1.000 | 0.989 | 0.990 | 0.986 | 1.000 | 0.967 |
| exact mix C1+C2+C3 +IDF | 0.911 | 0.912 | 1.000 | 0.988 | 0.992 | 0.986 | 1.000 | 0.967 |
| ablation: no taxonomy | 0.932 | 0.932 | 1.000 | 0.990 | 0.992 | 0.988 | 1.000 | 0.972 |
| ablation: no systematicity (beta=0) | 0.883 | 0.883 | 1.000 | 0.989 | 0.992 | 0.987 | 1.000 | 0.970 |
| ablation: no parent-child | 0.692 | 0.692 | 1.000 | 0.963 | 0.972 | 0.946 | 0.951 | 0.939 |
| ablation: no co-entity | 0.974 | 0.974 | 1.000 | 0.991 | 0.991 | 0.989 | 1.000 | 0.974 |
| ablation: co-entity weight 1.0 | 0.854 | 0.852 | 1.000 | 0.982 | 0.983 | 0.973 | 0.984 | 0.958 |
| ablation: co-entity weight 0.5 | 0.883 | 0.883 | 1.000 | 0.989 | 0.990 | 0.985 | 0.998 | 0.967 |
| ablation: no co-expr | 0.930 | 0.930 | 1.000 | 0.990 | 0.992 | 0.988 | 1.000 | 0.972 |
| ablation: WL depth 1 | 0.931 | 0.931 | 1.000 | 0.990 | 0.993 | 0.988 | 1.000 | 0.972 |
| ablation: WL depth 3 | 0.918 | 0.918 | 1.000 | 0.990 | 0.992 | 0.988 | 1.000 | 0.972 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 0.993 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 1.000 | 1.000 |
| loop | 0.007 | 1.000 | 0.979 |
| deep-ho (test) | 0.000 | 0.916 | 0.916 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.127 ± 0.029 | 0.297 ± 0.064 | 0.207 ± 0.049 | 0.427 ± 0.022 | 0.443 ± 0.025 |
| TA | 0.499 ± 0.015 | 0.297 ± 0.065 | 0.207 ± 0.049 | 0.426 ± 0.022 | 0.441 ± 0.024 |
| MA | 0.127 ± 0.029 | 0.296 ± 0.063 | 0.360 ± 0.091 | 0.480 ± 0.019 | 0.446 ± 0.023 |
| FOR | 0.500 ± 0.015 | 0.298 ± 0.065 | 0.360 ± 0.091 | 0.481 ± 0.019 | 0.447 ± 0.024 |
| RND | 0.496 ± 0.022 | 0.428 ± 0.038 | 0.495 ± 0.012 | 0.497 ± 0.011 | 0.467 ± 0.018 |
