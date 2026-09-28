#!/usr/bin/env python3
"""CI smoke/regression checks on small deterministic runs (a few seconds).

    python3 scripts/ci_smoke.py [TARGET_DIR]      # default: target/release

Fails (exit 1) when a core property regresses:
* E0 (200 groups, seed 1): the analogy fingerprint puts the true analogue on top
  (TA-top ≥ 0.95) while lexical / MAC content baselines do not (≤ 0.10);
* E2 (200 groups, seed 1): the mapper finds the true analogue's correspondences
  (precision/recall ≥ 0.98), FAC ranks it first (≥ 0.98), greedy = optimal (≥ 0.98);
* CLI: the classic Rutherford analogue is the solar system.
Thresholds leave a margin below the values at the v0.1 freeze (0.98–1.00).
"""

import json
import os
import subprocess
import sys
import tempfile

T = sys.argv[1] if len(sys.argv) > 1 else "target/release"
fails = []


def check(name, value, ok):
    print(f"{'ok  ' if ok else 'FAIL'} {name}: {value}")
    if not ok:
        fails.append(name)


with tempfile.TemporaryDirectory() as d:
    subprocess.run([f"{T}/mars-bench", "e0", "--groups", "200", "--out", d, "--tag", "smoke"], check=True, capture_output=True)
    m = {x["method"]: x["all"] for x in json.load(open(os.path.join(d, "E0-smoke.json")))["methods"]}
    check("E0 fingerprint analogy TA-top", m["fingerprint analogy +IDF D=8192"]["ta_top1"], m["fingerprint analogy +IDF D=8192"]["ta_top1"] >= 0.95)
    for b in ("B2 lexical TF-IDF", "B4 MAC content vectors"):
        check(f"E0 baseline {b} TA-top stays low", m[b]["ta_top1"], m[b]["ta_top1"] <= 0.10)
    subprocess.run([f"{T}/mars-bench", "e2", "--groups", "200", "--out", d, "--tag", "smoke"], check=True, capture_output=True)
    s = next(x for x in json.load(open(os.path.join(d, "E2-smoke.json")))["splits"] if x["split"] == "all")
    for k in ("ta_p", "ta_r", "fac_ta_top", "greedy_over_optimal"):
        check(f"E2 {k}", s[k], s[k] >= 0.98)
out = subprocess.run([f"{T}/mars", "analogies", "data/examples/classic.mars", "--case", "rutherford-atom"], check=True, capture_output=True, text=True).stdout
top = next(l for l in out.splitlines() if l.strip().startswith("1."))
check("CLI rutherford-atom top analogue", top.strip(), "solar-system" in top)
sys.exit(1 if fails else 0)
