# EV1 test: FB15k-237

Frozen v0.1 components. Settings (chosen on validation): profile literal, identity λ 0.85, k 30; cap 100, shortlist 50, FAC weight 0.5. Reliability learned from 87873 feedback events on all 35070 validation queries, then frozen. 40932 test queries (20466 triples × 2 directions); filtered ranks over 14541 entities; unproposed entities at their expected random rank. Runtime 7740.0s.

| ranking | subset | n | MRR | Hits@1 | Hits@3 | Hits@10 |
|---|---|---|---|---|---|---|
| MARS learned reliability | all | 40932 | 0.1629 | 0.1286 | 0.1770 | 0.2314 |
| MARS learned reliability | answer within 2 hops | 30228 | 0.1730 | 0.1383 | 0.1867 | 0.2417 |
| MARS learned reliability | answer farther | 10704 | 0.1343 | 0.1009 | 0.1497 | 0.2024 |
| MARS raw | all | 40932 | 0.1593 | 0.1240 | 0.1736 | 0.2302 |
| MARS raw | answer within 2 hops | 30228 | 0.1688 | 0.1330 | 0.1828 | 0.2403 |
| MARS raw | answer farther | 10704 | 0.1324 | 0.0987 | 0.1475 | 0.2017 |
| relation popularity | all | 40932 | 0.2334 | 0.1700 | 0.2500 | 0.3541 |
| relation popularity | answer within 2 hops | 30228 | 0.2421 | 0.1795 | 0.2576 | 0.3628 |
| relation popularity | answer farther | 10704 | 0.2090 | 0.1430 | 0.2287 | 0.3296 |
