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
- evidence-integrity bundles ok (31/31)
- workspace tests measured at reachable commit `c0190a4`
- MIT productization lane active-partial (93.1%)

### `native_linux` — Native Linux lane (CLI install + Studio + seccomp)

State: **pass**

Evidence:
- native Debian CLI clean-install proof verified
- native Linux Studio build+install+launch proof verified
- seccomp-apply proof deterministic and policy-driven

### `s114_acceptance` — S114 independent re-verification acceptance

State: **external-pending**

Evidence:
- F_Project_Management/GARNET_S114_INDEPENDENT_VERIFICATION_DOSSIER.html
- F_Project_Management/W_TRUST/S114_INDEPENDENT_REVERIFICATION_PACKAGE_2026-06-14.md

Blockers:
- Jon has not recorded an S114 acceptance decision; this reporter proves the static contract only and cannot grade acceptance

### `static_playground` — Static playground gallery (honest, recorded outputs)

State: **partial**

Evidence:
- 3 recorded examples, honesty markers present

Blockers:
- static gallery only; live execution is the W-PLAY workstream

### `live_wasm_playground` — Live WASM playground (W-PLAY, launch centerpiece)

State: **remaining**

Evidence:
- repo-owned WASM prerequisites ready

Blockers:
- live in-browser execution not built (W-PLAY workstream)
- garnet-interp pulls miette `fancy` (terminal/backtrace) — feature-gate it off for wasm

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
