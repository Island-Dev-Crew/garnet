# S87 Windows Hardening Plan

Slot: win-codex
Branch: codex/s87-windows-hardening
Slice: S87 Windows reporter hardening

## Goal

Close the Windows-only reporter hardening findings from the S1-S80 audit:
UTF-8-safe Markdown output under cp1252 consoles, graceful local-temp/probe
degradation for MIT readiness, and a committed-only readiness surface suitable
for byte-identical cross-machine comparison.

## Scope

- Add a shared reporter stdout helper and use it in the memory-eviction and MIT
  readiness reporters.
- Keep local-evidence failures quarantined from committed-truth MIT readiness.
- Add `--committed-only` to `scripts/garnet_mit_readiness_status.py`.
- Record the Windows proof in `F_Project_Management/WINDOWS_AUDIT_S1_S80.md`
  and `CHANGELOG.md`.

## Test-First Steps

1. Add a memory reporter subprocess test with `PYTHONIOENCODING=cp1252`; it must
   fail before the shared helper is wired.
2. Add MIT readiness tests for denied promo/temp probe fallback and
   `--committed-only` deterministic output.
3. Implement the smallest helper and reporter changes to pass those tests.
4. Run focused reporter tests, cp1252 CLI proof, full workspace test, clippy,
   and audit gate.

## Honest Scope

This slice hardens reporter behavior. It does not prove actual Mac/Windows byte
identity by itself; it exposes the committed-only surface and proves it is free
of local-evidence lanes and absolute source paths on this Windows box.
