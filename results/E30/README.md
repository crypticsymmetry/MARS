# E30: a memory that improves with use — online transfer reliability in the engine

Tables, learning curves and induced rules: `E30-{scientists,films}-hop2[-feedback-all].md` / `.json` (config and seed included).

Reproduce:
```
python3 tools/kg2mars.py data/kg-scientists OUT/sci2 --condition C --hop2
mars-bench e30 --data OUT/sci2 --tag scientists-hop2                      # --feedback 1000: check every suggestion
python3 tools/kg2mars.py data/kg-films OUT/films2 --condition C --hop2
mars-bench e30 --data OUT/films2 --tag films-hop2 --relations wdt:p57,wdt:p161,wdt:p58,wdt:p162,wdt:p86,wdt:p344,wdt:p136,wdt:p495,wdt:p364,wdt:p272
```

**Question:** E29 learned offline which analogical transfers to trust. Can the *engine* do this online? It would have to learn from feedback on its own suggestions while in use, keep the learned reliability across restarts, and surface what it has learned as rules.

**What was built** (`mars-engine`, exposed in the Python bindings):
- **Transfer types** (`transfer.rs`). Every candidate inference gets a domain-independent type, derived from its projection against the query:
  - a binary fact between two query entities is typed by the relation paths (length ≤ 2, with direction) that link them in the query, e.g. `educated-at<=advisor.employer`;
  - a fact about a hypothesized new entity is `f<=new:<positions>`, i.e. a copy;
  - anything else is typed by its argument kinds.
- **`Engine::feedback(query, inference, correct)`** updates per-type outcome counts. `TransferStats` gives the smoothed precision, using the global precision as a 5-pseudo-vote prior.
- **Ranking.** `Engine::infer(q, k, exclude)` (one-off) and `Engine::ranked_inferences(sq)` (standing queries) rank inferences by reliability × Σ fused score of the proposing analogues. `explain` shows each inference's transfer type and reliability.
- **`Engine::induced_rules(min_n, min_precision)`** renders the reliable types as rules, e.g. `educated-at(x, y) ⇐ advisor(x, z) ∧ employer(z, y)`.
- **Persistence.** Feedback is logged (`transfer-outcome …`) and the counts are snapshotted, so a reopened store keeps what it learned (unit-tested in Rust and Python).
- **Config.** `EngineConfig::first_order_inferences` draws inferences for first-order facts such as KG triples, and `Engine::set_profile` switches the fingerprint profile.
- **Python.** `Engine(..., first_order=, profile=, fac_weight=)`, `suggest(case, k, exclude)`, `ranked(sq)`, `feedback(case, text, correct)`, `induced_rules(min_n, min_precision)`.

**Protocol.**
- The E28 two-hop hold-out queries (2,400 scientist, 3,000 film) arrive as a stream, in seeded random order, at an engine holding the other entities. Settings: surface profile, ½FAC + ½FP over the fingerprint top-50, top-10 analogues, the entity's own full case excluded.
- After each query, a simulated user checks the top 3 suggestions for the held-out relation and the outcomes go back into the engine. The variant `-feedback-all` checks every suggestion.
- The learned ranking is compared, over the same stream, with the engine's raw ranking (Σ fused score of the proposing analogues).

## Results

| | raw Hits@1 | learned Hits@1 | Δ | Hits@10 raw → learned | MRR raw → learned | feedback events |
|---|---|---|---|---|---|---|
| scientists (2,400 queries) | 0.401 | **0.409** | +0.008 | 0.627 → 0.629 | 0.482 → 0.489 | 6,885 |
| scientists, check every suggestion | 0.401 | 0.410 | +0.009 | 0.627 → 0.630 | 0.482 → 0.489 | — |
| films (3,000 queries) | 0.360 | **0.375** | +0.015 | 0.486 → 0.488 | 0.408 → 0.418 | — |
| films, check every suggestion | 0.360 | 0.372 | +0.012 | 0.486 → 0.488 | 0.408 → 0.417 | — |

**Learning curves** (Δ Hits@1 per eighth of the stream):
- scientists: +0.003, +0.010, −0.003, +0.007, +0.020, +0.007, +0.013, +0.010;
- films: +0.016, +0.011, +0.013, +0.016, +0.016, +0.008, +0.019, +0.016.

**Rules induced online** (selected; ≥ 20 outcomes):

| rule (Wikidata properties) | reads as | precision | outcomes |
|---|---|---|---|
| `p27(x,y) ⇐ p19(x,z) ∧ p17(z,y)` | citizenship = country of the birthplace | 0.77 | 221 |
| `p27(x,y) ⇐ p108(x,z) ∧ p17(z,y)` | citizenship = country of the employer | 0.72 | 284 |
| `p69(x,y) ⇐ p184(x,z) ∧ p108(z,y)` | studied where the doctoral advisor worked | 0.56 | 164 |
| `p1412(x,y) ⇐ p103(x,y)` | speaks their native language | 1.00 | 28 |
| `p495(x,y) ⇐ p364(x,z) ∧ p37(y,z)` | film's country is the one whose official language is the film's language | 1.00 | 145 |
| `p495(x,y) ⇐ p57(x,z) ∧ p27(z,y)` | film's country = director's citizenship | 0.93 | 284 |
| `p364(x,y) ⇐ p57(x,z) ∧ p1412(z,y)` | film's language = a language the director speaks | 0.84 | 236 |
| `p86(x,y) ⇐ p175(x,y)` | composer = performer | 0.91 | 22 |

Least reliable types, learned and suppressed:
- scientists: educated-at or employer ⇐ birthplace or death place (0 / 45–58);
- films: director of photography ⇐ screenwriter or director (0 / 20–25); composer ⇐ cast member (0 / 26).

## Findings

1. **The engine now improves with use.** Feedback on its own suggestions raises Hits@1 over the same stream: +0.008 on scientists (0.401 → 0.409) and +0.015 on films (0.360 → 0.375). The gain is positive in 7 of 8 segments on scientists and all 8 on films, and appears within the first few hundred feedback events.
   - Checking every suggestion instead of the top 3 adds nothing, so cheap feedback is enough.
   - The engine's raw ranking reproduces the E27/E28 runner's analogy · MARS results (0.401, and 0.360 vs 0.363), a consistency check between the engine path and the experiment path.
2. **Online learning reaches about 60–80% of the offline gain.** E29, with 2-fold learned gates over all votes, gained +0.014 and +0.019. Online feedback sees only the top suggestions and starts from zero, which is what a deployed memory would face.
3. **What it learns is readable and auditable.** The induced rules are the domain regularities that analogy exploits, stated as rules with precision and evidence counts. The same holds for what it learns to distrust, e.g. inferring a director of photography from the screenwriter.
   - The film rule "country = the country whose official language is the film's language" is found in the *inverse* direction through the official-language relation. This is a two-hop, direction-sensitive pattern that nobody specified.
4. **Scope.** Gains are modest because most remaining errors are retrieval and coverage failures (no analogue proposes the right object), which reliability weighting cannot fix. The next levers are the induced rules themselves:
   - apply reliable rules directly, to candidates no analogue proposes;
   - store them as schema cases, completing the analogy → abstraction loop;
   - use feedback to choose analogues, not just transfers.
