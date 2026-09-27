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

# JavaScript: npm `algorithms` 0.10.0 (MIT), parsed with acorn (E12, cross-language).
JS=$EXT/js
mkdir -p "$JS"
(cd "$JS" && npm install --silent --no-audit --no-fund acorn@8 >/dev/null && \
  curl -s -o algorithms-0.10.0.tgz https://registry.npmjs.org/algorithms/-/algorithms-0.10.0.tgz && tar xzf algorithms-0.10.0.tgz)
node tools/js_ast.js "$JS/node_modules" "$JS/package" > "$JS/ast.jsonl"
python3 tools/js2mars.py "$JS/ast.jsonl" jsalgs "$OUT"
