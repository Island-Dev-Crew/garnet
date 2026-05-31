# S63 Plan — C ABI proof (compound native authority)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S63 (v0.8.1 band).
Map: reconciled plan §163-164 — native-interop boundary (C ABI).
Branch: `codex/s63-c-abi`. Base: `origin/main` @ `f6bcf5a` (S62).

## Distinct from S62: compound authority
A C binding that does IO needs BOTH @caps(ffi) AND @caps(fs). Prove the model
surfaces/sandboxes/seals compound native authority (not just single ffi).

## Deliverables
- `examples/ffi/c_stat.garnet` — @caps(ffi, fs) C `stat`-like binding (stand-in),
  checks clean.
- `C_Language_Specification/GARNET_C_ABI.md` — C ABI as canonical FFI contract +
  the value↔C-type marshalling table + compound authority + honest no-runtime.
- `garnet-cli/tests/c_abi_proof.rs` — 3 cross-OS tests (check clean; sandbox
  surfaces ffi warning + fs preopens; seal attests ["ffi","fs"]).

## Dogfood
- check clean; sandbox caps ['ffi','fs'] + preopens true + ffi warning; seal
  aggregate ['ffi','fs'].

## Honest scope (do not soften)
- C ABI contract + compound-authority proof; NO FFI runtime — marshalling +
  linked .so/.dylib deferred. No new readiness lane.

## Gates
- Rust tests + ladder (workspace 0 failed). Ledger: `s62 → merged(5)` advanced;
  `s63` rides with S64.
