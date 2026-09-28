# E35 (exploratory): soft relation similarity learned from role contexts

Upgrade 2 of the plan in [docs/PROGRESS.md](../../docs/PROGRESS.md): let the mapper match *different* relations with a graded score, instead of only identical predicates and taxonomy siblings. Exploratory only: development data and synthetic benchmarks, no test set, no pre-registration, and no positive claim.

**Method** (`crates/mars-map/src/relsim.rs`): similarity is learned from the memory itself, without labels, by the distributional hypothesis applied to relational roles.
- Each predicate is described by its role contexts:
  - which other predicates share its arguments, and at which positions;
  - what is nested inside it;
  - what wraps it.
- Contexts are weighted by PPMI and compared by cosine.
- Two ways to use the similarity:
  1. **Soft matching** (`MapConfig::soft`): each predicate's top-k neighbours become extra match hypotheses, with local score = scale × similarity.
  2. **Learned taxonomy** (`learn_taxonomy`): relations with no canonical ancestor are clustered by average linkage, and each cluster gets a canonical parent `~rel-N`. Fingerprint channels and the mapper resolve predicates to that ancestor, so learned synonyms match as one relation everywhere. This is iterated, with contexts restricted to already-resolved predicates.

A unit test checks the intended behaviour: `pulls`, used exactly like `attracts`, becomes its nearest neighbour, and soft matching then aligns the two cases.

## Results

**Code (CodeNet development sample, E34's final py2pdg front end; MARS fused MAP@R; baseline without soft matching 0.361):**

| soft-k | scale | MAP@R |
|---|---|---|
| 3 | 0.3 | 0.359 |
| 3 | 0.5 | 0.360 |
| 3 | 0.8 | 0.360 |
| 5 | 0.5 | 0.360 |

No effect. The learned neighbours are sensible (strip ↔ rstrip, is ↔ isnot, heappush ↔ heappop, the order comparisons, floordiv ↔ mod, output ↔ return). But py2pdg's hand-made predicate groups already let those pairs match through ascension. [E35-code-dev.json](E35-code-dev.json).

**Unresolved synonyms (E0, 300 groups, 2 distractors, seed 1).** Each domain has its own unlinked names for the same relations; this is E0's hardest condition, which its finding 6 said needs "predicate alignment". Fingerprint (analogy profile + IDF, D = 8192) TA-top:

| learned taxonomy | clusters | cluster purity | TA-top |
|---|---|---|---|
| off (relations anonymized to kind and arity) | — | — | 0.547 |
| min similarity 0.3 | 1 (all 648 relations) | 0.02 | 0.547 (same as off) |
| min similarity 0.5 | 35 | 0.13 | 0.133 |
| min similarity 0.7 | 180 | 0.41 | 0.077 |

Diagnostic: ranked by learned similarity among the 648 unresolved relations, a relation's true synonyms reach only MRR **0.18**.

## Findings

1. **Role-context similarity does not recover synonyms here.** The generator draws each template's predicates at random. A predicate's role context therefore says little about *which* predicate it is, only what category and arity it has. The signal is weak (synonym MRR 0.18).
2. **Clustering on a weak signal is worse than anonymizing.** Impure clusters give unrelated relations a shared identity and split true synonyms across clusters. TA-top drops from 0.55 to 0.08–0.13. Partial re-representation is harmful unless it is precise.
3. **Where the vocabulary is designed, soft similarity adds nothing.** On code, the predicate groups already provide graded matches.
4. **Consistent with E26.** There, vocabulary alignment worked with label anchors (14/16), and without them it failed (0/15). Unsupervised relation alignment from distribution alone is not enough for MARS's settings. The promising routes are anchored ones:
   - alignment induced from high-confidence mappings (a correct mapping aligns its predicates);
   - external text or embedding similarity of predicate names.

   Both would need their own experiment.

The code stays in the tree, **off by default** (`MapConfig::soft = None`; E0 `--learn-taxonomy 0`), for such follow-ups.

Reproduce:
```
mars-bench ev2 --data W_PDG_DEV --soft-k 3 --soft-scale 0.5 [--soft-dump 1] ; python3 tools/e34_eval.py evaluate W_MARS_DEV W_PDG_DEV OUT --mars-only
mars-bench e0 --groups 300 --naming unresolved --distractors 2 --learn-taxonomy 0.5 --lt-diag 1 --out OUT --tag lt0.5
```
