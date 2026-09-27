#!/usr/bin/env python3
"""E25 analysis: LLM front-end variants and ensembles on StoryAnalogy.

    python3 tools/e25_analysis.py MC.json OUT_DIR NAME=E23_JSON:E24_JSON ... [--patterns CACHE.jsonl]

Each NAME=E23_JSON:E24_JSON is one front end (mars-bench e23 / e24 outputs on
its converted cases). Reports, per front end and for every ensemble of two or
more front ends:
* E23 multiple choice: accuracy / target > noun / target > random for the
  MARS analogy score and the surface-discounted analogy score (ensemble =
  mean score over front ends);
* E24 pooled retrieval: R@1 / R@5 / R@10 / MRR of the MARS fused ranking
  (ensemble = reciprocal-rank fusion of the front ends' rankings).
With --patterns (v2 cache with topic-free 'pattern' sentences), also the
*pattern embedding* control: the LLM's abstraction compared by sentence
embeddings, without MARS.
"""

import itertools
import json
import os
import sys
from collections import Counter

sys.path.insert(0, os.path.dirname(__file__))

E23_METHODS = ["fingerprint analogy profile", "analogy − 0.5·surface"]
E24_METHOD = "MARS fused FAC + fingerprint analogy"


def mc_metrics(mc, scores):
    acc = vn = vr = nn = nr = 0
    for i, q in enumerate(mc):
        sc, a = scores[i], q["answer"]
        t = sc[a]
        acc += all(j == a or x < t for j, x in enumerate(sc))
        for j, ty in enumerate(q["types"]):
            if j == a:
                continue
            if ty == "noun":
                vn += t > sc[j]
                nn += 1
            else:
                vr += t > sc[j]
                nr += 1
    return acc / len(mc), vn / nn, vr / nr


def ret_metrics(mc, rankings):
    rr = []
    for i, q in enumerate(mc):
        t = f"q{i}-c{q['answer']}"
        l = rankings[i][:50]
        rr.append(l.index(t) if t in l else None)
    rec = lambda k: sum(1 for p in rr if p is not None and p < k) / len(rr)
    return rec(1), rec(5), rec(10), sum(1 / (p + 1) for p in rr if p is not None) / len(rr)


def rrf(lists):
    sc = Counter()
    for l in lists:
        for p, c in enumerate(l[:50]):
            sc[c] += 1 / (60 + p)
    return [c for c, _ in sorted(sc.items(), key=lambda t: (-t[1], t[0]))]


