# DBpedia ↔ Wikidata scientist sample (E26, second domain)

Fetched 2026-09-27 with `tools/kg_fetch.py data/kg-scientists --films 1000 --class Scientist --require almaMater`
(live endpoints change, so the sample is versioned). It holds 1,000 `dbo:Scientist` entities with an alma mater, linked by `owl:sameAs`.
The file layout is the same as `data/kg-films`, and `films.json` keeps its name, although here it lists scientist pairs:
- `films.json`: pairs of DBpedia IRI and Wikidata Q-id. 997 distinct DBpedia IRIs; 3 are linked to two Q-ids.
- `dbpedia.jsonl`: DBpedia ontology triples, ~9 per scientist.
- `wikidata.jsonl`: Wikidata direct claims, ~24 per scientist; 7 scientists have none.
- `gold.json`: DBpedia `owl:equivalentProperty` links, identical to the film sample's.
- `wd_prop_labels.json`: English labels of every Wikidata property in the sample and in gold (`tools/wd_prop_labels.py`).

DBpedia content is CC BY-SA; Wikidata content is CC0.

`wikidata_hop2.jsonl` (E28, fetched 2026-09-28): second-hop Wikidata claims of the sample's objects, from `tools/kg_hop2.py data/kg-scientists` (defaults: via P184 doctoral advisor, P19/P20 birth/death place, P69 educated at, P108 employer; properties P69, P108, P27, P101, P17).
