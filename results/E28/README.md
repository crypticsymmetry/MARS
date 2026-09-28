# E28: relational depth — does analogy gain on rules and copying when patterns run through longer paths?

Full tables (1, 5 and 10 analogues; per relation; calibration): `E27-{scientists,films}-hop{1,2}.md` / `.json`, each with its config and seed. The runner is `mars-bench e27` (E27's protocol), with length-≤2 rule baselines and root-only hold-out.

Reproduce:
```
python3 tools/kg_hop2.py data/kg-scientists              # optional re-fetch; versioned as wikidata_hop2.jsonl
python3 tools/kg_hop2.py data/kg-films --via P57,P58,P162,P272,P495,P86 --props P27,P17,P1412,P37,P495
python3 tools/kg2mars.py data/kg-scientists OUT/sci2 --condition C --hop2
python3 tools/kg2mars.py data/kg-films OUT/films2 --condition C --hop2
mars-bench e27 --data OUT/sci2 --out results/E28 --tag scientists-hop2
mars-bench e27 --data OUT/films2 --out results/E28 --tag films-hop2 --relations wdt:p57,wdt:p161,wdt:p58,wdt:p162,wdt:p86,wdt:p344,wdt:p136,wdt:p495,wdt:p364,wdt:p272
```
(`-hop1` runs: the same without `--hop2`.)

**Question:** E27's cases were one-hop stars. There, analogy's gain over copying came from substitutions that are exactly length-1 rules, and mined rules + nearest neighbours matched it. Structure mapping should matter more when a regularity runs through a longer relational path. For example, the language of a film is the official language of its country of origin; a scientist's citizenship is the country of their birthplace. Rules then need longer bodies, and the mining space grows as |R|^L. Does analogy keep pace with explicitly mined length-2 rules, without any mining?

**Setup.** As in E27 (Wikidata side, 2,400 scientist and 3,000 film queries, seed 1), plus the second-hop claims of each entity's objects (`kg2mars --hop2`):
- **Scientists:** advisor, birth and death place, alma mater and employer → their education, employer, citizenship, field and country. 5,131 claims.
- **Films:** director, screenwriter, producer, production company, country of origin and composer → their citizenship, country, languages spoken and official language. 4,773 claims.

Only the root entity's own facts of the relation are held out. The query keeps only facts still reachable from the root. A removed alma mater therefore cannot survive as the subject of its own claims; an earlier version leaked exactly that, which inflated educated-at to 0.80.

**New baseline: rules of length ≤ 2.** Both r′(x,y) ⇒ r(x,y) and r1(x,z) ∧ r2(z,y) ⇒ r(x,y), with leave-one-out confidence, mined exhaustively over the memory.

## Results (Hits@1, 10 analogues)

| method | scientists 1-hop | scientists 2-hop | films 1-hop | films 2-hop |
|---|---|---|---|---|
| popularity | 0.222 | 0.223 | 0.144 | 0.144 |
| copy · lexical neighbours | 0.378 | 0.406 | 0.266 | 0.254 |
| copy · MARS neighbours | 0.318 | 0.319 | 0.207 | 0.238 |
| analogy · lexical neighbours | 0.385 | 0.425 | 0.325 | 0.309 |
| analogy · MARS neighbours | 0.343 | 0.401 | 0.294 | **0.363** |
| analogy · hybrid neighbours | 0.387 | 0.434 | 0.318 | 0.360 |
| rules ≤ 2 alone | 0.109 | 0.220 | 0.177 | 0.330 |
| rules ≤ 2 + copy · lexical | **0.400** | 0.433 | 0.342 | 0.364 |
| rules ≤ 2 + analogy · hybrid | 0.396 | **0.445** | 0.344 | **0.384** |

Gain of analogy over copy with the same (MARS) analogues:
- scientists: +0.025 at 1 hop → +0.082 at 2 hops;
- films: +0.087 → +0.125.

Disagreements at 2 hops (analogy right : copy right) are 258 : 61 on scientists and 445 : 71 on films.

Per relation, where the second hop carries the regularity (Hits@1: copy · MARS → analogy · MARS; rules ≤ 2 alone):

| relation | 1-hop copy → analogy | 2-hop copy → analogy | rules ≤ 2 |
|---|---|---|---|
| film · country of origin | 0.57 → 0.58 | 0.74 → **0.95** | 0.89 |
| film · original language | 0.61 → 0.61 | 0.70 → **0.85** | 0.75 |
| scientist · educated at | 0.10 → 0.17 | 0.10 → 0.35 | 0.35 |
| scientist · citizenship | 0.50 → 0.50 | 0.53 → 0.76 | 0.69 |
| scientist · employer | 0.08 → 0.18 | 0.09 → 0.22 | 0.27 |

**Calibration** (analogy · MARS at 2 hops, top-1 precision by support 1 / 2 / 3 / 4 / 5+):
- scientists: 0.10 / 0.19 / 0.32 / 0.38 / 0.67;
- films: 0.07 / 0.17 / 0.30 / 0.43 / 0.73.

## Findings

1. **Analogy's advantage grows with relational depth.** With a second hop, analogical inference pulls further ahead of copying from the same analogues: the gain grows 3× on scientists and 1.4× on films.
   - Substitutions now travel along paths. For example, the base film's language is its country's official language; the country corresponds to the query film's country, so the query film gets that country's official language.
   - Country of origin reaches 0.95 and language 0.85, above length-2 rules mined over the whole memory (0.89 and 0.75).
2. **Without mining, analogy matches rules + kNN at depth 2; at depth 1 it did not.**
   - 1 hop: analogy alone 0.387 / 0.318 vs rules + lexical kNN 0.400 / 0.342.
   - 2 hops: analogy alone 0.434 / 0.363 vs rules ≤ 2 + lexical kNN 0.433 / 0.364.
   - Analogy gets there with no mining pass, no rule language and no path-length bound; its cost is one mapping per analogue, whatever the pattern length. Mining is still feasible at length 2 on 1,000 entities, but its space grows as |R|^L.
   - The combination remains best (0.445 / 0.384): rules supply global statistics, analogy supplies local, context-specific transfer.
3. **Structure-aware neighbours matter more as cases grow richer.** At 2 hops, lexical identity neighbours get *worse* on films (copy 0.266 → 0.254; genre 0.45 → 0.28), because the second-hop entities dilute the name overlap. MARS neighbours get better (copy 0.207 → 0.238), and analogy with MARS neighbours beats analogy with lexical ones (0.363 vs 0.309). E27's "identity beats structure for neighbours" holds for shallow cases only.
4. **Support stays a calibrated confidence** at 2 hops: precision 0.07–0.10 at support 1, 0.67–0.73 at support 5+.
5. **Failure mode: spurious substitutions.** Film composers get 193 substitutions with precision 0.04. The mapper aligns the base's composer with some query person who shares citizenship and language, and projects them. Substitution precision per relation is a natural gate: accept substitutions only for relations where they have held up, i.e. learn from corroboration (E11) which transfer types to trust.

**Bottom line.** E27 showed analogy ≈ lazy length-1 rules. E28 shows that as regularities get deeper, analogy keeps pace with explicitly mined longer rules without mining them. Combined with rules it gives the best results in both domains. This is the first real-data evidence for the structure-mapping thesis of the project: systematic relational structure transfers, and the benefit grows with relational depth.
