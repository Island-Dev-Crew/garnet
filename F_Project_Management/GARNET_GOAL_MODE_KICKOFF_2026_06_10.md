# Garnet Goal-Mode Kickoff Pack - 2026-06-10

Committed: 2026-06-11 by the S131-S134 consolidation PR as the episodic
operating sheet of the 2026-06-10 fleet run (referenced by the command center
and the kickoff addendum). Fleet-run outcome and transport lessons:
`GARNET_S131_S134_SOURCE_TRUTH_CONSOLIDATION.md`.
Status: paste-ready operating sheet
Purpose: launch the post-v0.8.1 fleet consolidation, source-truth PR, and
W-REBUILD preparation without drifting from Garnet dogfood/readiness discipline.

Current observed repo state from MacBook Pro:

- branch: `main`
- `origin/main`: `366e69f28812732df2f18fa6b80e1581dd7ac2d9`
- PR `#380 docs: add W-REBUILD foundation-rebuild workstream pack` is merged.
- W-REBUILD pack is present on `main` in `F_Project_Management/W_REBUILD/`.

## Operating Order

Do not start RB implementation work yet. The immediate goal is fleet truth,
then a docs/report-only consolidation PR, then W-REBUILD deploy-gate
verification.

1. MacBook Pro Codex writes or refreshes the MacBook Pro Codex fleet report.
2. Windows NUC Claude writes the Windows/Linux/Tauri fleet report.
3. Windows NUC Codex independently verifies Windows/Linux/Tauri facts.
4. Windows Surface runs the original-Tauri-lane provenance harvest read-only.
5. MacBook Air Claude writes the independent audit/public-story report.
6. MacBook Air Codex verifies public truth drift and release/site facts.
7. MacBook Pro Claude ingests all fleet reports and assembles the
   source-truth consolidation PR.
8. Only after the consolidation PR is merged, run the W-REBUILD deploy gate.
9. If W-REBUILD deploy gate passes, execute W-REBUILD P0, then RB slices one PR
   at a time.

Minimum viable kickoff if time is tight:

1. MacBook Pro Codex.
2. Windows NUC Claude.
3. Windows NUC Codex.
4. Windows Surface original-Tauri provenance harvest.
5. MacBook Pro Claude consolidation.

## Parallelization Rule

Run fleet reports in parallel. They are intentionally read-only or
report-first, so the MacBook Pro, Windows NUC, MacBook Air, and Surface can all
work at once.

Do not run these in parallel:

- MacBook Pro Claude source-truth consolidation before the required fleet
  reports exist.
- W-REBUILD deploy-gate verification before the source-truth consolidation PR
  is merged.
- RB implementation slices with each other. RB work is one PR per slice, in
  order.

The Windows Surface is not optional for truth. It is paused as a future builder
because it is slow, but it remains required as the original Tauri-lane
provenance source unless Jon explicitly declares it unavailable. If it cannot
run, the consolidation PR must say "Surface provenance unavailable/deferred,"
not silently omit it.

## Report Transport Rule

Each fleet lane writes its report on a dedicated branch and opens no PR.

Branch naming:

```text
fleet/2026-06-10-<machine>-<agent>
```

Examples:

```text
fleet/2026-06-10-macbook-pro-codex
fleet/2026-06-10-windows-nuc-claude-fable
fleet/2026-06-10-surface-original-tauri
```

The lane may push that branch for MacBook Pro Claude to fetch. MacBook Pro
Claude is the only lane that assembles the single S131-S134 source-truth
consolidation PR. This keeps the "one PR" promise true and prevents six
competing docs PRs from racing in `F_Project_Management/FLEET_REPORTS/`.

## Local Scope Rule

Each machine should write a "fleet PR rundown" of its own work. The report must
search beyond the currently open checkout and include every plausible Garnet
location where that system developed, tested, built, or stored evidence.

At minimum, each report should cover:

- all local Garnet checkouts and forks;
- all local branches and untracked files in those checkouts;
- `Desktop`, `Documents`, `Downloads`, temp folders, and tool-specific
  workspaces that may contain Garnet artifacts;
