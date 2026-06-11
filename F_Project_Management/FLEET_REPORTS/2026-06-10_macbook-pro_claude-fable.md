# MacBook Pro Claude (Fable 5) Fleet Report — 2026-06-10 run, authored 2026-06-11

This is the consolidation lane's own machine report, created under the GM2 stop
procedure: the S131–S134 consolidation deploy gate was checked and found unmet
(two Windows NUC core reports absent from every prescribed channel), so this
lane creates/refreshes its own fleet report and stops. Filename keeps the
2026-06-10 fleet-run date per the command-center contract; the body was
authored 2026-06-11.

## Report Header

- Machine: MacBook Pro (Mac17,8), Apple M5 Pro, 48 GB memory; serial redacted
- Agent/model: Claude Code / `claude-fable-5[1m]` (ultracode; 4 parallel
  read-only sweep subagents + first-party verification)
- Date/time: authored 2026-06-11 ~09:30 CDT; fleet-run date 2026-06-10
- OS: macOS 26.5 (25F71), arm64
- Repo path: `/Users/IDC2.5/Desktop/Garnet`
- Active user/account: `gh` active account `Navigata1` (keyring, https);
  `IslandDevCrew` authenticated but inactive
- Network state: GitHub fetch + `gh` API calls succeeded against both remotes
- Report scope: consolidation-lane deploy-gate inventory + bounded read-only
  local sweep of this machine (Desktop, Documents, Downloads, /tmp, clawd
  tool workspaces) + delivered-fleet-report ingestion

## Repo Truth

- Current branch: `main...origin/main` (in sync)
- HEAD: `366e69f28812732df2f18fa6b80e1581dd7ac2d9`
- origin/main: `366e69f28812732df2f18fa6b80e1581dd7ac2d9`
  (`docs: add W-REBUILD foundation-rebuild workstream pack (#380)`)
- Remotes: `origin` = `Island-Dev-Crew/garnet`; `fork` = `Navigata1/garnet`
- v0.8.1 tag target: local annotated tag object
  `4a6442d439e636bdfcfc2f2ee2c8c2f9b2c8e1c7` peeling to
  `8107c01a59a3f84925e9a4880a29a07a32b408eb` — byte-identical to origin.
  All four `v*` tags (v0.4.2, v0.5.0, v0.8.0, v0.8.1) MATCH origin. Two
  local-only lightweight tags exist (`parked/s15-promo-docs`,
  `parked/s5-fmt-wip`) — push-state drift only, no hash mismatch.
- Release state: `v0.8.1` published 2026-06-07T07:55:45Z, not draft, not
  prerelease, 9 assets (`.deb`, `.rpm`, two darwin tarballs, CycloneDX SBOM,
  `SHA256SUMS`, `SHA256SUMS.asc`, plus two stale `garnet-0.7.0-lsp-mvp-*.vsix`).
  Per the MBP Codex report's P0: describe as signed checksums/assets, never as
  a signed Git tag.
- Open PRs: `#381` only (MacBook Air Claude fleet report; see Drift section)
- Local untracked files (6):
  `F_Project_Management/FLEET_REPORTS/`,
  `GARNET_100X_FABLE_INTAKE_2026_06_10.md`,
  `GARNET_ECC_PRIME_ASSIMILATION.md`,
  `GARNET_ECC_PRIME_OVERLAY_RECOMMENDATION.md`,
  `GARNET_GOAL_MODE_KICKOFF_2026_06_10.md`,
  `GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md` (all under
  `F_Project_Management/`)
- Local modified files: none

## Toolchain Truth

- Rust: `rustc 1.95.0` / `cargo 1.95.0`, active toolchain
  `stable-aarch64-apple-darwin` (1.93.0 also installed), resolved via rustup
  shim on PATH
- Node/npm: Node `v26.0.0`
- Python: `Python 3.14.5`
- GitHub CLI: `gh version 2.92.0`
- GPG: `gpg (GnuPG) 2.5.19`
- Java: not installed (no Java Runtime found)
- Tauri/Xcode notes: Xcode 26.5 (17F42) present; no UTM/WSL on this lane's
  sweep scope (the Debian UTM image in Downloads is Linux test infrastructure,
  inventoried below)

## Garnet Verification Snapshot

- Agent contracts: pass — `agent-contracts: ok (21 contracts)` (exit 0)
- Contract checker self-test: pass — 5 tests, `OK` (exit 0)
- Diff check: pass — `git diff --check` clean (exit 0)
- Tracked readiness: 87/87 slices, 100.0%
  (`scripts/garnet_readiness_status.py`)
- MIT/productization readiness: `active-partial`, 92.8%
  (`scripts/garnet_mit_readiness_status.py`)
