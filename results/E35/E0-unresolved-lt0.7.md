# E0 — fingerprint separability (lt0.7)

Config: groups=300, seed=1, naming=Unresolved, distractors=2, cases=2400, mean features/case=161.9, runtime=6.2s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 300/300.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.496 | 0.794 | 0.000 | 0.410 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.000 | 0.498 | 0.809 | 0.000 | 0.472 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.456 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.089 | 0.489 | 0.825 | 0.000 | 0.460 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C2 relational | 0.265 | 0.522 | 0.885 | 0.103 | 0.535 | 0.102 | 0.032 | 0.195 | 0.102 |
| exact C3 WL | 0.170 | 0.510 | 0.627 | 0.112 | 0.557 | 0.103 | 0.041 | 0.188 | 0.103 |
| exact C4 topology | 0.608 | 0.617 | 0.975 | 0.638 | 0.603 | 0.482 | 0.398 | 0.594 | 0.482 |
| exact analogy profile | 0.205 | 0.518 | 0.956 | 0.110 | 0.528 | 0.107 | 0.029 | 0.211 | 0.107 |
| exact analogy profile +IDF | 0.141 | 0.526 | 0.947 | 0.073 | 0.573 | 0.073 | 0.023 | 0.141 | 0.073 |
| exact literal profile +IDF | 0.000 | 0.515 | 0.882 | 0.000 | 0.557 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.168 | 0.541 | 0.809 | 0.107 | 0.580 | 0.090 | 0.035 | 0.164 | 0.090 |
| fingerprint analogy +IDF D=2048 | 0.155 | 0.532 | 0.845 | 0.087 | 0.620 | 0.087 | 0.035 | 0.156 | 0.087 |
| fingerprint analogy +IDF D=4096 | 0.149 | 0.529 | 0.862 | 0.093 | 0.607 | 0.093 | 0.035 | 0.172 | 0.093 |
| fingerprint analogy +IDF D=8192 | 0.142 | 0.529 | 0.905 | 0.077 | 0.587 | 0.077 | 0.023 | 0.148 | 0.077 |
| fingerprint analogy +IDF D=16384 | 0.141 | 0.526 | 0.917 | 0.077 | 0.587 | 0.077 | 0.023 | 0.148 | 0.077 |
| fingerprint C1 only D=8192 | 0.066 | 0.483 | 0.743 | 0.003 | 0.443 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint C2 only D=8192 | 0.199 | 0.514 | 0.866 | 0.110 | 0.543 | 0.097 | 0.041 | 0.172 | 0.097 |
| fingerprint C3 only D=8192 | 0.178 | 0.524 | 0.605 | 0.152 | 0.535 | 0.113 | 0.064 | 0.180 | 0.113 |
| fingerprint C4 only D=8192 | 0.614 | 0.627 | 0.956 | 0.603 | 0.682 | 0.510 | 0.410 | 0.645 | 0.510 |
| exact mix C2 only +IDF | 0.194 | 0.519 | 0.883 | 0.097 | 0.530 | 0.097 | 0.035 | 0.180 | 0.097 |
| exact mix C3 only +IDF | 0.132 | 0.510 | 0.630 | 0.097 | 0.557 | 0.092 | 0.044 | 0.156 | 0.092 |
| exact mix C4 only +IDF | 0.601 | 0.609 | 0.973 | 0.633 | 0.668 | 0.507 | 0.407 | 0.641 | 0.507 |
| exact mix C2+C3 +IDF | 0.164 | 0.515 | 0.892 | 0.090 | 0.528 | 0.090 | 0.029 | 0.172 | 0.090 |
| exact mix C2+C3+C4 +IDF | 0.214 | 0.545 | 0.971 | 0.123 | 0.612 | 0.123 | 0.058 | 0.211 | 0.123 |
| exact mix C1+C2+C3 +IDF | 0.113 | 0.503 | 0.876 | 0.053 | 0.497 | 0.053 | 0.012 | 0.109 | 0.053 |
| ablation: no taxonomy | 0.152 | 0.525 | 0.953 | 0.077 | 0.570 | 0.077 | 0.023 | 0.148 | 0.077 |
| ablation: no systematicity (beta=0) | 0.135 | 0.524 | 0.952 | 0.070 | 0.577 | 0.070 | 0.023 | 0.133 | 0.070 |
| ablation: no parent-child | 0.133 | 0.517 | 0.945 | 0.013 | 0.567 | 0.013 | 0.006 | 0.023 | 0.013 |
| ablation: no co-entity | 0.157 | 0.525 | 0.948 | 0.073 | 0.573 | 0.073 | 0.023 | 0.141 | 0.073 |
| ablation: co-entity weight 1.0 | 0.076 | 0.524 | 0.943 | 0.033 | 0.567 | 0.033 | 0.012 | 0.062 | 0.033 |
| ablation: co-entity weight 0.5 | 0.116 | 0.525 | 0.946 | 0.067 | 0.577 | 0.067 | 0.017 | 0.133 | 0.067 |
| ablation: no co-expr | 0.136 | 0.525 | 0.929 | 0.073 | 0.590 | 0.073 | 0.023 | 0.141 | 0.073 |
| ablation: WL depth 1 | 0.138 | 0.524 | 0.945 | 0.070 | 0.563 | 0.070 | 0.023 | 0.133 | 0.070 |
| ablation: WL depth 3 | 0.143 | 0.526 | 0.948 | 0.073 | 0.573 | 0.073 | 0.023 | 0.141 | 0.073 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.070 | 0.070 |
| star | 0.000 | 0.000 | 0.000 |
| tree | 0.000 | 0.023 | 0.023 |
| loop | 0.000 | 0.000 | 0.000 |
| deep-ho (test) | 0.000 | 0.023 | 0.047 |
| comparison (test) | 0.000 | 0.372 | 0.372 |
| mixed (test) | 0.000 | 0.024 | 0.024 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.175 ± 0.032 | 0.214 ± 0.055 | 0.095 ± 0.027 | 0.359 ± 0.031 | 0.400 ± 0.057 |
| TA | 0.498 ± 0.016 | 0.425 ± 0.089 | 0.437 ± 0.073 | 0.488 ± 0.026 | 0.395 ± 0.068 |
| MA | 0.177 ± 0.031 | 0.219 ± 0.052 | 0.332 ± 0.112 | 0.455 ± 0.034 | 0.417 ± 0.057 |
| FOR | 0.499 ± 0.016 | 0.422 ± 0.091 | 0.446 ± 0.059 | 0.492 ± 0.019 | 0.420 ± 0.054 |
| RND | 0.497 ± 0.022 | 0.480 ± 0.030 | 0.498 ± 0.012 | 0.499 ± 0.011 | 0.482 ± 0.021 |
