#!/usr/bin/env python3
"""E23 text baselines on the StoryAnalogy multiple-choice set, merged with
MARS scores into the final table.

    python3 tools/e23_baselines.py MC.json MARS_SCORES.json OUT_DIR [--llm MODELS] [--embed MODEL]

* lexical: TF-IDF cosine between source and choice texts;
* embedding: cosine of sentence embeddings (fastembed, default
  BAAI/bge-small-en-v1.5);
* LLM direct (optional, OpenRouter; $OPENROUTER_API_KEY): the model is asked
  which choice is the best analogy (same relational structure) — answers
  cached in OUT_DIR/llm_answers.jsonl;
* MARS rows are read from MARS_SCORES.json (mars-bench e23).
Metrics as in e23: accuracy, target > noun, target > random.
"""

import json
import math
import os
import re
import sys
from collections import Counter

sys.path.insert(0, os.path.dirname(__file__))


def tfidf_scores(mc):
    docs = [q["source"] for q in mc] + [c for q in mc for c in q["choices"]]
    tok = lambda s: re.findall(r"[a-z]+", s.lower())
    df = Counter(w for d in docs for w in set(tok(d)))
    n = len(docs)
    vec = lambda s: {w: c * (math.log((n + 1) / (df[w] + 1)) + 1) for w, c in Counter(tok(s)).items()}
    def cos(a, b):
        dot = sum(v * b.get(w, 0) for w, v in a.items())
        na, nb = math.sqrt(sum(v * v for v in a.values())), math.sqrt(sum(v * v for v in b.values()))
        return dot / (na * nb) if na and nb else 0.0
    return [[cos(vec(q["source"]), vec(c)) for c in q["choices"]] for q in mc]


def embed_scores(mc, model):
    from fastembed import TextEmbedding
    import numpy as np
    m = TextEmbedding(model_name=model)
    texts = [q["source"] for q in mc] + [c for q in mc for c in q["choices"]]
    e = np.array(list(m.embed(texts)))
    e /= np.linalg.norm(e, axis=1, keepdims=True)
    out, k = [], len(mc)
    for i, q in enumerate(mc):
        s = e[i]
        cs = [e[k + j] for j in range(len(q["choices"]))]
        k += len(q["choices"])
        out.append([float(s @ c) for c in cs])
    return out


LLM_PROMPT = """For each question, the source story is followed by candidate stories. Pick the candidate that is the best ANALOGY to the source: the one sharing the same relational/causal structure, even if its topic and entities differ. Answer with one line per question: '<question id>: <candidate number>'.

{items}"""


def llm_answers(mc, models, out_dir, batch=10):
    from llm2mars import call
    path = os.path.join(out_dir, "llm_answers.jsonl")
    done = {}
    if os.path.exists(path):
        for l in open(path):
            r = json.loads(l)
            done[r["q"]] = r
    todo = [i for i in range(len(mc)) if i not in done]
    with open(path, "a") as f:
        for b in range(0, len(todo), batch):
            ids = todo[b : b + batch]
            items = "\n\n".join(f"Question {i}\nSource: {mc[i]['source']}\n" + "\n".join(f"  {j}. {c}" for j, c in enumerate(mc[i]["choices"])) for i in ids)
            text, used = None, None
            for m in models:
                try:
                    text, used = call(m, LLM_PROMPT.format(items=items)), m
                    break
                except RuntimeError as e:
                    print(f"  {m}: {e}; next model", file=sys.stderr, flush=True)
            if text is None:
                print("  all models failed; stopping", file=sys.stderr, flush=True)
                break
            got = {int(a): int(c) for a, c in re.findall(r"(?:question\s*)?(\d+)\s*[:\-]\s*(\d+)", text, re.I)}
            for i in ids:
                r = {"q": i, "model": used, "choice": got.get(i)}
                done[i] = r
                f.write(json.dumps(r) + "\n")
            f.flush()
            print(f"  llm {b + len(ids)}/{len(todo)} via {used}", file=sys.stderr, flush=True)
    # One-hot scores; unanswered questions score 0 everywhere (counted wrong).
    return [[1.0 if done.get(i, {}).get("choice") == j else 0.0 for j in range(len(q["choices"]))] for i, q in enumerate(mc)]


def metrics(mc, scores, qids):
    acc, vn, vr = [], [], []
    for i in qids:
        q, sc = mc[i], scores[i]
        a = q["answer"]
        t = sc[a]
        acc.append(all(j == a or x < t for j, x in enumerate(sc)))
        for j, ty in enumerate(q["types"]):
            if j == a:
                continue
            (vn if ty == "noun" else vr).append(t > sc[j])
    m = lambda v: sum(v) / len(v) if v else float("nan")
    return m(acc), m(vn), m(vr)


def main():
    mc_path, mars_path, out_dir = sys.argv[1:4]
    args = sys.argv[4:]
    mc = json.load(open(mc_path))
    mars = json.load(open(mars_path))
    qids = [r["q"] for r in mars["per_question"]]
    os.makedirs(out_dir, exist_ok=True)
    rows = []
    rows.append(("lexical TF-IDF", tfidf_scores(mc)))
    emb = args[args.index("--embed") + 1] if "--embed" in args else "BAAI/bge-small-en-v1.5"
    rows.append((f"sentence embedding ({emb})", embed_scores(mc, emb)))
    if "--llm" in args:
        models = args[args.index("--llm") + 1].split(",")
        rows.append((f"LLM direct ({models[0].split('/')[-1]} …)", llm_answers(mc, models, out_dir)))
    per_q = {r["q"]: r for r in mars["per_question"]}
    for name in mars["per_question"][0]["scores"]:
        sc = [per_q[i]["scores"][name] if i in per_q else [0.0] * len(mc[i]["choices"]) for i in range(len(mc))]
        rows.append((f"MARS: {name}", sc))
    lines = ["| method | accuracy | target > noun | target > random |", "|---|---|---|---|"]
    js = []
    for name, sc in rows:
        a, n, r = metrics(mc, sc, qids)
        lines.append(f"| {name} | {a:.3f} | {n:.3f} | {r:.3f} |")
        js.append({"method": name, "accuracy": a, "target_over_noun": n, "target_over_random": r})
    table = "\n".join(lines)
    with open(os.path.join(out_dir, "E23.md"), "w") as f:
        f.write(f"# E23: StoryAnalogy multiple choice — MARS vs text baselines\n\n{len(qids)} questions (all five stories converted), chance accuracy 0.25.\n\n{table}\n")
    json.dump({"questions": len(qids), "rows": js}, open(os.path.join(out_dir, "E23.json"), "w"), indent=1)
    print(table)


if __name__ == "__main__":
    main()
