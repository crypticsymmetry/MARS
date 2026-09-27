#!/usr/bin/env python3
"""Convert short natural-language stories into MARS relational cases with an
LLM (OpenRouter chat API; key from $OPENROUTER_API_KEY, never stored).

    python3 tools/llm2mars.py STORIES.jsonl OUT_DIR [--model M1,M2,...] [--batch 8]

Several comma-separated models form a fallback chain: a batch that fails on
one model (free models are often throttled or overloaded) is retried on the
next; the model that produced each story is recorded in the cache.

STORIES.jsonl: one {"id": ..., "text": ...} per line. Writes OUT_DIR/vocab.mars
(the controlled vocabulary), OUT_DIR/cases.mars (one defcase per story) and
OUT_DIR/cache.jsonl (raw LLM output per story id; reruns only call the LLM
for stories not yet cached).

Representation: entities are short identifiers; first-order relations come
from a fixed abstract vocabulary (Schank's conceptual-dependency primitives
plus abstract states and intentions); higher-order relations (cause, enable,
prevent, then, and, repeat, despite) connect event expressions. Predicates
outside the vocabulary are kept as unresolved `x:<name>` (structure-only).
"""

import json
import os
import re
import sys
import time
import urllib.request

# name: (arity, description) -- arity None = variadic
FO = {
    "ptrans": (3, "agent moves object to a location"),
    "atrans": (3, "agent transfers possession of object to recipient"),
    "mtrans": (3, "agent communicates information to recipient"),
    "mbuild": (2, "agent forms an idea, plan or decision"),
    "propel": (2, "agent applies force to object"),
    "ingest": (2, "agent takes in object"),
    "expel": (2, "agent pushes out object"),
    "grasp": (2, "agent grasps or holds object"),
    "attend": (2, "agent perceives or focuses on object"),
    "create": (2, "agent makes object"),
    "destroy": (2, "agent destroys or consumes object"),
    "increase": (1, "quantity or intensity of x grows"),
    "decrease": (1, "quantity or intensity of x shrinks"),
    "become": (2, "x comes to be in state/role y"),
    "has-property": (2, "x has property y"),
    "contain": (2, "x contains y"),
    "part-of": (2, "x is part of y"),
    "at": (2, "x is located at y"),
    "support": (2, "x supports or holds up y"),
    "block": (2, "x blocks or covers y"),
    "attach": (2, "x connects to y"),
    "separate": (2, "x is separated from y"),
    "want": (2, "agent wants/intends x"),
    "try": (2, "agent attempts x"),
    "succeed": (2, "agent achieves x"),
    "fail": (2, "agent fails at x"),
    "help": (2, "x helps y"),
    "harm": (2, "x harms y"),
    "compete": (2, "x competes with y"),
    "use": (3, "agent uses instrument for purpose"),
    "start": (1, "x begins"),
    "stop": (1, "x ends"),
}
HO = {
    "cause": (2, "event A causes event B"),
    "enable": (2, "A makes B possible"),
    "prevent": (2, "A prevents B"),
    "then": (2, "A is followed by B"),
    "despite": (2, "B happens despite A"),
    "repeat": (1, "A keeps recurring"),
    "and": (None, "conjunction of events"),
}


def vocab_mars():
    out = [";; controlled vocabulary for tools/llm2mars.py"]
    for n, (a, _) in FO.items():
        out.append(f"(defpredicate {n} :arity {a} :kind relation)")
    for n, (a, _) in HO.items():
        extra = " :commutative t" if n == "and" else ""
        out.append(f"(defpredicate {n} :arity {a if a else '*'} :kind relation{extra})")
    return "\n".join(out) + "\n"


PROMPT = """You convert short stories into relational facts for a structural analogy engine.

Rules:
- Entities: short lowercase identifiers (letters, digits, hyphens), e.g. wind, hail, person, coin.
- Events/states are expressions (predicate arg ...). Use ONLY these first-order predicates (arity in brackets):
{fo}
- Connect events with these higher-order predicates, whose arguments are event expressions:
{ho}
- Choose the most abstract fitting predicate; capture the causal/temporal structure of the story, not word choice.
- Write one top-level fact per line as an s-expression. Nest events inside higher-order facts.
- 3 to 8 top-level facts per story. No comments, no prose.

Example story: "The dam holds back the river. When heavy rain raises the water, the dam breaks and the valley floods."
Example output:
(block dam river)
(cause (increase water) (destroy water dam))
(cause (destroy water dam) (ptrans water water valley))
(then (block dam river) (destroy water dam))

Convert each story below. For each, output a line '### <id>' followed by its facts.

{stories}"""


def build_prompt(batch):
    fo = "\n".join(f"  {n} [{a}]: {d}" for n, (a, d) in FO.items())
    ho = "\n".join(f"  {n} [{a if a else 'n'}]: {d}" for n, (a, d) in HO.items())
    stories = "\n\n".join(f"### {s['id']}\n{s['text']}" for s in batch)
    return PROMPT.format(fo=fo, ho=ho, stories=stories)


