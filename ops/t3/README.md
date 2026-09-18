# T3 acceptance work and T-L shadow policy

This directory is the durable execution record for the user-approved September
2026 checked-acceptance train. It does not replace the archived Lane 0 mission,
change its denominators, or authorize a merge. `state.json` records current
work, evidence paths and remaining owner/reviewer actions; update it honestly.

The active scope is check → source-bound seal → fail-closed verify, followed by
the opt-in T-L shadow classifier. Playground Phase 1 builds on a separate branch
and receives its own review/proof round. The classifier never merges, labels a
remote PR, grants authority, or changes CI/settings. Missing coverage and unknown
policy are human-review outcomes, not permission.

Runtime step budgets, socket bridging/per-host policy and converter semantics
remain explicit T3 follow-on slices until their own acceptance contracts and
regressions land. Do not report the original broad T3 train complete when only
this acceptance slice is ready. T4 reconciliation remains with Jon/Claude.

Required checks for the shadow policy:
`python3 -I scripts/test_garnet_shadow_lanes.py`.
Exact Git input commits and executable digests are provenance, not signatures.
