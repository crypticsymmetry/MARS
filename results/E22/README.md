# E22: what score identifies membership in a schema? (negative result)

Full table: [E22-s1.md](E22-s1.md). Reproduce with `mars-bench e22 [--core 0.7]` (~6 s).

**Question:** E21 left one open idea. Whole-case similarity cannot identify a template's fragments, but perhaps a schema's high-probability *core* is a sharper membership test. The assimilation decision is isolated here.
- Level-1 schemas are built over the E15 memory.
- Test cases: fresh instances of stored templates, plus 250 instances of absent templates.
- Each score picks the best of the fingerprint top-16 schemas; the decision is correct if that schema's majority template is the case's template.

## Results

| memory | score | best schema correct | AUC correct vs wrong | recall @ precision ≥ 0.9 | recall @ precision ≥ 0.95 |
|---|---|---|---|---|---|
| 10³ | **symmetric FAC (SAGE default)** | 0.940 | **0.965** | **0.780** | **0.760** |
| | coverage | 0.950 | 0.940 | 0.610 | 0.380 |
| | core fraction (p ≥ 0.7) | 0.820 | 0.855 | 0 | 0 |
| | √(core × symmetric) | 0.910 | 0.950 | 0.690 | 0.550 |
| 10⁴ | **symmetric FAC** | 0.826 | **0.908** | **0.674** | **0.588** |
| | coverage | 0.792 | 0.874 | 0.526 | 0.386 |
| | core fraction | 0.636 | 0.679 | 0 | 0 |
| | √(core × symmetric) | 0.800 | 0.883 | 0.554 | 0.424 |

## Findings

1. **SAGE's default symmetric score is the best membership test of those tried.** Coverage (E15's option) and core-based scores are worse at both scales.
2. **Cores are too small to be identifying.** Cores average ~5 facts, and many unrelated cases of similar shape match most of them (core fraction AUC 0.68 at 10⁴). Penalizing a case for its extra structure, which the symmetric normalization does, is what separates members from look-alikes.
3. **This closes the schema-identifiability line (E15, E21, E22).** At 10⁴ cases even the best score reaches only 0.67 recall at precision 0.9. Membership is information-limited in the same way retrieval is (E10, E16). The architecture's answer stays the same: keep instances, add schemas, pool evidence at recall time (E15), and gate decisions by memory-size-invariant significance (E16).
