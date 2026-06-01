# win-codex S93 Plan - bounded-loop-verifier

Source of truth: `F_Project_Management/GARNET_v0_8_1_PLAN.md`, S93 row.

## Scope

S93 adds a static bounded-loop verifier for the safe subset:

- accept loops whose iteration bound is statically derivable,
- reject uncheckable loops,
- keep the proof static and checker/CLI-visible,
- do not claim Wasmtime fuel, runtime fuel metering, VM loop enforcement, or OS sandbox enforcement.

## Implementation Plan

- Extend `garnet-check-v0.3::bounds` beyond `@bounded(N)` extraction with a loop-bound verification report.
- Cover `for` loops over literal integer ranges/literal arrays, literal counter
  `while` loops, and loop bodies that exit before a second turn as statically
  bounded.
- Reject unbounded `loop { ... }` and `while` conditions that the verifier cannot prove finite.
- Surface S93 failures through `garnet check` as fatal diagnostics.
- Add a slice status reporter and tests so the S93 gate is machine-checkable.
- Update `.dogfood/goal.json` to mark S92 merged, leaving S93 pending until this PR merges.
- Update the v0.8.1 plan and changelog with calibrated scope.

## Verification Plan

- Focused red/green tests for `garnet-check` loop verification.
- CLI `garnet check` integration tests proving reject/pass behavior.
- Status reporter tests plus `--gate`.
- Full slice verification before PR:
  - `cargo fmt --all -- --check`
  - `git diff --check`
  - `cargo test -p garnet-check`
  - `cargo test -p garnet-cli --test bounds_report`
  - `python scripts/test_garnet_bounded_loop_verifier_status.py`
  - `python scripts/garnet_bounded_loop_verifier_status.py --gate --format json`
  - `python scripts/garnet_mit_readiness_status.py`
  - `cargo test --workspace --no-fail-fast`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## Honest Boundaries

- No Wasmtime fuel claim.
- No runtime fuel or VM loop enforcement claim.
- Static proof is conservative: rejected means unproven, not necessarily infinite.
- S94 provider/account-gated work remains outside this PR.
- S99+ remain reserved.
