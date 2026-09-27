# Original Concept (archived input)

> **Archived for provenance.** This is the concept exactly as it was brought to the project. The only edits are that inline citation markers from the source chat were removed, and math delimiters were converted to GitHub syntax.
>
> It contains known errors and superseded ideas. **[CONCEPT_REVIEW.md](CONCEPT_REVIEW.md) is the authoritative critique, and [DESIGN.md](DESIGN.md) is the current design.** Section numbers (§n) cited in the review refer to the sections below.

---

Yes. After digging into the literature, the combined idea survives—but in a more interesting form than my initial sketch.

The key discovery is that several pieces of it **were independently or partially built decades ago**, and one particularly relevant 2013 paper even combined binary vector-symbolic representations with Sparse Distributed Memory for analogical reasoning. However, I did **not** find evidence of the particular system I think is worth building now:

> **A large-scale, continuously learning associative reasoning engine that uses binary hyperdimensional memory for cheap retrieval, explicit structure mapping for exact relational reasoning, and incremental dependency maintenance so that learning one fact does not require recomputing the whole system.**

That is the version I would pursue.

I'll call the hypothetical system **MARS** here—*Memory-Assisted Relational System*—although the name would need changing because "MARS" has been used elsewhere in analogy research.

---

# 1. First: what prior art kills the naive novelty claim?

Quite a lot, fortunately. That's useful because we can build on it rather than reinvent everything.

## Sparse Distributed Memory is exactly the kind of old idea we thought it was

Kanerva's SDM stores information across many "hard locations" in a huge binary address space. Reads and writes activate locations within some Hamming-distance neighborhood; stored values are accumulated using counters.

The historical implementation problem was very concrete: every lookup can require computing Hamming distance against many hard-location addresses. An early SDM patent explicitly calls this out, saying that computing Hamming distance for every hard location required special hardware to avoid becoming excessively slow. The Stanford prototype described there used 256-bit addresses, 8,192 hard locations, and custom address-decoding hardware.

This is exactly the type of historical bottleneck that is much less frightening now.

And the architecture has nice theoretical characteristics: distributed storage, approximate/content-addressed recall and iterative recovery from noisy cues. Later work discussing Kanerva's 1,000-dimensional examples illustrates the characteristic "basin of attraction": sufficiently close queries converge toward a stored pattern, while sufficiently distant ones do not.

---

# 2. But SDM + hyperdimensional computing already has a modern revival

This changes our framing considerably.

Vector Symbolic Architectures, or Hyperdimensional Computing, are now an active field. Modern surveys cover Binary Spatter Codes, Holographic Reduced Representations, sparse binary representations and related systems, and explicitly describe their ability to represent compositional structures using high-dimensional vectors.

Even more directly, a 2023 study compared ordinary VSA superposition memory with **Sparse Distributed Memory used as long-term associative memory**. The motivation was essentially the same distinction we're interested in: a hypervector can act like limited working memory while an associative memory provides much larger long-term capacity.

So:

**"Let's combine SDM and VSA" is not novel.**

And neither is:

**"Let's use them for analogy."**

---

# 3. Someone actually tried SDM + VSA analogical reasoning in 2013

This was the most interesting find.

Emruli, Gayler and Sandin published:

**"Analogical mapping and inference with binary spatter codes and sparse distributed memory"**

in IJCNN 2013. Their system combined a Vector Symbolic Architecture with SDM and tested analogical prediction using Raven's Progressive Matrices. They reported useful behavior around hypervectors of roughly $10^4$ dimensions and specifically explored how sparse SDM activation affected analogical mapping.

That's remarkably close to part of our idea.

The encouraging part is that this line does not appear to have subsequently exploded into a huge research program. The paper exists, related work exists, but compared with transformers or ordinary neural retrieval systems it remains a relatively obscure branch.

So instead of discovering untouched territory, we've found something potentially better:

> **a trail that somebody explored far enough to establish plausibility, but apparently did not follow to its modern computational conclusion.**

That is often an excellent research target.

---

# 4. And an even older architecture already anticipated our two-stage retrieval scheme

Here's the really important historical connection.

The 1995 **MAC/FAC** architecture for analogical retrieval works in two stages:

1. **MAC:** cheaply retrieve a small set of plausible candidates using vector-like content summaries.
2. **FAC:** run the expensive Structure-Mapping Engine on only those candidates.

In other words:

```text
              enormous memory
                    │
                    ▼
             CHEAP RETRIEVAL
             approximate match
                    │
              maybe 10 cases
                    │
                    ▼
             EXPENSIVE MATCH
           relational structure
                    │
                    ▼
                 analogy
```

