#!/usr/bin/env python3
"""E36 (pre-registration addendum B): learned fusion (A3) and conformal selective
top-1 retrieval (A4) for code retrieval, on the code embedding's top-100 with MARS
FAC features from E34's py2pdg front end.

    python3 tools/e36_fusion.py DEV_MARS DEV_PDG TEST_MARS TEST_PDG OUT [--dev-only]

Each *_PDG work directory must hold emb.npy/emb_cases.json/emb_cands.json and
fac_cands.json (tools/e34_eval.py cands + mars-bench ev2 --cands). The ranker and
the conformal threshold are fit on DEV only and applied unchanged to TEST.
With --dev-only, only development numbers are produced (the test sample is not read).
"""

import json
import os
import sys
from collections import defaultdict

import numpy as np
from scipy.stats import binom

sys.path.insert(0, os.path.dirname(__file__))
from ev2_codenet import metrics  # noqa: E402
from stats import ci, paired  # noqa: E402

ALPHA, DELTA, GRID, C = 0.05, 0.1, 200, 1.0
FEATURES = ["cos", "fac", "cos_rank", "fac_rank", "q_top1", "q_margin", "q_maxfac", "cos*q_top1", "fac*q_top1"]


def load(w_mars, w_pdg):
    man = json.load(open(f"{w_mars}/manifest.json"))
    cases = [m["case"] for m in man["cases"]]
    prob = {m["case"]: m["problem"] for m in man["cases"]}
    group = defaultdict(set)
    for c in cases:
        group[prob[c]].add(c)
    assert json.load(open(f"{w_pdg}/emb_cases.json")) == cases
    E = np.load(f"{w_pdg}/emb.npy")
    top = json.load(open(f"{w_pdg}/emb_cands.json"))
    fac = json.load(open(f"{w_pdg}/fac_cands.json"))
    pos = {c: i for i, c in enumerate(cases)}
    q = []
    for i, c in enumerate(cases):
        keep = [j for j, x in enumerate(top[c]) if x != c]  # only in tiny samples can the query itself reach its top-100
        cand = [top[c][j] for j in keep]
        cos = np.array([float(E[i] @ E[pos[x]]) for x in cand])
        f0 = fac.get(c) or [0.0] * len(top[c])
        f = np.array([f0[j] for j in keep])
        rel = np.array([x in group[prob[c]] and x != c for x in cand], dtype=float)
        q.append({"case": c, "cand": cand, "cos": cos, "fac": f, "rel": rel, "relset": group[prob[c]] - {c}})
    return cases, q


def feats(r):
    cos, fac = r["cos"], r["fac"]
    n = len(cos)
    cos_rank = np.argsort(np.argsort(-cos, kind="stable"), kind="stable") / n
    fac_rank = np.argsort(np.argsort(-fac, kind="stable"), kind="stable") / n
    top1 = cos.max()
    s = np.sort(cos)[::-1]
    margin = s[0] - s[1] if n > 1 else 0.0
    maxfac = fac.max()
    one = np.ones(n)
    return np.column_stack([cos, fac, cos_rank, fac_rank, top1 * one, margin * one, maxfac * one, cos * top1, fac * top1])


def fit_logistic(X, y, C=1.0, iters=50):
    """sklearn-equivalent L2 logistic regression (0.5||w||^2 + C sum logloss; intercept unpenalized), Newton's method."""
    Xb = np.column_stack([X, np.ones(len(X))])
    w = np.zeros(Xb.shape[1])
    reg = np.ones(Xb.shape[1]) / C
    reg[-1] = 0.0
    for _ in range(iters):
        p = 1 / (1 + np.exp(-(Xb @ w)))
        g = Xb.T @ (p - y) + reg * w
        H = (Xb * (p * (1 - p))[:, None]).T @ Xb + np.diag(reg + 1e-9)
        step = np.linalg.solve(H, g)
        w -= step
        if np.abs(step).max() < 1e-8:
            break
    return w


def predict(w, X, mu, sd):
    Xb = np.column_stack([(X - mu) / sd, np.ones(len(X))])
    return 1 / (1 + np.exp(-(Xb @ w)))


