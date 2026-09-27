# E15: pooling evidence at storage (schemas) vs at recall (corroboration)

Tables: [E15-s1.md](E15-s1.md) (severity 1; 100 and 1,000 templates) and [E15-s2.md](E15-s2.md) (severity 2; 100 templates).

Reproduce:
```
mars-bench e15 --templates 100,1000 --cov-theta 0.5 [--severity 2] [--assimilate θ] [--pools inst,sym,cov,both]
```
The severity-1 run takes ~40 s.

**Question:** E7 showed that a *schema* beats the single best instance for few-shot inference. E11 showed that *corroboration* across the top-5 retrieved instances is a calibrated confidence. Both pool evidence over several instances of a pattern: schemas at storage time (SAGE), corroboration at recall time. Which should the architecture rely on, and does the answer hold as memory grows?

**Setup.**
- Memory holds 10 noisy instances of each of T hidden templates (mixed perturbations, 2 distractors).
- Queries are fresh instances with one root higher-order fact deleted: 190 queries at T = 100, 500 at T = 1,000.
- Four pools:
  - **instances**: every memory case.
  - **schemas, symmetric**: SAGE (E7) generalizations + outliers, assimilation θ = 0.4 by symmetric normalized FAC, plus sleep.
  - **schemas, coverage**: new `SageConfig::coverage`. Assimilation into an established generalization is scored by schema coverage S(g→x)/S(g→g).
  - **instances + schemas**: schemas added *alongside* all instances.
- Every pool is queried identically: fingerprint top-16, then fused 0.3·FAC + 0.7·FP, then the top-m items.
- An inference's confidence is Σ over proposing items of its base fact's Laplace probability (count+1)/(members+2). An instance contributes 2/3, so for instances this is E11's support count.
- **R@P≥x** is the best deleted-fact recall reachable by thresholding confidence while keeping precision ≥ x. It is the operating-point metric.

## Results (severity 1)

| T (memory) | pool | items | purity / completeness | template acc | m=1 recall / precision | **R@P≥0.6** (best m) | **R@P≥0.8** (best m) | query ms |
|---|---|---|---|---|---|---|---|---|
| 100 (10³) | instances | 1,000 | — | 0.968 | 0.374 / 0.577 | 0.437 (m=5) | 0.158 (m=3) | 0.28 |
| | schemas, symmetric | 221 | 0.964 / 0.812 | 0.937 | **0.574** / 0.606 | **0.574** (m=1) | 0.105 (m=1) | 0.12 |
| | schemas, coverage θ=0.5 | 303 | 0.993 / 0.729 | 0.968 | 0.542 / 0.582 | 0.521 (m=1) | 0.105 (m=5) | 0.13 |
| | **instances + schemas** | 1,139 | — | 0.974 | 0.479 / **0.664** | 0.511 (m=5) | **0.205** (m=3) | 0.29 |
| 1,000 (10⁴) | instances | 10,000 | — | 0.872 | 0.356 / 0.546 | 0.392 (m=5) | 0.158 (m=3) | 0.92 |
| | schemas, symmetric | 1,972 | **0.868** / 0.787 | 0.788 | 0.458 / 0.541 | 0.072 | 0.032 | 0.25 |
| | schemas, coverage θ=0.5 | 2,951 | 0.929 / 0.700 | 0.820 | 0.474 / 0.542 | 0.168 | 0.014 | 0.34 |
| | **instances + schemas** | 11,332 | — | 0.870 | 0.392 / **0.601** | **0.448** (m=3,5) | **0.178** (m=3) | 1.07 |

Severity 2 (T = 100) shows the same ordering:
- instances + schemas: R@P≥0.6 0.261 (m=3), against 0.228 for instances and ≤ 0.19 for schema-only pools;
- m=1 precision 0.443 vs 0.395.

**Assimilation threshold at T = 1,000** (symmetric, schema-only pool):

| θ | items | purity / completeness | m=1 recall / precision | R@P≥0.6 | R@P≥0.8 |
|---|---|---|---|---|---|
| 0.4 | 1,972 | 0.868 / 0.787 | 0.458 / 0.541 | 0.072 | 0.032 |
| 0.5 | 2,766 | 0.934 / 0.742 | 0.478 / 0.575 | 0.334 | 0.012 |
| 0.6 | 4,034 | 0.954 / 0.611 | 0.506 / 0.576 | 0.256 | 0.144 |
| 0.7 | 6,136 | 0.960 / 0.421 | 0.468 / 0.586 | 0.414 | 0.266 |

Instances + schemas at θ = 0.6: R@P≥0.6 **0.476**, R@P≥0.8 0.192.

## Findings

1. **Schemas complement instances; they should not replace them.** Adding schema cases alongside the instances is the best or tied-best pool at every scale and severity:
   - m=1 precision +0.05–0.09;
   - R@P≥0.8 up to +0.05;
   - at T = 1,000: R@P≥0.6 0.45–0.48 vs 0.39 for instances.
   
   Replacing instances with schemas wins only at small scale, and only for single-analogue inference (m=1 recall 0.574 vs 0.374: E7's result). At T = 1,000 it loses badly.
2. **A fixed assimilation threshold does not scale.**
   - Purity falls from 0.964 (10³ cases) to 0.868 (10⁴) at θ = 0.4. Template accuracy of schema retrieval falls to 0.79.
   - The cause is E10's information limit: small denoised schemas are generic, and with 10× more candidates, chance matches above θ become common.
   - Raising θ restores purity but fragments the templates (completeness 0.79 → 0.42), shrinking the pooling benefit.
   - Like BLAST E-values, the significance of an analogical match must be judged against memory size. **Open item:** a null-calibrated assimilation criterion.
3. **Schema fact probability is not a useful confidence; corroboration is.**
   - At m=1, schema-pool precision is flat in the fact's probability: 0.52 / 0.59 / 0.53 across probability buckets at T = 1,000.
   - Support across independently retrieved items rises cleanly: 0.09 → 0.38 → 0.60 → 0.70 at T = 1,000, as in E11.
   - The dominant error is *retrieving the wrong pattern*. Only recall-time corroboration averages over retrieval errors; storage-time frequency cannot.
4. **Coverage assimilation is a negative result.** It trades completeness for purity along the same curve as raising the symmetric threshold (T = 100: θ_cov 0.4/0.5/0.6 → completeness 0.80/0.73/0.60 at purity 0.96/0.99/0.98). It is kept as an option, off by default.
5. **Cost:**
   - Schemas cut the pool 4.5–5× and the query time 2–4× when they replace instances.
   - Added alongside, they cost ~14% more items for the precision gain.
   - Building costs 0.3–0.8 ms per case. SAGE's candidate scan is linear in the pool; an index is needed beyond ~10⁴ cases.

## Design consequences

- Consolidation **adds** schema cases to memory and keeps the instances. Confidence stays **corroboration** across the combined pool; schema fact probabilities are reported, not used as confidence.
- SAGE schema/entity names now come from a stable per-generalization id and a per-pool namespace (`SageConfig::namespace`). Previously they came from the vector index, which is reused after merges, and two pools in one KB collided.
