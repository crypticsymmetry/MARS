# E23: story analogies from natural language — MARS scores

360 of 360 StoryAnalogy multiple-choice questions fully converted (`tools/llm2mars.py`). Accuracy = target scored highest (chance 0.25; ties count as wrong); *target > noun* / *target > random* = pairwise wins over each distractor type.

| method | accuracy | target > noun | target > random |
|---|---|---|---|
| fingerprint analogy profile | 0.414 | 0.567 | 0.692 |
| fingerprint literal profile | 0.292 | 0.333 | 0.710 |
| FAC (structural) | 0.372 | 0.458 | 0.574 |
| fused 0.3·FAC + 0.7·FP-analogy | 0.428 | 0.525 | 0.693 |
| fused 0.5·FAC + 0.5·FP-analogy | 0.417 | 0.519 | 0.693 |
| FAC higher-order only | 0.306 | 0.397 | 0.450 |
| analogy − 0.25·surface | 0.464 | 0.708 | 0.669 |
| analogy − 0.5·surface | 0.467 | 0.767 | 0.657 |
| analogy − 1·surface | 0.467 | 0.817 | 0.633 |
| surface channel only (C0) | 0.108 | 0.197 | 0.526 |
