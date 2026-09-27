# Prior Art and Novelty Map

An annotated bibliography of the traditions MARS draws on. Each entry says what the work did, what we take from it, and where MARS differs.

Entries marked **†** were not checked against the primary source during this pass: the citation details come from memory or secondary sources and should be confirmed before any publication.

---

## 1. Structure-mapping lineage (analogy as relational alignment)

| Work | What it did | What MARS takes / how it differs |
|---|---|---|
| Gentner, D. (1983). *Structure-mapping: A theoretical framework for analogy.* Cognitive Science 7(2). | The theory: analogy aligns relational structure; one-to-one correspondence, parallel connectivity, systematicity. | The matching constraints and the systematicity preference. |
| Falkenhainer, B., Forbus, K. D., & Gentner, D. (1989). *The Structure-Mapping Engine: Algorithm and examples.* Artificial Intelligence 41(1). | SME: MHs, kernels, structural evaluation, gmap merge, candidate inferences. | The mapper architecture (DESIGN §8). The original exhaustive merge is worst-case exponential. |
| Forbus, K. D., & Oblinger, D. (1990). *Making SME greedy and pragmatic.* CogSci. † | Greedy merge. | The default merge strategy. |
| Forbus, K. D., Ferguson, R. W., & Gentner, D. (1994). *Incremental structure-mapping.* CogSci 1994. | I-SME: extends mappings as base/target grow; preserves computed results; `remap` backtracks at kernel level; mappings as first-class entities. | Incremental mapping policy (DESIGN §8.6). |
| Forbus, K. D., Gentner, D., & Law, K. (1995). *MAC/FAC: A model of similarity-based retrieval.* Cognitive Science 19(2). | Cheap content-vector dot products (MAC), then SME on a few candidates (FAC). | **The architectural template.** MAC content vectors are baseline B4 and channel C1. |
| Gentner, D., Rattermann, M. J., & Forbus, K. D. (1993). *The roles of similarity in transfer: Separating retrievability from inferential soundness.* Cognitive Psychology 25. | "Karla the Hawk" story sets: surface similarity drives remindings, structure drives soundness. | The LS/TA/MA/FOR variant classes; the E8 real-data track. |
| Markman, A. B., & Gentner, D. (1993). *Splitting the differences: A structural alignment view of similarity.* J. Memory and Language 32. † | Alignable vs non-alignable differences. | The formal basis of "negative analogies" and near-miss memory (DESIGN §11.3). |
| Forbus, K. D., Ferguson, R. W., Lovett, A., & Gentner, D. (2017). *Extending SME to handle large-scale cognitive modeling.* Cognitive Science 41, 1152–1201. | Five scaling techniques: greedy merging, incremental operation, ubiquitous predicates, structural evaluation of analogical inferences, match filters. | All five are adopted (ubiquitous predicates → IDF epochs; match filters → prefilters and constraints). |
| Gentner, D., & Forbus, K. D. (2025). *The structure-mapping engine: A multidecade interaction between psychology and AI.* Current Directions in Psychological Science 35, 148–155. | A retrospective; SME's role in CogSketch and Companions. | Context and positioning. |
| Weitekamp, D., & MacLellan, C. (2026). *SMTB: Fast Structure-Mapping with Tight Bounds.* arXiv:2609.25508. | 5–15× faster than SME, about 50% better mappings on large nested domains; maximizes relational connectivity without privileging higher-order relations; part of the Cognitive Rule Engine (C++, Python interface). | Reference matcher in E2; bounding ideas for early termination; the systematicity-vs-connectivity question (DESIGN §17). |
| Kuehne, S., Forbus, K., Gentner, D., & Quinn, B. (2000). *SEQL: Category learning as progressive abstraction using structure mapping.* CogSci. † | Incremental analogical generalization. | Consolidation (DESIGN §11). |
| SAGE (Halstead & Forbus 2005 †; McLure, Friedman & Forbus, *Extending analogical generalization with near-misses*, AAAI 2015 †). | Generalization pools, fact probabilities, assimilation threshold, MAC/FAC retrieval; near-misses. | **Adopted directly.** Our delta: scale, an idle-time scheduler, SDM prototypes as a comparison, difference fingerprints. |
| Winston, P. H. (1970). *Learning structural descriptions from examples.* MIT PhD thesis. | Near-miss learning. | Near-miss memory. |
| Forbus, K. D., & Hinrichs, T. (2006). *Companion cognitive systems.* AI Magazine. † | A cognitive architecture using SME/MAC/FAC/SAGE at scale. | The closest *system-level* prior art. Its focus is a cognitive architecture; ours is a commodity-hardware, bit-level, incremental memory engine. |

