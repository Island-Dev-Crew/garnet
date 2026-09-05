# U-116 — trust-surface widening: what v2 adds, what it costs, how sealed history is preserved

Measured at base `452a0e211f3dcaf2cff557e108e79f8f9e67b288` (main after #551) with the v1 and v2 predicates of `scripts/garnet_trust_kernel_review_status.py` at this PR's head. Reproduce: `git log --first-parent -60 <base>`; for each commit, `git diff --name-only <commit>^ <commit>`; classify each path with `trust_surface_predicate("v1")` and `trust_surface_predicate("v2")`; a PR *flips* when it touches v2 but not v1.

## What v2 adds over v1

Prefixes added: `F_Project_Management/W_TRUST/eras/`, `garnet-cli/src/`, `scripts/smoke_garnet_`

Files added: `scripts/check-agent-contracts.py`, `scripts/check_determinism_no_llm.py`, `scripts/check_dogfood_pr_body.py`, `scripts/package_garnet_studio_macos.sh`, `scripts/package_garnet_vscode_extension.sh`, `scripts/preflight_garnet_studio_notarization.sh`, `scripts/run_agentic_dogfood_matrix.py`, `scripts/smoke_garnet_studio_dmg.sh`, `scripts/smoke_garnet_web_pwa.sh`, `scripts/test_check_agent_contracts.py`, `scripts/test_check_determinism_no_llm.py`, `scripts/test_check_dogfood_pr_body.py`, `scripts/test_github_actions_node24_readiness.py`, `scripts/test_smoke_garnet_pages_pwa.py`

Files no longer listed individually (subsumed by an added prefix): `garnet-cli/src/bin/garnet.rs`, `garnet-cli/src/bound_source.rs`, `garnet-cli/src/cmd/add.rs`, `garnet-cli/src/cmd/doctest.rs`, `garnet-cli/src/cmd/eval.rs`, `garnet-cli/src/cmd/mod.rs`, `garnet-cli/src/cmd/run.rs`, `garnet-cli/src/cmd/test.rs`, `garnet-cli/src/lib.rs`

## Cost over the last 60 first-parent commits: 4 flip(s), 1 touching `garnet-cli/src/`

| PR | commit | newly covered paths |
|---|---|---|
| #551 | `452a0e21` | `scripts/check_dogfood_pr_body.py`, `scripts/test_check_dogfood_pr_body.py` |
| #554 | `b20869d0` | `garnet-cli/src/cmd/verify.rs`, `garnet-cli/src/manifest.rs` |
| #536 | `a8a66fcb` | `scripts/check_dogfood_pr_body.py`, `scripts/test_check_dogfood_pr_body.py` |
| #523 | `63a0d702` | `scripts/smoke_garnet_minimum_shelf.py` |

A flip means: had this surface been live, that PR would have needed a structured review record. It is the price of the coverage, stated per PR rather than argued.

## Sealed history under the era ledger

The two markers sealed before the surface carried a version landed before the `v2` era stone this change lays, so the ledger places them in v1 — no pin, no declaration, no copy of the gate read:

- `68317ae2` → `v1` (no findings)
- `41d6ced8` → `v1` (no findings)

The end-to-end regression `TrustSurfaceWideningTests` lands a v2 marker in a fixture repository, lays a v3 stone with the live surface widened, verifies the registry green with no marker bytes changed, shows a v3-era landing that omits a v3-only path reported, and shows a live surface with no stone unable to run green.
