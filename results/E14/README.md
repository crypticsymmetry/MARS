# E14: bootstrapped vocabulary alignment on real code

Tables: [E14-raw.md](E14-raw.md) (default) and [E14-cosine.md](E14-cosine.md) (frequency-normalized association).

Reproduce:
```
tools/fetch_e9_corpus.sh
E=data/external
MARS_RAW=1 python3 tools/py2mars.py data/e14 algorithms:$E/algorithms-1.0.1/algorithms pygorithm:$E/pygorithm-1.0.4/pygorithm pyalgs:$E/python_algorithms-0.2.2/python_algorithms
MARS_RAW=1 python3 tools/js2mars.py $E/js/ast.jsonl jsalgs data/e14
mars-bench e14 [--assoc raw|cosine] [--theta 0.2] [--neighbours 10] [--rounds 3]
```

**Question:** E13 learned cross-domain vocabulary alignment without labels on synthetic data. Does the same loop work on real code?

**Setup.**
- The corpus is the E12 corpus (1,308 functions: three Python packages plus one npm package), re-converted in *raw* mode (`MARS_RAW=1`).
- Library and method calls keep language-specific names (`py:len`, `py:append`, `js:length`, `js:push`, `js:Math-max`, …) as unresolved predicates. The hand-written py↔js mapping of E12 is not applied.
- That gives 359 raw predicates (260 Python, 99 JS). 18 py↔js pairs mean the same thing according to the converters' hand map.
- The E13 loop runs unchanged, with domain = language, other-language neighbours, and pairs as clusters.
- Retrieval is scored on E12 task A: 52 queries, each a main function whose algorithm also exists in the other language, ranked among all 1,308 functions.

## Results (default: θ = 0.2, 10 neighbours, raw evidence)

MRR (R@1 / R@10):

| | aligned pairs (correct / same identifier / wrong / unjudged) | FP analogy | FAC | fused ½FAC+½FP-lit | fused 0.3FAC+0.7FP-lit |
|---|---|---|---|---|---|
| no alignment | 0 | 0.117 | 0.139 | 0.152 (0.06/0.38) | 0.117 |
| learned, 3 rounds | 4 / 2 / 0 / 8 | 0.094 | 0.145 | 0.153 (0.04/0.38) | 0.112 |
| **oracle alignment** (hand map applied to the raw corpus) | all 18 gold pairs | 0.103 | 0.145 | 0.155 (0.04/0.38) | 0.111 |
| hand-mapped E12 corpus | — | 0.119 | 0.142 | 0.168 (0.06/0.37) | 0.141 |

**Learned pairs**, by evidence:

| pairs | verdict |
|---|---|
| `py:len`↔`js:length` (38.4), `py:append`↔`js:push` (36.6), `py:pop`↔`js:pop` (3.8), `py:abs`↔`js:Math-abs` (2.1) | correct |
| `py:is_empty`↔`js:isEmpty` (16.0), `py:root`↔`js:root` (1.9) | same identifier (user-defined APIs) |
| `py:put`↔`js:addEdge`, `py:helper`↔`js:inOrder`, `py:recur_search`↔`js:_find` | unjudged; plausible by manual inspection (role-level correspondences between helpers) |
| `py:insert`↔`js:forEach` (14.6), `py:pow`↔`js:getNode` (5.5), `py:defaultdict`↔`js:getNodeHeight`, `py:set`↔`js:Math-random`, `py:is_full`↔`js:_siftUp` | unjudged; wrong by manual inspection |

Sensitivity:
- θ = 0.4 / 5 neighbours (the E13 defaults) gives only 1 correct pair: cross-language mappings rarely reach a normalized score of 0.4.
- 20 neighbours gives the same pairs as 10.
- Frequency-normalized ("cosine") association is slightly worse (3 correct / 1 wrong).

## Findings

1. **Frequent correspondences are learned; the long tail is not.**
   - The four hand-map pairs recovered (length, push, pop, Math-abs) are the most frequent JS library calls. The two "same identifier" pairs are also right.
   - No pair judgeable against the hand map is wrong. By manual inspection, ~9 of the 14 learned pairs are right or plausible (~0.6 precision).
   - Recall is 4/18. The JS side is small (195 functions; `js:push` occurs 24 times, most gold JS calls under 10), so most gold pairs never co-occur in a strong cross-language mapping.
   - E13's clean result relied on every predicate occurring hundreds of times in parallel structure. Real corpora are Zipfian, and the loop inherits that.
2. **Vocabulary is not the bottleneck on this task.**
   - Even the *oracle* alignment moves fused MRR only from 0.152 to 0.155 and FAC from 0.139 to 0.145. The learned alignment gets the same FAC gain (0.145).
   - The reason is the representation: unresolved predicates are already anonymized to (kind, arity) in the structural channels (DESIGN §0). So `py:len` and `js:length` already match structurally, and cross-language algorithm identity is carried by control/data-flow shape rather than library names.
   - The remaining gap to the hand-mapped corpus (0.168) comes from the converters' other canonicalizations (unknown methods collapsed to `m-other`, calls given canonical `bi-*` names that also appear in the surface channel). It does not come from call-name alignment.
3. **Name specificity can hurt analogy retrieval.**
   - Aligning names *lowers* fingerprint-analogy MRR (0.117 → 0.103 oracle, 0.094 learned).
   - Anonymous shapes let *different* library calls of the same arity match (e.g. `py:len(x)` ~ `js:count(x)`), which helps when two authors in two languages implement an algorithm with different calls.
   - With 52 queries, that is about 2 queries losing rank 1, so this is suggestive only. It agrees with E9/E10: coarse relational shape beats specific names for cross-author analogy.
4. **Implication for the framework.** Bootstrapped alignment is a precision tool, not a recall tool, on sparse real data. It is worth running where vocabularies are large, shared structure is dense and the mapper (not the fingerprint) is the bottleneck. Examples are knowledge graphs with parallel schemas, and E13-like settings where unresolved names block FAC entirely. For code retrieval, investing in flow/strategy-level representation (E9/E12 failure analysis) has more headroom than vocabulary alignment.
