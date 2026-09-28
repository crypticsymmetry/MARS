"""Tests for the MARS Python bindings (run: python3 -m pytest crates/mars-py/python/tests
or python3 crates/mars-py/python/tests/test_mars.py from the repository root)."""

import os
import tempfile

import mars

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", ".."))
CLASSIC = os.path.join(ROOT, "data", "examples", "classic.mars")


def engine():
    return mars.Engine.from_files([CLASSIC])


def test_query_finds_the_classic_analogue():
    e = engine()
    hits, z = e.query("rutherford-atom", k=3)
    assert hits[0][0] == "solar-system"
    assert all(name != "rutherford-atom" for name, _ in hits)
    assert z is None or isinstance(z, float)  # 7 cases: too few candidates for a local null


def test_map_returns_correspondences_and_inferences():
    m = engine().map("solar-system", "rutherford-atom")
    assert m["score"] > 0
    ents = dict(m["entities"])
    assert ents.get("sun") == "nucleus" and ents.get("planet") == "electron"
    assert any("cause" in text for text, _, _ in m["inferences"])


def test_standing_query_updates_incrementally():
    e = engine()
    sq = e.watch("rutherford-atom", k=2)
    before = e.top(sq)
    e.add_case("(defcase binary-star (attracts star-a star-b) (greater (mass star-a) (mass star-b)) (revolve-around star-b star-a) "
               "(cause (and (attracts star-a star-b) (greater (mass star-a) (mass star-b))) (revolve-around star-b star-a)))")
    after = e.top(sq)
    assert "binary-star" in [n for n, _ in after]
    assert before != after
    assert e.events(), "result changes produce events"
    for text, support in e.infer(sq):
        assert support >= 1
        assert e.explain(sq, text)


def test_checkpoint_and_reopen_reproduce_results():
    e = engine()
    sq = e.watch("rutherford-atom", k=2)
    with tempfile.TemporaryDirectory() as d:
        e.checkpoint(d)
        e.remove_case("mere-appearance-sun")
        e.add_fact("water-flow", "(flat-top water)")
        top = e.top(sq)
        r = mars.Engine.open(d)
        assert r.top(sq) == top
        assert len(r) == len(e)


def test_errors_are_python_exceptions():
    e = engine()
    for bad in (lambda: e.query("no-such-case"), lambda: e.top(99)):
        try:
            bad()
        except KeyError:
            continue
        raise AssertionError("expected KeyError")
    try:
        e.add_case("(defcase broken (unbalanced")
    except ValueError:
        pass
    else:
        raise AssertionError("expected ValueError")


def test_feedback_learns_which_transfers_to_trust():
    decl = "".join(f"(defpredicate {p} :arity 2 :kind relation)" for p in ("director", "writer", "producer"))
    films = "".join(f"(defcase f{i} (director f{i} d{i}) (writer f{i} d{i}) (producer f{i} p{i}))" for i in range(8))
    e = mars.Engine(decl + films + "(defcase q1 (director q1 dq1)) (defcase q2 (director q2 dq2))", first_order=True)
    e.remove_case("q1")
    e.remove_case("q2")
    s1 = {i["text"]: i for i in e.suggest("q1", k=4)}
    assert s1["(writer q1 dq1)"]["transfers"] == ["writer<=director"]
    copy = next(t for t in s1 if t.startswith("(producer q1"))
    e.feedback("q1", "(writer q1 dq1)", True)
    e.feedback("q1", copy, False)
    s2 = e.suggest("q2", k=4)
    assert s2[0]["text"] == "(writer q2 dq2)"
    assert s2[0]["reliability"] > next(i for i in s2 if i["text"].startswith("(producer"))["reliability"]
    for _ in range(10):
        e.feedback("q2", "(writer q2 dq2)", True)
    assert e.induced_rules(min_n=5, min_precision=0.9)[0][0] == "writer(x, y) ⇐ director(x, y)"
    assert [i["text"] for i in e.rule_suggestions("q1", min_n=5, min_precision=0.9)] == ["(writer q1 dq1)"]
    with tempfile.TemporaryDirectory() as d:
        e.checkpoint(d)
        e.feedback("q2", "(writer q2 dq2)", True)
        r = mars.Engine.open(d, first_order=True)
        assert r.induced_rules(min_n=5) == e.induced_rules(min_n=5)


if __name__ == "__main__":
    for name, fn in list(globals().items()):
        if name.startswith("test_"):
            fn()
            print("ok", name)
