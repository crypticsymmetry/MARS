# E5: what does Sparse Distributed Memory add over plain Hamming k-NN? (gate G4 / H6)

Full tables: [E5-main.md](E5-main.md). Reproduce with `mars-bench e5 --groups 12500 --queries 1000 --templates 100 --per 100` (about 95 s on 4 cores). All SDM variants address hard locations with the same weighted segment distance as Mode K (analogy profile) and use top-A activation.

## E5b: candidate generation (100K corpus, target TA, exact re-scoring of the candidates)

Reference: exhaustive Mode K, R@64 = 1.000, 0.73 ms/query (batched, 4 threads; about 2.9 ms single-thread equivalent).

| addresses (H = 4096, A_write = 1) | A_query | R@64 | corpus scored | ms/query (1 thread) |
|---|---|---|---|---|
| **random (Kanerva)** | 16 / 64 | 0.147 / 0.282 | 0.49% / 1.87% | 0.26 / 0.46 |
| random, H = 1024, A_write = 3 | 64 | 0.800 | 19.5% | 2.14 |
| data-sampled | 16 / 64 | 0.988 / 0.999 | 0.45% / 1.65% | 0.25 / 0.44 |
| **k-means (2 iters) = IVF** | 16 | **1.000** | **0.47%** | **0.29** |

**Random addresses are the wrong geometry.** Fingerprints occupy a tiny, structured region of {0,1}^D. Measured from uniformly random addresses, every data point is about equally far away, so activation is dominated by noise. The ~10× single-thread speed-up at 10⁵ comes entirely from data-driven addresses, and that is IVF.

## E5a: partial-cue retrieval (100K corpus; the cue is the base with 25–75% of its facts removed)

| removed | Mode K on raw cue: self@1 / analogy R@10 | kNN-bundle(10) cleanup | Mode A cleanup (best of H = 8192 / 32768) |
|---|---|---|---|
| 25% | **0.947** / 0.945 | 0.618 / **0.962** | 0.026 / 0.097 |
| 50% | **0.647** / 0.710 | 0.427 / **0.738** | 0.016 / 0.079 |
| 75% | **0.217** / 0.383 | 0.207 / **0.406** | 0.010 / 0.033 |

**Autoassociative cleanup is destructive at this load.** 100K stored patterns over 8K–32K locations means each read sums hundreds of unrelated patterns, and partial cues start far outside the critical distance anyway (DESIGN §7.5). Explicit k-NN bundling slightly improves *analogy* recall, but it hurts self-retrieval.

## E5c: prototype emergence (100 hidden templates × 100 stored noisy instances; 500 held-out cues)

| method | similarity to the (never stored) clean template | template accuracy |
|---|---|---|
| cue itself | 0.411 | 0.986 |
| nearest stored neighbour | 0.447 | 0.978 |
| kNN-bundle(10) | 0.620 | 0.984 |
| kNN-bundle(50) | 0.704 | 0.982 |
| **SDM read-out (H = 8192, A = 64)** | **0.713** | 0.980 |
| SDM read-out (H = 2048, A = 64, 3 iters) | 0.342 | 0.298 |

**SDM does extract prototypes**: superimposing about 100 similar instances recovers the hidden template better than any single instance. But **explicit k-NN bundling does the same** (0.704 vs 0.713), and it is sensitive to neither H/A tuning nor iteration (SDM with poorly chosen H/A collapses).

## Gate G4 decision

H6 asked for a significant gain on at least one of partial-cue retrieval, prototype quality or continual-learning interference. **Not met.**

- **Drop** Mode A (autoassociative SDM) and random-address SDM from the core retrieval path.
- **Keep** Mode B with *learned* addresses as the sublinear candidate index for ≥ 10⁶ cases. It is IVF under another name, and it should be credited as such.
- Prototype formation for consolidation (P6) will use explicit k-NN bundles + SAGE-style generalization. SDM stays available as an optional fixed-memory variant (it matches k-NN bundling when tuned).
- This is the "Failure A" the original concept anticipated ("SDM retrieval isn't noticeably better than an ordinary Hamming ANN index"). The architecture's value comes from the *structural fingerprint + exact mapper* combination, not from SDM.