- proof bundles, screenshots, screen recordings, tarballs, VSIX files, package
  outputs, Tauri outputs, WSL/UTM outputs, Claude/Codex handoffs, and release
  candidate files;
- exact verdict for each artifact: commit, archive, ignore, duplicate, unsafe,
  or needs Jon.

Timebox the sweep: use targeted roots and patterns for up to 90 minutes per
machine, then write an honest coverage statement such as "searched these roots
with these patterns; not an exhaustive disk audit." This is sensible, not
excess. It is the one-time scatter-management pass before the foundations move.

## Later Tooling Rule

Fable's `.claude/` repo-config idea is useful, especially permission deny rules
for tags/releases and repo-discovered skills for verification. Do not add it
during fleet consolidation. It should be a separate Jon-approved
`FLEET-TOOLING` PR after the S131-S134 source-truth consolidation lands.

## Plugin Attachment Map

Attach only what the lane needs. Plugins are power tools; Garnet gates remain
the authority.

### Codex Global Attachments

Use these on most Garnet lanes:

- GitHub: repo/PR/CI truth, PR inspection, PR creation when authorized.
- Browser or Chrome: site verification, local docs render, Studio UI proof.
- Computer Use: GUI proof capture only; never settings, permissions, secrets,
  deletes, force pushes, or access-changing controls.
- Documents: durable handoff/report writing when needed.
- Spreadsheets: readiness matrices, fleet tables, evidence grids.
- Presentations: presentation deck only, not source-of-truth edits.
- Product Design: public site and README/front-door critique.
- Notion or Linear: optional external project capture only; repo docs remain
  source of truth.

Avoid by default:

- Full ECC hook installs.
- Any plugin that changes repository settings, secrets, permissions, release
  tags, or published release assets.

### Claude Code Global Attachments

Attach Claude-side tools by function:

- GitHub/repo tools for PR, CI, and live release state.
- Engineering architecture for W-REBUILD design review.
- Code review for every implementation PR.
- Security review for trust, release, sandbox, capability, and hook risk.
- Rust build/test resolver for cargo, clippy, parser, checker, interp, stdlib.
- Browser/Chrome/computer-use for Studio, website, and evidence capture.
- Research/synthesis for public positioning and paper alignment.
- Documents/presentations for handoff, launch packet, and deck prep.

Do not attach or enable:

- Repo hooks that rewrite workflow behavior.
- Secret-management helpers unless Jon is actively doing a secret operation.
- Release automation unless Jon explicitly says to cut or publish.

## Universal Preamble

Paste this before any lane-specific prompt when the tool accepts a preamble.

```text
You are working on Garnet after signed v0.8.1 and S130.

ECC-Prime and Fable/Opus/Codex workflow help with planning, review, research,
security critique, and long-running coordination. They are not a source of
truth. Garnet's AGENTS.md files, current repo state, dogfood/readiness gates,
deterministic proof artifacts, and Jon's release boundaries are authoritative.

Do recon first:
- read /AGENTS.md;
- read F_Project_Management/AGENTS.md;
- read the closest child AGENTS.md for any subsystem you touch;
- git fetch origin main --tags --prune;
- record git status, HEAD, origin/main, open PRs, and current v0.8.1 release
  truth;
- never assume another lane's result without checking repo/GitHub truth.

Hard stops:
- never push a tag;
- never cut or re-cut a release;
- never install ECC or enable ECC hooks inside Garnet unless Jon explicitly asks
  in the current session;
- never edit gates, CI, diff-caps thresholds, capability standards, or release
  policy without Jon;
- never claim production or v1.0;
- never claim "enforced" unless a deterministic trap proves it;
- keep S114 self-verified status labeled until an actually independent
  adversarial re-verification is complete.

For PRs:
- one coherent docs/report slice or one implementation slice per PR;
- focused tests first;
- run the appropriate Garnet verification ladder;
- run dogfood-readiness when the lane is readiness-sensitive;
- record exact commands and pass/fail evidence;
- stop and report after each PR or blocked gate.
```

## Goal Mode 1 - MacBook Pro Codex

