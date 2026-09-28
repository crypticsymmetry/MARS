# EV1 validation grid: FB15k-237

4000 validation queries (2000 triples × 2 directions, seed 1); raw ranking (Σ fused analogue score); cap 100, shortlist 50, FAC weight 0.5.

| profile | identity λ | k | MRR | Hits@1 | Hits@10 |
|---|---|---|---|---|---|
| analogy | 0 | 10 | 0.1288 | 0.1050 | 0.1725 |
| analogy | 0 | 30 | 0.1491 | 0.1170 | 0.2135 |
| analogy | 0.5 | 10 | 0.1352 | 0.1085 | 0.1827 |
| analogy | 0.5 | 30 | 0.1567 | 0.1227 | 0.2238 |
| analogy | 0.85 | 10 | 0.1349 | 0.1075 | 0.1852 |
| analogy | 0.85 | 30 | 0.1599 | 0.1247 | 0.2288 |
| literal | 0 | 10 | 0.1371 | 0.1115 | 0.1878 |
| literal | 0 | 30 | 0.1584 | 0.1232 | 0.2293 |
| literal | 0.5 | 10 | 0.1419 | 0.1148 | 0.1940 |
| literal | 0.5 | 30 | 0.1616 | 0.1260 | 0.2352 |
| literal | 0.85 | 10 | 0.1385 | 0.1108 | 0.1852 |
| literal | 0.85 | 30 | 0.1625 | 0.1255 | 0.2338 |

Chosen (max MRR): {"hits1":0.1255,"hits10":0.23375,"identity":0.85,"k":30,"mrr":0.16253070466948408,"profile":"literal"}
