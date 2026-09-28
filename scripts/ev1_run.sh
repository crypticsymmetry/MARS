#!/usr/bin/env bash
# EV1 (pre-registered): validation grid -> chosen settings -> one test run, per dataset.
# The test run reads its settings from the grid's JSON; nothing is chosen by hand.
set -euo pipefail
B=${B:-target/release/mars-bench}
OUT=${OUT:-results/EV1}
for ds in FB15k-237:data/external/kgc/FB15k-237 WN18RR:data/external/kgc/WN18RR-original; do
  name=${ds%%:*}; dir=${ds#*:}
  $B ev1 --data "$dir" --name "$name" --split valid --sample 2000 --profile analogy,literal --identity 0,0.5,0.85 --k 10,30 --out "$OUT"
  read -r p l k < <(python3 -c "import json; c=json.load(open('$OUT/EV1-$name-valid-grid.json'))['chosen']; print(c['profile'], c['identity'], c['k'])")
  echo "[ev1_run] $name chosen on validation: profile=$p identity=$l k=$k"
  $B ev1 --data "$dir" --name "$name" --split test --profile "$p" --identity "$l" --k "$k" --out "$OUT"
done
