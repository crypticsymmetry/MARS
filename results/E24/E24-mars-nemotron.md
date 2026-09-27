# E24: story-analogy retrieval over the pooled StoryAnalogy memory — MARS

Memory: 1800 stories (all sources, analogies and distractors of 360 questions). Each source retrieves among the other 1799; relevant = its analogy.

| method | R@1 | R@5 | R@10 | R@50 | MRR |
|---|---|---|---|---|---|
| MARS fingerprint analogy | 0.075 | 0.131 | 0.169 | 0.303 | 0.111 |
| MARS analogy − 0.5·surface | 0.069 | 0.128 | 0.167 | 0.283 | 0.103 |
| MARS fused FAC + fingerprint analogy | 0.067 | 0.142 | 0.178 | 0.308 | 0.106 |
| MARS fused FAC + (analogy − 0.5·surface) | 0.067 | 0.136 | 0.175 | 0.289 | 0.104 |
