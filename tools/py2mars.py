#!/usr/bin/env python3
"""Python source -> MARS relational cases (one case per function).

Representation (deliberately simple, fully automatic, no hand-tuning per
program):

* Variables, parameters and attribute names are *entities* (their names are
  surface features; structural channels ignore them). Constants become
  shared entities by kind: c0, c1, cint, cfloat, cstr, cnone, cbool.
* Expressions become relational terms: arithmetic (add, sub, ...),
  comparisons (lt, gt, ...; chained comparisons -> and), indexing
  (index a i), slicing, builtin calls (bi-len, bi-range, ...), method calls
  (m-append obj x), calls to the function itself (recurse ...), other calls
  (call ...).
* Statements become first-order facts: (assign t e), (store (index a i) e),
  (swap a b) for tuple swaps, (returns e), (do e), (iterates v it) ...
* Calls to other functions of the same module are inlined one level
  (MARS_INLINE, default 1) as (inlined <call-term> fact): helper structure
  (e.g. merge sort's `merge`) becomes part of the caller's case.
* Control flow becomes higher-order relations over statement facts:
  (in-loop v fact) for `for v in ...`, (while-loop test fact),
  (guards test fact) / (guards (not test) fact) for if/else. Nesting yields
  nested higher-order structure (systematicity).

Usage: py2mars.py OUT_DIR PKG_NAME:SRC_ROOT [PKG_NAME:SRC_ROOT ...]
Writes OUT_DIR/<pkg>.mars and OUT_DIR/manifest.json.
"""

import ast
import json
import os
import re
import sys

BINOPS = {
    ast.Add: "add", ast.Sub: "sub", ast.Mult: "mul", ast.Div: "div", ast.FloorDiv: "floordiv",
    ast.Mod: "mod", ast.Pow: "pow", ast.LShift: "lshift", ast.RShift: "rshift",
    ast.BitOr: "bitor", ast.BitXor: "bitxor", ast.BitAnd: "bitand", ast.MatMult: "matmul",
}
CMPOPS = {
    ast.Lt: "lt", ast.LtE: "le", ast.Gt: "gt", ast.GtE: "ge", ast.Eq: "eq", ast.NotEq: "ne",
    ast.In: "in", ast.NotIn: "notin", ast.Is: "is", ast.IsNot: "isnot",
}
UNOPS = {ast.USub: "neg", ast.UAdd: "pos", ast.Not: "not", ast.Invert: "invert"}
BUILTINS = {
    "len", "range", "min", "max", "abs", "sum", "sorted", "reversed", "enumerate", "zip", "int",
    "float", "str", "list", "dict", "set", "tuple", "print", "isinstance", "divmod", "pow", "round",
    "map", "filter", "any", "all", "iter", "next", "ord", "chr", "bool", "type", "hash",
}
METHODS = {
    "append", "pop", "insert", "extend", "remove", "index", "count", "sort", "reverse", "get", "keys",
    "values", "items", "add", "update", "copy", "clear", "popleft", "appendleft", "join", "split",
    "format", "startswith", "endswith", "lower", "upper", "strip", "setdefault", "discard",
}
SKIP_FUNCS = {"get_code", "time_complexities", "__repr__", "__str__"}

