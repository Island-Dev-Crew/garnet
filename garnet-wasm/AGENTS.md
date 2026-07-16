# AGENTS.md — garnet-wasm Contract

## Scope

Owns the browser-facing W-PLAY adapters: `run_source`, `check_source`, and
`diff_caps_source`, plus their versioned JSON and `wasm-bindgen` exports.
Consumes the interpreter, parser, and checker as-is; owns no language semantics.

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
- `garnet.wasm.check/1` preserves checker `ok`, diagnostic code, severity, and
  verbatim message; `garnet.wasm.diff-caps/1` preserves both declared surfaces
  and all six `CapsDiff` dimensions. Parse failure is side-specific and carries
  no authority verdict.
- The diff is declared-surface-only: it does not prove absence of undeclared
  authority, and bound annotations are outside this surface.
- Crash surface: `#![deny(clippy::unwrap_used, clippy::expect_used)]`
  (tests exempt via `cfg_attr`).
- Do not claim "runs in your browser" anywhere until a Playwright trap
  proves a browser executed source and rendered real output (next W-PLAY
  slice); the Node smoke proves wasm execution, not the browser page.
- `wasm-opt` is disabled (`Cargo.toml` metadata) — unoptimized module,
  revisited in the page slice; recorded, not silent.
- The committed `docs/playground/pkg` package is generated only by
  `scripts/build_playground_wasm.py`. Its exact three-file inventory is built
  twice, byte-compared, and bound to canonical source/tree digests plus exact
  tool identities. A branch commit SHA is diagnostic output, not package
  identity, because squash merges do not preserve branch ancestry.
## Required Checks

```sh
cargo test -p garnet-wasm
cargo build -p garnet-wasm --target wasm32-unknown-unknown
python scripts/build_playground_wasm.py --probe
python scripts/build_playground_wasm.py --verify-reproducible
```
