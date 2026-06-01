# S89 — @max_depth runtime enforcement seed (first real enforcement)

## Goal
Make the trust kernel ENFORCE at runtime — one ceiling, honestly. A function
declaring `@max_depth(N)` traps deterministically when its recursion depth exceeds
N. Real enforcement (the interpreter refuses to recurse further), distinct from the
S85 host-stack raise.

## Design
- `@max_depth(N)` is the recursion-depth annotation (`Annotation::MaxDepth(i64)`;
  the checker constrains N ∈ [1,64]). `@bounded(N)` is *Wasmtime fuel* (S39/S88,
  deferred — wasmtime absent), so the enforceable "one ceiling" is `@max_depth`.
- `garnet-interp-v0.3/src/eval.rs` `call_fn`: a thread-local per-function recursion
  counter (`MAX_DEPTH_DEPTHS`, `const`-init), incremented via an RAII
  `MaxDepthGuard` that unwinds on every return/error path (including the trap). On
  entry to a `@max_depth(N)` function, if the new per-function depth > N → return
  `RuntimeError::msg("bounded: @max_depth(N) exceeded for ...")`.
- Per-FUNCTION (not global) depth → precise `@max_depth` semantics, so a function
  recursing within its ceiling (e.g. `agentic_log_analyzer.garnet`'s `@max_depth(8)`)
  runs unchanged and parity stays 33/33.

## What ships
- The interpreter enforcement (eval.rs).
- `garnet-cli/tests/bounded_enforcement.rs` — 4 cross-OS tests (over-ceiling traps;
  within runs; deterministic trap; unannotated not capped).
- `scripts/garnet_bounded_enforcement_status.py` (+ `--gate`, 5 tests).
- Spec `GARNET_BOUNDED_ENFORCEMENT.md` (the enforced-vs-declared boundary);
  CI agent-contracts; CHANGELOG; this plan; S89 Windows-proof row.

## Verification
- `cargo test -p garnet-cli --test bounded_enforcement` → 4 pass; `cargo test
  --workspace` 0 failed; parity 33/33; `agentic_log_analyzer` still runs; fmt/diff/
  clippy clean.
- `python3 scripts/test_garnet_bounded_enforcement_status.py` → 5 OK; `--gate` rc 0.

## Honest scope (do not soften)
ONE enforced ceiling (`@max_depth` recursion). `@bounded` (fuel), memory, time,
mailbox remain declared-not-enforced; unannotated functions are not capped (host
stack); the VM backend does not yet enforce @max_depth. Mac-authored + Mac-tested;
Windows trap re-proves via the cross-OS matrix (Windows-proof-pending).
