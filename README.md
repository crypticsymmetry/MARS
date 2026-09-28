# MARS: Memory-Assisted Relational System

*(working codename; see [naming](docs/DESIGN.md#14-naming))*

MARS is a research project to build a **continuously learning analogical memory** that runs on commodity hardware:

- **Binary hyperdimensional fingerprints** cheaply propose structurally similar memories out of millions.
- An explicit **structure-mapping engine** verifies them and produces correspondences and candidate inferences.
- A **dependency / truth-maintenance layer** means that learning or retracting one fact recomputes only what depends on it.

It is a modern, persistent, large-scale take on the MAC/FAC architecture (Forbus, Gentner & Law 1995), combined with ideas from Sparse Distributed Memory, Vector Symbolic Architectures and incremental computation.

```text
 new situation ─▶ relational case ─▶ structural fingerprint ─▶ top-k candidates (cheap)
                                                                    │
                     provenance-tracked inferences ◀── structure mapping (explicit)
                                   │
                  dependency graph + truth maintenance ─▶ persistent memory ─▶ consolidation
```

## The research question

> Can a cheap binary associative front end keep the right structural analogues in a small candidate set, reliably enough that explicit analogical reasoning (structure mapping with correspondences, constraint checks and candidate inferences) becomes practical over millions of continually changing relational memories on one workstation?

## Status

**Working research prototype**: a Rust workspace of 11 crates, including Python bindings, and 33 experiments reproducible from versioned data. The evidence is summarized below by how well each claim holds, failures included. Details are in each `results/E*/README.md`. Paired 95% confidence intervals and p-values for the headline comparisons: [results/STATS.md](results/STATS.md) (`python3 tools/stats.py`).

### External, pre-registered evaluation of the frozen v0.1

These were planned in [docs/PREREGISTRATION.md](docs/PREREGISTRATION.md) before running: settings chosen on validation only, test run once, predictions scored with criteria committed in advance ([results/EV_PREDICTIONS.md](results/EV_PREDICTIONS.md)).

| evaluation | result | verdict on predictions | evidence |
|---|---|---|---|
| **KG link prediction**, FB15k-237 / WN18RR (standard filtered protocol) | MRR 0.163 / 0.372 vs RotatE 0.338 / 0.476. On FB15k-237 MARS is **below relation popularity** (0.233). On WN18RR, when the answer is within 2 hops, Hits@1 is 0.75; otherwise 0.05 | below the embedding models (P1 ✓); beats popularity on WN18RR only (P2 ✗ on FB15k-237); learned reliability helps (P4 ✓) | [EV1](results/EV1/README.md) |
| **Code retrieval by problem**, CodeNet Python800 (5,982 programs) | MAP@R: MARS 0.211 > lexical 0.129, but a pretrained code embedding scores 0.473; fusing MARS with lexical hurts (0.164) | P6 ✓, P7 ✓ (embedding ≫ MARS), P8 ✗ | [EV2](results/EV2/README.md) |
| **Incident agent** (synthetic, structural) vs stronger baselines | MARS 0.728 vs embedding RAG 0.349 vs name recall 0.200 (5 seeds); an LLM given MARS-retrieved episodes scores 0.676 vs 0.459 (embedding) / 0.372 (names) | P9 ✓ | [EV3](results/EV3/README.md) |

**Bottom line:**
- MARS is not competitive as a general link predictor or a general code retriever: learned embeddings win by wide margins, and on FB15k-237 even a frequency prior wins.
- It is strong where the answer lies in relational structure that its representation exposes and surface similarity misleads: WN18RR's within-2-hops half, and structural agent memory.
- Its main bottleneck on real data is representation.

### Works (strong evidence, mostly on controlled synthetic data)

| claim | result | evidence |
|---|---|---|
| Structural fingerprints separate true analogues from look-alikes | TA-top ≥ 0.995; MAC content vectors and lexical: 0.000 | [E0](results/E0/README.md) |
| Retrieval keeps the true analogue in a small candidate set at scale | 100% in the top-64 at 10⁶ cases | [E3](results/E3/README.md) |
| Cheap retrieval + explicit mapping ≈ mapping everything | same accuracy at ~1/1100 of the cost (12 ms vs 14 s per query at 10⁶) | [E3](results/E3/README.md) |
| The mapper finds correct correspondences | entity correspondences P ≈ 1.0, R ≈ 0.99; greedy = exhaustive optimum on all small cases tested (not guaranteed in general) | [E2](results/E2/README.md) |
| Updates cost ∝ change, not memory size | ~300 µs per update from 10⁴ to 10⁶ cases; identical to recomputation (0 mismatches / 1,150 checks); cost grows with the number of standing queries | [E6](results/E6/README.md) |
| Open-set abstention: MARS can say "no analogue" | local-null significance keeps precision 0.90–0.95 from 10³ to 10⁶ cases with one threshold (raw score thresholds: 0.90 → 0.44) | [E16](results/E16/README.md) |
| Corroboration across analogues is a calibrated confidence | precision 0.07 → 0.86 as support goes 1 → 5 (synthetic); 0.07 → 0.66–0.73 on Wikidata | [E11](results/E11/README.md), [E27](results/E27/README.md) |
| Provenance and retraction | every inference is justified by its mapping and base facts in a JTMS; retracting a fact withdraws exactly its dependants | [E6](results/E6/README.md) |

### Works partially

| claim | result | evidence |
|---|---|---|
| Structure beats surface on real code | cross-author algorithm retrieval MRR 0.33 vs lexical 0.15 (33 queries; paired difference +0.19, 95% CI [+0.04, +0.33]); cross-language R@10 0.37 vs 0.17 (significant), but MRR 0.17 vs 0.09 is not significant at 52 queries. Different *strategies* for the same algorithm are not found: the representation is too syntactic. On CodeNet (EV2) MARS beats lexical search (MAP@R 0.21 vs 0.13) but is far below a pretrained code embedding (0.47) | [E9](results/E9/README.md), [E12](results/E12/README.md), [EV2](results/EV2/README.md) |
| Candidate inference on real data | KGs: projecting through the mapping beats copying from the same analogues, more so with relational depth (E27, E28). It works where the answer is already linked in the query (Hits@1 0.71–0.84, 29–39% of queries); "new value" prediction is recommendation-like (0.10–0.32). Code: ~10× chance, but exact restoration is rare (7.6%) | [E27](results/E27/README.md), [E28](results/E28/README.md), [E31](results/E31/README.md), [E17](results/E17/README.md) |
| Learning which transfers to trust | feedback learns per-transfer-type precision online (persisted): +0.008 / +0.015 Hits@1; the learned types read as rules ("studied where the doctoral advisor worked") | [E29](results/E29/README.md), [E30](results/E30/README.md) |
| Vocabulary alignment by analogy | synthetic: precision 1.000. DBpedia ↔ Wikidata: 14/16 (films) and 15/15 (scientists) property pairs plausible, *given* entity-label anchors. On real code only frequent pairs are learned | [E13](results/E13/README.md), [E26](results/E26/README.md), [E14](results/E14/README.md) |
| Schemas (SAGE) | help as a *complement* to instances (few-shot inference +41%); schema-only memory degrades at scale | [E7](results/E7/README.md), [E15](results/E15/README.md) |
| Natural language via an LLM front end | resists same-topic look-alikes (0.51 vs 0.13–0.18, chance 0.25), but an LLM judging directly is better (0.79). The LLM front end is the bottleneck; abstraction-first prompting and ensembles help (0.59–0.61) | [E23](results/E23/README.md)–[E25](results/E25/README.md) |
| Agent episodic memory | synthetic incident response (task designed to be structural): remedy + target 0.995 / 0.76 at noise 0 / 2 vs recall by names 0.48 / 0.32. Against an embedding-RAG memory: 0.73 vs 0.35 (5 seeds); as an LLM's retriever: 0.68 vs 0.46 (EV3, pre-registered) | [E33](results/E33/README.md), [EV3](results/EV3/README.md) |
| Near-misses | soft per-fact emphasis from 1–2 near-misses beats 1-NN (+0.04–0.09); hard rules fail under noise | [E20](results/E20/README.md) |
| Identity (entity-overlap) channel | helps where instances share entities (Wikidata scientists 0.409 → 0.445), not elsewhere (films; incidents: hurts) | [E32](results/E32/README.md), [E33](results/E33/README.md) |

### Failed or dropped

| idea | outcome | evidence |
|---|---|---|
| Sparse Distributed Memory in the core (random addresses, autoassociative cleanup) | poor retrieval geometry, destructive under load; prototypes ≈ k-NN bundling. Dropped; learned-address buckets kept, and they are IVF | [E5](results/E5/README.md) |
| Local syntactic canonicalization of code | did not make different algorithm strategies match | [E9](results/E9/README.md), [E18](results/E18/README.md) |
| Schema-only memory; schema hierarchies (level-2 SAGE); schema membership scores | degrade at scale / identifiability limit | [E15](results/E15/README.md), [E21](results/E21/README.md), [E22](results/E22/README.md) |
| Fusing representation views at score level | no gain over the best single view | [E19](results/E19/README.md) |
| Applying rules induced from analogy directly | adds ~nothing: analogy already covers them | [E31](results/E31/README.md) |
| Support-conditioned transfer reliability | no gain (fragments evidence); reverted | [E32](results/E32/README.md) |
| Vocabulary alignment without anchors (structure or values alone) on real KGs | learns nothing | [E26](results/E26/README.md) |

### Unknown or not yet tested

- **Independent evaluation, still partial.** E0–E33 were built in the same loop as the method, and some of their settings were chosen on the data they report (e.g. E32's identity weight). The pre-registered EV1–EV3 above address this for three tasks, but they are still run by the same developers, and the SMTB comparison was never run.
- **Representation.** How to produce relational representations at the level where analogies live (algorithmic roles rather than syntax, abstract causal patterns rather than story text). This is the main open research problem.
- **Scale of standing queries** (10⁵+), real incident data, and non-English or multimodal inputs.

The mapper is validated in [E2](results/E2/README.md), and memory-bandwidth measurements are in [results/bw](results/bw/). Progress, decisions and the task board are in [docs/PROGRESS.md](docs/PROGRESS.md).

## Quickstart

```bash
cargo build --release
# find analogues + candidate inferences with provenance
./target/release/mars analogies data/examples/classic.mars --case rutherford-atom
./target/release/mars analogies data/examples/classic.mars --case port-chokepoint
./target/release/mars map data/examples/classic.mars --base water-flow --target heat-flow
# persistent session (snapshot + log in ./store; reopen later with just --store)
./target/release/mars serve --store store data/examples/classic.mars < data/examples/session.txt
# experiments
cargo run --release -p mars-bench -- e0      # fingerprint separability (1 s)
cargo run --release -p mars-bench -- e3 --groups 12500 --ops all --severity 1   # MAC→FAC at 10^5
tools/fetch_e9_corpus.sh && cargo run --release -p mars-bench -- e9           # real code
# knowledge-graph completion by analogy, with online feedback (data versioned in data/kg-*)
python3 tools/kg2mars.py data/kg-scientists /tmp/sci --condition C --hop2
cargo run --release -p mars-bench -- e30 --data /tmp/sci
```

From Python (bindings in [crates/mars-py](crates/mars-py/README.md); `pip install maturin`):

```bash
cd crates/mars-py && maturin build --release -o dist && pip install dist/mars_analogy-*.whl
```

```python
import mars
e = mars.Engine.from_files(["data/examples/classic.mars"])
hits, z = e.query("rutherford-atom", k=3)    # [("solar-system", ...), ...], significance z
m = e.map("solar-system", "rutherford-atom") # entity matches, candidate inferences
sq = e.watch("rutherford-atom", k=3)         # standing query, updated as the memory changes

kg = mars.Engine(open("facts.mars").read(), first_order=True, profile="surface")  # e.g. KG triples
for s in kg.suggest("some-entity", k=10):    # inferences ranked by learned reliability × support
    print(s["text"], s["support"], s["reliability"], s["transfers"])
kg.feedback("some-entity", s["text"], True)  # the memory learns which kinds of transfer to trust
kg.induced_rules()                           # ... and states them as rules, with precision and evidence
```

## What it is good for (so far)

Evidence-backed uses, strongest first:

- **Schema and vocabulary alignment** between independently built sources, with no labels (DBpedia ↔ Wikidata: E26).
- **Knowledge-graph completion and auditing by analogy**, with calibrated confidence and readable induced rules (E27–E30).
- **Structure-based code search** across authors and languages (E9, E12).
- **Open-set retrieval that knows when nothing matches** (E16).
- **An incremental, explainable memory for agents** and case-based reasoning: standing queries, provenance, feedback, abstention (E6, E11, E30, E33). A worked example: `python3 tools/e33_agent_memory.py --demo`.

On raw natural language, MARS needs an LLM front end and is best used as a shortlist and verifier stage (E23–E25).

## Documents

| Document | Contents |
|---|---|
| [docs/DESIGN.md](docs/DESIGN.md) | The full concept and system design: representation, encoding, retrieval modes, structure mapping, truth maintenance, consolidation, interfaces, implementation plan, hypotheses, roadmap, risks |
| [docs/CONCEPT_REVIEW.md](docs/CONCEPT_REVIEW.md) | A critique of the original idea: what was kept, fixed or changed, and why |
| [docs/EXPERIMENTS.md](docs/EXPERIMENTS.md) | The synthetic generator, metrics, baseline ladder, planned experiments E0–E9, decision gates, and an index of every experiment run (E0–E33, plus STATS) |
| [docs/PRIOR_ART.md](docs/PRIOR_ART.md) | Annotated bibliography and novelty map |
| [docs/PROGRESS.md](docs/PROGRESS.md) | Working document: task board, decision log, experiment log |
| [results/](results/) | One directory per experiment: README summary + raw tables/JSON |
| [docs/ORIGINAL_CONCEPT.md](docs/ORIGINAL_CONCEPT.md) | The unedited starting concept, archived for provenance |

## Key design decisions (short version)

1. **Approximate proposes, explicit mapping disposes.** Vectors only choose what to inspect. Every reported analogy passes explicit structure mapping (an SME-class greedy matcher: explicit correspondences, constraint checks, a mapping score and candidate inferences; greedy matched the exhaustive optimum on every small case tested, but optimality is not guaranteed).
2. **Entity-anonymous, multi-channel fingerprints.** A bundled binary fingerprint is a SimHash of a structural feature histogram, so the research content is the *feature map*.
3. **Exact batched Hamming k-NN is the baseline.** SDM has to earn its place (cleanup, prototypes, continual learning, 10⁷+ scale) behind an explicit keep/drop gate.
4. **Build on SME / MAC/FAC / SAGE**, and compare with SMTB (2026), rather than reinvent them.
5. **Provenance for everything; work proportional to change.**
6. **Rust core, no deep-learning dependencies;** LLMs only as optional parsers, explainers and baselines.

## Layout

```text
crates/
  mars-hv       packed hypervectors, seeded RNG, bit-sliced weighted majority
  mars-rel      vocabulary + taxonomy, hash-consed expression DAG, cases, .mars s-expression I/O
  mars-encode   feature channels C0–C4 → segmented SimHash fingerprints, IDF epochs
  mars-index    Mode K (AVX-512 exhaustive top-k), sparse baselines, SDM modes
  mars-map      SME-class structure mapper: kernels, greedy/optimal merge, candidate inferences
  mars-tms      justification-based truth maintenance
  mars-engine   incremental memory, standing queries, TMS inferences, SAGE generalization,
                transfer reliability learned from feedback, identity (entity-overlap) channel
  mars-gen      synthetic analogy generator (Gentner classes, perturbations, ground truth)
  mars-bench    experiment runners (E0–E33, plus STATS)
  mars-cli      `mars` command-line tool
  mars-py       Python bindings (PyO3; `import mars`)
tools/          front ends and data: py2mars.py / js2mars.py (code → cases), llm2mars.py (text → cases via an LLM),
                kg_fetch.py / kg_hop2.py / kg2mars.py / wd_prop_labels.py (DBpedia/Wikidata → cases), analysis scripts
data/examples/  hand-encoded classic analogies
data/kg-*/      versioned DBpedia/Wikidata samples (films, scientists; E26–E30)
data/storyanalogy/  versioned LLM conversions of StoryAnalogy (E23–E25)
docs/ results/  design documents; experiment reports
```
