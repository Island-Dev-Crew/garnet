# Garnet ultrapunch — capability-bounded acceptance of agent-authored code (S102–S104)

**The #1 capability:** a project can **autonomously ACCEPT agent-authored code on
enforced evidence, and REFUSE a silent authority expansion** — and the refusal is a
true gate failure, not a warning. Every pillar is precedented (capability
annotations, capability diffing, in-toto attestation, transparency logs); the
**novelty is the integration + the diff-gating discipline applied to agent-authored
code**. This document is the reproducible evidence record for that claim.

> **Honest framing (load-bearing).** Acceptance rests ONLY on the two **enforced**
> ceilings — `@caps` host-authority and `@max_depth` recursion (Stage V closed VM
> parity for both). The verdict is **"accepted on capability + depth evidence"** —
> never "fully bounded", "sandboxed", or "safe". `@bounded` (Wasmtime fuel), memory,
> time, `@mailbox`, and OS-level sandbox **application** remain declared-not-enforced.
> The agent in the demo is **simulated/scripted** (deterministic, reproducible), not
> a live LLM (that is S94, `[ACCT-GATED]`). The seal is **unsigned unless cosign is
> present**.

## The loop (`garnet agent-loop`, S102)

A proposal is ACCEPTED only if it passes three gated stages, in order:

1. **diff-caps (S37)** — the declared capability surface must **not widen**. A
   widening exits non-zero (`band 2/5`); the loop REFUSES it — it never runs and is
   never sealed (**Rule 2: widening hard-fails**).
2. **the enforced kernel (S99 `@max_depth` + S100 `@caps` traps)** — the proposal
   must run without tripping an enforced ceiling. A trap REFUSES it; no seal.
3. **seal (S38)** — an accepted proposal is attested, recording the autonomous
   acceptance + agent/model/gate-version provenance (S65/S66; **Rule 3**).

The loop wraps the real `garnet diff-caps`/`run`/`seal` subcommands — it reimplements
no gate, so it cannot drift from the gates it accepts under.

## The demo scenario (S103)

`garnet-cli/tests/fixtures/ultrapunch/`: a capability-bounded report tool declaring
`@caps(fs)`. A simulated agent proposes three changes:

| Proposal | What it does | Gate outcome | Sealed? |
|---|---|---|---|
| `accept_proposal.garnet` | safe refactor; surface stays `{fs}`, within bound | **ACCEPTED** (diff-caps PASS, kernel runs `=> 78`) | **yes** — 4 artifacts |
| `reject_widen.garnet` | "adds telemetry": widens `{fs}` → `{fs, net}` | **REFUSED at diff-caps** (band 2/5) — *the punch* | **no** |
| `reject_overdepth.garnet` | keeps `{fs}` but unwinds `@max_depth(4)` with `digest(20)` | **REFUSED at run** (enforced-kernel trap) | **no** |

The `reject_overdepth` case proves acceptance rests on the **enforced run**, not only
the static capability gate: a proposal can pass diff-caps yet be trapped by the
kernel — and is then never sealed.

## Reproduce (canonical run on this Mac; cross-OS via the matrix)

```sh
# Build, then drive the loop over the committed scenario.
cargo build -p garnet-cli --bin garnet
F=garnet-cli/tests/fixtures/ultrapunch

# ACCEPT — emits the 4 trust artifacts into <dir>.
garnet agent-loop --baseline $F/baseline.garnet --proposal $F/accept_proposal.garnet \
  --record-dir /tmp/ultrapunch/accept --attest agent=scripted-agent-v1 \
  --attest model=simulated --gate-version dogfood-gate-v1
#   -> ACCEPTED on capability+depth evidence ; exit 0

# REJECT (the punch) — a widening is refused; NO seal is written.
garnet agent-loop --baseline $F/baseline.garnet --proposal $F/reject_widen.garnet \
  --record-dir /tmp/ultrapunch/reject
#   -> REJECTED at stage diff-caps (AUTHORITY EXPANDED, band 2/5) ; exit 1

# The transparency-log chain of the accepted proposal verifies (tamper-evident).
garnet caps-log --verify /tmp/ultrapunch/accept/transparency_log.jsonl   # exit 0
```

`scripts/reproduce_ultrapunch.sh` runs this end-to-end and asserts the outcomes;
`garnet-cli/tests/ultrapunch_demo.rs` pins them under `cargo test` (so the ultrapunch
re-proves on the ubuntu/windows/macos matrix).

## The 4 trust artifacts (captured on ACCEPT, in `--record-dir`)

1. **`capability_manifest.json`** (S36) — the declared capability surface (`garnet caps`).
2. **`diff_caps.txt`** (S37) — the capability-surface delta decision (no widening, band 5/5).
3. **`seal.json`** (S38) — the in-toto attestation over the build + capability manifests,
   carrying the autonomous-acceptance provenance (`tool=garnet-agent-loop`,
   `autonomous=true`, `gate_version`, `agent`, `model=simulated`, `decision`).
4. **`transparency_log.jsonl`** (S68) — a BLAKE3 hash-chained, `--verify`-able entry.

## Two-level symmetry — Garnet dogfoods the exact acceptance it demonstrates

The demo's **inner** loop and the process that **builds Garnet** are the *same
discipline at two levels*:

| | OUTER loop — building Garnet (v0.8.1) | INNER loop — the ultrapunch demo |
|---|---|---|
| Proposer | a coding agent authoring each slice | a simulated/scripted agent |
| Capability gate | `diff-caps` on the slice's `@caps` surface | `diff-caps` on the proposal |
| Enforced run | the slice's `cargo test` exercises the enforced kernel (S99/S100) | `garnet run` on the proposal |
| Acceptance signal | the **dogfood-readiness gate at 5/5** + CI green | the three stages pass |
| Acceptance action | **autonomous merge** on the gate (no human click) | accept + **seal** |
| Refusal | a capability widening / failed gate **blocks merge** (human merge required for gate-definition changes) | a widening is **refused, never sealed** |
| Provenance | the seal/evidence chain records agent/model/gate-version | the seal records agent/model/gate-version |

This slice — and every slice in this runway — was accepted by the **outer** loop:
the dogfood-readiness gate scored it 5/5, CI went green, and it merged autonomously.
Garnet does to its *own* construction exactly what the demo does to the simulated
agent's code. That is the strongest form of the claim: not "we built a gate", but
"we accept our own agent-authored construction through it."

## What we refuse to claim

- **Not "fully bounded" / "sandboxed" / "safe."** Acceptance is on **capability +
  depth** evidence only. `@bounded` fuel, memory, time, `@mailbox`, and OS-sandbox
  application are declared-not-enforced (named, never faked).
- **diff-caps reads the DECLARED surface** — it does not prove the absence of
  *undeclared* authority (that is the sandbox-policy job, S46; OS application is
  infra-deferred). The enforced `@caps` trap (S100) is the runtime backstop for
  declared-but-unchained authority.
- **The agent is simulated.** No live-model claim is made here; the attested `model`
  is `simulated`. The live-LLM lane is S94 (`[ACCT-GATED]`).
- **Self-declared provenance.** The seal's authorship/attestation are bound to the
  artifact's digests (S97) but not independently verified — the seal surfaces those
  limitations verbatim.
- **No production / 1.0 claim; no tag.** v0.8.1 is a research-grade-prototype
  milestone; the release tag is a human decision (S120).
