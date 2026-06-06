# Garnet Post-v0.8.1 Goal Mode Prompt Pack

> **For Jon:** This is the fast-launch prompt pack for starting the next Garnet lanes across the MacBook Pro, MacBook Air, and Windows NUC. Paste the prompts below into Claude Code or Codex on the named machines. The broader system handoff lives at `F_Project_Management/GARNET_POST_0_8_1_SYSTEM_HANDOFF.md`.

**Created:** 2026-06-05

**Current post-cut truth observed locally:** `v0.8.1` exists on origin as an annotated tag object `0d3c217b472556522ef72ab52f299e77e0678723`, targeting commit `ca13bb2dc77109b6f621569b927839c7e7acc1bf`. GitHub Release exists: `https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.8.1`.

**Launch principle:** Start only the truth-sync and low-risk recon lanes immediately. Do not start runtime/kernel/security enforcement work while unattended until S121-S130 has cleaned the public and repo truth surfaces.

---

## 10-Minute Launch Order

1. **MacBook Pro, Claude Code Opus 4.8 Max Ultra Code:** start Prompt 1, the S121-S130 Truth Sync Gate. This is the priority lane.
2. **MacBook Air M5, Codex or Claude Code:** start Prompt 2, the read-only site/research drift audit plus optional Headroom pilot. This is safe to run while away because it should not mutate repo truth without opening its own branch.
3. **Windows NUC, Codex or Claude Code:** start Prompt 3, the post-tag Windows/WSL environment and smoke inventory. This should gather status, not implement runtime/OS boundary work yet.
4. **Optional MacBook Pro Codex side session:** start Prompt 4 only if you want a coordinator/watcher while Claude does Prompt 1. This side session must stay read-only unless explicitly told to review a PR.
5. Defer Prompts 5-9 until S121-S130 has either merged or produced a clear block report.

If you only have time to launch one session before leaving, launch **Prompt 1**.

---

## Prompt 1 - MacBook Pro - S121-S130 Truth Sync Gate

Paste into Claude Code Opus 4.8 Max Ultra Code on the MacBook Pro.

```text
ROLE: Claude Code on MacBook Pro - S121-S130 Truth Sync Gate for Garnet after the Jon-owned v0.8.1 cut.

SOURCE OF TRUTH:
- live GitHub repository Island-Dev-Crew/garnet
- local repo at /Users/IDC2.5/Desktop/Garnet
- tag v0.8.1 at ca13bb2dc77109b6f621569b927839c7e7acc1bf
- GitHub Release: https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.8.1
- F_Project_Management/GARNET_POST_0_8_1_SYSTEM_HANDOFF.md
- F_Project_Management/GARNET_v0_8_1_PLAN.md
- F_Project_Management/AGENT_COORDINATION_LEDGER.md
- scripts/garnet_v0_8_1_release_readiness.py --gate
- scripts/garnet_adoption_surface_status.py
- scripts/garnet_mit_readiness_status.py
- scripts/garnet_promo_video_status.py

MISSION:
Open the S121-S130 Truth Sync Gate. Bring repo-facing and public-facing truth into alignment with the now-tagged v0.8.1 release without touching kernel/runtime correctness work.

RECON FIRST:
1. cd /Users/IDC2.5/Desktop/Garnet
2. git fetch origin main --tags --prune
3. git status --short --branch
4. git rev-parse HEAD
5. git show-ref --tags v0.8.1
6. gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json name,tagName,url,isDraft,isPrerelease,publishedAt
7. gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,author
8. python3 scripts/garnet_v0_8_1_release_readiness.py --gate
9. python3 scripts/garnet_adoption_surface_status.py
10. python3 scripts/garnet_mit_readiness_status.py
11. python3 scripts/garnet_promo_video_status.py

REPORT BEFORE MUTATING:
Summarize:
- current main/head/tag/release state
- open PR state
- stale public truth surfaces found
- exact planned files to edit
- anything that would touch gates/CI/diff-caps/capability standards, which must be human-merge only

EXPECTED EDIT SCOPE:
- README.md
- docs/index.html
- docs/status.html
- F_Project_Management/AGENT_COORDINATION_LEDGER.md or a new post-cut project note if the ledger needs S120 closure
- F_Project_Management/GARNET_POST_0_8_1_SYSTEM_HANDOFF.md only if it needs post-cut factual correction
- Do not edit CI, dogfood gates, diff-caps thresholds, release-readiness gate logic, capability-manifest standards, or runtime/kernel behavior.

CLAIM RULES:
- Latest tagged release is v0.8.1.
- v0.8.1 is research-grade, not production/1.0.
- S114 found and fixed a HIGH issue, but do not claim independent red-team evidence.
- Linux seccomp is Linux-only OS-sandbox application.
- macOS/Windows OS-sandbox application remains deferred unless a deterministic trap proves otherwise.
- @bounded, memory, time, and @mailbox runtime ceilings remain declared-not-enforced unless proven otherwise.
- live LLM agent remains simulated/scripted unless provider-backed execution is proven.
- two LOW red-team findings remain open unless fixed by a later lane: caps-log tail and seal subject-digest.

BRANCH/PR:
Create a focused branch, for example:
  claude/s121-s130-truth-sync
Use one PR for this truth-sync gate unless the recon shows it should be split.

VERIFICATION:
Minimum docs/site truth-sync ladder:
  python3 scripts/garnet_adoption_surface_status.py
  python3 scripts/garnet_mit_readiness_status.py
  python3 scripts/garnet_promo_video_status.py
  python3 scripts/garnet_v0_8_1_release_readiness.py --gate
  python3 scripts/check-agent-contracts.py
  python3 scripts/test_check_agent_contracts.py
  git diff --check

If Rust behavior changes, stop and ask Jon. This gate should not need Rust behavior changes.

OUTPUT:
Create a PR or, if blocked, leave a precise block report. Do not start S131+ work in this session. Stop after the Truth Sync Gate PR/report.
```

