# S80 — v0.8.0 cut decision (the single tag for the whole run)

## Goal
Bring the entire S30–S80 completion run to its v0.8.0 cut decision (per
GARNET_v0_8_VERSION_MAP.md). Ship the cut-readiness gate; ESCALATE the tag to Jon.

## What ships
- `scripts/garnet_v0_8_0_cut_readiness.py` (+ `--gate`, 5 tests) — aggregates the
  whole run into one verdict: ledger S31–S79 all merged + the S60 release gate
  (S41–S59 + 11 sub-gates) + 11 runway gates (S69–S79). Current verdict: READY TO
  CUT (pending Jon).
- Doc `F_Project_Management/GARNET_v0_8_0_CUT.md`.
- CI agent-contracts; CHANGELOG; contract S80 block; this plan; ledger
  `s79 → merged`.

## CRITICAL honest scope (do not soften)
This slice does NOT cut, push, or authorize any tag. Cutting v0.8.0 is a
release-truth/strategy decision for Jon — the release of the whole completion run,
irreversible, reserved by the honesty anchors — ESCALATED, not autonomous. v0.8.0
is a research-grade-prototype milestone, not a production/1.0 claim; the
deferred-for-v0.8.0 list stands.

## After S80
- Escalate the v0.8.0 tag decision to Jon (AskUserQuestion) — like S60, but this
  is the REAL cut of the whole run.
- S81+ becomes the runway to v0.8.1 (the solutions-oriented real-world-proofs arc:
  agents running real builds/tests/solutions/simulations; the "ultrapunch"
  positioning). Offer to begin the S81+ replan.

## Verification
- `python3 scripts/test_garnet_v0_8_0_cut_readiness.py` → 5 OK; `--gate` rc 0.
- Ladder: fmt/diff clean; `cargo test --workspace` 0 failed (no Rust changed).