def ltt_threshold(conf, correct, alpha=ALPHA, delta=DELTA, grid=GRID):
    """Learn-then-Test, fixed-sequence over thresholds from high to low confidence:
    H0(λ): error rate among answered (conf ≥ λ) > alpha; exact binomial p-value.
    Returns the lowest threshold whose null (and all before it) is rejected at delta.
    The sequence starts at the first threshold with enough answered queries for a
    zero-error sample to be significant, n ≥ ln δ / ln(1 − α) (amendment to addendum B:
    this depends on confidences only, never on correctness, so validity is kept)."""
    n_min = int(np.ceil(np.log(delta) / np.log(1 - alpha)))
    lams = np.unique(np.quantile(conf, np.linspace(1, 0, grid)))[::-1]
    chosen = np.inf
    for lam in lams:
        ans = conf >= lam
        n = int(ans.sum())
        if n < n_min:
            continue
        err = int((~correct[ans]).sum())
        pval = binom.cdf(err, n, alpha)
        if pval <= delta:
            chosen = lam
        else:
            break
    return float(chosen)


def rankings(q, w=None, mu=None, sd=None):
    out = {"code embedding": {}, "fixed ½cos + ½FAC (E34)": {}, "learned fusion (A3)": {}}
    conf_l, conf_e, top_l, top_e = [], [], [], []
    for r in q:
        cand = r["cand"]
        out["code embedding"][r["case"]] = cand
        fx = 0.5 * r["cos"] + 0.5 * r["fac"]
        out["fixed ½cos + ½FAC (E34)"][r["case"]] = [cand[i] for i in sorted(range(len(cand)), key=lambda i: (-fx[i], cand[i]))]
        if w is not None:
            p = predict(w, feats(r), mu, sd)
            order = sorted(range(len(cand)), key=lambda i: (-p[i], cand[i]))
            out["learned fusion (A3)"][r["case"]] = [cand[i] for i in order]
            conf_l.append(p[order[0]])
            top_l.append(cand[order[0]] in r["relset"])
        conf_e.append(r["cos"][0])
        top_e.append(cand[0] in r["relset"])
    return out, (np.array(conf_l), np.array(top_l, dtype=bool)), (np.array(conf_e), np.array(top_e, dtype=bool))


def evaluate(name, q, rk):
    per = {m: [metrics(rk[m][r["case"]], r["relset"]) for r in q] for m in rk if rk[m]}
    L = [f"### {name}: {len(q)} queries (candidates: code embedding top-100)", "", "| method | MAP@R [95% CI] | Hits@1 | MRR |", "|---|---|---|---|"]
    res = {}
    for m, v in per.items():
        a = ci([x[0] for x in v])
        L.append(f"| {m} | {a[0]:.4f} [{a[1]:.4f}, {a[2]:.4f}] | {np.mean([x[1] for x in v]):.4f} | {np.mean([x[2] for x in v]):.4f} |")
        res[m] = {"map_at_r": a, "hits1": float(np.mean([x[1] for x in v])), "mrr": float(np.mean([x[2] for x in v]))}
    return L, res, per


def selective(conf, correct, lam):
    ans = conf >= lam
    n = int(ans.sum())
    prec = float(correct[ans].mean()) if n else float("nan")
    return ans, n, prec


