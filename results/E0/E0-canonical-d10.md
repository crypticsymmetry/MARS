# E0 — fingerprint separability (canonical-d10)

Config: groups=1000, seed=1, naming=Canonical, distractors=10, cases=8000, mean features/case=298.3, runtime=2.8s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) |
|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.502 | 0.917 | 0.000 | 0.516 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.059 | 0.501 | 0.962 | 0.001 | 0.503 | 0.001 | 0.002 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.464 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.500 | 0.502 | 0.976 | 0.511 | 0.499 | 0.339 | 0.339 | 0.339 |
| exact C2 relational | 0.873 | 0.873 | 1.000 | 0.951 | 0.957 | 0.938 | 0.897 | 0.993 |
| exact C3 WL | 0.931 | 0.930 | 0.999 | 0.982 | 0.987 | 0.969 | 0.951 | 0.993 |
| exact C4 topology | 0.553 | 0.561 | 0.939 | 0.561 | 0.583 | 0.423 | 0.321 | 0.561 |
| exact analogy profile | 0.899 | 0.900 | 1.000 | 0.990 | 0.993 | 0.987 | 0.983 | 0.993 |
| exact analogy profile +IDF | 0.930 | 0.930 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact literal profile +IDF | 0.000 | 0.894 | 1.000 | 0.000 | 0.998 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.919 | 0.917 | 1.000 | 0.975 | 0.982 | 0.967 | 0.946 | 0.994 |
| fingerprint analogy +IDF D=2048 | 0.925 | 0.926 | 1.000 | 0.987 | 0.994 | 0.982 | 0.970 | 0.998 |
| fingerprint analogy +IDF D=4096 | 0.929 | 0.930 | 1.000 | 0.993 | 0.997 | 0.991 | 0.984 | 1.000 |
| fingerprint analogy +IDF D=8192 | 0.928 | 0.929 | 1.000 | 0.996 | 0.999 | 0.995 | 0.991 | 1.000 |
| fingerprint analogy +IDF D=16384 | 0.927 | 0.928 | 1.000 | 0.999 | 0.999 | 0.998 | 0.997 | 1.000 |
| fingerprint C1 only D=8192 | 0.495 | 0.504 | 0.976 | 0.476 | 0.518 | 0.334 | 0.325 | 0.346 |
| fingerprint C2 only D=8192 | 0.921 | 0.922 | 1.000 | 0.953 | 0.958 | 0.944 | 0.907 | 0.993 |
| fingerprint C3 only D=8192 | 0.968 | 0.969 | 0.999 | 0.995 | 0.998 | 0.992 | 0.987 | 0.998 |
| fingerprint C4 only D=8192 | 0.553 | 0.570 | 0.807 | 0.562 | 0.582 | 0.353 | 0.311 | 0.409 |
| exact mix C2 only +IDF | 0.917 | 0.918 | 1.000 | 0.957 | 0.955 | 0.940 | 0.900 | 0.993 |
| exact mix C3 only +IDF | 0.982 | 0.981 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact mix C4 only +IDF | 0.560 | 0.569 | 0.924 | 0.584 | 0.603 | 0.422 | 0.344 | 0.526 |
| exact mix C2+C3 +IDF | 0.952 | 0.952 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact mix C2+C3+C4 +IDF | 0.941 | 0.941 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact mix C1+C2+C3 +IDF | 0.917 | 0.918 | 1.000 | 0.999 | 1.000 | 0.999 | 1.000 | 0.998 |
| ablation: no taxonomy | 0.938 | 0.938 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: no systematicity (beta=0) | 0.888 | 0.888 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: no parent-child | 0.695 | 0.695 | 1.000 | 0.975 | 0.982 | 0.960 | 0.951 | 0.972 |
| ablation: no co-entity | 0.979 | 0.979 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: co-entity weight 1.0 | 0.859 | 0.857 | 1.000 | 0.993 | 0.994 | 0.987 | 0.984 | 0.991 |
| ablation: co-entity weight 0.5 | 0.889 | 0.888 | 1.000 | 1.000 | 0.999 | 0.999 | 0.998 | 1.000 |
| ablation: no co-expr | 0.936 | 0.936 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: WL depth 1 | 0.937 | 0.937 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: WL depth 3 | 0.924 | 0.924 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 0.993 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 1.000 | 1.000 |
| loop | 0.007 | 1.000 | 0.972 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.127 ± 0.029 | 0.296 ± 0.064 | 0.207 ± 0.049 | 0.427 ± 0.022 | 0.444 ± 0.025 |
| TA | 0.499 ± 0.015 | 0.297 ± 0.064 | 0.207 ± 0.049 | 0.426 ± 0.022 | 0.441 ± 0.024 |
| MA | 0.127 ± 0.029 | 0.296 ± 0.063 | 0.363 ± 0.088 | 0.481 ± 0.018 | 0.446 ± 0.022 |
| FOR | 0.501 ± 0.015 | 0.298 ± 0.065 | 0.364 ± 0.088 | 0.482 ± 0.018 | 0.447 ± 0.024 |
| RND | 0.496 ± 0.022 | 0.427 ± 0.038 | 0.495 ± 0.012 | 0.497 ± 0.011 | 0.467 ± 0.018 |