That is almost exactly the architecture I independently proposed.

MAC/FAC used content vectors whose dot products approximately predicted structural compatibility, and then SME performed the real structural comparison.

This is an important clue.

The idea wasn't abandoned because it was silly.

Researchers already discovered that **approximate vector retrieval followed by expensive symbolic comparison is a good decomposition**.

Today's embedding retrieval + reranking pipelines have, in a different form, rediscovered the same architectural principle.

---

# 5. Structure Mapping itself is much more capable than I initially implied

The original Structure-Mapping Engine dates to 1989. Its designers reported that much of its processing is polynomial, typically around $O(N^2)$, rather than requiring unconstrained brute-force graph isomorphism.

More importantly, later SME work improved scalability.

A 2016 paper describes extensions including:

- greedy construction of mappings;
- approximately $O(n^2\log n)$ processing for that stage;
- incremental extension of existing mappings;
- filtering constraints;
- improved handling of large representations.

And incremental SME isn't new either: an **I-SME** system was explicitly described in 1994, allowing analogical mappings to be extended when new information arrives.

So again, the individual ingredients exist.

What appears much less explored is treating all of them as a **modern persistent runtime architecture**.

---

# 6. That suggests a much better research hypothesis

I would formulate it as:

> **Can we build a commodity-hardware cognitive database in which approximate high-dimensional associative memory continuously proposes structurally relevant memories, while an incremental explicit relational engine verifies analogies and maintains consequences as the knowledge base changes?**

Notice that this is **not**:

> Can hypervectors reason?

And it isn't:

> Can graph matching perform analogy?

And it isn't:

> Can SDM retrieve patterns?

Those are largely answered.

The interesting question is whether the combination yields an unusual practical machine.

---

# 7. The architecture I would actually build

I'd give every stored "experience" **two simultaneous representations**.

One is exact:

```text
RELATIONAL GRAPH
────────────────────────────────

object: sun
object: earth

mass(sun, large)
mass(earth, small)
attracts(sun, earth)
orbits(earth, sun)

causes(
    attracts(sun, earth),
    orbits(earth, sun)
)
```

The other is approximate:

```text
HYPERVECTOR
──────────────────────────────

101001101001...
~8,192–16,384 bits
```

These representations serve completely different purposes.

The hypervector answers:

> "What memories should I inspect?"

The graph answers:

> "Are these situations actually structurally analogous?"

That distinction is crucial.

---

# 8. Encoding relational structure into binary hypervectors

Suppose we assign random hypervectors to symbols:

```text
SUN       = random 8192-bit vector
EARTH     = random 8192-bit vector
ATTRACTS  = random 8192-bit vector

ARG1      = random vector
ARG2      = random vector
PRED      = random vector
```

Binary XOR gives us an inexpensive binding primitive.

We could encode:

```text
attracts(sun, earth)
```

approximately as:

$$
R =
(PRED \oplus ATTRACTS)
+
(ARG1 \oplus SUN)
+
(ARG2 \oplus EARTH)
$$

where `+` denotes a bundling/majority operation rather than ordinary integer addition.

Order matters because roles matter:

```text
attracts(sun, earth)
```

must not resemble:

```text
attracts(earth, sun)
```

too strongly.

VSA research has multiple binding approaches and explicitly considers consequences of different binding operators, including analogical examples.

A more hardware-friendly binary scheme could use permutation:

```text
relation =
    PRED ⊕ ATTRACTS
    +
    permute(SUN,  1)
    +
    permute(EARTH, 2)
```

or role-specific random masks.

---

# 9. Higher-order relations are where it gets interesting

Now encode:

```text
CAUSES(
    ATTRACTS(sun,earth),
    ORBITS(earth,sun)
)
```

The hypervector can recursively bind representations:

```text
cause_hv =
    CAUSES
    ⊕ permute(attract_hv, 1)
    ⊕ permute(orbit_hv, 2)
```

This gives us a crude **structural fingerprint**.

Importantly, we do **not** trust that fingerprint as proof of analogy.

It is a lossy hash of structure.

That's exactly what we want.

---

# 10. Store millions of those fingerprints in associative memory

Each episode/case would be represented as something like:

```rust
struct Case {
    id: CaseId,
    graph: RelationalGraph,
    accumulator: HyperAccumulator,
    fingerprint: HyperVector,
}
```

The fingerprint enters long-term associative memory.

Query:

```text
new problem
     │
     ▼
construct relational graph
     │
     ▼
hypervector fingerprint
     │
     ▼
Sparse Distributed Memory
     │
     ▼
candidate memories
```

Conceptually:

```rust
let candidates =
    memory.recall(query_fingerprint, 64);
```

