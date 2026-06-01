# S85 — interpreter deep-recursion robustness — closes WIN-S73-001

## Goal
`garnet run --interp` stack-overflowed on Windows (~1 MiB default thread stack) for
`mvp_function_call_demo.garnet` while `--vm` succeeded, so the VM/interpreter parity
campaign diverged on Windows (32/33). Fix it as cross-platform robustness, not a
Windows patch.

## Fix
- `garnet-cli/src/cmd/run.rs`: split `run_interpreter` into a thin wrapper that
  spawns `std::thread::Builder::new().name("garnet-interp").stack_size(256 MiB)`
  and joins it, and `run_interpreter_inner` (the existing body, returning a `u8`
  exit code). Inputs are owned-cloned across the boundary; the `Interpreter` is
  created inside the thread, so nothing non-`Send` crosses. A thread panic maps to
  exit 1.

## Finding (honest, surfaced during authoring)
The tree-walking interpreter spends ~tens of KiB of host stack per Garnet frame:
`countdown(5000)` runs on 256 MiB but `countdown(10000)` overflows. So the fix
raises the ceiling by hundreds× (the audit fixture recurses ~235 deep — far within
budget) but is **not** unbounded. Truly deep/unbounded recursion is the `@bounded`
enforcement story (S89), not a stack-size question. Stated in the test + CHANGELOG.

## What ships
- The large-stack interpreter wrapper.
- `garnet-cli/tests/interp_deep_recursion.rs` — 2 cross-OS integration tests (the
  audit fixture → `=> 7105`; a 5000-deep recursion that needs the large stack).
- `scripts/garnet_interp_stack_status.py` (+ `--gate`, 5 tests) — anti-regression:
  the large-stack thread + `run_interpreter_inner` routing + the tests stay.
- CI agent-contracts; CHANGELOG; this plan; the S85 Windows-proof row updated.

## Verification
- `cargo test -p garnet-cli --test interp_deep_recursion` → 2 pass; `cargo test
  --workspace` 0 failed; the parity campaign stays 33/33 on Mac; fmt/diff/clippy clean.
- `python3 scripts/test_garnet_interp_stack_status.py` → 5 OK; `--gate` rc 0.

## Honest scope (do not soften)
Robustness fix raising the recursion ceiling; NOT an unbounded guarantee. Must
precede S89 (which builds the @bounded *enforcement* trap on this same path).
Mac-authored + Mac-tested; the original Windows fixture re-proves via the cross-OS
matrix (`garnet.exe run --interp mvp_function_call_demo.garnet` → exit 0 `=> 7105`;
parity 33/33), recorded Windows-proof-pending.
