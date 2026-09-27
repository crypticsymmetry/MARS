# E18: representation granularity for code — nested statements vs flat blocks

Full table: [E18.md](E18.md). Reproduce with `mars-bench e18 [--seed 1]` (~4 s; needs `tools/fetch_e9_corpus.sh`).

**Question:** E17 found candidate inference on code is all-or-nothing, because the converters encode control structure by wrapping: a swap inside a guarded double loop is one fact, `(in-loop i (in-loop j (guards t (swap …))))`. Would a finer-grained representation help retrieval and inference?

**Flat representation.** It is built from the same KB, for any language:
- each distinct control context becomes a *block entity*: `(loop-block b1 i)`, `(guard-block b3 t)`, `(within b3 b2)`;
- each statement becomes a small fact placed in its block: `(in-block b3 (swap …))`.

Over the 1,113 Python functions, this turns 9,212 facts / 29,698 expressions into 14,079 facts / 31,436 expressions.

**Evaluation.**
- Retrieval: E9 task A (same algorithm, other package).
- Inference: E17's 700 deleted statements. The *same core statement* is deleted in both representations; in the flat one, blocks left empty are pruned.
- *core* = the statement itself is proposed; *in context* = it is proposed in the right control context.

## Results

Retrieval, MRR (R@1 / R@10):

| representation | fingerprint | FAC | fused w=0.3 | fused w=0.5 | fused w=0.7 |
|---|---|---|---|---|---|
| nested | **0.300** | 0.283 | **0.359** (0.27 / 0.55) | 0.334 | 0.313 |
| flat | 0.267 | **0.354** | 0.314 (0.18 / **0.67**) | 0.351 | **0.370** (0.27 / 0.58) |

Inference, fused top-1 analogue:

| representation | core restored | restored in context | precision | proposals / query |
|---|---|---|---|---|
| nested | **0.189** | 0.079 | 0.100 | 2.17 |
| flat | 0.106 | 0.091 | **0.126** | 1.23 |
| either (union of both top-1s) | 0.194 | **0.136** | | |

Core restored by both representations: 0.100 of queries; by nested only: 0.089; by flat only: 0.006.

## Findings

1. **Granularity moves information between MAC and FAC.**
   - Flattening makes the *mapper* much better at finding the same algorithm by another author (FAC MRR 0.283 → 0.354, +25%). Small statements with explicit block structure align across authors more easily than big nested trees.
   - It makes the *fingerprint* worse (0.300 → 0.267): nested trees give richer relational n-grams.
   - The best fusion weight moves with it, from w = 0.3 (nested, E10) to w = 0.7 (flat). At each representation's best weight, retrieval is equal (0.359 vs 0.370; ~30 queries, within noise).
   - The fusion weight is a property of the representation, not a constant.
2. **Nested restores statements; flat restores placement.**
   - Nested projection copies a whole wrapped statement, so the core often comes out right in the wrong context (0.189 core vs 0.079 in context).
   - Flat projection yields fewer, more precise proposals that more often land in the right block (0.091).
   - The two representations get *different* placements right, so their union restores 0.136 in context, 1.7× either alone. Flat adds almost nothing on cores (0.006).
3. **Design consequence.** Representation granularity is a real knob, and no single granularity dominates. MARS should support multiple *views* of a case (nested for fingerprint retrieval and statement proposals, flat for mapping and placement), fused at the score and proposal level. This generalizes the channel design of the fingerprint to whole representations. It is recorded as an open design direction (multi-view cases), not built.
