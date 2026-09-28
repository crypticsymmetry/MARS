#!/usr/bin/env python3
"""Score the pre-registered predictions P1–P9 (docs/PREREGISTRATION.md) from the
EV1/EV2/EV3 result files. Written and committed *before* any EV result was seen, so
the operational criteria below are fixed in advance.

    python3 tools/ev_report.py [results/EV_PREDICTIONS.md]

Operational criteria (paired bootstrap 95% CIs via tools/stats.py, seeded):
* "A > B": confirmed if the CI of A − B lies entirely above 0; not confirmed if
  entirely below 0; inconclusive otherwise.
* "A ≥ B": confirmed if the point estimate of A − B is ≥ 0 and its CI is not entirely
  below 0; not confirmed if the CI is entirely below 0; inconclusive otherwise
  (point estimate < 0 but CI includes 0).
* P1 "MARS below RotatE and TuckER": MARS's MRR CI upper bound < the published MRR
  of both, on both datasets.
* P3 "much higher within 2 hops": Hits@1 within 2 hops ≥ 2 × Hits@1 farther, and the
  bootstrap CI of the difference excludes 0.
* P5 "relatively closer on WN18RR": MARS MRR / RotatE MRR is higher on WN18RR than on
  FB15k-237 (point estimates; reported with both ratios).
"""

import json
import os
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(__file__))
from stats import ci, paired  # noqa: E402

PUBLISHED = {"FB15k-237": {"RotatE": 0.338, "TuckER": 0.358}, "WN18RR": {"RotatE": 0.476, "TuckER": 0.470}}


def verdict_gt(d):
    lo, hi = d[1], d[2]
    return "confirmed" if lo > 0 else ("not confirmed" if hi < 0 else "inconclusive")


def verdict_ge(d):
    m, lo, hi = d[0], d[1], d[2]
    if hi < 0:
        return "not confirmed"
    return "confirmed" if m >= 0 else "inconclusive"