# Vocabulary header: kinds, arities (* = variadic) and a small taxonomy.
VOCAB = """
(defpredicate order-cmp :arity 2 :kind relation)
(defpredicate eq-cmp :arity 2 :kind relation)
(defpredicate member-cmp :arity 2 :kind relation)
(defpredicate lt :arity 2 :kind relation :parents (order-cmp))
(defpredicate le :arity 2 :kind relation :parents (order-cmp))
(defpredicate gt :arity 2 :kind relation :parents (order-cmp))
(defpredicate ge :arity 2 :kind relation :parents (order-cmp))
(defpredicate eq :arity 2 :kind relation :parents (eq-cmp))
(defpredicate ne :arity 2 :kind relation :parents (eq-cmp))
(defpredicate is :arity 2 :kind relation :parents (eq-cmp))
(defpredicate isnot :arity 2 :kind relation :parents (eq-cmp))
(defpredicate in :arity 2 :kind relation :parents (member-cmp))
(defpredicate notin :arity 2 :kind relation :parents (member-cmp))
(defpredicate arith :arity 2 :kind function)
(defpredicate add :arity 2 :kind function :parents (arith))
(defpredicate sub :arity 2 :kind function :parents (arith))
(defpredicate mul :arity 2 :kind function :parents (arith))
(defpredicate div :arity 2 :kind function :parents (arith))
(defpredicate floordiv :arity 2 :kind function :parents (arith))
(defpredicate mod :arity 2 :kind function :parents (arith))
(defpredicate pow :arity 2 :kind function :parents (arith))
(defpredicate lshift :arity 2 :kind function :parents (arith))
(defpredicate rshift :arity 2 :kind function :parents (arith))
(defpredicate bitor :arity 2 :kind function :parents (arith))
(defpredicate bitxor :arity 2 :kind function :parents (arith))
(defpredicate bitand :arity 2 :kind function :parents (arith))
(defpredicate matmul :arity 2 :kind function :parents (arith))
(defpredicate neg :arity 1 :kind function)
(defpredicate pos :arity 1 :kind function)
(defpredicate invert :arity 1 :kind function)
(defpredicate not :arity 1 :kind logical)
(defpredicate and :arity * :kind logical :commutative t)
(defpredicate or :arity * :kind logical :commutative t)
(defpredicate index :arity 2 :kind function)
(defpredicate slice :arity 3 :kind function)
(defpredicate attr :arity 2 :kind function)
(defpredicate ifexp :arity 3 :kind function)
(defpredicate list-lit :arity * :kind function)
(defpredicate dict-lit :arity * :kind function)
(defpredicate comprehension :arity * :kind function)
(defpredicate lambda :arity * :kind function)
(defpredicate starred :arity 1 :kind function)
(defpredicate call :arity * :kind function)
(defpredicate recurse :arity * :kind function)
(defpredicate assign :arity 2 :kind relation)
(defpredicate store :arity 2 :kind relation)
(defpredicate swap :arity 2 :kind relation :commutative t)
(defpredicate returns :arity 1 :kind relation)
(defpredicate yields :arity 1 :kind relation)
(defpredicate do :arity 1 :kind relation)
(defpredicate raises :arity 1 :kind relation)
(defpredicate asserts :arity 1 :kind relation)
(defpredicate deletes :arity 1 :kind relation)
(defpredicate param :arity 1 :kind relation)
(defpredicate iterates :arity 2 :kind relation)
(defpredicate break :arity 0 :kind relation)
(defpredicate continue :arity 0 :kind relation)
(defpredicate control :arity 2 :kind relation)
(defpredicate in-loop :arity 2 :kind relation :parents (control))
(defpredicate while-loop :arity 2 :kind relation :parents (control))
(defpredicate guards :arity 2 :kind relation :parents (control))
(defpredicate on-error :arity 1 :kind relation)
(defpredicate inlined :arity 2 :kind relation)
(defpredicate flows :arity 2 :kind relation)
(defpredicate loop-cond :arity 2 :kind relation)
"""
EXTRA_PREDS = ["bi-" + b for b in sorted(BUILTINS)] + ["m-" + m for m in sorted(METHODS)] + ["m-other"]


def ident(s):
    s = re.sub(r"[^A-Za-z0-9_\-]", "_", s)
    return s if s else "_"