---

## Prompt 2 - MacBook Air - Read-Only Drift Audit And Headroom Pilot

Paste into Codex or Claude Code on the MacBook Air.

```text
ROLE: MacBook Air M5 - read-only post-v0.8.1 drift audit and optional Headroom pilot for Garnet.

SOURCE OF TRUTH:
- live Island-Dev-Crew/garnet main
- v0.8.1 tag/release
- F_Project_Management/GARNET_POST_0_8_1_SYSTEM_HANDOFF.md
- F_Project_Management/GARNET_POST_0_8_1_GOAL_MODE_PROMPTS.md if present

MISSION:
Support the S121-S130 Truth Sync Gate without racing the MacBook Pro editing lane. Do a read-only audit first. If Headroom is evaluated, treat it as productivity infrastructure only, not release evidence.

RECON FIRST:
1. cd /Users/IDC2.5/Desktop/Garnet
2. git fetch origin main --tags --prune
3. git status --short --branch
4. git rev-parse HEAD
5. git show-ref --tags v0.8.1
6. gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json name,tagName,url,isDraft,isPrerelease,publishedAt
7. python3 scripts/garnet_adoption_surface_status.py
8. python3 scripts/garnet_mit_readiness_status.py
9. python3 scripts/garnet_promo_video_status.py

READ-ONLY AUDIT:
Find stale or risky public claims in:
- README.md
- docs/index.html
- docs/status.html
- docs/synthesis.html
- A_Research_Papers/README.md
- F_Project_Management/ docs that still say pre-cut, v0.5.0, 92.3%, or comparator-first headline

HEADROOM PILOT RULES:
- Only pilot Headroom outside the repo or on a separate branch.
- Do not let Headroom modify AGENTS.md, CLAUDE.md, CI, release gates, or repo procedural memory.
- Preserve raw logs before compression.
- Do not route secrets, signing credentials, GitHub tokens, or release credentials through Headroom.
- If Headroom is not already installed, record install requirements instead of forcing install.

OPTIONAL BRANCH:
If you produce a repo artifact, make it a report-only branch:
  codex/headroom-drift-audit
Suggested report file:
  F_Project_Management/GARNET_POST_0_8_1_DRIFT_AUDIT.md

DO NOT:
- Edit the same files the MacBook Pro Truth Sync lane is editing unless Jon explicitly asks.
- Start S131+ implementation.
- Claim Headroom improves gates without measured evidence.

OUTPUT:
Return a concise drift list with file paths, current stale claim, recommended replacement, and confidence. If a Headroom pilot ran, include commands, touched files, raw-log preservation, compression result, and adopt/defer/reject recommendation.
```

