#!/usr/bin/env python3
"""English labels of every Wikidata property in a kg_fetch.py sample (and its gold links).

    python3 tools/wd_prop_labels.py KG_DIR      # writes KG_DIR/wd_prop_labels.json

Used to audit E26 alignments against Wikidata's own property names (the
owl:equivalentProperty gold standard is incomplete).
"""

import json
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from kg_fetch import WDQ, sparql  # noqa: E402


def main():
    kg = sys.argv[1]
    props = {t["p"] for l in open(f"{kg}/wikidata.jsonl") for t in json.loads(l)["triples"]}
    if os.path.exists(f"{kg}/gold.json"):
        props |= {q for _, q in json.load(open(f"{kg}/gold.json"))}
    props = sorted(props, key=lambda p: int(p[1:]))
    labels = {}
    for i in range(0, len(props), 200):
        vals = " ".join(f"wd:{p}" for p in props[i : i + 200])
        for r in sparql(WDQ, f"""SELECT ?p ?l WHERE {{ VALUES ?p {{ {vals} }} ?p rdfs:label ?l . FILTER(lang(?l)="en") }}"""):
            labels[r["p"]["value"].split("/")[-1]] = r["l"]["value"]
    json.dump(labels, open(f"{kg}/wd_prop_labels.json", "w"), indent=0, ensure_ascii=False)
    print(f"{len(labels)}/{len(props)} property labels", file=sys.stderr)


if __name__ == "__main__":
    main()
