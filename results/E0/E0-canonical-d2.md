# E0 — fingerprint separability (canonical-d2)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=153.9, runtime=1.4s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) |
|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.501 | 0.957 | 0.000 | 0.493 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.001 | 0.502 | 0.988 | 0.000 | 0.511 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.462 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.501 | 0.507 | 1.000 | 0.489 | 0.511 | 0.344 | 0.327 | 0.368 |
| exact C2 relational | 0.920 | 0.920 | 1.000 | 0.955 | 0.949 | 0.935 | 0.904 | 0.977 |
| exact C3 WL | 0.960 | 0.960 | 1.000 | 0.992 | 0.984 | 0.980 | 0.996 | 0.959 |
| exact C4 topology | 0.601 | 0.600 | 0.973 | 0.624 | 0.589 | 0.477 | 0.384 | 0.603 |
| exact analogy profile | 0.960 | 0.959 | 1.000 | 0.992 | 0.986 | 0.983 | 0.995 | 0.967 |
| exact analogy profile +IDF | 0.963 | 0.962 | 1.000 | 0.993 | 0.987 | 0.985 | 0.995 | 0.972 |
| exact literal profile +IDF | 0.000 | 0.950 | 1.000 | 0.000 | 0.988 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.946 | 0.948 | 1.000 | 0.989 | 0.980 | 0.974 | 0.976 | 0.972 |
| fingerprint analogy +IDF D=2048 | 0.950 | 0.951 | 1.000 | 0.994 | 0.985 | 0.983 | 0.990 | 0.974 |
| fingerprint analogy +IDF D=4096 | 0.954 | 0.953 | 1.000 | 0.994 | 0.989 | 0.986 | 0.995 | 0.974 |
| fingerprint analogy +IDF D=8192 | 0.955 | 0.954 | 1.000 | 0.994 | 0.988 | 0.984 | 0.993 | 0.972 |
| fingerprint analogy +IDF D=16384 | 0.953 | 0.954 | 1.000 | 0.991 | 0.987 | 0.982 | 0.993 | 0.967 |
| fingerprint C1 only D=8192 | 0.502 | 0.505 | 1.000 | 0.497 | 0.524 | 0.343 | 0.330 | 0.360 |
| fingerprint C2 only D=8192 | 0.934 | 0.934 | 1.000 | 0.948 | 0.942 | 0.926 | 0.892 | 0.972 |
| fingerprint C3 only D=8192 | 0.977 | 0.977 | 1.000 | 0.990 | 0.986 | 0.981 | 0.990 | 0.970 |
| fingerprint C4 only D=8192 | 0.603 | 0.612 | 0.936 | 0.623 | 0.648 | 0.491 | 0.406 | 0.604 |
| exact mix C2 only +IDF | 0.935 | 0.935 | 1.000 | 0.956 | 0.944 | 0.933 | 0.902 | 0.974 |
| exact mix C3 only +IDF | 0.981 | 0.980 | 1.000 | 0.987 | 0.984 | 0.979 | 0.993 | 0.960 |
| exact mix C4 only +IDF | 0.600 | 0.607 | 0.970 | 0.660 | 0.679 | 0.523 | 0.439 | 0.636 |
| exact mix C2+C3 +IDF | 0.981 | 0.980 | 1.000 | 0.990 | 0.987 | 0.984 | 0.995 | 0.970 |
| exact mix C2+C3+C4 +IDF | 0.951 | 0.950 | 1.000 | 0.993 | 0.986 | 0.983 | 0.993 | 0.970 |
| exact mix C1+C2+C3 +IDF | 0.973 | 0.972 | 1.000 | 0.991 | 0.988 | 0.984 | 0.997 | 0.967 |
| ablation: no taxonomy | 0.967 | 0.966 | 1.000 | 0.992 | 0.985 | 0.983 | 0.995 | 0.967 |
| ablation: no systematicity (beta=0) | 0.949 | 0.950 | 1.000 | 0.992 | 0.987 | 0.984 | 0.995 | 0.970 |
| ablation: no parent-child | 0.738 | 0.742 | 1.000 | 0.974 | 0.978 | 0.960 | 0.967 | 0.951 |
| ablation: no co-entity | 0.972 | 0.971 | 1.000 | 0.992 | 0.984 | 0.982 | 0.995 | 0.965 |
| ablation: co-entity weight 1.0 | 0.921 | 0.923 | 1.000 | 0.991 | 0.987 | 0.982 | 0.993 | 0.967 |
| ablation: co-entity weight 0.5 | 0.948 | 0.948 | 1.000 | 0.992 | 0.987 | 0.984 | 0.995 | 0.970 |
| ablation: no co-expr | 0.965 | 0.964 | 1.000 | 0.993 | 0.987 | 0.985 | 0.995 | 0.972 |
| ablation: WL depth 1 | 0.967 | 0.967 | 1.000 | 0.993 | 0.985 | 0.983 | 0.995 | 0.967 |
| ablation: WL depth 3 | 0.958 | 0.957 | 1.000 | 0.993 | 0.986 | 0.983 | 0.995 | 0.967 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 0.986 | 0.986 |
| loop | 0.000 | 0.993 | 0.986 |
| deep-ho (test) | 0.000 | 0.916 | 0.916 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.203 ± 0.054 | 0.100 ± 0.035 | 0.372 ± 0.031 | 0.402 ± 0.064 |
| TA | 0.501 ± 0.015 | 0.202 ± 0.054 | 0.100 ± 0.036 | 0.375 ± 0.032 | 0.400 ± 0.071 |
| MA | 0.107 ± 0.055 | 0.203 ± 0.054 | 0.325 ± 0.116 | 0.462 ± 0.028 | 0.421 ± 0.057 |
| FOR | 0.500 ± 0.016 | 0.204 ± 0.053 | 0.325 ± 0.116 | 0.462 ± 0.027 | 0.422 ± 0.057 |
| RND | 0.497 ± 0.023 | 0.454 ± 0.039 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.021 |
