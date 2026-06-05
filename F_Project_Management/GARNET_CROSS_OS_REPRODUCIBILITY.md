# Garnet v0.8.1 — cross-OS reproducibility ledger (S112 + S113)

The consolidated, **integrity-verified** status of the surge lanes' cross-OS
evidence (Stage X + R). This is the floor Stage P (S114–S119) rests on: nothing is
presented as cross-OS-verified unless its proof bundle hash-verifies here.

## Integrity floor — 31/31 bundles verify

`scripts/garnet_evidence_integrity_status.py --gate` verifies **every**
`proofs/**/MANIFEST.sha256` against the committed bytes: **31/31 bundles pass**.

> **Defect found + closed (transparency).** Five Windows/WSL bundles initially
> failed their own manifests — not tampering or missing evidence, but **git EOL
> normalization** (CRLF→LF) of the text proof files *after* sealing (the
> CRLF-restored hash matched the original seal byte-for-byte). Fixed by re-sealing
> those manifests against the committed bytes (file lists preserved exactly; only
> the hash column changed) and adding `proofs/** -text` to `.gitattributes` so proof
> bundles are never EOL-normalized again. This is the cross-OS analog of the S82
> seal-determinism fix, extended to evidence bundles.

## What is cross-OS VERIFIED (the floor)

| Capability | Windows | Mac | Linux | Evidence |
|---|---|---|---|---|
| `@max_depth` trap (S99) | ✅ 9 tests | ✅ 9 tests | ✅ 9 tests | `proofs/{windows,mac,linux}/.../bounded_enforcement*`; matrix row `trap=max_depth` |
| `@caps` trap incl. S92 entry-gate (S100) | ✅ 17 tests | ✅ 17 tests | ✅ 17 tests | `proofs/{...}/caps_enforcement*`; matrix `trap=caps` |
| `diff-caps` widening reject (S37) | ✅ | ✅ | ✅ | matrix `trap=diff_caps_reject` |
| seccomp policy **applied + trapped** (s105b) | — | — | ✅ EPERM 3/3 | `proofs/linux/enforcement/.../linux-seccomp-apply-stdout.txt` |
| ultrapunch accept + reject (S103/S104) | replay | **canonical** | replay (WSL) | `proofs/.../ultrapunch/` (accept + reject-widen + reject-overdepth) |
| 6 demonstrator domains (S105) | replay | ✅ **6/6 hash-verified** | replay (WSL) | `proofs/mac/domains/mac-domain-proofs-*` |
| package pipeline: Tauri/WSL/WSLg Studio launch (S117) | ✅ | ✅ (UI) | ✅ (WSL/WSLg) | 19 S117 bundles |

The cross-OS trap-parity matrix
(`proofs/cross-os/matrix/cross-os-trap-parity-20260604-s109/garnet-cross-os-trap-parity-matrix.json`,
`cross_os_complete=true`) consolidates the enforcement rows with its own
`honest_scope` block, preserved verbatim.

## The honest fences (named-deferred — carried into EVERY Stage P artifact)

- **`cross_os_complete=true` is LANGUAGE/runtime-trap parity** (`@max_depth`/`@caps`/
  `diff-caps`) across Win/Mac/Linux — **NOT** OS-sandbox enforcement on all three.
  Only **Linux** has the seccomp policy actually applied (s105b). **macOS
  sandbox-exec and Windows AppContainer are named-deferred.** This is the single
  sharpest overclaim risk; do not blur it.
- **`@bounded` (Wasmtime fuel), memory, time, `@mailbox`** ceilings: declared-not-
  enforced on all OSes (not recorded anywhere).
- **WSL = execution/portability only**, explicitly excluded from Linux *enforcement*.
- **Signed release artifacts + SBOM** (cosign / syft / CycloneDX): tooling **absent**;
  named-deferred. All manifests are plain SHA256, not cryptographic signatures.
- **The agent is simulated** (recorded demo), not a live LLM (S94 `[ACCT-GATED]`).
- **Mac (S107) is an *independent* machine**, not the canonical enforcement host —
  labelled as such.
- No production / 1.0 / tag readiness is claimed; the v0.8.1 cut (S120) is Jon's.

## Reconciliation notes

- The goal ledger (`.dogfood/goal.json`) drifted: `s117` was marked pending while 19
  S117 PRs (#337–#355) merged. Reconciled to `merged` here.
- The grounding survey flagged the s110/s111 repro bundles as integrity-failing with
  "reject-* missing"; **direct verification corrected both**: the failures were
  EOL-only (re-sealed), and the s110 ultrapunch bundle is complete (accept +
  reject-widen + reject-overdepth, rejects correctly unsealed). After the re-seal,
  all repro bundles hash-verify.
