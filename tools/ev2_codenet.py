#!/usr/bin/env python3
"""EV2 (pre-registered, docs/PREREGISTRATION.md): code retrieval by algorithm on
Project CodeNet Python800.

    python3 tools/ev2_codenet.py prepare WORK [--problems 200] [--per 30] [--seed 1]
            [--frontend mars|pdg] [--exclude-seed S] [--same-as WORK0]   # E34 (pre-registration addendum A)
    mars-bench ev2 --data WORK                 # MARS rankings -> WORK/mars_rankings.json
    python3 tools/ev2_codenet.py evaluate WORK OUT_DIR

prepare: samples problems and solutions (seeded) from
data/external/codenet/Python800.tar.gz and converts every solution with the frozen
tools/py2mars.py encoder. CodeNet solutions are scripts, so an adapter wraps each
file's top-level statements (excluding function/class definitions and imports) into
one synthetic function; module functions are inlined one level, as py2mars does for
helpers. Solutions that do not parse or encode to fewer than 5 facts are skipped.

evaluate: every program is a query; relevant = other solutions of the same problem.
Methods: MARS fused (½FAC + ½FP-literal over the FP-literal top-100), MARS FP-literal,
lexical TF-IDF over Python tokens, a code embedding (fastembed), and RRF(MARS fused,
lexical). Metrics: MAP@R (primary), Hits@1, MRR; paired statistics via tools/stats.py.
"""

import ast
import io
import json
import os
import random
import sys
import tokenize
from collections import defaultdict

sys.path.insert(0, os.path.dirname(__file__))
import py2mars  # noqa: E402  (frozen front end)
import py2pdg  # noqa: E402  (E34 dataflow front end)

TAR = "data/external/codenet/Python800.tar.gz"  # extracted once: tar -xzf … -C data/external/codenet
ROOT = "data/external/codenet/Project_CodeNet_Python800"


def encode_script(src):
    tree = ast.parse(src)
    module_funcs = {n.name: n for n in tree.body if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef))}
    body = [n for n in tree.body if not isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Import, ast.ImportFrom))]
    fn = ast.FunctionDef(name="solution", args=ast.arguments(posonlyargs=[], args=[], vararg=None, kwonlyargs=[], kw_defaults=[], kwarg=None, defaults=[]), body=body or [ast.Pass()], decorator_list=[], returns=None, type_params=[])
    return py2mars.FuncEncoder("solution", module_funcs).encode(fn, py2mars.INLINE)


def prepare(work, n_problems, per, seed, frontend="mars", exclude_seed=0, same_as=""):
    os.makedirs(work, exist_ok=True)
    root = ROOT
    rng = random.Random(seed)
    problems = sorted(d for d in os.listdir(root) if os.path.isdir(os.path.join(root, d)))
    if exclude_seed:  # E34 development sample: problems outside the sample drawn with exclude_seed (EV2's = 1)
        held = set(random.Random(exclude_seed).sample(problems, 200))  # EV2 drew 200 problems
        problems = [p for p in problems if p not in held]
    names_used = set()
    chosen = sorted(rng.sample(problems, n_problems))
    fixed = None
    if same_as:  # re-encode exactly the programs of another prepared sample (paired front-end comparison)
        m0 = json.load(open(f"{same_as}/manifest.json"))
        chosen, per, seed = m0["problems"], m0["per"], m0["seed"]
        fixed = defaultdict(list)
        for m in m0["cases"]:
            fixed[m["problem"]].append(m["file"].split("/", 1)[1])
    cases, manifest, sources = [], [], []
    skipped = 0
    for p in chosen:
        files = sorted(f for f in os.listdir(os.path.join(root, p)) if f.endswith(".py"))
        rng.shuffle(files)
        if fixed is not None:
            files = fixed[p]
        taken = 0
        for fn in files:
            if taken == per:
                break
            src = open(os.path.join(root, p, fn), encoding="utf-8", errors="replace").read()
            try:
                if frontend == "pdg":
                    facts, nm = py2pdg.encode_script(src)
                else:
                    facts, nm = encode_script(src), set()
            except (SyntaxError, ValueError, RecursionError, KeyError, AttributeError, TypeError):
                skipped += 1
                continue
            if len(facts) < 5:
                skipped += 1
                continue
            name = py2mars.ident(f"{p}_{fn[:-3]}")
            cases.append(f"(defcase {name}\n  " + "\n  ".join(facts) + ")")
            manifest.append({"case": name, "problem": p, "file": f"{p}/{fn}", "n_facts": len(facts)})
            sources.append({"case": name, "src": src})
            names_used |= nm
            taken += 1
    with open(f"{work}/vocab.mars", "w") as f:
        if frontend == "pdg":
            f.write(py2pdg.vocab())
            for a in sorted(names_used):
                f.write(f"(defpredicate {a} :arity 1 :kind attribute)\n")
        else:
            f.write(py2mars.VOCAB)
            for p in py2mars.EXTRA_PREDS:
                f.write(f"(defpredicate {p} :arity * :kind function)\n")
    with open(f"{work}/cases.mars", "w") as f:
        f.write(f";; tools/ev2_codenet.py prepare ({'py2pdg' if frontend == 'pdg' else 'py2mars'}, script adapter)\n")
        if frontend != "pdg":
            for rp in sorted(py2mars.RAW_USED):
                f.write(f"(defpredicate {rp} :arity * :kind relation :canonical nil)\n")
        f.write("\n".join(cases) + "\n")
    json.dump({"seed": seed, "frontend": frontend, "exclude_seed": exclude_seed, "same_as": same_as, "problems": chosen, "per": per, "skipped": skipped, "cases": manifest}, open(f"{work}/manifest.json", "w"), indent=0)
    with open(f"{work}/sources.jsonl", "w") as f:
        for s in sources:
            f.write(json.dumps(s) + "\n")
    print(f"{len(manifest)} programs from {len(chosen)} problems ({skipped} skipped)", file=sys.stderr)


