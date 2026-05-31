# S64 Plan — WASI interop (closes native-interop band)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S64 (v0.8.1 band).
Map: reconciled plan §163-164 — native-interop boundary (WASI).
Branch: `codex/s64-wasi`. Base: `origin/main` @ `2231045` (S63).

## Approach (authority mapping, not runtime)
WASI is capability-oriented → @caps map directly to WASI host capabilities. The
S46 sandbox `wasi` policy IS the mapping. Prove it; defer the wasm runtime (S55).

## Deliverables
- `examples/ffi/wasi_clock.garnet` — @caps(time, fs); checks clean.
- `C_Language_Specification/GARNET_WASI_INTEROP.md` — @caps→WASI mapping table +
  honest "no WASI runtime" boundary.
- `garnet-cli/tests/wasi_interop.rs` — 2 cross-OS tests (check clean; sandbox
  wasi policy clocks:true preopens:true sockets:false).

## Dogfood
- check clean; sandbox wasi = {clocks:true, preopens:true, sockets:false, ...}.

## Honest scope (do not soften)
- WASI authority MAPPING, NOT a WASI runtime. No wasm build / WASI host execution
  (wasm32/wasm-pack/wasmtime absent). Deferred. Closes the native-interop
  authority band (S62/S63/S64 = authority/attestation, no runtime). No new lane.

## Gates
- Rust tests + ladder (workspace 0 failed). Ledger: `s63 → merged(5)` advanced;
  `s64` rides with S65.