Return perhaps:

```text
64 possible analogues
```

out of:

```text
1,000,000 stored cases
```

Then structure-map only those 64.

---

# 11. Modern SDM does not have to slavishly reproduce Kanerva's implementation

This is an important point.

The classical scheme asks whether every hard location falls inside a fixed Hamming-radius sphere.

I'd test at least three implementations.

### Classical radius SDM

```text
for every hard_location:
    d = popcount(query XOR address)

    if d <= radius:
        activate(location)
```

The inner operation is beautiful on modern hardware:

```text
XOR
XOR
XOR
POPCOUNT
```

over packed machine words.

An 8,192-bit hypervector is only:

$$
8192/8 = 1024\text{ bytes}
$$

So 131,072 SDM addresses require about:

$$
128\text{ MiB}
$$

just for address bits.

That is completely ordinary desktop memory.

---

# 12. A surprisingly serious SDM can now fit in consumer RAM

Consider:

```text
dimensions       = 8192
hard locations   = 131072
counter width    = int8
```

Address matrix:

$$
131072 \times 8192 / 8
$$

≈ **128 MiB**.

Counter matrix:

$$
131072 \times 8192
$$

≈ **1 GiB** using signed 8-bit counters.

Even after indexes, graph data and metadata, that's not exotic.

Scale to:

```text
1,048,576 hard locations
```

and the rough figures become:

```text
addresses ≈ 1 GiB
counters  ≈ 8 GiB
```

Still plausible on a workstation with 32–64 GB RAM.

The old Stanford prototype discussed in the SDM patent had only 8,192 hard locations and required custom decoding circuitry for its 256-bit Hamming-distance operations.

That's exactly the hardware-era inversion we're looking for.

---

# 13. But memory bandwidth will probably replace CPU arithmetic as the bottleneck

This would be one of our first experiments.

Scanning:

```text
1 million × 8192-bit addresses
```

means reading roughly **1 GiB** of address data for one brute-force lookup.

The XOR/popcount isn't necessarily the expensive part anymore.

Moving the memory may be.

So I'd compare:

```text
A. exhaustive AVX/POPCNT search

B. multithreaded CPU search

C. GPU exhaustive Hamming search

D. coarse LSH → SDM refinement

E. hierarchical SDM

F. clustered hard-location address space
```

That alone could produce useful results.

---

# 14. The structural matcher should *not* use vector similarity

Once candidates arrive, discard the approximate shortcut.

For each candidate:

```text
Query graph
     │
     │ structural alignment
     ▼
Candidate graph
```

Construct correspondence hypotheses:

```text
query.sun   ↔ candidate.nucleus

query.earth ↔ candidate.electron

query.attracts
        ↔
candidate.electrostatically_attracts
```

The important criterion is preservation of relationships rather than surface identity.

Gentner's original structure-mapping theory emphasizes exactly this distinction: analogies primarily align relational structure, especially interconnected higher-order relational systems.

---

# 15. This avoids a huge weakness of embeddings

Embedding similarity may notice:

```text
dog ≈ wolf
```

because their semantics are similar.

But analogy sometimes requires:

```text
dog : puppy
≈
horse : foal
```

even though:

```text
dog ≠ horse
puppy ≠ foal
```

What matters is:

```text
offspring_of(puppy,dog)

offspring_of(foal,horse)
```

The mapping is relational.

Recent analogy research still explicitly distinguishes semantic-vector similarity from relational analogical mapping and constructs relation networks to capture this difference.

So our architecture deliberately keeps two search spaces:

$$
\text{similarity space}
$$

and

$$
\text{relational space}.
$$

---

# 16. Here's the part I think makes the project substantially more interesting

Make **the entire engine incremental**.

Suppose memory contains:

```text
star A
   attracts
planet B
```

and later learns:

```text
planet B has moon C
```

Naive architecture:

```text
re-encode everything
reindex everything
rerun all analogies
rerun all inference
```

Our architecture instead computes:

```text
Δknowledge
```

and propagates only consequences.

So:

```text
old state
   +
delta
   ↓
affected hypervector components
   ↓
affected candidate relations
   ↓
affected structure mappings
   ↓
affected inferred facts
```

This is where ideas from modern incremental computation become useful.

Differential Dataflow, for example, was built specifically to maintain computations as inputs change and supports iterative computations such as graph algorithms without recomputing everything from scratch.

Dynamic graph research similarly treats complete recomputation after small graph changes as prohibitively wasteful and develops incremental matching algorithms instead.

---

# 17. So our engine becomes a differential reasoner

Imagine the knowledge graph contains:

