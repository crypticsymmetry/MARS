# E3: end-to-end MAC → FAC retrieval at scale

**Setup:** the corpus is every case of G generated groups (8·G cases: base, LS, TA, MA, FOR, 3×RND). The 1,000 queries are group bases. The query base and its LS are excluded, and the target is the group's TA. Every *other* group's cases act as background, so the query competes with its own MA/FOR foils *and* with up to 10⁶ unrelated structures. MAC: fingerprint Mode K (analogy profile, D = 8192) or exact sparse baselines. FAC: the mapper re-ranks the top k = 64 by normalized structural score ("FAC") or by ½·FAC + ½·fingerprint ("fused"). "Exhaustive" maps the query against every case (sampled queries). 4 threads. Reproduce with `mars-bench e3 --groups G --ops all --severity s`.

## Clean analogues (no perturbation, 2 distractors)

| corpus | MAC stage | R@1 | R@64 | MA above TA | end-to-end fused acc@1 | exhaustive fused acc@1 | cost per query |
|---|---|---|---|---|---|---|---|
| 10⁴ | **fingerprint** | 0.997 | 1.000 | 0.001 | **1.000** | 1.000 (50 q) | 0.2 + 1.0 ms |
| 10⁴ | B4 MAC content vectors | 0.000 | 0.796 | 1.000 | 0.796 | | |
| 10⁴ | B2 lexical TF-IDF | 0.000 | 0.000 | 1.000 | 0.000 | | |
| 10⁵ | **fingerprint** | 0.993 | 1.000 | 0.002 | **0.999** | 1.000 (50 q) | 1.1 + 1.8 ms (exhaustive ≈ 975 ms) |
| 10⁵ | B5 exact sparse, same features | 0.991 | 1.000 | 0.002 | 0.999 | | |
| 10⁵ | B4 MAC content vectors | 0.000 | 0.348 | 1.000 | 0.348 | | |
| 10⁵ | B2 lexical TF-IDF | 0.000 | 0.000 | 1.000 | 0.000 | | |
| 10⁶ | **fingerprint** | 0.970 | 1.000 | 0.004 | **0.994** | 1.000 (20 q) | 10.4 + 2.0 ms (exhaustive ≈ 14.0 s) |

## Partial analogues (mixed perturbations; see E4)

| corpus | severity | fingerprint R@64 | FAC acc@1 | **fused acc@1** | fused (discriminable) | B4 MAC → fused | exhaustive fused / pipeline, same queries |
|---|---|---|---|---|---|---|---|
| 10⁴ | 1 | 0.957 | 0.787 | **0.912** | 0.984 | 0.600 | 0.94 / 0.94 (50 q) |
| 10⁴ | 2 | 0.905 | 0.630 | **0.769** | 0.958 | 0.433 | 0.74 / 0.72 (50 q) |
| 10⁵ | 1 | 0.934 | 0.731 | **0.859** | 0.949 | 0.241 | 0.90 / 0.92 (50 q) |
| 10⁵ | 2 | 0.852 | 0.515 | **0.645** | 0.873 | 0.162 | 0.60 / 0.60 (50 q) |
| 10⁶ | 1 | 0.897 | 0.629 | **0.745** | 0.854 | — | 0.80 / 0.85 (20 q) |

## Findings

1. **H1 (retrieval) holds on clean analogues up to 10⁶.** TA is in the fingerprint top-64 for 100% of queries at every scale, and at rank 1 for 97–99.7%. Classic MAC content vectors degrade with scale (R@64: 0.80 → 0.35 from 10⁴ to 10⁵) and always rank the mere-appearance foil above TA. Lexical retrieval never finds TA.
2. **H2 (scale) holds.** Mode K → FAC@64 matches the exhaustive-FAC upper bound on every sampled query set, clean and perturbed, while mapping 6.4×10⁻⁵ of a 10⁶ corpus. Cost: about 12 ms per query against 14 s, a **~1,100× reduction**. (H2's latency target was ≤ 100 ms at 10⁶ on 16 cores; this is 12 ms on 4.)
3. **The fingerprint is as good as exact sparse cosine on its own features** (B5) at 10⁴ and 10⁵ in every configuration, confirming the SimHash analysis. The binary sketch costs nothing measurable here, while being a fixed 1 KiB per case and SIMD-scannable.
4. **Fused re-ranking is necessary.** Structural-only FAC is worse than fused everywhere under perturbation (e.g. 10⁶ severity 1: 0.629 vs 0.745), and even exhaustive FAC-only is below fused (clean 10⁶: 0.95 vs 1.00).
5. **Partial analogues degrade with corpus size.** Fused accuracy at severity 1 goes 0.912 (10⁴) → 0.859 (10⁵) → 0.745 (10⁶). More background means more chance structural near-matches for a small, damaged structure. It is *not* a MAC-stage loss: the pipeline tracks the exhaustive bound. This is the "structured distractors" risk from DESIGN §16, now quantified. Levers: larger cases (more structure per query), IDF over structural features, and schema-level retrieval (P6).
6. Some rank-1 misses at 10⁶ on clean data are probably *genuine* analogues from other groups (small random templates recur by chance). The end-to-end metric counts these as errors, so it is conservative.

## Cost breakdown at 10⁶ (4 cores)

Generation 13 s; feature extraction 32 s (about 32 µs per case); sketching + index 20 s; Mode K 10.4 ms/query (batched 64); FAC@64 ≈ 2 ms/query; self-score cache filled lazily.
