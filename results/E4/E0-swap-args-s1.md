# E0 — fingerprint separability (swap-args-s1)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=153.8, runtime=1.5s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 671/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.513 | 0.955 | 0.000 | 0.548 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.001 | 0.507 | 0.987 | 0.000 | 0.525 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.462 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.506 | 0.509 | 1.000 | 0.496 | 0.503 | 0.342 | 0.330 | 0.356 | 0.331 |
| exact C2 relational | 0.926 | 0.926 | 1.000 | 0.960 | 0.950 | 0.938 | 0.896 | 0.993 | 0.907 |
| exact C3 WL | 0.960 | 0.961 | 1.000 | 0.998 | 1.000 | 0.998 | 0.997 | 1.000 | 0.999 |
| exact C4 topology | 0.593 | 0.591 | 0.967 | 0.585 | 0.587 | 0.450 | 0.357 | 0.574 | 0.499 |
| exact analogy profile | 0.960 | 0.960 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.999 |
| exact analogy profile +IDF | 0.969 | 0.970 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.999 |
| exact literal profile +IDF | 0.000 | 0.955 | 1.000 | 0.000 | 1.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.952 | 0.951 | 1.000 | 0.990 | 0.990 | 0.985 | 0.976 | 0.998 | 0.979 |
| fingerprint analogy +IDF D=2048 | 0.954 | 0.954 | 1.000 | 0.995 | 0.997 | 0.994 | 0.990 | 1.000 | 0.993 |
| fingerprint analogy +IDF D=4096 | 0.957 | 0.957 | 1.000 | 0.999 | 0.997 | 0.997 | 0.995 | 1.000 | 0.996 |
| fingerprint analogy +IDF D=8192 | 0.958 | 0.958 | 1.000 | 0.999 | 0.997 | 0.997 | 0.995 | 1.000 | 0.996 |
| fingerprint analogy +IDF D=16384 | 0.958 | 0.958 | 1.000 | 0.999 | 0.999 | 0.998 | 0.997 | 1.000 | 0.997 |
| fingerprint C1 only D=8192 | 0.507 | 0.511 | 1.000 | 0.495 | 0.511 | 0.348 | 0.340 | 0.359 | 0.338 |
| fingerprint C2 only D=8192 | 0.939 | 0.938 | 1.000 | 0.952 | 0.953 | 0.936 | 0.893 | 0.993 | 0.905 |
| fingerprint C3 only D=8192 | 0.983 | 0.983 | 1.000 | 0.998 | 0.995 | 0.995 | 0.993 | 0.998 | 0.994 |
| fingerprint C4 only D=8192 | 0.585 | 0.586 | 0.918 | 0.619 | 0.609 | 0.427 | 0.309 | 0.585 | 0.452 |
| exact mix C2 only +IDF | 0.941 | 0.941 | 1.000 | 0.959 | 0.954 | 0.940 | 0.900 | 0.993 | 0.911 |
| exact mix C3 only +IDF | 0.988 | 0.989 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.999 |
| exact mix C4 only +IDF | 0.587 | 0.589 | 0.962 | 0.638 | 0.637 | 0.481 | 0.382 | 0.614 | 0.528 |
| exact mix C2+C3 +IDF | 0.986 | 0.986 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.999 |
| exact mix C2+C3+C4 +IDF | 0.962 | 0.962 | 1.000 | 0.999 | 0.999 | 0.998 | 0.997 | 1.000 | 0.997 |
| exact mix C1+C2+C3 +IDF | 0.975 | 0.975 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.999 |
| ablation: no taxonomy | 0.973 | 0.974 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.999 |
| ablation: no systematicity (beta=0) | 0.951 | 0.952 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.999 |
| ablation: no parent-child | 0.710 | 0.713 | 1.000 | 0.984 | 0.987 | 0.974 | 0.977 | 0.970 | 0.976 |
| ablation: no co-entity | 0.981 | 0.981 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.999 |
| ablation: co-entity weight 1.0 | 0.914 | 0.916 | 1.000 | 0.998 | 1.000 | 0.998 | 0.997 | 1.000 | 0.999 |
| ablation: co-entity weight 0.5 | 0.948 | 0.949 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.999 |
| ablation: no co-expr | 0.971 | 0.972 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.999 |
| ablation: WL depth 1 | 0.974 | 0.974 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.999 |
| ablation: WL depth 3 | 0.964 | 0.965 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.999 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 0.993 | 0.993 |
| loop | 0.000 | 1.000 | 0.986 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.203 ± 0.054 | 0.100 ± 0.035 | 0.372 ± 0.031 | 0.404 ± 0.062 |
| TA | 0.500 ± 0.015 | 0.202 ± 0.055 | 0.115 ± 0.041 | 0.384 ± 0.028 | 0.422 ± 0.046 |
| MA | 0.108 ± 0.054 | 0.204 ± 0.054 | 0.336 ± 0.111 | 0.470 ± 0.026 | 0.434 ± 0.041 |
| FOR | 0.500 ± 0.016 | 0.204 ± 0.054 | 0.336 ± 0.111 | 0.469 ± 0.025 | 0.435 ± 0.041 |
| RND | 0.496 ± 0.023 | 0.454 ± 0.040 | 0.495 ± 0.014 | 0.499 ± 0.012 | 0.483 ± 0.021 |
