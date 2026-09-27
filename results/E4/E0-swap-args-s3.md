# E0 — fingerprint separability (swap-args-s3)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=153.9, runtime=1.5s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 484/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.514 | 0.957 | 0.000 | 0.513 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.001 | 0.507 | 0.989 | 0.000 | 0.514 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.462 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.509 | 0.511 | 1.000 | 0.526 | 0.540 | 0.374 | 0.385 | 0.360 | 0.370 |
| exact C2 relational | 0.926 | 0.925 | 1.000 | 0.959 | 0.959 | 0.945 | 0.907 | 0.995 | 0.932 |
| exact C3 WL | 0.964 | 0.962 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| exact C4 topology | 0.583 | 0.587 | 0.964 | 0.591 | 0.602 | 0.446 | 0.362 | 0.558 | 0.531 |
| exact analogy profile | 0.960 | 0.959 | 1.000 | 0.998 | 0.999 | 0.998 | 0.998 | 0.998 | 1.000 |
| exact analogy profile +IDF | 0.970 | 0.970 | 1.000 | 0.998 | 0.999 | 0.998 | 0.998 | 0.998 | 1.000 |
| exact literal profile +IDF | 0.000 | 0.955 | 1.000 | 0.000 | 0.999 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.950 | 0.952 | 1.000 | 0.991 | 0.989 | 0.986 | 0.979 | 0.995 | 0.988 |
| fingerprint analogy +IDF D=2048 | 0.954 | 0.955 | 1.000 | 0.995 | 0.996 | 0.992 | 0.990 | 0.995 | 0.994 |
| fingerprint analogy +IDF D=4096 | 0.956 | 0.958 | 1.000 | 0.997 | 0.999 | 0.996 | 0.995 | 0.998 | 1.000 |
| fingerprint analogy +IDF D=8192 | 0.958 | 0.959 | 1.000 | 0.998 | 0.999 | 0.998 | 0.998 | 0.998 | 1.000 |
| fingerprint analogy +IDF D=16384 | 0.958 | 0.958 | 1.000 | 0.997 | 0.999 | 0.997 | 0.997 | 0.998 | 1.000 |
| fingerprint C1 only D=8192 | 0.512 | 0.514 | 1.000 | 0.523 | 0.510 | 0.348 | 0.362 | 0.329 | 0.347 |
| fingerprint C2 only D=8192 | 0.937 | 0.938 | 1.000 | 0.961 | 0.962 | 0.944 | 0.905 | 0.995 | 0.925 |
| fingerprint C3 only D=8192 | 0.987 | 0.986 | 1.000 | 0.997 | 0.998 | 0.995 | 0.995 | 0.995 | 1.000 |
| fingerprint C4 only D=8192 | 0.570 | 0.572 | 0.904 | 0.601 | 0.605 | 0.422 | 0.309 | 0.572 | 0.513 |
| exact mix C2 only +IDF | 0.940 | 0.940 | 1.000 | 0.956 | 0.958 | 0.943 | 0.904 | 0.995 | 0.926 |
| exact mix C3 only +IDF | 0.991 | 0.990 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| exact mix C4 only +IDF | 0.566 | 0.569 | 0.957 | 0.594 | 0.608 | 0.425 | 0.318 | 0.568 | 0.517 |
| exact mix C2+C3 +IDF | 0.985 | 0.985 | 1.000 | 0.998 | 0.999 | 0.998 | 0.998 | 0.998 | 1.000 |
| exact mix C2+C3+C4 +IDF | 0.963 | 0.963 | 1.000 | 0.998 | 0.999 | 0.998 | 0.998 | 0.998 | 1.000 |
| exact mix C1+C2+C3 +IDF | 0.975 | 0.975 | 1.000 | 0.998 | 0.999 | 0.998 | 0.998 | 0.998 | 1.000 |
| ablation: no taxonomy | 0.974 | 0.974 | 1.000 | 0.998 | 0.999 | 0.998 | 0.998 | 0.998 | 1.000 |
| ablation: no systematicity (beta=0) | 0.950 | 0.950 | 1.000 | 0.998 | 0.999 | 0.998 | 0.998 | 0.998 | 1.000 |
| ablation: no parent-child | 0.702 | 0.704 | 0.999 | 0.956 | 0.963 | 0.934 | 0.930 | 0.939 | 0.975 |
| ablation: no co-entity | 0.984 | 0.984 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| ablation: co-entity weight 1.0 | 0.906 | 0.907 | 1.000 | 0.991 | 0.990 | 0.983 | 0.981 | 0.986 | 0.992 |
| ablation: co-entity weight 0.5 | 0.946 | 0.945 | 1.000 | 0.997 | 0.999 | 0.997 | 0.998 | 0.995 | 0.998 |
| ablation: no co-expr | 0.972 | 0.972 | 1.000 | 0.998 | 0.999 | 0.998 | 0.998 | 0.998 | 1.000 |
| ablation: WL depth 1 | 0.974 | 0.974 | 1.000 | 0.998 | 0.999 | 0.998 | 0.998 | 0.998 | 1.000 |
| ablation: WL depth 3 | 0.965 | 0.965 | 1.000 | 0.998 | 0.999 | 0.998 | 0.998 | 0.998 | 1.000 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 0.993 | 0.993 |
| loop | 0.000 | 1.000 | 1.000 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 0.993 | 0.993 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.203 ± 0.054 | 0.100 ± 0.035 | 0.372 ± 0.031 | 0.404 ± 0.062 |
| TA | 0.500 ± 0.015 | 0.202 ± 0.054 | 0.120 ± 0.044 | 0.387 ± 0.027 | 0.423 ± 0.047 |
| MA | 0.106 ± 0.054 | 0.204 ± 0.053 | 0.339 ± 0.112 | 0.472 ± 0.024 | 0.435 ± 0.041 |
| FOR | 0.501 ± 0.015 | 0.204 ± 0.056 | 0.339 ± 0.111 | 0.472 ± 0.024 | 0.435 ± 0.042 |
| RND | 0.496 ± 0.022 | 0.454 ± 0.040 | 0.495 ± 0.016 | 0.499 ± 0.011 | 0.483 ± 0.021 |
