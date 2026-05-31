# S77 — external package pilot

## Goal
Ecosystem maturation: pilot the external-package flow end-to-end against the
filesystem registry stub, with the slopsquatting guard in the loop.

## What ships
- **Rust (test only):** `garnet-registry-stub/tests/external_package_pilot.rs` —
  4 tests: external package resolves + BLAKE3 verifies; a tampered vendored file
  fails verification; a nonexistent dependency is refused (NotFound); the
  slopguard flags a hallucinated near-miss (separator-confusable + edit-distance).
  Runs in the `cargo test --workspace` matrix on every OS (real cross-OS proof).
- `scripts/garnet_external_package_pilot_status.py` (+ `--gate`, 5 tests) — static
  gate: pilot test present + registry infra (build_index/resolve/verify_package) +
  slopguard + doc present.
- Spec `C_Language_Specification/GARNET_EXTERNAL_PACKAGE_PILOT.md`; CI
  agent-contracts; CHANGELOG; contract S77 block; this plan; ledger `s76 → merged`.

## Why this scope
The package infra already exists (garnet-registry-stub build/verify + slopguard +
`garnet add`). S77 PILOTS the external-package scenario through it — the runnable
proof is a Rust integration test (in the matrix), and the agent-contracts gate is
static (no compiler there). No production code changed.

## Verification
- `cargo test -p garnet-registry-stub --test external_package_pilot` → 4 pass.
- `cargo test --workspace` 0 failed; fmt/diff/clippy clean.
- `python3 scripts/test_garnet_external_package_pilot_status.py` → 5 OK; gate rc 0.

## Honest scope (do not soften)
A LOCAL filesystem registry-stub pilot, NOT a live public ecosystem — no HTTP /
publish / auth / SemVer / signatures. The slopguard is a deterministic heuristic
("prompt to verify"), not a security guarantee. `garnet add` vendors local paths
only and does not yet load deps into `garnet run`.
