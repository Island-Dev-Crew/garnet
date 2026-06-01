# S90 — @caps host-authority runtime enforcement seed

## Goal
Extend runtime enforcement (S89) from `@max_depth` to capabilities: the interpreter
traps when a managed function invokes a host-authority primitive whose required
capability no frame in the call chain declared. `garnet run` skips the static
checker, so this is the runtime backstop (verified: `@caps()` main calling
`std::env::get` previously ran and returned the real env value).

## Design
- `garnet-interp-v0.3/src/eval.rs`: a per-run thread-local `CapsContext`
  (`managed_frames` + a multiset `counts` of declared cap idents). `call_fn` pushes
  a `CapsGuard` for managed (`FnMode::Managed`) functions — recording the frame +
  its `@caps` idents, RAII-unwound on every return/error path. `pub(crate)
  require_capability(needed, fn_name)`: **allow** when `managed_frames == 0` (no
  program context — direct host/test calls); else permit iff the union of active
  caps contains `needed` or `*`; else trap
  `capability: `<fn>` requires @caps(<cap>), not declared in the calling chain`.
- `stdlib_bridge.rs`: `require_capability(...)` is the first statement of each
  host-authority bridge — env (`env`), process (`proc`), fs + `std::log::to_file`
  (`fs`).

## Why the union + no-frame-allow semantics
The static caps-graph propagates caps up every managed frame, so a *checked*
program carries the required cap on every frame → the union always includes it
(no false trap). Only under-declared programs (run via `garnet run`, which skips
the check) trap. The no-managed-frame allow keeps the Rust stdlib-bridge tests
(which call bridges directly) valid. Blast radius on `.garnet` programs is zero:
no example calls these natives except `novel_06`, which declares `@caps(fs)`.

## What ships
- The interpreter enforcement + bridge gating.
- `garnet-cli/tests/caps_enforcement.rs` — 5 cross-OS tests.
- `scripts/garnet_caps_enforcement_status.py` (+ `--gate`, 5 tests).
- A `@caps` section in `GARNET_BOUNDED_ENFORCEMENT.md`; CI agent-contracts;
  CHANGELOG; this plan; S90 Windows-proof row.

## Verification
- `cargo test -p garnet-cli --test caps_enforcement` → 5 pass; `cargo test
  --workspace` 0 failed (140 suites); fmt/diff/clippy clean.
- `python3 scripts/test_garnet_caps_enforcement_status.py` → 5 OK; `--gate` rc 0.

## Honest scope (do not soften)
Host-authority surfaces only (env/proc/fs/log-to-file); pure computation
unaffected; no-managed-frame calls allowed; the VM backend does not yet enforce
`@caps`. Mac-authored + Mac-tested; Windows trap re-proves via the cross-OS matrix
(Windows-proof-pending). **Last slice in the kernel-authoring lane — STOP and
report after this; do not start S91+ without Jon's go.**