class FuncEncoder:
    def __init__(self, fname, module_funcs=None):
        self.fname = fname
        self.module_funcs = module_funcs or {}
        self.callees = []  # (callee name, call term) for same-module functions

    def const(self, v):
        if v is None:
            return "cnone"
        if isinstance(v, bool):
            return "cbool"
        if isinstance(v, int):
            return {0: "c0", 1: "c1"}.get(v, "cint")
        if isinstance(v, float):
            return "cfloat"
        return "cstr"

    def term(self, n):
        t = type(n)
        if t is ast.Name:
            return ident(n.id)
        if t is ast.Constant:
            return self.const(n.value)
        if t is ast.BinOp:
            return f"({BINOPS.get(type(n.op), 'add')} {self.term(n.left)} {self.term(n.right)})"
        if t is ast.UnaryOp:
            return f"({UNOPS[type(n.op)]} {self.term(n.operand)})"
        if t is ast.BoolOp:
            op = "and" if isinstance(n.op, ast.And) else "or"
            return f"({op} {' '.join(self.term(v) for v in n.values)})"
        if t is ast.Compare:
            parts, left = [], n.left
            for op, right in zip(n.ops, n.comparators):
                parts.append(f"({CMPOPS[type(op)]} {self.term(left)} {self.term(right)})")
                left = right
            return parts[0] if len(parts) == 1 else f"(and {' '.join(parts)})"
        if t is ast.Subscript:
            if isinstance(n.slice, ast.Slice):
                s = n.slice
                lo = self.term(s.lower) if s.lower else "cnone"
                hi = self.term(s.upper) if s.upper else "cnone"
                return f"(slice {self.term(n.value)} {lo} {hi})"
            return f"(index {self.term(n.value)} {self.term(n.slice)})"
        if t is ast.Call:
            args = " ".join(self.term(a) for a in n.args)
            f = n.func
            if isinstance(f, ast.Name):
                if f.id == self.fname:
                    return f"(recurse {args})".replace(" )", ")")
                if f.id in BUILTINS:
                    if RAW and f.id != "range":
                        return f"({raw(f.id)} {args})".replace(" )", ")")
                    return f"(bi-{f.id} {args})".replace(" )", ")")
                term = f"(call {ident(f.id)} {args})".replace(" )", ")")
                if f.id in self.module_funcs:
                    self.callees.append((f.id, term))
                return term
            if isinstance(f, ast.Attribute):
                if RAW:
                    return f"({raw(f.attr)} {self.term(f.value)} {args})".replace(" )", ")")
                m = f.attr if f.attr in METHODS else "other"
                if isinstance(f.value, ast.Name) and f.value.id == "self" and f.attr == self.fname:
                    return f"(recurse {args})".replace(" )", ")")
                return f"(m-{m} {self.term(f.value)} {args})".replace(" )", ")")
            return f"(call {self.term(f)} {args})".replace(" )", ")")
        if t is ast.Attribute:
            return f"(attr {self.term(n.value)} {ident(n.attr)})"
        if t in (ast.List, ast.Tuple, ast.Set):
            return f"(list-lit {' '.join(self.term(e) for e in n.elts)})".replace(" )", ")")
        if t is ast.Dict:
            return f"(dict-lit {' '.join(self.term(v) for v in n.values if v is not None)})".replace(" )", ")")
        if t in (ast.ListComp, ast.SetComp, ast.GeneratorExp):
            gens = " ".join(self.term(g.iter) for g in n.generators)
            return f"(comprehension {self.term(n.elt)} {gens})"
        if t is ast.DictComp:
            gens = " ".join(self.term(g.iter) for g in n.generators)
            return f"(comprehension {self.term(n.value)} {gens})"
        if t is ast.IfExp:
            return f"(ifexp {self.term(n.test)} {self.term(n.body)} {self.term(n.orelse)})"
        if t is ast.Lambda:
            return f"(lambda {self.term(n.body)})"
        if t is ast.Starred:
            return f"(starred {self.term(n.value)})"
        if t is ast.JoinedStr:
            return "cstr"
        return "cother"

    def stmts(self, body):
        out = []
        i = 0
        while i < len(body):
            if "swap" in NORM and i + 2 < len(body):
                a, b, c = body[i], body[i + 1], body[i + 2]
                if (all(isinstance(x, ast.Assign) and len(x.targets) == 1 for x in (a, b, c))
                        and isinstance(a.targets[0], ast.Name) and isinstance(c.value, ast.Name)
                        and c.value.id == a.targets[0].id
                        and ast.dump(b.targets[0]) == ast.dump(a.value)
                        and ast.dump(c.targets[0]) == ast.dump(b.value)):
                    out.append(f"(swap {self.term(a.value)} {self.term(b.value)})")
                    i += 3
                    continue
            out.extend(self.stmt(body[i]))
            i += 1
        return out

    def comp_as_loop(self, target_term, comp):
        """[elt for x in it if c] -> res = []; for x in it: if c: res.append(elt)."""
        elt = comp.value if isinstance(comp, ast.DictComp) else comp.elt
        inner = [f"(m-append {target_term} {self.term(elt)})"]
        facts = [f"(assign {target_term} (list-lit))"]
        for g in reversed(comp.generators):
            for cond in reversed(g.ifs):
                t = self.term(cond)
                inner = [f"(guards {t} {f})" for f in inner]
            var = self.term(g.target)
            inner = [f"(iterates {var} {self.term(g.iter)})"] + [f"(in-loop {var} {f})" for f in inner]
        return facts + inner

    def flows(self, tgt, val):
        if "flows" not in NORM or not isinstance(val, ast.AST):
            return []
        base = tgt
        while isinstance(base, (ast.Subscript, ast.Attribute)):
            base = base.value
        if not isinstance(base, ast.Name):
            return []
        return [f"(flows {ident(u)} {ident(base.id)})" for u in names_in(val) if u != base.id]

    def assign_facts(self, tgt, val):
        if isinstance(tgt, (ast.Tuple, ast.List)) and isinstance(val, (ast.Tuple, ast.List)) and len(tgt.elts) == len(val.elts):
            tg = [self.term(e) for e in tgt.elts]
            vl = [self.term(e) for e in val.elts]
            if len(tg) == 2 and tg[0] == vl[1] and tg[1] == vl[0]:
                return [f"(swap {tg[0]} {tg[1]})"]
            return [self.one_assign(a, b) for a, b in zip(tgt.elts, val.elts)]
        return [self.one_assign(tgt, val)]

    def one_assign(self, tgt, val):
        v = self.term(val) if isinstance(val, ast.AST) else val
        if isinstance(tgt, ast.Subscript):
            return f"(store {self.term(tgt)} {v})"
        return f"(assign {self.term(tgt)} {v})"

    def stmt(self, s):
        t = type(s)
        if t is ast.Assign:
            comps = (ast.ListComp, ast.SetComp, ast.GeneratorExp, ast.DictComp)
            if "comp" in NORM and isinstance(s.value, comps) and len(s.targets) == 1 and isinstance(s.targets[0], ast.Name):
                return self.comp_as_loop(self.term(s.targets[0]), s.value) + self.flows(s.targets[0], s.value)
            out = []
            for tgt in s.targets:
                out.extend(self.assign_facts(tgt, s.value))
                out.extend(self.flows(tgt, s.value))
            return out
        if t is ast.AugAssign:
            op = BINOPS.get(type(s.op), "add")
            return [self.one_assign(s.target, f"({op} {self.term(s.target)} {self.term(s.value)})")] + self.flows(s.target, s.value)
        if t is ast.AnnAssign:
            return [self.one_assign(s.target, s.value)] if s.value is not None else []
        if t is ast.Return:
            comps = (ast.ListComp, ast.SetComp, ast.GeneratorExp, ast.DictComp)
            if "comp" in NORM and isinstance(s.value, comps):
                return self.comp_as_loop("_ret", s.value) + ["(returns _ret)"]
            return [f"(returns {self.term(s.value) if s.value is not None else 'cnone'})"]
        if t is ast.Expr:
            if isinstance(s.value, ast.Constant) and isinstance(s.value.value, str):
                return []  # docstring
            if isinstance(s.value, (ast.Yield, ast.YieldFrom)):
                return [f"(yields {self.term(s.value.value) if s.value.value else 'cnone'})"]
            return [f"(do {self.term(s.value)})"]
        if t is ast.If:
            test = self.term(s.test)
            out = [f"(guards {test} {f})" for f in self.stmts(s.body)]
            out += [f"(guards (not {test}) {f})" for f in self.stmts(s.orelse)]
            return out
        if t is ast.While:
            test = self.term(s.test)
            if "while" in NORM:
                vs = names_in(s.test)
                var = ident(vs[0]) if vs else "cnone"
                return [f"(loop-cond {var} {test})"] + [f"(in-loop {var} {f})" for f in self.stmts(s.body) + self.stmts(s.orelse)]
            return [f"(while-loop {test} {f})" for f in self.stmts(s.body) + self.stmts(s.orelse)]
        if t in (ast.For, ast.AsyncFor):
            it = s.iter
            if ("enum" in NORM and isinstance(it, ast.Call) and isinstance(it.func, ast.Name) and it.func.id == "enumerate"
                    and it.args and isinstance(s.target, ast.Tuple) and len(s.target.elts) == 2):
                i_t, v_t = self.term(s.target.elts[0]), self.term(s.target.elts[1])
                seq = self.term(it.args[0])
                out = [f"(iterates {i_t} (bi-range (bi-len {seq})))", f"(in-loop {i_t} (assign {v_t} (index {seq} {i_t})))"]
                out += [f"(in-loop {i_t} {f})" for f in self.stmts(s.body) + self.stmts(s.orelse)]
                return out
            var = self.term(s.target)
            out = [f"(iterates {var} {self.term(s.iter)})"]
            out += [f"(in-loop {var} {f})" for f in self.stmts(s.body) + self.stmts(s.orelse)]
            return out
        if t in (ast.With, ast.AsyncWith):
            return self.stmts(s.body)
        if t is ast.Try:
            out = self.stmts(s.body)
            for h in s.handlers:
                out += [f"(on-error {f})" for f in self.stmts(h.body)]
            return out + self.stmts(s.orelse) + self.stmts(s.finalbody)
        if t is ast.Raise:
            return [f"(raises {self.term(s.exc) if s.exc else 'cnone'})"]
        if t is ast.Assert:
            return [f"(asserts {self.term(s.test)})"]
        if t is ast.Delete:
            return [f"(deletes {self.term(x)})" for x in s.targets]
        if t is ast.Break:
            return ["(break)"]
        if t is ast.Continue:
            return ["(continue)"]
        return []  # nested defs, pass, global, import ...

    def encode(self, fn, inline=0):
        facts = [f"(param {ident(a.arg)})" for a in fn.args.args if a.arg != "self"]
        facts += self.stmts(fn.body)
        if inline > 0:
            seen = set()
            for name, term in list(self.callees):
                if name in seen or name == self.fname:
                    continue
                seen.add(name)
                sub = FuncEncoder(name, self.module_funcs).encode(self.module_funcs[name], inline - 1)
                facts += [f"(inlined {term} {f})" for f in sub if not f.startswith("(param ")]
        return facts


