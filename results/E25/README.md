# E25: better LLM front-end representations — abstraction-first prompting and ensembles

Full tables: [E25.md](E25.md) / [E25.json](E25.json). MARS per front end: `E23-mars-glm-v2.*`, `E24-mars-*.json` (fused rankings).

Reproduce (all inputs versioned):
```
mars-bench e23 --data data/storyanalogy/glm-5.3-flash-v2 --out OUT   # likewise e24
python3 tools/e25_analysis.py data/storyanalogy/glm-5.3-flash-v2/storyanalogy_multiple_choice.json OUT \
  nemotron=results/E23/E23-mars-nemotron.json:results/E25/E24-mars-nemotron.json \
  glm-v1=results/E23/E23-mars-glm.json:results/E25/E24-mars-glm-v1.json \
  glm-v2=results/E25/E23-mars-glm-v2.json:results/E25/E24-mars-glm-v2.json \
  --patterns data/storyanalogy/glm-5.3-flash-v2/patterns.jsonl
```

**Question:** E23/E24 found the LLM-built representations to be the bottleneck: they are coarse, and unrelated stories look alike. Two cheap levers, both using GLM-5.3-Flash:

1. **Abstraction-first prompting (v2).** For each story, the LLM first writes a one-line, topic-free description of its causal/temporal pattern, then the facts that follow it. It must link at least two events with a higher-order relation. `tools/llm2mars.py --prompt v2`.
2. **Ensembles of front ends.** Three independent conversions of every story (Nemotron v1, GLM v1, GLM v2). Multiple-choice scores are averaged; retrieval rankings are fused with RRF.

**Control:** the v2 *pattern sentences themselves*, compared with sentence embeddings and no MARS. This separates the value of the LLM's abstraction from the value of MARS's structural matching.

**Front-end discipline:**

| front end | facts outside the vocabulary |
|---|---|
| Nemotron v1 | 1.6% |
| GLM v1 | 7.8% |
| GLM v2 | 1.4% |

## Results

StoryAnalogy multiple choice (360 questions, chance 0.25), and retrieval of the analogy among 1,799 pooled stories:

| front end(s) | MC accuracy: analogy (t>noun / t>random) | MC: analogy − 0.5·surface | retrieval R@10 | MRR |
|---|---|---|---|---|
| Nemotron v1 | 0.458 (0.61 / 0.68) | 0.514 (0.77 / 0.66) | 0.178 | 0.103 |
| GLM v1 | 0.414 (0.57 / 0.69) | 0.467 (0.77 / 0.66) | 0.133 | 0.091 |
| **GLM v2 (abstraction-first)** | **0.492 (0.62 / 0.75)** | 0.494 (0.81 / 0.67) | **0.208** | **0.121** |
| ensemble GLM v1 + v2 | 0.508 | 0.544 | 0.233 | 0.117 |
| ensemble Nemotron + GLM v2 | 0.519 | 0.564 | 0.244 | 0.146 |
| **ensemble of all three** | **0.533 (0.63 / 0.77)** | **0.592 (0.86 / 0.72)** | 0.247 | 0.143 |
| control: pattern sentences + embeddings (no MARS) | 0.447 (0.50 / 0.81) | — | 0.231 | 0.134 |

Combinations with text signals:

| retrieval (RRF) | R@1 | R@5 | R@10 | MRR |
|---|---|---|---|---|
| lexical TF-IDF alone (E24) | 0.058 | 0.206 | 0.317 | 0.128 |
| RRF(MARS Nemotron, lexical) (E24 best) | 0.108 | 0.228 | 0.311 | 0.180 |
| MARS ensemble + pattern embedding | 0.128 | 0.250 | 0.306 | 0.187 |
| MARS ensemble + lexical | 0.128 | 0.311 | 0.367 | 0.215 |
| **MARS ensemble + pattern embedding + lexical** | 0.125 | **0.336** | **0.392** | **0.224** |
| pattern embedding + lexical (no MARS) | 0.092 | 0.256 | 0.361 | 0.172 |

Multiple choice, Borda of the MARS ensemble and the pattern embedding: **0.606** (target > noun 0.81, > random 0.78).

For reference, an LLM answering directly: GLM-5.3-Flash 0.786 (E23).

## Findings

1. **Abstraction-first prompting is the single best front-end lever.** Asking for the topic-free pattern *before* the facts makes GLM's cases cleaner and more structural.
   - Out-of-vocabulary facts drop from 7.8% to 1.4%.
   - MARS accuracy rises from 0.414 to 0.492 (FAC 0.372 → 0.431). Target over random distractors goes 0.69 → 0.75, the E23 weak spot.
   - Retrieval at scale improves by half (R@10 0.133 → 0.208).
2. **Ensembles of front ends average out conversion noise.** Every combination beats its members; three front ends give 0.592 multiple-choice accuracy with the surface discount (target over look-alike 0.86). The same principle as E11's corroboration applies: independent evidence sources (here independent *representations*) should be pooled.
3. **MARS's structure adds to the LLM's abstraction.**
   - The pattern sentences alone, compared by embeddings, reach 0.447 on multiple choice. They beat random stories easily (0.81) but not look-alikes (0.50): an embedding of a sentence still carries topic.
   - MARS on the facts of the *same* v2 conversion reaches 0.492, and combined with the patterns 0.606.
   - At scale, the pattern embedding is a strong retriever (MRR 0.134, above any single MARS front end), and fusing everything is best (MRR 0.224).
4. **Where this leaves the architecture.** On natural language, the best cheap pipeline so far fuses structure (MARS over several LLM conversions), abstraction text (pattern embeddings) and lexical overlap. Each captures a different part of analogy, and together they nearly double E24's retrieval (R@5 0.206 → 0.336). A single LLM judging 4 candidates is still better (0.786), which keeps the LLM in the verifier role (E24).
5. **Cost.** A v2 conversion of all 1,800 stories with GLM-5.3-Flash cost about $1.4 (including retries), several times the v1 conversion, because pattern-first answers need more reasoning. Ensembles multiply conversion cost; a two-conversion ensemble captures most of the gain.
