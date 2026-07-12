# AGENTS.md — garnet-wasm Contract

## Scope

Owns the browser-facing interpreter surface for the W-PLAY playground:
`run_source` (Garnet source in → `garnet.wasm.run/1` JSON out) and its
`wasm-bindgen` export. Consumes `garnet-interp` as-is; owns no language
semantics.

## Stable Contracts

- `run_source` loads user source under the `main` entry's `@caps` frame
  (`load_source_with_entry_caps` + `call_entry`) — the same authority gate
  the CLI run lane applies. Never load outside the entry gate.
- The returned `stdout` is REAL captured program output from
  `garnet_interp::output` — never synthesized, truncated, or reordered. If
  execution fails, the result carries the partial real output plus the
  diagnostic; the playground renders what actually happened.
- The wasm interpreter environment carries no proc/fs/net natives: an
  authority-bearing call fails closed (unresolved / trapped), and this crate
  must never polyfill OS authority into the browser environment.
- `garnet.wasm.run/1` is a consumed schema (playground JS): `schema`,
  `exit_class` (`ok` | `load_error` | `runtime_error`), `stdout`,
  `diagnostic`. Additive evolution only; bump the schema tag on any
  breaking change.
- Crash surface: `#![deny(clippy::unwrap_used, clippy::expect_used)]`
  (tests exempt via `cfg_attr`).
- Do not claim "runs in your browser" anywhere until a Playwright trap
  proves a browser executed source and rendered real output (next W-PLAY
  slice); the Node smoke proves wasm execution, not the browser page.
- `wasm-opt` is disabled (`Cargo.toml` metadata) — unoptimized module,
  revisited in the page slice; recorded, not silent.

## Required Checks

```sh
cargo test -p garnet-wasm
cargo build -p garnet-wasm --target wasm32-unknown-unknown
wasm-pack build garnet-wasm --target web --out-dir pkg-web
```
