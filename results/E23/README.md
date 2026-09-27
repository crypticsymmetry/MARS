# E23: story analogies from natural language (StoryAnalogy multiple choice)

Tables:
- [E23-with-baselines-nemotron.md](E23-with-baselines-nemotron.md): everything, on the Nemotron front end;
- [E23-mars-nemotron.md](E23-mars-nemotron.md) and [E23-mars-glm.md](E23-mars-glm.md): MARS on each front end.

LLM answers: [llm-direct-glm-5.3-flash.jsonl](llm-direct-glm-5.3-flash.jsonl) (all 360) and [llm-direct-mimo-v2.6-pro-partial.jsonl](llm-direct-mimo-v2.6-pro-partial.jsonl) (70).

Reproduce (converted cases are versioned in `data/storyanalogy/`):
```
mars-bench e23 --data data/storyanalogy/nemotron-3-super     # or glm-5.3-flash
python3 tools/e23_baselines.py data/storyanalogy/nemotron-3-super/storyanalogy_multiple_choice.json \
    results/E23/E23-mars-nemotron.json OUT [--llm z-ai/glm-5.3-flash --reasoning low]
```

**Question:** MARS so far ran on synthetic cases and on code converted by parsers. Does it work when an **LLM front end** builds the relational cases from natural language? And does it prefer *analogies* over surface-similar stories, as the design claims?

**Data.** StoryAnalogy (Jiayang et al., EMNLP 2023) multiple-choice set: 360 questions. Each has a source story and 4 choices:
- the **analogy** (target);
- a **noun** distractor on the same topic as the source (often a continuation of the same scenario, i.e. a *literal-similarity* match);
- two **random** stories.

**Front end.** `tools/llm2mars.py` asks an LLM to rewrite each story as relational facts:
- a controlled vocabulary of Schank's conceptual-dependency primitives (`ptrans`, `atrans`, `mtrans`, `propel`, `ingest`, …) plus abstract states and intentions;
- canonical higher-order relations (`cause`, `enable`, `prevent`, `then`, `despite`, `repeat`, `and`).

Two independent front ends converted all 1,800 stories:

| front end | facts outside the vocabulary |
|---|---|
| NVIDIA Nemotron-3 Super 120B (free tier) | 1.6% |
| Z.ai GLM-5.3-Flash (low reasoning) | 7.8% |

A third candidate, Poolside Laguna, was dropped after producing 44% out-of-vocabulary facts: front-end discipline matters. The front end that followed the vocabulary better (Nemotron) also gives MARS the better accuracy.

## Results (accuracy; chance 0.25)

| method | Nemotron front end | GLM front end | target > noun distractor | target > random |
|---|---|---|---|---|
| lexical TF-IDF | 0.183 | — | 0.186 | 0.865 |
| sentence embedding (bge-small) | 0.125 | — | 0.125 | 0.875 |
| MARS surface channel only (C0) | 0.119 | 0.108 | 0.200 | 0.501 |
| MARS FAC (structural mapper) | 0.372 | 0.372 | 0.472 | 0.549 |
| MARS fingerprint, analogy profile | 0.458 | 0.414 | 0.614 | 0.682 |
| MARS fused 0.3·FAC + 0.7·FP | 0.453 | 0.428 | 0.603 | 0.686 |
| **MARS analogy − 0.5·surface** | **0.514** | **0.467** | **0.772** | 0.664 |
| LLM answers directly: GLM-5.3-Flash (all 360) | 0.786 | | | |
| LLM answers directly: MiMo-V2.6-Pro (70 answered) | 0.843 | | | |

Pairwise columns are for the Nemotron front end. λ ∈ {0.25, 0.5, 1} for the surface discount gives 0.508 / 0.514 / 0.511: the effect is not fragile tuning.

## Findings

1. **Surface-similarity methods fall into the surface trap.** Lexical and embedding similarity pick the same-topic distractor almost every time (the target beats it only 12–19% of the time) and score *below chance*. This is the failure MARS is designed to avoid.
2. **MARS on LLM-built cases prefers analogies over look-alikes.** The analogy-profile fingerprint reaches 0.458 accuracy, 2.5–3.7× the surface methods.
   - Discounting MARS's own surface channel follows Gentner's definition (an analogy is a relational match *without* entity/attribute overlap). It raises accuracy to 0.514, and the target beats the same-topic distractor 77% of the time.
   - The pattern replicates on an independent front end (GLM: 0.414 → 0.467).
3. **The weak spot is random distractors, not look-alikes.** MARS beats random stories only ~66–69% of the time, against ~87% for text methods. LLM-built cases are coarse: a few abstract facts, many of them shared by unrelated stories (`increase`, `cause`, `has-property`). The structural mapper (FAC) is weaker than the fingerprint here (0.372), and matching on higher-order relations alone is weaker still (0.275). Small, abstract cases carry little identifying structure, the per-case information limit of E10/E16 again.
4. **A capable LLM reasoning over 4 given candidates is far better (0.79–0.84).** MARS is not a replacement for LLM judgment on a single small question. Its role in the architecture is the one it was designed for: the cheap, surface-resistant MAC stage over a large memory, with an expensive verifier (FAC, or an LLM) on the top few. E24 tests that directly.