```text
A ──causes──▶ B ──causes──▶ C
```

and we add:

```text
D ──analogous_to──▶ A
```

Instead of rerunning every analogy in memory, maintain dependencies:

```text
fact #1029
   ↓
mapping #421
   ↓
candidate inference #871
   ↓
hypothesis #992
```

Every derived object records:

```rust
struct Justification {
    dependencies: SmallVec<[FactId; 4]>,
}
```

Then:

```text
remove fact #1029
```

can invalidate only things descended from it.

That gives us something LLM-style systems generally don't naturally provide:

**explicit causal provenance for reasoning.**

---

# 18. The full pipeline

Conceptually:

```text
                     INPUT
                       │
                       ▼
               relational parser
                       │
            ┌──────────┴──────────┐
            │                     │
            ▼                     ▼
     explicit graph        binary fingerprint
            │                     │
            │                     ▼
            │              ASSOCIATIVE MEMORY
            │                  SDM/HDC
            │                     │
            │              top candidate cases
            │                     │
            └──────────┬──────────┘
                       ▼
              STRUCTURE MAPPING
                       │
               candidate analogy
                       │
                       ▼
               inferred relations
                       │
                       ▼
                verification layer
                       │
             accepted / uncertain
                       │
                       ▼
            DIFFERENTIAL DEPENDENCY
                    NETWORK
                       │
                       ▼
                 persistent memory
```

That is the project I would build.

---

# 19. Where bit-slicing enters

Not initially in SDM itself.

I'd use bit-slicing for **candidate filtering and rule propagation**.

Suppose we have 64 or 256 candidate analogies and need check:

```text
candidate contains relation CAUSES
candidate maps entity X
candidate satisfies temporal-order constraint
candidate contains no contradiction
```

Instead of:

```text
for candidate:
    evaluate rule
```

store candidate properties as bitsets.

Example with 64 candidates:

```text
HAS_CAUSES      = 110101000101...
HAS_TEMPORAL    = 010101001111...
MAPS_OBJECT_X   = 110001010101...
CONTRADICTS     = 000010000000...
```

Then:

```text
valid =
    HAS_CAUSES
  & HAS_TEMPORAL
  & MAPS_OBJECT_X
  & ~CONTRADICTS
```

One machine operation evaluates 64 hypotheses.

With AVX2:

256.

With AVX-512:

512.

So the system would alternate between two modes:

```text
high-dimensional vectors
→ retrieve memories

bitset population processing
→ prune hypotheses

relational graphs
→ exact structural reasoning
```

That's an unusual but very hardware-friendly combination.

---

# 20. There's an especially good representation trick here

Don't store a case fingerprint simply as a finalized binary vector.

Store an integer accumulator:

```rust
struct HyperAccumulator {
    sums: Box<[i16]>,
}
```

Every relation contributes ±1 to each coordinate.

Then the visible binary hypervector is:

```rust
bit[i] = sums[i] >= 0;
```

Why?

Because adding a fact becomes:

```text
accumulator += relation_vector
```

Removing one becomes:

```text
accumulator -= relation_vector
```

Then only bits crossing zero change.

So the fingerprint itself becomes incrementally maintainable.

That's a very nice match for the architecture.

---

# 21. You can then ask something surprisingly powerful

Suppose this memory exists:

```text
CASE 19281

company_A controls supplier_B
supplier_B controls resource_C
competitor_D depends_on resource_C
```

And a new case says:

```text
country_X controls port_Y
company_Z depends_on port_Y
```

There may be almost no lexical similarity.

The HDC layer might nevertheless retrieve broad relational candidates because role/relation structures contribute to their representations.

Then SME-like matching discovers:

```text
company_A    ↔ country_X
resource_C   ↔ port_Y
competitor_D ↔ company_Z

controls     ↔ controls
depends_on   ↔ depends_on
```

and suggests:

```text
the new situation may have a dependency structure
analogous to CASE 19281
```

Critically, it would not simply assert whatever happened in the earlier case.

It would generate **candidate inferences with provenance**.

That's how SME treats analogical inferences too: as suggested knowledge requiring further evaluation rather than automatically established facts.

---

# 22. Long-term memory could learn transformations, not just cases

This is where SDM becomes more exciting.

Instead of storing:

```text
input → object
```

store:

```text
context → transformation
```

For example:

```text
STATE_t
     ↓
TRANSFORMATION
     ↓
STATE_t+1
```

SDM/VSA research has already explored sequence prediction and state-transition learning; the 2014 VSA+SDM work on interoperable systems explicitly learned and predicted state transitions from examples.

Our version could store transformations like:

```text
CAUSE propagation
CONSERVATION relation
dependency reversal
containment propagation
symmetry transformation
```

