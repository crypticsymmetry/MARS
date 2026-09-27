# E16: is this analogy real? Match significance vs memory size

Full table: [E16-p5-s1.md](E16-p5-s1.md). Reproduce with `mars-bench e16 --templates 200,2000,20000,200000` (~75 s, 10³–10⁶ cases).

**Question:** when should MARS say "no reliable analogue exists"? E10 and E15 both hit the same wall: the best *chance* match among N items rises with N, so a raw similarity threshold means different things at different memory sizes. A threshold tuned on a small memory silently accepts garbage on a large one.

**Setup.**
- Memory holds 5 noisy instances per hidden template (severity 1, 2 distractors), with N = 10³ to 10⁶ cases.
- There are 500 queries: 250 fresh instances of stored templates, and 250 instances of templates *absent* from memory (open set).
- Retrieval is the fingerprint top-64, re-ranked by fused 0.3·FAC + 0.7·FP. The top-1 is *correct* if it shares the query's template.
- Each candidate score decides accept/reject for the top-1. Its threshold is fixed at N = 10³ for precision ≥ 0.9 and **reused unchanged** at 10⁴, 10⁵ and 10⁶.

Candidate scores:
- **raw**: fused score; normalized FAC; fingerprint score; margin to #2.
- **global null** (fingerprint): μ, σ of the query's fingerprint scores against 2,048 random memory items; E = N·Q((fp₁ − μ)/σ); score = −log₁₀E (a BLAST-style E-value).
- **local null**: z of the top-1 against the lower half of the candidate list (fingerprint ranks 32–63), which are the best chance matches in *this* memory, for this query. Fused and fingerprint-only variants.

## Results

Threshold transfer. Cells: precision / recall of correct top-1s / fraction of absent-template queries wrongly accepted.

| score | τ (set at 10³) | N = 10³ | 10⁴ | 10⁵ | 10⁶ |
|---|---|---|---|---|---|
| fused score | 0.286 | 0.90 / 0.93 / 0.10 | 0.81 / 0.95 / 0.21 | 0.64 / 1.00 / 0.45 | **0.44** / 1.00 / **0.79** |
| normalized FAC | 0.469 | 0.90 / 0.81 / 0.08 | 0.78 / 0.76 / 0.17 | 0.62 / 0.85 / 0.39 | 0.46 / 0.86 / 0.61 |
| fingerprint score | 0.229 | 0.90 / 0.92 / 0.10 | 0.77 / 0.92 / 0.26 | 0.64 / 0.97 / 0.43 | 0.43 / 0.95 / 0.82 |
| margin to #2 | 0.095 | 0.91 / 0.08 / 0.01 | 0.91 / 0.12 / 0.01 | 0.83 / 0.11 / 0.01 | 0.80 / 0.18 / 0.02 |
| fingerprint −log₁₀E (global null) | 10.3 | 0.90 / 0.81 / 0.08 | 0.73 / 0.92 / 0.30 | 0.55 / 0.91 / 0.56 | 0.44 / 0.94 / 0.77 |
| **fused z (local null)** | **9.0** | 0.90 / 0.93 / 0.10 | **0.91** / 0.88 / 0.08 | **0.91** / 0.77 / 0.06 | **0.95** / 0.49 / **0.01** |
| fingerprint z (local null, FAC-free) | 44 | 0.90 / 0.67 / 0.07 | 0.95 / 0.55 / 0.02 | 0.93 / 0.53 / 0.02 | 0.87 / 0.41 / 0.02 |

Separation (AUC, correct vs wrong top-1) degrades similarly for all good scores as N grows:
- fused score: 0.976 → 0.885;
- fused local z: 0.972 → 0.868.

This is E10's information-per-case limit. The difference between the scores is **calibration**, not separability.

## Findings

1. **Raw thresholds silently fail as memory grows.** A fused-score threshold with precision 0.90 at 10³ has precision 0.44 at 10⁶. It accepts a "real analogue" for 79% of queries whose template is not in memory at all. The same holds for FAC, fingerprint and the Gaussian E-value.
2. **A local null makes the threshold memory-size invariant.** The z of the top-1 against the tail of its own candidate list keeps precision at 0.90–0.95 across three orders of magnitude, with the threshold never re-tuned. False acceptance of absent templates *falls* (0.10 → 0.01).
   - The tail of the top-64 is an automatic, per-query estimate of the best chance matches at the current memory size. It comes for free from the retrieval MAC already runs.
   - As memory grows, the criterion becomes conservative rather than permissive: recall falls to 0.49 at 10⁶, because true matches become less distinguishable (the AUC drop). That is the right failure direction for a reasoning system.
3. **The parametric global null fails.** Fingerprint scores are not Gaussian in the tail, so E = N·Q(z) under-estimates chance matches. Empirical, local tails beat parametric, global ones.
4. **The margin to #2 is useless as a primary criterion** (AUC ~0.68): memory holds several true analogues (instances of the same template), so a real match's margin is small.

## Framework changes

- `Engine::query_significance(q, k)` returns the ranked analogues plus the local-null z of the top-1. `mars_engine::SIGNIFICANT_Z = 9.0` (E16's τ).
- `mars serve`'s `query` prints `significance z = … (accept | abstain: no reliable analogue)`.
- A unit test checks that a deep isomorph is significant while a random chain query is not.
- `Engine::query` now excludes the query case itself from its own results.

## Next

- Use the same criterion for SAGE assimilation (E15 finding 2: a fixed θ loses purity as memory grows).
- Recover recall at 10⁶: a richer null (tail beyond rank 64, or decoys), or more information per case (E10: multi-template cases).

## Addendum: can recall at 10⁶ be recovered?

**Argument.** Under an exponential chance-score tail, a z-score against fixed tail ranks is equivalent to an E-value: the expected number of chance matches at least as good *in the whole memory*. A real match of fixed strength therefore *should* become less significant as memory grows. The recall drop at 10⁶ is the information limit of the case (E10), not miscalibration, and a better null cannot remove it. Only more evidence per decision can.

**Test: sibling pooling.** A real analogue usually has *siblings* in memory (other instances of its pattern) that also rank high for the query. New scores:
- sibling support: other top-16 candidates mapping onto the top-1 at FAC ≥ 0.5;
- the Stouffer combination Σz/√m of the local z of the top-1 and its siblings. It is parameter-free and reduces to z without siblings.

| score | AUC 10³ / 10⁴ / 10⁵ / 10⁶ | precision / recall / absent false-accept at 10⁶ (τ from 10³) |
|---|---|---|
| fused z (local null) | 0.972 / 0.961 / 0.940 / 0.868 | 0.951 / 0.492 / 0.012 |
| sibling count alone | 0.690 / 0.736 / 0.742 / 0.626 | 0.428 / 0.299 / 0.212 |
| **fused z + siblings (Stouffer)** | 0.972 / 0.969 / 0.957 / **0.895** | 0.945 / **0.523** / 0.016 |

- Pooling recovers a little: +3 points recall at 10⁶ and +2 at 10⁵ (0.790 vs 0.767), with memory-size-invariant precision.
- The sibling *count* is uninformative: every stored item, chance matches included, has siblings. Only siblings that also match the query carry evidence.
- Consistent with the argument, recall at 10⁶ is bounded by per-case information. E10's remedy (richer cases: 2-template cases reach 0.996 at 10⁶) is the lever, not the acceptance rule.
