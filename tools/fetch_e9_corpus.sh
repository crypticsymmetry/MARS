#!/usr/bin/env bash
# Fetch the E9 program-analogy corpus (MIT-licensed PyPI packages) and convert
# every function to a MARS case. Sources are not committed (data/external is
# git-ignored); the conversion is deterministic.
set -euo pipefail
cd "$(dirname "$0")/.."
EXT=data/external
OUT=data/e9
mkdir -p "$EXT"
cd "$EXT"
for spec in algorithms==1.0.1 pygorithm==1.0.4 python-algorithms==0.2.2; do
  pip download -q --no-deps --no-binary :all: "$spec"
done
for f in *.tar.gz; do tar xzf "$f"; done
cd - > /dev/null
python3 tools/py2mars.py "$OUT" \
  algorithms:$EXT/algorithms-1.0.1/algorithms \
  pygorithm:$EXT/pygorithm-1.0.4/pygorithm \
  pyalgs:$EXT/python_algorithms-0.2.2/python_algorithms
