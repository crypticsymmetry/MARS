# E0 — fingerprint separability (swap-args-s4)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=154.1, runtime=1.4s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 409/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.498 | 0.962 | 0.000 | 0.511 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.000 | 0.500 | 0.987 | 0.000 | 0.483 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.466 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.498 | 0.500 | 1.000 | 0.505 | 0.484 | 0.329 | 0.316 | 0.346 | 0.322 |
| exact C2 relational | 0.925 | 0.925 | 1.000 | 0.955 | 0.953 | 0.941 | 0.899 | 0.995 | 0.961 |
| exact C3 WL | 0.962 | 0.961 | 1.000 | 0.999 | 0.999 | 0.998 | 0.996 | 1.000 | 1.000 |
| exact C4 topology | 0.580 | 0.592 | 0.967 | 0.566 | 0.585 | 0.422 | 0.307 | 0.576 | 0.500 |
| exact analogy profile | 0.959 | 0.958 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| exact analogy profile +IDF | 0.968 | 0.968 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact literal profile +IDF | 0.000 | 0.953 | 1.000 | 0.000 | 0.998 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.949 | 0.948 | 1.000 | 0.990 | 0.988 | 0.979 | 0.963 | 1.000 | 0.985 |
| fingerprint analogy +IDF D=2048 | 0.953 | 0.952 | 1.000 | 0.997 | 0.991 | 0.990 | 0.983 | 1.000 | 1.000 |
| fingerprint analogy +IDF D=4096 | 0.956 | 0.955 | 1.000 | 1.000 | 0.994 | 0.994 | 0.990 | 1.000 | 1.000 |
| fingerprint analogy +IDF D=8192 | 0.958 | 0.956 | 1.000 | 1.000 | 0.996 | 0.996 | 0.993 | 1.000 | 1.000 |
| fingerprint analogy +IDF D=16384 | 0.957 | 0.956 | 1.000 | 1.000 | 0.994 | 0.994 | 0.990 | 1.000 | 1.000 |
| fingerprint C1 only D=8192 | 0.500 | 0.498 | 1.000 | 0.510 | 0.505 | 0.360 | 0.355 | 0.367 | 0.345 |
| fingerprint C2 only D=8192 | 0.935 | 0.937 | 1.000 | 0.950 | 0.954 | 0.938 | 0.895 | 0.995 | 0.954 |
| fingerprint C3 only D=8192 | 0.986 | 0.984 | 1.000 | 0.998 | 0.997 | 0.995 | 0.997 | 0.993 | 0.998 |
| fingerprint C4 only D=8192 | 0.564 | 0.567 | 0.906 | 0.587 | 0.607 | 0.409 | 0.331 | 0.513 | 0.500 |
| exact mix C2 only +IDF | 0.939 | 0.940 | 1.000 | 0.952 | 0.954 | 0.938 | 0.895 | 0.995 | 0.958 |
| exact mix C3 only +IDF | 0.990 | 0.989 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact mix C4 only +IDF | 0.565 | 0.574 | 0.957 | 0.591 | 0.609 | 0.428 | 0.330 | 0.558 | 0.518 |
| exact mix C2+C3 +IDF | 0.984 | 0.984 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| exact mix C2+C3+C4 +IDF | 0.961 | 0.961 | 1.000 | 0.999 | 0.999 | 0.998 | 0.997 | 1.000 | 1.000 |
| exact mix C1+C2+C3 +IDF | 0.973 | 0.973 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| ablation: no taxonomy | 0.972 | 0.972 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: no systematicity (beta=0) | 0.946 | 0.946 | 1.000 | 1.000 | 0.999 | 0.999 | 1.000 | 0.998 | 0.998 |
| ablation: no parent-child | 0.694 | 0.694 | 0.999 | 0.924 | 0.923 | 0.863 | 0.869 | 0.855 | 0.966 |
| ablation: no co-entity | 0.983 | 0.983 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: co-entity weight 1.0 | 0.896 | 0.897 | 1.000 | 0.988 | 0.986 | 0.975 | 0.963 | 0.991 | 0.998 |
| ablation: co-entity weight 0.5 | 0.940 | 0.941 | 1.000 | 1.000 | 0.995 | 0.995 | 0.993 | 0.998 | 0.998 |
| ablation: no co-expr | 0.970 | 0.970 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: WL depth 1 | 0.972 | 0.972 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: WL depth 3 | 0.964 | 0.963 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 1.000 | 0.993 |
| loop | 0.000 | 1.000 | 0.979 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.109 ± 0.053 | 0.202 ± 0.054 | 0.100 ± 0.035 | 0.372 ± 0.031 | 0.403 ± 0.062 |
| TA | 0.500 ± 0.015 | 0.203 ± 0.054 | 0.121 ± 0.046 | 0.388 ± 0.027 | 0.422 ± 0.051 |
| MA | 0.107 ± 0.055 | 0.204 ± 0.053 | 0.339 ± 0.111 | 0.473 ± 0.024 | 0.433 ± 0.043 |
| FOR | 0.501 ± 0.016 | 0.203 ± 0.053 | 0.339 ± 0.111 | 0.472 ± 0.024 | 0.433 ± 0.043 |
| RND | 0.496 ± 0.022 | 0.453 ± 0.040 | 0.494 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.022 |
