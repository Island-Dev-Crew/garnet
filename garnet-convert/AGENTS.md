# AGENTS.md — Converter Contract

## Scope

Owns migration frontends and conversion helpers for lifting Rust, Ruby, Python, and Go source toward Garnet.

## Stable Contracts

- Conversion output must be honest: uncertain mappings should become explicit TODOs, not fake confidence.
- Keep sandboxing assumptions visible; do not execute source language code as part of conversion.
- Preserve provenance from source constructs to generated Garnet where possible.

## Required Checks

```sh
cargo test -p garnet-convert
cargo test -p garnet-cli convert
```
