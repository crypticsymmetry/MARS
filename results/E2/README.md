# E2: mapper correctness and cost (summary)

**Setup:** the same generator as E0 (1,000 groups, seed 1, canonical naming). The greedy SME-class mapper (`mars-map`, `MapConfig::default()`) is run for base→LS, base→TA and base→TA⁻, where TA⁻ is TA with one root higher-order fact deleted. Normalized FAC score = `S(b,x) / √(S(b,b)·S(x,x))`. Reproduce with `mars-bench e2 --groups 1000 --distractors k`.

| distractors | TA entity corr. P / R | FAC TA-top | greedy / optimal merge | CI recall (deleted fact re-inferred) | structural CI precision |
|---|---|---|---|---|---|
| 0 | 1.000 / 0.994 | 1.000 | 1.0000 | 0.999 | 0.999 |
| 2 | 1.000 / 0.993 | 1.000 | 1.0000 | 0.992 | 0.992 |
| 5 | 0.999 / 0.992 | 1.000 | 0.9997 | 0.984 | 0.986 |
| 10 | 0.995 / 0.988 | 0.994 | 0.9994 | 0.918 | 0.934 |

Time per mapping (single thread): **16 µs** mean for pairs with 20–30 expressions, **24 µs** for 30–40, **38 µs** for 40–60. The design budget was 200 µs at about 50 facts, so FAC over 64 candidates costs about 1–2.5 ms on one core.

## Findings

1. **The greedy merge is essentially optimal** on these case sizes: 0.9994–1.0000 of the branch-and-bound optimum wherever that was computable (≤ 22 kernels).
2. **Correspondences are right.** Precision ≈ 1.0. The ~1% recall loss comes from the *mixed* family, where some template variables appear only in attribute facts and are correctly left unmapped in analogy mode.
3. **Candidate inference works as intended.** A deleted higher-order fact is projected back with the correct target entities 99% of the time with few distractors. Distractors reduce this (92% at 10) by creating competing kernels that claim entities first. That is where the mapper, not the fingerprint, is the weak link.
4. **FAC alone separates TA from MA/FOR perfectly** (1.000) once the generator's label noise was fixed. E2 *found* that noise: the mapper's "failures" on deep-ho were isomorphic FOR variants caused by commutative `and`.
