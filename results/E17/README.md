# E17: candidate inference on real code

Full table: [E17.md](E17.md). Reproduce with `mars-bench e17 [--seed 1] [--per-func 2]` (~1.5 s; needs `tools/fetch_e9_corpus.sh`).

**Question:** candidate inference (projecting unmatched base structure into the target) was validated only on synthetic data (E7, E11, E15). Does it work on real programs? Can an analogue restore a deleted statement?

**Setup.**
- Corpus: the 1,113 Python functions of E9 (three packages).
- There are 700 queries. Each is a main function with one top-level *control* statement (relational order ≥ 2: `in-loop`, `guards`, `while-loop`, `inlined`, …) deleted. Every entity of the deleted statement still occurs elsewhere in the function, so it is restorable in principle. At most 2 per function, seed 1.
- Analogues are all other functions, ranked by fingerprint-literal top-64 re-ranked with 0.3·FAC + 0.7·FP (the code setting of E9/E10).

Scoring:
- **exact**: the deleted statement is proposed verbatim.
- **up to skolems**: hypothesized new entities may stand for query entities.
- **precision**: fraction of skolem-free proposals that are statements of the original function.
- **shape overlap**: Dice over the sub-expression shapes (entities abstracted) of the best proposal vs the deleted statement. This gives partial credit for restoring the right construct with small differences.

## Results

| analogue | recall exact | precision | best shape overlap | overlap ≥ 0.5 |
|---|---|---|---|---|
| **fused top-1** | **0.076** | **0.100** | **0.243** | **0.219** |
| lexical TF-IDF top-1 | 0.046 | 0.079 | 0.180 | 0.160 |
| random function (chance) | 0.001 | 0.010 | 0.026 | 0.007 |
| oracle: same algorithm by another author (66 queries) | 0.015 | 0.013 | 0.206 | 0.197 |
| fused top-1 on those same 66 queries | 0.076 | 0.060 | 0.355 | 0.348 |

Split by where the fused top-1 analogue comes from:

| top-1 analogue from | queries | recall exact | best shape overlap |
|---|---|---|---|
| same package (same author) | 619 | 0.079 | 0.251 |
| other package | 81 | 0.049 | 0.178 |

Corroboration over the fused top-5. Precision of skolem-free proposals by support: 0.022 (1 analogue, n = 1,903) → 0.092 (2, n = 131) → 0.333 (3, n = 21).

Restored exactly, for example:
- `(while-loop (lt left right) (swap (index characters left) (index characters right)))` in `reverse_string.iterative`;
- `(do (call _backtrack result nums (list-lit) c0))` in `subsets`.

## Findings

1. **Inference transfers to real code, far above chance but low in absolute terms.**
   - Structural analogues restore ~10× more of a deleted statement's structure than a random function (shape overlap 0.243 vs 0.026) and 1.35× more than lexical retrieval.
   - Exact restoration is rare (7.6%): real statements are large nested trees, and authors differ in details.
2. **The structurally nearest function beats the semantic twin.** The same algorithm by another author is a *worse* source of inferences than the function MARS retrieves: exact 0.015 vs 0.076, shape 0.206 vs 0.355 on the same queries.
   - 88% of retrieved analogues come from the same package. Same-author variants (iterative/recursive, sort/improved_sort) share statement-level idiom, and that idiom is what projection copies.
   - For inference, *structural* similarity, not shared purpose, is the right retrieval criterion. This is the core MARS premise, seen on real data.
3. **Corroboration remains the calibrated confidence** (precision 0.02 → 0.09 → 0.33 with support 1 → 2 → 3), as in E11 and E15. It is much lower in absolute terms than on synthetic data: real functions rarely have several analogues containing the same statement.
4. **Implication.** On code, MARS inferences are *hypotheses to rank*, not facts to assert. Support ≥ 3 is the only regime with useful precision, and it is rare. The main limit is the representation granularity (whole nested statements as facts). A finer statement decomposition, e.g. control-flow edges as separate facts (P7b), would let partial structure be projected and corroborated.
