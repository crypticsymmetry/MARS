# E24: analogy retrieval at scale with an LLM verifier (MAC → LLM-as-FAC)

Tables: [E24.md](E24.md) (retrievers and verifier); [E24-mars-nemotron.md](E24-mars-nemotron.md) and [E24-mars-glm.md](E24-mars-glm.md) (MARS on each front end). Verifier answers: `verify-*.jsonl`.

Reproduce:
```
mars-bench e24 --data data/storyanalogy/nemotron-3-super --out OUT
python3 tools/e24_pipeline.py data/storyanalogy/nemotron-3-super/storyanalogy_multiple_choice.json OUT/E24-mars.json OUT \
    --llm z-ai/glm-5.3-flash --reasoning low --k 10 \
    --retrievers "lexical TF-IDF;sentence embedding (bge-small);MARS fused FAC + fingerprint analogy;RRF(MARS fused, lexical)"
```

**Question:** E23 showed that an LLM choosing among 4 given candidates beats MARS's structural scoring, while MARS resists surface look-alikes that fool text similarity. The architecture's claim is a division of labour: a cheap associative stage retrieves a few candidates from a *large* memory, and an expensive verifier (FAC, here an LLM) decides. Does MARS make a better first stage than text retrieval?

**Setup.**
- Memory: all 1,800 StoryAnalogy multiple-choice stories pooled. That is every source, analogy, same-topic distractor and random story of the 360 questions.
- Each source queries the other 1,799 stories; relevant = its analogy. Its own same-topic distractor is in the pool, as are every other question's stories.
- Retrievers: lexical TF-IDF, sentence embeddings (bge-small), MARS on the Nemotron-built cases (E23), and reciprocal-rank fusion of MARS and lexical.
- Verifier: GLM-5.3-Flash sees the query and a retriever's top-10 (shuffled) and picks the best analogy.

## Results

Retrieval (1,799 candidates per query):

| retriever | R@1 | R@5 | R@10 | MRR | analogy ranked above its same-topic distractor |
|---|---|---|---|---|---|
| lexical TF-IDF | 0.058 | 0.206 | **0.317** | 0.128 | 0.156 |
| sentence embedding | 0.047 | 0.100 | 0.161 | 0.083 | 0.108 |
| MARS fingerprint analogy | 0.075 | 0.131 | 0.169 | 0.108 | **0.283** |
| MARS fused FAC + fingerprint | 0.067 | 0.142 | 0.178 | 0.103 | **0.283** |
| **RRF(MARS fused, lexical)** | **0.108** | **0.228** | 0.311 | **0.180** | 0.272 |

End to end (LLM verifier over each retriever's top-10):

| retriever | analogy in top-10 | verifier picks the analogy | …given it is in the top-10 | picks the same-topic distractor |
|---|---|---|---|---|
| lexical TF-IDF | 0.317 | 0.211 | 0.67 | 0.097 |
| sentence embedding | 0.161 | 0.089 | 0.55 | 0.131 |
| MARS fused | 0.178 | 0.147 | **0.83** | **0.039** |
| **RRF(MARS fused, lexical)** | 0.311 | **0.222** | 0.71 | 0.081 |

MARS alone on the GLM-built cases: R@10 0.139, MRR 0.090 (Nemotron-built: 0.169 / 0.108).

## Findings

1. **At scale the task is hard for every retriever.** Among 1,799 stories the analogy is in the top 10 for at most a third of the queries. Each query's own same-topic distractor and many same-shape stories compete.
2. **MARS alone is not the best first stage here; lexical TF-IDF has twice its recall at 10.** StoryAnalogy's analogies were generated from their sources and often keep the same scaffolding words ("This pattern of … continues"), which lexical matching exploits. Embeddings, which weight topic, are worst.
3. **MARS's candidates are the cleanest.**
   - When the analogy is among MARS's top 10, the verifier picks it 83% of the time (lexical 67%, embeddings 55%).
   - The verifier falls for the same-topic distractor only 3.9% of the time (lexical 9.7%, embeddings 13.1%). MARS ranks the analogy above its look-alike twice as often as any text retriever (0.28 vs 0.11–0.16).
   - Structure-based retrieval fills the shortlist with genuinely analogous candidates rather than same-topic ones.
4. **Combining them gives the best pipeline.** Rank fusion of MARS and lexical has the best R@1, R@5 and MRR (0.180 vs 0.128), and the best end-to-end accuracy (0.222). The signals are complementary: lexical catches shared scaffolding, MARS catches shared structure and filters out look-alikes.
5. **Design consequence.** For natural-language memories, the MAC stage should fuse MARS's structural fingerprint with a cheap text signal, and hand a short list to an LLM verifier. This is the MAC/FAC architecture with the LLM in the FAC role. The absolute numbers stay low because LLM-built cases are coarse (E23 finding 3). Richer and more consistent front-end representations are the main lever, not the retrieval algorithm.
