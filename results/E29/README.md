# E29: learning which analogical transfers to trust (rules induced from analogy)

Full tables: `E27-{scientists,films}-hop{1,2}.md` / `.json`, each with config and seed. They contain the E27/E28 methods, the gated rows and the learned transfer table. The runner is `mars-bench e27`, with the same data and commands as [E28](../E28/README.md) and `--out results/E29`.

**Question:** E28's failure mode was spurious substitutions. For example, a film's composer was "inferred" as another person in the film who shares their citizenship, which was right 4% of the time. Can the memory learn from the outcomes of its own transfers which kinds of analogical inference to trust? And what does it learn?

This is the first step of the analogy → abstraction loop: consolidating corroborated transfers into reusable, explicit knowledge.

**Method.**
- **Typing.** Each analogical vote (an analogue's candidate inference for the missing relation) gets a *transfer type*:
  - a *copy* of the analogue's own object (a skolem); or
  - a *substitution* to a query entity, typed by the relation path that reaches it from the query root. The path is r1 or r1 · r2, e.g. advisor · employer.
- **Learning.** Queries are split 2-fold by entity. On one fold, the precision of each (relation, transfer type) is estimated from whether its votes were correct, smoothed towards the fold's mean vote precision with 5 pseudo-votes.
- **Gated analogy.** On the other fold, each vote is weighted by that learned precision. No rules are mined.

## Results (Hits@1, 10 analogues)

| method | scientists 1-hop | scientists 2-hop | films 1-hop | films 2-hop |
|---|---|---|---|---|
| analogy · MARS neighbours (E28) | 0.343 | 0.401 | 0.294 | 0.363 |
| **gated** analogy · MARS neighbours | 0.350 | 0.415 | 0.312 | **0.382** |
| analogy · hybrid neighbours (E28) | 0.387 | 0.434 | 0.318 | 0.360 |
| **gated** analogy · hybrid neighbours | 0.393 | 0.441 | 0.332 | 0.375 |
| mined rules ≤ 2 + copy · lexical (E28) | **0.400** | 0.433 | 0.342 | 0.364 |
| mined rules ≤ 2 + analogy · hybrid (E28) | 0.396 | **0.445** | **0.344** | 0.384 |
| mined rules ≤ 2 + gated analogy · hybrid | 0.395 | 0.442 | 0.340 | **0.386** |

**Transfers learned from analogy**, selected rows from the 2-hop runs. Precision is that of the analogical votes of the type; "mined" is the confidence of the same pattern as a globally mined rule.

| relation | ⇐ transfer (query path) | votes | precision | mined confidence |
|---|---|---|---|---|
| scientist · educated at | doctoral advisor · employer | 698 | **0.716** | 0.309 |
| scientist · educated at | doctoral advisor · educated at | 578 | 0.621 | 0.215 |
| scientist · citizenship | place of birth · country | 1,430 | **0.852** | 0.627 |
| scientist · citizenship | employer · country | 1,740 | 0.799 | 0.551 |
| scientist · field of work | doctoral advisor · field of work | 725 | 0.360 | 0.184 |
| scientist · languages spoken | native language | 145 | 1.000 | 0.950 |
| film · original language | director · languages spoken | 1,571 | **0.935** | 0.716 |
| film · original language | producer · languages spoken | 880 | 0.933 | 0.674 |
| film · country of origin | performer · citizenship | 100 | 1.000 | 0.933 |
| film · screenwriter | director | 999 | 0.592 | 0.337 |
| film · director | screenwriter | 955 | 0.468 | 0.400 |
| film · composer | performer | 92 | 0.924 | 0.756 |
| film · composer | director | 131 | **0.008** | 0.005 |
| film · composer | screenwriter | 406 | **0.002** | 0.007 |
| film · composer | copy (analogue's own composer) | 1,116 | 0.034 | — |

## Findings

1. **The memory learns which transfers to trust, and the learned table is readable domain knowledge.** Without any rule language or mining pass, the typed outcomes of analogical transfers recover rules such as:
   - *a scientist studied where their doctoral advisor worked*;
   - *citizenship is the country of birth*;
   - *a film's language is the language its director speaks*;
   - *the composer is often the performer*.

   The table also marks the E28 failure mode as untrustworthy: composer by substitution via the director or screenwriter is right 0.8% and 0.2% of the time. Each entry keeps its provenance (the analogues that produced the votes).
2. **Gating improves analogy in every condition,** by +0.007 to +0.019 Hits@1. **Gated analogy reaches mined rules + analogy without mining:** films 2-hop 0.382 vs 0.384, scientists 2-hop 0.441 vs 0.445. The two sources overlap, so adding mined rules on top of gated analogy gains nothing (0.386 / 0.442).
3. **Analogy applies rules in context.** The same pattern is far more precise when an analogy proposes it than when applied globally:
   - educated at ⇐ advisor · employer: 0.72 vs 0.31;
   - citizenship ⇐ birthplace · country: 0.85 vs 0.63;
   - screenwriter ⇐ director: 0.59 vs 0.34.

   The analogue exhibits the pattern *and* resembles the query, so the rule is used where it holds. Caveat: vote-level precision counts every agreeing analogue, while rule confidence counts each (x, y) once. The comparison is indicative, not exact.
4. **Limits.**
   - Gains are modest in absolute terms, because most errors are retrieval or coverage failures rather than bad transfers.
   - The transfer types are the paths in the query (length ≤ 2). Richer types (conjunctions, attributes) would need the SAGE generalization machinery.
   - The learned table is estimated offline over a fixed query set. An online version would update it from feedback on the engine's standing-query inferences (next step).

**Toward the abstraction loop:** the learned (relation ⇐ path, precision, provenance) table is a set of *rules induced from analogies*. The natural next step is to store these as first-class schema cases in the engine, apply them directly where they are reliable, and keep refining them from feedback. Analogy then serves both as a way to infer and as a way to learn.