Recommended Codex plugins: GitHub, Browser, Chrome, Computer Use, Documents,
Spreadsheets, Product Design.

```text
ROLE: Codex on MacBook Pro - Garnet source-truth and dogfood verifier.

MISSION:
Independently verify the post-v0.8.1 command center, W-REBUILD pack, and
Fable/Claude outputs against live repo truth, AGENTS contracts, and Garnet
readiness gates. Treat ECC/Fable as workflow input only.

SOURCE OF TRUTH:
- /Users/IDC2.5/Desktop/Garnet
- /AGENTS.md
- F_Project_Management/AGENTS.md
- F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md
- F_Project_Management/GARNET_100X_FABLE_INTAKE_2026_06_10.md
- F_Project_Management/W_REBUILD/
- live GitHub repo Island-Dev-Crew/garnet

RECON:
1. cd /Users/IDC2.5/Desktop/Garnet
2. git fetch origin main --tags --prune
3. git status --short --branch
4. git rev-parse HEAD origin/main
5. gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,author,updatedAt,url
6. gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json tagName,publishedAt,url,targetCommitish,assets
7. python3 scripts/garnet_readiness_status.py --format json
8. python3 scripts/garnet_mit_readiness_status.py --format json

OUTPUT:
Create or update:
  F_Project_Management/FLEET_REPORTS/2026-06-10_macbook-pro_codex.md

VERIFY:
Run:
1. python3 scripts/check-agent-contracts.py
2. python3 scripts/test_check_agent_contracts.py
3. git diff --check

REPORT:
Findings first, ordered by severity. Explicitly flag release/tag overclaims,
public-truth drift, self-authored red-team independence issues, and any attempt
to replace Garnet gates with ECC/Fable authority.

STOP:
Do not implement W-REBUILD slices. Return the report for MacBook Pro Claude
consolidation.
```

## Goal Mode 2 - MacBook Pro Claude Source-Truth Lead

Recommended Claude attachments: GitHub, engineering architecture, code review,
security review, Rust build resolver, documents, browser/chrome, computer use.

```text
ROLE: Claude Code Fable 5 Ultracode on MacBook Pro - Garnet S131-S134 source-truth lead.

MISSION:
Pair ECC-style workflow discipline with Garnet dogfood/readiness gates to
assemble the S131-S134 source-truth consolidation PR. Do not start W-REBUILD
implementation work in this goal.

SOURCE OF TRUTH:
- /Users/IDC2.5/Desktop/Garnet
- /AGENTS.md
- F_Project_Management/AGENTS.md
- F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md
- F_Project_Management/GARNET_100X_FABLE_INTAKE_2026_06_10.md
- F_Project_Management/GARNET_GOAL_MODE_KICKOFF_2026_06_10.md
- F_Project_Management/W_REBUILD/
- F_Project_Management/FLEET_REPORTS/
- live GitHub repo Island-Dev-Crew/garnet

RECON:
1. cd /Users/IDC2.5/Desktop/Garnet
2. git fetch origin main --tags --prune
3. git status --short --branch
4. git rev-parse HEAD origin/main
5. git log --oneline -8
6. gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,author,updatedAt,url
7. gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json tagName,name,isDraft,isPrerelease,publishedAt,url,targetCommitish,assets
8. python3 scripts/garnet_readiness_status.py --format json
9. python3 scripts/garnet_mit_readiness_status.py --format json

DEPLOY GATE FOR THIS GOAL:
Fleet reports must exist for MacBook Pro Codex, Windows NUC Claude, Windows NUC
Codex, and Surface original-Tauri provenance, OR Jon must have declared Surface
unavailable/deferred in this session. Record which condition held in the PR
body. If the required reports do not exist, create or refresh this machine's
Claude fleet report and STOP.

PRIMARY TASK:
Create the docs/report-only source-truth consolidation PR. It should consolidate:
- current repo/release truth;
- confirmed README/site/current-state drift;
- fleet artifacts and machine-specific evidence;
- W-REBUILD pack status;
- the S131-S200 runway;
- the correct ECC-plus-dogfood framing.

DO NOT:
- install ECC;
- enable hooks;
- replace README.md;
- implement xtask truth;
- rename crates;
- restructure root directories;
- change CI/gates;
- touch release assets;
- push tags;
- launch publicly.

VERIFY:
For docs/report-only work, run:
1. python3 scripts/check-agent-contracts.py
2. python3 scripts/test_check_agent_contracts.py
3. git diff --check

PR BODY MUST SAY:
- report-only;
- ECC/Fable is advisory and Garnet gates are authoritative;
- no release/tag/asset/gate/CI mutation occurred;
- W-REBUILD implementation is blocked until fleet consolidation is merged and
  the W-REBUILD deploy gate passes.

STOP:
Stop after PR creation or merge. Do not proceed into implementation slices in
the same goal unless Jon explicitly says continue.
```

