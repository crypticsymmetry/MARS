#!/usr/bin/env python3
"""Python source -> MARS relational cases in *dataflow* form (E34).

Where tools/py2mars.py encodes syntax (nested expression terms over variable
names), this front end encodes a program-dependence-style graph:

* every operation is one first-order fact over *value* entities, output first:
  (d-add v7 v3 v5), (d-index v9 v2 v8), (d-bi-len v4 v2), (d-m-split v6 v5) …
  Each evaluation yields a fresh value (v1, v2, …); constants are shared
  entities by kind (c0, c1, cint, cfloat, cstr, cnone, cbool); builtin and
  method function objects are entities (f-int, f-str, …);
* variables are resolved to their reaching definition, so names, expression
  nesting and statement grouping do not change the structure;
* loops: a loop entity per loop, (d-iter L elem iterable) / (d-while L cond),
  (d-within L2 L1) for nesting, and for every variable reassigned in the loop a
  loop-carried value h with (d-phi h before end): accumulators, counters and
  running maxima become explicit;
* branches: values defined under a condition get (d-guard v cond) or
  (d-guard-not v cond); variables assigned in a branch are merged afterwards,
  (d-merge m then else cond);
* context: every value defined inside a loop gets (d-ctx v L) (innermost loop);
* normalization: `xs = []` followed by `xs.append(e)` in a loop, and list
  comprehensions, both become (d-collect xs L e);
* I/O: (d-input v) for input()/stdin reads, (d-output v) for print/write;
* optional name attributes (nm-<var> v) keep variable names as surface features
  (PDG_NAMES=0 drops them).

Calls to module functions are inlined one level (parameters bound to argument
values), like py2mars. Usage (library): encode_script(src) -> list of facts.
"""

import ast
import os
import re

NAMES = os.environ.get("PDG_NAMES", "1") == "1"
# Context facts per value: "all" (d-ctx/d-guard on every value), "effects" (only on
# effectful operations: output, stores, collection updates), "none".
CTX = os.environ.get("PDG_CTX", "all")
CSE = os.environ.get("PDG_CSE", "1") == "1"

BINOPS = {ast.Add: "add", ast.Sub: "sub", ast.Mult: "mul", ast.Div: "div", ast.FloorDiv: "floordiv", ast.Mod: "mod", ast.Pow: "pow",
          ast.LShift: "lshift", ast.RShift: "rshift", ast.BitOr: "bitor", ast.BitXor: "bitxor", ast.BitAnd: "bitand", ast.MatMult: "matmul"}
CMPOPS = {ast.Lt: "lt", ast.LtE: "le", ast.Gt: "gt", ast.GtE: "ge", ast.Eq: "eq", ast.NotEq: "ne", ast.In: "in", ast.NotIn: "notin", ast.Is: "is", ast.IsNot: "isnot"}
UNOPS = {ast.USub: "neg", ast.UAdd: "pos", ast.Not: "not", ast.Invert: "invert"}
BUILTINS = {"len", "range", "min", "max", "abs", "sum", "sorted", "reversed", "enumerate", "zip", "int", "float", "str", "list", "dict", "set", "tuple",
            "print", "isinstance", "divmod", "pow", "round", "map", "filter", "any", "all", "iter", "next", "ord", "chr", "bool", "input", "open", "bin", "hex", "gcd"}
METHODS = {"append", "pop", "insert", "extend", "remove", "index", "count", "sort", "reverse", "get", "keys", "values", "items", "add", "update", "copy", "clear",
           "popleft", "appendleft", "join", "split", "format", "startswith", "endswith", "lower", "upper", "strip", "rstrip", "setdefault", "discard", "readline",
           "readlines", "read", "write", "replace", "find", "most_common", "heappush", "heappop", "sqrt", "ceil", "floor", "log", "factorial", "comb", "permutations",
           "combinations", "product", "accumulate", "bisect_left", "bisect_right", "deque", "defaultdict", "Counter"}

GROUPS = {
    "d-arith": [f"d-{v}" for v in BINOPS.values()],
    "d-cmp": [f"d-{v}" for v in CMPOPS.values()],
    "d-unary": [f"d-{v}" for v in UNOPS.values()],
    "d-builtin": [f"d-bi-{b}" for b in sorted(BUILTINS)],
    "d-method": [f"d-m-{m}" for m in sorted(METHODS)] + ["d-m-other"],
    "d-access": ["d-index", "d-slice", "d-attr", "d-store", "d-store-attr"],
    "d-make": ["d-list", "d-tuple", "d-dict", "d-set", "d-collect", "d-lambda"],
    "d-loop": ["d-iter", "d-while", "d-within"],
    "d-cond": ["d-guard", "d-guard-not"],
    "d-join": ["d-phi", "d-merge", "d-ifexp"],
    "d-logic": ["d-and", "d-or"],
    "d-io": ["d-input", "d-output"],
    "d-calls": ["d-return", "d-call", "d-recurse"],
}
PURE = set(GROUPS["d-arith"] + GROUPS["d-cmp"] + GROUPS["d-unary"]) | {"d-index", "d-slice", "d-attr", "d-and", "d-or", "d-ifexp", "d-unpack"} | {
    f"d-bi-{b}" for b in ("len", "range", "min", "max", "abs", "sum", "sorted", "int", "float", "str", "ord", "chr", "bool", "divmod", "pow", "round", "bin", "hex", "gcd", "tuple")}
