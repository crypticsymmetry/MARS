#!/usr/bin/env python3
"""JavaScript (ESTree JSON from tools/js_ast.js) -> MARS relational cases.

Uses the *same* relational vocabulary as tools/py2mars.py, and maps JS idioms
onto it so that cross-language comparison tests structure, not syntax:
  * `for (i = a; i < b; i++)` / `i += s` / `i--` -> (iterates i (bi-range a b [s]))
  * `x.length` -> (bi-len x); `Math.min/max/abs/floor/pow` -> builtins
  * `a.push(x)` -> m-append, `pop/shift` -> m-pop, `unshift` -> m-insert,
    `slice(a,b)` -> (slice x a b), `concat` -> add, `indexOf` -> m-index
  * comparator objects: `c.greaterThan(a,b)` -> (gt a b), lessThan -> lt, equal -> eq
  * temp-variable swaps and `[a,b] = [b,a]` -> (swap a b); `this` -> self
Usage: js2mars.py AST_JSONL PKG OUT_DIR   (appends to OUT_DIR/manifest.json)
"""
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(__file__))
from py2mars import ident  # noqa: E402

ARITH = {"+": "add", "-": "sub", "*": "mul", "/": "div", "%": "mod", "**": "pow", "<<": "lshift", ">>": "rshift",
         ">>>": "rshift", "&": "bitand", "|": "bitor", "^": "bitxor"}
CMP = {"<": "lt", "<=": "le", ">": "gt", ">=": "ge", "==": "eq", "===": "eq", "!=": "ne", "!==": "ne", "in": "in", "instanceof": "is"}
METHODS = {"push": "append", "pop": "pop", "shift": "pop", "unshift": "insert", "indexOf": "index", "sort": "sort",
           "reverse": "reverse", "join": "join", "split": "split", "keys": "keys", "values": "values", "add": "add",
           "get": "get", "has": "get", "set": "update", "toLowerCase": "lower", "toUpperCase": "upper", "trim": "strip"}
COMPARATOR = {"greaterThan": "gt", "lessThan": "lt", "greaterThanOrEqual": "ge", "lessThanOrEqual": "le", "equal": "eq"}
MATH = {"min": "min", "max": "max", "abs": "abs", "pow": "pow", "round": "round", "floor": "int", "ceil": "int", "trunc": "int"}
GLOBALS = {"parseInt": "int", "parseFloat": "float", "String": "str", "Number": "float", "Boolean": "bool"}


def const(v):
    if v is None:
        return "cnone"
    if isinstance(v, bool):
        return "cbool"
    if isinstance(v, (int, float)):
        if float(v).is_integer():
            return {0: "c0", 1: "c1"}.get(int(v), "cint")
        return "cfloat"
    return "cstr"


