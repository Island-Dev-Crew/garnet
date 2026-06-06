# Garnet Post-v0.8.1 System Handoff

> **For agentic workers:** REQUIRED OPERATING MODE: read current repo truth before acting, use one branch/PR per gate unless Jon explicitly approves a release cut, and preserve calibrated honesty. This handoff is episodic project memory, not a substitute for current source, specs, `AGENTS.md`, reporter scripts, or live GitHub state.

**Created:** 2026-06-05

**Baseline observed before S120:** `main` at `ca13bb2dc77109b6f621569b927839c7e7acc1bf`, S119 merged, `scripts/garnet_v0_8_1_release_readiness.py --gate` reporting `release_ready: true`, and no open PRs. Re-check before acting.

**Primary objective:** Cut `v0.8.1` as a research-grade evidence milestone, then move Garnet from proof-complete primitives toward a public, independent, product-shaped trust workbench without drifting into production, 1.0, or independence claims the evidence does not support.

**Core thesis after the research synthesis:** Garnet's adoption-worthy problem is not "a new language for LLMs" or "two modes." Garnet's problem is that agent-authored code is entering repositories faster than humans can review its authority, provenance, and runtime behavior. Garnet should become the evidence-native accept/refuse layer for that workflow.

---

## Non-Negotiable Boundaries

- Never claim production readiness, v1.0, or production safety for v0.8.x.
- "Enforced" only means a deterministic trap exists and has been proven by test.
- Do not claim independent red-team evidence unless a genuinely independent attacker/reviewer is named and provenance proves it.
- Do not call the S114 red-team independent. It found and fixed a real HIGH issue, but current GitHub metadata does not prove external independence.
- S120 and release tags are Jon-owned. Agents may prepare, verify, and document. They must not autonomously push release tags.
- A PR may not modify the gate it merges under. Changes to dogfood skill/CI/diff-caps thresholds/capability-manifest standards require a human merge.
- A diff-caps widening must fail the gate and block merge unless the branch is explicitly a human-reviewed standard/threshold change.
- Every autonomous merge must name agent, model, and gate version.
- Preserve raw evidence. Compression, summarization, LLM review, or Headroom use may assist context management, but must not replace canonical logs, bundles, artifacts, or reviewer-facing source pointers.

---

## Current Cut Directive For Claude Code On MacBook Pro

Paste this into Claude Code only when Jon is intentionally performing the human S120 cut.

```text
ROLE: Claude Code on MacBook Pro - Jon-authorized S120 human release-cut lane for Garnet v0.8.1.

SOURCE OF TRUTH: live `main`, `F_Project_Management/GARNET_v0_8_1_PLAN.md`, `F_Project_Management/AGENT_COORDINATION_LEDGER.md`, `scripts/garnet_v0_8_1_release_readiness.py --gate`, and live GitHub PR/CI state. Do not rely on stale transcript summaries.

MISSION:
1. Re-sync `main` from Island-Dev-Crew/garnet.
2. Confirm `main == origin/main`.
3. Confirm no open PRs need merge/maintenance.
4. Run the v0.8.1 readiness gate:
   `python3 scripts/garnet_v0_8_1_release_readiness.py --gate`
5. Confirm the gate reports `release_ready: true`, `binary_strict: true`, `sub_gates_pass: true`, `cross_os_complete: true`, and `integrity_ok: true`.
6. Re-state all honesty anchors and deferrals in the release note:
   - research-grade prototype, not production/1.0
   - S107 low-confidence marker from original bundled Mac proof, despite remedial re-attestation PRs
   - S114 found/fixed a HIGH issue but is not independently proven by GitHub metadata
   - Linux seccomp is Linux-only OS-sandbox application
   - macOS/Windows OS sandbox application deferred
   - `@bounded`, memory, time, and `@mailbox` runtime ceilings declared-not-enforced
   - live LLM agent simulated/scripted
   - seals unsigned, transparency log local stub, external SBOM/signing not wired
   - two LOW red-team findings open: caps-log tail and seal subject-digest
7. If and only if Jon explicitly approves the cut inside this session, prepare the v0.8.1 tag/release action. Ask Jon for the final human confirmation immediately before pushing the tag.
8. After the cut, do not start new implementation. Write a concise post-cut state note: tag pushed or not pushed, exact commit, gate output summary, release URL if created, and next recommended gate: S121-S130 Truth Sync Gate.

PROHIBITIONS:
- Do not claim production readiness.
- Do not call S114 independent.
- Do not enter the S121-S200 implementation runway in this same release-cut pass.
- Do not rewrite gates, CI, dogfood rules, diff-caps thresholds, or capability-manifest standards.
- Do not hide deferred items from the release note.
```

