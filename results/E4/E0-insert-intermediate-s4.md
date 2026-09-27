# E0 — fingerprint separability (insert-intermediate-s4)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=177.8, runtime=1.8s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 294/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.502 | 0.948 | 0.000 | 0.499 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.001 | 0.502 | 0.956 | 0.000 | 0.505 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.467 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.501 | 0.503 | 0.996 | 0.500 | 0.490 | 0.329 | 0.349 | 0.303 | 0.374 |
| exact C2 relational | 0.923 | 0.923 | 1.000 | 0.955 | 0.952 | 0.939 | 0.899 | 0.993 | 0.935 |
| exact C3 WL | 0.952 | 0.952 | 1.000 | 0.998 | 0.997 | 0.996 | 0.995 | 0.998 | 0.997 |
| exact C4 topology | 0.572 | 0.574 | 0.969 | 0.616 | 0.614 | 0.475 | 0.410 | 0.562 | 0.578 |
| exact analogy profile | 0.955 | 0.955 | 1.000 | 0.999 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| exact analogy profile +IDF | 0.962 | 0.961 | 1.000 | 0.999 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| exact literal profile +IDF | 0.006 | 0.937 | 1.000 | 0.000 | 0.999 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.944 | 0.942 | 1.000 | 0.988 | 0.989 | 0.983 | 0.969 | 1.000 | 0.978 |
| fingerprint analogy +IDF D=2048 | 0.948 | 0.947 | 1.000 | 0.995 | 0.993 | 0.991 | 0.986 | 0.998 | 0.986 |
| fingerprint analogy +IDF D=4096 | 0.950 | 0.949 | 1.000 | 0.997 | 0.995 | 0.995 | 0.993 | 0.998 | 0.993 |
| fingerprint analogy +IDF D=8192 | 0.952 | 0.951 | 1.000 | 0.996 | 0.998 | 0.995 | 0.991 | 1.000 | 0.997 |
| fingerprint analogy +IDF D=16384 | 0.952 | 0.951 | 1.000 | 0.997 | 0.999 | 0.997 | 0.995 | 1.000 | 0.997 |
| fingerprint C1 only D=8192 | 0.500 | 0.502 | 0.998 | 0.489 | 0.482 | 0.324 | 0.318 | 0.332 | 0.321 |
| fingerprint C2 only D=8192 | 0.936 | 0.935 | 1.000 | 0.960 | 0.958 | 0.945 | 0.908 | 0.993 | 0.929 |
| fingerprint C3 only D=8192 | 0.976 | 0.975 | 1.000 | 0.995 | 0.995 | 0.994 | 0.997 | 0.991 | 1.000 |
| fingerprint C4 only D=8192 | 0.601 | 0.597 | 0.954 | 0.647 | 0.631 | 0.492 | 0.415 | 0.596 | 0.592 |
| exact mix C2 only +IDF | 0.938 | 0.937 | 1.000 | 0.954 | 0.954 | 0.940 | 0.900 | 0.993 | 0.942 |
| exact mix C3 only +IDF | 0.985 | 0.984 | 1.000 | 0.999 | 0.997 | 0.997 | 0.997 | 0.998 | 0.997 |
| exact mix C4 only +IDF | 0.605 | 0.604 | 0.980 | 0.666 | 0.674 | 0.527 | 0.442 | 0.640 | 0.660 |
| exact mix C2+C3 +IDF | 0.981 | 0.981 | 1.000 | 0.999 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| exact mix C2+C3+C4 +IDF | 0.957 | 0.955 | 1.000 | 0.999 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| exact mix C1+C2+C3 +IDF | 0.962 | 0.962 | 1.000 | 0.999 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| ablation: no taxonomy | 0.966 | 0.965 | 1.000 | 0.999 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| ablation: no systematicity (beta=0) | 0.939 | 0.937 | 1.000 | 0.999 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| ablation: no parent-child | 0.685 | 0.684 | 0.999 | 0.972 | 0.962 | 0.940 | 0.951 | 0.925 | 0.976 |
| ablation: no co-entity | 0.978 | 0.977 | 1.000 | 0.999 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| ablation: co-entity weight 1.0 | 0.897 | 0.894 | 1.000 | 0.995 | 0.992 | 0.989 | 0.988 | 0.991 | 0.993 |
| ablation: co-entity weight 0.5 | 0.935 | 0.933 | 1.000 | 0.998 | 0.998 | 0.997 | 0.997 | 0.998 | 0.997 |
| ablation: no co-expr | 0.964 | 0.963 | 1.000 | 0.999 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| ablation: WL depth 1 | 0.965 | 0.964 | 1.000 | 0.999 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |
| ablation: WL depth 3 | 0.957 | 0.955 | 1.000 | 0.999 | 0.999 | 0.999 | 0.998 | 1.000 | 1.000 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 0.993 | 0.993 |
| loop | 0.000 | 1.000 | 0.972 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.199 ± 0.055 | 0.099 ± 0.035 | 0.372 ± 0.031 | 0.404 ± 0.063 |
| TA | 0.500 ± 0.015 | 0.252 ± 0.060 | 0.126 ± 0.043 | 0.400 ± 0.026 | 0.412 ± 0.045 |
| MA | 0.204 ± 0.035 | 0.252 ± 0.059 | 0.340 ± 0.109 | 0.472 ± 0.023 | 0.426 ± 0.040 |
| FOR | 0.502 ± 0.016 | 0.253 ± 0.059 | 0.339 ± 0.109 | 0.472 ± 0.023 | 0.425 ± 0.041 |
| RND | 0.497 ± 0.022 | 0.453 ± 0.041 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.484 ± 0.021 |
