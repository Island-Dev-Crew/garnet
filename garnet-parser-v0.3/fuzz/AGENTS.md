# AGENTS.md — Parser Fuzz Harness Contract

## Scope

Provides the `cargo-fuzz` (libfuzzer-sys) harness that drives `garnet-parser-v0.3` against unstructured byte input. The single target, `parse_input`, exercises `garnet_parser::parse_source_with_budget` to surface panics, hangs, and unbounded resource use before they reach users or downstream toolchains.

## Stable Contracts

- The harness is a separate Cargo workspace (`[workspace]` table in `fuzz/Cargo.toml`) so it does not pollute the main `garnet` workspace's build or lock file.
- The `parse_input` target must always wrap the parser call in a strict `ParseBudget` (source-byte cap, token cap, depth cap, literal cap). Removing budgets in pursuit of "deeper" coverage is an explicit anti-goal — the parser's contracts are guarded by those budgets and tests must respect them.
- Seed corpus lives under `corpus/parse_input/` and is seeded from canonical `examples/*.garnet` files. New seeds may be added but every seed must be a real `.garnet` source the parser can legitimately receive.
- Crashes discovered by `cargo fuzz run parse_input` MUST be triaged before merge. Either fix the parser path or commit a `#[ignore]`-equivalent and file an issue — never delete the input quietly.
- The harness must never escape the `garnet-parser-v0.3/fuzz/` subtree (no `fs::write` outside this dir, no network).

## Required Checks

Run before any change in this dir:

```sh
# Sub-workspace structural sanity (no nightly needed):
cd garnet-parser-v0.3/fuzz && cargo metadata --no-deps > /dev/null

# Real fuzz exercise (requires cargo-fuzz + nightly Rust):
cargo +nightly fuzz run parse_input -- -max_total_time=60
# Expect: 0 panics, 0 hangs, memory bounded under default sanitizer limits.
```

CI runs the nightly workflow `.github/workflows/fuzz-nightly.yml` ≥ 1 hour against each push to `main`; crash artifacts are uploaded on any failure.
