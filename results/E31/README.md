# E31: closing the loop — applying rules induced from analogy, and where KG completion errors really are

Tables: `E30-{scientists,films}-hop2[-feedback-combined].md` / `.json` (config and seed included). The runner is `mars-bench e30`, which now reports a third ranking and the answer-location split. Commands as in [E30](../E30/README.md), plus `--feedback-from combined` for the variant.

**Question:** E30's engine learns which transfer types are reliable and states them as rules. If those rules are applied *directly*, can they answer queries that no analogue answers? This is the analogy → abstraction loop: induce rules from analogies, then reason with the rules.

**Method.**
- `Engine::rule_inferences(q, min_n, min_precision)` takes every transfer type with ≥ 20 feedback outcomes and precision ≥ 0.5 whose body is a relation path. It fires the type as a rule on every entity pair of the query that the body links, skipping facts already present. Results have support 0, their reliability as score, and take part in feedback like any inference.
- **Rankings.** In the E30 stream, a third ranking combines the learned analogical vote share with the reliability of the rules induced *so far* (no look-ahead).
- **Feedback source.** By default the simulated user sees the learned ranking, so E30's two rankings are unchanged. The `-feedback-combined` variant shows the combined one.
- **Answer location.** Every query is classified by whether a correct object is already an entity of the query case, i.e. reachable by substitution or a rule.

## Results (Hits@1)

| | raw | learned (E30) | learned + induced rules |
|---|---|---|---|
| scientists, 2,400 queries | 0.401 | 0.409 | 0.407 |
| films, 3,000 queries | 0.360 | 0.375 | **0.379** |
| scientists, user sees combined ranking | 0.401 | 0.410 | 0.408 |
| films, user sees combined ranking | 0.360 | 0.374 | 0.378 |

Induced rules fired on 402 scientist and 947 film queries. They produced a correct top-1 that no analogue proposed on **0 and 5** queries.

**Where the answer is** (Hits@1, raw → learned → learned + rules):

| queries | scientists | films |
|---|---|---|
| a correct object is already in the query (relational inference) | 706 (29%): 0.710 → 0.746 → 0.742 | 1,160 (39%): 0.771 → 0.826 → **0.841** |
| no correct object in the query (a new value: only copying can reach it) | 1,694 (71%): 0.272 → 0.269 → 0.268 | 1,840 (61%): 0.102 → 0.090 → 0.087 |

## Findings

1. **Applying induced rules directly adds little:** −0.002 on scientists, +0.004 on films. Where a path rule can fire, an analogue with the same pattern is almost always among the top 10 and has already proposed the substitution. Analogy and its induced rules cover the same ground; the rules are its *summary*, not an extension. This agrees with E28/E29, where mined rules and analogy were interchangeable at depth ≤ 2.
2. **KG completion splits into two different problems, and MARS solves one of them.**
   - **Relational inference:** the answer is already in the query, linked by some path. Here analogy is strong (Hits@1 0.71–0.77 raw) and learning improves it (0.75 / 0.84).
   - **New-value prediction:** the answer is an entity the query does not mention, e.g. the award, occupation, genre or member-of institution. This is 61–71% of queries, and only copying from similar entities can reach it (0.27 / 0.10). Learned reliability slightly *hurts* here (−0.003 / −0.012), because it upweights substitution types, which cannot be right when the answer is absent.

   The headline Hits@1 of E27–E30 therefore mixes a strong relational-inference score with a weak recommendation score.
3. **Implications.**
   - **Report the split.** MARS's inference claim is about relational inference. New-value prediction is essentially collaborative filtering, and it is limited by neighbour quality (E27: identity-overlap neighbours help there).
   - **Route by answer type.** When no reliable substitution exists, the ranking should fall back to copying from the best instance neighbours, e.g. identity-overlap or hybrid neighbours, or a dedicated recommender. Reliability learning should condition on whether any substitution candidate exists.
   - **Where the next gains are:** better instance retrieval for copying (fusing an identity channel into the engine, the open design item), not more rules.
