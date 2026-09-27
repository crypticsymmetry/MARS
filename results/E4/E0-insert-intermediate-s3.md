# E0 — fingerprint separability (insert-intermediate-s3)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=171.7, runtime=1.7s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 364/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.497 | 0.946 | 0.000 | 0.491 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.002 | 0.496 | 0.969 | 0.000 | 0.488 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.459 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.501 | 0.497 | 0.998 | 0.509 | 0.486 | 0.329 | 0.331 | 0.325 | 0.337 |
| exact C2 relational | 0.923 | 0.922 | 1.000 | 0.955 | 0.953 | 0.938 | 0.895 | 0.994 | 0.924 |
| exact C3 WL | 0.950 | 0.954 | 1.000 | 0.995 | 0.998 | 0.994 | 0.990 | 1.000 | 0.992 |
| exact C4 topology | 0.576 | 0.572 | 0.978 | 0.596 | 0.593 | 0.454 | 0.369 | 0.567 | 0.530 |
| exact analogy profile | 0.953 | 0.956 | 1.000 | 0.998 | 0.998 | 0.997 | 0.997 | 0.998 | 0.992 |
| exact analogy profile +IDF | 0.959 | 0.961 | 1.000 | 0.998 | 0.998 | 0.996 | 0.997 | 0.995 | 0.992 |
| exact literal profile +IDF | 0.003 | 0.938 | 1.000 | 0.000 | 0.998 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.942 | 0.945 | 1.000 | 0.984 | 0.989 | 0.978 | 0.965 | 0.995 | 0.962 |
| fingerprint analogy +IDF D=2048 | 0.945 | 0.948 | 1.000 | 0.993 | 0.991 | 0.986 | 0.979 | 0.995 | 0.973 |
| fingerprint analogy +IDF D=4096 | 0.949 | 0.951 | 1.000 | 0.996 | 0.994 | 0.993 | 0.990 | 0.995 | 0.988 |
| fingerprint analogy +IDF D=8192 | 0.949 | 0.952 | 1.000 | 0.997 | 0.998 | 0.996 | 0.997 | 0.995 | 0.992 |
| fingerprint analogy +IDF D=16384 | 0.949 | 0.951 | 1.000 | 0.998 | 0.998 | 0.997 | 0.998 | 0.995 | 0.995 |
| fingerprint C1 only D=8192 | 0.493 | 0.490 | 0.999 | 0.496 | 0.488 | 0.328 | 0.315 | 0.346 | 0.320 |
| fingerprint C2 only D=8192 | 0.935 | 0.935 | 1.000 | 0.954 | 0.953 | 0.943 | 0.905 | 0.993 | 0.926 |
| fingerprint C3 only D=8192 | 0.974 | 0.977 | 1.000 | 0.991 | 0.994 | 0.987 | 0.988 | 0.986 | 0.995 |
| fingerprint C4 only D=8192 | 0.592 | 0.587 | 0.945 | 0.623 | 0.628 | 0.485 | 0.401 | 0.597 | 0.571 |
| exact mix C2 only +IDF | 0.936 | 0.936 | 1.000 | 0.951 | 0.951 | 0.936 | 0.893 | 0.993 | 0.918 |
| exact mix C3 only +IDF | 0.981 | 0.984 | 1.000 | 0.995 | 0.998 | 0.993 | 0.993 | 0.993 | 0.986 |
| exact mix C4 only +IDF | 0.589 | 0.592 | 0.980 | 0.631 | 0.637 | 0.501 | 0.409 | 0.624 | 0.596 |
| exact mix C2+C3 +IDF | 0.981 | 0.982 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 | 0.997 |
| exact mix C2+C3+C4 +IDF | 0.952 | 0.955 | 1.000 | 0.997 | 0.998 | 0.995 | 0.995 | 0.995 | 0.992 |
| exact mix C1+C2+C3 +IDF | 0.962 | 0.964 | 1.000 | 0.999 | 0.999 | 0.998 | 0.998 | 0.998 | 0.995 |
| ablation: no taxonomy | 0.963 | 0.965 | 1.000 | 0.998 | 0.998 | 0.996 | 0.997 | 0.995 | 0.992 |
| ablation: no systematicity (beta=0) | 0.937 | 0.938 | 1.000 | 0.997 | 0.998 | 0.995 | 0.997 | 0.993 | 0.989 |
| ablation: no parent-child | 0.687 | 0.687 | 0.999 | 0.964 | 0.966 | 0.938 | 0.948 | 0.925 | 0.962 |
| ablation: no co-entity | 0.974 | 0.976 | 1.000 | 0.999 | 0.999 | 0.998 | 0.998 | 0.998 | 0.997 |
| ablation: co-entity weight 1.0 | 0.896 | 0.896 | 1.000 | 0.990 | 0.992 | 0.983 | 0.984 | 0.981 | 0.984 |
| ablation: co-entity weight 0.5 | 0.933 | 0.935 | 1.000 | 0.997 | 0.997 | 0.995 | 0.997 | 0.993 | 0.989 |
| ablation: no co-expr | 0.961 | 0.963 | 1.000 | 0.998 | 0.998 | 0.996 | 0.997 | 0.995 | 0.992 |
| ablation: WL depth 1 | 0.963 | 0.964 | 1.000 | 0.999 | 0.998 | 0.997 | 0.998 | 0.995 | 0.995 |
| ablation: WL depth 3 | 0.954 | 0.955 | 1.000 | 0.998 | 0.998 | 0.996 | 0.997 | 0.995 | 0.992 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 0.986 | 0.986 |
| loop | 0.000 | 1.000 | 1.000 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 0.993 | 0.993 |
| mixed (test) | 0.000 | 0.993 | 0.993 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.200 ± 0.055 | 0.099 ± 0.035 | 0.372 ± 0.031 | 0.403 ± 0.063 |
| TA | 0.501 ± 0.016 | 0.246 ± 0.060 | 0.122 ± 0.042 | 0.396 ± 0.028 | 0.412 ± 0.046 |
| MA | 0.186 ± 0.034 | 0.245 ± 0.059 | 0.338 ± 0.110 | 0.470 ± 0.024 | 0.425 ± 0.043 |
| FOR | 0.500 ± 0.016 | 0.245 ± 0.059 | 0.338 ± 0.110 | 0.470 ± 0.023 | 0.425 ± 0.042 |
| RND | 0.496 ± 0.023 | 0.454 ± 0.040 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.021 |
