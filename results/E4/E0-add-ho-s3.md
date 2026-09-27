# E0 — fingerprint separability (add-ho-s3)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=166.7, runtime=1.6s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 968/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.502 | 0.958 | 0.000 | 0.514 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.011 | 0.496 | 0.985 | 0.000 | 0.481 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C0 surface | 0.000 | 0.500 | 0.464 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.506 | 0.498 | 0.998 | 0.510 | 0.473 | 0.329 | 0.338 | 0.317 | 0.331 |
| exact C2 relational | 0.814 | 0.815 | 1.000 | 0.930 | 0.926 | 0.890 | 0.837 | 0.960 | 0.903 |
| exact C3 WL | 0.829 | 0.831 | 0.998 | 0.935 | 0.931 | 0.887 | 0.861 | 0.922 | 0.904 |
| exact C4 topology | 0.544 | 0.539 | 0.769 | 0.577 | 0.551 | 0.294 | 0.226 | 0.386 | 0.298 |
| exact analogy profile | 0.817 | 0.818 | 1.000 | 0.965 | 0.965 | 0.939 | 0.916 | 0.970 | 0.956 |
| exact analogy profile +IDF | 0.875 | 0.879 | 1.000 | 0.969 | 0.972 | 0.949 | 0.934 | 0.970 | 0.966 |
| exact literal profile +IDF | 0.000 | 0.844 | 1.000 | 0.000 | 0.967 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.846 | 0.849 | 1.000 | 0.925 | 0.911 | 0.871 | 0.844 | 0.907 | 0.886 |
| fingerprint analogy +IDF D=2048 | 0.854 | 0.860 | 1.000 | 0.948 | 0.940 | 0.905 | 0.885 | 0.932 | 0.921 |
| fingerprint analogy +IDF D=4096 | 0.863 | 0.865 | 1.000 | 0.953 | 0.950 | 0.917 | 0.893 | 0.949 | 0.938 |
| fingerprint analogy +IDF D=8192 | 0.865 | 0.868 | 1.000 | 0.959 | 0.962 | 0.934 | 0.916 | 0.958 | 0.949 |
| fingerprint analogy +IDF D=16384 | 0.865 | 0.867 | 1.000 | 0.959 | 0.962 | 0.935 | 0.911 | 0.967 | 0.950 |
| fingerprint C1 only D=8192 | 0.510 | 0.498 | 1.000 | 0.502 | 0.470 | 0.319 | 0.330 | 0.304 | 0.318 |
| fingerprint C2 only D=8192 | 0.866 | 0.868 | 1.000 | 0.933 | 0.929 | 0.899 | 0.862 | 0.950 | 0.914 |
| fingerprint C3 only D=8192 | 0.871 | 0.880 | 0.988 | 0.908 | 0.914 | 0.844 | 0.844 | 0.843 | 0.858 |
| fingerprint C4 only D=8192 | 0.518 | 0.506 | 0.669 | 0.541 | 0.524 | 0.259 | 0.202 | 0.335 | 0.263 |
| exact mix C2 only +IDF | 0.865 | 0.868 | 1.000 | 0.937 | 0.933 | 0.904 | 0.864 | 0.958 | 0.916 |
| exact mix C3 only +IDF | 0.913 | 0.918 | 1.000 | 0.955 | 0.952 | 0.916 | 0.918 | 0.914 | 0.932 |
| exact mix C4 only +IDF | 0.521 | 0.516 | 0.772 | 0.551 | 0.540 | 0.290 | 0.185 | 0.430 | 0.296 |
| exact mix C2+C3 +IDF | 0.898 | 0.902 | 1.000 | 0.972 | 0.976 | 0.955 | 0.942 | 0.972 | 0.971 |
| exact mix C2+C3+C4 +IDF | 0.884 | 0.888 | 1.000 | 0.968 | 0.975 | 0.951 | 0.937 | 0.970 | 0.967 |
| exact mix C1+C2+C3 +IDF | 0.865 | 0.869 | 1.000 | 0.966 | 0.974 | 0.948 | 0.932 | 0.970 | 0.966 |
| ablation: no taxonomy | 0.885 | 0.889 | 1.000 | 0.970 | 0.976 | 0.953 | 0.941 | 0.970 | 0.968 |
| ablation: no systematicity (beta=0) | 0.840 | 0.841 | 1.000 | 0.965 | 0.966 | 0.943 | 0.923 | 0.970 | 0.960 |
| ablation: no parent-child | 0.620 | 0.613 | 0.999 | 0.839 | 0.827 | 0.736 | 0.699 | 0.785 | 0.747 |
| ablation: no co-entity | 0.881 | 0.885 | 1.000 | 0.969 | 0.972 | 0.950 | 0.935 | 0.970 | 0.967 |
| ablation: co-entity weight 1.0 | 0.823 | 0.822 | 1.000 | 0.966 | 0.959 | 0.936 | 0.923 | 0.953 | 0.954 |
| ablation: co-entity weight 0.5 | 0.862 | 0.865 | 1.000 | 0.972 | 0.974 | 0.953 | 0.941 | 0.970 | 0.969 |
| ablation: no co-expr | 0.889 | 0.893 | 1.000 | 0.971 | 0.974 | 0.952 | 0.939 | 0.970 | 0.969 |
| ablation: WL depth 1 | 0.889 | 0.893 | 1.000 | 0.977 | 0.979 | 0.959 | 0.948 | 0.974 | 0.976 |
| ablation: WL depth 3 | 0.871 | 0.874 | 1.000 | 0.966 | 0.969 | 0.945 | 0.928 | 0.967 | 0.961 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.951 | 0.951 |
| star | 0.000 | 0.972 | 0.972 |
| tree | 0.000 | 0.951 | 0.937 |
| loop | 0.000 | 0.860 | 0.804 |
| deep-ho (test) | 0.000 | 1.000 | 1.000 |
| comparison (test) | 0.000 | 0.944 | 0.923 |
| mixed (test) | 0.000 | 0.965 | 0.951 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.109 ± 0.054 | 0.206 ± 0.056 | 0.105 ± 0.036 | 0.370 ± 0.031 | 0.403 ± 0.062 |
| TA | 0.500 ± 0.016 | 0.225 ± 0.056 | 0.266 ± 0.049 | 0.447 ± 0.022 | 0.469 ± 0.025 |
| MA | 0.106 ± 0.055 | 0.226 ± 0.055 | 0.368 ± 0.074 | 0.479 ± 0.019 | 0.471 ± 0.023 |
| FOR | 0.500 ± 0.016 | 0.224 ± 0.055 | 0.368 ± 0.073 | 0.479 ± 0.018 | 0.470 ± 0.022 |
| RND | 0.496 ± 0.022 | 0.456 ± 0.039 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.021 |
