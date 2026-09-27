# E21: hierarchical generalization — schemas of schemas (negative result, with diagnosis)

Full table: [E21-s1.md](E21-s1.md). Reproduce with `mars-bench e21 [--templates 100,1000] [--theta2 0.3,0.4,0.5] [--l2-z z]` (~30 s).

**Question:** SAGE fragments each hidden template into several generalizations (E7, E15: completeness 0.79–0.81 at purity 0.87–0.96), and merging generalizations directly hurt purity (E7). Can a *second level* recover the templates? Level-1 schemas are kept intact, and SAGE runs again over the level-1 schema cases, which are denoised and so might align where noisy instances don't.

**Setup.**
- E15 memory: 10 noisy instances per template; 100 templates (10³ cases) and 1,000 templates (10⁴ cases).
- Level 1: θ₁ = 0.4 plus sleep. Level 2: SAGE over the level-1 schemas with θ₂ ∈ {0.3, 0.4, 0.5}, and optionally θ₂ ∈ {0.15, 0.2} with E16's local-null significance gate.
- Clusters are scored over the instances they cover.
- Inference uses the E15 protocol, with level-2 schemas added to the best pool (instances + level-1 schemas).

## Results

| memory | level | θ₂ | purity | completeness | R@P≥0.6 / R@P≥0.8 |
|---|---|---|---|---|---|
| 10³ | 1 | — | 0.964 | 0.812 | 0.511 / 0.205 |
| | 2 | 0.3 | 0.768 | 0.857 | 0.505 / 0.195 |
| | 2 | 0.4 | 0.919 | 0.845 | 0.516 / 0.205 |
| | 2 | 0.5 | 0.958 | 0.820 | 0.511 / 0.205 |
| 10⁴ | 1 | — | 0.868 | 0.787 | 0.448 / 0.178 |
| | 2 | 0.3 | 0.592 | 0.837 | 0.450 / 0.180 |
| | 2 | 0.4 | 0.721 | 0.824 | 0.452 / 0.182 |
| | 2 | 0.5 | 0.810 | 0.800 | 0.452 / 0.178 |

Low θ₂ with the significance gate reaches completeness 0.85–0.88. Purity is then 0.43–0.72 at 10³ and 0.30–0.75 at 10⁴. Inference does not improve (R@P≥0.6 0.44–0.52).

**Diagnostic.** Normalized FAC between level-1 schemas:

| memory | same-template pairs | different-template pairs | AUC |
|---|---|---|---|
| 10³ | n = 51, median 0.27 (q10 0.17) | n = 9,540, median 0.07, **q99 0.27**, max 0.58 | 0.975 |
| 10⁴ | n = 522, median 0.31 (q10 0.18) | n = 885,924, median 0.07, **q99 0.27**, max 0.73 | 0.975 |

## Findings

1. **Hierarchy does not repair fragmentation.** Every level-2 setting trades purity for completeness along the same curve as lowering θ₁ or merging directly (E7), and none improves inference.
2. **Why: fragments are rankable but not identifiable.**
   - Fragments of one template are only moderately similar (median FAC 0.27–0.31), because different perturbations delete or insert different structure.
   - They do rank above unrelated schemas (AUC 0.975), but the base rate is tiny: 0.5% of pairs at 10³, 0.06% at 10⁴. Above the same-template median, unrelated pairs outnumber true ones by ~34:1 at 10⁴.
   - No threshold, absolute or significance-gated, can merge fragments without merging more strangers. This is the E10/E16 information limit reappearing at the level of schemas.
3. **Design consequence.** Keep single-level SAGE, and treat fragmentation as a property of the data: a template whose instances are perturbed differently genuinely yields different schemas. The architecture already copes. Instances plus schemas plus corroboration (E15) pool evidence at recall time across fragments, where retrieval ranks them together.
   - Not built: hierarchical generalization (the bench code in `mars-bench e21` remains).
   - The open problem moves to *more identifiable schemas*, e.g. partial-mapping assimilation that keeps a shared core rather than whole-case scores.
