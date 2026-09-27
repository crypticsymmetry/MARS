# DBpedia ↔ Wikidata film sample (E26)

Fetched 2026-09-27 with `tools/kg_fetch.py data/kg-films --films 1000` (live endpoints change, so the
sample is versioned): 1,000 films linked by `owl:sameAs`, with their DBpedia ontology triples
(`dbpedia.jsonl`), Wikidata direct claims (`wikidata.jsonl`) and English labels of object entities;
`gold.json` holds DBpedia's `owl:equivalentProperty` links to Wikidata properties (alignment ground
truth); `wd_prop_labels.json` the English labels of the Wikidata properties discussed in
results/E26. DBpedia content is CC BY-SA; Wikidata content is CC0.
