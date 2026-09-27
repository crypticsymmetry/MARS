# E5: what does SDM add over plain Hamming k-NN?

## E5b: SDM-bucket candidate generation (corpus 100000, 1000 queries, target TA, R@64 after exact re-scoring)

Reference: exhaustive Mode K, R@64 = 1.000, 0.73 ms/query (batched, 4 threads), 100% of the corpus scored.

| H | addresses | A_write | mean / max bucket | A_query | R@64 | corpus scored | ms/query (1 thread) |
|---|---|---|---|---|---|---|---|
| 1024 | random | 1 | 97.7 / 420 | 1 | 0.037 | 0.13% | 0.09 |
| 1024 | random | 1 | 97.7 / 420 | 4 | 0.083 | 0.48% | 0.16 |
| 1024 | random | 1 | 97.7 / 420 | 16 | 0.209 | 1.83% | 0.34 |
| 1024 | random | 1 | 97.7 / 420 | 64 | 0.465 | 7.06% | 0.97 |
| 1024 | random | 3 | 293.0 / 1076 | 1 | 0.093 | 0.37% | 0.13 |
| 1024 | random | 3 | 293.0 / 1076 | 4 | 0.210 | 1.39% | 0.26 |
| 1024 | random | 3 | 293.0 / 1076 | 16 | 0.450 | 5.31% | 0.75 |
| 1024 | random | 3 | 293.0 / 1076 | 64 | 0.800 | 19.49% | 2.14 |
| 1024 | data | 1 | 97.7 / 360 | 1 | 0.630 | 0.13% | 0.07 |
| 1024 | data | 1 | 97.7 / 360 | 4 | 0.916 | 0.46% | 0.15 |
| 1024 | data | 1 | 97.7 / 360 | 16 | 0.992 | 1.71% | 0.32 |
| 1024 | data | 1 | 97.7 / 360 | 64 | 0.999 | 6.75% | 0.86 |
| 1024 | data | 3 | 293.0 / 912 | 1 | 0.884 | 0.36% | 0.13 |
| 1024 | data | 3 | 293.0 / 912 | 4 | 0.993 | 1.25% | 0.24 |
| 1024 | data | 3 | 293.0 / 912 | 16 | 1.000 | 4.43% | 0.63 |
| 1024 | data | 3 | 293.0 / 912 | 64 | 1.000 | 15.36% | 1.75 |
| 1024 | k-means(2) | 1 | 97.7 / 386 | 1 | 0.872 | 0.14% | 0.07 |
| 1024 | k-means(2) | 1 | 97.7 / 386 | 4 | 0.990 | 0.46% | 0.12 |
| 1024 | k-means(2) | 1 | 97.7 / 386 | 16 | 1.000 | 1.51% | 0.26 |
| 1024 | k-means(2) | 1 | 97.7 / 386 | 64 | 1.000 | 6.23% | 0.78 |
| 1024 | k-means(2) | 3 | 293.0 / 1037 | 1 | 0.988 | 0.36% | 0.12 |
| 1024 | k-means(2) | 3 | 293.0 / 1037 | 4 | 1.000 | 1.17% | 0.22 |
| 1024 | k-means(2) | 3 | 293.0 / 1037 | 16 | 1.000 | 3.84% | 0.56 |
| 1024 | k-means(2) | 3 | 293.0 / 1037 | 64 | 1.000 | 13.70% | 1.47 |
| 4096 | random | 1 | 24.4 / 167 | 1 | 0.016 | 0.03% | 0.13 |
| 4096 | random | 1 | 24.4 / 167 | 4 | 0.050 | 0.13% | 0.17 |
| 4096 | random | 1 | 24.4 / 167 | 16 | 0.147 | 0.49% | 0.26 |
| 4096 | random | 1 | 24.4 / 167 | 64 | 0.282 | 1.87% | 0.46 |
| 4096 | random | 3 | 73.2 / 414 | 1 | 0.035 | 0.10% | 0.17 |
| 4096 | random | 3 | 73.2 / 414 | 4 | 0.118 | 0.38% | 0.24 |
| 4096 | random | 3 | 73.2 / 414 | 16 | 0.319 | 1.44% | 0.42 |
| 4096 | random | 3 | 73.2 / 414 | 64 | 0.587 | 5.43% | 1.00 |
| 4096 | data | 1 | 24.4 / 174 | 1 | 0.638 | 0.04% | 0.13 |
| 4096 | data | 1 | 24.4 / 174 | 4 | 0.917 | 0.13% | 0.16 |
| 4096 | data | 1 | 24.4 / 174 | 16 | 0.988 | 0.45% | 0.25 |
| 4096 | data | 1 | 24.4 / 174 | 64 | 0.999 | 1.65% | 0.44 |
| 4096 | data | 3 | 73.2 / 300 | 1 | 0.880 | 0.10% | 0.17 |
| 4096 | data | 3 | 73.2 / 300 | 4 | 0.993 | 0.33% | 0.24 |
| 4096 | data | 3 | 73.2 / 300 | 16 | 1.000 | 1.13% | 0.37 |
| 4096 | data | 3 | 73.2 / 300 | 64 | 1.000 | 3.97% | 0.70 |
| 4096 | k-means(2) | 1 | 24.4 / 178 | 1 | 0.796 | 0.04% | 0.17 |
| 4096 | k-means(2) | 1 | 24.4 / 178 | 4 | 0.979 | 0.15% | 0.23 |
| 4096 | k-means(2) | 1 | 24.4 / 178 | 16 | 1.000 | 0.47% | 0.29 |
| 4096 | k-means(2) | 1 | 24.4 / 178 | 64 | 1.000 | 1.64% | 0.43 |
| 4096 | k-means(2) | 3 | 73.2 / 446 | 1 | 0.958 | 0.11% | 0.18 |
| 4096 | k-means(2) | 3 | 73.2 / 446 | 4 | 1.000 | 0.41% | 0.23 |
| 4096 | k-means(2) | 3 | 73.2 / 446 | 16 | 1.000 | 1.05% | 0.33 |
| 4096 | k-means(2) | 3 | 73.2 / 446 | 64 | 1.000 | 3.45% | 0.65 |