---

## Prompt 3 - Windows NUC - Post-Tag Windows/WSL Smoke Inventory

Paste into Codex or Claude Code on the Windows NUC.

```text
ROLE: Windows NUC - post-v0.8.1 Windows/Tauri/WSL smoke inventory lane for Garnet.

SOURCE OF TRUTH:
- Island-Dev-Crew/garnet main
- v0.8.1 tag/release
- Windows native environment
- WSL/WSLg and Tauri environment on this NUC
- Java/OpenJDK availability

MISSION:
Gather a post-tag Windows/WSL smoke inventory for the S121-S130 truth-sync work and future v0.8.2 planning. This is not a runtime/OS enforcement implementation lane yet.

RECON FIRST:
1. cd into the Garnet repo on the Windows NUC
2. git fetch origin main --tags --prune
3. git status --short --branch
4. git rev-parse HEAD
5. git show-ref --tags v0.8.1
6. gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json name,tagName,url,isDraft,isPrerelease,publishedAt
7. rustc --version
8. cargo --version
9. node --version
10. npm --version
11. java -version
12. wsl --status
13. wsl --list --verbose

STATUS SCRIPTS:
Run only status/reporting commands first:
  python scripts/garnet_v0_8_1_release_readiness.py --gate
  python scripts/garnet_mit_readiness_status.py
  python scripts/garnet_windows_linux_studio_status.py

SMOKE RULE:
If you run any smoke script that records artifacts, do it only on a branch such as:
  codex/windows-post-081-smoke-inventory
Do not overwrite existing proof bundles. Use timestamped output only.

DO NOT:
- Start S151-S170 runtime/OS enforcement.
- Claim WSL is Linux seccomp or OS-sandbox enforcement.
- Claim signed MSI, winget, Windows ARM64, clean-machine install, or production readiness unless the exact evidence exists.
- Touch CI, gates, diff-caps thresholds, or capability standards.

OUTPUT:
Produce an inventory report with:
- Windows native status
- Tauri status
- WSL/WSLg status
- Java/OpenJDK status
- scripts run and exit codes
- gaps relevant to v0.8.2 readiness
- recommended next Windows lane after S121-S130 merges
```

---

## Prompt 4 - Optional MacBook Pro Codex Coordinator

Paste into a separate Codex session only if you want a read-only watcher while Prompt 1 runs.

```text
ROLE: Codex on MacBook Pro - read-only coordinator/watcher for Garnet post-v0.8.1 lanes.

MISSION:
Monitor live repo/GitHub state while Claude Code runs the S121-S130 Truth Sync Gate. Do not edit files unless Jon explicitly redirects this session.

TASKS:
1. Verify v0.8.1 tag and release state.
2. Watch open PRs for S121-S130.
3. If a PR appears, review it from a code-review stance: findings first, file/line references, focus on overclaims, stale truth, missing deferrals, gate/CI modifications, and docs drift.
4. Do not merge. Do not tag. Do not alter gates.

OUTPUT:
Short status updates only. If the Truth Sync PR looks clean, say what was checked and what residual risk remains.
```

---

## Deferred Prompt 5 - S131-S140 Public Positioning Gate

Run only after S121-S130 is merged or clearly blocked.

```text
ROLE: Garnet S131-S140 Public Positioning Gate.

MISSION:
Move Garnet's public identity from comparator-first language marketing to evidence-native agent-code acceptance.

START CONDITIONS:
- S121-S130 truth sync is merged or explicitly blocked.
- Current README/site/status no longer contradict v0.8.1.

GOAL:
Create a focused PR that changes the public story to:
  Garnet is the evidence-native accept/refuse layer for agent-authored code.

DO:
- Use real Studio/trust artifacts.
- Show proposal -> capability diff -> trap run -> accept/refuse -> seal/provenance.
- Keep dual-mode as supporting ergonomics.
- Preserve research-grade boundaries.

DO NOT:
- Create a landing page with abstract claims only.
- Claim production, 1.0, independent red-team, live LLM authority, or full OS sandbox parity.
```

