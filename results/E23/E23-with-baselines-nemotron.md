# E23: StoryAnalogy multiple choice — MARS vs text baselines

360 questions (all five stories converted), chance accuracy 0.25.

| method | accuracy | target > noun | target > random |
|---|---|---|---|
| lexical TF-IDF | 0.183 | 0.186 | 0.865 |
| sentence embedding (BAAI/bge-small-en-v1.5) | 0.125 | 0.125 | 0.875 |
| LLM direct (glm-5.3-flash …) | 0.786 | 0.786 | 0.786 |
| MARS: FAC (structural) | 0.372 | 0.472 | 0.549 |
| MARS: FAC higher-order only | 0.275 | 0.344 | 0.407 |
| MARS: analogy − 0.25·surface | 0.508 | 0.731 | 0.675 |
| MARS: analogy − 0.5·surface | 0.514 | 0.772 | 0.664 |
| MARS: analogy − 1·surface | 0.511 | 0.822 | 0.653 |
| MARS: fingerprint analogy profile | 0.458 | 0.614 | 0.682 |
| MARS: fingerprint literal profile | 0.317 | 0.372 | 0.690 |
| MARS: fused 0.3·FAC + 0.7·FP-analogy | 0.453 | 0.603 | 0.686 |
| MARS: fused 0.5·FAC + 0.5·FP-analogy | 0.453 | 0.606 | 0.685 |
| MARS: surface channel only (C0) | 0.119 | 0.200 | 0.501 |
