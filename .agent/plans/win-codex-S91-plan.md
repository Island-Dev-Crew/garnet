# S91 Caps Entry + Net Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:test-driven-development and superpowers:verification-before-completion. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove and close the S91 interpreter-scoped `@caps` runtime gaps: net authority is gated at the host bridge, and program entry installs a capability frame so safe/direct entry paths cannot bypass runtime caps through a zero-frame context.

**Architecture:** Keep S91 interpreter-scoped. Add a program-entry call path in `garnet-interp-v0.3` and use it from `garnet run --interp`; keep direct host/test calls outside a program frame allowed and honestly named. Gate `tcp_connect` with `@caps(net)`, update the S90/S91 status reporter to require net + program-entry evidence, and keep the VM gap explicit.

**Tech Stack:** Rust workspace (`garnet-interp-v0.3`, `garnet-cli`), Python status reporters, dogfood-readiness goal ledger.

---

## Current Evidence

- P0 ledger was reinitialized with `dogfood_readiness --goal-action init` as `v0_8_1` for S91-S110.
- The finished S31-S80 ledger was archived at `.dogfood/v0_8_goal.json`.
- Existing v0.8 release gates were patched to read the archived ledger and their focused tests pass.
- The active S91-S98 lane still stops after S98; S99-S110 are ledger-reserved only.

## Files

- Modify: `garnet-interp-v0.3/src/eval.rs`
- Modify: `garnet-interp-v0.3/src/lib.rs`
- Modify: `garnet-interp-v0.3/src/stdlib_bridge.rs`
- Modify: `garnet-cli/src/cmd/run.rs`
- Modify: `garnet-cli/tests/caps_enforcement.rs`
- Modify: `scripts/garnet_caps_enforcement_status.py`
- Modify: `scripts/test_garnet_caps_enforcement_status.py`
- Add: `F_Project_Management/GARNET_v0_8_1_PLAN.md`
- Modify: `.dogfood/goal.json`
- Add: `.dogfood/v0_8_goal.json`

### Task 1: Preserve the Ledger Transition

- [x] Archive the S31-S80 goal ledger before reinitializing the active ledger.
- [x] Reinitialize `.dogfood/goal.json` with S91-S110 via dogfood-readiness goal mode.
- [x] Verify old v0.8 gates fail when they read the new active ledger.
- [x] Patch v0.8 gates to read `.dogfood/v0_8_goal.json`.
- [x] Re-run v0.8 gate tests.

### Task 2: Write S91 RED Tests

- [x] Add `undeclared_net_traps_before_connect_policy` to `garnet-cli/tests/caps_enforcement.rs`.
- [x] Add `program_entry_frame_traps_safe_main_env_without_caps` to prove safe-mode entry no longer bypasses runtime caps.
- [x] Add `program_entry_frame_allows_safe_main_declared_env` to prove declared caps still run.
- [x] Update the caps status tests to require net bridge gating and program-entry evidence.
- [x] Run the focused test command and confirm the new tests fail for the expected reasons.

### Task 3: Implement S91 Runtime Changes

- [x] In `garnet-interp-v0.3/src/eval.rs`, rename the frame count semantics from managed-only to authority/program frames without changing direct host/test fallback.
- [x] Add an entry-call helper that installs a capability frame from the entry function annotations before dispatch.
- [x] In `garnet-interp-v0.3/src/lib.rs`, expose `Interpreter::call_entry`.
- [x] In `garnet-cli/src/cmd/run.rs`, call `interp.call_entry("main", vec![])` for `--interp`.
- [x] In `garnet-interp-v0.3/src/stdlib_bridge.rs`, add `require_capability("net", "net::tcp_connect")` before host/network policy resolution.

### Task 4: Update Truth Surfaces

- [x] Create `F_Project_Management/GARNET_v0_8_1_PLAN.md` from the current objective, with S91-S98 detailed and S99-S110 reserved.
- [x] Update `scripts/garnet_caps_enforcement_status.py` to report S91 as interpreter-scoped, net-gated, program-entry-framed, and VM-deferred.
- [x] Keep wording calibrated: S91 closes interpreter net/program-entry gaps only; VM `@caps` enforcement remains not implemented.

### Task 5: Verify and Prepare PR

- [x] Run focused Rust tests: `cargo test -p garnet-cli --test caps_enforcement -- --nocapture`.
- [x] Run focused reporter tests: `python scripts/test_garnet_caps_enforcement_status.py`.
- [x] Run S91 gate: `python scripts/garnet_caps_enforcement_status.py --gate --format json`.
- [x] Run `cargo test --workspace --no-fail-fast`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Build a dogfood-readiness bundle for S91 and keep deferred scope explicit.
- [ ] Commit, push branch, open PR titled `S91: gate net and add program-entry caps frame`.
