# MARS: working notes for agents

Research codebase for a structural analogical memory (see `README.md`, `docs/DESIGN.md`).

- **Track work in `docs/PROGRESS.md`** (task board, decision log, experiment log). Update it when tasks finish or decisions change.
- Experiment outputs go in `results/` (a markdown summary plus JSON). Every result must record its config and seed.

## Commands

```bash
cargo build --release                      # all crates
cargo test --workspace -q                  # unit/property tests (fast)
cargo run --release -p mars-bench -- e0    # experiment E0 (see docs/EXPERIMENTS.md)
cargo clippy --workspace --all-targets     # lint
```

## Conventions

- Determinism: every random choice derives from an explicit `u64` seed (`mars_hv::Rng`, `mars_hv::rng::hash_*`). Never use time or thread-dependent randomness.
- Feature hashes are name-based (`hash_str`), so they are stable across knowledge bases.
- Hot loops work on `&[u64]` word slices; `count_ones()` compiles to VPOPCNTQ via `.cargo/config.toml` (`target-cpu=native`).
- Crates: `mars-hv` (bits) → `mars-rel` (representation) → `mars-encode` (features + fingerprints) → `mars-gen` (synthetic data) → `mars-bench` (experiments). Later: `mars-map`, `mars-index`, `mars-tms`, `mars-engine`.
