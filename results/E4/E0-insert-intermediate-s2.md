# E0 — fingerprint separability (insert-intermediate-s2)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=166.2, runtime=1.5s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 473/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.492 | 0.955 | 0.000 | 0.480 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.001 | 0.495 | 0.972 | 0.000 | 0.473 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.466 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.496 | 0.494 | 0.999 | 0.489 | 0.505 | 0.341 | 0.333 | 0.350 | 0.352 |
| exact C2 relational | 0.924 | 0.925 | 1.000 | 0.952 | 0.962 | 0.940 | 0.899 | 0.995 | 0.926 |
| exact C3 WL | 0.956 | 0.956 | 1.000 | 0.995 | 0.999 | 0.995 | 0.991 | 1.000 | 0.994 |
| exact C4 topology | 0.596 | 0.595 | 0.981 | 0.623 | 0.621 | 0.500 | 0.414 | 0.614 | 0.550 |
| exact analogy profile | 0.958 | 0.958 | 1.000 | 0.998 | 0.999 | 0.998 | 0.997 | 1.000 | 0.996 |
| exact analogy profile +IDF | 0.965 | 0.966 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 0.998 |
| exact literal profile +IDF | 0.002 | 0.946 | 1.000 | 0.000 | 0.997 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.950 | 0.952 | 1.000 | 0.994 | 0.993 | 0.988 | 0.979 | 1.000 | 0.979 |
| fingerprint analogy +IDF D=2048 | 0.952 | 0.953 | 1.000 | 0.996 | 0.993 | 0.989 | 0.981 | 1.000 | 0.987 |
| fingerprint analogy +IDF D=4096 | 0.954 | 0.956 | 1.000 | 0.996 | 0.998 | 0.994 | 0.991 | 0.998 | 0.996 |
| fingerprint analogy +IDF D=8192 | 0.955 | 0.956 | 1.000 | 0.998 | 0.996 | 0.994 | 0.991 | 0.998 | 0.992 |
| fingerprint analogy +IDF D=16384 | 0.954 | 0.956 | 1.000 | 0.998 | 0.998 | 0.996 | 0.995 | 0.998 | 0.994 |
| fingerprint C1 only D=8192 | 0.494 | 0.493 | 0.999 | 0.492 | 0.500 | 0.326 | 0.317 | 0.338 | 0.342 |
| fingerprint C2 only D=8192 | 0.937 | 0.938 | 1.000 | 0.955 | 0.955 | 0.940 | 0.900 | 0.993 | 0.924 |
| fingerprint C3 only D=8192 | 0.981 | 0.978 | 1.000 | 0.996 | 0.995 | 0.991 | 0.988 | 0.995 | 0.994 |
| fingerprint C4 only D=8192 | 0.609 | 0.607 | 0.949 | 0.637 | 0.657 | 0.492 | 0.399 | 0.618 | 0.554 |
| exact mix C2 only +IDF | 0.938 | 0.940 | 1.000 | 0.951 | 0.962 | 0.940 | 0.899 | 0.995 | 0.930 |
| exact mix C3 only +IDF | 0.986 | 0.985 | 1.000 | 0.998 | 0.999 | 0.998 | 0.997 | 1.000 | 0.998 |
| exact mix C4 only +IDF | 0.607 | 0.605 | 0.979 | 0.643 | 0.669 | 0.522 | 0.425 | 0.652 | 0.586 |
| exact mix C2+C3 +IDF | 0.985 | 0.984 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 0.998 |
| exact mix C2+C3+C4 +IDF | 0.958 | 0.959 | 1.000 | 1.000 | 0.998 | 0.998 | 0.997 | 1.000 | 0.996 |
| exact mix C1+C2+C3 +IDF | 0.969 | 0.969 | 1.000 | 0.999 | 0.999 | 0.998 | 0.997 | 1.000 | 0.996 |
| ablation: no taxonomy | 0.969 | 0.969 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 0.998 |
| ablation: no systematicity (beta=0) | 0.945 | 0.946 | 1.000 | 1.000 | 0.998 | 0.998 | 0.997 | 1.000 | 0.996 |
| ablation: no parent-child | 0.699 | 0.698 | 1.000 | 0.969 | 0.957 | 0.933 | 0.934 | 0.932 | 0.939 |
| ablation: no co-entity | 0.979 | 0.978 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 0.998 |
| ablation: co-entity weight 1.0 | 0.906 | 0.906 | 1.000 | 0.994 | 0.993 | 0.987 | 0.984 | 0.991 | 0.985 |
| ablation: co-entity weight 0.5 | 0.942 | 0.943 | 1.000 | 1.000 | 0.998 | 0.998 | 0.997 | 1.000 | 0.996 |
| ablation: no co-expr | 0.967 | 0.968 | 1.000 | 1.000 | 0.998 | 0.998 | 0.997 | 1.000 | 0.996 |
| ablation: WL depth 1 | 0.969 | 0.969 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 | 0.998 |
| ablation: WL depth 3 | 0.959 | 0.960 | 1.000 | 1.000 | 0.998 | 0.998 | 0.997 | 1.000 | 0.996 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 0.993 | 1.000 |
| loop | 0.000 | 1.000 | 0.965 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 0.993 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.201 ± 0.054 | 0.099 ± 0.035 | 0.372 ± 0.031 | 0.403 ± 0.063 |
| TA | 0.500 ± 0.015 | 0.234 ± 0.059 | 0.118 ± 0.040 | 0.390 ± 0.028 | 0.410 ± 0.049 |
| MA | 0.168 ± 0.038 | 0.233 ± 0.058 | 0.337 ± 0.110 | 0.469 ± 0.024 | 0.426 ± 0.046 |
| FOR | 0.501 ± 0.016 | 0.233 ± 0.058 | 0.337 ± 0.110 | 0.469 ± 0.024 | 0.426 ± 0.046 |
| RND | 0.497 ± 0.022 | 0.454 ± 0.039 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.021 |