class Enc:
    def __init__(self, fname, module_funcs=None):
        self.fname = fname
        self.module_funcs = module_funcs or {}
        self.callees = []

    def args(self, xs):
        return " ".join(self.term(a) for a in xs)

    def call(self, pred, args):
        return f"({pred} {args})".replace(" )", ")")

    def term(self, n):
        if n is None:
            return "cnone"
        t = n["type"]
        if t == "Identifier":
            return "self" if n["name"] == "this" else ident(n["name"])
        if t == "ThisExpression":
            return "self"
        if t == "Literal":
            return const(n.get("value")) if "regex" not in n else "cstr"
        if t == "TemplateLiteral":
            return "cstr"
        if t == "BinaryExpression":
            op = n["operator"]
            if op in CMP:
                return f"({CMP[op]} {self.term(n['left'])} {self.term(n['right'])})"
            return f"({ARITH.get(op, 'add')} {self.term(n['left'])} {self.term(n['right'])})"
        if t == "LogicalExpression":
            op = {"&&": "and", "||": "or"}.get(n["operator"], "or")
            return f"({op} {self.term(n['left'])} {self.term(n['right'])})"
        if t == "UnaryExpression":
            op = {"!": "not", "-": "neg", "+": "pos", "~": "invert"}.get(n["operator"])
            return f"({op} {self.term(n['argument'])})" if op else self.term(n["argument"])
        if t == "UpdateExpression":
            return self.term(n["argument"])
        if t == "AssignmentExpression":
            return self.term(n["left"])
        if t == "MemberExpression":
            obj = self.term(n["object"])
            if n.get("computed"):
                return f"(index {obj} {self.term(n['property'])})"
            prop = n["property"].get("name", "p")
            if prop == "length":
                return f"(bi-len {obj})"
            return f"(attr {obj} {ident(prop)})"
        if t in ("CallExpression", "NewExpression"):
            c = n["callee"]
            args = n.get("arguments", [])
            if t == "NewExpression":
                if c.get("type") == "Identifier" and c["name"] in ("Array",):
                    return "(list-lit)"
                return self.call("call", (ident(c.get("name", "ctor")) + " " + self.args(args)).strip())
            if c["type"] == "Identifier":
                name = c["name"]
                if name == self.fname:
                    return self.call("recurse", self.args(args))
                if name in GLOBALS:
                    return self.call(f"bi-{GLOBALS[name]}", self.args(args))
                term = self.call("call", (ident(name) + " " + self.args(args)).strip())
                if name in self.module_funcs:
                    self.callees.append((name, term))
                return term
            if c["type"] == "MemberExpression" and not c.get("computed"):
                obj, m = c["object"], c["property"].get("name", "other")
                if obj.get("type") == "Identifier" and obj["name"] == "Math" and m in MATH:
                    if m == "floor" and args and args[0]["type"] == "BinaryExpression" and args[0]["operator"] == "/":
                        return f"(floordiv {self.term(args[0]['left'])} {self.term(args[0]['right'])})"
                    return self.call(f"bi-{MATH[m]}", self.args(args))
                if obj.get("type") == "Identifier" and obj["name"] == "console":
                    return self.call("bi-print", self.args(args))
                if m in COMPARATOR and len(args) == 2:
                    return f"({COMPARATOR[m]} {self.term(args[0])} {self.term(args[1])})"
                if m == "slice":
                    lo = self.term(args[0]) if args else "cnone"
                    hi = self.term(args[1]) if len(args) > 1 else "cnone"
                    return f"(slice {self.term(obj)} {lo} {hi})"
                if m == "concat":
                    return f"(add {self.term(obj)} {self.term(args[0]) if args else 'cnone'})"
                if obj.get("type") == "Identifier" and obj["name"] == self.fname:
                    return self.call("recurse", self.args(args))
                if m == self.fname and obj.get("type") == "ThisExpression":
                    return self.call("recurse", self.args(args))
                return self.call(f"m-{METHODS.get(m, 'other')}", (self.term(obj) + " " + self.args(args)).strip())
            return self.call("call", (self.term(c) + " " + self.args(args)).strip())
        if t == "ArrayExpression":
            return self.call("list-lit", self.args([e for e in n["elements"] if e]))
        if t == "ObjectExpression":
            return "(dict-lit)"
        if t == "ConditionalExpression":
            return f"(ifexp {self.term(n['test'])} {self.term(n['consequent'])} {self.term(n['alternate'])})"
        if t in ("FunctionExpression", "ArrowFunctionExpression"):
            return "(lambda cother)"
        if t == "SequenceExpression":
            return self.term(n["expressions"][-1])
        if t == "SpreadElement":
            return f"(starred {self.term(n['argument'])})"
        return "cother"

    # ---------------------------------------------------------------- statements
    def target(self, lhs, rhs_term):
        if lhs["type"] == "MemberExpression" and lhs.get("computed"):
            return f"(store {self.term(lhs)} {rhs_term})"
        return f"(assign {self.term(lhs)} {rhs_term})"

    def assign_expr(self, e):
        op = e["operator"]
        lhs, rhs = e["left"], e["right"]
        if lhs["type"] == "ArrayPattern" and rhs["type"] == "ArrayExpression" and len(lhs["elements"]) == 2 == len(rhs["elements"]):
            a, b = [self.term(x) for x in lhs["elements"]]
            c, d = [self.term(x) for x in rhs["elements"]]
            if a == d and b == c:
                return [f"(swap {a} {b})"]
        if op == "=":
            return [self.target(lhs, self.term(rhs))]
        arith = ARITH.get(op[:-1], "add")
        return [self.target(lhs, f"({arith} {self.term(lhs)} {self.term(rhs)})")]

    def expr_stmt(self, e):
        t = e["type"]
        if t == "AssignmentExpression":
            return self.assign_expr(e)
        if t == "UpdateExpression":
            op = "add" if e["operator"] == "++" else "sub"
            return [self.target(e["argument"], f"({op} {self.term(e['argument'])} c1)")]
        if t == "SequenceExpression":
            return [f for x in e["expressions"] for f in self.expr_stmt(x)]
        return [f"(do {self.term(e)})"]

    def block(self, s):
        if s is None:
            return []
        body = s["body"] if s["type"] == "BlockStatement" else [s]
        out, i = [], 0
        while i < len(body):
            swap = self.temp_swap(body, i)
            if swap:
                out.append(swap)
                i += 3
                continue
            out.extend(self.stmt(body[i]))
            i += 1
        return out

    def simple_assign(self, s):
        """(lhs node, rhs node) for `x = e;` / `var x = e;` statements."""
        if s["type"] == "ExpressionStatement" and s["expression"]["type"] == "AssignmentExpression" and s["expression"]["operator"] == "=":
            return s["expression"]["left"], s["expression"]["right"]
        if s["type"] == "VariableDeclaration" and len(s["declarations"]) == 1 and s["declarations"][0].get("init"):
            d = s["declarations"][0]
            return d["id"], d["init"]
        return None

    def temp_swap(self, body, i):
        if i + 2 >= len(body):
            return None
        a, b, c = (self.simple_assign(x) for x in body[i:i + 3])
        if not (a and b and c) or a[0]["type"] != "Identifier" or c[1]["type"] != "Identifier" or c[1]["name"] != a[0]["name"]:
            return None
        if self.term(b[0]) == self.term(a[1]) and self.term(c[0]) == self.term(b[1]):
            return f"(swap {self.term(a[1])} {self.term(b[1])})"
        return None

    def for_range(self, s):
        """Canonical counting loop -> (var, range term), else None."""
        init, test, upd = s.get("init"), s.get("test"), s.get("update")
        if not (init and test and upd):
            return None
        if init["type"] == "VariableDeclaration" and len(init["declarations"]) == 1:
            var, start = init["declarations"][0]["id"], init["declarations"][0].get("init")
        elif init["type"] == "AssignmentExpression" and init["operator"] == "=":
            var, start = init["left"], init["right"]
        else:
            return None
        if var["type"] != "Identifier" or start is None or test["type"] != "BinaryExpression":
            return None
        v = var["name"]
        if test["left"].get("name") != v:
            return None
        bound = self.term(test["right"])
        if test["operator"] == "<=":
            bound = f"(add {bound} c1)"
        elif test["operator"] == ">=":
            bound = f"(sub {bound} c1)"
        step = None
        if upd["type"] == "UpdateExpression" and upd["argument"].get("name") == v:
            step = None if upd["operator"] == "++" else "(neg c1)"
        elif upd["type"] == "AssignmentExpression" and upd["left"].get("name") == v and upd["operator"] in ("+=", "-="):
            step = self.term(upd["right"]) if upd["operator"] == "+=" else f"(neg {self.term(upd['right'])})"
        else:
            return None
        rng = f"(bi-range {self.term(start)} {bound}{'' if step is None else ' ' + step})"
        return ident(v), rng

    def stmt(self, s):
        t = s["type"]
        if t == "VariableDeclaration":
            out = []
            for d in s["declarations"]:
                if d.get("init") is None or d["init"]["type"] in ("FunctionExpression", "ArrowFunctionExpression"):
                    continue
                if d["id"]["type"] == "Identifier":
                    out.append(f"(assign {ident(d['id']['name'])} {self.term(d['init'])})")
            return out
        if t == "ExpressionStatement":
            return self.expr_stmt(s["expression"])
        if t == "ReturnStatement":
            return [f"(returns {self.term(s.get('argument'))})"]
        if t == "IfStatement":
            test = self.term(s["test"])
            out = [f"(guards {test} {f})" for f in self.block(s["consequent"])]
            out += [f"(guards (not {test}) {f})" for f in self.block(s.get("alternate"))]
            return out
        if t == "ForStatement":
            r = self.for_range(s)
            body = self.block(s["body"])
            if r:
                v, rng = r
                return [f"(iterates {v} {rng})"] + [f"(in-loop {v} {f})" for f in body]
            pre = self.stmt({"type": "ExpressionStatement", "expression": s["init"]}) if s.get("init") and s["init"]["type"] != "VariableDeclaration" else (self.stmt(s["init"]) if s.get("init") else [])
            upd = self.expr_stmt(s["update"]) if s.get("update") else []
            test = self.term(s.get("test")) if s.get("test") else "cbool"
            return pre + [f"(while-loop {test} {f})" for f in body + upd]
        if t in ("ForInStatement", "ForOfStatement"):
            left = s["left"]
            var = ident(left["declarations"][0]["id"].get("name", "v")) if left["type"] == "VariableDeclaration" else self.term(left)
            it = self.term(s["right"])
            if t == "ForInStatement":
                it = f"(bi-range (bi-len {it}))"
            return [f"(iterates {var} {it})"] + [f"(in-loop {var} {f})" for f in self.block(s["body"])]
        if t in ("WhileStatement", "DoWhileStatement"):
            test = self.term(s["test"])
            return [f"(while-loop {test} {f})" for f in self.block(s["body"])]
        if t == "BlockStatement":
            return self.block(s)
        if t == "BreakStatement":
            return ["(break)"]
        if t == "ContinueStatement":
            return ["(continue)"]
        if t == "ThrowStatement":
            return [f"(raises {self.term(s['argument'])})"]
        if t == "TryStatement":
            out = self.block(s["block"])
            if s.get("handler"):
                out += [f"(on-error {f})" for f in self.block(s["handler"]["body"])]
            return out + self.block(s.get("finalizer"))
        if t == "SwitchStatement":
            d = self.term(s["discriminant"])
            out = []
            for c in s["cases"]:
                test = f"(eq {d} {self.term(c['test'])})" if c.get("test") else "cbool"
                for x in c["consequent"]:
                    out += [f"(guards {test} {f})" for f in self.stmt(x)]
            return out
        return []

    def encode(self, fn, inline=1):
        facts = [f"(param {ident(p.get('name', 'p'))})" for p in fn["params"] if p["type"] == "Identifier"]
        body = fn["body"]
        if body["type"] != "BlockStatement":
            facts.append(f"(returns {self.term(body)})")
        else:
            facts += self.block(body)
        if inline > 0:
            seen = set()
            for name, term in list(self.callees):
                if name in seen or name == self.fname:
                    continue
                seen.add(name)
                sub = Enc(name, self.module_funcs).encode(self.module_funcs[name], inline - 1)
                facts += [f"(inlined {term} {f})" for f in sub if not f.startswith("(param ")]
        return facts


