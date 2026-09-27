# E0 — fingerprint separability (add-ho-s2)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=163.5, runtime=1.7s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 978/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.498 | 0.963 | 0.000 | 0.522 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.005 | 0.499 | 0.987 | 0.000 | 0.499 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.466 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.496 | 0.499 | 0.999 | 0.476 | 0.507 | 0.330 | 0.332 | 0.327 | 0.332 |
| exact C2 relational | 0.844 | 0.847 | 1.000 | 0.933 | 0.933 | 0.902 | 0.855 | 0.965 | 0.912 |
| exact C3 WL | 0.841 | 0.842 | 0.998 | 0.958 | 0.939 | 0.906 | 0.891 | 0.926 | 0.914 |
| exact C4 topology | 0.544 | 0.547 | 0.825 | 0.544 | 0.559 | 0.324 | 0.242 | 0.433 | 0.326 |
| exact analogy profile | 0.849 | 0.850 | 1.000 | 0.976 | 0.965 | 0.951 | 0.927 | 0.984 | 0.961 |
| exact analogy profile +IDF | 0.896 | 0.897 | 1.000 | 0.975 | 0.972 | 0.956 | 0.934 | 0.986 | 0.971 |
| exact literal profile +IDF | 0.000 | 0.867 | 1.000 | 0.000 | 0.967 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.867 | 0.867 | 1.000 | 0.933 | 0.941 | 0.896 | 0.868 | 0.935 | 0.908 |
| fingerprint analogy +IDF D=2048 | 0.874 | 0.876 | 1.000 | 0.949 | 0.954 | 0.917 | 0.885 | 0.960 | 0.929 |
| fingerprint analogy +IDF D=4096 | 0.884 | 0.884 | 1.000 | 0.962 | 0.957 | 0.932 | 0.911 | 0.960 | 0.946 |
| fingerprint analogy +IDF D=8192 | 0.886 | 0.887 | 1.000 | 0.963 | 0.959 | 0.936 | 0.906 | 0.977 | 0.953 |
| fingerprint analogy +IDF D=16384 | 0.884 | 0.887 | 1.000 | 0.962 | 0.955 | 0.930 | 0.904 | 0.965 | 0.947 |
| fingerprint C1 only D=8192 | 0.500 | 0.499 | 1.000 | 0.514 | 0.491 | 0.341 | 0.318 | 0.370 | 0.346 |
| fingerprint C2 only D=8192 | 0.886 | 0.887 | 1.000 | 0.931 | 0.934 | 0.899 | 0.851 | 0.963 | 0.912 |
| fingerprint C3 only D=8192 | 0.888 | 0.885 | 0.993 | 0.930 | 0.927 | 0.875 | 0.872 | 0.879 | 0.888 |
| fingerprint C4 only D=8192 | 0.537 | 0.535 | 0.716 | 0.547 | 0.550 | 0.314 | 0.260 | 0.387 | 0.317 |
| exact mix C2 only +IDF | 0.885 | 0.887 | 1.000 | 0.932 | 0.933 | 0.900 | 0.858 | 0.956 | 0.912 |
| exact mix C3 only +IDF | 0.921 | 0.920 | 1.000 | 0.971 | 0.964 | 0.942 | 0.937 | 0.949 | 0.953 |
| exact mix C4 only +IDF | 0.529 | 0.527 | 0.828 | 0.558 | 0.567 | 0.336 | 0.260 | 0.437 | 0.339 |
| exact mix C2+C3 +IDF | 0.918 | 0.918 | 1.000 | 0.975 | 0.976 | 0.956 | 0.937 | 0.981 | 0.970 |
| exact mix C2+C3+C4 +IDF | 0.901 | 0.902 | 1.000 | 0.973 | 0.969 | 0.950 | 0.923 | 0.986 | 0.965 |
| exact mix C1+C2+C3 +IDF | 0.889 | 0.890 | 1.000 | 0.976 | 0.972 | 0.954 | 0.934 | 0.981 | 0.969 |
| ablation: no taxonomy | 0.904 | 0.904 | 1.000 | 0.975 | 0.971 | 0.953 | 0.928 | 0.986 | 0.970 |
| ablation: no systematicity (beta=0) | 0.862 | 0.863 | 1.000 | 0.971 | 0.967 | 0.947 | 0.925 | 0.977 | 0.962 |
| ablation: no parent-child | 0.627 | 0.626 | 0.999 | 0.878 | 0.858 | 0.787 | 0.745 | 0.843 | 0.798 |
| ablation: no co-entity | 0.902 | 0.902 | 1.000 | 0.973 | 0.970 | 0.952 | 0.928 | 0.984 | 0.967 |
| ablation: co-entity weight 1.0 | 0.840 | 0.842 | 1.000 | 0.975 | 0.972 | 0.951 | 0.941 | 0.965 | 0.961 |
| ablation: co-entity weight 0.5 | 0.882 | 0.883 | 1.000 | 0.978 | 0.973 | 0.957 | 0.935 | 0.986 | 0.970 |
| ablation: no co-expr | 0.907 | 0.908 | 1.000 | 0.977 | 0.974 | 0.958 | 0.937 | 0.986 | 0.972 |
| ablation: WL depth 1 | 0.907 | 0.908 | 1.000 | 0.982 | 0.982 | 0.965 | 0.951 | 0.984 | 0.979 |
| ablation: WL depth 3 | 0.891 | 0.893 | 1.000 | 0.974 | 0.965 | 0.949 | 0.927 | 0.979 | 0.963 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.965 | 0.944 |
| star | 0.000 | 0.965 | 0.965 |
| tree | 0.000 | 0.944 | 0.937 |
| loop | 0.000 | 0.860 | 0.776 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 0.965 | 0.937 |
| mixed (test) | 0.000 | 0.993 | 0.993 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.109 ± 0.053 | 0.205 ± 0.055 | 0.104 ± 0.036 | 0.370 ± 0.031 | 0.404 ± 0.061 |
| TA | 0.500 ± 0.015 | 0.220 ± 0.055 | 0.239 ± 0.048 | 0.438 ± 0.025 | 0.463 ± 0.030 |
| MA | 0.107 ± 0.055 | 0.220 ± 0.055 | 0.360 ± 0.081 | 0.477 ± 0.020 | 0.468 ± 0.024 |
| FOR | 0.501 ± 0.016 | 0.220 ± 0.054 | 0.362 ± 0.081 | 0.477 ± 0.021 | 0.467 ± 0.026 |
| RND | 0.496 ± 0.022 | 0.455 ± 0.039 | 0.495 ± 0.015 | 0.499 ± 0.012 | 0.483 ± 0.021 |
