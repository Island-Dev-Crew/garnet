# Garnet v0.8.1 Plan - Substrate Execution + Real-World Proof Runway

Status: reconciled on 2026-06-01 after S97 merged in #321; S98 is active.

This file is the repo-visible v0.8.1 runway map. It preserves the Windows audit
burn-down context from S81-S90, reflects that S91-S97 are already merged, and keeps
the remaining work calibrated: v0.8.1 is a research-grade prototype lane, not a
production or 1.0 claim.

Do not claim VM-backed capability enforcement, OS sandbox enforcement, provider
measurements, signing/SBOM proof, or cross-OS domain proof unless the slice's own
dogfood block records that exact evidence.

## Active Goal Ledger

- Active ledger: `.dogfood/goal.json` (`v0_8_1`, S91-S110).
- Archived v0.8.0 ledger: `.dogfood/v0_8_goal.json` (S31-S80, including Jon's
  `v0.8.0` cut record).
- S91-S98 are the active substrate lane.
- S99-S110 are reserved for the real-world proof finale and must not start until
  Jon explicitly opens that lane.

## S81-S90 Windows Audit Burn-Down

S81-S90 are merged on `origin/main`. They closed the tracked Windows audit
findings by recording real Windows proof where available and honest
`pending-infra` status where tooling was absent.

| Slice | Implementation PR | Windows Proof PR | Outcome |
|---|---:|---:|---|
| P0 Windows audit import | #298 | - | Tracked the Windows audit as repo evidence. |
| S81 case-insensitive `.GARNET` discovery | #299 | #303 | Closed the uppercase extension trust gap. |
| S82 seal LF/CRLF determinism | #301 | #305 | Closed the Windows line-ending seal drift. |
| S83 post-tag release truth | #311 | - | Reconciled v0.8.0 cut truth. |
| S84 Paper VI Exp 3 WSL path proof | #300 | #300 | Closed the Windows/WSL script path failure. |
| S85 interpreter deep-recursion parity | #302 | #306 | Recorded Windows interp/VM parity proof. |
| S86 binary-strict cut readiness | #313 | - | Prevented `--no-run` from hiding runtime failures. |
| S87 Windows readiness reporter hardening | #309 | - | Hardened stdout/temp/committed-only reporting. |
| S88 Windows release tooling status | #312 | - | Recorded signing/SBOM/fuel gaps as honest partial. |
| S89 `@max_depth` enforcement seed | #304 | #308 | Added bounded recursion proof seed. |
| S90 `@caps` enforcement seed | #307 | #310 | Added interpreter-scope caps proof seed. |

Honest-partial carry-forward: S88 reports cosign, syft/CycloneDX, and Wasmtime
fuel tooling as absent rather than faking signed/SBOM/fuel proof. Those remain
named infra gates for later slices.

## S91 Merged

S91 merged in #315. It gates the interpreter `net` bridge and adds a
program-entry capability frame so `garnet run --interp` cannot bypass runtime
caps through a zero-frame entry context.

Honest scope: the proof is interpreter-scoped. Direct host/test calls outside a
program frame remain allowed for embedded testing, and the VM still does not
enforce user-function `@caps`.

## S92 Merged

S92 merged in #316. It closes the interpreter-visible subprocess authority
laundering gap for `std::process::{spawn, spawn_args, output}` by requiring
both the live call chain and the program entry point to declare `@caps(proc)`.

Honest scope: process launch bridges only. Executable FFI runtime enforcement is
still deferred because no executable FFI bridge exists yet; Linux seccomp /
OS-policy application remains infra-deferred on Windows; the VM still does not
enforce user-function `@caps`.

## S93 Merged

S93 merged in #317. It adds a static bounded-loop verifier for safe /
`@bounded` code and rejects loops the checker cannot prove bounded in that
subset.

Honest scope: static verifier only. No Wasmtime fuel, runtime loop metering, VM
loop enforcement, or OS sandbox enforcement is claimed.

## S94 Merged

S94 merged in #318. It adds the provider-gated Paper VI Exp 1 pass@1 harness,
including provider-free and fixture proof paths.

Honest scope: no provider-backed pass@1 measurement, full 500-task corpus,
hidden-test scorer, or statistical result is claimed.

