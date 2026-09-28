# EV3 (pre-registered, secondary): the E33 incident agent with stronger baselines

Protocol and predictions: [docs/PREREGISTRATION.md](../../docs/PREREGISTRATION.md). Tables: [EV3.md](EV3.md); per-incident outcomes: `EV3.json`.

Reproduce: `python3 tools/e33_agent_memory.py results/EV3 --ev3 1,2,3,4,5` (the LLM part needs `$OPENROUTER_API_KEY`; `--no-llm` skips it).

**Scope:** E33's incident task is synthetic and was designed by us to be structural. EV3 therefore tests **baseline strength** (is MARS better than a standard embedding-RAG memory, and as a retriever for an LLM agent?), not external validity.

**Deviations:**
- The embedding-RAG memory is compared on the *raw* ranking only: its analogues come from outside the engine, so learned transfer reliability does not apply. MARS and name recall are reported both raw and learned.
- The LLM subset is 148 known-mechanism incidents (every 4th incident of seed 1, excluding first occurrences of novel mechanisms), not 150.

## Results (noise 2; 5 seeds × 600 incidents; known mechanisms)

| memory | fix@1 raw (mean ± sd over seeds) | fix@1 learned |
|---|---|---|
| **MARS** (structural fingerprints + mapping) | **0.728 ± 0.013** | **0.767 ± 0.014** |
| embedding RAG (bge-small top-5, same mapper) | 0.349 ± 0.021 | — |
| name recall (identity channel only) | 0.200 ± 0.008 | 0.308 ± 0.015 |

- Paired per seed, MARS − embedding RAG is +0.380 [+0.363, +0.396] (raw).
- Seed 1 per incident: +0.398 [+0.356, +0.442].

**LLM agent** (GLM-5.3-Flash, reasoning low, temperature 0; 148 incidents; 444 calls, $0.32, no failures):

| LLM given the top-5 episodes from | fix@1 [95% CI] |
|---|---|
| **MARS** | **0.676** [0.601, 0.750] |
| embedding RAG | 0.459 [0.378, 0.541] |
| name recall | 0.372 [0.297, 0.453] |
| *(MARS alone, no LLM, same incidents)* | *0.757 [0.682, 0.824]* |

- LLM + MARS vs LLM + embedding RAG: +0.216 [+0.142, +0.291].
- LLM + MARS vs LLM + name recall: +0.304 [+0.216, +0.392].

**Prediction P9** (MARS > embedding RAG, as memory and as an LLM's retriever): **confirmed** on both parts.

## Findings

1. **On a structural task, a standard embedding-RAG memory is a much weaker memory than MARS** (0.35 vs 0.73 raw). Embeddings of the incident text retrieve incidents with similar *words* (the same services and symptoms), not the same *mechanism*. The same holds when the retrieved episodes go to an LLM: MARS's episodes lift it from 0.46 to 0.68.
2. **E33's LLM result replicates, with per-incident statistics:** 0.676 with MARS episodes here, vs 0.66 in E33. MARS alone (0.757) is still better than this small LLM reading MARS's episodes.
3. **The contrast with EV2 is the point.** On real code retrieval, a pretrained embedding beat MARS by a wide margin. On a task whose regularities are relational and whose surface vocabulary is misleading, MARS beats embeddings by as much. The value of MARS is task-dependent in a predictable way: it wins when the analogy is in the relational structure *and* the representation exposes it. The open question for real domains is still representation (EV2).
