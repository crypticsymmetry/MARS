#!/usr/bin/env python3
"""Fetch a DBpedia/Wikidata film sample for knowledge-graph vocabulary
alignment (E26).

    python3 tools/kg_fetch.py OUT_DIR [--films 1000]

Writes OUT_DIR/films.json (DBpedia IRI <-> Wikidata Q-id), OUT_DIR/dbpedia.jsonl
and OUT_DIR/wikidata.jsonl (one film per line: its outgoing triples with object
kind, value and English label), and OUT_DIR/gold.json (DBpedia ontology
owl:equivalentProperty links to Wikidata properties: the alignment ground truth).
"""

import json
import os
import sys
import time
import urllib.parse
import urllib.request

UA = "MARS-research/0.1 (analogical vocabulary alignment experiments)"
DBP = "https://dbpedia.org/sparql"
WDQ = "https://query.wikidata.org/sparql"


def sparql(endpoint, query, retries=4):
    url = endpoint + "?" + urllib.parse.urlencode({"query": query})
    for i in range(retries):
        try:
            req = urllib.request.Request(url, headers={"Accept": "application/sparql-results+json", "User-Agent": UA})
            with urllib.request.urlopen(req, timeout=120) as r:
                return json.load(r)["results"]["bindings"]
        except Exception as e:
            print(f"  retry {i + 1}: {str(e)[:120]}", file=sys.stderr, flush=True)
            time.sleep(5 * (i + 1))
    raise RuntimeError("SPARQL query failed")


# DBpedia ontology properties that are page/bookkeeping metadata, not film facts.
SKIP_DBO = ("wikiPage", "abstract", "thumbnail", "wikiPageID", "wikiPageRevisionID", "wikiPageLength", "wikiPageWikiLink", "wikiPageExternalLink")


def literal(b):
    """(kind, value) for a SPARQL literal binding, or None for text/identifiers."""
    dt = b.get("datatype", "")
    v = b["value"]
    if dt.endswith(("#date", "#dateTime")):
        return ("date", v[:10].lstrip("+"))
    if dt.endswith("#gYear"):
        return ("year", v.lstrip("+")[:4])
    if dt.endswith(("#integer", "#double", "#decimal", "#float", "#nonNegativeInteger", "#positiveInteger")) or "dbpedia.org/datatype" in dt:
        try:
            return ("number", repr(round(float(v), 3)))
        except ValueError:
            return None
    return None


def main():
    out = sys.argv[1]
    n = int(sys.argv[sys.argv.index("--films") + 1]) if "--films" in sys.argv else 1000
    os.makedirs(out, exist_ok=True)
    gold = sparql(DBP, """PREFIX owl: <http://www.w3.org/2002/07/owl#>
SELECT DISTINCT ?p ?q WHERE { ?p owl:equivalentProperty ?q .
 FILTER(STRSTARTS(STR(?p),"http://dbpedia.org/ontology/") && STRSTARTS(STR(?q),"http://www.wikidata.org/entity/P")) }""")
    json.dump(sorted({(b["p"]["value"].split("/")[-1], b["q"]["value"].split("/")[-1]) for b in gold}), open(f"{out}/gold.json", "w"), indent=0)
    print(f"gold equivalences: {len(gold)}", file=sys.stderr)
    films = sparql(DBP, f"""PREFIX dbo: <http://dbpedia.org/ontology/> PREFIX owl: <http://www.w3.org/2002/07/owl#>
SELECT DISTINCT ?f ?wd WHERE {{ ?f a dbo:Film ; dbo:director ?d ; dbo:starring ?s ; owl:sameAs ?wd .
 FILTER(STRSTARTS(STR(?wd),"http://www.wikidata.org/entity/Q")) }} ORDER BY ?f LIMIT {n}""")
    pairs = sorted({(b["f"]["value"], b["wd"]["value"].split("/")[-1]) for b in films})
    json.dump(pairs, open(f"{out}/films.json", "w"), indent=0)
    print(f"films: {len(pairs)}", file=sys.stderr)
    B = 40
    with open(f"{out}/dbpedia.jsonl", "w") as f:
        for i in range(0, len(pairs), B):
            vals = " ".join(f"<{p[0]}>" for p in pairs[i : i + B])
            rows = sparql(DBP, f"""PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT ?f ?p ?o ?l WHERE {{ VALUES ?f {{ {vals} }} ?f ?p ?o .
 FILTER(STRSTARTS(STR(?p),"http://dbpedia.org/ontology/"))
 OPTIONAL {{ ?o rdfs:label ?l . FILTER(lang(?l)="en") }} }}""")
            per = {p[0]: [] for p in pairs[i : i + B]}
            for r in rows:
                p = r["p"]["value"].split("/")[-1]
                if p.startswith(SKIP_DBO):
                    continue
                o = r["o"]
                if o["type"] == "uri":
                    per[r["f"]["value"]].append({"p": p, "kind": "entity", "value": o["value"].split("/")[-1], "label": r.get("l", {}).get("value")})
                else:
                    lit = literal(o)
                    if lit:
                        per[r["f"]["value"]].append({"p": p, "kind": lit[0], "value": lit[1]})
            for fi, triples in per.items():
                f.write(json.dumps({"film": fi.split("/")[-1], "triples": triples}) + "\n")
            print(f"  dbpedia {i + B}/{len(pairs)}", file=sys.stderr, flush=True)
    with open(f"{out}/wikidata.jsonl", "w") as f:
        for i in range(0, len(pairs), B):
            vals = " ".join(f"wd:{p[1]}" for p in pairs[i : i + B])
            rows = sparql(WDQ, f"""SELECT ?f ?p ?o ?oLabel WHERE {{ VALUES ?f {{ {vals} }} ?f ?p ?o .
 FILTER(STRSTARTS(STR(?p),"http://www.wikidata.org/prop/direct/P"))
 FILTER(isIRI(?o) || datatype(?o) IN (xsd:dateTime, xsd:decimal, xsd:integer, xsd:double))
 SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }} }}""")
            per = {p[1]: [] for p in pairs[i : i + B]}
            for r in rows:
                p = r["p"]["value"].split("/")[-1]
                o = r["o"]
                q = r["f"]["value"].split("/")[-1]
                if o["type"] == "uri":
                    if "wikidata.org/entity/Q" not in o["value"]:
                        continue
                    per[q].append({"p": p, "kind": "entity", "value": o["value"].split("/")[-1], "label": r.get("oLabel", {}).get("value")})
                else:
                    lit = literal(o)
                    if lit:
                        per[q].append({"p": p, "kind": lit[0], "value": lit[1]})
            for q, triples in per.items():
                f.write(json.dumps({"film": q, "triples": triples}) + "\n")
            print(f"  wikidata {i + B}/{len(pairs)}", file=sys.stderr, flush=True)
            time.sleep(1)


if __name__ == "__main__":
    main()