def main():
    a = sys.argv[1:]
    dev_only = "--dev-only" in a
    a = [x for x in a if x != "--dev-only"]
    dm, dp, tm, tp, out = a
    _, dq = load(dm, dp)
    X = np.vstack([feats(r) for r in dq])
    y = np.concatenate([r["rel"] for r in dq])
    mu, sd = X.mean(0), X.std(0) + 1e-12
    w = fit_logistic((X - mu) / sd, y, C)
    drk, (dconf, dcorr), (dconf_e, dcorr_e) = rankings(dq, w, mu, sd)
    lam_l = ltt_threshold(dconf, dcorr)
    lam_e = ltt_threshold(dconf_e, dcorr_e)
    L = ["# E36: learned fusion and conformal selective retrieval (code)", "",
         f"Pre-registered in docs/PREREGISTRATION.md, addendum B. Logistic ranker (C = {C}) fit on development pairs; features: {', '.join(FEATURES)}.",
         f"Weights (standardized features, then intercept): {', '.join(f'{x:+.3f}' for x in w)}.",
         f"Conformal (Learn-then-Test, fixed sequence, exact binomial; α = {ALPHA}, δ = {DELTA}, {GRID}-point grid): threshold on learned confidence {lam_l:.4f}; on embedding top-1 cosine {lam_e:.4f}.", ""]
    dL, dres, dper = evaluate("Development (fit and calibration data: in-sample)", dq, drk)
    L += dL
    rec = {"config": {"alpha": ALPHA, "delta": DELTA, "grid": GRID, "C": C, "features": FEATURES, "weights": w.tolist(), "mu": mu.tolist(), "sd": sd.tolist(),
                      "threshold_learned": lam_l, "threshold_embedding": lam_e, "dev": dm, "test": None if dev_only else tm}, "dev": dres}
    for tag, (conf, corr, lam) in {"learned (A3)": (dconf, dcorr, lam_l), "embedding cosine": (dconf_e, dcorr_e, lam_e)}.items():
        _, n, prec = selective(conf, corr, lam)
        L.append(f"- dev selective top-1, {tag} confidence: answered {n}/{len(conf)} ({n / len(conf):.3f}), precision {prec:.4f}")
    if not dev_only:
        _, tq = load(tm, tp)
        trk, (tconf, tcorr), (tconf_e, tcorr_e) = rankings(tq, w, mu, sd)
        tL, tres, tper = evaluate("Test (EV2's sample, run once)", tq, trk)
        L += ["", *tL, ""]
        ap = lambda m: [x[0] for x in tper[m]]
        d13 = paired(ap("learned fusion (A3)"), ap("fixed ½cos + ½FAC (E34)"))
        d13e = paired(ap("learned fusion (A3)"), ap("code embedding"))
        ans_l, n_l, prec_l = selective(tconf, tcorr, lam_l)
        ans_e, n_e, prec_e = selective(tconf_e, tcorr_e, lam_e)
        pci = ci(tcorr[ans_l].astype(float).tolist()) if n_l else (float("nan"),) * 3
        pci_e = ci(tcorr_e[ans_e].astype(float).tolist()) if n_e else (float("nan"),) * 3
        d15 = paired(ans_l.astype(float).tolist(), ans_e.astype(float).tolist())
        L += ["| comparison / quantity | value [95% CI] |", "|---|---|",
              f"| P13: A3 − fixed ½/½ (MAP@R, paired) | {d13[0]:+.4f} [{d13[1]:+.4f}, {d13[2]:+.4f}] (p {d13[3]:.4f}) |",
              f"| A3 − embedding (MAP@R, paired) | {d13e[0]:+.4f} [{d13e[1]:+.4f}, {d13e[2]:+.4f}] |",
              f"| P14: test precision among answered, learned confidence | {pci[0]:.4f} [{pci[1]:.4f}, {pci[2]:.4f}] (answered {n_l}/{len(tconf)}) |",
              f"| test precision among answered, embedding confidence | {pci_e[0]:.4f} [{pci_e[1]:.4f}, {pci_e[2]:.4f}] (answered {n_e}/{len(tconf_e)}) |",
              f"| P15: coverage learned − coverage embedding (paired) | {d15[0]:+.4f} [{d15[1]:+.4f}, {d15[2]:+.4f}] |"]
        rec.update({"test": tres, "P13": d13, "A3_vs_embedding": d13e, "P14": {"precision": pci, "answered": n_l, "n": len(tconf)},
                    "embedding_selective": {"precision": pci_e, "answered": n_e}, "P15": d15,
                    "per_query": {m: [list(x) for x in v] for m, v in tper.items()}, "answered_learned": ans_l.tolist(), "answered_embedding": ans_e.tolist(),
                    "correct_learned": tcorr.tolist(), "correct_embedding": tcorr_e.tolist()})
    os.makedirs(out, exist_ok=True)
    tag = "E36-dev" if dev_only else "E36"
    open(f"{out}/{tag}.md", "w").write("\n".join(L) + "\n")
    json.dump(rec, open(f"{out}/{tag}.json", "w"))
    print("\n".join(L))


if __name__ == "__main__":
    main()