---

## Fleet Map

### MacBook Pro, 48 GB RAM

Primary role: release coordinator, high-context strategy/verification, Studio/macOS proof, and final reviewer-facing synthesis.

Recommended tools:

- Claude Code Opus 4.8 Max Ultra Code for long-running release/gate orchestration.
- Codex with GitHub, Chrome/browser, computer-use, documents/presentations/spreadsheets/product-design when needed.
- UTM Debian for Linux cross-checks when a Mac-local Linux repro is useful.
- Browser/Chrome/computer-use only for UI proof, evidence capture, and GitHub flow. No destructive settings/secrets/permissions work.

Best lanes:

- S120 human cut support.
- S121-S130 truth sync and release-state reconciliation.
- S131-S140 public positioning and website proof-cockpit design.
- S187-S194 flagship Studio/Workbench product proof.
- S195-S200 v0.8.2 readiness aggregation.

### MacBook Air M5, 16 GB RAM

Primary role: lower-risk documentation, site, research, Headroom pilot, and independent read-only review.

Recommended tools:

- Codex or Claude Code for docs/site copy, report generation, and static validation.
- UTM Debian for small Linux proof replay only, not heavy multi-hour gates.
- Headroom pilot should start here, isolated from canonical release evidence.

Best lanes:

- Independent repo/site drift audit.
- Research/copy replacement plan.
- Public docs and `A_Research_Papers/` readability cleanup.
- Headroom compression pilot on non-canonical planning artifacts.

### Windows NUC, 32 GB RAM

Primary role: Windows native, Tauri, WSL/Linux smoke, Java/OpenJDK checks, and cross-OS proof replay.

Recommended tools:

- Codex and Claude Code for Windows PR lanes.
- Tauri for Windows Studio builds and Linux/WSL smoke.
- OpenJDK where Java interop, converter, or package proof lanes require it.
- Browser/Chrome/computer-use for Studio UI proof and screenshots.

Best lanes:

- Windows Studio smoke.
- WSL/WSLg Linux Tauri package smoke.
- Java/OpenJDK converter/advisory experiments.
- Cross-OS matrix replay for v0.8.2 readiness.

---

## Post-Cut Gate Runway: S121-S200

This is intentionally gate/stage shaped rather than slice shaped. Each gate can be decomposed into one-PR slices only after recon establishes current truth.

| Gate | Target Range | Owner Bias | Exit Criteria |
|---|---:|---|---|
| Human Release Cut | S120 | Jon + Claude Code on MBP | `v0.8.1` tag/release created by Jon or explicitly deferred with exact reason. No autonomous tag. |
| Truth Sync Gate | S121-S130 | MBP primary, Air review | README, docs site, status page, release notes, ledger, and reporter outputs agree on v0.8.1 truth. Stale v0.5/status claims corrected. Deferrals visible. |
| Public Positioning Gate | S131-S140 | MBP/Air split | Comparator-led headline replaced or demoted. Site leads with evidence-native agent-code acceptance. Product proof visible in first viewport. |
| Independent Trust Gate | S141-S150 | External/Jon-owned attacker plus agent support | Independent red-team rerun completed or explicitly deferred. Findings fixed/deferred with provenance. No self-authored independence claim. |
| Runtime Boundaries Gate | S151-S160 | MBP/Windows split | `@bounded` fuel or equivalent deterministic runtime ceiling has trap proof, or remains named-deferred. No declared-not-enforced surface is marketed as enforced. |
| OS Enforcement Gate | S161-S170 | Windows NUC + Mac UTM | Linux seccomp replay remains clean. macOS/Windows sandbox claims only advance if trap proof exists. WSL remains portability evidence, not Linux enforcement. |
| Seal Integrity Gate | S171-S178 | MBP primary | LOW red-team issues addressed: caps-log tail binding and cap-aware seal subject digest. Tamper tests prove rejection. Signing/SBOM story either wired or honestly deferred. |
| Local LLM Advisory Gate | S179-S186 | Air pilot, MBP review | Local/provider-neutral LLM advisory can summarize and critique without authorizing. Raw source omitted by default where required. Deterministic gates remain authoritative. |
| Flagship Workbench Gate | S187-S194 | MBP + Windows UI proof | Garnet Trust Review Workbench demonstrates PR/package/MCP/generated-code intake, diff-caps, trap run, accept/refuse dossier, and Studio UI proof. |
| v0.8.2 Readiness Gate | S195-S200 | MBP coordinator, all machines verify | Binary gate aggregates post-v0.8.1 truth and returns READY TO CUT v0.8.2 only if truth sync, independent trust status, runtime/OS boundaries, seal integrity, LLM advisory, and flagship proof are current. Jon owns the v0.8.2 cut/tag. |

