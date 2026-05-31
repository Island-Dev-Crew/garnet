# Garnet WASI interop (S64)

WASI is the *capability-oriented* native boundary: a wasm guest gets exactly the
host facilities it is granted, nothing more. That maps cleanly onto Garnet's
`@caps`. S64 closes the native-interop band (S61–S64) by making the
`@caps` → WASI capability mapping explicit — the wasm-side counterpart to the
C ABI (S63).

## The mapping (`@caps` → WASI host capabilities)

A Garnet program's declared `@caps` are exactly the WASI capabilities it would
request from a wasm host. The S46 sandbox `wasi` policy **is** this mapping:

| `@caps(...)` | WASI host capability |
|---|---|
| `fs` | preopened directories (`preopens`) |
| `net` / `net_internal` | `wasi-sockets` |
| `time` | `wasi clocks` |
| `env` | environment variables |
| *(always)* | inherited `stdio` |

So a `@caps(time, fs)` program (`examples/ffi/wasi_clock.garnet`) requests WASI
clocks + preopens and **not** sockets — the host can grant precisely that, and a
`diff-caps` gain (S37) of `net` would be visible before it could reach
`wasi-sockets`.

## The proof

`garnet-cli/tests/wasi_interop.rs` (cross-OS) shows `wasi_clock.garnet` checks
clean and that `garnet sandbox`'s WASI policy reflects the caps: `clocks: true`
(time), `preopens: true` (fs), `sockets: false` (no net declared). The WASI
authority surface is derived directly from the declared capabilities.

## Honest scope (do not soften)

This is the WASI **authority mapping**, not a WASI **runtime**. Garnet does not
compile to wasm here and does not run under a WASI host — `wasm32`/`wasm-pack`/
`wasmtime` are absent (S55), and the interpreter executes nothing under WASI. S64
ships the `@caps` → WASI capability mapping + its proof; the actual wasm build and
WASI host execution are **deferred** (the path is in `GARNET_WASM_TARGET.md`).
This closes the native-interop *authority* band (Rust S62, C ABI S63, WASI S64);
each ships the authority/attestation half, none ship a native/wasm runtime.
