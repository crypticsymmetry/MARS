# E27: knowledge-graph completion by analogy

Full tables (1, 5 and 10 analogues; per relation): `E27-wikidata-scientists.md`, `E27-wikidata-films.md`. The `-literal` files are the structure-weighted fingerprint ablation. Each `.json` records its config and seed.

Reproduce:
```
python3 tools/kg2mars.py data/kg-scientists OUT/sci --condition C
python3 tools/kg2mars.py data/kg-films OUT/films --condition C
mars-bench e27 --data OUT/sci --tag wikidata-scientists
mars-bench e27 --data OUT/films --tag wikidata-films --relations wdt:p57,wdt:p161,wdt:p58,wdt:p162,wdt:p86,wdt:p344,wdt:p136,wdt:p495,wdt:p364,wdt:p272
```

**Question:** Can candidate inferences predict missing facts in a real KG? And is *analogical* inference better than copying from similar entities?

Analogical inference here means projecting through the structure mapping. A projected object is either a *substitution* (an entity the query already has, reached through the correspondences) or a *copy* (a skolem for the analogue's own object). Copying from similar entities is nearest-neighbour voting. E17 found candidate inference weak on code (7.6% exact). KGs give a large, exactly scored test.

**Setup.**
- Memory: the Wikidata side of E26's samples, one case per entity (its claims plus the labels of object entities).
  - 993 scientists; 8 relations: educated at, employer, field of work, citizenship, occupation, award, member of, languages spoken.
  - 992 films; 10 relations: director, cast member, screenwriter, producer, composer, director of photography, genre, country of origin, original language, production company.
- Query: an entity with *all* its facts of one relation removed, and the labels of objects no longer mentioned. Up to 300 queries per relation, seed 1: 2,400 scientist queries and 3,000 film queries.
- Predictions are votes over the top-m analogues, weighted by similarity. Metrics: Hits@1, Hits@10 and MRR of the first correct object.

**Neighbours.**
- **MARS fused:** fingerprint top-50 (surface profile) re-ranked by ½FAC + ½FP.
- **Lexical:** TF-IDF over entity and predicate names.
- **Hybrid:** RRF of MARS fused and lexical.
- **Random.**

**Baselines.**
- **Popularity:** each relation's most frequent objects.
- **Length-1 rules** r′(x,y) ⇒ r(x,y) (AMIE-style), with leave-one-out confidence. This is the global counterpart of a substitution, alone and added to neighbour votes.

## Results (10 analogues)

| method | scientists Hits@1 | Hits@10 | MRR | films Hits@1 | Hits@10 | MRR |
|---|---|---|---|---|---|---|
| popularity | 0.222 | 0.535 | 0.322 | 0.144 | 0.305 | 0.200 |
| copy · random neighbours | 0.179 | 0.348 | 0.237 | 0.128 | 0.207 | 0.156 |
| copy · MARS fused neighbours | 0.318 | 0.541 | 0.395 | 0.207 | 0.323 | 0.249 |
| copy · lexical neighbours | 0.378 | 0.601 | 0.454 | 0.266 | 0.381 | 0.309 |
| **analogy** · MARS fused neighbours | 0.343 | 0.570 | 0.421 | 0.294 | 0.431 | 0.344 |
| **analogy** · lexical neighbours | 0.385 | 0.605 | 0.461 | 0.325 | 0.461 | 0.375 |
| **analogy** · hybrid neighbours | 0.387 | 0.612 | 0.463 | 0.318 | 0.460 | 0.369 |
| rules alone | 0.109 | 0.130 | 0.118 | 0.177 | 0.211 | 0.190 |
| rules + copy · lexical | **0.400** | **0.633** | **0.482** | 0.342 | **0.525** | **0.405** |
| rules + analogy · hybrid | 0.396 | 0.629 | 0.477 | **0.344** | 0.510 | 0.402 |

Per-relation Hits@1 where substitution matters (films, MARS neighbours; copy → analogy; rules alone):
- director: 0.06 → 0.28 (rules 0.37)
- screenwriter: 0.06 → 0.48 (rules 0.57)
- production company: 0.14 → 0.26 (rules 0.32)

Scientists:
- educated at: 0.10 → 0.17 (rules 0.22)
- employer: 0.08 → 0.18 (rules 0.26)

**Analogy vs copy with the same analogues** (MARS, 10 analogues):
- On queries where the two top-1 predictions differ, analogy is right far more often: 292 vs 31 on films, 89 vs 29 on scientists.
- Top-1 predictions that are pure substitutions (21% on films, 11% on scientists) have precision 0.39 and 0.21.

**Calibration** (analogy · MARS, top-1 precision by support, i.e. by the number of analogues proposing it):

| support | 1 | 2 | 3 | 4 | 5+ |
|---|---|---|---|---|---|
| scientists | 0.076 | 0.184 | 0.341 | 0.453 | 0.665 |
| films | 0.063 | 0.274 | 0.414 | 0.442 | 0.661 |

**Fewer analogues** (full tables): with 1 analogue, rules + analogy · hybrid is best on scientists (0.263 vs 0.248 for rules + copy · lexical) and ties on films (0.286).

**Ablation:** the structure-weighted (literal) fingerprint makes MARS neighbours much worse. Scientists: copy 0.245, analogy 0.270. Films: copy 0.146, analogy 0.210.

## Findings

1. **Analogical inference beats copying, consistently.** With every neighbour set, projecting through the mapping beats voting for the analogues' own objects. Films gain +42% relative Hits@1 with MARS neighbours (0.207 → 0.294) and +22% with lexical ones. On disagreements analogy wins 9 : 1 on films and 3 : 1 on scientists. The gain comes from *substitutions*, which carry relational regularities from the analogue into the query:
   - the director also wrote the film;
   - the scientist works where they studied;
   - the producer's company is the production company.

   This is the first real-data result where candidate inference clearly works; compare E17 on code.
2. **But a substitution is a local, lazy version of a length-1 rule.** Rules mined over the whole KG predict these relations slightly better than substitutions from 10 analogues (director 0.37 vs 0.28, screenwriter 0.57 vs 0.48). Once rules are added to neighbour votes, analogy adds nothing at 10 analogues: 0.400 vs 0.396 and 0.342 vs 0.344. Analogy only helps when few analogues are used.
   - Analogy's value is operational, not accuracy: it needs no mining pass and no fixed rule language, it updates incrementally, and each prediction carries its provenance (which analogue, which correspondence).
   - Where rules can be mined, they are the better global summary.
3. **Neighbour choice: exact identity overlap beats fingerprints for instance similarity.** TF-IDF over entity names finds better neighbours (0.378) than MARS's fingerprint + FAC (0.318). Fusing both helps a little.
   - The surface profile is essential. Structure-weighted fingerprints find scientists who look alike *structurally*, and that says little about their specific objects.
   - This matches E26: labels identify instances; structure identifies schemas and roles.
4. **Support is a calibrated confidence on real KGs.** Top-1 precision rises monotonically, from 0.06–0.08 at support 1 to 0.66 at support 5 or more, in both domains. This replicates E11's corroboration result outside synthetic data and supports accept/abstain thresholds for KG completion (support ≥ 4: precision ≈ 0.58 at 31–37% of queries).
5. **Where this leaves KG completion with MARS.** A practical pipeline would retrieve analogues by exact identity (or hybrid) neighbours and project through the mapping. It would add mined rules where available, and accept only corroborated predictions. On this data that gives Hits@1 about 0.40 on scientists and 0.34 on films, against 0.22 and 0.14 for popularity, with MARS contributing calibration and explanations rather than raw accuracy.