def functions(ast):
    """(qualified name, function node) for top-level and prototype/object functions."""
    out = []

    def fn_node(n):
        return n and n.get("type") in ("FunctionExpression", "ArrowFunctionExpression", "FunctionDeclaration")

    def visit(node, prefix=""):
        if isinstance(node, list):
            for x in node:
                visit(x, prefix)
            return
        if not isinstance(node, dict):
            return
        t = node.get("type")
        if t == "FunctionDeclaration" and node.get("id"):
            out.append((prefix + node["id"]["name"], node))
        elif t == "VariableDeclarator" and fn_node(node.get("init")) and node["id"].get("type") == "Identifier":
            out.append((prefix + node["id"]["name"], node["init"]))
        elif t == "AssignmentExpression" and fn_node(node.get("right")):
            left = node["left"]
            name = left.get("name") or (left.get("property") or {}).get("name")
            if name:
                owner = (left.get("object") or {}).get("object", {}).get("name") if left.get("type") == "MemberExpression" else None
                out.append(((owner + "." if owner else "") + name, node["right"]))
        elif t == "Property" and fn_node(node.get("value")):
            k = node.get("key", {})
            out.append((k.get("name") or str(k.get("value")), node["value"]))
        elif t == "MethodDefinition":
            out.append((node["key"].get("name", "m"), node["value"]))
        for k, v in node.items():
            if k not in ("loc", "start", "end") and isinstance(v, (dict, list)):
                visit(v, prefix)

    visit(ast)
    return out