### Competing analogy models (for context)

- **ACME / ARCS** (Holyoak & Thagard 1989; Thagard et al. 1990): constraint-satisfaction mapping and retrieval (ARCS is the retrieval model).
- **LISA** (Hummel & Holyoak 1997) and **DORA** (Doumas, Hummel & Sandhofer 2008): neurally inspired, role-binding by synchrony; DORA learns relations.
- **Copycat / high-level perception** (Chalmers, French & Hofstadter 1992): the *representation* critique of SME-style systems, a key risk for MARS (DESIGN §16).

---

## 2. Sparse Distributed Memory

| Work | Relevance |
|---|---|
| Kanerva, P. (1988). *Sparse Distributed Memory.* MIT Press. | The model: hard locations, radius activation, counters, critical distance (≈209/1000 bits at 10⁴ items in the classic analysis). |
| Flynn, M. J., Kanerva, P., & Bhadkamkar, N. (1989). *Sparse Distributed Memory: Principles and Operation.* Stanford CSL-TR-89-400. | The Stanford prototype (256-bit addresses, 8,192 hard locations, custom address decoding): the historical hardware bottleneck. |
| Bricken, T., & Pehlevan, C. (2021). *Attention approximates sparse distributed memory.* NeurIPS. | A modern bridge between SDM and transformers. |
| Bricken, T., Davies, X., Singh, D., Krotov, D., & Kreiman, G. (2023). *Sparse distributed memory is a continual learner.* ICLR. | Top-K activation and SDM properties give continual learning without replay → motivates E5d and top-A activation in Mode B. |
| Teeters, J. L., Kleyko, D., Kanerva, P., & Olshausen, B. A. (2023). *On separating long- and short-term memories in hyperdimensional computing.* Frontiers in Neuroscience 16. | Superposition as short-term memory vs SDM as long-term memory; compares SDM variants. Supports keeping bundles small (one case) and storage separate. |
| Kanerva, P. (2025). *Autonomous learning with high-dimensional computing architecture similar to von Neumann's.* arXiv:2503.23608. | Recent architecture proposal from SDM's originator. |

## 3. Vector Symbolic Architectures / Hyperdimensional Computing

| Work | Relevance |
|---|---|
| Kanerva, P. (1996). *Binary spatter-coding of ordered K-tuples.* ICANN. † | Binary Spatter Codes (XOR binding, majority bundling): our algebra. |
| Kanerva, P. (2009). *Hyperdimensional computing.* Cognitive Computation 1(2). | The canonical introduction. |
| Kanerva, P. (2010). *What we mean when we say "What's the dollar of Mexico?"* AAAI Fall Symposium. † | Analogy by holistic mapping vectors in BSC. |
| Plate, T. A. (1995). *Holographic reduced representations.* IEEE TNN. | HRR. |
| Plate, T. A. (1994). *Estimating analogical similarity by dot-products of HRRs.* NIPS 6; Plate (2000). *Analogy retrieval and processing with distributed vector representations.* Expert Systems 17. | **Direct precedent** for vector-based structural similarity estimation for analogy retrieval, mirroring human patterns. MARS differs in its binary sketch, scale, and pairing with exact mapping. |
| Eliasmith, C., & Thagard, P. (2001). *Integrating structure and meaning: A distributed model of analogical mapping.* Cognitive Science 25. † | DRAMA: HRR-based mapping. |
| Gayler, R. W. (2003). *VSAs answer Jackendoff's challenges for cognitive neuroscience.* † ; Gayler & Levy (2009). *A distributed basis for analogical mapping.* † | VSA analogical mapping via replicator dynamics. |
| Rachkovskij, D. A., & Kussul, E. M. (2001). *Binding and normalization of binary sparse distributed representations by context-dependent thinning.* Neural Computation 13. † ; Rachkovskij & Slipchenko (2012). *Similarity-based retrieval with structure-sensitive sparse binary distributed representations.* Computational Intelligence 28. † | **Direct precedent** for structure-sensitive *binary* codes for analogical retrieval. Must be read closely and used as a comparison encoding. |
| Kleyko, D., Rachkovskij, D., Osipov, E., & Rahimi, A. (2022/2023). *A survey on HDC aka VSA, Parts I & II.* ACM Computing Surveys. | Survey. Part II covers analogical retrieval and mapping models. |
| Schlegel, K., Neubert, P., & Protzel, P. (2022). *A comparison of vector symbolic architectures.* Artificial Intelligence Review. | Binding operator trade-offs. |
| Frady, E. P., Kent, S. J., Olshausen, B. A., & Sommer, F. T. (2020). *Resonator networks 1 & 2.* Neural Computation 32(12). | Factorizing bound vectors; possible use for decoding fingerprints (DESIGN §17). |
| Nunes, I., Heddes, M., Givargis, T., Nicolau, A., & Veidenbaum, A. (2022). *GraphHD: Efficient graph classification using hyperdimensional computing.* DATE. | HD graph encoding (PageRank-ranked node vectors, bound edges, bundled graph) → baseline B7. |
| Hersche, M., et al. (2023). *A neuro-vector-symbolic architecture for solving Raven's progressive matrices.* Nature Machine Intelligence. | VSA reasoning at modern scale; optional I-RAVEN contact point. |
| Goldowsky, H., & Sarathy, V. (2024). *Analogical reasoning within a conceptual hyperspace.* arXiv:2411.08684. | HDC + Conceptual Spaces analogy; toy-domain proof of concept. |
| Heddes, M., et al. (2023). *Torchhd.* JMLR (MLOSS). † | A reference HDC library for cross-checking encodings in the Python harness. |

