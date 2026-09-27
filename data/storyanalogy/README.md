# StoryAnalogy multiple choice, converted to MARS cases

Source: StoryAnalogy (Jiayang et al., EMNLP 2023; MIT license), multiple-choice subset
`src/data/storyanalogy_multiple_choice.json` from github.com/loginaway/StoryAnalogy: 360
questions, each a source story with 4 choices (the analogy, a same-topic "noun" distractor,
two random stories).

Each directory holds the 1,800 stories converted by one LLM front end with
`tools/llm2mars.py` (controlled vocabulary: conceptual-dependency primitives + canonical
higher-order relations):

- `nemotron-3-super/`: nvidia/nemotron-3-super-120b-a12b (free tier; 1.8% of facts outside the vocabulary)
- `glm-5.3-flash/`: z-ai/glm-5.3-flash (reasoning effort low)

LLM output is not reproducible, so the converted cases are versioned here. Evaluate with
`mars-bench e23 --data data/storyanalogy/<dir>` (multiple choice) and
`mars-bench e24 --data data/storyanalogy/<dir>` (retrieval over the pooled memory).
