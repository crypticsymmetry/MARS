# mars-py: Python bindings for MARS

`import mars` exposes the MARS engine: fingerprint retrieval (Mode K), structure mapping, standing queries with truth-maintained candidate inferences, and persistence.

## Build

```bash
pip install maturin
cd crates/mars-py && maturin build --release -o dist
pip install dist/mars_analogy-*.whl       # abi3 wheel: any CPython >= 3.9
python3 python/tests/test_mars.py         # or: python3 -m pytest python/tests
```

The `extension-module` feature is turned on only by maturin (`pyproject.toml`), so `cargo test --workspace` and clippy still link against libpython normally.

## API

| call | returns |
|---|---|
| `Engine(source="")`, `Engine.from_files([paths])`, `Engine.open(dir)` | an engine over `.mars` text / files / a checkpointed store |
| `e.checkpoint(dir)` | writes a snapshot, then logs every later mutation there |
| `e.add_case("(defcase ...)")`, `e.add_fact(case, "(pred a b)")`, `e.remove_fact(...)`, `e.remove_case(case)`, `e.apply(record)` | mutations (the same records as `mars serve`) |
| `e.cases()`, `e.render(case)`, `len(e)` | live case names, `.mars` text of a case, number of live cases |
| `e.query(case, k=5)` | `([(name, fused score)], z)`; accept the top-1 when `z >= mars.SIGNIFICANT_Z` (E16), `z` is `None` if the memory is too small |
| `e.fac(base, target)` | normalized structural (FAC) score |
| `e.map(base, target)` | dict: `score`, `entities` [(base, target)], `matches` [(base fact, target fact)], `inferences` [(fact, support, has_skolem)], `differences` |
| `sq = e.watch(case, k=5)` | id of a standing query maintained incrementally |
| `e.top(sq)`, `e.infer(sq, min_support=1)`, `e.explain(sq, fact)`, `e.events()` | its current analogues, corroborated inferences (support = number of agreeing analogues, E11), JTMS provenance, drained change events |

Unknown cases or standing queries raise `KeyError`; parse and I/O errors raise `ValueError`. An `Engine` is not thread-safe (`unsendable`); use one per thread.

## Example

```python
import mars
e = mars.Engine.from_files(["data/examples/classic.mars"])
hits, z = e.query("rutherford-atom", k=3)
m = e.map("solar-system", "rutherford-atom")
print(dict(m["entities"])["sun"])            # nucleus
for fact, support, skolem in m["inferences"]:
    print(fact, support)                     # e.g. (cause ... (revolve-around electron nucleus))
sq = e.watch("rutherford-atom", k=2)
e.add_case("(defcase binary-star (attracts star-a star-b) (revolve-around star-b star-a))")
print(e.top(sq), e.events())
```
