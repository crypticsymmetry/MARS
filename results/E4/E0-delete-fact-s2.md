# E0 — fingerprint separability (delete-fact-s2)

Config: groups=1000, seed=1, naming=Canonical, distractors=2, cases=8000, mean features/case=136.2, runtime=1.3s.

Metrics: pooled ROC-AUC of TA against MA / FOR / RND scores; per-group win rates P(TA > MA), P(TA > FOR); **TA-top** = P(TA outscores MA, FOR and all RND of its group). Ties count 1/2. LS is excluded (it is a legitimate match).

Discriminable groups (TA preserves more base higher-order facts than MA and FOR): 388/1000.

| method | AUC TA/MA | AUC TA/FOR | AUC TA/RND | win TA>MA | win TA>FOR | TA-top (all) | TA-top (dev fam.) | TA-top (test fam.) | TA-top (discriminable) |
|---|---|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF | 0.000 | 0.491 | 0.904 | 0.000 | 0.485 | 0.000 | 0.000 | 0.000 | 0.000 |
| B4 MAC content vectors | 0.013 | 0.489 | 0.845 | 0.006 | 0.481 | 0.006 | 0.002 | 0.012 | 0.010 |
| exact C0 surface | 0.000 | 0.500 | 0.461 | 0.000 | 0.500 | 0.000 | 0.000 | 0.000 | 0.000 |
| exact C1 content | 0.490 | 0.491 | 0.913 | 0.509 | 0.495 | 0.319 | 0.314 | 0.325 | 0.448 |
| exact C2 relational | 0.567 | 0.574 | 0.780 | 0.606 | 0.611 | 0.429 | 0.420 | 0.443 | 0.860 |
| exact C3 WL | 0.606 | 0.621 | 0.891 | 0.665 | 0.682 | 0.517 | 0.503 | 0.535 | 0.812 |
| exact C4 topology | 0.522 | 0.524 | 0.749 | 0.524 | 0.534 | 0.278 | 0.221 | 0.354 | 0.433 |
| exact analogy profile | 0.575 | 0.577 | 0.889 | 0.624 | 0.631 | 0.481 | 0.469 | 0.498 | 0.889 |
| exact analogy profile +IDF | 0.578 | 0.582 | 0.956 | 0.633 | 0.652 | 0.523 | 0.523 | 0.523 | 0.912 |
| exact literal profile +IDF | 0.000 | 0.571 | 0.949 | 0.000 | 0.641 | 0.000 | 0.000 | 0.000 | 0.000 |
| fingerprint analogy +IDF D=1024 | 0.595 | 0.592 | 0.869 | 0.632 | 0.641 | 0.490 | 0.483 | 0.500 | 0.869 |
| fingerprint analogy +IDF D=2048 | 0.589 | 0.589 | 0.899 | 0.627 | 0.633 | 0.501 | 0.504 | 0.498 | 0.894 |
| fingerprint analogy +IDF D=4096 | 0.583 | 0.578 | 0.914 | 0.635 | 0.622 | 0.501 | 0.500 | 0.502 | 0.899 |
| fingerprint analogy +IDF D=8192 | 0.581 | 0.578 | 0.926 | 0.637 | 0.640 | 0.512 | 0.517 | 0.505 | 0.899 |
| fingerprint analogy +IDF D=16384 | 0.580 | 0.579 | 0.939 | 0.627 | 0.644 | 0.507 | 0.510 | 0.502 | 0.907 |
| fingerprint C1 only D=8192 | 0.483 | 0.487 | 0.967 | 0.491 | 0.473 | 0.297 | 0.295 | 0.301 | 0.396 |
| fingerprint C2 only D=8192 | 0.581 | 0.583 | 0.829 | 0.618 | 0.627 | 0.476 | 0.490 | 0.457 | 0.869 |
| fingerprint C3 only D=8192 | 0.633 | 0.628 | 0.804 | 0.669 | 0.676 | 0.499 | 0.504 | 0.493 | 0.856 |
| fingerprint C4 only D=8192 | 0.521 | 0.539 | 0.708 | 0.517 | 0.539 | 0.281 | 0.251 | 0.322 | 0.361 |
| exact mix C2 only +IDF | 0.582 | 0.588 | 0.849 | 0.621 | 0.624 | 0.484 | 0.491 | 0.474 | 0.869 |
| exact mix C3 only +IDF | 0.627 | 0.638 | 0.885 | 0.684 | 0.689 | 0.547 | 0.538 | 0.560 | 0.923 |
| exact mix C4 only +IDF | 0.525 | 0.529 | 0.785 | 0.551 | 0.546 | 0.322 | 0.274 | 0.386 | 0.448 |
| exact mix C2+C3 +IDF | 0.589 | 0.594 | 0.891 | 0.650 | 0.654 | 0.520 | 0.524 | 0.514 | 0.920 |
| exact mix C2+C3+C4 +IDF | 0.587 | 0.591 | 0.902 | 0.640 | 0.647 | 0.507 | 0.502 | 0.514 | 0.912 |
| exact mix C1+C2+C3 +IDF | 0.573 | 0.576 | 0.966 | 0.631 | 0.648 | 0.514 | 0.519 | 0.507 | 0.910 |
| ablation: no taxonomy | 0.583 | 0.587 | 0.968 | 0.633 | 0.645 | 0.522 | 0.523 | 0.521 | 0.918 |
| ablation: no systematicity (beta=0) | 0.572 | 0.576 | 0.961 | 0.622 | 0.640 | 0.511 | 0.514 | 0.507 | 0.899 |
| ablation: no parent-child | 0.516 | 0.535 | 0.969 | 0.558 | 0.559 | 0.408 | 0.400 | 0.418 | 0.644 |
| ablation: no co-entity | 0.576 | 0.576 | 0.940 | 0.645 | 0.634 | 0.516 | 0.512 | 0.521 | 0.928 |
| ablation: co-entity weight 1.0 | 0.562 | 0.569 | 0.973 | 0.604 | 0.615 | 0.490 | 0.495 | 0.484 | 0.838 |
| ablation: co-entity weight 0.5 | 0.574 | 0.580 | 0.965 | 0.623 | 0.637 | 0.513 | 0.516 | 0.509 | 0.897 |
| ablation: no co-expr | 0.581 | 0.585 | 0.957 | 0.639 | 0.653 | 0.528 | 0.530 | 0.526 | 0.925 |
| ablation: WL depth 1 | 0.583 | 0.586 | 0.957 | 0.641 | 0.651 | 0.524 | 0.528 | 0.519 | 0.902 |
| ablation: WL depth 3 | 0.576 | 0.580 | 0.956 | 0.626 | 0.645 | 0.510 | 0.512 | 0.507 | 0.910 |

