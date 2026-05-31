# S61 Plan — FFI authority model

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S61 (v0.8.1 band).
Map: reconciled plan §163-164 — native-interop boundary (FFI authority).
Branch: `codex/s61-ffi-authority`. Base: `origin/main` @ `6ef081e` (S60).

## Model
FFI is explicit, declared, diff-gated, sealed — not an implicit escape hatch. A
function wrapping a native call must declare `@caps(ffi)`, which flows through
surface (S35) → manifest (S36) → diff-caps GAINED ffi (S37) → seal (S38) →
sandbox escape-hatch warning (S46).

## Deliverables
- `examples/ffi/no_native.garnet` (baseline) + `native_boundary.garnet`
  (`@caps(ffi)`). Both check clean.
- `C_Language_Specification/GARNET_FFI_AUTHORITY.md` — the model + honest "no
  runtime" boundary.
- `garnet-cli/tests/ffi_authority.rs` — 3 cross-OS tests (check clean; sandbox
  flags ffi; diff-caps flags gaining ffi).

## Dogfood
- check both clean; `sandbox native_boundary` → ffi escape-hatch warning;
  `diff-caps no_native native_boundary` → GAINED ffi + AUTHORITY EXPANDED (exit 1).

## Honest scope (do not soften)
- NO FFI runtime — interpreter executes no extern "C"; this slice adds none. The
  authority MODEL is the deliverable; value = transparency + review, NOT
  containment (sandbox cannot constrain FFI). No new readiness lane.

## Also this branch
- Record S60 tag-deferral decision (Jon 2026-05-31) in GARNET_v0_8_0_RELEASE.md +
  contract S60 block; advance `s60 → merged(5)`.

## Gates
- Rust tests + ladder (workspace 0 failed). Ledger: `s60 → merged(5)` advanced;
  `s61` rides with S62.
