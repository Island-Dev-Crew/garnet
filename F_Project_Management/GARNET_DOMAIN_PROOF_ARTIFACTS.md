# Garnet v0.8.1 — use-case domains as proof artifacts (S116)

The six demonstrator domains selected in S105, rendered **as proof artifacts** — each
one **built, run, and recorded** by the single `garnet` binary over committed fixtures,
not marketing copy. Every command, exit code, and produced-artifact set below is taken
verbatim from the Mac-native execution floor
(`proofs/mac/domains/mac-domain-proofs-20260604-064412/garnet-mac-domain-proofs.json`,
schema `garnet.mac_domain_proofs.v1`, `status=passed`, **6/6 domains, 15 commands
recorded**, `provider_api_called=false`, `source_included=false`,
`evidence_tier=macos-native-domain-execution`, host `macOS-26.5-arm64`).

> **The honest spine (load-bearing).** Every domain emits the verdict **"accepted on
> capability + depth evidence"** — never "fully bounded", "sandboxed", or "safe". The
> accept path produces a sealed dossier; the **refusal paths produce NO seal** (auditable
> negative evidence). Only **one** domain (`accept_provenance_dossier`) seals. The MCP
> domain is a **static report, `enforced=false`** — `mcp-caps` surfaces the declared
> tool-authority surface; it is **not** an MCP-host-enforced runtime budget. This is the
> Mac floor: **not** seccomp, **not** OS-sandbox on macOS, **not** Wasmtime fuel, **not**
> production/v1.0. Windows + Linux rows are consolidated separately (S107→S109); this
> artifact is the macOS-native row.

## The novelty each domain demonstrates

Each pillar (capability annotations, a capability diff, a BLAKE3 chain, a recursion
guard) is reproducible elsewhere with effort. What these artifacts demonstrate is the
**integration**: a machine-checkable, sealed capability-diff **gate** wired into an
**autonomous** accept/reject loop (`garnet agent-loop`) that *wraps* the real `garnet`
subcommands — accept yields the **4 trust artifacts + `decision.md`**, reject yields a
**refusal record with no seal**.

## Domain artifacts (Mac-verified)

### 1. Data-pipeline net-egress widening — REJECT (`@caps`, `diff-caps` stage)
An agent adds "anonymous usage telemetry" that silently widens `{fs}` → `{fs, net}`.
- **Commands (recorded, all `passed`):**
  - `garnet caps .../reject_widen.garnet` → exit **0**
  - `garnet agent-loop --baseline .../baseline.garnet --proposal .../reject_widen.garnet
    --record-dir OUT` → exit **1** (`expected_failure`)
- **Artifacts produced:** `decision.md`, `diff_caps.txt` — **no `seal.json`, no
  `transparency_log.jsonl`.** The widening is refused before any seal.

### 2. Supply-chain installer proc-escalation — REJECT (`@caps`, `diff-caps` stage)
An agent "adds automatic update checking" that silently adds `@caps(proc)` subprocess
spawn to an `@caps(fs, net)` installer.
- **Commands (recorded, all `passed`):**
  - `garnet caps .../supply_chain_proc_escalation.garnet` → exit **0**
  - `garnet diff-caps .../supply_chain_base.garnet
    .../supply_chain_proc_escalation.garnet` → exit **1** (`expected_failure`,
    `caps GAINED: proc` / AUTHORITY EXPANDED)
- **Artifacts produced:** `capability_manifest.json`, `decision.md`, `diff_caps.txt` —
  **no seal.**

### 3. Config processor recursion-depth trap — REJECT (`@max_depth`, enforced RUN)
A capability-clean proposal (`diff-caps` passes) that recurses past its declared
`@max_depth` ceiling — refused by the **enforced kernel at run**, not the static gate.
- **Commands (recorded, all `passed`):**
  - `garnet caps .../reject_overdepth.garnet` → exit **0**
  - `garnet agent-loop --baseline .../baseline.garnet --proposal
    .../reject_overdepth.garnet --record-dir OUT` → exit **1** (`expected_failure`)
- **Artifacts produced:** `decision.md`, `diff_caps.txt` (band 5/5, no expansion),
  `run_trap.txt` (`@max_depth(N) exceeded …`) — **no seal.** Acceptance is decoupled
  from the static gate: capability-clean is not enough; the enforced run must pass.

### 4. Accept-path provenance dossier — ACCEPT (the only sealed domain)
A within-ceiling, no-widening proposal is **accepted**, producing the full sealed
dossier with verifiable provenance.
- **Commands (recorded, all `passed`):**
  - `garnet caps .../accept_proposal.garnet` → exit **0**
  - `garnet agent-loop --baseline .../baseline.garnet --proposal
    .../accept_proposal.garnet --attest agent=scripted-agent-v1 --attest model=simulated
    --gate-version dogfood-gate-v1 --record-dir OUT` → exit **0**
  - `garnet caps-log --verify OUT/transparency_log.jsonl` → exit **0**
- **Artifacts produced (the 4 trust artifacts + run record):** `capability_manifest.json`,
  `decision.md`, `diff_caps.txt`, `run_output.txt`, **`seal.json`**, **`transparency_log.jsonl`**.
  `model=simulated` is stamped into the provenance — the agent is scripted, declared, not
  a live LLM.

### 5. Agent-authored PR-review collapse — diff-caps as the acceptance gate
The `examples/wedge_pr_review` before/after pair: both `check`-clean, but `diff-caps`
flags the authority delta and **fails** — the one-screen "what new authority?" review.
- **Commands (recorded, all `passed`):**
  - `garnet check .../before.garnet` → exit **0** · `garnet check .../after.garnet` → exit **0**
  - `garnet caps .../after.garnet` → exit **0**
  - `garnet diff-caps .../before.garnet .../after.garnet` → exit **1** (`expected_failure`)
- **Artifacts produced:** `capability_manifest.json`, `decision.md`, `diff_caps.txt` —
  **no seal.** The widening is surfaced as a hard gate failure, not a soft warning.

### 6. MCP tool-set authority-creep lens — REPORT ONLY (`enforced=false`)
`mcp-caps` renders the declared tool-authority surface of an MCP toolset for review.
- **Commands (recorded, all `passed`):**
  - `garnet mcp-caps examples/mcp/agent_toolset.mcpcaps` → exit **0**
  - `garnet mcp-caps --format json examples/mcp/agent_toolset.mcpcaps` → exit **0**
- **Artifacts produced:** `decision.md`, `mcp_caps.json`, `mcp_caps.txt`.
- **HONEST: `enforced=false`.** This is a **static report**, not an MCP-host-enforced
  runtime budget — Garnet is not an MCP host. It governs the *declared* tool surface for
  review; it does not intercept the live MCP transport.

## What this artifact is — and is not

- **Is:** macOS-native proof that all six domains build, run, and produce exactly the
  recorded artifact sets via the single committed `garnet` binary — accept seals, the
  four refusal/report domains do not, reproducibly.
- **Is not:** Windows or Linux completion (consolidated at S109); seccomp or macOS
  OS-sandbox enforcement; Wasmtime fuel; an MCP-host runtime budget; a live-LLM agent
  (`provider_api_called=false`); a production / v1.0 readiness claim. **No production /
  1.0 claim** — the v0.8.1 cut (S120) is Jon's; no tag is pushed here.
