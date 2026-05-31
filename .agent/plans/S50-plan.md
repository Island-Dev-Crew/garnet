# S50 Plan — v0.8 beta gate (closes the S41–S50 hardening band)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S50.
Map: reconciled plan §152 + "v0.8 beta gate @ S50". The band milestone — a
checkpoint, NOT a release.
Branch: `codex/s50-beta-gate`. Base: `origin/main` @ `75237c3` (S49).

## CRITICAL honesty constraints (do not soften)
- S50 is a GATE/checkpoint, NOT a release. Do NOT cut v0.8.0-beta or any tag.
- Preserve verbatim anchors (research-grade prototype; Paper VI scorecard;
  production allocator path; human/aesthetic acceptance). Surface, don't change.

## Deliverables
- `scripts/garnet_v0_8_beta_gate.py`: verify S41–S49 merged at conf 5 in
  `.dogfood/goal.json`; re-run sub-gates (build-proof S47, proof-matrix S48);
  gate OPEN iff both. Inventory shipped + deferred; surface anchors + tag-note.
  `--format md|json`; `--gate` exits non-zero unless OPEN.
- `scripts/test_garnet_v0_8_beta_gate.py`: 8 unit tests.
- Wire test + `--gate` into ci.yml agent-contracts.
- `F_Project_Management/GARNET_v0_8_BETA_GATE.md`: criteria + in/deferred + anchors.

## Dogfood
- `garnet_v0_8_beta_gate.py --format md` → gate OPEN (band complete, sub-gates
  pass); `--gate` exits 0. Honesty anchors verbatim; tag-note present.

## Ledger timing
- `s49 → merged(5)` advanced in this branch (rode per S49 contract) → band
  s41-s49 = 9/9 merged. `s50` advance rides with S51.

## End-state / gates
- Full ladder green (zero Rust changed; workspace 0 failed). CHANGELOG + contract
  S50 block + beta-gate doc.

## Honest scope
- Beta gate = "the v0.8 hardening band is complete and its gates hold" — NOT
  production-readiness or a release. Tag deferred to Jon. No new readiness lane.
