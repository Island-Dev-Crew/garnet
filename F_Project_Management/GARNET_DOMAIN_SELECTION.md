# Garnet ultrapunch — core demonstrator domains (S105)

This selects the **demonstrator domains** for the v0.8.1 real-world-proof finale and,
for each, the **specific trust-artifact delta a non-Garnet build cannot produce** —
honestly. Authoring this **unblocks Stage X**: each domain ships a concrete per-OS
proof command the surge lanes (Windows-Codex, Mac-Codex, Linux) run via the single
`garnet` binary over committed fixtures for byte-identical reproduction.

> **The honest gap (load-bearing).** Every individual pillar — capability
> annotations, a capability diff, a BLAKE3 hash chain, a recursion guard — is
> reproducible elsewhere *with effort*. The novelty is the **integration**: a
> machine-checkable, sealed capability-diff **gate** wired into an **autonomous**
> accept/reject loop (`garnet agent-loop`) that *wraps* the real `garnet`
> subcommands (so it cannot drift from the gates it accepts under) and produces a
> reproducible dossier — the **4 trust artifacts + `decision.md`** on accept, a
> **refusal record with NO seal** on reject. Each domain emits the verdict
> **"accepted on capability + depth evidence"** — never "fully bounded", "sandboxed",
> or "safe".

All domains rest **only** on the two **enforced** ceilings — `@caps` (S100 runtime
host-authority trap; the static `diff-caps` gate refuses widenings before run) and
`@max_depth` (S99 recursion trap, VM/interp parity S101). `@bounded` (a *static*
bounded-loop verifier — **not** runtime fuel metering), memory, time, `@mailbox`,
and OS-level sandbox **application** remain declared-not-enforced
(named, never faked).

## The domains

### 1. Data-pipeline net-egress widening (the canonical headline) · `@caps`
An agent refactors a `@caps(fs)` ETL job, proposing "anonymous usage telemetry"
that silently adds HTTP egress (`{fs}` → `{fs, net}`), framed as instrumentation.
- **Trust-artifact delta:** `agent-loop` stage 1 (`diff-caps`) emits `diff_caps.txt`
  with `+ caps GAINED: net` / `AUTHORITY EXPANDED (band 2/5)`, exits 1, writes the
  rejected `decision.md`, and **no `seal.json` / no `transparency_log.jsonl`**.
- **Non-Garnet gap:** no INTEGRATED machine-checkable gate that derives the
  per-function aggregate surface, computes a typed widening verdict, and **hard-fails
  the acceptance loop before run/seal** as one sealed dossier.
- **Stage X proof:** `garnet agent-loop --baseline F/baseline.garnet --proposal
  F/reject_widen.garnet --record-dir OUT` (F=`garnet-cli/tests/fixtures/ultrapunch`)
  → exit 1, `OUT/diff_caps.txt` contains `AUTHORITY EXPANDED`, **no `OUT/seal.json`**.
