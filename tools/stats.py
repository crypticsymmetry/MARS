#!/usr/bin/env python3
"""Statistics for headline comparisons: bootstrap confidence intervals and paired tests.

    python3 tools/stats.py [results/STATS.md]      # regenerate the report from results/*/…json

Library (also used by tools/e33_agent_memory.py):
* ci(values)            -> (mean, lo, hi): percentile bootstrap 95% CI of the mean;
* paired(a, b)          -> (mean diff a−b, lo, hi, p): paired bootstrap 95% CI of the
                           mean difference and a two-sided sign-flip permutation p-value;
* seeds(values)         -> (mean, sd) across independent runs.
All resampling is seeded (deterministic). Per-query outcomes come from the runners'
JSON (`ranks`, `rr`, `per_query`, …), so any pairing can be computed post hoc.
"""

import json
import os
import sys

import numpy as np

B = 10000


def ci(values, seed=0):
    x = np.asarray([v for v in values if v is not None], dtype=float)
    if len(x) == 0:
        return float("nan"), float("nan"), float("nan")
    rng = np.random.default_rng(seed)
    boots = x[rng.integers(0, len(x), (B, len(x)))].mean(axis=1)
    return float(x.mean()), float(np.percentile(boots, 2.5)), float(np.percentile(boots, 97.5))


def paired(a, b, seed=0):
    pairs = [(x, y) for x, y in zip(a, b) if x is not None and y is not None]
    d = np.asarray([x - y for x, y in pairs], dtype=float)
    if len(d) == 0:
        return float("nan"), float("nan"), float("nan"), float("nan"), 0
    rng = np.random.default_rng(seed)
    boots = d[rng.integers(0, len(d), (B, len(d)))].mean(axis=1)
    signs = rng.choice([-1.0, 1.0], (B, len(d)))
    null = np.abs((signs * d).mean(axis=1))
    p = (1 + np.sum(null >= abs(d.mean()) - 1e-12)) / (B + 1)
    return float(d.mean()), float(np.percentile(boots, 2.5)), float(np.percentile(boots, 97.5)), float(p), len(d)


def seeds(values):
    x = np.asarray(values, dtype=float)
    return float(x.mean()), float(x.std(ddof=1)) if len(x) > 1 else 0.0


# ---------------------------------------------------------------- report

def rr(rank):
    return 0.0 if rank is None else 1.0 / (rank + 1)


def hit(rank, k):
    return 1.0 if rank is not None and rank < k else 0.0


ROWS = []


def row(exp, comparison, metric, a, b, note=""):
    ma = ci(a)
    mb = ci(b)
    d = paired(a, b)
    ROWS.append({"exp": exp, "comparison": comparison, "metric": metric, "a": ma, "b": mb, "diff": d, "note": note})


def load(p):
    return json.load(open(p))


def e9_e12():
    for exp, path, n in (("E9", "results/E9/E9.json", "cross-author, 33 queries"), ("E12", "results/E12/E12.json", "cross-language, 52 queries")):
        m = {x["method"]: x["ranks"] for x in load(path)["task_a"]}
        best = "fused ½FAC+½FP-literal (exhaustive)"
        lex = "B2 lexical TF-IDF (identifiers + operations)"
        row(exp, f"MARS fused ½FAC+½FP-literal vs lexical TF-IDF ({n})", "MRR", [rr(r) for r in m[best]], [rr(r) for r in m[lex]])
        row(exp, f"MARS fused ½FAC+½FP-literal vs lexical TF-IDF ({n})", "R@10", [hit(r, 10) for r in m[best]], [hit(r, 10) for r in m[lex]])
        row(exp, f"MARS fused vs MAC content vectors ({n})", "MRR", [rr(r) for r in m[best]], [rr(r) for r in m["B4 MAC content vectors"]])


def e17():
    d = {x["analogue"]: x for x in load("results/E17/E17.json")["per_query"]}
    f, l = d["fused top-1"], d["lexical top-1 (baseline)"]
    row("E17", "fused top-1 vs lexical top-1 analogue (700 deleted-statement queries)", "exact recovery", [None if v is None else float(v) for v in f["exact"]], [None if v is None else float(v) for v in l["exact"]])
    row("E17", "fused top-1 vs lexical top-1 analogue", "best shape overlap", f["shape"], l["shape"])
    r = d["random analogue (chance)"]
    row("E17", "fused top-1 vs random analogue", "exact recovery", [None if v is None else float(v) for v in f["exact"]], [None if v is None else float(v) for v in r["exact"]])


def e23():
    per = {x["q"]: x for x in load("results/E23/E23-mars-nemotron.json")["per_question"]}
    llm = {json.loads(l)["q"]: json.loads(l)["choice"] for l in open("results/E23/llm-direct-glm-5.3-flash.jsonl")}
    qs = sorted(per)

    def correct(q, method):
        sc, a = per[q]["scores"][method], per[q]["answer"]
        return float(all(j == a or x < sc[a] for j, x in enumerate(sc)))
    mars = [correct(q, "analogy − 0.5·surface") for q in qs]
    surf = [correct(q, "surface channel only (C0)") for q in qs]
    direct = [None if q not in llm else float(llm[q] == per[q]["answer"]) for q in qs]
    row("E23", "MARS analogy − 0.5·surface vs surface channel only (StoryAnalogy MC, 360 q, Nemotron front end)", "accuracy", mars, surf)
    row("E23", "MARS analogy − 0.5·surface vs LLM direct (GLM-5.3-Flash)", "accuracy", mars, direct, "negative: the LLM is better")


