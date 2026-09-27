# E0 — fingerprint separability (add-ho-s1)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=159.1, runtime=1.6s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 986/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.498 | 0.955 | 0.000 | 0.501 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.001 | 0.497 | 0.989 | 0.000 | 0.475 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.462 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.496 | 0.499 | 1.000 | 0.492 | 0.489 | 0.318 | 0.301 | 0.341 | 0.319 |
| exact C2 relational | 0.885 | 0.883 | 1.000 | 0.948 | 0.946 | 0.924 | 0.877 | 0.988 | 0.931 |
| exact C3 WL | 0.886 | 0.884 | 1.000 | 0.982 | 0.976 | 0.961 | 0.950 | 0.974 | 0.968 |
| exact C4 topology | 0.557 | 0.566 | 0.902 | 0.561 | 0.590 | 0.385 | 0.266 | 0.544 | 0.385 |
| exact analogy profile | 0.893 | 0.892 | 1.000 | 0.980 | 0.979 | 0.963 | 0.939 | 0.995 | 0.971 |
| exact analogy profile +IDF | 0.921 | 0.920 | 1.000 | 0.976 | 0.977 | 0.958 | 0.930 | 0.995 | 0.968 |
| exact literal profile +IDF | 0.000 | 0.897 | 1.000 | 0.000 | 0.977 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.902 | 0.902 | 1.000 | 0.954 | 0.957 | 0.924 | 0.890 | 0.970 | 0.932 |
| fingerprint analogy +IDF D=2048 | 0.908 | 0.907 | 1.000 | 0.964 | 0.962 | 0.934 | 0.897 | 0.984 | 0.943 |
| fingerprint analogy +IDF D=4096 | 0.910 | 0.908 | 1.000 | 0.969 | 0.964 | 0.944 | 0.909 | 0.991 | 0.952 |
| fingerprint analogy +IDF D=8192 | 0.911 | 0.909 | 1.000 | 0.972 | 0.972 | 0.951 | 0.921 | 0.991 | 0.959 |
| fingerprint analogy +IDF D=16384 | 0.912 | 0.909 | 1.000 | 0.972 | 0.974 | 0.952 | 0.923 | 0.991 | 0.960 |
| fingerprint C1 only D=8192 | 0.492 | 0.495 | 1.000 | 0.491 | 0.470 | 0.321 | 0.297 | 0.353 | 0.324 |
| fingerprint C2 only D=8192 | 0.909 | 0.905 | 1.000 | 0.943 | 0.937 | 0.918 | 0.872 | 0.979 | 0.926 |
| fingerprint C3 only D=8192 | 0.926 | 0.925 | 0.998 | 0.960 | 0.958 | 0.929 | 0.906 | 0.961 | 0.937 |
| fingerprint C4 only D=8192 | 0.547 | 0.544 | 0.807 | 0.587 | 0.565 | 0.352 | 0.278 | 0.452 | 0.353 |
| exact mix C2 only +IDF | 0.911 | 0.907 | 1.000 | 0.936 | 0.935 | 0.909 | 0.857 | 0.979 | 0.918 |
| exact mix C3 only +IDF | 0.944 | 0.943 | 1.000 | 0.979 | 0.979 | 0.962 | 0.958 | 0.967 | 0.971 |
| exact mix C4 only +IDF | 0.542 | 0.548 | 0.897 | 0.587 | 0.604 | 0.399 | 0.284 | 0.551 | 0.400 |
| exact mix C2+C3 +IDF | 0.942 | 0.940 | 1.000 | 0.980 | 0.979 | 0.964 | 0.941 | 0.995 | 0.974 |
| exact mix C2+C3+C4 +IDF | 0.920 | 0.919 | 1.000 | 0.974 | 0.971 | 0.952 | 0.921 | 0.993 | 0.961 |
| exact mix C1+C2+C3 +IDF | 0.920 | 0.919 | 1.000 | 0.978 | 0.979 | 0.962 | 0.937 | 0.995 | 0.972 |
| ablation: no taxonomy | 0.927 | 0.925 | 1.000 | 0.976 | 0.977 | 0.958 | 0.930 | 0.995 | 0.968 |
| ablation: no systematicity (beta=0) | 0.894 | 0.893 | 1.000 | 0.974 | 0.974 | 0.953 | 0.925 | 0.991 | 0.962 |
| ablation: no parent-child | 0.652 | 0.654 | 1.000 | 0.887 | 0.908 | 0.831 | 0.759 | 0.928 | 0.840 |
| ablation: no co-entity | 0.927 | 0.926 | 1.000 | 0.978 | 0.976 | 0.959 | 0.932 | 0.995 | 0.969 |
| ablation: co-entity weight 1.0 | 0.869 | 0.868 | 1.000 | 0.974 | 0.976 | 0.958 | 0.937 | 0.986 | 0.968 |
| ablation: co-entity weight 0.5 | 0.907 | 0.906 | 1.000 | 0.975 | 0.976 | 0.958 | 0.932 | 0.993 | 0.968 |
| ablation: no co-expr | 0.929 | 0.928 | 1.000 | 0.977 | 0.977 | 0.959 | 0.932 | 0.995 | 0.969 |
| ablation: WL depth 1 | 0.931 | 0.930 | 1.000 | 0.981 | 0.983 | 0.967 | 0.949 | 0.991 | 0.977 |
| ablation: WL depth 3 | 0.917 | 0.916 | 1.000 | 0.975 | 0.976 | 0.956 | 0.927 | 0.995 | 0.966 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.979 | 0.972 |
| star | 0.000 | 0.972 | 0.972 |
| tree | 0.000 | 0.937 | 0.937 |
| loop | 0.000 | 0.832 | 0.804 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 0.986 | 0.972 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.204 ± 0.055 | 0.102 ± 0.036 | 0.371 ± 0.031 | 0.405 ± 0.061 |
| TA | 0.500 ± 0.016 | 0.214 ± 0.055 | 0.198 ± 0.049 | 0.420 ± 0.030 | 0.448 ± 0.038 |
| MA | 0.107 ± 0.053 | 0.213 ± 0.054 | 0.350 ± 0.092 | 0.472 ± 0.022 | 0.455 ± 0.033 |
| FOR | 0.500 ± 0.015 | 0.213 ± 0.056 | 0.350 ± 0.093 | 0.473 ± 0.023 | 0.455 ± 0.032 |
| RND | 0.496 ± 0.022 | 0.454 ± 0.040 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.020 |
