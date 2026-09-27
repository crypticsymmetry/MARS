# E4: robustness to structural perturbation

Full tables: [SUMMARY_TABLE.md](SUMMARY_TABLE.md). Raw data: `E0-<op>-s<k>.*` (fingerprints and baselines) and `E2-<op>-s<k>.*` (mapper, fused). Reproduce with `scripts/e4_sweep.sh` (about 40 s).

**Setup:** as in E0/E2 (1,000 groups, 2 distractors, canonical naming), but TA, MA and FOR each independently receive `severity` random perturbations. Operators: `delete-fact` (with cascade), `insert-intermediate` (r(a,b) → r(a,x), r′(x,b)), `substitute-predicate` (sibling or random), `swap-args`, `add-ho` (spurious higher-order link), and `all` (a random mix). TA is now only a *partial* analogue.

## Headline (TA-top, all groups)

| operator, severity | MAC | fingerprint D=8192 | FAC (mapper) | **fused ½FAC+½FP** |
|---|---|---|---|---|
| mixed, 1 | 0.001 | 0.888 | 0.868 | **0.951** |
| mixed, 2 | 0.001 | 0.813 | 0.754 | **0.855** |
| mixed, 4 | 0.004 | 0.672 | 0.595 | **0.698** |
| swap-args, 4 | 0.000 | 0.996 | 0.525 | **0.903** |
| insert-intermediate, 4 | 0.000 | 0.995 | 0.695 | **0.977** |
| add-ho, 4 | 0.001 | 0.899 | 0.970 | **0.970** |
| delete-fact, 1 / 4 | 0.000 | 0.704 / 0.350 | 0.813 / 0.375 | **0.782 / 0.382** |

## Findings

1. **MAC and FAC fail in complementary ways.**
   - *Fingerprints* use local features (parent→child functor links, WL neighbourhoods). They are nearly invariant to argument-order errors and inserted intermediates (≥ 0.99 at severity 4), but spurious additions dilute them (add-ho: 0.90).
   - *The SME-style mapper* enforces parallel connectivity strictly. One role reversal breaks a whole higher-order kernel (swap-args severity 4: 0.53), but spurious additions barely hurt it (0.97).
2. **Fusion beats both.** ½·normalized FAC + ½·fingerprint score is ≥ max(FAC, FP) in almost every cell, and on *discriminable* groups it reaches 0.96–1.00 for every operator and severity. **Design consequence: the FAC stage should re-rank with a fused score, not the structural score alone.** Recorded in the decision log.
3. **Deletions destroy the evidence itself.** "Discriminable" means TA still preserves more of the base's higher-order facts than MA and FOR (ground truth from the generator). Under delete-fact that holds for only 69% → 39% → 18% → 6.5% of groups at severity 1–4. All methods collapse toward chance on the rest, *as they should*. On the discriminable subset every method stays at 0.90–0.99. The all-groups collapse is label ambiguity, not encoder failure.
4. The discriminability criterion is strict: any change to a higher-order fact's arguments counts as "not preserved". So for insert/swap/substitute it underestimates recoverability, and methods still score well on "non-discriminable" groups (e.g. fused insert-intermediate is 0.977 overall).
5. **Substitute-predicate is the hardest *recoverable* perturbation for fingerprints** (0.92–0.97 on discriminable groups). A random predicate substitution breaks both the level-0 and the taxonomy-level features. The mapper's minimal ascension recovers sibling substitutions better.

## Implications

- Adopt **fused re-ranking** in the FAC stage (E3 will use and test it at scale).
- Mapper robustness to role noise is an open design question. Options: a "soft" parallel-connectivity mode (allow a kernel with a mismatched argument at a penalty), or use the fingerprint's tolerance and let fusion handle it (the current choice).
