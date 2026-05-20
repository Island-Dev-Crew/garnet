# AGENTS.md — Parser Fuzz Contract

## Scope

Owns the cargo fuzz harnesses for `garnet-parser-v0.3`, including the `parse_input` target, curated seed corpus, and nightly fuzz workflow evidence.

## Stable Contracts

- Keep fuzz targets deterministic, bounded, and parser-focused; do not turn them into product demos or language conformance claims.
- Keep the seed corpus curated from real Garnet examples unless a crash minimization artifact is promoted intentionally.
- Do not commit expanded libFuzzer-generated corpus files by default; keep generated material in ignored local or CI artifact paths.
- Treat nightly fuzz results as evidence only after the scheduled workflow or an equivalent recorded run completes.

## Required Checks

Run the local S5 dogfood block after changing this harness:

```sh
cargo +nightly fuzz run parse_input -- -max_total_time=60
```

Run the fuzz manifest license check when dependencies or the harness manifest change:

```sh
cargo deny --manifest-path garnet-parser-v0.3/fuzz/Cargo.toml check
```
