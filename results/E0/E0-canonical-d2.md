# E0 — fingerprint separability (canonical-d2)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=153.8, runtime=1.4s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) |
|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.500 | 0.957 | 0.000 | 0.490 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.000 | 0.503 | 0.988 | 0.000 | 0.512 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.462 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.501 | 0.507 | 1.000 | 0.486 | 0.507 | 0.342 | 0.327 | 0.361 |
| exact C2 relational | 0.928 | 0.928 | 1.000 | 0.961 | 0.956 | 0.944 | 0.904 | 0.998 |
| exact C3 WL | 0.963 | 0.967 | 1.000 | 0.998 | 0.999 | 0.997 | 0.996 | 0.998 |
| exact C4 topology | 0.599 | 0.601 | 0.972 | 0.618 | 0.588 | 0.475 | 0.384 | 0.598 |
| exact analogy profile | 0.964 | 0.966 | 1.000 | 0.997 | 1.000 | 0.997 | 0.995 | 1.000 |
| exact analogy profile +IDF | 0.968 | 0.970 | 1.000 | 0.997 | 0.999 | 0.997 | 0.995 | 1.000 |
| exact literal profile +IDF | 0.000 | 0.957 | 1.000 | 0.000 | 0.999 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.953 | 0.957 | 1.000 | 0.991 | 0.991 | 0.987 | 0.979 | 0.998 |
| fingerprint analogy +IDF D=2048 | 0.957 | 0.959 | 1.000 | 0.997 | 0.996 | 0.994 | 0.990 | 1.000 |
| fingerprint analogy +IDF D=4096 | 0.960 | 0.960 | 1.000 | 0.999 | 0.998 | 0.997 | 0.995 | 1.000 |
| fingerprint analogy +IDF D=8192 | 0.960 | 0.961 | 1.000 | 0.998 | 0.998 | 0.996 | 0.993 | 1.000 |
| fingerprint analogy +IDF D=16384 | 0.959 | 0.961 | 1.000 | 0.997 | 0.998 | 0.996 | 0.993 | 1.000 |
| fingerprint C1 only D=8192 | 0.502 | 0.503 | 1.000 | 0.499 | 0.524 | 0.345 | 0.332 | 0.363 |
| fingerprint C2 only D=8192 | 0.943 | 0.943 | 1.000 | 0.953 | 0.949 | 0.934 | 0.891 | 0.993 |
| fingerprint C3 only D=8192 | 0.982 | 0.985 | 1.000 | 0.996 | 0.998 | 0.994 | 0.990 | 1.000 |
| fingerprint C4 only D=8192 | 0.601 | 0.613 | 0.936 | 0.624 | 0.654 | 0.489 | 0.413 | 0.592 |
| exact mix C2 only +IDF | 0.944 | 0.943 | 1.000 | 0.965 | 0.952 | 0.944 | 0.904 | 0.998 |
| exact mix C3 only +IDF | 0.985 | 0.988 | 1.000 | 0.995 | 0.999 | 0.994 | 0.993 | 0.995 |
| exact mix C4 only +IDF | 0.599 | 0.608 | 0.970 | 0.655 | 0.680 | 0.524 | 0.439 | 0.638 |
| exact mix C2+C3 +IDF | 0.987 | 0.989 | 1.000 | 0.997 | 1.000 | 0.997 | 0.995 | 1.000 |
| exact mix C2+C3+C4 +IDF | 0.956 | 0.958 | 1.000 | 0.996 | 0.999 | 0.996 | 0.993 | 1.000 |
| exact mix C1+C2+C3 +IDF | 0.977 | 0.979 | 1.000 | 0.998 | 1.000 | 0.998 | 0.997 | 1.000 |
| ablation: no taxonomy | 0.972 | 0.974 | 1.000 | 0.997 | 0.999 | 0.997 | 0.995 | 1.000 |
| ablation: no systematicity (beta=0) | 0.954 | 0.957 | 1.000 | 0.997 | 0.999 | 0.997 | 0.995 | 1.000 |
| ablation: no parent-child | 0.740 | 0.745 | 1.000 | 0.979 | 0.989 | 0.973 | 0.969 | 0.979 |
| ablation: no co-entity | 0.976 | 0.979 | 1.000 | 0.998 | 0.999 | 0.997 | 0.995 | 1.000 |
| ablation: co-entity weight 1.0 | 0.926 | 0.930 | 1.000 | 0.996 | 0.998 | 0.995 | 0.993 | 0.998 |
| ablation: co-entity weight 0.5 | 0.953 | 0.956 | 1.000 | 0.997 | 0.999 | 0.997 | 0.995 | 1.000 |
| ablation: no co-expr | 0.970 | 0.971 | 1.000 | 0.997 | 0.999 | 0.997 | 0.995 | 1.000 |
| ablation: WL depth 1 | 0.971 | 0.974 | 1.000 | 0.998 | 0.999 | 0.997 | 0.995 | 1.000 |
| ablation: WL depth 3 | 0.963 | 0.965 | 1.000 | 0.997 | 0.999 | 0.997 | 0.995 | 1.000 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 0.986 | 0.986 |
| loop | 0.000 | 0.993 | 0.986 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.203 ± 0.054 | 0.100 ± 0.035 | 0.372 ± 0.031 | 0.403 ± 0.063 |
| TA | 0.501 ± 0.015 | 0.202 ± 0.054 | 0.100 ± 0.036 | 0.375 ± 0.032 | 0.400 ± 0.071 |
| MA | 0.107 ± 0.055 | 0.203 ± 0.055 | 0.330 ± 0.111 | 0.464 ± 0.026 | 0.421 ± 0.057 |
| FOR | 0.500 ± 0.016 | 0.203 ± 0.053 | 0.330 ± 0.112 | 0.464 ± 0.025 | 0.423 ± 0.056 |
| RND | 0.496 ± 0.023 | 0.454 ± 0.039 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.021 |
