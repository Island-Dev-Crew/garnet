# S8 CI Negative-Example Regression Plan

Date: 2026-05-20
Contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` § S8
PR: `S8: Signed hot-reload BLAKE3 demo` (#193)

## Current Truth

- S8 intentionally adds two runnable examples:
  - `examples/mvp_11_signed_hotreload.garnet` exits 0 and prints `reloaded successfully`.
  - `examples/mvp_11_signed_hotreload_mismatch.garnet` exits 1 and prints `BLAKE3 fingerprint mismatch`.
- The generic `canonical MVP examples` CI loop currently runs every `examples/mvp_*.garnet` as a success case.
- PR #193 therefore fails CI for the correct reason: the negative fixture is being treated as a positive fixture.

## Plan

1. Update the canonical MVP CI loop to parse/check every `examples/mvp_*.garnet`, but treat `*_mismatch.garnet` as an expected-failure run that must emit `BLAKE3 fingerprint mismatch`.
2. Keep the S8 dogfood contract unchanged; this is a CI interpretation fix, not a semantic change to the examples.
3. Update the PR body/evidence language from "expected PASS" to "negative fixture expected-failure handled".
4. Re-run the canonical examples block locally and the focused S8 dogfood commands before pushing.

## Honest Boundary

This does not expose managed-mode `actor.reload_signed` syntax. S8 remains a program-level BLAKE3 fingerprint demonstration with the Rust runtime API still separately tested in `garnet-actor-runtime/tests/reload.rs`.