---

## Deferred Prompt 6 - S141-S150 Independent Trust Gate

Run only when Jon has selected or authorized an independent attacker/reviewer.

```text
ROLE: Garnet S141-S150 Independent Trust Gate coordinator.

MISSION:
Package and support an independent re-run of S114-style red-team work without self-grading it.

RULES:
- Independent attacker/reviewer must be named or provenance-described.
- Codex/Claude may package evidence and reproduce commands, but must not be the sole attacker and grader.
- If independence cannot be achieved, record the gap and stop.

OUTPUT:
Independent report or block note. Do not bless independence without provenance.
```

---

## Deferred Prompt 7 - S151-S178 Runtime, OS, And Seal Integrity Gates

Run only after Truth Sync and Independent Trust status are clear.

```text
ROLE: Garnet S151-S178 enforcement/integrity lane.

MISSION:
Advance only claims that can be backed by deterministic trap tests.

PRIORITY ORDER:
1. Seal integrity LOWs: caps-log tail and seal subject-digest.
2. @bounded deterministic fuel trap.
3. memory/time/mailbox ceilings if deterministic trap design is ready.
4. macOS/Windows OS sandbox application only if trap proof is possible.

RULE:
Write failing tests first where implementation changes are required. "Enforced" is forbidden unless the trap test proves it.

DO NOT:
- Broaden capability thresholds without human merge.
- Claim OS sandbox parity from WSL portability evidence.
- Change release gate logic without human review.
```

---

## Deferred Prompt 8 - S179-S186 Local LLM Advisory Gate

Run after Truth Sync has landed and Headroom/local context lessons are known.

```text
ROLE: Garnet S179-S186 local LLM advisory lane.

MISSION:
Wire or evaluate local/provider-neutral LLM advisory as a non-authoritative review aid.

RULES:
- Deterministic context pack first.
- Source omitted by default where required.
- Model output tagged non-deterministic.
- LLM may suggest, summarize, or critique.
- LLM must not authorize acceptance.
- diff-caps, garnet check, tests, dogfood, and seal remain authoritative.

OUTPUT:
Measured advisory value or honest defer. No provider-backed claim without real provider execution.
```

---

## Deferred Prompt 9 - S187-S200 Flagship Workbench And v0.8.2 Readiness

Run only after S121-S186 gates have produced enough current evidence.

```text
ROLE: Garnet S187-S200 flagship and v0.8.2 readiness lane.

MISSION:
Build or assemble the Garnet Trust Review Workbench proof and then aggregate v0.8.2 readiness.

FLAGSHIP WORKFLOW:
1. Ingest PR/package/MCP/generated-code candidate.
2. Derive capability surface.
3. Run diff-caps.
4. Run deterministic trap checks.
5. Emit accept/refuse dossier.
6. Optionally add local LLM advisory summary.
7. Present in Studio.

V0.8.2 READINESS:
S195-S200 is the v0.8.2 readiness gate, not v0.9.
Jon owns the v0.8.2 cut/tag.

DO NOT:
- Cut or tag autonomously.
- Claim production or 1.0.
- Hide open deferrals.
```

---

## Unattended Safety Rules

- If a lane sees unexpected local changes, stop and report before editing.
- If `git fetch` fails, stop and report; do not treat local state as current.
- If a command touches credentials, signing, settings, permissions, or destructive OS controls, stop and wait for Jon.
- If a PR would modify CI, gates, dogfood, diff-caps thresholds, capability standards, or release policy, mark it human-merge only.
- If two machines want to edit the same file, only the Truth Sync primary lane edits; the other lane writes a review report.
- Every lane must end with exact commands run, exit codes, files changed, PR URL if created, and honest residual gaps.
