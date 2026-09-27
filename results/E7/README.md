# E7: consolidation — SAGE-style generalization (H4)

Full tables: `E7-s1.md`, `E7-s2.md`. Reproduce with `mars-bench e7 --templates 100 --per 60 --severity {1,2} --queries-per-template 5` (a few seconds each).

**Setup:** `template_instances` draws 100 hidden templates (7 families) and generates noisy instances of each: random domain, mixed perturbations (severity 1 or 2), and 2 distractors. A *clean* instance of each template is kept aside, never stored, as ground truth. `mars-engine::sage` implements SAGE-style generalization on top of the MARS fingerprint + mapper:
- **Assimilation:** assimilate into the best generalization if normalized FAC ≥ θ. Otherwise the case is an outlier, and an outlier that later matches seeds a new generalization.
- **Fact probabilities:** each fact carries count / members; facts below 0.2 wear away.
- **Materialization:** the facts with p ≥ 0.5 become a retrievable `Schema` case.
- **Sleep:** an idle consolidation pass re-offers outliers and merges generalizations that map onto each other at ≥ 0.6.

## E7a: template recovery (100 templates × 60 instances)

| severity | θ | generalizations (after sleep) | purity | completeness | schema fact recall / **precision** | single instance recall / precision |
|---|---|---|---|---|---|---|
| 1 | 0.3 | 205 | 0.941 | 0.797 | 0.746 / **0.901** | 0.714 / 0.536 |
| 1 | 0.4 | 286 | 0.983 | 0.795 | 0.741 / **0.854** | 0.713 / 0.544 |
| 1 | 0.6 | 461 | 0.981 | 0.716 | 0.773 / 0.812 | 0.755 / 0.557 |
| 2 | 0.4 | 529 | 0.943 | 0.580 | 0.620 / **0.780** | 0.621 / 0.464 |

## E7b: few-shot inference — schema vs best single instance (θ = 0.4)

A query is a fresh noisy instance with one root higher-order fact deleted. The best pool item (fingerprint prefilter + fused FAC) supplies candidate inferences.

| severity | M instances / template | pool | template acc | **deleted-fact recall** | CI precision |
|---|---|---|---|---|---|
| 1 | 1 | instances / schemas | 0.906 / 0.854 | 0.480 / 0.468 | 0.534 / 0.517 |
| 1 | 3 | instances / schemas | 0.910 / 0.884 | 0.452 / **0.513** | 0.571 / **0.618** |
| 1 | 10 | instances / schemas | 0.951 / 0.930 | 0.420 / **0.591** | 0.696 / **0.717** |
| 2 | 10 | instances / schemas | 0.880 / 0.860 | 0.287 / **0.362** | 0.486 / **0.562** |

## Findings

1. **H4 is supported for inference.** Schemas beat the single best instance at re-inferring a deleted fact once M ≥ 3, and the gap grows with M: +0.17 at M = 10, severity 1, i.e. 41% more recovered facts. CI precision rises too. With only instances, recall actually *falls* as memory grows (0.48 → 0.42): more near-duplicates that happen to lack the fact compete for "best match". Schemas pool evidence instead.
2. **Schemas denoise.** Schema facts match the clean template with precision 0.85–0.90, against 0.54 for a raw instance. Distractors and perturbation artifacts wear away, while recall is kept (slightly higher).
3. **Clusters are pure but fragmented.** Purity is ≥ 0.94 at θ ≥ 0.4, but each template splits into ~3 generalizations (completeness 0.80 at severity 1, 0.58 at severity 2). Differently perturbed instances of one template genuinely do not map onto each other above threshold.
4. **Sleep helps modestly.** Re-offering outliers and merging at ≥ 0.6 removes 6–26% of generalizations and gains 1–2 points of completeness at unchanged purity. Merging at the assimilation threshold (0.4) was tried and **rejected**: small denoised schemas look alike, so purity fell to 0.87 and few-shot recall fell. Merge thresholds must be stricter than assimilation thresholds.
5. **Template identification is slightly worse with schemas** (−0.02). Fewer, more abstract items give a coarser match; the inference gain outweighs it.
6. **Cost:** 0.25–0.65 ms per assimilation; the sleep pass takes ≤ 1.2 s for 6,000 cases.

## Open

- Fragmentation: hierarchical generalization (schemas of schemas), or assimilating by *partial* mappings (the core subgraph) rather than whole-case normalized scores.
- Schema-level retrieval at 10⁵–10⁶ scale, to address the E3 finding that partial analogues degrade with corpus size.
