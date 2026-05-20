# v0.5 Release-Gate Truth Plan

Scope: align the v0.5 daily truth after the six v0.5.0-blocking slices have merged. This plan references `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` sections "Slice State Machine", "Slice Contracts" for S1/S2/S5/S8/S9/S10, and "v0.5.0 Release Gate".

## Intent

Record the current post-merge truth without tagging v0.5.0 and without widening claims beyond reproducible evidence. The six blocking slice PRs are merged; the remaining work is documentation/status alignment and clean-machine release-gate evidence.

## Edits

1. Update `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` so S8, S9, and S10 no longer read `not-started`; mark the merged blocking slices and release-gate checklist according to current evidence.
2. Draft `docs/blog/2026-Qx-garnet-v0-5.md` using the contract's substance-over-surface framing and preserving the honesty anchors verbatim.
3. Update `CURRENT_STATE.md` where the canonical example/run guidance is now stale because S8 intentionally adds an expected-failure mismatch fixture.
4. Update `docs/blog/index.html` only if needed to surface the draft without presenting it as a shipped v0.5.0 announcement.
5. Leave v0.5.0 untagged until the clean-machine reproduction gate is run and recorded.

## Verification

- `python3 scripts/garnet_mit_readiness_status.py --format json`
- `python3 scripts/garnet_mit_readiness_status.py --check-no-regression`
- `python3 scripts/garnet_proof_benchmark_status.py --format json`
- `python3 scripts/garnet_converter_status.py --format json`
- `python3 scripts/check_dogfood_pr_body.py` against the PR body before opening the PR
- Targeted release-gate reproduction commands where feasible; record blockers honestly for any step requiring published artifacts.

## Non-Goals

- No v0.5.0 tag.
- No Apple Developer ID or notarization claim.
- No Windows/Linux runtime proof beyond existing CI/runtime evidence.
- No new dependencies.
