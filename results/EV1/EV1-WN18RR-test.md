# EV1 test: WN18RR

Frozen v0.1 components. Settings (chosen on validation): profile literal, identity λ 0.85, k 30; cap 100, shortlist 50, FAC weight 0.5. Reliability learned from 13504 feedback events on all 6068 validation queries, then frozen. 6268 test queries (3134 triples × 2 directions); filtered ranks over 40943 entities; unproposed entities at their expected random rank. Runtime 362.9s.

| ranking | subset | n | MRR | Hits@1 | Hits@3 | Hits@10 |
|---|---|---|---|---|---|---|
| MARS learned reliability | all | 6268 | 0.3718 | 0.3596 | 0.3802 | 0.3957 |
| MARS learned reliability | answer within 2 hops | 2774 | 0.7575 | 0.7487 | 0.7642 | 0.7714 |
| MARS learned reliability | answer farther | 3494 | 0.0655 | 0.0507 | 0.0753 | 0.0973 |
| MARS raw | all | 6268 | 0.3506 | 0.3216 | 0.3724 | 0.3953 |
| MARS raw | answer within 2 hops | 2774 | 0.7098 | 0.6629 | 0.7466 | 0.7707 |
| MARS raw | answer farther | 3494 | 0.0655 | 0.0507 | 0.0753 | 0.0973 |
| relation popularity | all | 6268 | 0.0256 | 0.0155 | 0.0250 | 0.0440 |
| relation popularity | answer within 2 hops | 2774 | 0.0095 | 0.0036 | 0.0105 | 0.0180 |
| relation popularity | answer farther | 3494 | 0.0384 | 0.0249 | 0.0366 | 0.0647 |
