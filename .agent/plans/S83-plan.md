# S83 — post-tag release-truth reconciliation (closes WIN-S80-002)

## Goal
Reconcile the split truth the Windows audit flagged: `v0.8.0` is cut (Jon,
`cc165e8`, 2026-05-31), yet `GARNET_v0_8_0_CUT.md` read "READY TO CUT (pending
Jon)", the CHANGELOG header read "v0.8.0 in flight", and `.dogfood/goal.json` kept
`s80` pending. Record **both** truths in one place: the tag was cut by Jon **and**
the S80 PR produced cut-readiness evidence only.

## Context (why this is the right continuation)
A Stop-hook replayed the original S39→S80 `/goal` against a stale opening snapshot.
Current truth: S39 merged long ago (#256); the whole train is on `main`; `v0.8.0`
is tagged — the original stop condition (S80 readiness decision) was reached. The
*live* "slice train" is the v0.8.1 burn-down. S83 is its next Mac-doable, non-S91+
slice and it directly fixes the on-`main` inconsistency the hook surfaced (ledger
49/50, `s80` pending despite the cut). S91+ (the strategic arc) stays reserved for
Jon's go.

## What ships (pure docs/ledger + a gate)
- `GARNET_v0_8_0_CUT.md` — a "Post-cut release truth" note (tag cut by Jon AND the
  S80 PR produced readiness evidence only; both true).
- `CHANGELOG.md` — `[Unreleased]` header → "v0.8.1 runway" with the cut recorded
  (S31–S80 shipped under `v0.8.0`; P0/S81+ are the v0.8.1 runway). Full
  Keep-a-Changelog restructure stays deferred for Jon.
- `.dogfood/goal.json` — `s80 → merged(5)` + a `cut_record` (tag/commit/tagger).
  Goal now **50/50**.
- `scripts/garnet_release_truth_status.py` (+ `--gate`, 5 tests) — enforces the two
  truths coexist (doc + CHANGELOG + ledger cut_record).
- CI agent-contracts; the WIN-S80-002 burn-down row marked ✅ closed; this plan.

## Verification
- `python3 scripts/test_garnet_release_truth_status.py` → 5 OK; `--gate` rc 0.
- Ledger 50/50. fmt/diff clean; `cargo test --workspace` 0 failed (no Rust changed).

## Honest scope (do not soften)
Pure docs/ledger reconciliation — no code, **no new tag** (the tag already exists).
The full CHANGELOG restructure + any retroactive `v0.6.0`/`v0.7.0` tagging remain a
deferred release-truth decision for Jon. Not the reserved S91+ strategic arc.
