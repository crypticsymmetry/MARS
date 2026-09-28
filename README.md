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
                     provenance-tracked inferences ◀── structure mapping (exact)
                                   │
                  dependency graph + truth maintenance ─▶ persistent memory ─▶ consolidation
```

## The research question

> Can a cheap binary associative front end keep the right structural analogues in a small candidate set, reliably enough that exact analogical reasoning becomes practical over millions of continually changing relational memories on one workstation?

## Status

**Working research prototype** (Rust workspace, 11 crates incl. Python bindings; 30 experiments, all reproducible from versioned data). Current results:

| hypothesis | result | evidence |
|---|---|---|
| H0 structural fingerprints separate true analogues from look-alikes | ✅ ≥ 0.995 (MAC content vectors / lexical: 0.000) | [E0](results/E0/README.md) |
| H1 retrieval keeps the true analogue in the top-k | ✅ 100% in top-64 at 10⁶ cases | [E3](results/E3/README.md) |
| H2 cheap MAC + exact FAC matches exhaustive mapping | ✅ same accuracy at ~1/1100 of the cost (12 ms vs 14 s per query at 10⁶) | [E3](results/E3/README.md) |
| H3 updates cost ∝ change, not memory size | ✅ ~300 µs per update from 10⁴ to 10⁶ cases, exact vs recompute, 21,000× cheaper | [E6](results/E6/README.md) |
| H4 generalized schemas help | ✅ as a *complement*: few-shot inference +41% (E7); schemas added alongside instances are best at 10³–10⁴ cases, while schema-only memory degrades at scale (fixed assimilation threshold) | [E7](results/E7/README.md), [E15](results/E15/README.md) |
| H5 structure beats surface on adversarial / real data | ✅ synthetic; partial on real code (MRR 0.33 vs 0.15 lexical) | [E4](results/E4/README.md), [E9](results/E9/README.md) |
| Works on natural language via an LLM front end | ✅ partial: on StoryAnalogy, MARS resists same-topic look-alikes that fool text similarity (0.51 vs 0.13–0.18, chance 0.25) but an LLM judging 4 candidates directly is better (0.79); abstraction-first prompting and ensembles lift MARS to 0.59–0.61; at scale, fusing MARS with pattern embeddings and lexical retrieval nearly doubles recall | [E23](results/E23/README.md), [E24](results/E24/README.md), [E25](results/E25/README.md) |
| Candidate inference works on real data | ✅ on knowledge graphs, partial on code. Wikidata completion (5,400 held-out relations): projecting through the mapping beats copying from the same analogues, and the gain grows with relational depth (+0.03 → +0.08 Hits@1 on scientists); at 2 hops, analogy without rule mining matches mined length-≤2 rules + kNN. On real code, analogues restore deleted statements ~10× above chance, but exact restoration is rare (7.6%) | [E27](results/E27/README.md), [E28](results/E28/README.md), [E17](results/E17/README.md) |
| The memory learns which analogical transfers to trust | ✅ inferences are typed by how they relate to the query; feedback learns each type's precision (online, persisted). +0.008 / +0.015 Hits@1 on a query stream, and the learned types read as rules induced from analogy ("studied where the doctoral advisor worked", "film's country = director's citizenship") | [E29](results/E29/README.md), [E30](results/E30/README.md) |
| MARS knows when no analogue exists (open-set abstention) | ✅ local-null significance keeps precision 0.90–0.95 from 10³ to 10⁶ cases with one fixed threshold (raw scores: 0.90 → 0.44) | [E16](results/E16/README.md) |
| Inference confidence (corroboration across analogues) is calibrated | ✅ precision 0.07 → 0.86 as support goes 1 → 5 | [E11](results/E11/README.md) |
| Analogy can learn its own vocabulary alignment (re-representation) | ✅ synthetic: precision 1.000, retrieval 0.15 → 0.99. Real KGs (DBpedia ↔ Wikidata), two domains: 14 of 16 learned property pairs correct by label on films, and none of 15 wrong on scientists, given entity-label anchors. Partial on real code: frequent py↔js calls learned (len↔length, append↔push), long tail missed; alignment barely matters there because anonymous shapes already match | [E13](results/E13/README.md), [E14](results/E14/README.md), [E26](results/E26/README.md) |
| Near-misses teach what matters | ✅ soft per-fact emphasis from 1–2 near-misses beats nearest-neighbour classification (+0.04–0.09 balanced accuracy); hard must-have rules fail under noise | [E20](results/E20/README.md) |
| H6 SDM adds value over plain k-NN | ❌ dropped from the core (k-means buckets = IVF kept) | [E5](results/E5/README.md) |

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
- **An incremental, explainable memory** for agents or case-based reasoning: standing queries, provenance, feedback (E6, E11, E30).

On raw natural language, MARS needs an LLM front end and is best used as a shortlist and verifier stage (E23–E25).

## Documents

| Document | Contents |
|---|---|
| [docs/DESIGN.md](docs/DESIGN.md) | The full concept and system design: representation, encoding, retrieval modes, structure mapping, truth maintenance, consolidation, interfaces, implementation plan, hypotheses, roadmap, risks |
| [docs/CONCEPT_REVIEW.md](docs/CONCEPT_REVIEW.md) | A critique of the original idea: what was kept, fixed or changed, and why |
| [docs/EXPERIMENTS.md](docs/EXPERIMENTS.md) | The synthetic generator, metrics, baseline ladder, planned experiments E0–E9, decision gates, and an index of every experiment run (E0–E30) |
| [docs/PRIOR_ART.md](docs/PRIOR_ART.md) | Annotated bibliography and novelty map |
| [docs/PROGRESS.md](docs/PROGRESS.md) | Working document: task board, decision log, experiment log |
| [results/](results/) | One directory per experiment: README summary + raw tables/JSON |
| [docs/ORIGINAL_CONCEPT.md](docs/ORIGINAL_CONCEPT.md) | The unedited starting concept, archived for provenance |

## Key design decisions (short version)

1. **Approximate proposes, exact disposes.** Vectors only choose what to inspect. Every reported analogy passes explicit structure mapping.
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
                transfer reliability learned from feedback
  mars-gen      synthetic analogy generator (Gentner classes, perturbations, ground truth)
  mars-bench    experiment runners (E0–E30)
  mars-cli      `mars` command-line tool
  mars-py       Python bindings (PyO3; `import mars`)
tools/          front ends and data: py2mars.py / js2mars.py (code → cases), llm2mars.py (text → cases via an LLM),
                kg_fetch.py / kg_hop2.py / kg2mars.py / wd_prop_labels.py (DBpedia/Wikidata → cases), analysis scripts
data/examples/  hand-encoded classic analogies
data/kg-*/      versioned DBpedia/Wikidata samples (films, scientists; E26–E30)
data/storyanalogy/  versioned LLM conversions of StoryAnalogy (E23–E25)
docs/ results/  design documents; experiment reports
```