def kg(exp, path, label):
    t = [x for x in load(path)["tables"] if x["m"] == max(y["m"] for y in load(path)["tables"])][0]
    m = {x["method"]: x["rr"] for x in t["methods"]}
    h1 = lambda v: [1.0 if x == 1.0 else 0.0 for x in v]
    row(exp, f"{label}: analogy vs copy, MARS neighbours", "Hits@1", h1(m["analogy · MARS fused neighbours"]), h1(m["copy · MARS fused neighbours"]))
    row(exp, f"{label}: analogy · hybrid vs rules ≤ 2 + copy · lexical", "Hits@1", h1(m["analogy · hybrid neighbours (RRF MARS + lexical)"]), h1(m["rules ≤ 2 + copy · lexical"]))
    row(exp, f"{label}: rules ≤ 2 + analogy · hybrid vs rules ≤ 2 + copy · lexical", "Hits@1", h1(m["rules ≤ 2 + analogy · hybrid"]), h1(m["rules ≤ 2 + copy · lexical"]))
    row(exp, f"{label}: gated vs ungated analogy, MARS neighbours", "Hits@1", h1(m["gated analogy · MARS fused (E29)"]), h1(m["analogy · MARS fused neighbours"]))


def stream(path):
    pq = load(path)["per_query"]
    return {x["q"]: x for x in pq}


def e30_32():
    for dom in ("scientists", "films"):
        s = stream(f"results/E30/E30-{dom}-hop2.json")
        qs = sorted(s)
        top = lambda q, i: 1.0 if s[q]["hit"][i] == 0 else 0.0
        row("E30", f"{dom} 2-hop stream: learned reliability vs raw ranking", "Hits@1", [top(q, 0) for q in qs], [top(q, 1) for q in qs])
        row("E31", f"{dom}: learned + induced rules vs learned", "Hits@1", [top(q, 2) for q in qs], [top(q, 0) for q in qs])
        best = "0.85" if dom == "scientists" else "0.5"
        a = stream(f"results/E32/E30-{dom}-identity-{best}.json")
        b = stream(f"results/E32/E30-{dom}-identity-0.json")
        row("E32", f"{dom}: identity channel λ = {best} vs none (learned ranking)", "Hits@1", [1.0 if a[q]["hit"][0] == 0 else 0.0 for q in qs], [1.0 if b[q]["hit"][0] == 0 else 0.0 for q in qs], "λ selected on this stream (optimistic)")


def e33():
    p = "results/E33/E33-seeds.json"
    if not os.path.exists(p):
        return
    d = load(p)
    for noise, runs in sorted(d["paired"].items()):
        a, b = runs["MARS (structure)"], runs["recall by names (identity only)"]
        row("E33", f"incident agent, noise {noise}: MARS vs recall by names (seed 1, known mechanisms)", "fix@1", a, b)


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "results/STATS.md"
    e9_e12()
    e17()
    e23()
    for exp, path, label in (("E27", "results/E29/E27-scientists-hop1.json", "scientists 1-hop"), ("E27", "results/E29/E27-films-hop1.json", "films 1-hop"), ("E28", "results/E29/E27-scientists-hop2.json", "scientists 2-hop"), ("E28", "results/E29/E27-films-hop2.json", "films 2-hop")):
        kg(exp, path, label)
    e30_32()
    e33()
    L = ["# Statistics for headline comparisons", "",
         "Generated by `python3 tools/stats.py` from the per-query outputs in `results/*/…json`. For each comparison: both means with 95% bootstrap CIs (10,000 resamples), the paired mean difference with its 95% bootstrap CI, and a two-sided sign-flip permutation p-value (10,000 permutations; seeded). Paired over the same queries. A difference is *significant* here when its CI excludes 0.", "",
         "| exp | comparison | metric | n | A: mean [95% CI] | B: mean [95% CI] | A − B [95% CI] | p | note |", "|---|---|---|---|---|---|---|---|---|"]
    for r in ROWS:
        a, b, (dm, lo, hi, p, n) = r["a"], r["b"], r["diff"]
        sig = "" if lo <= 0 <= hi else " **"
        L.append(f"| {r['exp']} | {r['comparison']} | {r['metric']} | {n} | {a[0]:.3f} [{a[1]:.3f}, {a[2]:.3f}] | {b[0]:.3f} [{b[1]:.3f}, {b[2]:.3f}] | {dm:+.3f} [{lo:+.3f}, {hi:+.3f}]{sig} | {p:.4f} | {r['note']} |")
    L += ["", "`**` = the CI of the difference excludes 0.", ""]
    if os.path.exists("results/E33/E33-seeds.json"):
        d = load("results/E33/E33-seeds.json")
        L += ["## E33 across seeds", "", f"fix@1 on known mechanisms, mean ± sd over seeds {d['seeds']} (independent mechanism sets, streams and noise):", "", "| noise | " + " | ".join(d["memories"]) + " |", "|---|" + "---|" * len(d["memories"])]
        for noise, v in sorted(d["across_seeds"].items()):
            L.append(f"| {noise} | " + " | ".join(f"{v[m][0]:.3f} ± {v[m][1]:.3f}" for m in d["memories"]) + " |")
        L.append("")
    L += ["## Not covered", "",
          "- E24/E25 retrieval fusion, E26 alignment counts and E33's LLM comparison (150 incidents, one seed; per-incident outcomes were not saved) have no paired statistics yet. The LLM comparison will be redone with per-incident outputs in the pre-registered evaluation.",
          "- Synthetic experiments (E0–E7, E10–E16, E20–E22) use N ≥ 500–10⁶ queries, so their sampling error is small (binomial SE ≤ 0.02 at N ≥ 500); their main threat is benchmark circularity, not sampling.", ""]
    open(out, "w").write("\n".join(L))
    json.dump(ROWS, open(out.replace(".md", ".json"), "w"), indent=1)
    print("\n".join(L))


if __name__ == "__main__":
    main()