---

## Goal Modes To Instantiate

Use these as starting prompts for long-running Claude/Codex sessions. Each goal mode must begin with live recon and must stop/report before mutation if repo truth is unclear.

### Goal Mode A: Release Cut And Post-Cut Freeze

**Machine:** MacBook Pro.

**Agent:** Claude Code Opus 4.8 Max Ultra Code.

**Objective:** Execute Jon-authorized S120 cut support for `v0.8.1`, then stop at a clean post-cut state note.

**Hard stop:** Do not enter S121-S200 implementation in the same session.

**Verification:** `git fetch origin main --prune`, `git status --short --branch`, `gh pr list`, `python3 scripts/garnet_v0_8_1_release_readiness.py --gate`, and release/tag verification if Jon completes the cut.

### Goal Mode B: Truth Sync Gate

**Machine:** MacBook Pro primary, MacBook Air review.

**Agent:** Codex or Claude Code.

**Objective:** Bring public and repo truth into alignment after `v0.8.1`.

**Scope:** README, `docs/index.html`, `docs/status.html`, release-facing project-management docs, and reporter references. Keep the claim surface honest.

**Key checks:** adoption surface reporter, MIT readiness reporter, promo reporter, release readiness gate, and manual scan for stale `v0.5.0`, stale percentages, comparator-first headline, or missing deferrals.

### Goal Mode C: Positioning And Website Proof Cockpit

**Machine:** MacBook Air for design/copy, MacBook Pro for final proof.

**Agent:** Codex with product-design/browser support, then Claude/Codex implementation.

**Objective:** Replace the comparator-led homepage story with an evidence-native Studio trust workflow.

**Output:** A public site that shows proposal -> capability diff -> trap run -> accept/refuse -> seal/provenance, using real artifacts and honest status language.

**Do not:** Create abstract marketing claims without artifact links. Do not imply production readiness or active broad LLM conversion.

### Goal Mode D: Independent Trust Gate

**Machine:** External reviewer preferred; MacBook Pro only coordinates.

**Agent:** Jon-selected independent attacker or reviewer. Codex/Claude may package evidence but must not self-grade.

**Objective:** Re-run S114-style red-team independently.

**Exit:** Independent report with named author/provenance, reproduced attack attempts, findings, fixes/deferred issues, and reviewer-facing evidence. If no independent attacker is available, record the gap and do not bless independence.

### Goal Mode E: Runtime And OS Boundary Proof

**Machine:** Windows NUC plus MacBook Pro/UTM Debian.

**Agent:** Codex/Claude split by OS.

**Objective:** Advance declared-not-enforced runtime and OS boundaries only through deterministic trap tests.

**Priority:** Wasmtime fuel or equivalent `@bounded` trap first, then memory/time/mailbox if feasible. OS sandbox expansion comes after deterministic trap design, not before.

**Exit:** Trap tests fail before fix and pass after fix, with Mac/Windows/Linux deltas honestly recorded.

### Goal Mode F: Seal Integrity And Supply Chain Evidence

**Machine:** MacBook Pro primary, Windows NUC replay.

**Agent:** Codex for implementation, Claude for review.

**Objective:** Close caps-log tail and seal subject-digest LOW findings, then decide whether external signing/SBOM integration is ready or deferred.

**Exit:** Tamper tests prove forged tail/digest mismatch rejection. Any signing/SBOM claim must be backed by actual tool execution.

### Goal Mode G: Local LLM Advisory, Not Authority

**Machine:** MacBook Air pilot, MacBook Pro review.

**Agent:** Codex/Claude with local Ollama/provider-neutral path if available.

**Objective:** Add a quarantined local LLM advisory lane that can explain risk but cannot authorize acceptance.

**Rules:** Deterministic context pack first. Source omitted by default where required. Model output tagged non-deterministic. `diff-caps`, `garnet check`, tests, dogfood, and seal remain authoritative.

**Exit:** Advisory output improves human review without changing gate decisions. If it cannot be measured, keep it experimental.

### Goal Mode H: Headroom Pilot

**Machine:** MacBook Air first.

**Agent:** Codex or Claude Code with explicit pilot scope.

