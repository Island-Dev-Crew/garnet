# Garnet v0.8.1 Plan - Substrate Execution + Real-World Proof Runway

Status: active plan seed for S91-S110, created because no committed v0.8.1 PRD existed on `origin/main`.

This plan is intentionally narrow and calibrated. v0.8.1 is still a research-grade prototype lane. It strengthens the language substrate first, then leaves the real-world proof finale reserved until the substrate is merged. Do not claim production readiness, 1.0 readiness, or VM-backed capability enforcement from this lane.

## Active Goal Ledger

- Active ledger: `.dogfood/goal.json` (`v0_8_1`, S91-S110).
- Archived v0.8.0 ledger: `.dogfood/v0_8_goal.json` (S31-S80, including Jon's `v0.8.0` cut record).
- S91-S98 are the current execution lane.
- S99-S110 are reserved for the real-world-proofs finale and must not start until Jon explicitly opens that lane.

## S91-S98 Substrate Lane

| Slice | Title | Goal | Honest Scope |
|---|---|---|---|
| S91 | caps-entry-net | Gate `net` at the interpreter host bridge and install a program-entry caps frame so safe/direct entry paths cannot bypass runtime caps through a zero-frame context. | Interpreter-scoped proof only; direct host/test calls outside a program frame remain allowed; VM still enforces no `@caps`. |
| S92 | spawn-ffi-authority | Close authority laundering for spawn/FFI subprocess surfaces against declared `@caps` where the interpreter can prove it. | Linux seccomp/OS-policy apply is Linux-infra deferred on Windows. |
| S93 | bounded-loop-verifier | Add a static bounded-loop verifier for the safe subset; accept statically derivable bounds and reject uncheckable loops. | No Wasmtime claim. |
| S94 | paper-vi-exp1-llm-pass1 | Wire Paper VI Exp 1 LLM pass@1 harness behind a provider flag. | Account/credential gated; if credentials are absent, record honest-pending with harness wired. |
| S95 | paper-vi-exp3-5k-loc | Re-run Paper VI Exp 3 at a 5K LOC scale and resolve the h3a 6.5% to 10% question honestly. | Provider/runtime evidence only; no invented measurement. |
| S96 | linear-effect-safe-mode | Seed linear/effect typed safe-mode analysis toward provable `@caps` soundness. | First increment only; not whole-language verification. |
| S97 | provenance-seal-chain | Bind and verify the agent/model/prompt-to-artifact chain in seal. | Self-declared provenance unless independently verified. |
| S98 | cap-manifest-standard | Advance the capability-manifest schema and reference implementation seed. | Intent + ref implementation only; no standards body has adopted it. |

## Reserved S99-S110 Finale

These slices are represented in the active ledger for progress continuity only. Their detailed PRDs are reserved until S91-S98 merge and Jon explicitly opens the finale.

| Slice | Reserved Title | Status |
|---|---|---|
| S99 | real-world-proofs-lane-gate | Reserved; do not implement in S91-S98. |
| S100 | core-12-domain-selection | Reserved; do not implement in S91-S98. |
| S101 | windows-domain-execution | Reserved; do not implement in S91-S98. |
| S102 | linux-domain-execution | Reserved; do not implement in S91-S98. |
| S103 | mac-domain-execution | Reserved; do not implement in S91-S98. |
| S104 | cross-os-smoke-matrix | Reserved; do not implement in S91-S98. |
| S105 | studio-ui-control-proof | Reserved; do not implement in S91-S98. |
| S106 | package-pipeline-proof | Reserved; do not implement in S91-S98. |
| S107 | agent-workflow-proof | Reserved; do not implement in S91-S98. |
| S108 | failure-mode-red-team | Reserved; do not implement in S91-S98. |
| S109 | presentation-evidence-bundle | Reserved; do not implement in S91-S98. |
| S110 | v0.8.1-readiness-decision | Reserved; do not implement in S91-S98. |

## Stop Rule

After S98, report which substrate gaps are closed and proven, which remain named-deferred, and hold for explicit instruction before S99+.
