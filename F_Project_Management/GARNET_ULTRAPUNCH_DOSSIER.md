# Garnet v0.8.1 — the ultrapunch dossier (S115)

The headline positioning, **evidence-supported, not asserted**: every claim below
names the proof that backs it (a test, a recorded bundle, or a source location). The
honest concessions are part of the dossier, not a footnote. v0.8.1 is a research-grade
prototype milestone — never production or 1.0.

## The #1 claim

> **Capability-bounded acceptance of agent-authored code, enforced and cross-OS.** A
> project can autonomously **accept** an agent-authored change and **refuse** a silent
> authority expansion — the refusal being a true gate failure, not a warning — with
> the runtime kernel actually *enforcing* the capability + recursion ceilings the
> acceptance rests on, verified identically across Windows, Mac, and Linux, and
> red-teamed.

Every individual pillar is precedented (capability annotations: Austral/E/Koka;
attestation: in-toto/Sigstore; sandboxing: Wasmtime/seccomp). **The novelty is the
integration** — a machine-checkable, sealed capability-diff *gate* wired into an
*autonomous* accept/reject loop that *wraps* the real subcommands (so it cannot drift
from the gates it accepts under) — applied to **agent-authored** code.

**Evidence:**
- The loop + accept/reject demo: `C_Language_Specification/GARNET_ULTRAPUNCH.md`;
  `garnet-cli/tests/ultrapunch_demo.rs` (accept → 4 sealed artifacts; widening →
  refused, never sealed; over-ceiling → trapped, never sealed).
- The enforcement it rests on is REAL and **cross-OS verified**: the consolidated
  trap-parity matrix
  (`proofs/cross-os/matrix/cross-os-trap-parity-20260604-s109/garnet-cross-os-trap-parity-matrix.json`,
  `status=passed`, `cross_os_complete=true`) records three traps — `max_depth`,
  `caps`, and `diff_caps_reject` — each `status:true` on **Windows, Mac, and Linux**
  (Linux additionally `enforcement+linux-seccomp`; WSL is execution-portability,
  excluded from Linux enforcement). The underlying `@max_depth` (S99) + `@caps`
  entry-gate (S100) traps fire on **both** interpreter and VM backends (S101).
  Integrity-verified 31/31 (`garnet_evidence_integrity_status.py`).
- The kernel was **red-teamed**: a real HIGH hole (impl-method surface blindness) was
  found *and fixed* (`C_Language_Specification/GARNET_RED_TEAM.md`, S114). The claim
  is stronger for surviving an adversarial pass, not weaker.

## Ranked runners-up (each evidence-backed)

1. **`diff-caps` as a hard acceptance gate** (S37) — answers "what new authority does
   this change grant?" in one screen and **fails the pipeline** on a widening
   (`exit 1`). No cross-language equivalent as an *acceptance gate*. Evidence:
   `garnet-cli/src/cmd/diff_caps.rs`; `garnet-cli/tests/pr_review_wedge.rs`.
2. **Two-level symmetry** — the project **auto-accepts the agent-authored slices that
   build Garnet** on the *same* discipline the demo applies (diff-caps + enforced run
   + the dogfood gate + autonomous merge). Garnet dogfoods the exact acceptance it
   demonstrates. Evidence: this entire S99–S119 runway (every slice merged that way);
   `GARNET_ULTRAPUNCH.md` § two-level symmetry.
3. **Cross-OS enforcement parity** — the same traps fire on Windows, Mac, and Linux,
   recorded + hash-verified (`GARNET_CROSS_OS_REPRODUCIBILITY.md`).
4. **OS-sandbox policy applied + trapped on a real Linux kernel** (s105b) — S46's
   *generated* seccomp policy denies a disallowed syscall (EPERM) on UTM Debian-12,
   policy-driven. Evidence: `tools/seccomp-apply/PROOF_utm_debian12_aarch64.txt`.
5. **Sealed, tamper-evident provenance** — in-toto seal (S38) bound to source+AST +
   a BLAKE3 transparency log (S68), with autonomous-acceptance attestation (S65/S66).
   Evidence: the accept dossier's `seal.json` + `transparency_log.jsonl`.
6. **6 demonstrator domains, hash-verified on an independent Mac** (S107) — and a
   skeptical honesty filter that rejected overclaims (`GARNET_DOMAIN_SELECTION.md`).

## What we refuse to claim (the honest concessions)

- **Accepted on capability + depth evidence ONLY.** `@caps` + `@max_depth` are
  enforced (both backends); seccomp is applied on **Linux only**. **Not** "fully
  bounded"/"sandboxed"/"safe": `@bounded` (Wasmtime fuel), memory, time, `@mailbox`
  are **declared-not-enforced**; **macOS sandbox-exec and Windows AppContainer are
  named-deferred** (cross-OS parity is at the language-runtime trap layer, not the
  OS-sandbox layer except on Linux).
- **The agent is simulated/scripted** (deterministic, reproducible), not a live LLM
  (S94 `[ACCT-GATED]`).
- **Provenance is self-declared** (bound to digests, not independently witnessed); the
  transparency log is a **local stub** (not Rekor); seals are **unsigned unless cosign
  is present** (cosign/syft/CycloneDX absent → no signing/SBOM).
- **No standard adopted** (RFC-0001 is intent + reference impl).
- **Two LOW red-team findings remain open** (caps-log tail; seal subject-digest),
  recorded honestly within their stub/mitigated scope.
- **No production / 1.0 claim; no tag.** The v0.8.1 cut (S120) is a human decision.

## Why this clears the review bar

The dossier's strength is not that nothing is deferred — it is that **what is claimed
is enforced and verified, what is deferred is named, and the kernel survived an
adversarial red-team that found and fixed a real hole.** Every line above resolves to
a proof in the repo.