- Cargo fmt/test/clippy: not run — this lane is docs/report-only and touched
  no Rust
- Dogfood-readiness: not run — no readiness-affecting change in this lane

## Fleet Inventory (the deploy-gate fact this report exists to record)

Required core-four reports and their delivery status as verified 2026-06-11
from this machine (channels: local `FLEET_REPORTS/`, all `origin` branches,
all 368 `fork` branches, open/closed PRs, Downloads/Telegram transfer
folders, Spotlight content search):

| Report | Status | Channel |
|---|---|---|
| `2026-06-10_macbook-pro_claude-fable.md` | present (this file) | local, consolidation lane |
| `2026-06-10_macbook-pro_codex.md` | present | local untracked, this machine |
| `2026-06-10_windows-nuc_claude-fable.md` | ABSENT at authoring → **delivered ~11:45** | fork branch `fleet/2026-06-10-windows-nuc-claude-fable` (see Resolution below) |
| `2026-06-10_windows-nuc_codex.md` | ABSENT at authoring → **delivered ~11:45** | fork branch `fleet/2026-06-10-windows-nuc-codex` (see Resolution below) |

Additional lanes, as delivered:

| Report | Status | Channel |
|---|---|---|
| `2026-06-10_macbook-air_claude-fable.md` | delivered | open PR #381 (branch `codex/fleet-report-macbook-air-claude` on fork) |
| `2026-06-10_macbook-air_codex.md` | not delivered (recommended, not required) | — |
| `2026-06-10_surface_original-tauri-provenance.md` | delivered (single combined report) | fork branch `fleet/2026-06-10-surface-original-tauri-provenance`, no PR (transport-conformant) |

Corpus-search note: a miss in this sweep is a delivery miss on the channels
searched, not proof the NUC runs never happened. The Surface report
explicitly states the NUC was not accessible from its lane; the Air report's
own next-steps list also records the NUC reports as still outstanding at its
authoring time. No explicit Jon note deferring the NUC lanes exists in this
session's working set, and per the standing rule such a note is never
inferred.

Gate verdict at authoring (~09:30): **S131–S134 consolidation deploy gate
UNMET** (2 of 4 core reports absent). Per GM2: create/refresh this machine's
Claude report and STOP. No consolidation PR was created at that point.

**Resolution (~11:45 the same morning):** root cause diagnosed via the GitHub
events feed — the NUC runs had completed but never pushed (the GM4/GM5 lane
prompts end at "create file → STOP"; the transport rule lives in a separate
kickoff section; the MBP-Codex lane made the identical omission locally). Jon
re-ran the push step; both reports then arrived transport-rule-conformant as
fork branches `fleet/2026-06-10-windows-nuc-claude-fable` (`df6cbcd`) and
`fleet/2026-06-10-windows-nuc-codex` (`7244847`). Core four complete → gate
OPEN → this consolidation PR. Lesson folded into the command center's
consolidation-protocol section: future lane prompts carry the push step
inline.

## Local Evidence Inventory

Bounded sweep, 4 parallel read-only subagents, ~7 minutes wall clock.
Verdict taxonomy: commit / archive / ignore / duplicate / unsafe / needs-Jon.

### Other Garnet checkouts on this machine

- `/Users/IDC2.5/Desktop/Garnet-ios/Garnet` — git repo, but an unrelated
  standalone Xcode iOS app named "Garnet" (single "Initial Commit", no
  remotes, clean). Not a clone of the language repo. Verdict: needs-Jon
  (keep/archive/abandon decision).
- No second clone of `Island-Dev-Crew/garnet` exists on this machine within
  the swept roots. All other `*garnet*` directories are plain (non-git)
  bundles inventoried below.

### Main repo branch/stash/worktree state

- 89 local branches; squash-merge workflow means ancestry-merge count is
  meaningless (1). By `git cherry` patch-equivalence: 76 branches are fully
  represented on `origin/main` (safe-delete candidates); 12 have at least one
  novel patch not on main by patch-id:
  `agent-mac-opus/s15-compare-addendum`, `agent-mac-opus/s15-cst-rowan`,
  `agent-mac-opus/s15-cst-trait-stub`, `codex/s0-housekeeping`,
  `codex/s1-lsp-mvp-archive-claude`, `codex/s108-linux-utm-enforcement-proof`,
  `codex/s109-mac-cross-os-matrix`, `codex/s32-editions-compat`,
  `codex/s42-error-policy`, `codex/s43-doctest`, `codex/s5-parser-fuzz`,
  `codex/s60-release-readiness`. A `+` may reflect squash-combined or
  superseded work, not necessarily lost content — review before any deletion.
  Verdict: needs-Jon (branch hygiene pass, same shape as the Air machine's).
- Stashes: none. Worktrees: sole worktree at the repo path.

### Desktop

- `dogfood/` — 80 bundles, 803 MB, May 26 → Jun 5 (newest:
  `garnet-s119-release-readiness-4cec538`). MANIFEST.sha256 confirmed present
  in the 3 newest bundles; no `*source-included*`/`*secret*`/`*token*` paths
  found on this machine's evidence root (contrast: the Surface lane found
  source-included bundles on its machine and marked them unsafe). Verdict:
  archive (canonical durable-evidence root; stays).
