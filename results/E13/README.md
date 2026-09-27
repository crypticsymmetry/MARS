# E13: analogical bootstrapping of vocabulary alignment

Full table for the default run: [E13.md](E13.md). Reproduce with `mars-bench e13 --groups 1000 --rounds 4 [--distractors k] [--reestimate yes|no]`.

**Problem:** E0's hardest setting is the *unresolved vocabulary*. Each of 12 domains names its first-order relations differently (`bio:inhibits`, `law:inhibits`, …) with no taxonomy linking them. Only the canonical higher-order relations (`cause`, `enable`, `prevent`, `implies`) are shared, as a minimal upper ontology. Without alignment, structure mapping cannot match any first-order fact across domains:
- the mapper's TA-top is **0.000**;
- the fingerprint falls back to anonymous shapes, at 0.55;
- the fused score reaches 0.15.

**Method** (no labels; ground truth only *scores* the alignment). Each round:
1. Retrieve 5 cross-domain nearest neighbours per case with the current fingerprints.
2. Map each pair with **wildcard matching** (`MapConfig::wildcard`): relations of equal arity may correspond at a low local score (0.3) when their arguments align, anchored by the shared higher-order structure.
3. From mappings with normalized score ≥ 0.4, accumulate evidence for predicate correspondences (p ↔ q).
4. Keep *mutual-best* pairs per domain pair, and union them greedily by evidence, **never placing two predicates of one domain in a cluster**.
5. **Re-estimate** the whole alignment each round from the latest evidence (EM-style), materialize clusters as new canonical predicates in the vocabulary, re-encode, and re-evaluate.

## Results

| setting | round | clusters (true: 54) | alignment pair precision | coverage | TA-top fingerprint | TA-top FAC | **TA-top fused** |
|---|---|---|---|---|---|---|---|
| 1,000 groups, 2 distractors | before | — | — | 0 | 0.554 | 0.000 | 0.151 |
| | after 4 rounds | 55 | **1.000** | 1.000 | 0.990 | 0.992 | **0.993** |
| | oracle (canonical names) | 54 | 1 | 1 | 0.996 | 1.000 | 1.000 |
| 500 groups | after 4 rounds | 57 | 0.995 | 0.998 | 0.980 | 0.966 | 0.978 |
| 250 groups | after 4 rounds | 59 | 0.953 | 0.978 | 0.968 | 0.950 | 0.972 |
| 1,000 groups, 5 distractors | after 4 rounds | 56 | 0.996 | 1.000 | 0.986 | 0.979 | 0.989 |
| 1,000 groups, 10 distractors | after 4 rounds | 60 | 0.977 | 0.998 | 0.945 | 0.897 | 0.938 |

## Ablations (1,000 groups unless noted)

| variant | outcome |
|---|---|
| no one-predicate-per-domain constraint | mutual-best pairs chain transitively: **all 648 predicates collapse into one cluster** (pair precision 0.017). FAC still rises to 0.92, because it degenerates into pure shape matching, but the vocabulary is meaningless |
| no re-estimation (merges permanent) | first-round errors entrench. Raw evidence precision is only 0.52–0.76 in round 0 → final alignment precision 0.70 (250 groups) / 0.81 (10 distractors); fused 0.71 / 0.73 |
| with re-estimation | evidence precision rises to 0.99 after one round; final alignment ≥ 0.95 precision in every setting |

## Findings

1. **Analogy can learn its own representation.** Starting from mutually unintelligible domain vocabularies, the system recovers the hidden cross-domain predicate correspondences *from its own analogies*: 1.000 pair precision at 1,000 groups. Retrieval accuracy goes from 0.15 to 0.99, which is oracle level. This directly addresses the representation problem (Chalmers, French & Hofstadter 1992; E0 finding 6; E9): re-representation need not be hand-engineered when enough structure is shared.
2. **Two ingredients are essential:**
   - **A structural prior on the alignment** (one name per concept per domain). Without it, transitive chaining destroys the vocabulary.
   - **Iterative re-estimation.** Better alignment → better retrieval → cleaner evidence → better alignment. This is a self-reinforcing loop that converges in ~2 rounds.
3. **Anchors matter.** The shared higher-order relations act as the "Rosetta stone": they fix which first-order facts must correspond, and wildcard matching then reads off the predicate correspondence. How few anchors suffice is an open question. So is whether the method works when anchors are also unresolved; that would need joint alignment of higher-order relations.
4. **Cost** is small: ~1.3 s per round for 8,000 cases (40K wildcard mappings on 4 cores).

## Next

Apply the same loop to real data. Candidates: code (align identifier-level library calls across languages, e.g. `push` ↔ `append`, which E12 hand-coded), and knowledge graphs with independently developed schemas.
