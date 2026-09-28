# DBpedia ↔ Wikidata film sample (E26)

Fetched 2026-09-27 with `tools/kg_fetch.py data/kg-films --films 1000` (live endpoints change, so the
sample is versioned): 1,000 films linked by `owl:sameAs`, with their DBpedia ontology triples
(`dbpedia.jsonl`), Wikidata direct claims (`wikidata.jsonl`) and English labels of object entities;
`gold.json` holds DBpedia's `owl:equivalentProperty` links to Wikidata properties (alignment ground
truth); `wd_prop_labels.json` the English labels of the Wikidata properties discussed in
results/E26. DBpedia content is CC BY-SA; Wikidata content is CC0.

`wikidata_hop2.jsonl` (E28, fetched 2026-09-28): second-hop Wikidata claims of the sample's objects, from `tools/kg_hop2.py data/kg-films --via P57,P58,P162,P272,P495,P86 --props P27,P17,P1412,P37,P495` (director, screenwriter, producer, production company, country of origin, composer → citizenship, country, languages spoken, official language, country of origin).
