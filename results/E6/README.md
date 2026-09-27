# E6: incremental maintenance (gate G5 / H3)

**Setup:** `mars-engine` holds N live cases (generated groups) and 200 **standing queries**. Each query is a group base with one root higher-order fact removed, i.e. an open problem whose analogues can supply the missing fact. The engine receives 5,000 random updates:

| update | share |
|---|---|
| remove a fact | 35% |
| re-add a removed fact | 25% |
| add a new distractor fact | 10% |
| add a new case | 20% |
| remove a case | 10% |

30% of the updates that pick a case pick one currently in some standing query's result (a "hot" case). Every 1,000 updates, 50 random standing queries are checked against a **from-scratch recomputation**: the ranked top-k (by fused score) *and* the set of believed candidate inferences must match exactly. Reproduce with `mars-bench e6 --groups G --sq 200 --updates 5000 [--mode pipeline|exact]`.

Two standing-query semantics:

- **Pipeline** (default): exact top-64 by fingerprint (the MAC set), maintained incrementally, re-ranked by fused FAC score. The same semantics as the E3 pipeline, which matched the exhaustive bound.
- **Exact-fused**: exact top-k by fused score over *all* cases, using the bound fused ≤ w + (1−w)·fp.

## Results

| mode | live cases | exactness (mismatches / checks) | mean µs / update | median µs | recompute all SQs from scratch | speed-up |
|---|---|---|---|---|---|---|
| pipeline | 10⁴ | **0 / 250** | 350 | 174 | 241 ms | 680× |
| pipeline | 10⁵ | **0 / 250** | 300 | 129 | 438 ms | 1,482× |
| pipeline | 10⁶ | **0 / 250** | 277 | 127 | 5,860 ms | **21,182×** |
| exact-fused | 10⁴ | **0 / 150** | 3,077 | 1,806 | 15,519 ms | 5,447× |

Per-update work (pipeline, 10⁶; details in `E6-pipeline-n1000000.md`):

| update | encodes | standing-query bound checks | FAC evaluations | full SQ recomputes | TMS nodes touched |
|---|---|---|---|---|---|
| fact update | 1 | 200 (one O(1) Hamming test per SQ) | 7–14 | ≈ 0 | ≈ 0.4 |
| new case | 1 | 200 | 0.6 | 0 | 0 |

## Findings

1. **H3 holds.** Update cost is *flat in N*: 350 → 300 → 277 µs mean from 10⁴ to 10⁶ cases. It is bounded by one re-encode plus one O(1) test per standing query, plus a few memoized re-mappings when a changed case sits in a result set. Recomputing from scratch grows with N (0.24 s → 5.9 s), so the advantage grows from 680× to **21,000×**.
2. **Exactness is verified, not assumed.** 0 mismatches across 1,150 checks, covering ranked results *and* the TMS-maintained inference sets. The one mismatch in an earlier 10⁶ run exposed a real semantic ambiguity: fingerprint-score *ties* at the 64th-candidate boundary, which appear at scale. It was fixed by making the candidate set tie-inclusive.
3. **Provenance works end to end.** Inferences are justified in the JTMS by the current mapping *and* the base facts they were projected from. Retracting a base fact withdraws exactly the dependent inferences through the TMS (≈ 0.4 nodes touched per update), before any re-mapping, and emits `InferenceOut` events.
4. **Exact-fused semantics costs ~10× more per update.** The bound fused ≤ w + (1−w)·fp is loose (FAC ≤ 1), so nearly every update must map the changed case against every standing query (~200–440 FAC evaluations), and initialization is ~45× slower. **Pipeline is the default.** A tighter cheap upper bound on normalized SME scores would make exact-fused viable (open question).
5. **Scaling cost is linear in the number of standing queries,** not in N. At 200 SQs this is ~200 Hamming tests per update. Thousands of SQs would benefit from indexing the queries themselves (reverse k-NN), which the design anticipated.
