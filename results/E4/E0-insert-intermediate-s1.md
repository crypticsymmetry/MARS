# E0 — fingerprint separability (insert-intermediate-s1)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=160.2, runtime=1.5s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 663/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.502 | 0.951 | 0.000 | 0.504 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.001 | 0.502 | 0.982 | 0.000 | 0.517 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.458 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.507 | 0.501 | 1.000 | 0.519 | 0.493 | 0.342 | 0.362 | 0.315 | 0.361 |
| exact C2 relational | 0.925 | 0.926 | 1.000 | 0.954 | 0.951 | 0.938 | 0.894 | 0.995 | 0.906 |
| exact C3 WL | 0.960 | 0.962 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact C4 topology | 0.607 | 0.599 | 0.980 | 0.624 | 0.597 | 0.468 | 0.380 | 0.584 | 0.483 |
| exact analogy profile | 0.960 | 0.960 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact analogy profile +IDF | 0.966 | 0.966 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact literal profile +IDF | 0.001 | 0.950 | 1.000 | 0.000 | 1.000 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.948 | 0.949 | 1.000 | 0.991 | 0.990 | 0.985 | 0.974 | 1.000 | 0.977 |
| fingerprint analogy +IDF D=2048 | 0.952 | 0.953 | 1.000 | 0.995 | 0.994 | 0.992 | 0.986 | 1.000 | 0.988 |
| fingerprint analogy +IDF D=4096 | 0.955 | 0.955 | 1.000 | 0.998 | 0.997 | 0.996 | 0.993 | 1.000 | 0.994 |
| fingerprint analogy +IDF D=8192 | 0.956 | 0.956 | 1.000 | 0.998 | 0.996 | 0.995 | 0.991 | 1.000 | 0.992 |
| fingerprint analogy +IDF D=16384 | 0.956 | 0.956 | 1.000 | 0.999 | 0.998 | 0.998 | 0.997 | 1.000 | 0.997 |
| fingerprint C1 only D=8192 | 0.513 | 0.505 | 1.000 | 0.529 | 0.506 | 0.346 | 0.342 | 0.352 | 0.339 |
| fingerprint C2 only D=8192 | 0.937 | 0.939 | 1.000 | 0.947 | 0.949 | 0.933 | 0.886 | 0.995 | 0.899 |
| fingerprint C3 only D=8192 | 0.981 | 0.983 | 1.000 | 0.998 | 0.997 | 0.995 | 0.997 | 0.993 | 0.997 |
| fingerprint C4 only D=8192 | 0.594 | 0.598 | 0.936 | 0.617 | 0.641 | 0.451 | 0.365 | 0.565 | 0.480 |
| exact mix C2 only +IDF | 0.939 | 0.940 | 1.000 | 0.948 | 0.949 | 0.934 | 0.888 | 0.995 | 0.900 |
| exact mix C3 only +IDF | 0.986 | 0.987 | 1.000 | 0.999 | 1.000 | 0.999 | 1.000 | 0.998 | 0.998 |
| exact mix C4 only +IDF | 0.610 | 0.609 | 0.974 | 0.669 | 0.663 | 0.524 | 0.437 | 0.640 | 0.558 |
| exact mix C2+C3 +IDF | 0.985 | 0.985 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact mix C2+C3+C4 +IDF | 0.957 | 0.957 | 1.000 | 0.999 | 0.999 | 0.999 | 0.998 | 1.000 | 0.998 |
| exact mix C1+C2+C3 +IDF | 0.973 | 0.973 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: no taxonomy | 0.970 | 0.970 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: no systematicity (beta=0) | 0.949 | 0.949 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: no parent-child | 0.716 | 0.712 | 1.000 | 0.982 | 0.972 | 0.958 | 0.953 | 0.965 | 0.967 |
| ablation: no co-entity | 0.977 | 0.977 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: co-entity weight 1.0 | 0.914 | 0.914 | 1.000 | 0.996 | 0.998 | 0.994 | 0.993 | 0.995 | 0.992 |
| ablation: co-entity weight 0.5 | 0.947 | 0.947 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.998 |
| ablation: no co-expr | 0.968 | 0.968 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: WL depth 1 | 0.971 | 0.971 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: WL depth 3 | 0.961 | 0.961 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.998 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 1.000 | 1.000 |
| loop | 0.000 | 1.000 | 0.965 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.202 ± 0.054 | 0.100 ± 0.035 | 0.372 ± 0.031 | 0.403 ± 0.063 |
| TA | 0.500 ± 0.016 | 0.225 ± 0.056 | 0.112 ± 0.039 | 0.384 ± 0.030 | 0.408 ± 0.055 |
| MA | 0.144 ± 0.039 | 0.227 ± 0.057 | 0.335 ± 0.111 | 0.467 ± 0.025 | 0.425 ± 0.049 |
| FOR | 0.500 ± 0.015 | 0.226 ± 0.058 | 0.334 ± 0.111 | 0.467 ± 0.025 | 0.425 ± 0.048 |
| RND | 0.497 ± 0.022 | 0.455 ± 0.039 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.021 |