## TA-top by family

| family | B4 MAC content vectors | exact analogy profile +IDF | fingerprint analogy +IDF D=8192 |
|---|---|---|---|
| chain | 0.000 | 0.524 | 0.510 |
| star | 0.000 | 0.545 | 0.531 |
| tree | 0.000 | 0.448 | 0.469 |
| loop | 0.007 | 0.573 | 0.559 |
| deep-ho (test) | 0.000 | 0.315 | 0.273 |
| comparison (test) | 0.035 | 0.629 | 0.601 |
| mixed (test) | 0.000 | 0.627 | 0.641 |

## Normalized Hamming distance to base, by class (fingerprint D=8192, mean ± sd)

| class | C0 surface | C1 content | C2 relational | C3 WL | C4 topology |
|---|---|---|---|---|---|
| LS | 0.108 ± 0.053 | 0.197 ± 0.053 | 0.097 ± 0.035 | 0.367 ± 0.032 | 0.396 ± 0.065 |
| TA | 0.500 ± 0.015 | 0.302 ± 0.076 | 0.388 ± 0.112 | 0.470 ± 0.031 | 0.462 ± 0.029 |
| MA | 0.107 ± 0.053 | 0.299 ± 0.071 | 0.432 ± 0.079 | 0.485 ± 0.020 | 0.465 ± 0.027 |
| FOR | 0.499 ± 0.015 | 0.298 ± 0.077 | 0.431 ± 0.080 | 0.484 ± 0.019 | 0.467 ± 0.025 |
| RND | 0.496 ± 0.022 | 0.450 ± 0.040 | 0.495 ± 0.016 | 0.499 ± 0.011 | 0.482 ± 0.022 |
