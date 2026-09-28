# E34 (pre-registered, addendum A): a dataflow front end for code

Protocol and predictions: [docs/PREREGISTRATION.md, addendum A](../../docs/PREREGISTRATION.md#addendum-a-2026-09-28-after-ev1ev3-e34-a-dataflow-front-end-for-code), written before any E34 run.

The front end is `tools/py2pdg.py`. The harness is `tools/ev2_codenet.py prepare --frontend pdg --same-as`, `mars-bench ev2 [--cands]` and `tools/e34_eval.py`. Everything downstream of the front end is EV2's frozen pipeline: the v0.1 encoder, Mode K and the mapper, with ½FAC + ½FP-literal over the fingerprint top-100.

**What changes.** py2mars encodes syntax: nested expression terms over variable names. py2pdg encodes dataflow:
- one fact per operation over value entities, e.g. `(d-add v7 v3 v5)` and `(d-bi-len v4 v2)`;
- variables resolved to their reaching definition;
- loops as `(d-iter L elem iterable)`, with a `(d-phi head before end)` fact for every loop-carried variable, which makes accumulators and running maxima explicit;
- branches merged by `(d-merge m then else cond)`;
- `xs = []` + `xs.append(e)` in a loop, and list comprehensions, both normalized to `(d-collect xs L e)`;
- common-subexpression elimination for pure operations;
- `(d-input v)` / `(d-output v)` for I/O.

Variable names are kept as unary attributes `(nm-x v)`. Attributes do not enter the mapper; they reach only the fingerprint's surface channel.

Reproduce (test = EV2's sample):
```
python3 tools/ev2_codenet.py prepare W                    # EV2's sample (seed 1), py2mars
python3 tools/ev2_codenet.py prepare WP --frontend pdg --same-as W
mars-bench ev2 --data W ; mars-bench ev2 --data WP
python3 tools/e34_eval.py cands WP W
mars-bench ev2 --data WP --cands emb_cands.json
python3 tools/e34_eval.py evaluate W WP results/E34
```
The development sample is `prepare --seed 2 --exclude-seed 1`: 200 problems from the 600 not in EV2's sample.

## Development (settings chosen here only)

The development sample has 5,970 programs. The two front ends encode the same programs; py2pdg skips 1.

| py2pdg variant (MARS fused) | MAP@R | FP-literal only |
|---|---|---|
| py2mars (reference) | 0.240 | 0.196 |
| v1: loop/guard context fact on every value | stopped after 41 min: context facts (a third of all facts) cross-matched and made the mapper too slow | — |
| context only on effects (output, stores) | 0.359 | 0.291 |
| **no context facts (chosen)** | **0.361** | **0.293** |
| context on effects, no variable names | 0.326 | 0.231 |
| context on effects, no CSE | 0.349 | 0.283 |

Chosen: `PDG_CTX=none` (the simplest variant; context did not help), names on, CSE on. Tables: [E34-dev.md](E34-dev.md) and [E34-dev-ablations.json](E34-dev-ablations.json).

## Test (EV2's sample, run once)

The test sample is EV2's 5,982 programs; py2pdg skips 1, which is ranked as an empty list. Table: [E34-test.md](E34-test.md); per-query outcomes are in `E34-test.json`.

| method | MAP@R [95% CI] | Hits@1 | MRR |
|---|---|---|---|
| code embedding (jina-embeddings-v2-base-code, as in EV2) | 0.473 [0.466, 0.480] | 0.856 | 0.892 |
| **embedding top-100 re-ranked by ½ cosine + ½ MARS FAC (py2pdg)** | **0.498** [0.491, 0.504] | **0.892** | **0.919** |
| **MARS fused, py2pdg** | **0.319** [0.313, 0.325] | 0.810 | 0.855 |
| MARS fused, py2mars (EV2) | 0.211 [0.206, 0.216] | 0.699 | 0.761 |
| MARS FP-literal, py2pdg | 0.252 | 0.725 | 0.786 |
| MARS FP-literal, py2mars (EV2) | 0.166 | 0.609 | 0.689 |

The py2mars and embedding rows reproduce EV2's numbers exactly.

**Predictions** (scored by `tools/ev_report.py`, criteria committed before the test; see [EV_PREDICTIONS.md](../EV_PREDICTIONS.md)):

| | prediction | paired Δ MAP@R [95% CI] | verdict |
|---|---|---|---|
| P10 | py2pdg > py2mars | +0.108 [+0.103, +0.112] | **confirmed** |
| P11 | MARS with py2pdg still below the code embedding | embedding − MARS +0.154 [+0.147, +0.162] | **confirmed** |
| P12 | embedding + MARS re-rank ≥ embedding alone | +0.024 [+0.020, +0.028] | **confirmed** (and strictly above) |

**Deviations from addendum A:**
1. The addendum describes control structure as "kept as loop and guard context". The development sample showed the per-value context facts do not help and make the mapper far slower, so the final front end drops them (`PDG_CTX=none`). Loop and branch structure remain, through the `d-iter`/`d-while`/`d-within`, `d-phi` and `d-merge` facts. This was chosen on development only.
2. The addendum says structure is "independent of variable names". That holds for the relational facts the mapper uses. Names are also kept as attributes, which reach only the fingerprint's surface channel, because they helped on development (0.359 vs 0.326). Without names, the development gain over py2mars is still +0.086.
3. Harness bug, fixed before any test metric was computed. The first test pass computed the embedding over py2pdg's 5,981 encoded programs instead of EV2's 5,982 and stopped with an error, so no metric was produced. The embedding now uses the full program set, and FAC scores stay aligned with their candidates (commit `0aed147`).

## Findings

1. **Representation was the bottleneck, as EV2 suggested.** The change of representation alone lifts MARS by half, from MAP@R 0.211 to 0.319 (Hits@1 0.70 → 0.81), with the same retrieval, mapper and settings. The fingerprint alone gains as much (0.166 → 0.252), so the dataflow form helps both the cheap and the explicit stage.
2. **MARS is still well below a pretrained code embedding** (0.319 vs 0.473). An embedding trained on large code corpora captures lexical and idiomatic regularities that a hand-built structural encoding does not.
3. **As a re-ranker on top of the embedding, MARS helps** (+0.024 MAP@R, Hits@1 0.856 → 0.892). This is the division of labour the positioning after EV1–EV3 proposed: an embedding for recall, MARS for structural re-ranking and explanation. It is the first external benchmark where adding MARS improves on the strong baseline.
4. **The gain is concentrated where the embedding is unsure.** This analysis is exploratory, not pre-registered ([E34-test-by-confidence.json](E34-test-by-confidence.json)). By quartile of the embedding's top-1 cosine, which is known at query time, the re-rank gains:

   | embedding top-1 cosine quartile | 1 (lowest) | 2 | 3 | 4 |
   |---|---|---|---|---|
   | MAP@R change from the re-rank | +0.049 | +0.033 | +0.014 | +0.003 |

   It never loses on average in any quartile. A confidence-gated or learned fusion weight is the obvious next step; that is priority 3 of the upgrade list, and needs its own development and test split.