Then a query retrieves not merely:

> What does this resemble?

but:

> **What transformations tended to apply to structures like this?**

That's much closer to reasoning.

---

# 23. Another important research question: can structural retrieval beat semantic retrieval?

This would make a very clean experiment.

Construct cases where surface similarity is deliberately misleading.

For example:

### Query

```text
A inhibits B
B activates C
therefore A indirectly suppresses C
```

Candidate 1:

```text
enzyme inhibits receptor
receptor activates protein
```

Lots of semantic overlap.

Candidate 2:

```text
law blocks permit
permit enables construction
```

Almost none.

But structurally:

```text
A ─| B → C
```

matches both.

Then compare:

```text
text embedding retrieval

vs

plain hypervector retrieval

vs

SDM retrieval

vs

SDM → SME reranking
```

The metric isn't simply retrieval accuracy.

Measure:

```text
structurally correct analogue @ k
```

That would directly test the premise.

---

# 24. The decisive benchmark should contain adversarial surface similarity

Otherwise a language model or embedding index will trivially look impressive.

Build cases such as:

```text
Source:

wolf chases deer
deer chases rabbit
```

and candidate:

```text
company acquires startup
startup acquires patent
```

Then test whether structural reasoning recognizes the chain.

Conversely:

```text
wolf watches deer
deer sleeps beside rabbit
```

has surface overlap but not the same relations.

A good system should prefer:

```text
company → startup → patent
```

over the semantically similar animal example.

---

# 25. I would make the first prototype deliberately small

Not an LLM.

Not natural language.

Not millions of facts.

Version 0 should use synthetic structured data.

Something like:

```rust
let source = case! {
    causes(a, b),
    causes(b, c),
    stronger_than(a, b),
};

let target = case! {
    enables(x, y),
    enables(y, z),
};
```

Then measure whether candidate retrieval identifies structurally similar cases.

This isolates the interesting algorithm from NLP noise.

---

# 26. Phase 1: implement only the hypervector layer

Rust is probably my first choice.

Core type:

```rust
#[repr(transparent)]
pub struct HyperVector {
    words: Box<[u64]>,
}

impl HyperVector {
    pub fn hamming(&self, other: &Self) -> u32 {
        self.words
            .iter()
            .zip(other.words.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum()
    }

    pub fn bind(&self, other: &Self) -> Self {
        let words = self
            .words
            .iter()
            .zip(other.words.iter())
            .map(|(a, b)| a ^ b)
            .collect();

        Self { words }
    }
}
```

An 8192-dimensional vector becomes:

```text
128 × u64
```

so a Hamming-distance comparison requires approximately:

```text
128 XORs
128 popcounts
```

plus accumulation.

That's extremely manageable.

---

# 27. Then benchmark SDM brutally

Vary:

$$
D = 1024,\ 4096,\ 8192,\ 16384
$$

and:

$$
H = 2^{12}, 2^{14}, 2^{16}, 2^{18}, 2^{20}
$$

hard locations.

Measure:

```text
lookup latency
memory bandwidth
recall accuracy
noise tolerance
capacity
write amplification
```

Compare:

```text
classical radius activation

top-k nearest hard locations

random subsampling

hierarchical activation

LSH-prefiltered activation
```

The 2023 SDM/HDC study itself evaluated multiple SDM variants rather than assuming the original construction was uniquely optimal.

---

# 28. Phase 2: implement a tiny structure mapper

Don't reproduce full SME initially.

Define relations:

```rust
enum Term {
    Entity(EntityId),
    Relation(RelationId),
}

struct Predicate {
    kind: PredicateKind,
    args: SmallVec<[Term; 4]>,
}
```

Candidate correspondences:

```rust
struct MatchHypothesis {
    left: Term,
    right: Term,
    score: f32,
}
```

Constraints:

```text
one-to-one mapping

predicate compatibility

argument consistency

higher-order relation consistency
```

Then maximize:

$$
S(M)=
\sum_i s(m_i)
+
\lambda\sum_{i,j} consistency(m_i,m_j)
$$

subject to one-to-one mapping constraints.

Initially a greedy approximation is perfectly adequate.

---

# 29. Phase 3: build the MAC/FAC experiment, but with modern memory

This is the moment the project becomes scientifically meaningful.

Store:

```text
10
100
1,000
10,000
100,000
1,000,000
```

synthetic relational cases.

Compare:

### Method A

Structure-match every case.

### Method B

Embedding/vector retrieval → structure match.

### Method C

HDC nearest-neighbor → structure match.

### Method D

SDM associative retrieval → structure match.

Measure:

