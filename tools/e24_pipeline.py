#!/usr/bin/env python3
"""E24 pipeline: story-analogy retrieval over the pooled StoryAnalogy memory,
text retrievers vs MARS, and an LLM verifier on each retriever's top-k.

    python3 tools/e24_pipeline.py MC.json E24-MARS.json OUT_DIR [--llm MODEL] [--k 5] [--retrievers "a;b"]

* retrieval: lexical TF-IDF and sentence embeddings (fastembed) over the same
  memory; MARS rankings come from `mars-bench e24`;
* verification (optional): the LLM sees the query story and a retriever's
  top-k (shuffled deterministically) and picks the best analogy; end-to-end
  accuracy = the analogy was picked. Answers cached in OUT_DIR.
"""

import json
import math
import os
import random
import re
import sys
from collections import Counter

sys.path.insert(0, os.path.dirname(__file__))


def memory(mc):
    names, texts = [], []
    for i, q in enumerate(mc):
        names.append(f"q{i}-s")
        texts.append(q["source"])
        for j, c in enumerate(q["choices"]):
            names.append(f"q{i}-c{j}")
            texts.append(c)
    return names, texts


def tfidf_rank(texts, qidx):
    tok = lambda s: re.findall(r"[a-z]+", s.lower())
    df = Counter(w for d in texts for w in set(tok(d)))
    n = len(texts)
    vecs = []
    for d in texts:
        v = {w: c * (math.log((n + 1) / (df[w] + 1)) + 1) for w, c in Counter(tok(d)).items()}
        norm = math.sqrt(sum(x * x for x in v.values())) or 1.0
        vecs.append({w: x / norm for w, x in v.items()})
    out = {}
    for qi in qidx:
        q = vecs[qi]
        sc = [(sum(x * vecs[j].get(w, 0) for w, x in q.items()), j) for j in range(n) if j != qi]
        sc.sort(key=lambda t: (-t[0], t[1]))
        out[qi] = [j for _, j in sc]
    return out


def embed_rank(texts, qidx, model="BAAI/bge-small-en-v1.5"):
    from fastembed import TextEmbedding
    import numpy as np
    e = np.array(list(TextEmbedding(model_name=model).embed(texts)))
    e /= np.linalg.norm(e, axis=1, keepdims=True)
    out = {}
    for qi in qidx:
        s = e @ e[qi]
        s[qi] = -9
        out[qi] = list(np.argsort(-s, kind="stable"))
    return out


PROMPT = """Source story: {src}

Which candidate is the best ANALOGY to the source story — the one sharing the same relational/causal structure, even if its topic and entities differ? If several share the topic, prefer the one whose structure matches, not the one with the same entities.

{cands}

Answer with the candidate number only."""


def _verify_one(job):
    """One verifier call (runs in a worker process: the deadline uses SIGALRM)."""
    from llm2mars import call
    i, src, order, cand_texts, model = job
    text = "\n".join(f"{j + 1}. {t}" for j, t in enumerate(cand_texts))
    try:
        ans = call(model, PROMPT.format(src=src, cands=text))
        m = re.search(r"\d+", ans)
        return i, (order[int(m.group()) - 1] if m and 1 <= int(m.group()) <= len(order) else None)
    except RuntimeError:
        return i, None


def verify(mc, names, texts, rankings, k, model, out_dir, tag, workers=8):
    from multiprocessing import Pool
    path = os.path.join(out_dir, f"verify-{tag}.jsonl")
    done = {}
    if os.path.exists(path):
        for l in open(path):
            r = json.loads(l)
            done[r["q"]] = r
    jobs = []
    for i, q in enumerate(mc):
        if i in done:
            continue
        order = rankings[i][:k]
        order = order[:]
        random.Random(i).shuffle(order)  # the verifier must not see the retriever's order
        jobs.append((i, q["source"], order, [texts[c] for c in order], model))
    with open(path, "a") as f, Pool(workers) as pool:
        for n, (i, pick) in enumerate(pool.imap_unordered(_verify_one, jobs), 1):
            r = {"q": i, "pick": None if pick is None else names[pick]}
            done[i] = r
            f.write(json.dumps(r) + "\n")
            f.flush()
            if n % 40 == 0:
                print(f"  verify {tag}: {n}/{len(jobs)}", file=sys.stderr, flush=True)
    return done


