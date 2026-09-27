#!/usr/bin/env bash
# E4: robustness to structural perturbation. Runs E0 (fingerprints) and E2
# (mapper) for each operator and severity, then summarizes to results/E4/README.md.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -q -p mars-bench
BIN=./target/release/mars-bench
NGROUPS=${NGROUPS:-1000}
OUT=results/E4
mkdir -p $OUT
for op in all delete-fact insert-intermediate substitute-predicate swap-args add-ho; do
  for sev in 1 2 3 4; do
    tag="${op}-s${sev}"
    $BIN e0 --groups $NGROUPS --distractors 2 --ops $op --severity $sev --out $OUT --tag "$tag" > /dev/null 2>&1
    $BIN e2 --groups $NGROUPS --distractors 2 --ops $op --severity $sev --out $OUT --tag "$tag" > /dev/null 2>&1
    echo "done $tag"
  done
done
python3 scripts/summarize_e4.py
