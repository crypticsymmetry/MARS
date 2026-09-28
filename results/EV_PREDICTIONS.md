# Pre-registered predictions: scored

Criteria fixed in `tools/ev_report.py` before any result was seen (see its docstring). Paired bootstrap 95% CIs.

| prediction | evidence | verdict |
|---|---|---|
| P1 MARS below RotatE and TuckER (MRR) | FB15k-237: MARS MRR 0.163 [0.160, 0.166] vs RotatE 0.338, TuckER 0.358; WN18RR: MARS MRR 0.372 [0.360, 0.383] vs RotatE 0.476, TuckER 0.47 | confirmed |
| P2 MARS > relation popularity (FB15k-237, MRR) | Δ -0.0705 [-0.0740, -0.0672] | not confirmed |
| P3 Hits@1 within 2 hops ≫ farther (FB15k-237) | 0.138 (n 30228) vs 0.101 (n 10704); Δ CI [+0.031, +0.044] | inconclusive |
| P4 learned ≥ raw (FB15k-237, MRR) | Δ +0.0036 [+0.0032, +0.0040] | confirmed |
| P2 MARS > relation popularity (WN18RR, MRR) | Δ +0.3462 [+0.3342, +0.3583] | confirmed |
| P3 Hits@1 within 2 hops ≫ farther (WN18RR) | 0.749 (n 2774) vs 0.051 (n 3494); Δ CI [+0.681, +0.716] | confirmed |
| P4 learned ≥ raw (WN18RR, MRR) | Δ +0.0211 [+0.0185, +0.0239] | confirmed |
| P5 relatively closer to KGE on WN18RR | MARS/RotatE MRR ratio: WN18RR 0.78, FB15k-237 0.48 | confirmed |
| P6 MARS fused > lexical (MAP@R) | Δ +0.0823 [+0.0776, +0.0870] | confirmed |
| P7 code embedding ≥ MARS fused (MAP@R) | Δ +0.2621 [+0.2557, +0.2687] | confirmed |
| P8 RRF(MARS, lexical) ≥ both (MAP@R) | vs MARS -0.0467 [-0.0500, -0.0435]; vs lexical +0.0356 [+0.0332, +0.0379] | not confirmed |
| P9 MARS > embedding RAG, as memory and as LLM retriever | memory (raw fix@1, per-seed paired over 5 seeds): Δ +0.3797 [+0.3634, +0.3959]; LLM + MARS vs LLM + embedding: Δ +0.2162 [+0.1419, +0.2905] | confirmed (memory: confirmed; LLM: confirmed) |