def main():
    mc_path, out = sys.argv[1], sys.argv[2]
    args = sys.argv[3:]
    pat_path = None
    if "--patterns" in args:
        pat_path = args[args.index("--patterns") + 1]
        args = [a for a in args if a not in ("--patterns", pat_path)]
    mc = json.load(open(mc_path))
    fes = {}
    for a in args:
        name, paths = a.split("=", 1)
        e23, e24 = paths.split(":")
        p23 = {r["q"]: r["scores"] for r in json.load(open(e23))["per_question"]}
        p24 = {r["q"]: r["rankings"][E24_METHOD] for r in json.load(open(e24))["per_query"]}
        fes[name] = (p23, p24)
    os.makedirs(out, exist_ok=True)
    rows = []
    lines = ["| front end(s) | analogy: acc (t>noun / t>random) | analogy − 0.5·surface: acc (t>noun / t>random) | retrieval R@1 | R@5 | R@10 | MRR |", "|---|---|---|---|---|---|---|"]
    names = list(fes)
    combos = [(n,) for n in names] + [c for k in range(2, len(names) + 1) for c in itertools.combinations(names, k)]
    for combo in combos:
        cells = []
        rec = {"front_ends": list(combo)}
        for m in E23_METHODS:
            sc = {i: [sum(fes[n][0][i][m][j] for n in combo) / len(combo) for j in range(len(mc[i]["choices"]))] for i in range(len(mc))}
            a, vn, vr = mc_metrics(mc, sc)
            cells.append(f"{a:.3f} ({vn:.2f} / {vr:.2f})")
            rec[m] = {"accuracy": a, "target_over_noun": vn, "target_over_random": vr}
        rk = {i: (fes[combo[0]][1][i] if len(combo) == 1 else rrf([fes[n][1][i] for n in combo])) for i in range(len(mc))}
        r1, r5, r10, mrr = ret_metrics(mc, rk)
        rec["retrieval"] = {"r1": r1, "r5": r5, "r10": r10, "mrr": mrr}
        label = combo[0] if len(combo) == 1 else "ensemble: " + " + ".join(combo)
        lines.append(f"| {label} | {cells[0]} | {cells[1]} | {r1:.3f} | {r5:.3f} | {r10:.3f} | {mrr:.3f} |")
        rows.append(rec)
    text = "\n".join(lines) + "\n"
    if pat_path:
        from fastembed import TextEmbedding
        import numpy as np
        pats = {}
        for l in open(pat_path):
            r = json.loads(l)
            if r.get("pattern"):
                pats[r["id"]] = r["pattern"]
        names_all = [f"q{i}-s" for i in range(len(mc))] + [f"q{i}-c{j}" for i, q in enumerate(mc) for j in range(len(q["choices"]))]
        texts = [pats.get(n, "") for n in names_all]
        e = np.array(list(TextEmbedding(model_name="BAAI/bge-small-en-v1.5").embed(texts)))
        e /= np.linalg.norm(e, axis=1, keepdims=True)
        idx = {n: k for k, n in enumerate(names_all)}
        sc = {i: [float(e[idx[f"q{i}-s"]] @ e[idx[f"q{i}-c{j}"]]) for j in range(len(q["choices"]))] for i, q in enumerate(mc)}
        a, vn, vr = mc_metrics(mc, sc)
        mem = names_all
        rk = {}
        for i in range(len(mc)):
            s = e @ e[idx[f"q{i}-s"]]
            order = [mem[k] for k in np.argsort(-s, kind="stable") if mem[k] != f"q{i}-s"]
            rk[i] = order
        r1, r5, r10, mrr = ret_metrics(mc, rk)
        cover = sum(1 for n in names_all if n in pats) / len(names_all)
        # Combinations: MARS (all front ends) with the pattern embedding and lexical TF-IDF.
        from e24_pipeline import memory, tfidf_rank
        mnames, mtexts = memory(mc)
        qidx = [mnames.index(f"q{i}-s") for i in range(len(mc))]
        lex = tfidf_rank(mtexts, qidx)
        lexr = {i: [mnames[j] for j in lex[qidx[i]]] for i in range(len(mc))}
        mars_all = {i: rrf([fes[n][1][i] for n in names]) for i in range(len(mc))}
        combos2 = [("MARS ensemble + pattern embedding", [mars_all, rk]), ("MARS ensemble + lexical", [mars_all, lexr]), ("MARS ensemble + pattern embedding + lexical", [mars_all, rk, lexr]), ("pattern embedding + lexical", [rk, lexr])]
        text += "\n**Rank fusion (RRF) with text signals** (retrieval R@1 / R@5 / R@10 / MRR):\n\n| combination | R@1 | R@5 | R@10 | MRR |\n|---|---|---|---|---|\n"
        for label, lists in combos2:
            fused = {i: rrf([l[i] for l in lists]) for i in range(len(mc))}
            c1, c5, c10, cm = ret_metrics(mc, fused)
            text += f"| {label} | {c1:.3f} | {c5:.3f} | {c10:.3f} | {cm:.3f} |\n"
            rows.append({"combination": label, "retrieval": {"r1": c1, "r5": c5, "r10": c10, "mrr": cm}})
        # Multiple choice: Borda (rank sum) of the MARS ensemble score and the pattern embedding.
        ens = {i: [sum(fes[n][0][i]["analogy − 0.5·surface"][j] for n in names) for j in range(len(mc[i]["choices"]))] for i in range(len(mc))}
        def ranks(v):
            o = sorted(range(len(v)), key=lambda j: -v[j])
            r = [0] * len(v)
            for k, j in enumerate(o):
                r[j] = k
            return r
        borda = {i: [-(ranks(ens[i])[j] + ranks(sc[i])[j]) + 1e-6 * ens[i][j] for j in range(len(mc[i]["choices"]))] for i in range(len(mc))}
        ba, bn, br = mc_metrics(mc, borda)
        text += f"\nMultiple choice, Borda of the MARS ensemble (analogy − 0.5·surface) and the pattern embedding: {ba:.3f} (target > noun {bn:.2f}, > random {br:.2f}).\n"
        rows.append({"combination": "Borda(MARS ensemble, pattern embedding)", "mc": {"accuracy": ba, "target_over_noun": bn, "target_over_random": br}})
        text += f"\n**Pattern-embedding control** (the v2 front end's topic-free pattern sentences compared with bge-small embeddings, no MARS; {cover:.1%} of stories have a pattern): multiple choice {a:.3f} (target > noun {vn:.2f}, > random {vr:.2f}); retrieval R@1 {r1:.3f}, R@5 {r5:.3f}, R@10 {r10:.3f}, MRR {mrr:.3f}.\n"
        rows.append({"control": "pattern embedding", "mc": {"accuracy": a, "target_over_noun": vn, "target_over_random": vr}, "retrieval": {"r1": r1, "r5": r5, "r10": r10, "mrr": mrr}})
    open(os.path.join(out, "E25.md"), "w").write("# E25: LLM front-end variants and ensembles\n\n" + text)
    json.dump(rows, open(os.path.join(out, "E25.json"), "w"), indent=1)
    print(text)


if __name__ == "__main__":
    main()
