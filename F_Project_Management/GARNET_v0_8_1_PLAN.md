# Garnet v0.8.1 Plan - Substrate Execution + Real-World Proof Runway

Status: reconciled on 2026-06-02 — Jon opened lane A (the S99-S120 critical chain);
the S81-S90 burn-down and the S91-S98 substrate are merged on `origin/main`.

This file is the repo-visible v0.8.1 runway map and the spec the surge/repro lanes
read. It preserves the Windows audit burn-down context from S81-S90, records that
S91-S98 are merged, and lays out the S99-S120 five-stage proof runway. It keeps the
remaining work calibrated: v0.8.1 is a research-grade prototype lane, not a
production or 1.0 claim.

Do not claim VM-backed capability enforcement, OS sandbox enforcement, provider
measurements, signing/SBOM proof, or cross-OS domain proof unless the slice's own
dogfood block records that exact evidence.

## Active Goal Ledger

- Active ledger: `.dogfood/goal.json` (`v0_8_1`, S91-S120).
- Archived v0.8.0 ledger: `.dogfood/v0_8_goal.json` (S31-S80, including Jon's
  `v0.8.0` cut record).
- S91-S98 are the completed substrate lane (merged).
- S99-S120 is the real-world proof runway, opened by Jon as lane A (2026-06-02),
  across five stages: V (S99-S101) closes the VM-enforcement seam, U (S102-S104)
  runs the ultrapunch for real, S105 selects domains, X (S106-S109) + R (S110-S113)
  prove + reproduce cross-OS on the surge/repro lanes, and P (S114-S120) positions
  and escalates the cut. Mac #1 authors V / U / S105 / P; the surge + repro lanes
  own X / R.

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
| S98 | cap-manifest-standard | Merged in #322: advances the capability-manifest schema and reference implementation seed through `garnet caps --standard-profile`, docs, vectors, and a status gate. | Intent plus reference implementation only; no standards body has adopted it; declared-surface manifests do not prove absence of undeclared authority. |

## S99-S120 Real-World Proof Runway (opened 2026-06-02, lane A)

Jon opened the finale. It is renumbered/extended from the old S99-S110 reserve to
**S99-S120 across five stages**. **Mac #1 (build lead)** authors the
correctness-critical spine (Stage V, Stage U, S105, Stage P); the cross-OS **surge
lanes** and **repro lanes** PROVE and reproduce (Stage X, Stage R) and MUST NOT
author on the build-lead branches. Calibrated honesty governs every slice: no
"enforced" without a deterministic trap proven by test; no cross-OS-complete claim
from one machine; no tag pushed; v0.8.1 stays a research-grade prototype.

Order: **P0 → V (S99-S101) → U (S102-S104) → S105 → [hold while Stage X + R run on
other machines] → P (S114-S120)**. Stop + report after V, after U, and before S120.

### Stage V — close the VM-enforcement seam (the gate) · build lead · all 3 OSes prove
The substrate correctly NAMED-DEFERRED VM enforcement (S90/S91/S92 are
interpreter-scoped — "the VM enforces nothing"). Stage V closes that seam; the
ultrapunch cannot be honest until the VM traps too.

| Slice | Title | Goal | Honest scope / gate |
|---|---|---|---|
| S99 | vm-max-depth-trap-parity | The VM traps on the same `@max_depth` recursion ceiling the interpreter does — extend the S73/S85 result-parity campaign to **TRAP-parity**. | Deterministic trap proven by test on both backends; not a Wasmtime-fuel claim. |
| S100 | vm-caps-trap-parity | The VM traps on undeclared `@caps` at the same env/proc/fs boundary the interpreter gates (S90/S91/S92 → VM). | Host-authority bridges only; OS-sandbox/seccomp application stays infra-deferred. |
| S101 | vm-interp-enforcement-parity-gate | A reporter + `--gate` proving every enforcement trap fires identically on both backends; the "VM enforces nothing" gap is CLOSED. | Parity of the traps that exist; surfaces remaining named-deferred gaps explicitly. |

**STOP + report after S101:** which traps reach VM/interp parity, which stay named-deferred.