def fmt(d):
    return f"{d[0]:+.4f} [{d[1]:+.4f}, {d[2]:+.4f}]"


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "results/EV_PREDICTIONS.md"
    L = ["# Pre-registered predictions: scored", "", "Criteria fixed in `tools/ev_report.py` before any result was seen (see its docstring). Paired bootstrap 95% CIs.", "", "| prediction | evidence | verdict |", "|---|---|---|"]
    rec = {}
    ev1 = {}
    for ds in ("FB15k-237", "WN18RR"):
        p = f"results/EV1/EV1-{ds}-test.json"
        if os.path.exists(p):
            ev1[ds] = json.load(open(p))
    if ev1:
        # P1
        ok, ev = True, []
        for ds, d in ev1.items():
            rr = [1 / x["rank_learned"] for x in d["per_query"]]
            m = ci(rr)
            ev.append(f"{ds}: MARS MRR {m[0]:.3f} [{m[1]:.3f}, {m[2]:.3f}] vs RotatE {PUBLISHED[ds]['RotatE']}, TuckER {PUBLISHED[ds]['TuckER']}")
            ok &= m[2] < min(PUBLISHED[ds].values())
        v = "confirmed" if ok and len(ev1) == 2 else ("not confirmed" if not ok else "inconclusive (one dataset)")
        L.append(f"| P1 MARS below RotatE and TuckER (MRR) | {'; '.join(ev)} | {v} |")
        rec["P1"] = v
        # P2, P3, P4
        for ds, d in ev1.items():
            q = d["per_query"]
            d2 = paired([1 / x["rank_learned"] for x in q], [1 / x["rank_pop"] for x in q])
            v2 = verdict_gt(d2)
            L.append(f"| P2 MARS > relation popularity ({ds}, MRR) | Δ {fmt(d2)} | {v2} |")
            rec[f"P2-{ds}"] = v2
            near = [float(x["rank_learned"] <= 1) for x in q if x["within2"]]
            far = [float(x["rank_learned"] <= 1) for x in q if not x["within2"]]
            a, b = ci(near), ci(far)
            rng = np.random.default_rng(0)
            nb, fb = np.array(near), np.array(far)
            diffs = [nb[rng.integers(0, len(nb), len(nb))].mean() - fb[rng.integers(0, len(fb), len(fb))].mean() for _ in range(5000)] if len(nb) and len(fb) else [0.0]
            lo, hi = np.percentile(diffs, 2.5), np.percentile(diffs, 97.5)
            v3 = "confirmed" if (a[0] >= 2 * b[0] and lo > 0) else ("not confirmed" if hi <= 0 or a[0] < b[0] else "inconclusive")
            L.append(f"| P3 Hits@1 within 2 hops ≫ farther ({ds}) | {a[0]:.3f} (n {len(near)}) vs {b[0]:.3f} (n {len(far)}); Δ CI [{lo:+.3f}, {hi:+.3f}] | {v3} |")
            rec[f"P3-{ds}"] = v3
            d4 = paired([1 / x["rank_learned"] for x in q], [1 / x["rank_raw"] for x in q])
            v4 = verdict_ge(d4)
            L.append(f"| P4 learned ≥ raw ({ds}, MRR) | Δ {fmt(d4)} | {v4} |")
            rec[f"P4-{ds}"] = v4
        if len(ev1) == 2:
            ratio = {ds: np.mean([1 / x["rank_learned"] for x in d["per_query"]]) / PUBLISHED[ds]["RotatE"] for ds, d in ev1.items()}
            v5 = "confirmed" if ratio["WN18RR"] > ratio["FB15k-237"] else "not confirmed"
            L.append(f"| P5 relatively closer to KGE on WN18RR | MARS/RotatE MRR ratio: WN18RR {ratio['WN18RR']:.2f}, FB15k-237 {ratio['FB15k-237']:.2f} | {v5} |")
            rec["P5"] = v5
    if os.path.exists("results/EV2/EV2.json"):
        d = json.load(open("results/EV2/EV2.json"))
        pq = d["per_query"]
        fused = next(n for n in pq if n.startswith("MARS fused"))
        lex = next(n for n in pq if n.startswith("lexical"))
        emb = next(n for n in pq if n.startswith("code embedding"))
        rrf = next(n for n in pq if n.startswith("RRF"))
        ap = lambda n: [x[0] for x in pq[n]]
        d6 = paired(ap(fused), ap(lex))
        L.append(f"| P6 MARS fused > lexical (MAP@R) | Δ {fmt(d6)} | {verdict_gt(d6)} |")
        d7 = paired(ap(emb), ap(fused))
        L.append(f"| P7 code embedding ≥ MARS fused (MAP@R) | Δ {fmt(d7)} | {verdict_ge(d7)} |")
        d8a, d8b = paired(ap(rrf), ap(fused)), paired(ap(rrf), ap(lex))
        v8 = "confirmed" if verdict_ge(d8a) == "confirmed" and verdict_ge(d8b) == "confirmed" else ("not confirmed" if "not confirmed" in (verdict_ge(d8a), verdict_ge(d8b)) else "inconclusive")
        L.append(f"| P8 RRF(MARS, lexical) ≥ both (MAP@R) | vs MARS {fmt(d8a)}; vs lexical {fmt(d8b)} | {v8} |")
        rec.update({"P6": verdict_gt(d6), "P7": verdict_ge(d7), "P8": v8})
    if os.path.exists("results/EV3/EV3.json"):
        d = json.load(open("results/EV3/EV3.json"))
        mem = d["memories"]
        m, e = mem["MARS (structure)"]["raw_per_seed"], mem["embedding RAG (bge-small)"]["raw_per_seed"]
        dseed = paired(m, e)
        parts = [f"memory (raw fix@1, per-seed paired over {len(m)} seeds): Δ {fmt(dseed)}"]
        v9a = verdict_gt(dseed) if len(m) > 2 else verdict_gt(d["paired_mars_vs_embedding_seed1"])
        v9b = "not run"
        if "llm" in d:
            pl = d["llm"]["per_incident"]
            dl = paired(pl["MARS (structure)"], pl["embedding RAG (bge-small)"])
            parts.append(f"LLM + MARS vs LLM + embedding: Δ {fmt(dl)}")
            v9b = verdict_gt(dl)
        v9 = "confirmed" if v9a == v9b == "confirmed" else ("not confirmed" if "not confirmed" in (v9a, v9b) else "inconclusive")
        L.append(f"| P9 MARS > embedding RAG, as memory and as LLM retriever | {'; '.join(parts)} | {v9} (memory: {v9a}; LLM: {v9b}) |")
        rec["P9"] = v9
    L.append("")
    open(out, "w").write("\n".join(L) + "\n")
    json.dump(rec, open(out.replace(".md", ".json"), "w"), indent=1)
    print("\n".join(L))


if __name__ == "__main__":
    main()