## Goal Mode 3 - MacBook Pro Claude W-REBUILD Lead

Run only after the S131-S134 source-truth consolidation PR is merged.

Recommended Claude attachments: GitHub, engineering architecture, code review,
security review, Rust build resolver, documents, browser/chrome for evidence.

```text
ROLE: Claude Code Fable 5 Ultracode on MacBook Pro - Garnet W-REBUILD lead.

MISSION:
Execute W-REBUILD exactly as specified in
F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md, after deploy-gate verification.

SOURCE OF TRUTH:
- /AGENTS.md and closest child AGENTS.md for every subsystem touched
- F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md
- F_Project_Management/GARNET_100X_FABLE_INTAKE_2026_06_10.md
- F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md
- F_Project_Management/W_REBUILD/README_PROPOSED.md
- F_Project_Management/W_REBUILD/GARNET_TRUTH_DRIFT_PUNCHLIST.md
- F_Project_Management/FLEET_REPORTS/
- live repo + GitHub truth

DEPLOY GATE:
STOP unless all are true:
1. core fleet reports exist;
2. S131-S134 source-truth consolidation PR is merged;
3. clean working tree on current origin/main.

ORDER:
P0 docs PR if still needed -> RB-0a -> RB-0b -> RB-0c -> RB-0d -> RB-1 ->
RB-2 -> RB-3 -> RB-4a -> RB-4b -> RB-5 -> RB-6 memo -> RB-7.

CONSTRAINTS:
RB-1 through RB-5 must change zero language semantics. Any semantic delta is
STOP+report, never patch around it. RB-6 is memo only. No backend code lands.
No release/tag/gate/CI/hook mutation without Jon.

VERIFICATION LADDER:
focused tests first -> cargo test --workspace --no-fail-fast with 0 failed ->
cargo clippy --workspace --all-targets -- -D warnings -> cargo fmt --all
-- --check -> dogfood-readiness fused 5/5 -> python3
scripts/check-agent-contracts.py -> PR to Navigata1 -> CI green -> merge
IslandDevCrew -> switch back.

STOP POINTS:
After RB-0 band, after RB-3, after RB-5, at RB-6, and after final workstream
report.
```

## Goal Mode 4 - Windows NUC Claude

Recommended Claude attachments: GitHub, code review, Rust build resolver,
browser/chrome/computer-use for Tauri evidence, documents.

```text
ROLE: Claude Code Fable 5 Ultracode on Windows NUC - Garnet Windows/Linux/Tauri evidence lane.

MISSION:
Produce the Windows NUC fleet report and prepare the Windows/Linux/Tauri
evidence plan for S131-S200. Use ECC-style workflow for planning and review,
but keep Garnet dogfood/readiness as acceptance authority.

RECON:
1. Locate the active Garnet checkout.
2. git fetch origin main --tags --prune
3. git status --short --branch
4. git rev-parse HEAD origin/main
5. gh auth status
6. gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,author,updatedAt,url
7. wsl --status
8. wsl -l -v
9. java --version
10. rustc --version
11. cargo --version
12. node --version

OUTPUT:
Create or update:
  F_Project_Management/FLEET_REPORTS/2026-06-10_windows-nuc_claude-fable.md

REPORT:
Document Windows repo path, WSL/Debian state, Tauri state, Java/OpenJDK state,
local proof artifacts, smoke commands, package artifacts, and what remains
unproven.

NEXT-STEP PLAN:
Draft the Windows/Linux package and smoke plan for S166-S178:
- Windows installer/CLI smoke;
- Linux desktop GUI proof not confused with WSL portability;
- VSIX naming/publication cleanup;
- winget/scoop feasibility;
- clean-machine evidence requirements.

DO NOT:
Push tags, create releases, install ECC hooks, change gates/CI, claim Linux
seccomp or OS-sandbox enforcement from WSL, or claim production/v1.0.

STOP:
After the fleet report and plan, stop for MacBook Pro consolidation.
```