```text
retrieval recall
structural accuracy
latency
memory usage
energy if measurable
```

What we really want to know:

> Can associative binary memory reduce the structural search space by orders of magnitude without throwing away the correct structural analogue?

MAC/FAC demonstrated the broad principle with older representations decades ago.

We can ask it again under completely different computational conditions.

---

# 30. Phase 4: make the entire thing incremental

Now add facts continuously.

For each update:

```text
+ relation
- relation
+ object
- object
```

measure:

```text
fingerprint update time

SDM maintenance time

number of candidate mappings invalidated

number of structural matches recomputed

number of derived facts changed
```

Compare against:

```text
full recomputation
```

This is where modern incremental computation techniques could produce the biggest difference.

---

# 31. Phase 5: let memories consolidate

Here's a more speculative extension I think could be excellent.

Suppose memory contains:

```text
case A
case B
case C
case D
```

and structure mapping repeatedly discovers the same relational skeleton:

```text
X controls Y
Y enables Z
X indirectly influences Z
```

Create a generalized schema:

```text
SCHEMA #481
```

Store *that* back into SDM.

Then future retrieval becomes:

```text
specific experience
       ↓
generalized structure
       ↓
new specific experience
```

So memory learns reusable relational abstractions.

This is close in spirit to old analogy-based learning systems, but coupled to a giant modern associative memory it gets much more interesting.

---

# 32. And then something resembling "sleep" becomes computationally useful

During idle periods:

```text
sample memory pair
       ↓
structure-map
       ↓
strong commonality?
       │
       yes
       ↓
derive schema
       ↓
store schema
```

No mysticism required.

It's simply offline memory consolidation.

Consumer machines have many idle hours and lots of cores.

So a system could continuously perform:

```text
memory → comparison → abstraction → memory
```

overnight.

That is another example of a strategy that looks wasteful if compute is scarce and attractive when compute is cheap.

---

# 33. A potentially very powerful feature: negative analogies

Most retrieval systems ask:

> What is similar?

Our structural engine can also record:

> What nearly matched, and exactly where did it fail?

Example:

```text
CASE A:
A causes B
B causes C

CASE B:
X causes Y
Y prevents Z
```

The systems align until:

```text
causes(B,C)
   ≠
prevents(Y,Z)
```

Store that boundary.

Later you can retrieve:

```text
similar except for causal polarity at relation #3
```

This gives the system explicit notions of:

```text
analogy
near analogy
counterexample
exception
contradiction
```

All extremely useful for reasoning.

---

# 34. This is where the incremental dependency graph pays off again

Suppose we infer:

```text
hypothesis H
```

because:

```text
analogy A
+
fact B
+
relation mapping C
```

Record:

```text
H
├── analogy A
├── fact B
└── mapping C
```

Later `B` disappears.

We don't retrain anything.

We retract:

```text
H
```

and every dependent conclusion.

That's classic truth-maintenance-system territory—another old AI idea worth borrowing.

The engine becomes less like a neural network and more like a **persistent computational epistemology**:

```text
what do I know?

why do I know it?

what memory suggested it?

what structural correspondence supports it?

what changes if this fact is wrong?
```

---

# 35. Could an LLM be involved?

Eventually, yes—but on the outside.

Not as the reasoning engine.

Use an LLM for:

```text
natural language
      ↓
candidate relations
      ↓
MARS engine
      ↓
explicit reasoning
      ↓
result graph
      ↓
LLM
      ↓
natural-language explanation
```

For example:

```text
"Company A depends heavily on supplier B.
Supplier B gets all of its lithium from mine C."
```

LLM emits:

```text
depends_on(A,B)
depends_on(B,C)
resource(C,lithium)
```

Then our engine handles memory and analogy.

This gives a clean division:

**LLM = interface / semantic parser**

**associative reasoner = persistent cognition**

---

# 36. Why not just use a vector database?

Because nearest-neighbor databases answer:

$$
\operatorname*{argmin}_{x} d(q,x)
$$

They're extremely useful.

But SDM does something conceptually different.

Memory is **distributed across storage locations**, and nearby cues can activate overlapping subsets of the same physical memory. Classical SDM storage and retrieval therefore behaves more like an associative field than a conventional key/value ANN index.

Whether that distinction gives us a practical advantage is an empirical question.

That's exactly why the project is worth doing.

---

# 37. Why not just use HDC alone?

Because high-dimensional superposition has limited capacity.

If we bind more and more structures:

```text
A + B + C + ... + Z
```

noise rises.

The 2023 HDC/SDM work explicitly argues for separating short-term superposition memory from larger associative long-term memory for this reason.

So:

```text
hypervector
```

should be viewed as a compressed working representation.

Not the whole database.

---

# 38. Why not just use a knowledge graph?

Because conventional graph lookup requires knowing what to ask for.

Our associative front-end gives us:

```text
query structure
     ↓
"show me remotely similar experiences"
```

before explicit matching.

That is the interesting combination.

---

# 39. And why not just use SME?

Scale.

Running structural comparison against:

```text
1,000,000 memories
```

would waste huge amounts of computation.

MAC/FAC recognized this in 1995 and introduced a cheap filtering stage precisely so expensive structural matching only touched a few candidates.

Our proposed contribution is essentially:

> **Build a 2026 version of this architectural idea using enormous binary associative memory, modern vector hardware, incremental graphs and parallel hypothesis processing.**

---

# 40. What appears genuinely underexplored

After searching, I would divide the ideas like this:

| Component | Research status |
|---|---|
| Sparse Distributed Memory | old but still studied |
| Hyperdimensional/VSA computation | currently active |
| SDM + VSA | demonstrated |
| VSA analogy | demonstrated |
| SDM + VSA analogy | demonstrated in 2013 |
| Structure Mapping Engine | mature classic research |
| cheap retrieval → SME | MAC/FAC already did it |
| incremental SME | demonstrated |
| incremental graph computation | highly active elsewhere |
| bit-parallel hypothesis filtering | standard technique individually |
| **SDM/HDC + explicit SME-like reasoning + differential maintenance** | **much less obvious** |
| **large commodity-hardware implementation of whole architecture** | **I found little evidence** |
| **millions of persistent cases + online schema consolidation** | **especially interesting** |
| **structural retrieval benchmark against modern embeddings** | **very worthwhile** |
| **provenance-aware analogical memory continuously updated online** | **strong research direction** |

So we have narrowed the novelty substantially.

That's good.

---

# 41. The research question I'd put at the top of the repository

Something like:

> **Can a binary associative-memory front end make explicit analogical reasoning practical over millions of continually changing relational memories on commodity hardware?**

Then break that into five hypotheses.

**H1 — Retrieval**

Binary associative retrieval retains structurally useful memories in its top-$k$ candidates despite surface dissimilarity.

**H2 — Scale**

The SDM/VSA stage reduces structural comparisons enough to outperform exhaustive relational matching by orders of magnitude.

**H3 — Incrementality**

Small knowledge changes require work proportional mostly to their affected dependency region rather than total memory size.

**H4 — Abstraction**

Repeated structural mappings can automatically form reusable generalized schemas.

**H5 — Robustness**

Explicit structural reranking reduces false analogies caused by semantic/surface similarity.

Those are falsifiable.

That's important.

---

# 42. There are several ways this experiment could fail

And those failures would still be interesting.

### Failure A

SDM retrieval isn't noticeably better than an ordinary Hamming ANN index.

Then:

> SDM's distributed storage isn't buying enough.

Useful result.

### Failure B

Hypervectors lose too much relational structure.

Then:

> use them only as loose filters.

Still useful.

### Failure C

SME-like mapping dominates runtime even after filtering.

Then:

> develop bit-parallel structural constraints.

Interesting systems project.

### Failure D

Embeddings retrieve analogies just as well.

Then:

> explicit structure adds little for the benchmark.

Important negative result.

### Failure E

Incremental maintenance uses more RAM than recomputation saves CPU.

Then we know the crossover.

Again useful.

---

# 43. The benchmark I'd really like to see

Create perhaps:

```text
1,000,000 synthetic cases
```

constructed from:

```text
100 entity types
100 relation types
50 structural templates
```

Randomize surface labels aggressively.

Example underlying template:

```text
A → B
B → C
C ─| D
```

Generate domains:

```text
biology
business
mechanics
social relationships
computer networks
logistics
ecology
```

with unrelated surface vocabulary.

Then hide the common template.

Query:

```text
one previously unseen domain
```

Retrieve structural analogues.

Compare:

```text
BM25/text
embedding ANN
random projection
HDC exact Hamming
HDC + SDM
HDC + SDM + structural matcher
```

Now we have a scientifically interpretable result.

---

# 44. Then make it harder

Perturb structures:

```text
delete edge
add distractor
reverse edge
swap argument
change predicate
insert intermediate node
```

Measure degradation.

The retrieval curve:

$$
P(\text{correct analogue} \mid \text{structural corruption})
$$

would tell us a lot.

---

# 45. Then test continual learning

Feed:

```text
case 1
case 2
...
case 1,000,000
```

without retraining epochs.

After each insertion:

```text
memory.store(case)
```

and immediately query it.

Measure:

