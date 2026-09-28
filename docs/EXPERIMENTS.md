# Experiments, Benchmarks and Decision Gates

This document is the experimental plan for MARS. Every experiment has a question, a protocol, metrics, baselines, a success threshold, and a statement of what we learn if it fails. Thresholds are initial and may be revised, but any revision must be recorded here *with the reason* before the next run.

See [DESIGN.md §14–§15](DESIGN.md#14-hypotheses-and-success-criteria) for the hypotheses (H0–H6) and the phase gates.

---

## 1. The synthetic relational-analogy generator (`mars-gen`)

### 1.1 Goals

- Known ground truth: which cases are analogues, *and* the correct correspondences.
- Adversarial surface similarity: mere-appearance distractors that share vocabulary and entities but not structure.
- Controllable difficulty: case size, depth, distractor facts, perturbations.
- **No circularity**: the templates used to design the encoder are not the ones used to evaluate it.

### 1.2 Components

1. **Canonical predicate vocabulary.** About 100 relations in a taxonomy. Families include causal (`cause`, `enable`, `prevent`, `inhibit`), control and dependency (`controls`, `owns`, `depends-on`, `supplies`), transfer (`flow`, `transfer`), spatial (`contains`, `adjacent`), temporal (`before`, `during`), comparison over functions (`greater (f x) (f y)`), and logical (`and`, `implies`, `not`).
2. **Structural templates.** At least 50 small predicate-calculus DAGs (3–15 relations, depth 1–4), grouped into about 10 **families**:
   - chains (A→B→C…);
   - inhibition motifs (A ─| B → C);
   - feedback loops (positive and negative);
   - common cause / common effect;
   - competition for a resource;
   - control + dependency (the supply-chain / chokepoint pattern);
   - flow-with-gradient (water/heat/current: `cause (greater (pressure a) (pressure b)) (flow a b w p)`);
   - conservation and transfer;
   - containment propagation;
   - symmetric exchange.
3. **Domain lexicons.** About 10 domains (biology, business, mechanics, social, networks, logistics, ecology, law, electronics, cooking). Each has entity names, attribute predicates, and **surface synonyms** for canonical predicates (for example `inhibit` → `represses` in biology, `blocks` in law).
4. **Instantiation.** Sample a template, a domain and entities. Add 0–N **distractor facts** from domain background (attributes, irrelevant relations). Optionally **compose** 2–3 templates into one case sharing entities.

### 1.3 Variant classes per base case (Gentner, Rattermann & Forbus 1993)

| Class | Relational structure | Surface (entities, attributes, domain) | Expected role |
|---|---|---|---|
| **LS** literal similarity | same | same | relevant (easy) |
| **TA** true analogy | same | different domain | **the target** of the analogy profile |
| **MA** mere appearance | different template | same domain, overlapping entities and attributes | **adversarial distractor** |
| **FOR** first-order relations only | shares first-order relations, but higher-order (`cause`) structure differs | different | partial; must rank below TA |
| **RND** random | different | different | irrelevant |

### 1.4 Perturbations (severity-parameterized)

`delete-edge`, `add-distractor`, `reverse-edge`, `swap-args`, `predicate-substitute` (sibling in the taxonomy vs an unrelated predicate), `insert-intermediate` (A→B becomes A→X→B), `split-entity`, `merge-entities`, `synonym-unresolved` (surface synonym left as a distinct, undeclared predicate).

### 1.5 Splits and circularity control

- **Template-family holdout.** Families are split into *dev* (used to design and tune features and weights) and *test* (never seen during design). Headline numbers are reported on test families only.
- **Perturbation holdout.** At least two perturbation types are reserved for test.
- **Domain holdout.** At least two domains appear only in test.
- **Independent authorship** where possible. Test templates written by someone other than the encoder designer, or produced by an automated structural enumerator.

### 1.6 Ground truth

- Case-level: template ID(s) plus variant class, giving graded relevance LS = TA > FOR > MA = RND for the analogy profile.
- Correspondence-level: the generator records the entity and expression correspondence between base and each LS/TA variant (modulo perturbations).
- Sanity: for small cases, exhaustive FAC scores confirm that no accidental isomorphic RND case outranks a designated TA. Any that do are relabelled as TA.

### 1.7 Output format

The `.mars` s-expression files (see [DESIGN §5.1](DESIGN.md#51-case-language)) plus a JSONL manifest with labels. The generator is deterministic given `(seed, config)`.

---

## 2. Metrics

| Area | Metrics |
|---|---|
| Separability | pairwise ROC-AUC (TA vs MA ∪ RND); Hamming-distance histograms by class; effect size (Cohen's d) |
| Retrieval | recall@k (k = 1, 10, 64, 256) for TA; MRR; nDCG@k on graded relevance; **MA-intrusion@k** (fraction of top-k that is MA) |
| Mapping | correspondence precision/recall/F1 vs ground truth; greedy vs exhaustive score gap; candidate-inference precision |
| End-to-end | top-1 structural accuracy vs exhaustive FAC; fraction of corpus mapped; latency p50/p95/p99; throughput (QPS, batched and unbatched) |
| Systems | bytes per case; peak RSS; achieved memory bandwidth vs a STREAM-like probe; build/rebuild time |
| Incremental | work units (features re-encoded, bits flipped, mappings recomputed, TMS nodes touched) and wall time per update; vs corpus size; vs full recompute |
| SDM | critical distance (empirical); partial-cue recall vs cue completeness; prototype purity; interference (old-item recall vs number of later writes) |
| Consolidation | schema purity/completeness vs hidden templates; retrieval gain with schemas; near-miss precision |
| Representation | parse stability (Jaccard over N re-parses) for LLM-parsed data |

All results are emitted as JSON with the config, git SHA, seed and a hardware fingerprint.

---

## 3. Baseline ladder

| ID | Method | Isolates |
|---|---|---|
| B1 | random | floor |
| B2 | BM25 over a natural-language rendering of each case | lexical retrieval |
| B3 | dense text embeddings (off-the-shelf sentence embedding model) over the NL rendering | "just use embeddings" |
| B4 | **MAC content vectors** (functor counts, exact dot product) | the 1995 baseline we claim to modernize |
| B5 | **exact sparse cosine on MARS features** (inverted index) | feature-map quality, without the sketch |
| B6 | WL subtree kernel (exact) | a standard structural graph similarity |
| B7 | GraphHD-style HD graph encoding | a published HDC graph encoding |
| B8 | **MARS fingerprint, exhaustive Hamming** (Mode K) | the sketch |
| B9 | MARS fingerprint + MIH / FAISS binary IVF / HNSW | sublinear indexes |
| B10 | MARS fingerprint + SDM Modes B / A | what SDM adds |
| B11 | **exhaustive FAC** (mapper over the whole corpus, where feasible) | the pipeline upper bound |
| B12 | (optional) LLM re-ranker over top-k text renderings | a modern non-structural comparison, outside the core |

---

## 4. Experiments

### E0: Fingerprint separability and feature ablation (Phase P1, gate G1 → H0)

- **Question:** do the entity-anonymous structural features separate TA from MA and RND before any index exists?
- **Protocol:** 1,000 bases × {LS, TA, MA, FOR, RND ×10} from dev families for iteration, and the same from test families for reporting. Compute pairwise similarity under each channel alone, each channel dropped (leave-one-out), and the analogy profile. Sweep D ∈ {1024, 2048, 4096, 8192, 16384}, WL depth h ∈ {1, 2, 3}, β, and taxonomy α.
- **Also measure:** the distribution of normalized Hamming distance for TA pairs. This decides whether SDM attraction is ever relevant for far analogues (expectation: TA δ ≫ 0.2, so it is not).
- **Compare:** B4, B5 and B6 on the same pairs.
- **Success:** AUC(TA vs MA ∪ RND) ≥ 0.95 on test families; the fingerprint within 0.02 AUC of B5 at D = 8192.
- **If it fails:** (a) B5 also fails → the features or representations are inadequate, so iterate on features and do not build more machinery; (b) B5 succeeds but the fingerprint fails → the sketch is too lossy, so raise D or reallocate segments; (c) all structural methods fail on MA → the generator's MA class may be structurally too close, so audit the generator.

### E1: Retrieval at small scale (P1)

- **Protocol:** corpora of 10³, 10⁴ and 10⁵ cases (mixtures of families, domains and variants). Queries are held-out bases. Report recall@k, MRR, nDCG and MA-intrusion for B2–B8.
- **Success:** trend toward H1 (TA recall@64 ≥ 0.9 at 10⁵; ≥ B4 + 0.15).

### E2: Mapper correctness and cost (P2)

- **Protocol:** (1) hand-encoded gold analogies (solar/atom, water/heat flow, Karla-the-Hawk sets once encoded, and others); (2) generator LS/TA pairs with known correspondences; (3) small cases (≤ 12 relations) against an exhaustive optimal matcher; (4) the same pairs through SMTB (CRE Python interface) for a head-to-head.
- **Metrics:** correspondence F1, candidate-inference precision, greedy gap, time per pair vs case size (10 → 500 facts).
- **Success:** F1 ≥ 0.95 on gold and generator TA; greedy gap ≤ 5%; ≤ 0.2 ms per pair at about 50 facts.

### E3: End-to-end MAC/FAC at scale (P3, gate G3 → H1, H2)

- **Protocol:** corpora of 10, 10², …, 10⁶ (10⁷ if hardware allows). For each size, 1,000 queries from test families. Pipelines: B2/B3/B4/B8/B9 → FAC(top-k) for k ∈ {8, 32, 64, 256}; compare to B11 where feasible (exhaustive FAC is sampled at large N).
- **Metrics:** top-1 structural accuracy relative to exhaustive; fraction mapped; latency p50/p99 (batched and single query); memory.
- **Success (G3):** at 10⁶, ≥ 95% of exhaustive accuracy with ≤ 10⁻⁴ of the corpus mapped; p50 ≤ 100 ms on a 16-core desktop.

### E4: Robustness to structural perturbation (P3)

- **Protocol:** for each perturbation type and severity s ∈ {0, 1, 2, 3, 4}, measure recall@64 and end-to-end accuracy. This plots P(correct analogue | corruption).
- **Readout:** which channels degrade gracefully (expect C1/C2 more robust than deep WL labels) → informs profile weights.

### E5: What does SDM add? (P4, gate G4 → H6)

- **E5a, partial-cue retrieval:** remove 25/50/75% of a query's structure. Compare Mode K directly vs Mode A cleanup → Mode K.
- **E5b, SDM-bucket vs IVF vs MIH vs exhaustive:** random addresses vs k-means addresses, at 10⁶ and 10⁷. Recall vs candidates scored vs latency.
- **E5c, prototype emergence:** write many noisy instances of hidden templates. Measure whether Mode A attractors decode to template-like feature sets (purity) vs SAGE generalizations (E7).
- **E5d, interference and capacity:** stream writes. Track recall of early items vs number of later writes; empirical critical distance vs load.
- **Gate:** keep SDM in the core only if it wins significantly on at least one of E5a, E5c or E5d, or on E5b at 10⁷. Otherwise document it as a negative result and keep Mode B only if it beats IVF.

### E6: Incremental maintenance (P5, gate G5 → H3)

- **Protocol:** build a corpus of N ∈ {10⁴, 10⁵, 10⁶}, with 1,000 standing queries and a population of accepted inferences. Replay an update stream of 10⁵ operations: add/remove fact (70%), add case (20%), remove case (5%), vocabulary epoch change (rare, measured separately).
- **Metrics:** per update, work units and wall time; correctness check against a from-scratch recompute on a sampled snapshot every 10⁴ updates (results must match exactly, modulo tie ordering).
- **Success:** median per-update cost flat across N; ≥ 100× cheaper than recomputing affected results from scratch; zero divergence in the correctness checks.

### E7: Consolidation, near-misses and inference calibration (P6 → H4)

- **E7a:** stream instances of hidden templates (with noise and distractors) through SAGE-style gpools. Measure schema purity and completeness vs the hidden templates.
- **E7b:** few-shot far-analogue retrieval with and without schemas in the index.
- **E7c:** near-miss retrieval: queries of the form "similar to X except for difference Δ" against planted near-miss pairs.
- **E7d:** candidate-inference calibration: bin inferences by the confidence heuristic and measure the fraction true under the generator's ground truth.

### E8: Real-data analogy (P7 → H5)

- **Karla-the-Hawk story sets** (Gentner, Rattermann & Forbus 1993): 20 sets of base + 4 variants. Compare MARS rankings with published human soundness and retrieval patterns (MAC/FAC reproduced them).
- **ARN** (Sourati et al., TACL 2024): about 1.1k query/analogy/distractor triples with near and far partitions. Pipeline: LLM parse → MARS. Metric: accuracy on far analogies vs LLM zero- and few-shot numbers reported in the paper.
- **StoryAnalogy** (EMNLP 2023): a story-level analogy corpus, as a secondary set.
- **I-RAVEN** (optional): a point of contact with Emruli et al. 2013 and NVSA (Hersche et al. 2023). It is not a core target, since visual parsing is out of scope.
- **Parse stability** is reported for every LLM-parsed set.

### E9: Program analogy and mechanism search (P7)

- **Program analogy:** extract dataflow/control-flow relational cases from programs. Ground truth: **Project CodeNet** (many solutions per problem, many languages) and **Rosetta Code** (the same task across languages). Solutions to the same problem are analogues; solutions to different problems with shared idioms are MA-like distractors. Metrics: recall@k and MAP of same-problem retrieval vs code-embedding and AST-kernel baselines.
- **Mechanism search pilot:** encode mechanisms from pathway or systems descriptions (licence permitting) and domain-crossing textbook mechanisms (hydraulic ↔ electrical ↔ thermal). Qualitative case studies plus retrieval of planted cross-domain analogues.

---

## 5. Hardware and reporting protocol

- Reference machine class: a 16-core x86-64 desktop (AVX2, and AVX-512 where available), 64 GiB RAM. An ARM (Apple Silicon) run is reported where kernels exist.
- Each run reports the CPU model, core count, measured memory bandwidth, OS, compiler version, `target-cpu`, git SHA, config hash and seed.
- Latency is reported both single-query and batched. Throughput is reported at saturation.
- Negative results are published in the same format as positive ones.

---

## 6. Experiment dependency graph

```text
E0 ──▶ E1 ──▶ E3 ──▶ E4
         │      │
E2 ──────┘      ├──▶ E5 (SDM gate)
                ├──▶ E6 (incremental gate)
                └──▶ E7 ──▶ E8, E9
```

E0 and E2 can run in parallel. Nothing after E3 is started until G1 passes.

---

## 7. Index of experiments as run

The plan above (E0–E9) was written before the work began. The table lists every experiment actually run; each results README records the protocol, config, seed and reproduction commands.

| Exp | Question | Result |
|---|---|---|
| [E0](../results/E0/README.md) | Do structural fingerprints separate true analogues from look-alikes? | yes (≥ 0.995; content vectors / lexical 0.000) |
| [E2](../results/E2/README.md) | Is the mapper correct and fast? | entity correspondences P ≈ 1.0; 16–38 µs per pair |
| [E3](../results/E3/README.md) | Does MAC → FAC scale to 10⁶? | true analogue in top-64 100%; ≈ exhaustive accuracy at 1/1100 of the cost |
| [E4](../results/E4/README.md) | Robustness to structural perturbation | fused MAC + FAC ≥ either alone |
| [E5](../results/E5/README.md) | Does SDM add value? | no: dropped from the core (k-means buckets kept) |
| [E6](../results/E6/README.md) | Are updates proportional to change? | ~300 µs per update, flat from 10⁴ to 10⁶; exact |
| [E7](../results/E7/README.md) | Do SAGE schemas help? | few-shot inference +41% |
| [E9](../results/E9/README.md) | Program analogy on real code | fused MRR 0.33 vs lexical 0.15 |
| [E10](../results/E10/README.md) | Scoring variants, fusion weight, case size | w = 0.3; cases need ~10 connected facts at 10⁶ |
| [E11](../results/E11/README.md) | Is corroboration a calibrated confidence? | precision 0.07 → 0.86 with support 1 → 5 |
| [E12](../results/E12/README.md) | Cross-language program analogy | R@10 0.37 vs lexical 0.17 (significant); MRR 0.17 vs 0.09 not significant at 52 queries |
| [E13](../results/E13/README.md) | Can analogy learn vocabulary alignment? | synthetic: precision 1.000, retrieval 0.15 → 0.99 |
| [E14](../results/E14/README.md) | … on real code? | frequent pairs only; alignment barely matters there |
| [E15](../results/E15/README.md) | Schemas vs instances as memory grows | schemas complement instances; schema-only degrades |
| [E16](../results/E16/README.md) | Knowing when no analogue exists | local-null z keeps precision 0.90–0.95 from 10³ to 10⁶ |
| [E17](../results/E17/README.md) | Candidate inference on real code | ~10× chance; exact restoration rare (7.6%) |
| [E18](../results/E18/README.md), [E19](../results/E19/README.md) | Representation views (nested vs flat) | granularity trades fingerprint vs mapper strength; views have roles |
| [E20](../results/E20/README.md) | Near-misses | soft emphasis from 1–2 near-misses beats 1-NN |
| [E21](../results/E21/README.md), [E22](../results/E22/README.md) | Schema hierarchies, membership scores | negative (identifiability limit) |
| [E23](../results/E23/README.md) | Natural language (StoryAnalogy) via an LLM front end | MARS resists look-alikes (0.51 vs ≤ 0.18); an LLM judging directly is better (0.79) |
| [E24](../results/E24/README.md) | Retrieval at scale + LLM verifier | MARS shortlists cleanest; RRF with lexical best |
| [E25](../results/E25/README.md) | Better LLM front ends | abstraction-first prompting and ensembles: 0.59–0.61 |
| [E26](../results/E26/README.md) | Vocabulary alignment on real KGs (DBpedia ↔ Wikidata) | with label anchors: 14/16 (films), 0/15 wrong (scientists) |
| [E27](../results/E27/README.md) | KG completion by analogy | analogy > copying; ≈ copying + mined length-1 rules |
| [E28](../results/E28/README.md) | Relational depth | analogy's gain grows with depth; matches mined length-≤2 rules without mining |
| [E29](../results/E29/README.md) | Learning which transfers to trust (offline) | gated analogy ≈ rules + analogy; readable induced rules |
| [E30](../results/E30/README.md) | … online, in the engine, from feedback | +0.008 / +0.015 Hits@1 over a query stream; persisted |
| [E31](../results/E31/README.md) | Applying induced rules; where completion errors are | rules add little; errors are new-value prediction (answer not in the query), not relational inference |
| [E32](../results/E32/README.md) | An identity (entity-overlap) channel in engine retrieval | scientists +0.036 Hits@1 (new values 0.27 → 0.32); films small; fusion beats identity alone |
| [E33](../results/E33/README.md) | MARS as an agent's episodic memory (incident response) | remedy + target 0.995 → 0.65 with noise vs names 0.48 → 0.23; better retriever for an LLM agent (0.66 vs 0.35) and better alone (0.78) |
| [STATS](../results/STATS.md) | Paired 95% CIs and permutation tests for headline comparisons (E9–E33) | E12 MRR and E28's scientists combination not significant; everything else in the table holds |
| [bw](../results/bw/) | Memory bandwidth of Mode K | batched 3.7 ms per query at 10⁶ (kernel v2) |