## 4. SDM + VSA for analogy (the closest prior art)

| Work | Relevance |
|---|---|
| Emruli, B., Gayler, R. W., & Sandin, F. (2013). *Analogical mapping and inference with binary spatter codes and sparse distributed memory.* IJCNN 2013, pp. 872–879. | A VSA + SDM network for analogical mapping of compositional structures; non-commutative binding; predicts novel patterns in sequences; explores SDM sparsity. Hypervectors around 10⁴ dimensions. |
| Emruli, B., & Sandin, F. (2014). *Analogical mapping with sparse distributed memory: A simple model that learns to generalize from examples.* Cognitive Computation 6. | Learns mapping vectors from examples in SDM. **Precedent for Mode H** (transition/mapping memory). |

**How MARS differs:** these systems perform analogical mapping *within* the vector space at small scale. MARS uses vectors only to *retrieve*, delegates mapping to an explicit structure mapper, and targets 10⁶+ persistent, changing cases with provenance.

## 5. Similarity search in Hamming space

| Work | Relevance |
|---|---|
| Charikar, M. (2002). *Similarity estimation techniques from rounding algorithms.* STOC. | SimHash: `P[bit differs] = θ/π`. **The analytical lens for MARS fingerprints** (DESIGN §6.2). |
| Norouzi, M., Punjani, A., & Fleet, D. J. (2014). *Fast exact search in Hamming space with multi-index hashing.* IEEE TPAMI. | MIH, a sublinear exact baseline (B9). |
| Johnson, J., Douze, M., & Jégou, H. (2019). *Billion-scale similarity search with GPUs.* IEEE TBD; Douze et al. (2024). *The Faiss library.* | Binary IVF/HNSW baselines; GPU scan reference. |

## 6. Graph similarity baselines

| Work | Relevance |
|---|---|
| Shervashidze, N., et al. (2011). *Weisfeiler–Lehman graph kernels.* JMLR 12. | WL refinement: channel C3 and baseline B6. |
| Bai, Y., et al. (2019). *SimGNN.* WSDM; Li, Y., et al. (2019). *Graph matching networks.* ICML. | Learned graph similarity. Optional deep baselines, never in the core. |

## 7. Incremental computation and truth maintenance