- **Caveat:** `diff-caps` reads the DECLARED surface; it does not prove the code
  lacks undeclared authority (S46's job, not enforced). The runtime `@caps` trap is
  the backstop for undeclared use that executes.

### 2. Supply-chain installer proc-escalation (`fs,net` → `+proc`) · `@caps`
An agent "adds automatic update checking" to a checksum-verifying `@caps(fs, net)`
installer, silently adding subprocess-spawn (`@caps(proc)`) to invoke a package
manager.
- **Trust-artifact delta:** `diff-caps` detects the `proc` addition (`+ caps GAINED:
  proc`, band 2/5, exit 1); the loop writes the refusal `decision.md`, **no seal**.
- **Non-Garnet gap:** human review misses a `proc` escalation buried in an imported
  helper; a linter warns but does not fail. Garnet's gap is the sealed, diffable
  **REFUSAL bound into the loop** so the widened installer never runs and produces
  no seal-record — auditable negative evidence, not a soft warning.
- **Stage X proof (verified):** `printf '@caps(fs, net)\ndef main() { 1 }\n' > base;
  printf '@caps(fs, net, proc)\ndef main() { 1 }\n' > prop; garnet diff-caps base
  prop` → stdout `caps GAINED: proc` + `AUTHORITY EXPANDED`, **exit 1**.
- **Caveat:** gates the declared surface, not actual syscall use; declared-but-unused
  `proc` is not audited; undeclared `proc` that runs is caught only by the runtime
  `@caps` trap. No claim the installer is "safe".

### 3. Config processor recursion-depth trap (passes `diff-caps`, traps at run) · `@max_depth`
An agent "optimizes" a recursive config/template processor declaring `@caps(fs),
@max_depth(8)`, keeping the capability surface identical but recursing past the
declared ceiling.
- **Trust-artifact delta:** `agent-loop` passes stage 1 (`diff_caps.txt`: band 5/5,
  no expansion) but stage 2 — the **enforced kernel** — TRAPS: `run_trap.txt` records
  `bounded: @max_depth(N) exceeded ... recursion depth M`, exit 1, **no seal**.
- **Non-Garnet gap:** acceptance rests on the enforced **RUN**, decoupled from the
  static capability gate — a capability-clean proposal is still refused. The trap is
  a DECLARED, enforced ceiling (interp/VM parity, S99/S101), not an incidental stack
  overflow.
- **Stage X proof:** `garnet agent-loop --baseline F/baseline.garnet --proposal
  F/reject_overdepth.garnet --record-dir OUT` → stage-run REJECT, `OUT/run_trap.txt`
  contains `@max_depth(4) exceeded for \`digest\``, **no seal**. Repeat `--backend vm`
  for parity.
- **Caveat:** bounds **depth** on `@max_depth`-declared functions only; not CPU time,
  heap, or recursion in unannotated helpers. `@max_depth` is not a cost/wall-clock
  bound; no termination guarantee.

### 4. Accept-path provenance dossier (the positive proof) · `@caps` + `@max_depth`
A capability-clean, depth-respecting refactor is **ACCEPTED autonomously** and sealed.
- **Trust-artifact delta:** the loop writes all **4 artifacts** + `decision.md`:
  `capability_manifest.json` (S36), `diff_caps.txt` (S37, band 5/5), `seal.json`
  (S38, in-toto Statement attesting `autonomous=true`,
  `decision=accepted-on-capability+depth-evidence`, agent/model/`gate_version`), and
  `transparency_log.jsonl` (S68, BLAKE3-chained; `caps-log --verify` → "chain intact").
- **Non-Garnet gap:** not the pieces, but a **single sealed dossier where the
  acceptance verdict is bound to (and reproducible from)** the capability manifest,
  the diff verdict, the enforced-run value, and a tamper-evident log entry — produced
  autonomously, wrap-don't-rebuild. The accept case makes the refusals meaningful
  rather than vacuous.
- **Stage X proof:** `garnet agent-loop --baseline F/baseline.garnet --proposal
  F/accept_proposal.garnet --record-dir OUT --attest model=simulated --gate-version
  dogfood-gate-v1` → exit 0; `OUT` contains all 5 files; `garnet caps-log --verify
  OUT/transparency_log.jsonl` → "chain intact".
- **Caveat:** the seal is **UNSIGNED unless cosign is present**; provenance is
  **self-declared** (bound to digests, not third-party-witnessed). The transparency
  log is a **local** BLAKE3 chain (no Rekor/witness). The agent is **simulated**
  (`model=simulated`), not a live LLM (S94, `[ACCT-GATED]`).

### 5. Agent-authored PR-review collapse (`diff-caps` as the merge gate) · `@caps` + `@max_depth`
`diff-caps` makes an authority widening a **pipeline failure** (exit 1) before the
kernel runs — not advice — answering "what new authority does this PR grant?" in one
screen.
- **Trust-artifact delta:** the refused slice yields `diff_caps.txt` + the
  `RejectedDiffCaps` `decision.md`, no seal; a clean refactor yields the 4 artifacts.
  The **two-level dogfood angle**: Garnet's own construction merges agent-authored
  slices under the *same* gate (inner loop == outer loop).
- **Non-Garnet gap:** plain review + CI has no integrated authority-difference gate
  that **hard-fails a merge**; reviewers miss implicit caps in imported modules. The
  architectural identity (the demo's gate == the gate that accepts Garnet's own
  slices) is something a non-Garnet build cannot assert.
- **Stage X proof:** `bash scripts/reproduce_ultrapunch.sh` (or `cargo test -p
  garnet-cli --test ultrapunch_demo`) across the ubuntu/windows/macos matrix — the
  three sub-cases (accept→4 artifacts / widen→refused, no seal / overdepth→trapped,
  no seal).
- **Caveat:** the agent is **simulated**; `autonomous=true` means the loop decided
  accept/reject without human approval, not that an LLM authored it live. `diff-caps`
  proves the declared surface did not widen, not that the code honors its declaration
  at the syscall level. "Accepted" = "on capability + depth evidence", never "safe".

### 6. MCP tool-set authority-creep lens (`mcp-caps`, static **report**, not a hard-fail) · `@caps`
An orchestrator's MCP tool-set adds a high-authority shell tool (`proc`/`ffi`),
expanding aggregate authority that is otherwise invisible because MCP tools don't
attest capabilities.
- **Trust-artifact delta:** `garnet mcp-caps prop.mcpcaps` reports per-tool +
  aggregate surface and flags `high-authority: \`shell\` declares \`proc\`/\`ffi\` —
  review`; the creep is visible by comparing the `aggregate authority:` line across
  two `mcp-caps` runs (verified).
- **Non-Garnet gap:** plain MCP setups allow-all or require manual review; there is
  no attested, diffable **per-tool capability surface**. Garnet brings the `@caps`
  declared-surface lens to MCP tool-sets where capability attestation is absent.
- **Stage X proof (verified):** `printf 'filesystem: fs\nfetch: net\nshell: proc,
  ffi\n' > prop.mcpcaps; garnet mcp-caps prop.mcpcaps` → stdout `high-authority` +
  `aggregate authority: ffi, fs, net, proc`.
- **Caveat (corrected from the design draft):** `diff-caps` does **NOT** accept
  `.mcpcaps` (it parses `.garnet` — verified to error). So this domain is a static
  **report + flag**, **not** a `diff-caps` hard-fail. It is **strictly self-declared,
  NOT MCP-host-enforced** — Garnet does not intercept or meter tool calls. It proves
  the declared tool-set surface is *visible and reviewable*, not that tools honor
  their declarations.

## Rejected overclaims (the honesty filter, including one I caught empirically)

- **`diff-caps` as an MCP-tool-set hard-fail gate** — REJECTED. `diff-caps` parses
  `.garnet`; on a `.mcpcaps` file it errors. Domain 6 is scoped to the `mcp-caps`
  report + flag only. *(Caught by empirical verification, not the design draft.)*
- **`@bounded`/memory/time as enforced ceilings** — REJECTED. `@bounded` is a
  *static* bounded-loop verifier (`bounds.rs`: "not runtime fuel metering"); treating
  it as an enforced runtime budget would be false provenance.
- **`caps-log` proves distributed/witnessed tamper-evidence** — REJECTED. It is a
  **local** BLAKE3-chained stub (no Rekor/witness); `--verify` is local recomputation.
- **The seal is signed/third-party-verified by default** — REJECTED. UNSIGNED unless
  cosign is present; self-declared provenance bound to digests.
- **A live LLM authored the proposals** — REJECTED. The agent is simulated/scripted
  (on-disk fixtures); the live-LLM lane is S94 (`[ACCT-GATED]`).
- **"Python/Rust literally cannot do this"** — REJECTED. Each pillar is reproducible
  with effort; the honest gap is the **integrated, sealed, autonomous** capability-
  diff gate, not any single piece.
- **A bounded-simulation/digital-twin domain** — REJECTED as adding nothing over the
  net/proc widening + accept-dossier domains, and leaning on a "tamper-proof without
  external signing" overclaim (the log is a local stub).

## For the surge lanes (Stage X)

Each domain ships a concrete per-OS proof driven by the single `garnet` binary over
committed fixtures (`garnet-cli/tests/fixtures/ultrapunch/` for domains 1, 3, 4, 5;
inline `.garnet`/`.mcpcaps` for 2, 6). A domain is **cross-OS-complete only when all
three machines (Windows/Mac/Linux) recorded its proof** — no single-machine claim.
The verdict every domain emits is **"accepted on capability + depth evidence"**;
v0.8.1 is a research-grade prototype, never production/1.0.
