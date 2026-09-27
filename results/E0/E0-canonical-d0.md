# E0 — fingerprint separability (canonical-d0)

Config: groups=1000, seed=1, naming=Canonical, distractors=0, cases=8000, mean features/case=118.0, runtime=1.0s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) |
|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.500 | 0.972 | 0.000 | 0.508 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.000 | 0.499 | 0.992 | 0.000 | 0.502 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.466 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.500 | 0.500 | 1.000 | 0.500 | 0.500 | 0.500 | 0.500 | 0.500 |
| exact C2 relational | 0.956 | 0.956 | 1.000 | 0.944 | 0.944 | 0.944 | 0.920 | 0.974 |
| exact C3 WL | 0.988 | 0.988 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| exact C4 topology | 0.877 | 0.877 | 1.000 | 0.874 | 0.874 | 0.874 | 0.798 | 0.974 |
| exact analogy profile | 0.988 | 0.988 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| exact analogy profile +IDF | 0.989 | 0.989 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| exact literal profile +IDF | 0.000 | 0.990 | 1.000 | 0.000 | 0.991 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.991 | 0.991 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| fingerprint analogy +IDF D=2048 | 0.991 | 0.991 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| fingerprint analogy +IDF D=4096 | 0.991 | 0.991 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| fingerprint analogy +IDF D=8192 | 0.991 | 0.991 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| fingerprint analogy +IDF D=16384 | 0.991 | 0.991 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| fingerprint C1 only D=8192 | 0.500 | 0.500 | 1.000 | 0.500 | 0.500 | 0.500 | 0.500 | 0.500 |
| fingerprint C2 only D=8192 | 0.944 | 0.944 | 1.000 | 0.944 | 0.944 | 0.944 | 0.920 | 0.974 |
| fingerprint C3 only D=8192 | 0.991 | 0.991 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| fingerprint C4 only D=8192 | 0.874 | 0.874 | 1.000 | 0.874 | 0.874 | 0.874 | 0.798 | 0.974 |
| exact mix C2 only +IDF | 0.940 | 0.940 | 1.000 | 0.944 | 0.944 | 0.944 | 0.920 | 0.974 |
| exact mix C3 only +IDF | 0.990 | 0.990 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| exact mix C4 only +IDF | 0.888 | 0.888 | 1.000 | 0.874 | 0.874 | 0.874 | 0.798 | 0.974 |
| exact mix C2+C3 +IDF | 0.989 | 0.989 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| exact mix C2+C3+C4 +IDF | 0.989 | 0.989 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| exact mix C1+C2+C3 +IDF | 0.989 | 0.989 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| ablation: no taxonomy | 0.993 | 0.993 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| ablation: no systematicity (beta=0) | 0.991 | 0.991 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| ablation: no parent-child | 0.990 | 0.990 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| ablation: no co-entity | 0.989 | 0.989 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| ablation: co-entity weight 1.0 | 0.990 | 0.990 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| ablation: co-entity weight 0.5 | 0.988 | 0.988 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| ablation: no co-expr | 0.989 | 0.989 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| ablation: WL depth 1 | 0.989 | 0.989 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |
| ablation: WL depth 3 | 0.990 | 0.990 | 1.000 | 0.991 | 0.991 | 0.991 | 1.000 | 0.979 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 1.000 | 1.000 |
| loop | 0.000 | 1.000 | 1.000 |
| deep-ho (test) | 0.000 | 0.937 | 0.937 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.000 ± 0.000 | 0.000 ± 0.000 | 0.000 ± 0.000 | 0.000 ± 0.000 | 0.000 ± 0.000 |
| TA | 0.500 ± 0.016 | 0.000 ± 0.000 | 0.000 ± 0.000 | 0.000 ± 0.000 | 0.000 ± 0.000 |
| MA | 0.000 ± 0.000 | 0.000 ± 0.000 | 0.301 ± 0.134 | 0.368 ± 0.067 | 0.269 ± 0.162 |
| FOR | 0.500 ± 0.015 | 0.000 ± 0.000 | 0.301 ± 0.134 | 0.368 ± 0.067 | 0.269 ± 0.162 |
| RND | 0.497 ± 0.021 | 0.462 ± 0.040 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.478 ± 0.031 |