| Work | Relevance |
|---|---|
| Doyle, J. (1979). *A truth maintenance system.* Artificial Intelligence 12. | JTMS: justifications, well-founded support, retraction (DESIGN §10.5). |
| de Kleer, J. (1986). *An assumption-based TMS.* Artificial Intelligence 28. | ATMS: multiple contexts; later option. |
| Forgy, C. L. (1982). *Rete.* Artificial Intelligence 19. | Incremental many-pattern/many-object matching; optional rule layer. |
| Gupta, A., Mumick, I. S., & Subrahmanian, V. S. (1993). *Maintaining views incrementally.* SIGMOD. | DRed (delete and re-derive) for recursive views. |
| McSherry, F., Murray, D. G., Isaacs, R., & Isard, M. (2013). *Differential dataflow.* CIDR; Murray et al. (2013). *Naiad.* SOSP. | Incremental iterative dataflow in Rust; candidate for the rule layer. |
| Acar, U. A. (2005). *Self-adjusting computation.* PhD thesis, CMU; Hammer, M. A., et al. (2014). *Adapton.* PLDI; **Salsa** (rust-analyzer's incremental framework). | Demand-driven memoization with dependency tracking: the model for coarse-grained invalidation. |

## 8. Case-based reasoning and memory consolidation

| Work | Relevance |
|---|---|
| Aamodt, A., & Plaza, E. (1994). *Case-based reasoning: Foundational issues, methodological variations, and system approaches.* AI Communications 7(1). | The retrieve–reuse–revise–retain cycle; MARS is a structural CBR memory. |
| McClelland, J. L., McNaughton, B. L., & O'Reilly, R. C. (1995). *Why there are complementary learning systems in the hippocampus and neocortex.* Psychological Review. | Motivation for offline consolidation ("sleep"), used only as an analogy. |

## 9. Analogy benchmarks and LLM analogy

| Work | Relevance |
|---|---|
| Zhang, N., et al. (2023). *Multimodal analogical reasoning over knowledge graphs.* ICLR. | Introduces the **MARS** dataset and MarKG → the name collision. |
| Sourati, Z., et al. (2024). *ARN: Analogical reasoning on narratives.* TACL. | 1.1k triples; near vs far analogies; LLMs struggle with far analogies zero-shot → E8. |
| Jiayang, C., et al. (2023). *StoryAnalogy.* EMNLP. † | Story-level analogy corpus → E8 secondary. |
| Webb, T., Holyoak, K. J., & Lu, H. (2023). *Emergent analogical reasoning in large language models.* Nature Human Behaviour. | The LLM analogy capability debate. |
| Lewis, M., & Mitchell, M. (2024). *Using counterfactual tasks to evaluate the generality of analogical reasoning in LLMs.* † | Counter-evidence: LLM analogy is brittle under counterfactual variants. |
| Zhang, C., et al. (2019). *RAVEN.* CVPR; Hu, S., et al. (2021). *I-RAVEN.* AAAI. | Optional visual-analogy contact point. |
| Puri, R., et al. (2021). *Project CodeNet.* NeurIPS Datasets & Benchmarks. | Ground truth for program analogy (E9). |

---

## 10. Novelty map (revised)

| Component | Status | Our stance |
|---|---|---|
| Sparse Distributed Memory | old, still studied | use in explicit modes; must beat plain k-NN (H6) |
| HDC/VSA | active | used as a sketching and composition layer |
| SDM + VSA | demonstrated | — |
| VSA / HRR analogical retrieval | demonstrated (Plate; Rachkovskij) | compare against; our fingerprint is entity-anonymous and multi-channel |
| SDM + VSA analogy | demonstrated at small scale (2013–2014) | — |
| SME, greedy merge, I-SME, match filters | mature | adopt |
| Fast structure mapping | new in 2026 (SMTB) | reference and compare |
| Cheap retrieval → SME | MAC/FAC (1995) | the template |
| Analogical generalization, near-misses | SEQL/SAGE, McLure et al. 2015 | adopt |
| Incremental graph computation, TMS | mature elsewhere | adopt at coarse granularity |
| **Entity-anonymous, multi-channel structural sketch, analysed as SimHash, benchmarked against MAC / exact cosine / WL / embeddings under held-out families** | **not found** | contribution 1 |
| **Persistent, incremental MAC/FAC runtime at 10⁶–10⁷ cases on one machine, with standing queries and provenance** | **not found** | contribution 2 |
| **Controlled test of SDM's value-add over Hamming k-NN for analogical memory** | **not found** | contribution 3 |
| **Adversarial structural-retrieval benchmark (generator + held-out families + LS/TA/MA/FOR)** | partially (Karla sets, ARN are small and NL) | contribution 4 |
| **Online consolidation and near-miss memory at millions of cases, with difference fingerprints** | **not found at this scale** | contribution 5 |

---

## 11. Reading list, in priority order

1. MAC/FAC (1995) and SME extensions (2017): the core of what we are modernizing.
2. Emruli et al. (2013) and Emruli & Sandin (2014): the closest SDM+VSA analogy work.
3. Plate (2000) and Rachkovskij & Slipchenko (2012): vector structural-similarity precedents; our encoder must be compared against theirs.
4. SMTB (2026): the state of the art in fast mapping.
5. Teeters et al. (2023) and Bricken et al. (2023): modern SDM behaviour.
6. SAGE with near-misses (McLure et al. 2015): consolidation design.
7. ARN (2024) and Gentner, Rattermann & Forbus (1993): benchmark design.