## Goal Mode 5 - Windows NUC Codex

Recommended Codex plugins: GitHub, Browser/Chrome, Computer Use for GUI proof,
Documents, Spreadsheets.

```text
ROLE: Codex on Windows NUC - Garnet cross-OS verifier.

MISSION:
Verify Windows NUC state independently from Claude/Fable. Produce a concise
fleet report and identify which Windows/Linux/Tauri claims are reproducible now.

RECON:
1. Locate the active Garnet checkout.
2. Read AGENTS.md and closest relevant child AGENTS.md files.
3. git fetch origin main --tags --prune
4. git status --short --branch
5. git rev-parse HEAD origin/main
6. gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json tagName,publishedAt,url,targetCommitish,assets
7. wsl --status
8. wsl -l -v
9. rustc --version
10. cargo --version
11. node --version

OUTPUT:
Create or update:
  F_Project_Management/FLEET_REPORTS/2026-06-10_windows-nuc_codex.md

VERIFY WHERE APPROPRIATE:
- python3 scripts/check-agent-contracts.py
- python3 scripts/garnet_readiness_status.py --format json
- python3 scripts/garnet_mit_readiness_status.py --format json
- Windows/WSL/Tauri smoke scripts already supported by the repo

REPORT:
Separate byte-identical committed truth, machine-local evidence, Windows proof,
WSL portability only, clean-Linux proof, and not-proven items.

STOP:
Do not implement packaging changes. Return facts and recommended next commands
for the orchestration lane.
```

## Goal Mode 6 - MacBook Air Claude

Recommended Claude attachments: GitHub, research/synthesis, product/design or
public-story critique, documents, presentations, browser/chrome.

```text
ROLE: Claude Code Fable 5 Ultracode on MacBook Air - Garnet independent audit and public-story lane.

MISSION:
Act as an independent read-only audit/research lane for Garnet's S131-S200
runway. Focus on presentation clarity, README/site critique, research alignment,
and launch readiness. Do not author correctness-critical implementation.

RECON:
1. Locate or clone the Garnet checkout.
2. git fetch origin main --tags --prune
3. git status --short --branch
4. gh repo view Island-Dev-Crew/garnet --json name,owner,url,stargazerCount,forkCount,updatedAt
5. gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json tagName,publishedAt,url,targetCommitish,assets
6. Read README.md, CURRENT_STATE.md, docs/index.html, docs/status.html,
   F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md, and
   F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md.

OUTPUT:
Create or update:
  F_Project_Management/FLEET_REPORTS/2026-06-10_macbook-air_claude-fable.md

AUDIT QUESTIONS:
- Can a stranger understand Garnet in 30 seconds?
- Is the README still an internal ledger?
- Which public claims are stale or overcomplicated?
- What needs to be true before launch?
- What screenshots/diagrams would make the presentation clearer?
- Which Fable/W-REBUILD recommendations are high-impact but unsafe to rush?

DO NOT:
Push tags, merge PRs, install ECC hooks, make release decisions, or change gate
logic.
```

## Goal Mode 7 - MacBook Air Codex

Recommended Codex plugins: GitHub, Browser/Chrome, Product Design, Documents,
Presentations.

