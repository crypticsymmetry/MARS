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

**Concept and design stage.** No code yet. The first build step is the cheapest experiment that could falsify the premise: fingerprint separability of true analogues vs mere-appearance distractors (experiment **E0**).

## Documents

| Document | Contents |
|---|---|
| [docs/DESIGN.md](docs/DESIGN.md) | The full concept and system design: representation, encoding, retrieval modes, structure mapping, truth maintenance, consolidation, interfaces, implementation plan, hypotheses, roadmap, risks |
| [docs/CONCEPT_REVIEW.md](docs/CONCEPT_REVIEW.md) | A critique of the original idea: what was kept, fixed or changed, and why |
| [docs/EXPERIMENTS.md](docs/EXPERIMENTS.md) | The synthetic generator, metrics, baseline ladder, experiments E0–E9, decision gates |
| [docs/PRIOR_ART.md](docs/PRIOR_ART.md) | Annotated bibliography and novelty map |
| [docs/ORIGINAL_CONCEPT.md](docs/ORIGINAL_CONCEPT.md) | The unedited starting concept, archived for provenance |

## Key design decisions (short version)

1. **Approximate proposes, exact disposes.** Vectors only choose what to inspect. Every reported analogy passes explicit structure mapping.
2. **Entity-anonymous, multi-channel fingerprints.** A bundled binary fingerprint is a SimHash of a structural feature histogram, so the research content is the *feature map*.
3. **Exact batched Hamming k-NN is the baseline.** SDM has to earn its place (cleanup, prototypes, continual learning, 10⁷+ scale) behind an explicit keep/drop gate.
4. **Build on SME / MAC/FAC / SAGE**, and compare with SMTB (2026), rather than reinvent them.
5. **Provenance for everything; work proportional to change.**
6. **Rust core, no deep-learning dependencies;** LLMs only as optional parsers, explainers and baselines.

## Planned layout

```text
crates/   mars-hv  mars-rel  mars-encode  mars-index  mars-map  mars-tms  mars-engine  mars-gen  mars-bench  mars-cli  mars-py
eval/     Python evaluation harness and baselines
docs/     design documents
```