- `dogfood/garnet-s45-slopguard-20260531T120756Z` vs `...T120827Z` — the only
  doubled slice (re-seal 71 s apart). Verdict: duplicate; Jon confirms via
  manifests which seal is authoritative before pruning the other.
- `garnet-audit-review-bundle-20260527/` (live copy, edited May 28 after the
  zip was sealed), `...20260527.zip` (sealed May 27 snapshot) — verdict:
  archive (both; the dir is the live copy). `garnet-audit-review-bundle-
  20260527 2/` — byte-identical MANIFEST, Finder-style copy. Verdict:
  duplicate.
- Loose `Screenshot 2026-*.png` (clustered on milestone dates) and
  `Screen Recording 2026-06-10 at 10.46.44 AM.mov` (kickoff-day session
  capture, content unverified). Verdict: needs-Jon (contents not opened in a
  read-only sweep).

### Downloads / Documents / temp

- `GARNET_REASSESSMENT_2026-06-11.md` (47 KB, Jun 11 07:31) — the reassessment
  document slated for `F_Project_Management/RESEARCH/` in the consolidation
  PR (`RESEARCH/` does not exist yet; the PR creates it). Verdict: commit
  (via PR-A when the gate opens).
- `TLDR analysis od Dual mode at v0.8.1.md` (Jun 11 07:01) — same research
  family/morning as the reassessment. Verdict: needs-Jon (ride the same
  `RESEARCH/` consolidation or stay out).
- `GARNET_KICKOFF_ADDENDUM_2026_06_10.md` (Jun 10 15:45) — companion to the
  untracked kickoff sheet; no repo copy. Verdict: needs-Jon (route alongside
  the kickoff doc or drop). The Surface lane independently inventoried its
  own copy of this file as needs-Jon.
- `W_REBUILD_SPEC.md`, `GARNET_TRUTH_DRIFT_PUNCHLIST.md`,
  `README_PROPOSED.md` in Downloads — `diff -q` byte-identical to the tracked
  copies under `F_Project_Management/W_REBUILD/`. Verdict: duplicate (safe to
  delete after Jon confirms).
- `GARNET_100X_STATUS_REPORT_2026-06-10.html` — generated one-shot status
  report, superseded by the fleet-report lineage. Verdict: archive.
- `garnet_captures/` (3 files, ~3.9 MB: png + 2 mp4 of the 2026-06-10 Fable
  session) — verdict: archive.
- `Garnet Creative Synth.zip` (Jun 11 09:01; 4 creative HTMLs + poster SVG +
  `stochastic-attestation-philosophy.md`) — fresh creative pack, unrouted.
  Verdict: needs-Jon.
- `GARNET_HORIZON_RESEARCH_AND_TRAJECTORY.md` (Jun 4) — strategy note, no
  repo counterpart. Verdict: needs-Jon (`RESEARCH/` candidate or archive).
- `garnet-windows-audit-opus-handoff-s1-s80-20260531-172318.tar.gz` — durable
  evidence bundle with MANIFEST; per CLAUDE.md belongs under
  `~/Desktop/dogfood/` with manifest resealed, not Downloads. Verdict:
  archive (relocate). The extracted sibling dir is a duplicate (keep the
  tarball as canonical).
- `debian-12-arm64-utm*` (~5.3 GB + torrent) — Linux cross-OS test
  infrastructure. Verdict: ignore (keep locally; not project docs).
- Promo SVGs (`garnet-promo-30s/60s/poster`, late May; 30s/60s appear to be
  the same asset renamed) and three historical planning zips
  (`opus 4.8 garnet.zip`, `garnet -site opus svg.zip`,
  `4agent garnet work 0.7.zip`) — verdict: archive (superseded planning
  history).
- `Telegram Desktop/garnet-release-readiness-report.pdf` (May 12) —
  historical pre-v0.8 record. Verdict: archive.
- `/private/tmp` — 4,449 ephemeral `garnet_*` test-scratch items, S107–S109
  reattest scratch checkouts, capture logs, and two `0.7.0-lsp-mvp` VSIX
  copies staged in a `v081full` verification dir on re-cut day (Jun 7;
  canonical copies live in the GitHub release). Verdict: ignore (all of it;
  OS tmp reaper handles it).
