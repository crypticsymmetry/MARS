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


if __name__ == "__main__":
    for name, fn in list(globals().items()):
        if name.startswith("test_"):
            fn()
            print("ok", name)
