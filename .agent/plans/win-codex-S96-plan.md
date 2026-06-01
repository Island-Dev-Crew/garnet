# S96 Linear/Effect Safe-Mode Plan

> Required workflow: test-first implementation. S96 is a narrow static-analysis seed, not a whole-language proof.

**Goal:** Add a first linear/effect safe-mode checker that ties safe authority-bearing functions to an explicit ownership-qualified parameter boundary.

**Architecture:** Reuse the existing `garnet-check-v0.3` safe-mode and cap-graph architecture. Add a focused `effects` pass that consumes the cap graph's transitive requirements, reports effect summaries, and emits a fatal checker error only for non-`main` safe functions that perform authority effects without any `own`/`borrow`/`ref`/`mut` parameter.

**Files:**
- Create: `garnet-check-v0.3/src/effects.rs`
- Modify: `garnet-check-v0.3/src/lib.rs`
- Create: `garnet-check-v0.3/tests/linear_effects.rs`
- Create: `scripts/garnet_linear_effect_status.py`
- Create: `scripts/test_garnet_linear_effect_status.py`
- Modify: `scripts/garnet_mit_readiness_status.py`
- Modify: `CHANGELOG.md`
- Modify: `F_Project_Management/GARNET_v0_8_1_PLAN.md`
- Modify: `.dogfood/goal.json`
- Append: `F_Project_Management/AGENT_COORDINATION_LEDGER.md`

## Task 1 - Red Tests

- [x] Add `garnet-check-v0.3/tests/linear_effects.rs` with four cases:
  - effectful safe helper with no ownership-qualified parameter is fatal
  - effectful safe helper with `borrow path: String` passes the S96 check
  - pure safe helper without linear parameters remains allowed
  - `main` remains the program boundary and is not rejected by the S96 rule
- [x] Add `scripts/test_garnet_linear_effect_status.py` expecting an S96 status script and source inventory.
- [x] Run focused tests and capture the expected missing-module/missing-script failures.

## Task 2 - Minimal Checker

- [x] Add `effects.rs` with `LinearEffectReport`, `FnEffectSummary`, `LinearEffectViolation`, and `linear_effect_report(module, caps_report)`.
- [x] Add `CheckError::LinearEffect`, canonical code `check.linear_effect`, fatal severity, and `CheckReport::ok()` handling.
- [x] Wire the pass after cap-graph analysis in `check_module`.
- [x] Keep the rule deliberately narrow: non-`main`, safe/effective-safe functions with non-empty transitive caps and zero ownership-qualified parameters are rejected.

## Task 3 - Status + Readiness

- [x] Add `scripts/garnet_linear_effect_status.py` with quick inventory by default and `--gate` focused cargo proof.
- [x] Wire a readiness lane named `linear_effect_safe_mode_seed`.
- [x] Add focused Python tests for the status script and readiness no-regression.

## Task 4 - Documentation + Evidence

- [x] Mark S95 merged in `.dogfood/goal.json` and the v0.8.1 plan.
- [x] Append the S96 `CHANGELOG.md` entry with calibrated scope.
- [x] Build a desktop evidence bundle with status JSON/MD, focused tests, workspace tests, clippy, readiness, diff-check, and manifest.
- [ ] Open PR `S96: add linear/effect safe-mode seed`, run the PR body dogfood check, then merge only after CI is green.