**Objective:** Evaluate `chopratejas/headroom` as a context-compression layer for long Garnet agent sessions.

**Why:** Headroom currently presents itself as a local context optimization layer for Claude Code, Codex, Cursor, Aider, MCP, and proxy/library use. Its GitHub page reports agent wrapping, reversible compression, cross-agent memory, and `headroom learn`.

**Rules:**

- Treat Headroom as productivity infrastructure only, never as release evidence.
- Preserve raw logs and raw proof bundles before compression.
- Do not allow `headroom learn` to write `AGENTS.md`, `CLAUDE.md`, or repo procedural memory without human review.
- Do not route secrets or signing credentials through the pilot.
- Compare a Headroom-assisted planning session against an uncompressed baseline before using it in high-stakes release work.

**Exit:** Written pilot note with install mode, commands used, files touched, compression savings, observed failures, and recommendation: adopt, defer, or reject.

### Goal Mode I: Flagship Garnet Trust Review Workbench

**Machine:** MacBook Pro primary, Windows NUC cross-OS, Air copy/research.

**Agent:** Multi-lane Codex/Claude.

**Objective:** Build the product-shaped flagship: an intake workbench for agent-authored changes.

**Workflow:** Ingest PR/package/MCP/generated-code candidate -> derive capability surface -> run diff-caps -> run deterministic trap checks -> emit accept/refuse dossier -> optionally add local LLM advisory summary -> present in Studio.

**Exit:** One end-to-end demo where Garnet accepts a safe candidate and rejects authority creep, with four trust artifacts and Studio UI proof.

---

## Branching And Merge Discipline

- Start each lane from fresh `main`.
- Use separate worktrees or separate machines for concurrent work.
- Branch prefix: `codex/` for Codex, `claude/` for Claude Code unless GitHub flow already established otherwise.
- One PR per gate slice after decomposition.
- Do not directly commit to `main` except Jon-owned release/tag operations.
- Every PR includes:
  - source-of-truth recon summary
  - exact scope
  - tests run
  - dogfood/readiness gate output where relevant
  - honesty/deferred claims
  - autonomous merge provenance, if applicable
- If a lane touches CI, dogfood gates, diff-caps thresholds, capability standards, release cut logic, or signing/secrets policy, it is human-merge only.

---

## Verification Ladder By Work Type

### Docs/site truth sync

Minimum:

```bash
python3 scripts/garnet_adoption_surface_status.py
python3 scripts/garnet_mit_readiness_status.py
python3 scripts/garnet_promo_video_status.py
python3 scripts/garnet_v0_8_1_release_readiness.py --gate
python3 scripts/check-agent-contracts.py
git diff --check
```

Add browser/Chrome screenshots when public UI changed.

### Rust/runtime/tooling work

Minimum:

```bash
cargo fmt --all -- --check
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/garnet_v0_8_1_release_readiness.py --gate
```

Add focused tests before the workspace run.

### Release-impacting work

Minimum:

```bash
git fetch origin main --prune
git status --short --branch
gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,author
python3 scripts/garnet_v0_8_1_release_readiness.py --gate
python3 scripts/garnet_evidence_integrity_status.py
```

For v0.8.2, replace the v0.8.1 readiness script only after a new v0.8.2 gate exists and has itself been human-reviewed if it changes release policy.

---

## Product Direction To Preserve

The post-v0.8.1 product should converge around one hard problem:

> Can a team accept an agent-authored change without silently expanding authority or losing provenance?

That means:

- Lead with accept/refuse evidence, not syntax.
- Use dual-mode as supporting ergonomics, not the headline.
- Treat local LLMs as reviewers/advisors, not authorities.
- Make Studio the canonical proof cockpit.
- Keep every public claim connected to repo artifacts or reporter outputs.
- Prefer integrations and standards over rebuilding mature primitives.
- Only claim what trap tests, artifacts, and reviewers prove.

---

## Immediate After-S120 Sequence

1. Jon completes or explicitly defers S120 in Claude Code.
2. Record the result: tag/release URL or deferral reason.
3. Start Goal Mode B on a fresh branch for S121-S130 Truth Sync Gate.
4. In parallel, start Goal Mode H on the MacBook Air as a Headroom pilot only if it stays isolated from canonical release evidence.
5. Start Goal Mode C only after Truth Sync has identified the exact stale public claims.
6. Do not begin runtime/kernel work until S141-S150 independent trust status is decided or explicitly scheduled.
7. Treat S195-S200 as the v0.8.2 readiness gate and Jon-owned v0.8.2 cut boundary, not v0.9.
