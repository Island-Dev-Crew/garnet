# S81 — case-insensitive `.GARNET` discovery (closes WIN-S33/36/37/46)

## Goal
One fix clears four high-severity Windows trust findings. The shared target
collector compared the file extension case-sensitively, so on Windows'
case-insensitive filesystem an uppercase `.GARNET` file was silently skipped by
`garnet verify`, capability manifests, `diff-caps`, and sandbox-policy walks (a
planted `BAD.GARNET` passed a 5/5 gate).

## Root + fix
- Root: `garnet-cli/src/cmd/verify_gate.rs:218` — `path.extension().is_some_and(|e| e == "garnet")`.
- Fix: `e.eq_ignore_ascii_case("garnet")` in the shared `walk()`. Because
  `garnet-cli/src/cap_manifest.rs:91` (`surface_for_path`) reuses the same
  `collect_targets`/`walk`, the single fix covers verify / caps / diff-caps /
  sandbox-policy (closes WIN-S33-001, WIN-S36-001, WIN-S37-001, WIN-S46-001).

## What ships
- The one-line collector fix (case-insensitive) + 2 Rust unit tests in a new
  `verify_gate::tests` module (a `BAD.GARNET` directory fixture is discovered; an
  uppercase file target resolves).
- `scripts/garnet_garnet_ext_discovery_status.py` (+ `--gate`, 5 tests) —
  anti-regression: collector stays case-insensitive + caps surfaces route through it.
- CI agent-contracts (test + gate); CHANGELOG; this plan; the S81 Windows-proof row
  in `WINDOWS_AUDIT_S1_S80.md` updated to "Mac fix landed; Windows-proof-pending".

## Verification
- `cargo test -p garnet-cli verify_gate::tests` → 8 OK; `cargo test --workspace` 0 failed.
- `python3 scripts/test_garnet_garnet_ext_discovery_status.py` → 5 OK; `--gate` rc 0.
- fmt/diff/clippy clean.

## Honest scope (do not soften)
Mac-authored + Mac-unit-tested (macOS preserves filename case, so the skip
reproduces on Mac). **Not marked Windows-complete** — the end-to-end Windows proof
(`garnet verify <dir with BAD.GARNET>` → exit 1 on a real Windows FS) is handed off
to the Windows lane and recorded as Windows-proof-pending.