def main():
    mc_path, mars_path, out_dir = sys.argv[1:4]
    args = sys.argv[4:]
    if "--reasoning" in args:  # reasoning effort for reasoning models (inherited by worker processes)
        import llm2mars
        llm2mars.REASONING = args[args.index("--reasoning") + 1]
    k = int(args[args.index("--k") + 1]) if "--k" in args else 10
    mc = json.load(open(mc_path))
    mars = json.load(open(mars_path))
    names, texts = memory(mc)
    pos = {n: i for i, n in enumerate(names)}
    qidx = [pos[f"q{i}-s"] for i in range(len(mc))]
    target = {i: pos[f"q{i}-c{q['answer']}"] for i, q in enumerate(mc)}
    noun = {i: pos[f"q{i}-c{q['types'].index('noun')}"] for i, q in enumerate(mc)}
    os.makedirs(out_dir, exist_ok=True)
    ranks = {}
    lex = tfidf_rank(texts, qidx)
    ranks["lexical TF-IDF"] = {i: lex[qidx[i]] for i in range(len(mc))}
    emb = embed_rank(texts, qidx)
    ranks["sentence embedding (bge-small)"] = {i: emb[qidx[i]] for i in range(len(mc))}
    for r in mars["per_query"]:
        for name, lst in r["rankings"].items():
            ranks.setdefault(name, {})[r["q"]] = [pos[n] for n in lst]
    # Rank fusion (RRF, k = 60) of the MARS fused ranking and lexical TF-IDF.
    mname = "MARS fused FAC + fingerprint analogy"
    if mname in ranks:
        fused = {}
        for i in range(len(mc)):
            sc = Counter()
            for lst in (ranks[mname][i][:50], ranks["lexical TF-IDF"][i][:50]):
                for p, c in enumerate(lst):
                    sc[c] += 1.0 / (60 + p)
            fused[i] = [c for c, _ in sorted(sc.items(), key=lambda t: (-t[1], t[0]))]
        ranks["RRF(MARS fused, lexical)"] = fused
    lines = ["| retriever | R@1 | R@5 | R@10 | MRR (top-50) | analogy above its noun distractor |", "|---|---|---|---|---|---|"]
    rows = []
    for name, rk in ranks.items():
        rr = []
        above = []
        for i in range(len(mc)):
            lst = rk[i][:50]
            p = lst.index(target[i]) if target[i] in lst else None
            rr.append(p)
            pn = lst.index(noun[i]) if noun[i] in lst else None
            above.append(p is not None and (pn is None or p < pn))
        rec = lambda kk: sum(1 for p in rr if p is not None and p < kk) / len(rr)
        mrr = sum(1 / (p + 1) for p in rr if p is not None) / len(rr)
        ab = sum(above) / len(above)
        lines.append(f"| {name} | {rec(1):.3f} | {rec(5):.3f} | {rec(10):.3f} | {mrr:.3f} | {ab:.3f} |")
        rows.append({"retriever": name, "r1": rec(1), "r5": rec(5), "r10": rec(10), "mrr50": mrr, "analogy_above_noun": ab})
    out = "## Retrieval over the pooled memory\n\n" + "\n".join(lines) + "\n"
    ver_rows = []
    if "--llm" in args:
        model = args[args.index("--llm") + 1]
        which = args[args.index("--retrievers") + 1].split(";") if "--retrievers" in args else list(ranks)
        vl = [f"\n## LLM verifier ({model}) on each retriever's top-{k}\n", "| retriever | analogy in top-k | verifier picks the analogy (end-to-end) | picks the noun distractor |", "|---|---|---|---|"]
        for name in which:
            tag = re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")
            d = verify(mc, names, texts, ranks[name], k, model, out_dir, tag)
            inside = sum(1 for i in range(len(mc)) if target[i] in ranks[name][i][:k]) / len(mc)
            ok = sum(1 for i in range(len(mc)) if d[i]["pick"] == names[target[i]]) / len(mc)
            nn = sum(1 for i in range(len(mc)) if d[i]["pick"] == names[noun[i]]) / len(mc)
            vl.append(f"| {name} | {inside:.3f} | {ok:.3f} | {nn:.3f} |")
            ver_rows.append({"retriever": name, "in_topk": inside, "end_to_end": ok, "picks_noun": nn})
        out += "\n".join(vl) + "\n"
    with open(os.path.join(out_dir, "E24.md"), "w") as f:
        f.write(f"# E24: story-analogy retrieval + LLM verification\n\nMemory: {len(names)} stories; {len(mc)} queries (each source must find its analogy).\n\n{out}")
    json.dump({"retrieval": rows, "verification": ver_rows, "k": k}, open(os.path.join(out_dir, "E24.json"), "w"), indent=1)
    print(out)


if __name__ == "__main__":
    main()
