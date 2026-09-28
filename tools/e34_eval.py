#!/usr/bin/env python3
"""E34 (pre-registration addendum A): dataflow front end for code retrieval.

Paired comparison on one program set, encoded twice:

    python3 tools/ev2_codenet.py prepare W_MARS [--seed S] [--exclude-seed 1]
    python3 tools/ev2_codenet.py prepare W_PDG --frontend pdg --same-as W_MARS
    mars-bench ev2 --data W_MARS ; mars-bench ev2 --data W_PDG
    python3 tools/e34_eval.py cands W_PDG W_MARS     # code embedding top-100 over W_MARS's programs -> W_PDG/emb_cands.json (cached)
    mars-bench ev2 --data W_PDG --cands emb_cands.json   # -> W_PDG/fac_cands.json
    python3 tools/e34_eval.py evaluate W_MARS W_PDG OUT [--mars-only]

Arms: A1 MARS fused with py2pdg vs with py2mars; A2 the code embedding's top-100
re-ranked by ½ cosine + ½ MARS FAC (py2pdg) vs the embedding alone.
Predictions: P10 A1 > 0; P11 MARS (py2pdg) < embedding; P12 A2 ≥ 0.
Programs the py2pdg encoder skips are ranked as missing (empty list) for MARS-pdg,
so the program set and relevance sets are exactly EV2's.
"""

import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(__file__))
from ev2_codenet import metrics  # noqa: E402

MODEL = "jinaai/jina-embeddings-v2-base-code"
DEPTH = 100


def load(work):
    man = json.load(open(f"{work}/manifest.json"))
    src = {}
    for l in open(f"{work}/sources.jsonl"):
        d = json.loads(l)
        src[d["case"]] = d["src"]
    return man, src


def embedding(work, src_work):
    """Cosine top-DEPTH lists and scores of the code embedding (EV2's model and caps) over
    src_work's programs (the full program set, including any py2pdg could not encode);
    cached in work/emb.npy with its case list."""
    import numpy as np
    path = f"{work}/emb.npy"
    man, src = load(src_work)
    cases = [m["case"] for m in man["cases"]]
    if os.path.exists(path) and json.load(open(f"{work}/emb_cases.json")) == cases:
        E = np.load(path)
    else:
        from fastembed import TextEmbedding
        E = np.array(list(TextEmbedding(model_name=MODEL).embed([src[c][:2000] for c in cases], batch_size=4)))
        E /= np.linalg.norm(E, axis=1, keepdims=True)
        np.save(path, E)
        json.dump(cases, open(f"{work}/emb_cases.json", "w"))
    S = E @ E.T
    np.fill_diagonal(S, -np.inf)
    order = np.argsort(-S, axis=1, kind="stable")[:, :DEPTH]
    return cases, {c: [cases[j] for j in order[i]] for i, c in enumerate(cases)}, {c: [float(S[i, j]) for j in order[i]] for i, c in enumerate(cases)}


def cands(work, src_work):
    cases, top, _ = embedding(work, src_work)
    json.dump(top, open(f"{work}/emb_cands.json", "w"))
    print(f"{len(cases)} embedding candidate lists -> {work}/emb_cands.json", file=sys.stderr)