## E5a: partial-cue retrieval (corpus 100000; cue = base with a fraction of facts removed)

*Self* = top-1 is the full base (or its literal twin LS). *Analogy* = rank of TA with base and LS excluded. Mode A: autoassociative SDM storing all 100000 fingerprints; cleanup = iterated read-out used as the query. kNN-bundle = majority of the cue's top-10 Mode K neighbours (explicit cleanup baseline).

| removed | method | self@1 | analogy R@1 | analogy R@10 |
|---|---|---|---|---|
| 25% | Mode K on the raw cue | 0.947 | 0.820 | 0.945 |
| 25% | kNN-bundle(10) cleanup | 0.618 | 0.746 | 0.962 |
| 25% | Mode A cleanup (H=8192, A=16, 3 iters) | 0.026 | 0.020 | 0.097 |
| 25% | Mode A cleanup (H=32768, A=64, 3 iters) | 0.023 | 0.020 | 0.095 |
| 50% | Mode K on the raw cue | 0.647 | 0.511 | 0.710 |
| 50% | kNN-bundle(10) cleanup | 0.427 | 0.521 | 0.738 |
| 50% | Mode A cleanup (H=8192, A=16, 3 iters) | 0.016 | 0.019 | 0.079 |
| 50% | Mode A cleanup (H=32768, A=64, 3 iters) | 0.014 | 0.012 | 0.067 |
| 75% | Mode K on the raw cue | 0.217 | 0.194 | 0.383 |
| 75% | kNN-bundle(10) cleanup | 0.207 | 0.247 | 0.406 |
| 75% | Mode A cleanup (H=8192, A=16, 3 iters) | 0.010 | 0.010 | 0.033 |
| 75% | Mode A cleanup (H=32768, A=64, 3 iters) | 0.008 | 0.008 | 0.032 |

## E5c: prototype emergence (100 hidden templates × 100 stored noisy instances; 500 held-out cues; perturbation severity 1, 2 distractors)

*Sim to prototype* = analogy-profile similarity of the method's output to the template's clean instance (never stored). *Template acc* = nearest clean prototype is the right template.

| method | sim to prototype | template acc |
|---|---|---|
| cue itself | 0.411 | 0.986 |
| nearest stored neighbour | 0.447 | 0.978 |
| kNN-bundle(10) | 0.620 | 0.984 |
| kNN-bundle(50) | 0.704 | 0.982 |
| SDM read-out (H=2048, A=16, 1 iter) | 0.707 | 0.978 |
| SDM read-out (H=2048, A=16, 3 iter) | 0.706 | 0.962 |
| SDM read-out (H=2048, A=64, 1 iter) | 0.428 | 0.778 |
| SDM read-out (H=2048, A=64, 3 iter) | 0.342 | 0.298 |
| SDM read-out (H=8192, A=64, 1 iter) | 0.713 | 0.980 |
| SDM read-out (H=8192, A=64, 3 iter) | 0.713 | 0.978 |