def call(model, prompt, retries=3, timeout=90):
    key = os.environ["OPENROUTER_API_KEY"]
    body = json.dumps({"model": model, "messages": [{"role": "user", "content": prompt}], "temperature": 0}).encode()
    for i in range(retries):
        req = urllib.request.Request("https://openrouter.ai/api/v1/chat/completions", data=body, headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"})
        try:
            t0 = time.time()
            with urllib.request.urlopen(req, timeout=timeout) as r:
                d = json.load(r)
            if "error" in d:
                raise RuntimeError(str(d["error"])[:200])
            msg = d["choices"][0]["message"].get("content") or ""
            if msg.strip():
                return msg
            print(f"  attempt {i + 1}: empty response ({time.time() - t0:.0f}s)", file=sys.stderr, flush=True)
        except Exception as e:  # rate limits, timeouts: back off briefly
            print(f"  attempt {i + 1} failed: {str(e)[:200]}", file=sys.stderr, flush=True)
        time.sleep(5 * (i + 1))
    raise RuntimeError(f"LLM call failed after {retries} attempts")


def split_output(text):
    out, cur = {}, None
    for line in text.splitlines():
        m = re.match(r"^\s*#+\s*(\S+)", line)
        if m:
            cur = m.group(1).strip()
            out[cur] = []
        elif cur is not None and line.strip().startswith("("):
            out[cur].append(line.strip())
    return out


TOKEN = re.compile(r"\(|\)|[^\s()]+")


def parse(s):
    toks = TOKEN.findall(s)
    pos = 0

    def rec():
        nonlocal pos
        t = toks[pos]
        pos += 1
        if t == "(":
            items = []
            while toks[pos] != ")":
                items.append(rec())
            pos += 1
            return items
        return t

    e = rec()
    if pos != len(toks):
        raise ValueError("trailing tokens")
    return e


def ident(x):
    x = re.sub(r"[^a-z0-9-]+", "-", x.lower()).strip("-")
    return x or "thing"


def normalize(e, depth=0):
    """Validated s-expression text; unknown predicates become x:<name>."""
    if isinstance(e, str):
        return ident(e)
    if not e or not isinstance(e[0], str) or depth > 6:
        raise ValueError("bad expression")
    p = e[0].lower()
    args = [normalize(a, depth + 1) for a in e[1:]]
    if not args:
        raise ValueError("no args")
    if p in HO or p in FO:
        name = p
    else:
        name = "x:" + ident(p)
    return f"({name} {' '.join(args)})"


def main():
    src, out_dir = sys.argv[1], sys.argv[2]
    model = "google/gemma-4-31b-it:free"
    batch_size = 8
    args = sys.argv[3:]
    if "--model" in args:
        model = args[args.index("--model") + 1]
    if "--batch" in args:
        batch_size = int(args[args.index("--batch") + 1])
    os.makedirs(out_dir, exist_ok=True)
    stories = [json.loads(l) for l in open(src)]
    cache_path = os.path.join(out_dir, "cache.jsonl")
    cache = {}
    if os.path.exists(cache_path):
        for l in open(cache_path):
            r = json.loads(l)
            cache[r["id"]] = r
    todo = [s for s in stories if s["id"] not in cache]
    models = model.split(",")
    print(f"{len(stories)} stories, {len(todo)} to convert with {models}", file=sys.stderr, flush=True)
    with open(cache_path, "a") as cf:
        for i in range(0, len(todo), batch_size):
            batch = todo[i : i + batch_size]
            t0 = time.time()
            print(f"  request: stories {i + 1}-{i + len(batch)} ...", file=sys.stderr, flush=True)
            out = split_output(call(model, build_prompt(batch)))
            for s in batch:
                r = {"id": s["id"], "model": used, "facts": out.get(s["id"], [])}
                cache[s["id"]] = r
                cf.write(json.dumps(r) + "\n")
            cf.flush()
            got = sum(1 for s in batch if out.get(s["id"]))
            print(f"  done {i + len(batch)}/{len(todo)}: {got}/{len(batch)} stories parsed by {used} ({time.time() - t0:.1f}s)", file=sys.stderr, flush=True)
    with open(os.path.join(out_dir, "vocab.mars"), "w") as f:
        f.write(vocab_mars())
    unresolved, bad, empty = set(), 0, 0
    lines = []
    for s in stories:
        facts = []
        for raw in cache.get(s["id"], {"facts": []})["facts"]:
            try:
                t = normalize(parse(raw))
                facts.append(t)
                unresolved.update(re.findall(r"\(x:([a-z0-9-]+)", t))
            except Exception:
                bad += 1
        if not facts:
            empty += 1
            facts = ["(x:empty story)"]
        lines.append(f"(defcase {s['id']}\n  " + "\n  ".join(sorted(set(facts))) + ")")
    with open(os.path.join(out_dir, "cases.mars"), "w") as f:
        for u in sorted(unresolved):
            f.write(f"(defpredicate x:{u} :arity * :kind relation :canonical nil)\n")
        f.write("\n".join(lines) + "\n")
    print(f"wrote {len(stories)} cases; {bad} unparsable facts dropped; {empty} empty; {len(unresolved)} unresolved predicates", file=sys.stderr)


if __name__ == "__main__":
    main()
