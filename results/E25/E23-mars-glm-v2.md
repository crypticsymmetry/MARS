# E23: story analogies from natural language — MARS scores

360 of 360 StoryAnalogy multiple-choice questions fully converted (`tools/llm2mars.py`). Accuracy = target scored highest (chance 0.25; ties count as wrong); *target > noun* / *target > random* = pairwise wins over each distractor type.

| method | accuracy | target > noun | target > random |
|---|---|---|---|
| fingerprint analogy profile | 0.492 | 0.622 | 0.749 |
| fingerprint literal profile | 0.353 | 0.397 | 0.756 |
| FAC (structural) | 0.431 | 0.544 | 0.647 |
| fused 0.3·FAC + 0.7·FP-analogy | 0.481 | 0.614 | 0.736 |
| fused 0.5·FAC + 0.5·FP-analogy | 0.467 | 0.597 | 0.736 |
| FAC higher-order only | 0.444 | 0.533 | 0.614 |
| analogy − 0.25·surface | 0.503 | 0.750 | 0.710 |
| analogy − 0.5·surface | 0.494 | 0.811 | 0.672 |
| analogy − 1·surface | 0.467 | 0.839 | 0.614 |
| surface channel only (C0) | 0.131 | 0.186 | 0.576 |
