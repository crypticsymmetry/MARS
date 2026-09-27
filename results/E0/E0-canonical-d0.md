# E0 — fingerprint separability (canonical-d0)

Config: groups=1000, seed=1, naming=Canonical, distractors=0, cases=8000, mean features/case=118.0, runtime=1.1s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) |
|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.499 | 0.973 | 0.000 | 0.506 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.000 | 0.499 | 0.992 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.467 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.500 | 0.500 | 1.000 | 0.500 | 0.500 | 0.500 | 0.500 | 0.500 |
| exact C2 relational | 0.961 | 0.961 | 1.000 | 0.953 | 0.953 | 0.953 | 0.920 | 0.996 |
| exact C3 WL | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact C4 topology | 0.890 | 0.890 | 1.000 | 0.882 | 0.882 | 0.882 | 0.798 | 0.995 |
| exact analogy profile | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact analogy profile +IDF | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact literal profile +IDF | 0.000 | 1.000 | 1.000 | 0.000 | 1.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| fingerprint analogy +IDF D=2048 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| fingerprint analogy +IDF D=4096 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| fingerprint analogy +IDF D=8192 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| fingerprint analogy +IDF D=16384 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| fingerprint C1 only D=8192 | 0.500 | 0.500 | 1.000 | 0.500 | 0.500 | 0.500 | 0.500 | 0.500 |
| fingerprint C2 only D=8192 | 0.953 | 0.953 | 1.000 | 0.953 | 0.953 | 0.953 | 0.920 | 0.996 |
| fingerprint C3 only D=8192 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| fingerprint C4 only D=8192 | 0.882 | 0.882 | 1.000 | 0.882 | 0.882 | 0.882 | 0.798 | 0.995 |
| exact mix C2 only +IDF | 0.955 | 0.955 | 1.000 | 0.953 | 0.953 | 0.953 | 0.920 | 0.996 |
| exact mix C3 only +IDF | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact mix C4 only +IDF | 0.893 | 0.893 | 1.000 | 0.882 | 0.882 | 0.882 | 0.798 | 0.995 |
| exact mix C2+C3 +IDF | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact mix C2+C3+C4 +IDF | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| exact mix C1+C2+C3 +IDF | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: no taxonomy | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: no systematicity (beta=0) | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: no parent-child | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: no co-entity | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: co-entity weight 1.0 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: co-entity weight 0.5 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: no co-expr | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: WL depth 1 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |
| ablation: WL depth 3 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 1.000 | 1.000 |
| star | 0.000 | 1.000 | 1.000 |
| tree | 0.000 | 1.000 | 1.000 |
| loop | 0.000 | 1.000 | 1.000 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 1.000 | 1.000 |
| mixed (test) | 0.000 | 1.000 | 1.000 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.000 ± 0.000 | 0.000 ± 0.000 | 0.000 ± 0.000 | 0.000 ± 0.000 | 0.000 ± 0.000 |
| TA | 0.500 ± 0.016 | 0.000 ± 0.000 | 0.000 ± 0.000 | 0.000 ± 0.000 | 0.000 ± 0.000 |
| MA | 0.000 ± 0.000 | 0.000 ± 0.000 | 0.308 ± 0.128 | 0.375 ± 0.045 | 0.275 ± 0.158 |
| FOR | 0.500 ± 0.015 | 0.000 ± 0.000 | 0.308 ± 0.128 | 0.375 ± 0.045 | 0.275 ± 0.158 |
| RND | 0.497 ± 0.021 | 0.462 ± 0.040 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.478 ± 0.031 |
