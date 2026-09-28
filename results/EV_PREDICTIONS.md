# Pre-registered predictions: scored

Criteria fixed in `tools/ev_report.py` before any result was seen (see its docstring). Paired bootstrap 95% CIs.

| prediction | evidence | verdict |
|---|---|---|
| P6 MARS fused > lexical (MAP@R) | Δ +0.0823 [+0.0776, +0.0870] | confirmed |
| P7 code embedding ≥ MARS fused (MAP@R) | Δ +0.2621 [+0.2557, +0.2687] | confirmed |
| P8 RRF(MARS, lexical) ≥ both (MAP@R) | vs MARS -0.0467 [-0.0500, -0.0435]; vs lexical +0.0356 [+0.0332, +0.0379] | not confirmed |
| P9 MARS > embedding RAG, as memory and as LLM retriever | memory (raw fix@1, per-seed paired over 5 seeds): Δ +0.3797 [+0.3634, +0.3959]; LLM + MARS vs LLM + embedding: Δ +0.2162 [+0.1419, +0.2905] | confirmed (memory: confirmed; LLM: confirmed) |