```text
ROLE: Codex on MacBook Air - Garnet independent facts and screenshot verifier.

MISSION:
Produce an independent MacBook Air fleet report and verify the public-site,
README, and release-truth claims that will be used in presentation or S131-S140.

RECON:
1. Locate or clone the Garnet checkout.
2. git fetch origin main --tags --prune
3. git status --short --branch
4. gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json tagName,publishedAt,url,targetCommitish,assets
5. python3 scripts/garnet_readiness_status.py --format json
6. python3 scripts/garnet_mit_readiness_status.py --format json
7. rg -n "post-v0.5.0|24 stdlib|24 registry|24 bridged|v0.5.0 is research-grade|S1 LSP|0.7.0-lsp|monthly is the floor" README.md docs CURRENT_STATE.md

OUTPUT:
Create or update:
  F_Project_Management/FLEET_REPORTS/2026-06-10_macbook-air_codex.md

REPORT:
- exact stale strings found;
- exact files needing first-pass cleanup;
- screenshots worth preserving;
- claims safe for presentation;
- claims that need rewording.

STOP:
Report only unless Jon explicitly asks for a docs PR.
```

## Goal Mode 8 - Windows Surface Original Tauri Provenance Harvest

Recommended attachments: GitHub read-only, Documents. Avoid browser/computer
use unless needed for file discovery. Keep this slow machine read-only.

```text
ROLE: Garnet Windows Surface original-Tauri provenance harvest lane.

MISSION:
Inventory Garnet files/artifacts on the Surface because it was the original
Tauri lane and worker. The Surface is paused as a future builder, but its
historical evidence is required for fleet truth.

DO:
- locate every Garnet repo/checkout/fork on the machine;
- inspect Desktop, Documents, Downloads, temp folders, WSL home paths, and any
  Claude/Codex working directories for Garnet, Tauri, Studio, VSIX, package,
  screenshot, proof, or release artifacts;
- record local branches, untracked files, proof bundles, downloads,
  screenshots, release artifacts, handoff docs, Tauri build outputs, and WSL
  outputs;
- record paths, timestamps, hashes, and safety notes;
- identify what is already present on origin/main or the Windows NUC;
- assign each artifact a verdict: commit, archive, ignore, duplicate, unsafe,
  or needs Jon.

DO NOT:
- modify repo files;
- push branches;
- install ECC;
- delete anything;
- print secrets.

OUTPUT:
Create one report:
  F_Project_Management/FLEET_REPORTS/2026-06-10_surface_original-tauri-provenance.md

STOP:
No mutation. No deletion. No push.
```

## Plugin Use By Workstream

| Workstream | Codex plugins | Claude attachments | Notes |
|---|---|---|---|
| Fleet/source-truth | GitHub, Documents, Spreadsheets | GitHub, docs, code review | Report-only. |
| W-REBUILD | GitHub, Documents | engineering architecture, code review, Rust build resolver, security review | No start before deploy gate. |
| W-TRUST | GitHub, Documents, Spreadsheets | security review, research, code review | Independent re-verification must be genuinely independent. |
| W-SHIP | GitHub, Browser/Chrome, Computer Use, Spreadsheets | GitHub, browser/computer-use, Rust/Tauri build resolver | Computer-use only for evidence capture and UI/package proof. |
| W-PLAY | Browser/Chrome, Product Design, Computer Use | browser/chrome, product design, engineering architecture | Verify local browser proof and screenshots. |
| W-LAUNCH | Presentations, Documents, Product Design, Browser | presentations, research, docs, product/public-story | Drafts only. Posting and release moments stay Jon-owned. |

## Final Launch Command For Today

Use this first on MacBook Pro Claude only after at least the MacBook Pro Codex,
Windows NUC Claude, Windows NUC Codex, and Surface original-Tauri provenance
reports are available, or after Jon explicitly marks Surface unavailable:

```text
Continuing Garnet after signed v0.8.1. Read
F_Project_Management/GARNET_GOAL_MODE_KICKOFF_2026_06_10.md,
F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md, and
F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md. Verify current repo/GitHub
truth, ingest the fleet reports, then create the S131-S134 docs/report-only
source-truth consolidation PR. Do not start W-REBUILD implementation until that
PR is merged and the W-REBUILD deploy gate passes.
```
