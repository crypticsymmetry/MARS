# E0 — fingerprint separability (swap-args-s2)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=153.8, runtime=1.5s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 542/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.499 | 0.954 | 0.000 | 0.493 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.000 | 0.500 | 0.989 | 0.000 | 0.492 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.462 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.496 | 0.502 | 1.000 | 0.507 | 0.519 | 0.346 | 0.337 | 0.357 | 0.348 |
| exact C2 relational | 0.926 | 0.926 | 1.000 | 0.950 | 0.953 | 0.941 | 0.901 | 0.993 | 0.958 |
| exact C3 WL | 0.962 | 0.961 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| exact C4 topology | 0.587 | 0.590 | 0.968 | 0.584 | 0.577 | 0.437 | 0.312 | 0.604 | 0.521 |
| exact analogy profile | 0.959 | 0.961 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact analogy profile +IDF | 0.967 | 0.968 | 1.000 | 0.996 | 1.000 | 0.996 | 0.993 | 1.000 | 1.000 |
| exact literal profile +IDF | 0.000 | 0.954 | 1.000 | 0.000 | 1.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.950 | 0.949 | 1.000 | 0.988 | 0.989 | 0.983 | 0.970 | 1.000 | 0.993 |
| fingerprint analogy +IDF D=2048 | 0.953 | 0.953 | 1.000 | 0.995 | 0.993 | 0.990 | 0.984 | 0.998 | 0.996 |
| fingerprint analogy +IDF D=4096 | 0.955 | 0.955 | 1.000 | 0.994 | 0.995 | 0.991 | 0.984 | 1.000 | 0.996 |
| fingerprint analogy +IDF D=8192 | 0.956 | 0.957 | 1.000 | 0.994 | 0.997 | 0.992 | 0.986 | 1.000 | 0.996 |
| fingerprint analogy +IDF D=16384 | 0.956 | 0.956 | 1.000 | 0.994 | 0.996 | 0.990 | 0.983 | 1.000 | 0.998 |
| fingerprint C1 only D=8192 | 0.497 | 0.502 | 1.000 | 0.506 | 0.488 | 0.333 | 0.323 | 0.345 | 0.335 |
| fingerprint C2 only D=8192 | 0.936 | 0.936 | 1.000 | 0.955 | 0.949 | 0.939 | 0.897 | 0.995 | 0.952 |
| fingerprint C3 only D=8192 | 0.986 | 0.986 | 1.000 | 0.998 | 0.996 | 0.995 | 0.993 | 0.998 | 0.996 |
| fingerprint C4 only D=8192 | 0.574 | 0.580 | 0.918 | 0.582 | 0.610 | 0.419 | 0.303 | 0.574 | 0.511 |
| exact mix C2 only +IDF | 0.939 | 0.940 | 1.000 | 0.952 | 0.953 | 0.940 | 0.900 | 0.993 | 0.958 |
| exact mix C3 only +IDF | 0.988 | 0.989 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| exact mix C4 only +IDF | 0.580 | 0.582 | 0.960 | 0.614 | 0.610 | 0.464 | 0.348 | 0.619 | 0.572 |
| exact mix C2+C3 +IDF | 0.984 | 0.985 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact mix C2+C3+C4 +IDF | 0.959 | 0.960 | 1.000 | 0.996 | 0.996 | 0.992 | 0.986 | 1.000 | 0.996 |
| exact mix C1+C2+C3 +IDF | 0.973 | 0.975 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: no taxonomy | 0.971 | 0.973 | 1.000 | 0.997 | 1.000 | 0.997 | 0.995 | 1.000 | 1.000 |
| ablation: no systematicity (beta=0) | 0.948 | 0.949 | 1.000 | 0.996 | 0.998 | 0.994 | 0.990 | 1.000 | 1.000 |
| ablation: no parent-child | 0.706 | 0.702 | 0.999 | 0.922 | 0.924 | 0.864 | 0.874 | 0.850 | 0.958 |
| ablation: no co-entity | 0.981 | 0.982 | 1.000 | 0.998 | 1.000 | 0.998 | 0.997 | 1.000 | 1.000 |
| ablation: co-entity weight 1.0 | 0.900 | 0.898 | 1.000 | 0.986 | 0.975 | 0.966 | 0.953 | 0.984 | 0.985 |
| ablation: co-entity weight 0.5 | 0.942 | 0.943 | 1.000 | 0.995 | 0.995 | 0.990 | 0.984 | 0.998 | 0.996 |
| ablation: no co-expr | 0.969 | 0.970 | 1.000 | 0.996 | 0.999 | 0.995 | 0.991 | 1.000 | 1.000 |
| ablation: WL depth 1 | 0.972 | 0.973 | 1.000 | 0.998 | 1.000 | 0.998 | 0.997 | 1.000 | 1.000 |
| ablation: WL depth 3 | 0.962 | 0.963 | 1.000 | 0.996 | 0.999 | 0.995 | 0.991 | 1.000 | 1.000 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 1.000 | 1.000 |
| loop | 0.000 | 0.972 | 0.944 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.202 ± 0.054 | 0.100 ± 0.035 | 0.372 ± 0.031 | 0.403 ± 0.062 |
| TA | 0.500 ± 0.016 | 0.203 ± 0.054 | 0.117 ± 0.044 | 0.384 ± 0.029 | 0.418 ± 0.054 |
| MA | 0.107 ± 0.053 | 0.203 ± 0.054 | 0.338 ± 0.112 | 0.471 ± 0.025 | 0.431 ± 0.047 |
| FOR | 0.500 ± 0.015 | 0.203 ± 0.055 | 0.337 ± 0.112 | 0.470 ± 0.025 | 0.432 ± 0.047 |
| RND | 0.496 ± 0.022 | 0.453 ± 0.040 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.021 |