```text
recall accuracy over time
interference
capacity saturation
old-memory degradation
schema emergence
```

This is where associative memories can be conceptually attractive compared with gradient-trained networks.

---

# 46. The most interesting eventual experiment might be program analogy

Instead of stories:

```text
graph = program dataflow
```

Example:

```text
parse
 ↓
filter
 ↓
aggregate
```

versus:

```text
decode
 ↓
validate
 ↓
summarize
```

The system could retrieve:

> These two programs have analogous computational structure despite different functions/names.

Possible applications:

```text
code search
algorithm discovery
bug pattern matching
compiler optimization
program synthesis
```

This might actually be easier to evaluate objectively than human conceptual analogy.

---

# 47. Another excellent target: scientific mechanism search

Represent mechanisms as relations:

```text
A inhibits B
B activates C
C diffuses through D
```

Then search a database of mechanisms for the same relational pattern in completely different domains.

That could potentially surface:

```text
biology ↔ electronics
ecology ↔ economics
fluid systems ↔ electrical systems
```

exactly the sort of cross-domain analogy ordinary keyword search misses.

Whether that works in practice is very uncertain—but that uncertainty is the research opportunity.

---

# 48. My recommended repository structure

If we actually build it, I'd organize it roughly:

```text
associative-reasoner/
│
├── crates/
│   ├── hypervector/
│   │   ├── packed.rs
│   │   ├── binding.rs
│   │   ├── bundle.rs
│   │   └── distance.rs
│   │
│   ├── sdm/
│   │   ├── address_space.rs
│   │   ├── counters.rs
│   │   ├── radius.rs
│   │   └── nearest.rs
│   │
│   ├── relations/
│   │   ├── graph.rs
│   │   ├── predicate.rs
│   │   └── encode.rs
│   │
│   ├── structure_map/
│   │   ├── hypotheses.rs
│   │   ├── constraints.rs
│   │   ├── scoring.rs
│   │   └── mapping.rs
│   │
│   ├── incremental/
│   │   ├── dependency.rs
│   │   ├── delta.rs
│   │   └── invalidation.rs
│   │
│   └── memory/
│       ├── case.rs
│       ├── retrieve.rs
│       └── consolidate.rs
│
├── benchmarks/
│   ├── sdm_scaling/
│   ├── structural_retrieval/
│   ├── continual_learning/
│   └── analogy/
│
└── datasets/
    ├── synthetic/
    └── raven/
```

And keep the first version absolutely free of deep-learning dependencies.

We want to know what the architecture itself can do.

---

# 49. The minimal experiment is surprisingly small

We don't initially need a grand AI system.

We need:

```text
8,192-bit vectors

65,536–131,072 SDM locations

1,000–100,000 relational cases

simple binary predicates

synthetic analogy generator

small structure matcher
```

and then answer:

> Does the right analogue survive associative retrieval often enough that exact structure mapping can recover it?

If the answer is **no**, stop.

If it is **yes**, scale.

That avoids spending six months building elaborate infrastructure around a false premise.

---

# 50. The finding that makes me most interested in continuing

It's this historical chain:

```text
1980s:
Kanerva
Sparse Distributed Memory
        │
        │
1989:
Structure Mapping Engine
        │
        │
1995:
MAC/FAC
cheap vector retrieval
→ expensive structural reasoning
        │
        │
2013:
SDM + binary VSA
for analogical inference
        │
        │
2020s:
HDC revival
incremental graph systems
huge RAM
SIMD/popcount
commodity GPUs
        │
        ▼
?????
```

We don't have to invent a speculative architecture out of thin air.

Multiple independent research traditions lead almost directly to it.

Yet the modern synthesis seems much less developed than you'd expect.

That is exactly the kind of thing I'd investigate.

---

## What I would do next

I'd now switch from literature exploration to **building the smallest falsifiable prototype**:

1. an optimized 8,192-bit Rust hypervector implementation;
2. a classical/top-$k$ SDM;
3. a generator for relational analogies with intentionally misleading surface features;
4. a minimal structure mapper;
5. benchmarks of exhaustive matching vs Hamming retrieval vs SDM→structure-mapping;
6. only if those results are promising, add incremental updates and schema consolidation.

The most important lesson from this research pass is that we should **not** try to invent a new "AI architecture" first. We should test one sharply defined proposition:

$$
\boxed{
\text{Can associative binary memory cheaply preserve structural analogues well enough to make exact analogical search scalable?}
}
$$

If that works, the incremental-memory system becomes worth pursuing. If it doesn't, we'll have learned exactly which link in the proposed architecture fails rather than merely producing another complicated cognitive architecture demo.
