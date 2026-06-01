# S86 — binary-strict S80 cut-readiness proof

Contract: `F_Project_Management/WINDOWS_AUDIT_S1_S80.md` -> WIN-S80-001.

## Goal

Close the Windows audit gap where the S80 aggregate could report READY while
using `--no-run` for the S71/S72/S73 binary-backed lanes. The default S80 gate
must remain lenient for Python-only CI, but Windows audit mode must run the
direct binary/provider-free proofs and fail the gate if any direct proof fails.

## Scope

- Modify `scripts/garnet_v0_8_0_cut_readiness.py`.
- Modify `scripts/test_garnet_v0_8_0_cut_readiness.py`.
- Update `F_Project_Management/WINDOWS_AUDIT_S1_S80.md`.
- Update `CHANGELOG.md`.
- Do not cut or push any tag.
- Do not change S71/S72/S73 reporters unless the S86 aggregate cannot be made
  honest without a handoff. At current `origin/main`, S84 and S85 have landed,
  so direct Windows runs should pass.

## Implementation Plan

1. Add failing tests that assert the default runway spec keeps `--no-run` for
   S71/S72/S73, while binary-strict mode drops `--no-run` for exactly those
   binary-backed gates.
2. Add a failing test that `--windows-audit` is an alias for binary-strict mode.
3. Implement a small runway spec builder in `garnet_v0_8_0_cut_readiness.py`
   that returns lenient or binary-strict argv lists without mutating the global
   table.
4. Add `--binary-strict` and `--windows-audit` CLI flags. `--windows-audit` is
   an alias because this proof is Windows-runway motivated, but the behavior is
   portable: it just refuses to hide direct binary failures.
5. Add `mode` / `binary_strict` to the JSON/Markdown output so the evidence says
   whether the aggregate was lenient or strict.
6. Record S86 in the Windows audit doc as passed only after
   `python -B scripts\garnet_v0_8_0_cut_readiness.py --gate --binary-strict
   --format json` exits 0 on Windows.
7. Run focused tests, the strict gate, full workspace tests, and clippy before
   opening the PR.

## Dogfood Block

- `python -B scripts\test_garnet_v0_8_0_cut_readiness.py`
- `python -B scripts\garnet_v0_8_0_cut_readiness.py --gate --format json`
- `python -B scripts\garnet_v0_8_0_cut_readiness.py --gate --binary-strict --format json`
- `python -B scripts\garnet_v0_8_0_cut_readiness.py --gate --windows-audit --format json`
- `python -B scripts\test_garnet_windows_audit_status.py`
- `python -B scripts\garnet_windows_audit_status.py --gate --format json`
- `cargo test --workspace --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`

