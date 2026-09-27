# MARS: working notes for agents

Research codebase for a structural analogical memory (see `README.md`, `docs/DESIGN.md`).

- **Track work in `docs/PROGRESS.md`** (task board, decision log, experiment log). Update it when tasks finish or decisions change.
- Experiment outputs go in `results/` (a markdown summary plus JSON). Every result must record its config and seed.

## Commands

```bash
cargo build --release                      # all crates
cargo test --workspace -q                  # unit/property tests (fast)
cargo run --release -p mars-bench -- e0    # experiments: e0 e2 e3 e5 e6 e7 e9, bw (see crates/mars-bench/src/main.rs)
./target/release/mars analogies data/examples/classic.mars --case rutherford-atom   # CLI demo
scripts/e4_sweep.sh                        # E4 perturbation sweep
tools/fetch_e9_corpus.sh                   # real-code corpus for E9 (PyPI; sources git-ignored)
cargo clippy --workspace --all-targets     # lint
(cd crates/mars-py && maturin build --release -o dist) && pip install --force-reinstall crates/mars-py/dist/*.whl
python3 crates/mars-py/python/tests/test_mars.py                           # Python binding tests
```

## Conventions

- Determinism: every random choice derives from an explicit `u64` seed (`mars_hv::Rng`, `mars_hv::rng::hash_*`). Never use time or thread-dependent randomness.
- Feature hashes are name-based (`hash_str`), so they are stable across knowledge bases.
- Hot loops work on `&[u64]` word slices; `count_ones()` compiles to VPOPCNTQ via `.cargo/config.toml` (`target-cpu=native`).
- Crates: `mars-hv` (bits) → `mars-rel` (representation) → `mars-encode` (features + fingerprints) → `mars-index` (retrieval) / `mars-map` (mapper) → `mars-tms` → `mars-engine` (incremental memory, SAGE) → `mars-bench` (experiments), `mars-cli` (tool), `mars-py` (Python bindings; `extension-module` is enabled only by maturin so `cargo test` links libpython); `mars-gen` generates synthetic data.
- Never `pkill -f` a pattern that also appears in your own shell command line (it kills the shell). Never use `GROUPS` as a bash variable (it is reserved).
