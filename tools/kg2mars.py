#!/usr/bin/env python3
"""Convert the E26 DBpedia/Wikidata film sample (tools/kg_fetch.py) into MARS
cases under three anchoring conditions.

    python3 tools/kg2mars.py KG_DIR OUT_DIR --condition A|B|C [--hop2]

One case per film per KG: its outgoing triples as (<kg>:<property> film object),
with property names left unresolved (`dbo:director`, `wdt:P57`: no shared
vocabulary). Conditions differ only in what the two KGs visibly share:

* A (structure only): every entity and literal value is KG-specific;
* B (value anchors): literal values (dates, years, numbers) are shared
  entities carrying an exact-match attribute `(val:<kind>:<value> v)`;
* C (value + label anchors): B, and entity objects carry their English label
  as an attribute `(lbl:<label> e)`.

With --hop2 (E28), Wikidata cases also get the second-hop claims of their
objects from KG_DIR/wikidata_hop2.jsonl (tools/kg_hop2.py), e.g. the doctoral
advisor's employer: (wdt:p108 advisor org), labelled like first-hop objects.

Also writes OUT_DIR/manifest.json (case, kg, film pair index) and copies
gold.json (DBpedia owl:equivalentProperty links, the alignment ground truth).
"""

import json
import os
import re
import sys


def tok(s):
    s = re.sub(r"[^A-Za-z0-9._-]+", "-", s).strip("-").lower()
    return s or "x"


def main():
    kg, out = sys.argv[1], sys.argv[2]
    cond = sys.argv[sys.argv.index("--condition") + 1]
    os.makedirs(out, exist_ok=True)
    pairs = json.load(open(f"{kg}/films.json"))
    dbp = {json.loads(l)["film"]: json.loads(l)["triples"] for l in open(f"{kg}/dbpedia.jsonl")}
    wdq = {json.loads(l)["film"]: json.loads(l)["triples"] for l in open(f"{kg}/wikidata.jsonl")}
    hop2 = {}
    if "--hop2" in sys.argv:
        for l in open(f"{kg}/wikidata_hop2.jsonl"):
            r = json.loads(l)
            hop2[r["entity"]] = r["triples"]
    preds, attrs, lines, manifest = set(), set(), [], []
    for idx, (d_iri, q) in enumerate(pairs):
        d = d_iri.split("/")[-1]
        for kgname, triples, film, prefix in (("dbpedia", dbp.get(d, []), f"d-{tok(d)}", "dbo"), ("wikidata", wdq.get(q, []), f"w-{tok(q)}", "wdt")):
            ek = prefix[0]  # entity namespace per KG: d / w
            facts = set()
            subj = {id(t): film for t in triples}
            if kgname == "wikidata" and hop2:
                extra = []
                for t in triples:
                    if t["kind"] == "entity" and t["value"] in hop2:
                        for u in hop2[t["value"]]:
                            extra.append(u)
                            subj[id(u)] = f"{ek}-{tok(t['value'])}"
                triples = triples + extra
            for t in triples:
                p = f"{prefix}:{tok(t['p'])}"
                if t["kind"] == "entity":
                    obj = f"{ek}-{tok(t['value'])}"
                    if cond == "C" and t.get("label"):
                        a = f"lbl:{tok(t['label'])}"
                        attrs.add(a)
                        facts.add(f"({a} {obj})")
                else:
                    val = f"{t['kind']}:{tok(t['value'])}"
                    if cond == "A":
                        obj = f"{ek}-v-{tok(val)}"
                    else:
                        obj = f"v-{tok(val)}"
                        a = f"val:{tok(val)}"
                        attrs.add(a)
                        facts.add(f"({a} {obj})")
                preds.add(p)
                facts.add(f"({p} {subj[id(t)]} {obj})")
            if not any(f.startswith(f"({prefix}:") for f in facts):
                continue
            name = f"{kgname[:2]}{idx}"
            lines.append(f"(defcase {name}\n  " + "\n  ".join(sorted(facts)) + ")")
            manifest.append({"case": name, "kg": kgname, "film": idx})
    with open(f"{out}/cases.mars", "w") as f:
        f.write(f";; tools/kg2mars.py condition {cond}\n")
        for p in sorted(preds):
            f.write(f"(defpredicate {p} :arity 2 :kind relation :canonical nil)\n")
        for a in sorted(attrs):
            f.write(f"(defpredicate {a} :arity 1 :kind attribute)\n")
        f.write("\n".join(lines) + "\n")
    json.dump(manifest, open(f"{out}/manifest.json", "w"), indent=0)
    gold = json.load(open(f"{kg}/gold.json"))
    json.dump(gold, open(f"{out}/gold.json", "w"), indent=0)
    n_d = sum(1 for m in manifest if m["kg"] == "dbpedia")
    print(f"condition {cond}: {len(manifest)} cases ({n_d} dbpedia), {len(preds)} predicates, {len(attrs)} anchor attributes", file=sys.stderr)


if __name__ == "__main__":
    main()