def functions(tree):
    """Top-level functions and class methods (qualified name, node, is_method)."""
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            yield node.name, node, False
        elif isinstance(node, ast.ClassDef):
            for sub in node.body:
                if isinstance(sub, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    yield f"{node.name}.{sub.name}", sub, True


INLINE = int(os.environ.get("MARS_INLINE", "1"))
# Raw mode: library calls keep language-specific names (py:len, py:append, ...),
# declared as unresolved predicates (for vocabulary-alignment experiments).
RAW = os.environ.get("MARS_RAW") == "1"
RAW_USED = set()


def raw(name):
    p = f"py:{ident(name)}"
    RAW_USED.add(p)
    return p
# Normalization passes (comma list): comp, enum, swap, while, flows.
NORM = set(x for x in os.environ.get("MARS_NORM", "").split(",") if x)


def names_in(node):
    return sorted({n.id for n in ast.walk(node) if isinstance(n, ast.Name)})


def main():
    out_dir = sys.argv[1]
    os.makedirs(out_dir, exist_ok=True)
    manifest = []
    used = set()
    for spec in sys.argv[2:]:
        pkg, root = spec.split(":", 1)
        lines = []
        for dirpath, _, files in os.walk(root):
            for fn in sorted(files):
                if not fn.endswith(".py") or fn.startswith("test") or fn == "__init__.py" or fn == "setup.py":
                    continue
                path = os.path.join(dirpath, fn)
                rel = os.path.relpath(path, root)
                category = rel.split(os.sep)[0] if os.sep in rel else "_root"
                stem = fn[:-3]
                try:
                    tree = ast.parse(open(path, encoding="utf-8").read())
                except (SyntaxError, UnicodeDecodeError):
                    continue
                funcs = [(q, n, m) for q, n, m in functions(tree) if n.name not in SKIP_FUNCS]
                if not funcs:
                    continue
                sizes = {q: sum(1 for _ in ast.walk(n)) for q, n, _ in funcs}
                top = [q for q, _, m in funcs if not m]
                main_q = stem if stem in top else (max(top, key=lambda q: sizes[q]) if top else None)
                module_funcs = {n.name: n for q, n, m in funcs if not m}
                for q, node, is_method in funcs:
                    facts = FuncEncoder(node.name, module_funcs).encode(node, INLINE)
                    if len(facts) < 2:
                        continue
                    name = ident(f"{pkg}.{rel[:-3].replace(os.sep, '.')}.{q}")
                    if name in used:
                        continue
                    used.add(name)
                    lines.append(f"(defcase {name}\n  " + "\n  ".join(facts) + ")")
                    manifest.append({"case": name, "pkg": pkg, "category": category, "stem": stem, "func": q,
                                     "is_main": q == main_q, "is_method": is_method, "n_facts": len(facts)})
        with open(os.path.join(out_dir, f"{pkg}.mars"), "w") as f:
            f.write(f";; generated by tools/py2mars.py from {pkg}\n")
            for rp in sorted(RAW_USED):
                f.write(f"(defpredicate {rp} :arity * :kind relation :canonical nil)\n")
            f.write("\n".join(lines) + "\n")
    with open(os.path.join(out_dir, "vocab.mars"), "w") as f:
        f.write(VOCAB)
        for p in EXTRA_PREDS:
            f.write(f"(defpredicate {p} :arity * :kind function)\n")
    with open(os.path.join(out_dir, "manifest.json"), "w") as f:
        json.dump(manifest, f, indent=1)
    print(f"{len(manifest)} functions, {sum(m['is_main'] for m in manifest)} main", file=sys.stderr)


if __name__ == "__main__":
    main()
