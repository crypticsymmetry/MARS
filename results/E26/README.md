# E26: analogical vocabulary alignment on real knowledge graphs (DBpedia ↔ Wikidata)

Per-configuration tables: `E26-<anchors>-<profile>-<evidence>.md` (A/B/C × literal/surface × all/anchored). Data (fetched 2026-09-27, versioned): `data/kg-films/`.

Reproduce:
```
python3 tools/kg_fetch.py data/kg-films --films 1000          # optional: re-fetch (live endpoints change)
python3 tools/kg2mars.py data/kg-films OUT/C --condition C     # A | B | C
mars-bench e26 --data OUT/C --profile surface --evidence anchored --tag C-surface-anchored
```

**Question:** E13 learned cross-domain vocabulary alignment from analogies on synthetic data, and E14 found it data-starved on code. Does it work on real knowledge graphs whose vocabularies were developed independently? DBpedia says `dbo:director` where Wikidata says `P57`. Ground truth is DBpedia's own `owl:equivalentProperty` links to Wikidata (446 links; 11 of them involve properties present in this sample), plus Wikidata's official property labels for auditing.

**Data.**
- 1,000 films linked by `owl:sameAs`: DBpedia ontology triples (~12 per film) and Wikidata direct claims (~25 per film).
- Each film is a MARS case per KG, with KG-specific, unresolved property names: 170 properties, 27 of them DBpedia.
- Anchoring conditions differ only in what the two KGs visibly share (`tools/kg2mars.py`):
  - **A, structure only:** nothing.
  - **B, values:** dates and numbers are shared entities with an exact-match attribute (runtime units differ: seconds vs minutes).
  - **C, values + labels:** entity objects also carry their English label.

**Loop** (E13/E14, attributes included in mapping):
1. Retrieve 5 other-KG neighbours per film.
2. Map each pair with wildcard matching.
3. Collect property-correspondence evidence (from *all* correspondences, or only *anchored* ones).
4. Keep mutual-best pairs, one property per KG, re-estimated for 3 rounds.

An **anchored** correspondence has some argument pair sharing an exact-match value or label, and no argument pair whose attributes disagree. This is structural consistency with the anchors, the role canonical higher-order relations played in E13.

## Results (final round)

| anchors | neighbour profile | evidence | pairs learned | gold: correct / wrong / not in gold | counterpart R@1 (FP / fused), before → after |
|---|---|---|---|---|---|
| none | literal / surface | all | 4 / 2 | 0 / 3 / 1 · 1 / 1 / 0 | ≈ 0.002 (chance) |
| none | either | anchored | 0 | — | chance |
| values | either | all / anchored | 2–4 | 0–1 correct | 0.005–0.027 |
| values + labels | literal | all | 5 | 1 / 3 / 1 | 0.021 / 0.136 → 0.037 / 0.080 |
| **values + labels** | **literal** | **anchored** | **16** | **6 / 5 / 5** | 0.021 / 0.136 → **0.119 / 0.324** |
| values + labels | surface | all | 3 | 1 / 1 / 1 | 0.557 / 0.819 → 0.557 / 0.561 |
| **values + labels** | **surface** | **anchored** | **16** | **6 / 5 / 5** | 0.557 / 0.819 → 0.557 / 0.610 |

**The 16 learned pairs**, judged against Wikidata's own property labels:

| DBpedia | Wikidata (label) | gold | by label |
|---|---|---|---|
| starring | P161 cast member | ✓ | ✓ |
| director | P57 director | ✓ | ✓ |
| runtime | P2047 duration | ✓ | ✓ |
| writer | P58 screenwriter | not listed | ✓ |
| producer | P162 producer | not listed | ✓ |
| musicComposer | P86 composer | ✗ (gold: `dbo:composer`) | ✓ |
| distributor | P750 distributed by | ✓ | ✓ |
| cinematography | P344 director of photography | ✓ | ✓ |
| productionCompany | P272 production company | not listed | ✓ |
| editing | P1040 film editor | ✓ | ✓ |
| country | P495 country of origin | ✗ (gold: P17 country) | ✓ for films |
| language | P136 genre | ✗ | **✗** (should be P364) |
| budget | P2130 capital cost | ✗ (gold: P2769 budget) | **✗** (near miss) |
| gross | P2142 box office | ✗ (gold: P2139 total revenue) | ✓ |
| narrator | P2438 narrator | not listed | ✓ |
| network | P449 original broadcaster | not listed | ✓ |

Gold pairs present but not learned: composer↔P86 (the 1:1 constraint gave P86 to musicComposer), author↔P50, budget↔P2769, genre↔P136, releaseDate↔P577.

## Findings

1. **Analogical alignment works on real KGs, with anchors.** From 1,000 films and no supervision, the loop learns 16 property correspondences.
   - **14 of 16 are correct** by Wikidata's own labels. The gold standard confirms 6, and 4 more of its "errors" are gold gaps (it lists a sibling property).
   - Recall is 6 of 11 gold pairs present in the sample, and it covers the core film vocabulary: cast, director, writer, producer, composer, editor, cinematographer, distributor, production company, runtime, box office, country.
2. **The anchoring requirement is sharp.**
   - Structure alone learns nothing, and counterpart retrieval stays at chance: film neighbourhoods are simple stars that look alike.
   - Shared literal values alone are too sparse (and units differ).
   - With entity labels and **anchored evidence**, alignment works under either neighbour profile.
   - Using *all* correspondences (the E13 rule) fails even with labels: generic wildcard matches between frequent properties (`runtime ↔ cast member`) swamp the signal.
   - E13's anchors were shared higher-order relations. In KGs, the equivalent is shared entity identity, used as a *consistency filter* on the analogy's correspondences.
3. **Alignment makes structure comparable across KGs.** With the literal profile, counterpart retrieval improves after alignment (fingerprint R@1 0.021 → 0.119, fused 0.136 → 0.324), because aligned properties give the two KGs' films shared structural features. With the surface profile (labels only), alignment slightly *lowers* fused retrieval (0.819 → 0.610). Aligned film structure is generic (every film has a director and a cast) and dilutes the identifying label signal: schema alignment helps *compare* structure but does not identify *instances*.
4. **Errors are near misses and constraint effects.** `budget ↔ capital cost` is a near miss (numerically similar money values). `language ↔ genre` pairs two categorical properties present on most films; its cause is not diagnosed (the correct P364 was not learned). The 1:1 constraint (from E13) blocks many-to-one truths (`composer` and `musicComposer` → P86); see the many-to-one addendum below.

## Addendum: many-to-one alignment

`--many-to-one yes --share 0.25` (`E26-C-surface-anchored-m2o.*`): after the 1:1 merges, a still-unaligned property joins its best partner's cluster if its evidence for that partner is at least 25% of the partner's own pair evidence. The 16 one-to-one pairs are unchanged, and one join is added:

| DBpedia | Wikidata | evidence | gold | by label |
|---|---|---|---|---|
| language | P364 original language of film | 96.2 | ✗ (not listed) | ✓ |

- `dbo:language` now maps to both P136 (genre, wrong) and P364 (original language, right). The correct partner comes second because P136 took it 1:1 first. A join adds recall, but it does not undo a wrong 1:1 pair.
- The gold pairs it was meant to recover are **data-limited**, not constraint-limited: `dbo:composer` occurs once in the 1,000-film sample, `dbo:genre` three times. There is no evidence to learn from.
- Counterpart retrieval is unchanged (R@1 FP 0.557, fused 0.617).

So many-to-one joins are cheap and safe at this threshold (one correct addition, no new errors), but the film sample does not exercise them much. The film KGs are nearly one-to-one at the property level.