def py_tokens(src):
    out = []
    try:
        for t in tokenize.generate_tokens(io.StringIO(src).readline):
            if t.type in (tokenize.NAME, tokenize.OP, tokenize.NUMBER):
                out.append(t.string)
            elif t.type == tokenize.STRING:
                out.append("<str>")
    except (tokenize.TokenError, IndentationError, SyntaxError):
        out += src.split()
    return out


def metrics(ranked, rel_set):
    """(AP@R, hit@1, RR) of a ranked list of case names; R = |rel_set|."""
    R = len(rel_set)
    hits, ap = 0, 0.0
    for i, c in enumerate(ranked[:R]):
        if c in rel_set:
            hits += 1
            ap += hits / (i + 1)
    ap /= max(R, 1)
    first = next((i for i, c in enumerate(ranked) if c in rel_set), None)
    return ap, float(first == 0), 0.0 if first is None else 1.0 / (first + 1)


def rrf(lists, k=60):
    sc = defaultdict(float)
    for l in lists:
        for p, c in enumerate(l):
            sc[c] += 1.0 / (k + p)
    return [c for c, _ in sorted(sc.items(), key=lambda t: (-t[1], t[0]))]


def evaluate(work, out_dir):
    import numpy as np
    from stats import ci, paired
    man = json.load(open(f"{work}/manifest.json"))
    cases = [m["case"] for m in man["cases"]]
    prob = {m["case"]: m["problem"] for m in man["cases"]}
    group = defaultdict(set)
    for c in cases:
        group[prob[c]].add(c)
    src = {json.loads(l)["case"]: json.loads(l)["src"] for l in open(f"{work}/sources.jsonl")}
    mars = json.load(open(f"{work}/mars_rankings.json"))
    depth = 100
    rankings = {"MARS fused (½FAC + ½FP-literal, FP top-100)": {c: mars["fused"][c] for c in cases}, "MARS FP-literal": {c: mars["fp"][c] for c in cases}}
    # Lexical TF-IDF over Python tokens.
    from collections import Counter
    toks = {c: Counter(py_tokens(src[c])) for c in cases}
    df = Counter(t for c in cases for t in toks[c])
    N = len(cases)
    vocab = {t: i for i, t in enumerate(df)}
    import scipy.sparse as sp
    rows, cols, vals = [], [], []
    for i, c in enumerate(cases):
        for t, n in toks[c].items():
            rows.append(i)
            cols.append(vocab[t])
            vals.append((1 + np.log(n)) * (np.log((N + 1) / (df[t] + 1)) + 1))
    X = sp.csr_matrix((vals, (rows, cols)), shape=(N, len(vocab)))
    X = sp.diags(1 / np.sqrt(X.multiply(X).sum(axis=1).A.ravel() + 1e-12)) @ X
    S = (X @ X.T).toarray()
    np.fill_diagonal(S, -np.inf)
    order = np.argsort(-S, axis=1, kind="stable")[:, :depth]
    rankings["lexical TF-IDF (Python tokens)"] = {c: [cases[j] for j in order[i]] for i, c in enumerate(cases)}
    # Code embedding.
    from fastembed import TextEmbedding
    avail = {m["model"] for m in TextEmbedding.list_supported_models()}
    model = "jinaai/jina-embeddings-v2-base-code" if "jinaai/jina-embeddings-v2-base-code" in avail else "BAAI/bge-small-en-v1.5"
    # Inputs capped at 2,000 characters, batches of 4 (8,000 / 32 ran out of memory; deviation recorded in results/EV2).
    E = np.array(list(TextEmbedding(model_name=model).embed([src[c][:2000] for c in cases], batch_size=4)))
    E /= np.linalg.norm(E, axis=1, keepdims=True)
    S = E @ E.T
    np.fill_diagonal(S, -np.inf)
    order = np.argsort(-S, axis=1, kind="stable")[:, :depth]
    rankings[f"code embedding ({model})"] = {c: [cases[j] for j in order[i]] for i, c in enumerate(cases)}
    fused, lexn = "MARS fused (½FAC + ½FP-literal, FP top-100)", "lexical TF-IDF (Python tokens)"
    rankings["RRF(MARS fused, lexical)"] = {c: rrf([rankings[fused][c], rankings[lexn][c]]) for c in cases}
    per = {}
    for name, rk in rankings.items():
        per[name] = [metrics(rk[c], group[prob[c]] - {c}) for c in cases]
    os.makedirs(out_dir, exist_ok=True)
    L = [f"# EV2: code retrieval by algorithm (CodeNet Python800)\n", f"{N} programs from {len(man['problems'])} problems (seed {man['seed']}; ≤ {man['per']} per problem; {man['skipped']} unparseable or < 5 facts skipped). Every program is a query; relevant = other solutions of the same problem (R = group size − 1). Rankings to depth {depth}.\n",
         "| method | MAP@R [95% CI] | Hits@1 [95% CI] | MRR [95% CI] |", "|---|---|---|---|"]
    res = {}
    for name, v in per.items():
        a, h, r = ci([x[0] for x in v]), ci([x[1] for x in v]), ci([x[2] for x in v])
        L.append(f"| {name} | {a[0]:.4f} [{a[1]:.4f}, {a[2]:.4f}] | {h[0]:.4f} [{h[1]:.4f}, {h[2]:.4f}] | {r[0]:.4f} [{r[1]:.4f}, {r[2]:.4f}] |")
        res[name] = {"map_at_r": a, "hits1": h, "mrr": r}
    emb = next(n for n in per if n.startswith("code embedding"))
    L += ["", "**Pre-registered paired comparisons** (MAP@R, paired over queries):", "", "| comparison | A − B [95% CI] | p |", "|---|---|---|"]
    comps = [("B2", fused, lexn), ("B3", fused, emb), ("P8a", "RRF(MARS fused, lexical)", fused), ("P8b", "RRF(MARS fused, lexical)", lexn)]
    pc = {}
    for tag, a, b in comps:
        d = paired([x[0] for x in per[a]], [x[0] for x in per[b]])
        L.append(f"| {tag}: {a} − {b} | {d[0]:+.4f} [{d[1]:+.4f}, {d[2]:+.4f}] | {d[3]:.4f} |")
        pc[tag] = {"a": a, "b": b, "diff": d}
    open(f"{out_dir}/EV2.md", "w").write("\n".join(L) + "\n")
    json.dump({"config": {"seed": man["seed"], "problems": len(man["problems"]), "per": man["per"], "programs": N, "skipped": man["skipped"], "depth": depth, "embedding_model": model, "tar_sha256_prefix": "39297d11df8030ce"}, "results": res, "paired": pc,
               "per_query": {name: [list(x) for x in v] for name, v in per.items()}, "cases": cases}, open(f"{out_dir}/EV2.json", "w"))
    print("\n".join(L))


if __name__ == "__main__":
    cmd = sys.argv[1]
    arg = lambda k, d: type(d)(sys.argv[sys.argv.index(k) + 1]) if k in sys.argv else d
    if cmd == "prepare":
        prepare(sys.argv[2], arg("--problems", 200), arg("--per", 30), arg("--seed", 1), arg("--frontend", "mars"), arg("--exclude-seed", 0), arg("--same-as", ""))
    elif cmd == "evaluate":
        evaluate(sys.argv[2], sys.argv[3])
