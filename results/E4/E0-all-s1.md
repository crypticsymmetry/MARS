# E0 — fingerprint separability (all-s1)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=154.6, runtime=1.5s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 731/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.506 | 0.953 | 0.000 | 0.502 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.005 | 0.503 | 0.973 | 0.001 | 0.498 | 0.001 | 0.000 | 0.001 | 0.001 |
| exact C0 surface | 0.000 | 0.500 | 0.463 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.508 | 0.502 | 0.996 | 0.512 | 0.519 | 0.348 | 0.343 | 0.356 | 0.356 |
| exact C2 relational | 0.863 | 0.862 | 0.986 | 0.892 | 0.890 | 0.864 | 0.827 | 0.914 | 0.891 |
| exact C3 WL | 0.878 | 0.877 | 0.996 | 0.941 | 0.950 | 0.925 | 0.909 | 0.946 | 0.952 |
| exact C4 topology | 0.580 | 0.572 | 0.941 | 0.580 | 0.597 | 0.425 | 0.344 | 0.534 | 0.427 |
| exact analogy profile | 0.868 | 0.865 | 0.996 | 0.921 | 0.920 | 0.897 | 0.883 | 0.916 | 0.932 |
| exact analogy profile +IDF | 0.881 | 0.880 | 0.999 | 0.918 | 0.919 | 0.896 | 0.881 | 0.916 | 0.933 |
| exact literal profile +IDF | 0.001 | 0.858 | 0.998 | 0.000 | 0.909 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.867 | 0.866 | 0.992 | 0.903 | 0.902 | 0.875 | 0.858 | 0.897 | 0.912 |
| fingerprint analogy +IDF D=2048 | 0.870 | 0.869 | 0.994 | 0.908 | 0.898 | 0.871 | 0.846 | 0.904 | 0.912 |
| fingerprint analogy +IDF D=4096 | 0.872 | 0.871 | 0.997 | 0.910 | 0.905 | 0.878 | 0.857 | 0.907 | 0.922 |
| fingerprint analogy +IDF D=8192 | 0.873 | 0.871 | 0.998 | 0.913 | 0.914 | 0.888 | 0.869 | 0.914 | 0.926 |
| fingerprint analogy +IDF D=16384 | 0.872 | 0.871 | 0.998 | 0.914 | 0.917 | 0.891 | 0.874 | 0.914 | 0.927 |
| fingerprint C1 only D=8192 | 0.513 | 0.508 | 0.999 | 0.533 | 0.503 | 0.361 | 0.369 | 0.352 | 0.384 |
| fingerprint C2 only D=8192 | 0.874 | 0.872 | 0.993 | 0.898 | 0.892 | 0.871 | 0.838 | 0.914 | 0.894 |
| fingerprint C3 only D=8192 | 0.903 | 0.904 | 0.985 | 0.922 | 0.932 | 0.896 | 0.891 | 0.903 | 0.933 |
| fingerprint C4 only D=8192 | 0.578 | 0.560 | 0.871 | 0.589 | 0.561 | 0.405 | 0.316 | 0.522 | 0.412 |
| exact mix C2 only +IDF | 0.876 | 0.875 | 0.992 | 0.899 | 0.892 | 0.872 | 0.839 | 0.916 | 0.897 |
| exact mix C3 only +IDF | 0.912 | 0.914 | 0.997 | 0.942 | 0.950 | 0.927 | 0.921 | 0.935 | 0.973 |
| exact mix C4 only +IDF | 0.581 | 0.572 | 0.933 | 0.626 | 0.603 | 0.463 | 0.372 | 0.584 | 0.461 |
| exact mix C2+C3 +IDF | 0.899 | 0.900 | 0.996 | 0.921 | 0.928 | 0.904 | 0.888 | 0.925 | 0.941 |
| exact mix C2+C3+C4 +IDF | 0.881 | 0.881 | 0.995 | 0.917 | 0.923 | 0.896 | 0.883 | 0.914 | 0.932 |
| exact mix C1+C2+C3 +IDF | 0.879 | 0.878 | 0.999 | 0.916 | 0.918 | 0.895 | 0.879 | 0.916 | 0.936 |
| ablation: no taxonomy | 0.885 | 0.884 | 0.999 | 0.919 | 0.921 | 0.897 | 0.881 | 0.918 | 0.934 |
| ablation: no systematicity (beta=0) | 0.862 | 0.861 | 0.999 | 0.916 | 0.919 | 0.897 | 0.885 | 0.914 | 0.938 |
| ablation: no parent-child | 0.665 | 0.658 | 0.999 | 0.832 | 0.829 | 0.766 | 0.724 | 0.822 | 0.803 |
| ablation: no co-entity | 0.886 | 0.885 | 0.997 | 0.917 | 0.918 | 0.895 | 0.879 | 0.916 | 0.934 |
| ablation: co-entity weight 1.0 | 0.834 | 0.834 | 0.999 | 0.911 | 0.916 | 0.883 | 0.874 | 0.895 | 0.936 |
| ablation: co-entity weight 0.5 | 0.866 | 0.866 | 0.999 | 0.916 | 0.921 | 0.896 | 0.881 | 0.916 | 0.937 |
| ablation: no co-expr | 0.887 | 0.886 | 0.999 | 0.919 | 0.921 | 0.899 | 0.883 | 0.921 | 0.937 |
| ablation: WL depth 1 | 0.886 | 0.884 | 0.999 | 0.922 | 0.923 | 0.901 | 0.888 | 0.918 | 0.938 |
| ablation: WL depth 3 | 0.877 | 0.877 | 0.999 | 0.919 | 0.918 | 0.897 | 0.881 | 0.918 | 0.933 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.951 | 0.958 |
| star | 0.000 | 0.909 | 0.909 |
| tree | 0.000 | 0.902 | 0.888 |
| loop | 0.000 | 0.762 | 0.720 |
| deep-ho (test) | 0.000 | 0.853 | 0.853 |
| comparison (test) | 0.003 | 0.958 | 0.944 |
| mixed (test) | 0.000 | 0.937 | 0.944 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.202 ± 0.054 | 0.100 ± 0.035 | 0.371 ± 0.031 | 0.403 ± 0.063 |
| TA | 0.500 ± 0.015 | 0.228 ± 0.063 | 0.186 ± 0.098 | 0.410 ± 0.038 | 0.427 ± 0.055 |
| MA | 0.114 ± 0.055 | 0.230 ± 0.062 | 0.355 ± 0.101 | 0.472 ± 0.024 | 0.440 ± 0.046 |
| FOR | 0.499 ± 0.015 | 0.229 ± 0.064 | 0.355 ± 0.102 | 0.472 ± 0.024 | 0.438 ± 0.045 |
| RND | 0.496 ± 0.022 | 0.455 ± 0.039 | 0.495 ± 0.014 | 0.499 ± 0.011 | 0.483 ± 0.021 |