def evaluate(w_mars, w_pdg, out, mars_only):
    from stats import ci, paired
    man_m, _ = load(w_mars)
    man_p, _ = load(w_pdg)
    cases = [m["case"] for m in man_m["cases"]]
    prob = {m["case"]: m["problem"] for m in man_m["cases"]}
    group = defaultdict(set)
    for c in cases:
        group[prob[c]].add(c)
    rm = json.load(open(f"{w_mars}/mars_rankings.json"))
    rp = json.load(open(f"{w_pdg}/mars_rankings.json"))
    rk = {"MARS fused, py2mars": {c: rm["fused"][c] for c in cases},
          "MARS fused, py2pdg": {c: rp["fused"].get(c, []) for c in cases},
          "MARS FP-literal, py2mars": {c: rm["fp"][c] for c in cases},
          "MARS FP-literal, py2pdg": {c: rp["fp"].get(c, []) for c in cases}}
    if not mars_only:
        _, top, cos = embedding(w_pdg, w_mars)
        fac = json.load(open(f"{w_pdg}/fac_cands.json"))
        rk["code embedding"] = top
        rer = {}
        for c in cases:
            f = fac.get(c, [])
            sc = [(0.5 * cos[c][i] + 0.5 * (f[i] if i < len(f) else 0.0), top[c][i]) for i in range(len(top[c]))]
            rer[c] = [x for _, x in sorted(sc, key=lambda t: (-t[0], t[1]))]
        rk["embedding top-100 re-ranked by ½cos + ½FAC (py2pdg)"] = rer
    per = {n: [metrics(r[c], group[prob[c]] - {c}) for c in cases] for n, r in rk.items()}
    N = len(cases)
    L = ["# E34: dataflow front end (py2pdg) for code retrieval\n",
         f"{N} programs from {len(man_m['problems'])} CodeNet Python800 problems (seed {man_m['seed']}, exclude-seed {man_m.get('exclude_seed', 0)}; "
         f"the program set is py2mars's sample; py2pdg re-encodes the same programs, {man_p['skipped']} of them skipped). Relevant = other solutions of the same problem.\n",
         "| method | MAP@R [95% CI] | Hits@1 [95% CI] | MRR [95% CI] |", "|---|---|---|---|"]
    res = {}
    for n, v in per.items():
        a, h, r = ci([x[0] for x in v]), ci([x[1] for x in v]), ci([x[2] for x in v])
        L.append(f"| {n} | {a[0]:.4f} [{a[1]:.4f}, {a[2]:.4f}] | {h[0]:.4f} [{h[1]:.4f}, {h[2]:.4f}] | {r[0]:.4f} [{r[1]:.4f}, {r[2]:.4f}] |")
        res[n] = {"map_at_r": a, "hits1": h, "mrr": r}
    comps = [("A1 (P10)", "MARS fused, py2pdg", "MARS fused, py2mars"), ("A1-FP", "MARS FP-literal, py2pdg", "MARS FP-literal, py2mars")]
    if not mars_only:
        comps += [("P11", "MARS fused, py2pdg", "code embedding"), ("A2 (P12)", "embedding top-100 re-ranked by ½cos + ½FAC (py2pdg)", "code embedding")]
    L += ["", "**Paired comparisons** (MAP@R, paired over queries):", "", "| comparison | A − B [95% CI] | p |", "|---|---|---|"]
    pc = {}
    for tag, a, b in comps:
        d = paired([x[0] for x in per[a]], [x[0] for x in per[b]])
        L.append(f"| {tag}: {a} − {b} | {d[0]:+.4f} [{d[1]:+.4f}, {d[2]:+.4f}] | {d[3]:.4f} |")
        pc[tag] = {"a": a, "b": b, "diff": d}
    os.makedirs(out, exist_ok=True)
    tag = "E34-dev" if man_m.get("exclude_seed") else "E34-test"
    open(f"{out}/{tag}.md", "w").write("\n".join(L) + "\n")
    json.dump({"config": {"seed": man_m["seed"], "exclude_seed": man_m.get("exclude_seed", 0), "programs": N, "pdg_skipped": man_p["skipped"], "pdg_names": os.environ.get("PDG_NAMES", "1"),
                          "depth": DEPTH, "embedding_model": None if mars_only else MODEL},
               "results": res, "paired": pc, "per_query": {n: [list(x) for x in v] for n, v in per.items()}, "cases": cases}, open(f"{out}/{tag}.json", "w"))
    print("\n".join(L))


if __name__ == "__main__":
    cmd = sys.argv[1]
    if cmd == "cands":
        cands(sys.argv[2], sys.argv[3])
    elif cmd == "evaluate":
        evaluate(sys.argv[2], sys.argv[3], sys.argv[4], "--mars-only" in sys.argv)
