# E9: program analogy on real code (first real-data result)

Full tables, including per-query ranks: [E9.md](E9.md). Reproduce with `tools/fetch_e9_corpus.sh && cargo run --release -p mars-bench -- e9` (about 3 s).

**Corpus:** every function of three *independently written* MIT-licensed Python algorithm packages from PyPI: `algorithms` 1.0.1 (keon), `pygorithm` 1.0.4 and `python-algorithms` 0.2.2. That is 1,113 functions, 381 of them a file's main function. They are converted to relational cases fully automatically by `tools/py2mars.py`, with no per-program tuning:
- variables and constants → entities;
- operations → relational terms;
- statements → first-order facts;
- `for` / `while` / `if` → higher-order `in-loop` / `while-loop` / `guards` relations over statement facts;
- same-module helpers → inlined one level as `(inlined <call> fact)`.

This is the first test in which **the representations were not designed together with the data**: the "representation problem" risk from the design, measured honestly.

- **Task A, same algorithm by a different author** (33 queries, e.g. `algorithms/sorting/bubble_sort` → `pygorithm/sorting/bubble_sort`), ranked among all 1,113 functions.
- **Task B, same-category retrieval** among the 381 main functions (365 queries in categories with ≥ 3 mains).

## Results

| method | A: R@1 | A: R@5 | A: R@10 | A: MRR | B: P@1 | B: P@5 | B: MRR |
|---|---|---|---|---|---|---|---|
| B2 lexical TF-IDF (identifiers + operations) | 0.061 | 0.273 | 0.303 | 0.147 | 0.578 | 0.495 | 0.688 |
| B4 MAC content vectors | 0.152 | 0.212 | 0.273 | 0.188 | 0.493 | 0.362 | 0.601 |
| exact analogy profile | 0.212 | 0.273 | 0.333 | 0.256 | 0.468 | 0.380 | 0.596 |
| fingerprint analogy profile | 0.182 | 0.273 | 0.303 | 0.234 | 0.460 | 0.361 | 0.581 |
| fingerprint literal profile | 0.212 | 0.424 | 0.485 | 0.300 | 0.600 | 0.488 | 0.699 |
| FAC only (exhaustive) | 0.212 | 0.364 | 0.424 | 0.283 | 0.479 | 0.355 | 0.597 |
| fused ½FAC + ½FP (analogy) | 0.242 | 0.394 | 0.455 | 0.307 | 0.556 | 0.409 | 0.661 |
| **fused ½FAC + ½FP (literal)** | **0.242** | **0.424** | **0.545** | **0.334** | **0.636** | **0.500** | **0.724** |

(Task A has only 33 queries: one query = 0.03. Differences below about 0.06 are within noise.)

## Findings

1. **Structure helps on real code.** For finding the same algorithm by a different author, structural methods beat lexical retrieval by a wide margin: MRR 0.33 vs 0.15, R@10 0.55 vs 0.30. MAC content vectors (0.19) are also beaten. The mapper rescues cases where surface fails completely: exponential search goes from rank 42 → 0, bubble sort from 22 → 7–9, counting sort from 14 → 0.
2. **In code, identifier names are signal, not noise.** Unlike the synthetic analogy task, the *literal* profile (which includes variable and attribute names in C0) beats the pure-analogy profile, and fused FAC + literal fingerprint is best on both tasks. Different authors share conventions (`arr`, `lo`/`hi`, `pivot`), and category retrieval (Task B) is strongly surface-correlated. **The right profile is domain-dependent.** That is a design requirement (per-domain profile tuning), not a failure.
3. **Syntax-level relations miss algorithm-level identity.** Two quicksorts (in-place Lomuto partition vs list-comprehension partitions) or two merge sorts (index-based vs slice-based) are the same *algorithm* but different *programs*. No representation-free method finds them (ranks 30–800). One-level helper inlining fixed part of this: merge sort went from rank ~600 to ~120, but pygorithm's quicksort got worse. Real program analogy needs **semantic normalization** of the representation (e.g. dataflow canonicalization, idiom recognition such as "partition", "swap", "accumulate"). This is the representation problem (Chalmers, French & Hofstadter 1992), observed directly.
4. **Some ground truth is wrong.** `anagram` in the two packages is two different algorithms (sort-and-compare vs counting) that share a name. Every method ranks it low, correctly.

## Implications and next steps

- Add **profile tuning per domain** (a small dev split). For code, surface identifiers are legitimately informative.
- Try **representation normalization** for code: canonical loop forms (`while` + counter ↔ `for range`), comprehension ↔ loop-append, swap idioms, one more level of inlining, and a dataflow (def-use) view instead of syntax trees.
- Get more real data: larger code corpora (other PyPI/npm algorithm collections), and narrative analogies (ARN) if network access to datasets becomes available.
