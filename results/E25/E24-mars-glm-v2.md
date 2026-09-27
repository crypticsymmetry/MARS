# E24: story-analogy retrieval over the pooled StoryAnalogy memory — MARS

Memory: 1800 stories (all sources, analogies and distractors of 360 questions). Each source retrieves among the other 1799; relevant = its analogy.

| method | R@1 | R@5 | R@10 | R@50 | MRR |
|---|---|---|---|---|---|
| MARS fingerprint analogy | 0.075 | 0.150 | 0.203 | 0.331 | 0.117 |
| MARS analogy − 0.5·surface | 0.072 | 0.131 | 0.175 | 0.278 | 0.108 |
| MARS fused FAC + fingerprint analogy | 0.083 | 0.153 | 0.208 | 0.333 | 0.124 |
| MARS fused FAC + (analogy − 0.5·surface) | 0.078 | 0.144 | 0.200 | 0.275 | 0.116 |
