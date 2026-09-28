# EV1 validation grid: WN18RR

4000 validation queries (2000 triples × 2 directions, seed 1); raw ranking (Σ fused analogue score); cap 100, shortlist 50, FAC weight 0.5.

| profile | identity λ | k | MRR | Hits@1 | Hits@10 |
|---|---|---|---|---|---|
| analogy | 0 | 10 | 0.1014 | 0.0968 | 0.1070 |
| analogy | 0 | 30 | 0.1572 | 0.1522 | 0.1640 |
| analogy | 0.5 | 10 | 0.1495 | 0.1278 | 0.1670 |
| analogy | 0.5 | 30 | 0.2167 | 0.1968 | 0.2367 |
| analogy | 0.85 | 10 | 0.2638 | 0.2275 | 0.3075 |
| analogy | 0.85 | 30 | 0.3270 | 0.2963 | 0.3690 |
| literal | 0 | 10 | 0.1277 | 0.1230 | 0.1323 |
| literal | 0 | 30 | 0.1922 | 0.1852 | 0.2005 |
| literal | 0.5 | 10 | 0.2035 | 0.1650 | 0.2367 |
| literal | 0.5 | 30 | 0.2691 | 0.2377 | 0.2988 |
| literal | 0.85 | 10 | 0.2783 | 0.2440 | 0.3197 |
| literal | 0.85 | 30 | 0.3460 | 0.3207 | 0.3830 |

Chosen (max MRR): {"hits1":0.32075,"hits10":0.383,"identity":0.85,"k":30,"mrr":0.34600622519229013,"profile":"literal"}
