# S114 Acceptance Record — Scoped

- **Date:** 2026-07-12
- **State:** `accepted-scoped`
- **Scope:** first-party CLI/Wasm trust-kernel baseline
- **Decided by:** Jon (repo owner)
- **Recorded by:** Claude Code (Fable 5)
- **Machine-readable artifact:** [`F_Project_Management/LAUNCH/S114_ACCEPTANCE.json`](../LAUNCH/S114_ACCEPTANCE.json) (schema `garnet.s114_acceptance/v1`) — read by the launch-readiness reporter.

## What this record is

This is the human-readable companion to the machine-readable acceptance
artifact. It records Jon's decision to **accept the S114 independent
re-verification result at a stated scope** and to convert each acceptance
caveat into tracked hardening work, per the S114 Plan of Attack (2026-07-12).

The decision basis is Jon's written directive of 2026-07-12 to execute that
plan end to end. Phase 0 of the plan directs recording a Jon-authored/approved
acceptance artifact. The launch reporter **reads** this decision; it does not
grade it. Per the repo's standing rules, only Jon relabels independence — and
**this record does not do that.**

## What this record is NOT

- **Not an independence relabel.** The S114 verdict language in
  `C_Language_Specification/GARNET_RED_TEAM.md`, `CURRENT_STATE.md`,
  `CHANGELOG.md`, and `docs/why.html` is unchanged. S114 remains
  *"independently-re-verified-with-fixes."* This record adds a governance
  acceptance row; it does not upgrade the independence claim.
- **Not a runtime-safety or launch-readiness claim.** `launch_ready` stays
  `False` and the recommendation stays `HOLD`: the static playground, live
  WASM playground, and minimum sealed shelf gates are still open. Accepting
  S114 closes only the S114 governance row.
- **Not an expansion of public claims.** `/why` keeps exactly its two bounded
  enforced claims from PR #471.

## The five recorded conditions

1. **Scope S114 to first-party CLI/Wasm trust-kernel behavior.** Acceptance
   does not imply every embedding mode, platform, or library pathway is sealed.
   *Closed by this record + the launch-ledger `accepted-scoped` row.*
2. **Restore or rerun the independent proof bundle into durable history.** The
   strongest evidence must not live only in unreachable Git objects.
   *Closed by mission phase P1* — landing the Codex verdict bundle (commit
   `61cfbae`) and the Windows lane-2 review bundle (commit `6153726`) under
   tracked `proofs/`, enforced by `scripts/garnet_evidence_integrity_status.py`.
3. **Preserve the PR #471 `/why` wording; do not expand public claims.**
   *Closed by mission phase P2* — a claim fixture asserts the two bounded
   claims remain and forbidden universal-enforcement phrasing stays out.
4. **Do not claim universal runtime enforcement for all `@caps`.** Public
   language must distinguish declared, checker-rejected, runtime-gated, and
   OS-sandboxed authority.
   *Closed by mission phase P2* — the capability enforcement scope table.
5. **Track embedder strict mode, dependency-preload fail-soft, and
   test-helper fail-soft as hardening work.** These are engineering debt, not
   reasons to reject the whole S114 result.
   *Initially closed by mission phase P3* — high-level `Interpreter::new()`
   methods became strict-by-default, and dependency-preload/test-helper/REPL
   preload became fail-closed. The 2026-07-14 post-acceptance delta review
   reopened the low-level Rust-host portion; see the current-status section.

## Review lineage (for audit)

| Role | Who | Detail |
|------|-----|--------|
| Original red-team | Claude fleet | PR #365 — self-found/self-fixed one HIGH, two LOWs |
| **Independent re-verifier** | **Codex (OpenAI)** | 2026-06-25, base `a7f946d` — found two further HIGHs |
| Final review (≠ independent) | Opus (Claude) | found a residual fail-open lane |
| Residual fix | — | S114-FIX-2 deny-by-default mediation, PR #421 (merge `47a7ba7`) |
| Reviewed commit | — | `2e2fe843e87be0c8fc9a4745a5bb138fba597d23` |
| Fix commits | — | `4994867`, `47a7ba7` |
| Verdict relabel | Jon | PR #438, merged 2026-06-29; scoped acceptance recorded separately on 2026-07-12 |

## Decision-date scope limits (historical snapshot)

The following bullets record what the acceptance artifact said on 2026-07-12.
They are preserved in git at commit `155dec9`; the current status immediately
below supersedes their future-tense wording.

- **Third-party embedders** of `garnet-interp` / `garnet-vm` can still run
  permissive (unframed); hardened in P3, not covered by this acceptance.
- **Checker-only rows** (`time::*`, `uuid` v4/v7) and **declared-only rows**
  (`ffi`, `net_internal`) are not uniformly runtime-gated; documented in the
  P2 scope table.
- **OS-level sandbox** is generated policy (`enforced:false`), applied only on
  Linux via an external reference harness; macOS/Windows deferred.
- **`memory::*` natives** are bridge-only and caps-invisible.

## Post-acceptance current status (2026-07-14)

- **Acceptance:** Jon's decision is recorded as `accepted-scoped`; the separate
  verdict remains `independently-re-verified-with-fixes`. This is not an
  independence relabel.
- **Conditions 1–4:** closed. Durable evidence is reachable, the two bounded
  `/why` claims remain fixed in count and meaning, and the normative scope table
  separates checker/runtime/entry/OS enforcement.
- **Condition 5:** **reopened in part by adversarial delta review.** The CLI,
  dependency preload, test helper, and REPL preload fail closed, and
  `Interpreter::new()` is strict during its high-level load/eval/call methods.
  However, a Rust host can extract a native through the public `global`/`Value`
  surface and invoke it via public low-level eval APIs outside the instance's
  strict scope. That path is not reachable from Garnet source, the first-party
  CLI, or the current Wasm wrapper, but it is inconsistent with an unqualified
  embedder-wide strict-default statement and must be fixed or permanently
  fenced before launch.
- **Wasm:** WV-5 proves wasm32 build, wasm-pack web/Node packaging, real Node
  execution, and a fail-closed authority result. Browser-page execution remains
  unproven until the W-PLAY Playwright gate passes.
- **Remaining scope limits:** checker-only/declared-only capability rows,
  Linux-only external seccomp application, caps-invisible `memory::*`, and the
  explicit trusted-host `new_permissive()` opt-out remain exactly bounded.

See [`S114_ACCEPTANCE.json`](../LAUNCH/S114_ACCEPTANCE.json) for the
machine-readable form the reporter consumes.
