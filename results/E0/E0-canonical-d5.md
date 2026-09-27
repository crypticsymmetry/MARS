# E0 — fingerprint separability (canonical-d5)

Config: groups=1000, seed=1, naming=Canonical, distractors=5, cases=8000, mean features/case=204.2, runtime=2.0s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) |
|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.502 | 0.948 | 0.000 | 0.513 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.020 | 0.503 | 0.980 | 0.000 | 0.499 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.465 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.500 | 0.501 | 0.995 | 0.491 | 0.501 | 0.337 | 0.341 | 0.332 |
| exact C2 relational | 0.908 | 0.907 | 1.000 | 0.949 | 0.947 | 0.929 | 0.899 | 0.970 |
| exact C3 WL | 0.940 | 0.939 | 1.000 | 0.991 | 0.989 | 0.987 | 0.997 | 0.974 |
| exact C4 topology | 0.590 | 0.580 | 0.963 | 0.597 | 0.594 | 0.449 | 0.327 | 0.612 |
| exact analogy profile | 0.932 | 0.930 | 1.000 | 0.992 | 0.991 | 0.990 | 1.000 | 0.977 |
| exact analogy profile +IDF | 0.948 | 0.946 | 1.000 | 0.994 | 0.993 | 0.990 | 1.000 | 0.977 |
| exact literal profile +IDF | 0.000 | 0.915 | 1.000 | 0.000 | 0.991 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.931 | 0.926 | 1.000 | 0.982 | 0.976 | 0.967 | 0.962 | 0.972 |
| fingerprint analogy +IDF D=2048 | 0.937 | 0.933 | 1.000 | 0.985 | 0.986 | 0.979 | 0.983 | 0.973 |
| fingerprint analogy +IDF D=4096 | 0.940 | 0.935 | 1.000 | 0.990 | 0.989 | 0.984 | 0.990 | 0.977 |
| fingerprint analogy +IDF D=8192 | 0.940 | 0.937 | 1.000 | 0.991 | 0.989 | 0.986 | 0.998 | 0.970 |
| fingerprint analogy +IDF D=16384 | 0.941 | 0.938 | 1.000 | 0.992 | 0.990 | 0.988 | 0.998 | 0.974 |
| fingerprint C1 only D=8192 | 0.496 | 0.498 | 0.995 | 0.492 | 0.493 | 0.317 | 0.309 | 0.328 |
| fingerprint C2 only D=8192 | 0.929 | 0.928 | 1.000 | 0.954 | 0.940 | 0.923 | 0.894 | 0.963 |
| fingerprint C3 only D=8192 | 0.968 | 0.965 | 0.999 | 0.983 | 0.980 | 0.974 | 0.990 | 0.953 |
| fingerprint C4 only D=8192 | 0.566 | 0.557 | 0.877 | 0.597 | 0.569 | 0.387 | 0.280 | 0.529 |
| exact mix C2 only +IDF | 0.929 | 0.927 | 1.000 | 0.949 | 0.945 | 0.930 | 0.900 | 0.970 |
| exact mix C3 only +IDF | 0.980 | 0.979 | 1.000 | 0.992 | 0.990 | 0.988 | 1.000 | 0.972 |
| exact mix C4 only +IDF | 0.571 | 0.573 | 0.962 | 0.605 | 0.615 | 0.440 | 0.341 | 0.572 |
| exact mix C2+C3 +IDF | 0.968 | 0.967 | 1.000 | 0.990 | 0.992 | 0.988 | 0.998 | 0.974 |
| exact mix C2+C3+C4 +IDF | 0.947 | 0.946 | 1.000 | 0.989 | 0.989 | 0.986 | 1.000 | 0.967 |
| exact mix C1+C2+C3 +IDF | 0.942 | 0.941 | 1.000 | 0.991 | 0.992 | 0.989 | 0.998 | 0.977 |
| ablation: no taxonomy | 0.953 | 0.952 | 1.000 | 0.992 | 0.992 | 0.989 | 1.000 | 0.974 |
| ablation: no systematicity (beta=0) | 0.918 | 0.916 | 1.000 | 0.993 | 0.992 | 0.988 | 0.998 | 0.974 |
| ablation: no parent-child | 0.691 | 0.692 | 1.000 | 0.982 | 0.971 | 0.961 | 0.972 | 0.946 |
| ablation: no co-entity | 0.973 | 0.972 | 1.000 | 0.990 | 0.991 | 0.988 | 1.000 | 0.972 |
| ablation: co-entity weight 1.0 | 0.879 | 0.876 | 1.000 | 0.992 | 0.990 | 0.986 | 0.995 | 0.974 |
| ablation: co-entity weight 0.5 | 0.913 | 0.912 | 1.000 | 0.993 | 0.992 | 0.988 | 0.998 | 0.974 |
| ablation: no co-expr | 0.952 | 0.950 | 1.000 | 0.994 | 0.993 | 0.990 | 1.000 | 0.977 |
| ablation: WL depth 1 | 0.952 | 0.950 | 1.000 | 0.991 | 0.992 | 0.988 | 1.000 | 0.972 |
| ablation: WL depth 3 | 0.943 | 0.941 | 1.000 | 0.994 | 0.993 | 0.990 | 1.000 | 0.977 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 1.000 | 0.993 |
| loop | 0.000 | 1.000 | 1.000 |
| deep-ho (test) | 0.000 | 0.930 | 0.909 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.125 ± 0.032 | 0.267 ± 0.064 | 0.153 ± 0.043 | 0.412 ± 0.024 | 0.439 ± 0.034 |
| TA | 0.500 ± 0.015 | 0.267 ± 0.065 | 0.152 ± 0.043 | 0.413 ± 0.024 | 0.438 ± 0.036 |
| MA | 0.125 ± 0.031 | 0.266 ± 0.065 | 0.342 ± 0.104 | 0.478 ± 0.022 | 0.446 ± 0.032 |
| FOR | 0.501 ± 0.016 | 0.266 ± 0.065 | 0.342 ± 0.104 | 0.477 ± 0.022 | 0.444 ± 0.033 |
| RND | 0.497 ± 0.022 | 0.444 ± 0.039 | 0.496 ± 0.015 | 0.498 ± 0.011 | 0.480 ± 0.018 |
