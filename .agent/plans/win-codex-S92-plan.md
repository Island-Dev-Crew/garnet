# S92 Spawn/FFI Authority Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:test-driven-development and superpowers:verification-before-completion. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the interpreter-visible subprocess authority laundering gap: executable `std::process::*` bridges must require the program entry point to declare `@caps(proc)`, not merely a downstream helper frame.

**Architecture:** Extend the existing S91 capability frame with a separate program-entry capability set. Keep direct embedded host/test calls allowed when no entry frame exists. Add a subprocess-specific guard for `std::process::{spawn,spawn_args,output}` and keep `wait`/`exit_code` tied to `proc` as before. Preserve calibrated honesty: FFI is declared, diffed, sandbox-flagged, and sealed today, but no executable FFI bridge exists for runtime enforcement in S92.

**Tech Stack:** Rust workspace (`garnet-interp-v0.3`, `garnet-cli` tests), Python status reporter, dogfood-readiness goal ledger.

---

## Files

- Modify: `garnet-interp-v0.3/src/eval.rs`
- Modify: `garnet-interp-v0.3/src/stdlib_bridge.rs`
- Modify: `garnet-cli/tests/caps_enforcement.rs`
- Add: `scripts/garnet_spawn_ffi_authority_status.py`
- Add: `scripts/test_garnet_spawn_ffi_authority_status.py`
- Modify: `.dogfood/goal.json`
- Modify: `CHANGELOG.md`
- Modify: `F_Project_Management/GARNET_v0_8_1_PLAN.md`

## Task 1: Advance Prior Slice Truth

- [x] Use dogfood-readiness goal mode to mark S91 merged at confidence 5 after PR #315.
- [x] Confirm `.dogfood/goal.json` reports slice 1 of 20 merged and next slice S92.

## Task 2: Write S92 RED Tests

- [x] Add a `proc_helper_laundering_traps_when_entry_lacks_proc` test where `main @caps()` calls a helper `@caps(proc)` that invokes `std::process::output`; expected failure is a `requires program entry @caps(proc)` trap.
- [x] Add a `proc_helper_runs_when_entry_declares_proc` control test where both entry and helper declare `proc`; expected success and exit-code `0`.
- [x] Add reporter tests requiring a program-entry subprocess guard and an explicit FFI-runtime-deferred honesty marker.
- [x] Run the focused tests and confirm the new laundering test/reporter fail for the expected reasons.

## Task 3: Implement Entry-Authority Guard

- [x] Extend `CapsContext` with an entry-frame count and entry capability multiset.
- [x] Add `CapsGuard::enter_entry` for program entry frames without changing managed function frame behavior.
- [x] Add `require_entry_capability(needed, fn_name)` that allows direct host/test calls when no entry frame exists but rejects subprocess bridges when the entry frame lacks `needed`.
- [x] Wire `call_value_with_entry_caps` through `CapsGuard::enter_entry`.
- [x] Apply `require_entry_capability("proc", ...)` to `std::process::spawn`, `spawn_args`, and `output`.

## Task 4: Update Truth Surfaces

- [x] Add `scripts/garnet_spawn_ffi_authority_status.py` with machine-checkable evidence for subprocess entry-guard wiring and FFI deferred scope.
- [x] Add status reporter tests.
- [x] Update changelog and v0.8.1 plan language without claiming Linux seccomp or executable FFI enforcement.

## Task 5: Verify and Prepare PR

- [x] Run focused Rust tests: `cargo test -p garnet-cli --test caps_enforcement -- --nocapture`.
- [x] Run focused reporter tests: `python scripts/test_garnet_spawn_ffi_authority_status.py`.
- [x] Run S91/S92 status gates.
- [x] Run `cargo fmt --check`.
- [x] Run `cargo test --workspace --no-fail-fast`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Build a dogfood-readiness bundle for S92.
- [ ] Open PR titled `S92: guard subprocess authority at program entry`.