def exported_name(ast):
    for st in ast.get("body", []):
        e = st.get("expression") if st.get("type") == "ExpressionStatement" else None
        if e and e.get("type") == "AssignmentExpression" and e["left"].get("type") == "MemberExpression":
            obj = e["left"].get("object", {})
            if obj.get("name") == "module" and e["right"].get("type") == "Identifier":
                return e["right"]["name"]
    return None


def size(n):
    return sum(size(v) for v in n.values() if isinstance(v, (dict, list))) + 1 if isinstance(n, dict) else sum(size(x) for x in n) if isinstance(n, list) else 0


def main():
    jsonl, pkg, out_dir = sys.argv[1], sys.argv[2], sys.argv[3]
    man_path = os.path.join(out_dir, "manifest.json")
    manifest = json.load(open(man_path)) if os.path.exists(man_path) else []
    manifest = [m for m in manifest if m.get("pkg") != pkg]
    used = {m["case"] for m in manifest}
    lines = []
    for line in open(jsonl):
        rec = json.loads(line)
        rel, ast = rec["file"], rec["ast"]
        parts = rel.split("/")
        category = parts[-2] if len(parts) > 1 else "_root"
        stem = parts[-1][:-3]
        funcs = functions(ast)
        if not funcs:
            continue
        exp = exported_name(ast)
        names = [q for q, _ in funcs]
        main_q = exp if exp in names else max(funcs, key=lambda f: size(f[1]))[0]
        module_funcs = {q: n for q, n in funcs if "." not in q}
        for q, node in funcs:
            facts = Enc(q.split(".")[-1], module_funcs).encode(node, int(os.environ.get("MARS_INLINE", "1")))
            if len(facts) < 2:
                continue
            name = ident(f"{pkg}.{rel[:-3].replace('/', '.')}.{q}")
            if name in used:
                continue
            used.add(name)
            lines.append(f"(defcase {name}\n  " + "\n  ".join(facts) + ")")
            manifest.append({"case": name, "pkg": pkg, "lang": "js", "category": category, "stem": stem, "func": q,
                             "is_main": q == main_q, "is_method": "." in q, "n_facts": len(facts)})
    with open(os.path.join(out_dir, f"{pkg}.mars"), "w") as f:
        f.write(f";; generated by tools/js2mars.py from {pkg}\n" + "\n".join(lines) + "\n")
    json.dump(manifest, open(man_path, "w"), indent=1)
    print(f"{pkg}: {len(lines)} functions", file=sys.stderr)


if __name__ == "__main__":
    main()
