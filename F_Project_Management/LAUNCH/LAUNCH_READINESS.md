# Garnet Launch Readiness Ledger

Rendered state — regenerate with:
`python3 scripts/garnet_launch_readiness_status.py --format markdown`.
Machine authority applies to reporter-derived rows and the
evidence-base validator only; S114 acceptance, the minimum shelf,
and launch fire are external/manual gates.

- Schema: `garnet.launch_readiness/v1`
- Evidence-base commit: `c0190a4` (measured)
- Release grade: research-grade v0.x prototype; release_ready=True (binary-strict)
- Launch ready: **False**
- Recommendation: **HOLD**

## Gate Ledger

### `foundation_integrity` — Foundation integrity (release + red-team + evidence + measured base)

State: **pass**

Evidence:
- v0.8.1 release readiness reporter green (binary-strict)
- red-team static contract green (report, HIGH fix, regressions)
- evidence-integrity bundles ok (38/38)
- workspace tests measured at reachable commit `c0190a4`
- MIT productization lane active-partial (93.1%)

### `native_linux` — Native Linux lane (CLI install + Studio + seccomp)

State: **pass**

Evidence:
- native Debian CLI clean-install proof verified
- native Linux Studio build+install+launch proof verified
- seccomp-apply proof deterministic and policy-driven

### `s114_acceptance` — S114 independent re-verification acceptance

State: **accepted-scoped**

Evidence:
- S114 accepted (scoped) by Jon (repo owner) on 2026-07-12: first-party CLI/Wasm trust-kernel baseline
- recorded in F_Project_Management/LAUNCH/S114_ACCEPTANCE.json (read, not graded, by this reporter)
- not an independence relabel: S114 verdict language is unchanged (independently-re-verified-with-fixes)
- post-acceptance closure: condition-5-reopened-by-post-acceptance-delta-review
- current scope limit (tracked): Interpreter::new_permissive() deliberately permits unframed direct calls for trusted integrations; strict-by-default is not a claim that the opt-out is impossible.
- current scope limit (tracked): Direct use of the public low-level Env/Value/eval APIs is controlled by the Rust host and can execute outside an Interpreter method's per-instance strict scope; it is not covered as a sandbox boundary.
- current scope limit (tracked): Capability rows that are checker-only (time::*, uuid v4/v7) or declared-only (ffi, net_internal) are not uniformly runtime-gated.
- current scope limit (tracked): OS-level sandbox policy is generated with enforced:false and applied only on Linux via the external seccomp reference harness; macOS/Windows OS sandboxing remains named-deferred.
- current scope limit (tracked): memory::* natives remain bridge-only and caps-invisible.
- current scope limit (tracked): WV-5 proves the Wasm build and Node execution, not live browser-page execution.

### `static_playground` — Static playground gallery (honest, recorded outputs)

State: **partial**

Evidence:
- 3 recorded examples, honesty markers present

Blockers:
- static gallery only; live execution is the W-PLAY workstream

### `live_wasm_playground` — Live WASM playground (W-PLAY, launch centerpiece)

State: **remaining**

Evidence:
- WV-5 clean-Windows evidence: wasm32 + wasm-pack web/node builds passed
- WV-5 real Node execution passed, including fail-closed authority smoke

Blockers:
- docs/playground/live.js browser adapter is not implemented
- browser Wasm package is not present under docs/playground/pkg
- W-PLAY Playwright browser proof is not recorded

### `minimum_sealed_shelf` — Minimum sealed shelf (Core Ring Tier 1 + MCP library)

State: **manual-deferred**

Evidence:
- stdlib registry: 80 primitives, 100.0% explicit stability

Blockers:
- no reporter covers the shelf in Truth Lock; this is an explicit manual fence, never reporter-derived machine truth

### `promo_video` — Promo video (human-produced launch asset)

State: **pending-human**

Evidence:
- promo reporter status: public-site-embedded (95.0%)

Blockers:
- human render/QA decision outside this reporter

### `launch_fire` — Launch fire (marketing wave, tag, release)

State: **jon-only**

Blockers:
- FIRE/HOLD is Jon's alone; never autonomous

## Deferred (named, not enforced)

- @bounded (Wasmtime fuel) — declared, not enforced
- memory limits — declared, not enforced
- time limits — declared, not enforced
- @mailbox — declared, not enforced
- macOS/Windows OS-sandbox application — seccomp applies on Linux only
- Core Ring Tier 1 shelf — post-Truth-Lock workstream (Minimum Shelf)
- MCP tool-server library — post-Truth-Lock workstream (Minimum Shelf)

## Jon-only actions

- push a git tag
- cut or release a version
- record the S114 acceptance decision
- change CI/release policy or any gate a PR merges under
- RB-6 backend decision
- RB-8 root-reorg cut
- fire the launch/marketing wave
