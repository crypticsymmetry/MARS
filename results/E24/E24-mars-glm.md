# E24: story-analogy retrieval over the pooled StoryAnalogy memory — MARS

Memory: 1800 stories (all sources, analogies and distractors of 360 questions). Each source retrieves among the other 1799; relevant = its analogy.

| method | R@1 | R@5 | R@10 | R@50 | MRR |
|---|---|---|---|---|---|
| MARS fingerprint analogy | 0.058 | 0.117 | 0.139 | 0.261 | 0.090 |
| MARS analogy − 0.5·surface | 0.061 | 0.111 | 0.136 | 0.256 | 0.089 |
| MARS fused FAC + fingerprint analogy | 0.067 | 0.114 | 0.133 | 0.267 | 0.094 |
| MARS fused FAC + (analogy − 0.5·surface) | 0.069 | 0.108 | 0.131 | 0.247 | 0.094 |
