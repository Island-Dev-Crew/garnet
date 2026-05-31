# S62 Plan — Rust FFI proof

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S62 (v0.8.1 band).
Map: reconciled plan §163-164 — native-interop boundary (Rust FFI).
Branch: `codex/s62-rust-ffi`. Base: `origin/main` @ `f319ee1` (S61).

## Approach (attestation proof, not runtime)
No FFI runtime → prove the AUTHORITY + ATTESTATION half: a @caps(ffi) Rust-wrapper
is sealed (the in-toto predicate's capability manifest attests `ffi`).

## Deliverables
- `examples/ffi/rust_extern.garnet` — @caps(ffi) Rust-wrapper (stand-in body),
  checks clean + runs.
- `C_Language_Specification/GARNET_RUST_FFI.md` — the Rust binding design (C ABI,
  @caps(ffi) mandatory, seal attestation) + honest "no runtime" boundary.
- `garnet-cli/tests/rust_ffi_proof.rs` — 2 cross-OS tests (checks clean + runs;
  seal predicate attests ffi).

## Dogfood
- check clean; run → "payload"; `seal` predicate caps aggregate = ["ffi"].

## Honest scope (do not soften)
- Proves AUTHORITY + ATTESTATION half; NO FFI runtime — value↔C-ABI marshalling +
  a linked Rust cdylib are deferred. No new readiness lane.

## Gates
- Rust tests + ladder (workspace 0 failed). Ledger: `s61 → merged(5)` advanced;
  `s62` rides with S63.