## S95 Merged

S95 merged in #319. It adds the deterministic Paper VI Exp 3 5K-LOC rerun
harness, ten generated 5K-LOC snapshots, provider-free stateless/history-aware
rows, and cautious aggregate/analyze/status reporting.

Honest scope: no new provider-backed 5K h3a timing measurement is claimed. The
recorded v4.0 6.5% partial result stands until provider-backed 5K runtime rows
exist and are reviewed.

## S96 Merged

S96 merged in #320. It adds a narrow static linear/effect safe-mode seed that
rejects authority-bearing safe helper functions unless they expose an explicit
ownership-qualified parameter boundary.

Honest scope: first static increment only. It is not whole-language linear
typing, not VM/runtime capability enforcement, and not OS sandbox enforcement.

## S97 Active

S97 adds a provenance seal-chain block to `garnet seal` that validates
self-declared `agent`, `model`, and `prompt_sha256` attestation keys, then binds
them to the current seal's source and subject digests.

Honest scope: the binding is machine-checkable, but the origin remains
self-declared. S97 does not prove the named model executed the prompt, that the
named agent produced the artifact, or that the declared tool list is complete.

## S92-S98 Substrate Lane

| Slice | Title | Goal | Honest Scope |
|---|---|---|---|
| S92 | spawn-ffi-authority | Merged in #316: process-launch bridges require `@caps(proc)` on both the live call chain and the program entry point, while executable FFI enforcement stays named-deferred. | Process launch bridges only; direct host/test calls outside a program-entry frame remain allowed; Linux seccomp/OS-policy application remains infra-deferred on Windows; no VM caps claim. |
| S93 | bounded-loop-verifier | Merged in #317: static bounded-loop verifier for the safe subset accepts literal range/array bounds, literal counter `while` loops, and immediate-exit loop bodies while rejecting uncheckable loops in safe / `@bounded` scope. | Static verifier only; no Wasmtime fuel, runtime loop, VM, or OS sandbox enforcement claim. |
| S94 | paper-vi-exp1-llm-pass1 | Merged in #318: Paper VI Exp 1 LLM pass@1 harness behind a provider flag, with provider-free and fixture proof. | Harness wired only; provider-backed pass@1 measurement, full 500-task corpus, hidden-test scorer, and statistical run remain pending infrastructure. |
| S95 | paper-vi-exp3-5k-loc | Merged in #319: deterministic 5K-LOC Exp 3 rerun harness with provider-free stateless/history-aware rows and cautious aggregate/analyze output. | No new h3a timing measurement is claimed; recorded 6.5% partial stands until provider-backed 5K runtime rows exist and are reviewed. |
| S96 | linear-effect-safe-mode | Merged in #320: seed linear/effect typed safe-mode analysis toward provable `@caps` soundness by tying authority-bearing safe helper functions to explicit ownership-qualified parameter boundaries. | First static increment only; not whole-language verification, not runtime/VM enforcement, and not a proof that every capability path is sealed. |
| S97 | provenance-seal-chain | Merged in #321: binds and verifies self-declared agent/model/prompt metadata against the current seal source/artifact digests. | Binding verification only; no independent model-run, agent-origin, or complete tool-history proof. |
| S98 | cap-manifest-standard | Active: advances the capability-manifest schema and reference implementation seed through `garnet caps --standard-profile`, docs, vectors, and a status gate. | Intent plus reference implementation only; no standards body has adopted it; declared-surface manifests do not prove absence of undeclared authority. |

## Reserved S99-S110 Finale

These slices are represented in the active ledger for progress continuity only.
Their detailed PRDs are reserved until S91-S98 merge and Jon explicitly opens the
finale.

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

## Verification Expectations

Each slice needs focused tests proportional to the code change, the workspace
test/clippy gates, a dogfood evidence bundle, a green PR dogfood body check, and
CI green before merge.

When a slice is Windows-tagged, it is only Windows-complete when the named
Windows proof command passes on the Windows machine and the result is recorded in
repo evidence.

## Stop Rule

After S98, report which substrate gaps are closed and proven, which remain
named-deferred, and hold for explicit instruction before S99+.
