# S60 Plan — v0.8.0 tag (release-readiness gate + ESCALATION)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S60.
Map: reconciled plan §158 — "v0.8.0 tag".
Branch: `codex/s60-release-readiness`. Base: `origin/main` @ `26838a4` (S59).

## CRITICAL — the tag is escalated, NOT cut
Cutting `v0.8.0` is a release-truth/strategy decision for Jon (only v0.4.2/v0.5.0
are tagged; honesty anchors reserve it). S60 ships the release-readiness GATE
(the honest, dogfoodable part) and ESCALATES the tag cut to Jon. No tag is
created or pushed by this slice.

## Deliverables
- `scripts/garnet_v0_8_0_release_readiness.py`: aggregate both bands (hardening
  S41–S50, adoption S51–S59) + 11 anti-rot sub-gates → READY/NOT-READY verdict +
  in/deferred inventory + honesty anchors + the "does NOT cut a tag" note.
  `--gate` fails unless bands merged + sub-gates pass. `--format md|json`.
- `scripts/test_garnet_v0_8_0_release_readiness.py`: 5 unit tests.
- Wire test + `--gate` into ci.yml agent-contracts.
- `F_Project_Management/GARNET_v0_8_0_RELEASE.md`: readiness summary + escalation.

## Dogfood
- `--format md` → READY TO TAG (pending Jon); 10/10 + 9/9 + 11/11; `--gate` 0.

## End-state / gates
- Ship the gate PR; merge when green; advance `s59 → merged(5)` (rode this
  branch). Then ESCALATE the tag decision to Jon. `s60` stays pending until the
  tag decision (the slice's headline deliverable, the tag, is Jon's).

## Honest scope (do not soften)
- Does NOT cut/push a tag. READY TO TAG = evidence-backed advice. Research-grade
  prototype; deferrals enumerated; anchors verbatim. No new readiness lane.
