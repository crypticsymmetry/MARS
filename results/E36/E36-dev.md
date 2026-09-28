# E36: learned fusion and conformal selective retrieval (code)

Pre-registered in docs/PREREGISTRATION.md, addendum B. Logistic ranker (C = 1.0) fit on development pairs; features: cos, fac, cos_rank, fac_rank, q_top1, q_margin, q_maxfac, cos*q_top1, fac*q_top1.
Weights (standardized features, then intercept): -2.509, +1.954, -1.099, -0.814, -1.453, +0.014, -0.063, +3.816, -1.714, -2.223.
Conformal (Learn-then-Test, Bonferroni over the grid, exact binomial; α = 0.05, δ = 0.1, 200-point grid): threshold on learned confidence 0.9568; on embedding top-1 cosine inf.

### Development (fit and calibration data: in-sample): 5970 queries (candidates: code embedding top-100)

| method | MAP@R [95% CI] | Hits@1 | MRR |
|---|---|---|---|
| code embedding | 0.4964 [0.4887, 0.5040] | 0.8591 | 0.8944 |
| fixed ½cos + ½FAC (E34) | 0.5249 [0.5177, 0.5319] | 0.9047 | 0.9301 |
| learned fusion (A3) | 0.5293 [0.5222, 0.5365] | 0.8980 | 0.9239 |
- dev selective top-1, learned (A3) confidence: answered 1020/5970 (0.171), precision 0.9716
- dev selective top-1, embedding cosine confidence: answered 0/5970 (0.000), precision nan
