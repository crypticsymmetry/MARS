# E0 — fingerprint separability (canonical-d5)

Config: groups=1000, seed=1, naming=Canonical, distractors=5, cases=8000, mean features/case=204.2, runtime=1.9s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) |
|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.502 | 0.949 | 0.000 | 0.514 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.020 | 0.503 | 0.980 | 0.000 | 0.499 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.465 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.499 | 0.501 | 0.995 | 0.492 | 0.499 | 0.335 | 0.341 | 0.327 |
| exact C2 relational | 0.917 | 0.915 | 1.000 | 0.957 | 0.956 | 0.940 | 0.899 | 0.995 |
| exact C3 WL | 0.945 | 0.944 | 1.000 | 0.999 | 0.997 | 0.997 | 0.997 | 0.998 |
| exact C4 topology | 0.590 | 0.580 | 0.963 | 0.595 | 0.592 | 0.446 | 0.327 | 0.605 |
| exact analogy profile | 0.937 | 0.936 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact analogy profile +IDF | 0.954 | 0.952 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact literal profile +IDF | 0.000 | 0.921 | 1.000 | 0.000 | 1.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.938 | 0.933 | 1.000 | 0.988 | 0.986 | 0.978 | 0.963 | 0.998 |
| fingerprint analogy +IDF D=2048 | 0.944 | 0.940 | 1.000 | 0.993 | 0.993 | 0.988 | 0.981 | 0.998 |
| fingerprint analogy +IDF D=4096 | 0.946 | 0.942 | 1.000 | 0.997 | 0.996 | 0.993 | 0.988 | 1.000 |
| fingerprint analogy +IDF D=8192 | 0.947 | 0.944 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 |
| fingerprint analogy +IDF D=16384 | 0.947 | 0.945 | 1.000 | 0.999 | 0.999 | 0.998 | 0.998 | 0.998 |
| fingerprint C1 only D=8192 | 0.495 | 0.498 | 0.995 | 0.486 | 0.498 | 0.318 | 0.309 | 0.329 |
| fingerprint C2 only D=8192 | 0.938 | 0.936 | 1.000 | 0.965 | 0.951 | 0.940 | 0.898 | 0.995 |
| fingerprint C3 only D=8192 | 0.974 | 0.973 | 0.999 | 0.996 | 0.995 | 0.992 | 0.991 | 0.993 |
| fingerprint C4 only D=8192 | 0.564 | 0.555 | 0.877 | 0.594 | 0.571 | 0.386 | 0.283 | 0.522 |
| exact mix C2 only +IDF | 0.937 | 0.935 | 1.000 | 0.958 | 0.954 | 0.942 | 0.900 | 0.998 |
| exact mix C3 only +IDF | 0.985 | 0.984 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact mix C4 only +IDF | 0.572 | 0.572 | 0.963 | 0.608 | 0.614 | 0.440 | 0.341 | 0.572 |
| exact mix C2+C3 +IDF | 0.974 | 0.973 | 1.000 | 0.999 | 1.000 | 0.999 | 0.998 | 1.000 |
| exact mix C2+C3+C4 +IDF | 0.954 | 0.953 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact mix C1+C2+C3 +IDF | 0.948 | 0.947 | 1.000 | 0.999 | 0.999 | 0.999 | 0.998 | 1.000 |
| ablation: no taxonomy | 0.959 | 0.958 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: no systematicity (beta=0) | 0.923 | 0.921 | 1.000 | 0.999 | 0.999 | 0.998 | 0.998 | 0.998 |
| ablation: no parent-child | 0.694 | 0.695 | 1.000 | 0.988 | 0.982 | 0.973 | 0.970 | 0.977 |
| ablation: no co-entity | 0.977 | 0.977 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: co-entity weight 1.0 | 0.884 | 0.882 | 1.000 | 0.998 | 0.996 | 0.995 | 0.995 | 0.995 |
| ablation: co-entity weight 0.5 | 0.919 | 0.918 | 1.000 | 0.999 | 0.999 | 0.998 | 0.998 | 0.998 |
| ablation: no co-expr | 0.958 | 0.957 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: WL depth 1 | 0.957 | 0.956 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: WL depth 3 | 0.949 | 0.948 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 1.000 | 0.993 |
| loop | 0.000 | 1.000 | 1.000 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.125 ± 0.032 | 0.267 ± 0.064 | 0.153 ± 0.043 | 0.412 ± 0.024 | 0.438 ± 0.034 |
| TA | 0.500 ± 0.015 | 0.267 ± 0.065 | 0.152 ± 0.043 | 0.413 ± 0.024 | 0.438 ± 0.036 |
| MA | 0.125 ± 0.031 | 0.266 ± 0.064 | 0.346 ± 0.100 | 0.479 ± 0.021 | 0.446 ± 0.032 |
| FOR | 0.500 ± 0.016 | 0.266 ± 0.064 | 0.346 ± 0.101 | 0.478 ± 0.021 | 0.444 ± 0.033 |
| RND | 0.497 ± 0.022 | 0.444 ± 0.039 | 0.496 ± 0.015 | 0.498 ± 0.011 | 0.480 ± 0.018 |
