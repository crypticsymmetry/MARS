# E0 — fingerprint separability (add-ho-s4)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=170.5, runtime=1.7s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 953/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.499 | 0.961 | 0.000 | 0.489 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.020 | 0.502 | 0.986 | 0.001 | 0.500 | 0.001 | 0.000 | 0.002 | 0.001 |
| exact C0 surface | 0.000 | 0.500 | 0.460 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.499 | 0.507 | 0.997 | 0.499 | 0.521 | 0.337 | 0.365 | 0.300 | 0.344 |
| exact C2 relational | 0.786 | 0.787 | 1.000 | 0.910 | 0.902 | 0.856 | 0.787 | 0.949 | 0.875 |
| exact C3 WL | 0.815 | 0.814 | 0.997 | 0.923 | 0.930 | 0.873 | 0.855 | 0.897 | 0.890 |
| exact C4 topology | 0.538 | 0.530 | 0.720 | 0.573 | 0.546 | 0.258 | 0.196 | 0.341 | 0.262 |
| exact analogy profile | 0.783 | 0.785 | 1.000 | 0.944 | 0.952 | 0.912 | 0.881 | 0.953 | 0.933 |
| exact analogy profile +IDF | 0.855 | 0.858 | 1.000 | 0.949 | 0.957 | 0.923 | 0.900 | 0.953 | 0.944 |
| exact literal profile +IDF | 0.000 | 0.819 | 1.000 | 0.000 | 0.942 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.820 | 0.823 | 1.000 | 0.903 | 0.908 | 0.848 | 0.815 | 0.893 | 0.866 |
| fingerprint analogy +IDF D=2048 | 0.835 | 0.839 | 1.000 | 0.918 | 0.929 | 0.874 | 0.832 | 0.930 | 0.890 |
| fingerprint analogy +IDF D=4096 | 0.845 | 0.848 | 1.000 | 0.933 | 0.941 | 0.897 | 0.860 | 0.946 | 0.918 |
| fingerprint analogy +IDF D=8192 | 0.844 | 0.847 | 1.000 | 0.938 | 0.943 | 0.899 | 0.862 | 0.949 | 0.919 |
| fingerprint analogy +IDF D=16384 | 0.845 | 0.848 | 1.000 | 0.943 | 0.939 | 0.903 | 0.867 | 0.951 | 0.924 |
| fingerprint C1 only D=8192 | 0.493 | 0.501 | 1.000 | 0.495 | 0.518 | 0.340 | 0.343 | 0.335 | 0.341 |
| fingerprint C2 only D=8192 | 0.848 | 0.849 | 1.000 | 0.913 | 0.912 | 0.864 | 0.809 | 0.938 | 0.885 |
| fingerprint C3 only D=8192 | 0.859 | 0.860 | 0.985 | 0.896 | 0.892 | 0.817 | 0.809 | 0.826 | 0.831 |
| fingerprint C4 only D=8192 | 0.501 | 0.529 | 0.638 | 0.509 | 0.537 | 0.247 | 0.193 | 0.320 | 0.253 |
| exact mix C2 only +IDF | 0.847 | 0.848 | 1.000 | 0.914 | 0.919 | 0.870 | 0.813 | 0.946 | 0.892 |
| exact mix C3 only +IDF | 0.905 | 0.909 | 1.000 | 0.948 | 0.957 | 0.918 | 0.923 | 0.911 | 0.933 |
| exact mix C4 only +IDF | 0.511 | 0.513 | 0.744 | 0.544 | 0.551 | 0.269 | 0.191 | 0.374 | 0.272 |
| exact mix C2+C3 +IDF | 0.880 | 0.883 | 1.000 | 0.954 | 0.954 | 0.925 | 0.904 | 0.953 | 0.949 |
| exact mix C2+C3+C4 +IDF | 0.866 | 0.870 | 1.000 | 0.952 | 0.954 | 0.923 | 0.899 | 0.956 | 0.948 |
| exact mix C1+C2+C3 +IDF | 0.843 | 0.845 | 1.000 | 0.948 | 0.949 | 0.915 | 0.890 | 0.949 | 0.935 |
| ablation: no taxonomy | 0.867 | 0.870 | 1.000 | 0.948 | 0.951 | 0.917 | 0.893 | 0.949 | 0.940 |
| ablation: no systematicity (beta=0) | 0.816 | 0.818 | 1.000 | 0.946 | 0.949 | 0.915 | 0.888 | 0.951 | 0.937 |
| ablation: no parent-child | 0.602 | 0.606 | 0.999 | 0.828 | 0.833 | 0.738 | 0.698 | 0.792 | 0.751 |
| ablation: no co-entity | 0.861 | 0.864 | 1.000 | 0.950 | 0.954 | 0.921 | 0.899 | 0.951 | 0.942 |
| ablation: co-entity weight 1.0 | 0.800 | 0.804 | 1.000 | 0.951 | 0.962 | 0.920 | 0.906 | 0.939 | 0.942 |
| ablation: co-entity weight 0.5 | 0.841 | 0.844 | 1.000 | 0.952 | 0.959 | 0.923 | 0.902 | 0.951 | 0.944 |
| ablation: no co-expr | 0.871 | 0.874 | 1.000 | 0.952 | 0.961 | 0.927 | 0.907 | 0.953 | 0.949 |
| ablation: WL depth 1 | 0.870 | 0.873 | 1.000 | 0.962 | 0.968 | 0.937 | 0.920 | 0.960 | 0.956 |
| ablation: WL depth 3 | 0.850 | 0.853 | 1.000 | 0.949 | 0.949 | 0.917 | 0.892 | 0.951 | 0.938 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.951 | 0.923 |
| star | 0.000 | 0.916 | 0.895 |
| tree | 0.000 | 0.916 | 0.895 |
| loop | 0.000 | 0.818 | 0.734 |
| deep-ho (test) | 0.000 | 0.993 | 0.993 |
| comparison (test) | 0.007 | 0.923 | 0.895 |
| mixed (test) | 0.000 | 0.944 | 0.958 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.207 ± 0.056 | 0.107 ± 0.036 | 0.370 ± 0.031 | 0.402 ± 0.062 |
| TA | 0.500 ± 0.016 | 0.233 ± 0.056 | 0.285 ± 0.048 | 0.452 ± 0.020 | 0.472 ± 0.024 |
| MA | 0.107 ± 0.054 | 0.232 ± 0.057 | 0.374 ± 0.069 | 0.480 ± 0.018 | 0.473 ± 0.020 |
| FOR | 0.500 ± 0.016 | 0.234 ± 0.056 | 0.373 ± 0.068 | 0.479 ± 0.017 | 0.475 ± 0.021 |
| RND | 0.497 ± 0.022 | 0.457 ± 0.039 | 0.495 ± 0.015 | 0.499 ± 0.011 | 0.483 ± 0.021 |