> **Stage V CLOSED (2026-06-02).** S99 (#325, `5d36219`) — VM `@max_depth`
> trap-parity (ABI `GARNVM03`, `VmDepthGuard`). S100 (#326, `e630337`) — VM `@caps`
> trap-parity; closed a real authority-laundering hole (the S92 program-entry gate
> was bypassed under `--vm`, so undeclared subprocess authority laundered through
> an `@caps(proc)` helper ran). S101 (this slice) — the consolidated
> enforcement-parity reporter + `--gate`. **Reached VM/interp TRAP-parity:**
> `@max_depth` + `@caps` (env/proc/fs/net + the S92 entry gate), proven by
> both-backends tests on the cross-OS matrix. **Still named-deferred on BOTH
> backends (never faked):** `@bounded` (Wasmtime fuel — wasmtime absent), memory,
> time, `@mailbox`, and OS-level sandbox application. This is trap-parity for the
> enforced ceilings, not total backend equivalence.

### Stage U — the ultrapunch, run for REAL once on the enforced kernel · canonical run = Mac #1
| Slice | Title | Goal | Honest scope |
|---|---|---|---|
| S102 | agent-build-test-loop-real | A REAL (not mock) agent-driven loop: an agent proposes real Garnet code → `diff-caps` gates the capability delta → the ENFORCED kernel runs it in bounds → `seal` attests it. | The proposer is a real agent on the canonical run; the loop is the gated primitive. |
| S103 | ultrapunch-accept-reject-demo | Capability-bounded ACCEPTANCE of agent-authored code end-to-end, for real — INCLUDING a rejection case (an agent widening the capability surface is caught + refused). The negative proof is the punch. Capture the 4 trust artifacts + an honest accept/reject decision on capability evidence. | Evidence, not assertion; the reject case must actually be refused by the gate, proven by test. |
| S104 | ultrapunch-evidence-bundle | Full reproducible record: commands, outputs, accept AND reject, the 4 trust artifacts. | A reproducible bundle; reproduction is Stage R's job, not claimed here. |

**STOP + report after S104:** ultrapunch proven (accept + reject), evidence bundle sealed.

### S105 — core-domain-selection · build lead (unblocks the parallel Stage X)
Select the 5-10 real demonstrator domains and, for EACH, the specific trust-artifact
delta a non-Garnet build cannot produce. Authoring this is the handoff: it gives the
surge lanes their specs.

**HANDOFF after S105:** the cross-OS lanes (Windows-Codex, Mac-Codex, Linux) run
Stage X in parallel; the repro lanes run Stage R. Mac #1 HOLDS the spine and does
NOT author S106-S113 — it provides their specs only.

> **Hold-window build-lead slice — `s105b` os-sandbox-apply (UTM, 2026-06-03).**
> Closes S46's one genuinely-deferred enforcement piece: the *generated* seccomp
> policy is now **applied + deterministically trapped on a real Linux kernel** (the
> Mac's UTM Debian-12 ARM64 guest, Linux 6.1). `@caps(fs)` denies `socket` (EPERM,
> 3/3 runs); `@caps(fs, net)` allows it (policy-driven). Reference harness
> `tools/seccomp-apply/`; record `C_Language_Specification/GARNET_SECCOMP_APPLY.md`.
> Honest scope: Linux seccomp only (macOS/Windows named-deferred); proves the
> generated policy is enforceable, not program safety; a `garnet`-native apply path
> + applying to a spawned subprocess (S92 `[LINUX-INFRA]`) are follow-ups.

### Stage X — cross-OS enforcement + ultrapunch proof · surge lanes (NOT build-lead branches)
| Slice | Title | Goal | Proof discipline |
|---|---|---|---|
| S106 | windows-cross-os-enforcement-proof | Windows-Codex: run the S101 enforcement-parity gate on Windows and reproduce the S103 accept/reject; record every trap fires identically. | Recorded in repo evidence; Windows-only proof command named. |
| S107 | mac-codex-cross-os-enforcement-proof | Mac-Codex (independent of Mac #1): same proof on a second macOS environment. | Independent machine; not the canonical run. |
| S108 | linux-cross-os-enforcement-proof | Linux lane: same proof on Linux; additionally, where seccomp is present, attempt the S92 named-deferred OS-policy application as a real Linux-only datapoint (never faked). | seccomp result is honest-partial if tooling absent. |
| S109 | cross-os-trap-parity-matrix | Consolidate S106-S108 into a Win×Mac×Linux × {max_depth, caps, diff-caps-reject} trap-parity matrix + gate. | A trap is cross-OS-complete only when all three machines recorded it. |

**S106 Windows-lane split (recorded 2026-06-03):** the first Windows PR records
the Stage V trap re-proof only: S101 `@max_depth` parity, `@caps(env/proc/fs/net)`
host-authority traps, and the S92 program-entry `@caps(proc)` trap on Windows,
plus a WSL execution/portability rerun. WSL is not Linux seccomp, Wasmtime, or
OS-sandbox enforcement proof. The S103 ultrapunch accept/reject reproduction and
S105 domain execution stay in the later Phase 2 Windows lane after the hold gate;
they are not claimed by the Phase 1 evidence bundle.

### Stage R — independent reproduction · repro lanes
| Slice | Title | Goal |
|---|---|---|
| S110 | ultrapunch-evidence-repro | A fresh-checkout lane replays S104's commands and byte-compares the 4 artifacts + the accept/reject decision; reproducibility verdict. |
| S111 | domain-proof-repro | Independently reproduce the S105-selected domain trust-artifact deltas. |
| S112 | cross-os-repro-consolidation | Aggregate Stage X + Stage R evidence into one reproducibility ledger feeding Stage P. |
| S113 | evidence-integrity-gate | A final integrity gate (seal-verify + manifest hash-chain over the whole evidence corpus) before positioning. |

**S110 Windows/WSL reproduction split (recorded 2026-06-03):** the Windows repro
lane records the S104 ultrapunch accept/reject replay as committed evidence under
`proofs/windows/ultrapunch/` and `proofs/linux/repro/`. Both rows retain the
ACCEPT trust artifacts (`capability_manifest.json`, `diff_caps.txt`, `seal.json`,
`transparency_log.jsonl`, `decision.md`), verify the transparency-log chain, and
prove both refusal modes: capability widening is refused by diff-caps and an
over-depth proposal is refused by the enforced-kernel trap. WSL remains
portability-repro only: it is not Linux seccomp, OS-sandbox enforcement,
Wasmtime fuel, or Linux desktop/Tauri GUI launch proof.

**S111 Windows/WSL reproduction split (recorded 2026-06-03):** the Windows
repro lane records the Studio domain-matrix execution proof as committed evidence
under `proofs/windows/domains/` and `proofs/linux/execution/domains/`. Both rows
run the current 20-example `--suite all` matrix through parse/check/run (60/60
commands) and include the expected BLAKE3 mismatch rejection. WSL remains
execution/portability only: it is not Linux seccomp, OS-sandbox enforcement,
Wasmtime fuel, or Linux desktop/Tauri GUI launch proof.

### Stage P — positioning + cut · build lead (after Stage X + R report back)
| Slice | Title | Goal | Honest scope |
|---|---|---|---|
| S114 | failure-mode-red-team | Build lead coordinates (surge attackers help): actively try to defeat the kernel — laundering, VM-bypass, seal forgery. Record what you COULDN'T break AND any hole found. | Both outcomes recorded; a found hole is a finding, not a thing to hide. |
| S115 | ultrapunch-dossier | Evidence SUPPORTING the #1 claim — capability-bounded acceptance, enforced, cross-OS — NOT asserted; + ranked runners-up. | Every pillar precedented; the integration + diff-gating discipline is the novelty. |
| S116 | use-case-domain-proof-artifacts | The 5-10 use-case domains as evidenced proof artifacts (built / run / sealed). | Proof artifacts, not marketing. |
| S117 | package-pipeline-proof | Signed / SBOM where tooling present (S88 honest-partial → real where possible; named-deferred otherwise). | No signed/SBOM stamp without the tool present. |
| S118 | academic-evidence-package | CMU/MIT/Rice/UC-Berkeley package — every claim sourced to a slice, test, or sealed artifact. | The honest dossier, including what we refuse to claim. |
| S119 | v0_8_1-release-readiness-gate | Whole-runway aggregator, binary-strict by default (the S86 lesson). | READY must carry real binary + cross-OS results. |
| S120 | v0_8_1-cut-decision | Ship the cut-readiness verdict; ESCALATE the tag to Jon (NEVER autonomous) + honest 1.0 horizon (~1yr, validation-gated). | The tag is Jon's; no autonomous tag. |

**S117 Windows/WSL Studio smoke increment (recorded 2026-06-03):** the Windows
lane records a package-pipeline proof increment under `proofs/windows/studio/`
and `proofs/linux/execution/studio/`. The Windows row builds the Tauri Studio
surface and runs `garnet-studio --studio-smoke`, copying the generated
`studio-smoke.json` into a manifest-backed proof bundle. The WSL row replays the
Studio status command contract and status regression tests. This does **not**
close full S117 package-pipeline readiness: WSL is execution/portability only,
not Linux seccomp or OS-sandbox enforcement, and Linux desktop GUI launch, native
Linux packages, signed MSI, winget, Windows ARM64, production, and v1.0 remain
unclaimed.

**STOP + report before S120:** the cut is Jon's decision.

## Verification Expectations

Each slice needs focused tests proportional to the code change, the workspace
test/clippy gates, a dogfood evidence bundle, a green PR dogfood body check, and
CI green before merge.

When a slice is Windows-tagged, it is only Windows-complete when the named
Windows proof command passes on the Windows machine and the result is recorded in
repo evidence.

## Stop Rule

S91-S98 is closed as of #322; lane A (S99-S120) is open. Mac #1 drives the spine
and STOPS to report at three checkpoints: after Stage V (S101 — is the VM seam
closed, which traps reach parity, which stay named-deferred), after Stage U (S104 —
ultrapunch proven with both accept and reject, bundle sealed), and before S120 (the
v0.8.1 cut is Jon's decision — escalate the tag, never push it). After S105, Mac #1
HOLDS the spine while the surge lanes (Stage X) and repro lanes (Stage R) run on
other machines; it does not author S106-S113. No "enforced" claim without a
deterministic trap proven by test; no cross-OS-complete claim from one machine.
