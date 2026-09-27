# E14: bootstrapped vocabulary alignment on real code

The E12 corpus (1308 functions, Python + JavaScript) converted in *raw* mode: library calls keep language-specific names (`py:len`, `js:length`, `js:push`, `js:Math-max`, …) as unresolved predicates — 359 of them (260 Python, 99 JavaScript), 18 py↔js pairs mean the same thing according to the converters' hand mapping. Each round retrieves 10 other-language neighbours per function (fingerprint analogy profile), maps each pair with wildcard matching (local score 0.3), harvests predicate correspondences from mappings with normalized score ≥ 0.2, and aligns mutual-best pairs (association `cosine`, evidence ≥ 1), one predicate per language per cluster, re-estimated from scratch each round. The hand mapping is used only to score.

Retrieval: E12 task A (52 queries: main functions whose algorithm exists in the other language, ranked among all 1308 functions). Cells: MRR (R@1/R@10).

| round | aligned pairs: correct / same name / wrong / unjudged | fingerprint analogy | fingerprint literal | FAC only | fused ½FAC+½FP-literal | fused 0.3FAC+0.7FP-literal |
|---|---|---|---|---|---|---|
| 0 | 0 / 0 / 0 / 0 | 0.117 (0.06/0.27) | 0.085 (0.00/0.29) | 0.139 (0.04/0.31) | 0.152 (0.06/0.38) | 0.117 (0.00/0.35) |
| 1 | 3 / 1 / 1 / 6 | 0.094 (0.02/0.23) | 0.076 (0.00/0.29) | 0.145 (0.04/0.37) | 0.153 (0.04/0.38) | 0.111 (0.00/0.35) |
| 2 | 3 / 1 / 1 / 9 | 0.093 (0.02/0.23) | 0.077 (0.00/0.29) | 0.145 (0.04/0.37) | 0.153 (0.04/0.38) | 0.112 (0.00/0.35) |
| 3 | 3 / 1 / 1 / 9 | 0.093 (0.02/0.23) | 0.077 (0.00/0.29) | 0.145 (0.04/0.37) | 0.153 (0.04/0.38) | 0.112 (0.00/0.35) |
| oracle alignment (hand map on raw corpus) | 18 gold pairs | 0.103 (0.04/0.25) | 0.078 (0.00/0.25) | 0.145 (0.04/0.37) | 0.155 (0.04/0.38) | 0.111 (0.00/0.35) |
| hand mapping (E12 corpus) | — | 0.119 (0.04/0.27) | 0.105 (0.00/0.29) | 0.142 (0.04/0.37) | 0.168 (0.06/0.37) | 0.141 (0.00/0.38) |

## Final alignment (round 3), by evidence

| Python | JavaScript | evidence | verdict |
|---|---|---|---|
| `py:len` | `js:length` | 38.89 | correct |
| `py:append` | `js:push` | 36.12 | correct |
| `py:is_empty` | `js:isEmpty` | 16.34 | same identifier |
| `py:insert` | `js:forEach` | 14.63 | unjudged |
| `py:pow` | `js:getNode` | 5.50 | unjudged |
| `py:put` | `js:addEdge` | 4.00 | unjudged |
| `py:helper` | `js:inOrder` | 3.17 | unjudged |
| `py:abs` | `js:Math-abs` | 2.11 | correct |
| `py:recur_search` | `js:_find` | 2.00 | unjudged |
| `py:find_set` | `js:root` | 1.88 | unjudged |
| `py:popleft` | `js:pop` | 1.86 | **wrong** |
| `py:fix_insert` | `js:getNodeHeight` | 1.64 | unjudged |
| `py:set` | `js:Math-random` | 1.28 | unjudged |
| `py:is_full` | `js:_siftUp` | 1.24 | unjudged |

Gold-pair recall: 3 / 18. Runtime 5.3s.
