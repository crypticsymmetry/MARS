# EV3: E33 incident stream with stronger baselines (noise 2)

Seeds [1, 2, 3, 4, 5]; 600 incidents each, 40 mechanisms (10 novel), known mechanisms only. Raw = ranking by Σ analogue weight (all memories); learned = with transfer reliability from feedback (engine memories only).

| memory | fix@1 raw, mean ± sd over seeds | fix@1 learned |
|---|---|---|
| MARS (structure) | 0.728 ± 0.013 | 0.767 ± 0.014 |
| recall by names (identity only) | 0.200 ± 0.008 | 0.308 ± 0.015 |
| embedding RAG (bge-small) | 0.349 ± 0.021 | — |

Paired (seed 1, raw, n = 590): MARS − embedding RAG = +0.398 [+0.356, +0.442], p = 0.0001.

**LLM agent** (GLM-5.3-Flash; 148 known-mechanism incidents of seed 1; $0.3199; 0 failed calls):

| LLM given top-5 episodes from | fix@1 [95% CI] |
|---|---|
| MARS (structure) | 0.676 [0.601, 0.750] |
| recall by names (identity only) | 0.372 [0.297, 0.453] |
| embedding RAG (bge-small) | 0.459 [0.378, 0.541] |
| (MARS alone, raw, same incidents) | 0.757 [0.682, 0.824] |

Paired: LLM+MARS (structure) − LLM+embedding RAG (bge-small) = +0.216 [+0.142, +0.291], p = 0.0001.

Paired: LLM+MARS (structure) − LLM+recall by names (identity only) = +0.304 [+0.216, +0.392], p = 0.0001.
