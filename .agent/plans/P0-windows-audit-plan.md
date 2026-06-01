# P0 — commit the Windows audit as tracked truth (v0.8.1 runway)

## Goal
The Codex Windows audit of S1–S80 (HEAD cc165e8) found 14 open `WIN-*` findings but
lives only in a handoff tarball. Commit it as the tracked source of truth so the
S81–S88 burn-down has a committed ledger. Pure docs/data import — no code, no
Windows proof.

## What ships
- `F_Project_Management/WINDOWS_AUDIT_S1_S80.md` — the summary, the 14-finding →
  owning-slice table, the resolved `WIN-S70-001`, the burn-down rules, and a
  Windows-proof-status table the Windows lane updates.
- `.dogfood/windows-core-audit.json` (S1–S30) + `.dogfood/windows-audit-goal.json`
  (S31–S80) committed verbatim from the handoff.
- `scripts/garnet_windows_audit_status.py` (+ `--gate`) — asserts every open
  finding has an owning slice and the ledgers pin HEAD cc165e8.
- `scripts/test_garnet_windows_audit_status.py` (5 tests); CI agent-contracts;
  CHANGELOG; this plan.

## Out of scope
- Fixing any finding (those are S81–S88).
- The post-tag release-truth reconciliation / `[Unreleased]` header / `s80→merged`
  ledger advance — that is **S83**, a different lane.
- Re-initing the dogfood goal ledger for v0.8.1 (deferred; burn-down is tracked in
  WINDOWS_AUDIT_S1_S80.md + the task list for now).

## Verification
- `python3 scripts/test_garnet_windows_audit_status.py` → 5 OK; `--gate` rc 0.
- Ladder: fmt/diff clean; `cargo test --workspace` 0 failed (no Rust changed).

## Honest scope (do not soften)
A tracking gate over imported audit evidence — it does not re-run the audit or
claim any finding fixed. Windows proofs are recorded by the Windows lane. v0.8.1 is
a research-grade-prototype runway; v0.8.0 stays the cut milestone.
