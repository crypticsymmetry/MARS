# E12: cross-language program analogy (Python ↔ JavaScript)

Full tables with per-query ranks: [E12.md](E12.md). Reproduce with `tools/fetch_e9_corpus.sh && cargo run --release -p mars-bench -- e9 --task cross-lang`.

**Corpus:** the E9 Python corpus (1,113 functions from 3 packages) plus the npm `algorithms` 0.10.0 package (MIT; 195 JavaScript functions), 1,308 functions in total. JavaScript is parsed with acorn (`tools/js_ast.js`) and converted by `tools/js2mars.py` into the **same relational vocabulary** as Python. JS idioms are mapped onto shared relations so the comparison tests structure, not syntax:
- C-style counting loops → `range` iteration;
- `.length` → `len`; `push` → `append`;
- comparator methods (`greaterThan`) → comparison relations;
- temp-variable swaps → `swap`.

**Task:** query = the main function of an algorithm file; relevant = main functions of the *same algorithm in the other language* (normalized file stem, e.g. `bubble_sort` ↔ `bubble_sort`, `binary_search` ↔ `binarysearch`, `bfs` ↔ `breadth_first_search`). 52 queries, ranked among all 1,308 functions.

| method | R@1 | R@5 | R@10 | MRR |
|---|---|---|---|---|
| B2 lexical TF-IDF | 0.038 | 0.096 | 0.173 | 0.089 |
| B4 MAC content vectors | 0.058 | 0.135 | 0.288 | 0.114 |
| fingerprint analogy profile | 0.038 | 0.192 | 0.269 | 0.119 |
| FAC only | 0.038 | 0.250 | 0.365 | 0.142 |
| **fused ½FAC + ½FP (literal)** | **0.058** | **0.346** | **0.365** | **0.168** |
| fused 0.3FAC + 0.7FP (literal) | 0.000 | 0.308 | 0.385 | 0.141 |

## Findings

1. **Structure transfers across languages where surface does not.** Structural fusion roughly doubles lexical retrieval (MRR 0.168 vs 0.089; R@5 0.35 vs 0.10). Some cross-language pairs are found at rank 0–2 by structure only: bubble sort, selection sort, insertion sort, GCD, Bellman–Ford and longest common subsequence. For bubble sort, lexical ranks the counterpart 17–102 and FAC 1–5.
2. **Absolute accuracy is low.** R@1 is ≤ 0.06, and R@10 about 0.37. The failures are the same kind as in E9, but worse:
   - algorithms whose implementations use different strategies (heap sort, merge sort, radix sort, quicksort variants);
   - graph searches whose data structures differ (adjacency objects vs lists).
   Cross-language program analogy needs representations above the syntax level (see E9).
3. **The literal-profile advantage from E9 shrinks.** Identifier conventions differ across languages (`camelCase` vs `snake_case`, `a` vs `array`), so names carry less signal. The best fusion weight shifts back toward the structural score: ½ beats 0.3 here, whereas 0.3 won within one language. **The right fusion weight is domain-dependent, like the profile.**
4. **One-level helper inlining in JS had no effect:** the npm package rarely calls module-local helpers.