- `Documents/`: zero garnet/vsix matches.

### Sweep coverage statement

Searched: `*garnet*`-named directories (maxdepth 4) under `~/Desktop`,
`~/Documents`, `~/Downloads`, `~/clawd` (pruning `node_modules` and the main
repo's `target/`); full branch/stash/worktree/tag state of the main repo;
Desktop root enumeration; `~/Desktop/dogfood` counts + 3-newest manifest
checks + unsafe-pattern scan; template find patterns over Documents/
Downloads/`/tmp`/`/private/tmp/claude-502`; zips listed with `unzip -l` only.
This is a targeted read-only sweep of those roots, not an exhaustive disk
audit; deeper nesting, other accounts/volumes, and non-garnet-named clones
were not examined. No file was modified, moved, or deleted; no secrets
printed.

## Drift And Conflicts

- The two Windows NUC core fleet reports were absent from every prescribed
  channel at authoring — the blocking fact for S131–S134 at that point.
  Resolved the same morning (see the Fleet Inventory resolution note).
- PR #381 (Air report) deviates from the report-transport rule (lanes push a
  fleet branch, open no PR; consolidation absorbs them into one PR). Needs a
  disposition at consolidation time: absorb the file into the consolidation
  PR and close #381, or merge #381 first. Needs-Jon (closing another lane's
  open PR is outward-facing).
- The command center + Fable intake docs are untracked and their "Verified
  Baseline" sections cite `5161e64` (S130), one commit behind current main —
  confirmed stale by the MBP Codex report; refresh them in the consolidation
  PR. The Air report independently flags the untracked command center as a
  dead reference for any cold lane reading the W-REBUILD spec from main.
- `GARNET_ECC_PRIME_ASSIMILATION.md` (untracked) contains a historical
  "Signed Re-Cut Lane" prompt that is non-executable under current hard
  stops — supersede-or-label at consolidation (per MBP Codex report).
- Public-surface drift (README primitive counts, post-v0.5.0 wording, FAQ,
  `docs/getting-started.html:40` v0.5.0 installer reference, stale 0.7.0 VSIX
  release-asset names, hardcoded site metrics vs live 92.8%) is already
  verified with file:line evidence in the MBP Codex and Air reports; this
  report defers to those hit lists rather than duplicating them. The
  consolidation PR carries them verbatim as the RB-0c pre-verified set.
- Machine-specific evidence on this machine (dogfood bundles, captures,
  audit bundles) is machine-local until consolidated; nothing here is scored
  globally by this report.

## S129-S200 Recommendations From This Machine

1. Deliver the two Windows NUC reports (fleet branches per the transport
   rule, or files to this machine), or record an explicit Jon decision about
   the NUC lanes. This is the only blocker for PR-A.
2. Assemble PR-A immediately after: all other inputs are staged (3 delivered
   reports + this report + template, reassessment doc located, W-REBUILD pack
   tracked on main, drift hit lists pre-verified by two lanes).
3. Decide PR #381 disposition (absorb-and-close vs merge-first) as part of
   the consolidation, with Jon's sign-off on the close.
4. Schedule the branch-hygiene pass: 76 patch-equivalent local branches here
   (+ 25 on the Air machine per its report) are safe-delete candidates; the
   12 cherry-positive branches need review first.
5. Route the unrouted research/creative docs (TLDR dual-mode, HORIZON,
   Creative Synth, kickoff addendum) through the needs-Jon decision table in
   PR-A rather than ad-hoc commits.

## Safe Next Action

At authoring: Stop (deploy gate unmet; this report was the prescribed stop
output; needed the Windows NUC lanes to deliver).

After the same-morning resolution: Continue — assemble the S131–S134
source-of-truth consolidation PR (this report rides in it), then stop per GM2
after PR creation/merge.

## Claim Boundaries

- What this report proves: live recon evidence (HEAD/origin/tags/release/PR
  state), readiness-script outputs, verification-gate passes, and the sweep
  inventory of this machine on 2026-06-11, within the stated coverage.
- What this report does not prove: anything about Windows NUC state (no NUC
  claim is made beyond delivery absence on searched channels); production or
  v1.0 readiness; a signed Git tag (checksums/assets only); independent S114
  re-verification (S114 stays self-verified); macOS/Windows OS-sandbox,
  Wasmtime-fuel, memory, time, or `@mailbox` enforcement (named-deferred
  fences unchanged; only `@caps` + `@max_depth` are enforced, with seccomp
  Linux-only).
- "Enforced" claims in this report: none made.
- Self-authored/self-graded items: this report is authored by the same lane
  that will assemble the consolidation PR (same operator, not independent
  review); the sweep subagents are the same operator too. Independence
  boundaries are unchanged by anything here.
