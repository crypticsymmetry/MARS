# EV2: code retrieval by algorithm (CodeNet Python800)

5982 programs from 200 problems (seed 1; ≤ 30 per problem; 4496 unparseable or < 5 facts skipped). Every program is a query; relevant = other solutions of the same problem (R = group size − 1). Rankings to depth 100.

| method | MAP@R [95% CI] | Hits@1 [95% CI] | MRR [95% CI] |
|---|---|---|---|
| MARS fused (½FAC + ½FP-literal, FP top-100) | 0.2110 [0.2060, 0.2159] | 0.6993 [0.6876, 0.7108] | 0.7608 [0.7510, 0.7703] |
| MARS FP-literal | 0.1663 [0.1617, 0.1708] | 0.6092 [0.5968, 0.6214] | 0.6887 [0.6784, 0.6988] |
| lexical TF-IDF (Python tokens) | 0.1287 [0.1247, 0.1327] | 0.4940 [0.4816, 0.5069] | 0.5935 [0.5831, 0.6042] |
| code embedding (jinaai/jina-embeddings-v2-base-code) | 0.4731 [0.4659, 0.4804] | 0.8561 [0.8470, 0.8648] | 0.8915 [0.8845, 0.8983] |
| RRF(MARS fused, lexical) | 0.1642 [0.1599, 0.1685] | 0.6048 [0.5924, 0.6172] | 0.6936 [0.6836, 0.7035] |

**Pre-registered paired comparisons** (MAP@R, paired over queries):

| comparison | A − B [95% CI] | p |
|---|---|---|
| B2: MARS fused (½FAC + ½FP-literal, FP top-100) − lexical TF-IDF (Python tokens) | +0.0823 [+0.0776, +0.0870] | 0.0001 |
| B3: MARS fused (½FAC + ½FP-literal, FP top-100) − code embedding (jinaai/jina-embeddings-v2-base-code) | -0.2621 [-0.2687, -0.2557] | 0.0001 |
| P8a: RRF(MARS fused, lexical) − MARS fused (½FAC + ½FP-literal, FP top-100) | -0.0467 [-0.0500, -0.0435] | 0.0001 |
| P8b: RRF(MARS fused, lexical) − lexical TF-IDF (Python tokens) | +0.0356 [+0.0332, +0.0379] | 0.0001 |
