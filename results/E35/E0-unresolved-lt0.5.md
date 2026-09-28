# E0 — fingerprint separability (lt0.5)

Config: groups=300, seed=1, naming=Unresolved, distractors=2, cases=2400, mean features/case=158.3, runtime=4.0s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 300/300.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.496 | 0.794 | 0.000 | 0.410 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.000 | 0.498 | 0.809 | 0.000 | 0.472 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.456 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.090 | 0.489 | 0.806 | 0.023 | 0.520 | 0.022 | 0.000 | 0.051 | 0.022 |
| exact C2 relational | 0.277 | 0.524 | 0.868 | 0.143 | 0.558 | 0.137 | 0.023 | 0.289 | 0.137 |
| exact C3 WL | 0.188 | 0.519 | 0.671 | 0.168 | 0.572 | 0.140 | 0.041 | 0.273 | 0.140 |
| exact C4 topology | 0.608 | 0.617 | 0.975 | 0.638 | 0.603 | 0.482 | 0.398 | 0.594 | 0.482 |
| exact analogy profile | 0.218 | 0.521 | 0.933 | 0.137 | 0.600 | 0.133 | 0.023 | 0.281 | 0.133 |
| exact analogy profile +IDF | 0.201 | 0.522 | 0.920 | 0.127 | 0.580 | 0.127 | 0.023 | 0.266 | 0.127 |
| exact literal profile +IDF | 0.000 | 0.512 | 0.868 | 0.000 | 0.573 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.223 | 0.534 | 0.857 | 0.173 | 0.607 | 0.160 | 0.052 | 0.305 | 0.160 |
| fingerprint analogy +IDF D=2048 | 0.213 | 0.528 | 0.876 | 0.163 | 0.590 | 0.157 | 0.035 | 0.320 | 0.157 |
| fingerprint analogy +IDF D=4096 | 0.206 | 0.528 | 0.894 | 0.150 | 0.583 | 0.143 | 0.035 | 0.289 | 0.143 |
| fingerprint analogy +IDF D=8192 | 0.203 | 0.525 | 0.903 | 0.137 | 0.593 | 0.133 | 0.029 | 0.273 | 0.133 |
| fingerprint analogy +IDF D=16384 | 0.199 | 0.522 | 0.910 | 0.137 | 0.573 | 0.133 | 0.023 | 0.281 | 0.133 |
| fingerprint C1 only D=8192 | 0.115 | 0.488 | 0.782 | 0.027 | 0.495 | 0.020 | 0.000 | 0.047 | 0.020 |
| fingerprint C2 only D=8192 | 0.256 | 0.518 | 0.871 | 0.185 | 0.557 | 0.165 | 0.052 | 0.316 | 0.165 |
| fingerprint C3 only D=8192 | 0.228 | 0.521 | 0.652 | 0.187 | 0.555 | 0.143 | 0.058 | 0.258 | 0.143 |
| fingerprint C4 only D=8192 | 0.614 | 0.627 | 0.956 | 0.603 | 0.682 | 0.510 | 0.410 | 0.645 | 0.510 |
| exact mix C2 only +IDF | 0.251 | 0.518 | 0.868 | 0.167 | 0.537 | 0.160 | 0.047 | 0.312 | 0.160 |
| exact mix C3 only +IDF | 0.164 | 0.519 | 0.692 | 0.132 | 0.563 | 0.127 | 0.029 | 0.258 | 0.127 |
| exact mix C4 only +IDF | 0.601 | 0.609 | 0.973 | 0.633 | 0.668 | 0.507 | 0.407 | 0.641 | 0.507 |
| exact mix C2+C3 +IDF | 0.222 | 0.518 | 0.882 | 0.150 | 0.555 | 0.147 | 0.035 | 0.297 | 0.147 |
| exact mix C2+C3+C4 +IDF | 0.265 | 0.539 | 0.951 | 0.160 | 0.577 | 0.153 | 0.052 | 0.289 | 0.153 |
| exact mix C1+C2+C3 +IDF | 0.163 | 0.507 | 0.874 | 0.113 | 0.563 | 0.113 | 0.017 | 0.242 | 0.113 |
| ablation: no taxonomy | 0.212 | 0.523 | 0.930 | 0.127 | 0.593 | 0.127 | 0.023 | 0.266 | 0.127 |
| ablation: no systematicity (beta=0) | 0.194 | 0.517 | 0.931 | 0.120 | 0.580 | 0.120 | 0.023 | 0.250 | 0.120 |
| ablation: no parent-child | 0.209 | 0.522 | 0.927 | 0.043 | 0.547 | 0.043 | 0.006 | 0.094 | 0.043 |
| ablation: no co-entity | 0.220 | 0.523 | 0.920 | 0.137 | 0.580 | 0.137 | 0.023 | 0.289 | 0.137 |
| ablation: co-entity weight 1.0 | 0.121 | 0.517 | 0.911 | 0.063 | 0.580 | 0.063 | 0.000 | 0.148 | 0.063 |
| ablation: co-entity weight 0.5 | 0.168 | 0.520 | 0.918 | 0.113 | 0.577 | 0.113 | 0.023 | 0.234 | 0.113 |
| ablation: no co-expr | 0.188 | 0.524 | 0.886 | 0.127 | 0.590 | 0.127 | 0.023 | 0.266 | 0.127 |
| ablation: WL depth 1 | 0.192 | 0.521 | 0.917 | 0.127 | 0.580 | 0.127 | 0.023 | 0.266 | 0.127 |
| ablation: WL depth 3 | 0.203 | 0.522 | 0.920 | 0.127 | 0.573 | 0.127 | 0.023 | 0.266 | 0.127 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.093 | 0.116 |
| star | 0.000 | 0.000 | 0.000 |
| tree | 0.000 | 0.000 | 0.000 |
| loop | 0.000 | 0.000 | 0.000 |
| deep-ho (test) | 0.000 | 0.047 | 0.047 |
| comparison (test) | 0.000 | 0.674 | 0.698 |
| mixed (test) | 0.000 | 0.071 | 0.071 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.175 ± 0.032 | 0.177 ± 0.060 | 0.091 ± 0.032 | 0.367 ± 0.033 | 0.400 ± 0.057 |
| TA | 0.498 ± 0.016 | 0.357 ± 0.108 | 0.399 ± 0.102 | 0.482 ± 0.034 | 0.395 ± 0.068 |
| MA | 0.177 ± 0.031 | 0.182 ± 0.067 | 0.305 ± 0.117 | 0.454 ± 0.036 | 0.417 ± 0.057 |
| FOR | 0.499 ± 0.016 | 0.356 ± 0.105 | 0.412 ± 0.079 | 0.487 ± 0.023 | 0.420 ± 0.054 |
| RND | 0.497 ± 0.022 | 0.444 ± 0.052 | 0.493 ± 0.021 | 0.499 ± 0.012 | 0.482 ± 0.021 |
