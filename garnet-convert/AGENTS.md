# AGENTS.md — Converter Contract

## Scope

Owns migration frontends and conversion helpers for lifting Rust, Ruby, Python, and Go source toward Garnet.

## Stable Contracts

- Conversion output must not overclaim: uncertain mappings should become explicit TODOs, not fake confidence.
- Keep sandboxing assumptions visible; do not execute source language code as part of conversion.
- Preserve provenance from source constructs to generated Garnet where possible.
- Every emitted file parses as Garnet (C1-18): the emitter parses its own output
  and refuses to emit text that does not parse.
- A statement the frontend cannot lower is kept whole as one `@migrate_todo`
  carrying its source lines; no fragment of it may stay active code. Blocks are
  recognized on a line's code without its trailing comment, a Python statement
  that leaves a bracket open (or ends in `\`) runs on until it balances, and a Go
  line that opens a block or a bracket is read on until its brackets balance.
  Cover a new construct with a test in `tests/parses.rs` that asserts its
  fragments are not active (`assert_inactive`).

## Required Checks

```sh
cargo test -p garnet-convert
cargo test -p garnet-cli convert
```