SOLO = ["d-ctx", "d-unpack"]  # no taxonomy parent (no ascension matches)


def vocab():
    lines = []
    for g, preds in GROUPS.items():
        lines.append(f"(defpredicate {g} :arity * :kind relation)")
        for p in preds:
            lines.append(f"(defpredicate {p} :arity * :kind relation :parents ({g}))")
    for p in SOLO:
        lines.append(f"(defpredicate {p} :arity * :kind relation)")
    return "\n".join(lines) + "\n"


def ident(s):
    s = re.sub(r"[^A-Za-z0-9_\-]", "_", s)
    return s if s else "_"


class Enc:
    def __init__(self, module_funcs, inline=1):
        self.facts = []
        self.n = 0
        self.env = {}
        self.loops = []  # stack of loop entities
        self.guards = []  # stack of (cond value, positive?)
        self.module_funcs = module_funcs
        self.inline = inline
        self.empty_lists = set()  # values of `[]` literals
        self.names_used = set()
        self.memo = {}  # value numbering: (pred, ins) -> value, for pure operations

    # ---- values
    def fresh(self):
        self.n += 1
        return f"v{self.n}"

    def emit(self, pred, *args):
        self.facts.append(f"({pred} {' '.join(args)})")

    def defval(self, pred, *ins):
        """A new value produced by operation `pred` from `ins`, with its context.
        Pure operations with identical inputs reuse the earlier value (CSE)."""
        key = (pred, ins)
        if CSE and pred in PURE:
            if key in self.memo:
                return self.memo[key]
        if pred.startswith("d-m-"):
            self.memo.clear()  # method calls may mutate their receiver
        v = self.fresh()
        if CSE and pred in PURE:
            self.memo[key] = v
        self.emit(pred, v, *ins)
        if CTX == "all" or (CTX == "effects" and pred.startswith("d-m-")):
            self.context(v)
        return v

    def effect(self, pred, *args):
        """An effectful operation (output, store, collect, return): the fact, plus the
        loop/guard context of its subject (first argument) unless PDG_CTX=none."""
        self.emit(pred, *args)
        if pred in ("d-store", "d-store-attr", "d-collect"):
            self.memo.clear()  # a store or update may change what earlier reads return
        if CTX != "none" and args and pred != "d-collect":
            self.context(args[0])

    def context(self, v):
        if self.loops:
            self.emit("d-ctx", v, self.loops[-1])
        if self.guards:
            c, pos = self.guards[-1]
            self.emit("d-guard" if pos else "d-guard-not", v, c)

    def const(self, val):
        if val is None:
            return "cnone"
        if isinstance(val, bool):
            return "cbool"
        if isinstance(val, int):
            return {0: "c0", 1: "c1"}.get(val, "cint")
        if isinstance(val, float):
            return "cfloat"
        return "cstr"

    def bind(self, name, v):
        self.env[name] = v
        if NAMES and v.startswith("v"):
            a = f"nm-{ident(name)}"
            self.names_used.add(a)
            self.facts.append(f"({a} {v})")

    def lookup(self, name):
        if name in self.env:
            return self.env[name]
        if name in BUILTINS:
            return f"f-{name}"
        v = self.fresh()  # free variable (global / builtin): an input value
        self.env[name] = v
        return v

    # ---- expressions
    def expr(self, n):
        t = type(n)
        if t is ast.Constant:
            return self.const(n.value)
        if t is ast.Name:
            return self.lookup(n.id)
        if t is ast.BinOp:
            return self.defval(f"d-{BINOPS.get(type(n.op), 'add')}", self.expr(n.left), self.expr(n.right))
        if t is ast.UnaryOp:
            return self.defval(f"d-{UNOPS[type(n.op)]}", self.expr(n.operand))
        if t is ast.BoolOp:
            return self.defval("d-and" if isinstance(n.op, ast.And) else "d-or", *[self.expr(v) for v in n.values])
        if t is ast.Compare:
            vals = [self.expr(n.left)] + [self.expr(c) for c in n.comparators]
            outs = [self.defval(f"d-{CMPOPS.get(type(op), 'eq')}", vals[i], vals[i + 1]) for i, op in enumerate(n.ops)]
            return outs[0] if len(outs) == 1 else self.defval("d-and", *outs)
        if t is ast.Subscript:
            base = self.expr(n.value)
            if isinstance(n.slice, ast.Slice):
                parts = [self.expr(p) if p is not None else "cnone" for p in (n.slice.lower, n.slice.upper, n.slice.step)]
                return self.defval("d-slice", base, *parts)
            return self.defval("d-index", base, self.expr(n.slice))
        if t is ast.Attribute:
            return self.defval("d-attr", self.expr(n.value), f"a-{ident(n.attr)}")
        if t is ast.Call:
            return self.call(n)
        if t in (ast.List, ast.Tuple, ast.Set):
            kind = {ast.List: "d-list", ast.Tuple: "d-tuple", ast.Set: "d-set"}[t]
            v = self.defval(kind, *[self.expr(e) for e in n.elts])
            if t is ast.List and not n.elts:
                self.empty_lists.add(v)
            return v
        if t is ast.Dict:
            return self.defval("d-dict", *[self.expr(e) for e in list(n.keys) + list(n.values) if e is not None])
        if t is ast.IfExp:
            c = self.expr(n.test)
            return self.defval("d-ifexp", c, self.expr(n.body), self.expr(n.orelse))
        if t in (ast.ListComp, ast.SetComp, ast.GeneratorExp, ast.DictComp):
            return self.comprehension(n)
        if t is ast.Lambda:
            return self.defval("d-lambda", *[f"a-{ident(a.arg)}" for a in n.args.args])
        if t is ast.JoinedStr:
            return self.defval("d-bi-str", *[self.expr(v.value) for v in n.values if isinstance(v, ast.FormattedValue)])
        if t is ast.Starred:
            return self.expr(n.value)
        if t is ast.NamedExpr:
            v = self.expr(n.value)
            self.bind(n.target.id, v)
            return v
        return self.defval("d-call", "cnone")

    def comprehension(self, n):
        out = self.defval("d-list")
        saved = dict(self.env)
        depth = 0
        for g in n.generators:
            it = self.expr(g.iter)
            L = self.enter_loop()
            depth += 1
            elem = self.defval("d-iter", L, it)
            self.assign_target(g.target, elem)
            for cond in g.ifs:
                c = self.expr(cond)
                self.guards.append((c, True))
        elt = self.expr(n.elt if not isinstance(n, ast.DictComp) else n.value)
        self.effect("d-collect", out, self.loops[-1], elt)
        for g in n.generators:
            for _ in g.ifs:
                self.guards.pop()
        for _ in range(depth):
            self.loops.pop()
        self.env = saved
        return out

    def call(self, n):
        f = n.func
        args = [self.expr(a) for a in n.args] + [self.expr(k.value) for k in n.keywords]
        if isinstance(f, ast.Name):
            name = f.id
            if name in ("input",):
                return self.defval("d-input")
            if name == "print":
                self.effect("d-output", *args) if args else None
                return "cnone"
            if name in self.module_funcs and self.inline > 0:
                return self.inline_call(name, args)
            if name in BUILTINS:
                return self.defval(f"d-bi-{name}", *args)
            return self.defval("d-call", f"fn-{ident(name)}", *args)
        if isinstance(f, ast.Attribute):
            obj = self.expr(f.value)
            m = f.attr
            if m in ("readline", "read", "readlines") or (isinstance(f.value, ast.Attribute) and getattr(f.value, "attr", "") == "stdin"):
                return self.defval("d-input")
            if m == "write":
                self.effect("d-output", *args)
                return "cnone"
            if m == "append" and obj in self.empty_lists and self.loops:
                self.effect("d-collect", obj, self.loops[-1], *args)
                return "cnone"
            return self.defval(f"d-m-{m}" if m in METHODS else "d-m-other", obj, *args)
        return self.defval("d-call", self.expr(f), *args)

    def inline_call(self, name, args):
        fn = self.module_funcs[name]
        saved_env, saved_inline = self.env, self.inline
        self.env = dict(self.env)
        self.inline -= 1
        params = [a.arg for a in fn.args.args]
        for p, a in zip(params, args):
            self.env[p] = a
        self.returns = []
        prev_returns = getattr(self, "_ret_stack", [])
        self._ret_stack = prev_returns + [[]]
        self.stmts(fn.body)
        rets = self._ret_stack.pop()
        self._ret_stack = prev_returns
        self.env, self.inline = saved_env, saved_inline
        if not rets:
            return "cnone"
        return rets[0] if len(rets) == 1 else self.defval("d-merge", *rets)

    # ---- statements
    def assign_target(self, t, v):
        if isinstance(t, ast.Name):
            self.bind(t.id, v)
        elif isinstance(t, (ast.Tuple, ast.List)):
            for i, e in enumerate(t.elts):
                self.assign_target(e, self.defval("d-unpack", v, f"i{min(i, 3)}"))
        elif isinstance(t, ast.Subscript):
            base = self.expr(t.value)
            idx = self.expr(t.slice) if not isinstance(t.slice, ast.Slice) else "cnone"
            self.effect("d-store", base, idx, v)
        elif isinstance(t, ast.Attribute):
            self.effect("d-store-attr", self.expr(t.value), f"a-{ident(t.attr)}", v)
        elif isinstance(t, ast.Starred):
            self.assign_target(t.value, v)

    def enter_loop(self):
        L = f"L{len(self.facts)}_{self.n}"
        if self.loops:
            self.emit("d-within", L, self.loops[-1])
        self.loops.append(L)
        return L

    def assigned_in(self, body):
        out = set()
        for s in body:
            for n in ast.walk(s):
                if isinstance(n, (ast.Assign, ast.AugAssign, ast.AnnAssign)):
                    for t in (n.targets if isinstance(n, ast.Assign) else [n.target]):
                        for x in ast.walk(t):
                            if isinstance(x, ast.Name):
                                out.add(x.id)
                elif isinstance(n, (ast.For, ast.comprehension)):
                    for x in ast.walk(n.target):
                        if isinstance(x, ast.Name):
                            out.add(x.id)
        return out

    def loop(self, head, body, orelse):
        carried = [v for v in sorted(self.assigned_in(body)) if v in self.env]
        before = {v: self.env[v] for v in carried}
        L = self.enter_loop()
        heads = {}
        for v in carried:
            h = self.fresh()
            heads[v] = h
            self.env[v] = h
        head(L)
        self.stmts(body)
        for v in carried:
            self.emit("d-phi", heads[v], before[v], self.env.get(v, heads[v]))
            self.env[v] = heads[v]
        self.loops.pop()
        self.stmts(orelse)

    def stmts(self, body):
        for s in body:
            self.stmt(s)

    def stmt(self, s):
        t = type(s)
        if t is ast.Assign:
            v = self.expr(s.value)
            for tg in s.targets:
                self.assign_target(tg, v)
        elif t is ast.AugAssign:
            cur = self.expr(s.target)
            v = self.defval(f"d-{BINOPS.get(type(s.op), 'add')}", cur, self.expr(s.value))
            self.assign_target(s.target, v)
        elif t is ast.AnnAssign and s.value is not None:
            self.assign_target(s.target, self.expr(s.value))
        elif t is ast.Expr:
            self.expr(s.value)
        elif t is ast.Return:
            v = self.expr(s.value) if s.value is not None else "cnone"
            self.effect("d-return", v)
            if getattr(self, "_ret_stack", None):
                self._ret_stack[-1].append(v)
        elif t is ast.For:
            it = self.expr(s.iter)

            def head(L):
                elem = self.defval("d-iter", L, it)
                self.assign_target(s.target, elem)
            self.loop(head, s.body, s.orelse)
        elif t is ast.While:
            def head(L):
                c = self.expr(s.test)
                self.emit("d-while", L, c)
            self.loop(head, s.body, s.orelse)
        elif t is ast.If:
            c = self.expr(s.test)
            before = dict(self.env)
            self.guards.append((c, True))
            self.stmts(s.body)
            self.guards.pop()
            then_env = self.env
            self.env = dict(before)
            self.guards.append((c, False))
            self.stmts(s.orelse)
            self.guards.pop()
            else_env = self.env
            merged = dict(before)
            for v in set(then_env) | set(else_env):
                a, b = then_env.get(v), else_env.get(v)
                if a == b:
                    merged[v] = a
                elif a is not None and b is not None:
                    m = self.fresh()
                    self.emit("d-merge", m, a, b, c)
                    merged[v] = m
                else:
                    merged[v] = a or b
            self.env = merged
        elif t in (ast.With, ast.Try):
            self.stmts(s.body)
            for h in getattr(s, "handlers", []):
                self.stmts(h.body)
            self.stmts(getattr(s, "orelse", []))
            self.stmts(getattr(s, "finalbody", []))
        # defs, imports, pass, break, continue, global: no dataflow facts


def encode_script(src, inline=1):
    """Facts of a whole script: top-level statements (module functions inlined one level)."""
    tree = ast.parse(src)
    module_funcs = {n.name: n for n in tree.body if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef))}
    body = [n for n in tree.body if not isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Import, ast.ImportFrom))]
    e = Enc(module_funcs, inline)
    e.stmts(body)
    return e.facts, e.names_used


if __name__ == "__main__":
    import sys
    facts, _ = encode_script(open(sys.argv[1]).read())
    print("\n".join(facts))
