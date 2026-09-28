#!/usr/bin/env python3
"""Fetch second-hop Wikidata claims for a kg_fetch.py sample (E28).

    python3 tools/kg_hop2.py KG_DIR [--via P184,P19,P20,P69,P108] [--props P69,P108,P27,P101,P17]

For every object of the --via relations in KG_DIR/wikidata.jsonl (e.g. a
scientist's doctoral advisor, birthplace, alma mater, employer), fetches its
--props claims (entity objects, with English labels). Writes
KG_DIR/wikidata_hop2.jsonl: one {"entity": Q, "triples": [...]} per line.
"""

import json
import os
import sys
import time

sys.path.insert(0, os.path.dirname(__file__))
from kg_fetch import WDQ, sparql  # noqa: E402


def arg(name, default):
    return sys.argv[sys.argv.index(name) + 1] if name in sys.argv else default


def main():
    kg = sys.argv[1]
    via = set(arg("--via", "P184,P19,P20,P69,P108").split(","))
    props = arg("--props", "P69,P108,P27,P101,P17").split(",")
    ents = sorted({t["value"] for l in open(f"{kg}/wikidata.jsonl") for t in json.loads(l)["triples"] if t["p"] in via and t["kind"] == "entity"})
    print(f"{len(ents)} second-hop entities", file=sys.stderr)
    per = {e: [] for e in ents}
    B = 150
    pv = " ".join(f"wdt:{p}" for p in props)
    for i in range(0, len(ents), B):
        vals = " ".join(f"wd:{e}" for e in ents[i : i + B])
        rows = sparql(WDQ, f"""SELECT ?s ?p ?o ?oLabel WHERE {{ VALUES ?s {{ {vals} }} VALUES ?p {{ {pv} }} ?s ?p ?o .
 FILTER(isIRI(?o)) SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }} }}""")
        for r in rows:
            o = r["o"]["value"]
            if "wikidata.org/entity/Q" not in o:
                continue
            per[r["s"]["value"].split("/")[-1]].append({"p": r["p"]["value"].split("/")[-1], "kind": "entity", "value": o.split("/")[-1], "label": r.get("oLabel", {}).get("value")})
        print(f"  hop2 {min(i + B, len(ents))}/{len(ents)}", file=sys.stderr, flush=True)
        time.sleep(1)
    with open(f"{kg}/wikidata_hop2.jsonl", "w") as f:
        for e in ents:
            # Deduplicate (a claim can come back once per label row).
            seen, ts = set(), []
            for t in per[e]:
                if (t["p"], t["value"]) not in seen:
                    seen.add((t["p"], t["value"]))
                    ts.append(t)
            f.write(json.dumps({"entity": e, "triples": ts}) + "\n")


if __name__ == "__main__":
    main()
