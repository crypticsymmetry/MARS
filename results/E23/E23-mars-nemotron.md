# E23: story analogies from natural language — MARS scores

360 of 360 StoryAnalogy multiple-choice questions fully converted (`tools/llm2mars.py`). Accuracy = target scored highest (chance 0.25; ties count as wrong); *target > noun* / *target > random* = pairwise wins over each distractor type.

| method | accuracy | target > noun | target > random |
|---|---|---|---|
| fingerprint analogy profile | 0.458 | 0.614 | 0.682 |
| fingerprint literal profile | 0.317 | 0.372 | 0.690 |
| FAC (structural) | 0.372 | 0.472 | 0.549 |
| fused 0.3·FAC + 0.7·FP-analogy | 0.453 | 0.603 | 0.686 |
| fused 0.5·FAC + 0.5·FP-analogy | 0.453 | 0.606 | 0.685 |
| FAC higher-order only | 0.275 | 0.344 | 0.407 |
| analogy − 0.25·surface | 0.508 | 0.731 | 0.675 |
| analogy − 0.5·surface | 0.514 | 0.772 | 0.664 |
| analogy − 1·surface | 0.511 | 0.822 | 0.653 |
| surface channel only (C0) | 0.119 | 0.200 | 0.501 |
