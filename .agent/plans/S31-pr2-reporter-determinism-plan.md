# S31-PR2 Plan — Deterministic readiness reporter (committed-truth / local-evidence split)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S31 PR-2.
Branch: `codex/s31-pr2-reporter-determinism`.

## Bug (verified in code)
`scripts/garnet_mit_readiness_status.py` mixes machine-independent committed evidence
with machine-variable live probes:
- Headline % = `sum(_lane_score(lane.status))/len(lanes)` — machine-dependent because
  `windows_linux_domain_proof_matrix` and `promo_video` flip status on the presence of
  `~/Desktop` dogfood/render artifacts.
- `--check-no-regression` compares `completion_percent` — `windows_linux_distribution`
  is 60% on this Mac vs a committed baseline of 70% (captured on a Windows-capable
  machine) → false regression → `test_no_regression_gate_passes_source_only_floor` fails.

## Fix (Jon-approved: full committed-truth split)
1. Add `evidence_class: str = "committed"` to `ObjectiveLane`.
2. Tag the 3 machine-variable lanes `evidence_class="local"`:
   `windows_linux_distribution`, `windows_linux_domain_proof_matrix`, `promo_video`.
   (Their per-lane `completion_percent` computation is UNCHANGED — only their class.)
3. Headline % = `_lane_score` averaged over **committed** lanes only (machine-independent).
4. `check_no_regression` gates **committed** lanes only; local lanes are reported, never gated.
5. `render_markdown` emits two sections: "Committed truth (scored, gated)" and
   "Local evidence (machine-specific; not scored, not gated)".
6. Add a `reporter_determinism` committed lane (verified) documenting the split.
7. Regenerate `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json`.

## Guardrails / tests
- Per-lane assertions (promo=50, windows_linux=60, domain_matrix=100) stay green
  (computation unchanged).
- `test_no_regression_gate_passes_source_only_floor` → passes (gate skips local lanes).
- Overall stays `< 100` and `active-partial`.
- Full CI-gated python set stays green; `--check-no-regression` exits 0.
- Show Jon before/after headline % + new baseline for sign-off BEFORE opening the PR.

## Out of scope
- Updating the docs site % string (separate release-truth follow-up).
- Refactoring sub-module internal scoring (wls/promo) — only the aggregation layer changes.
