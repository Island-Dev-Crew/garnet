# Garnet S131–S134 Source-of-Truth Consolidation

Date: 2026-06-11 · Lane: MacBook Pro Claude (Fable 5), consolidation lead
Scope: the single docs/report-only PR that converges the 2026-06-10 fleet run
into repo-visible truth. No code, no CI/gate/threshold change, no release/tag/
asset mutation, no live README/site edits, no crate renames, no ECC hooks.

This document is episodic memory (per `F_Project_Management/AGENTS.md`): what
the fleet found, what was verified live, which artifacts got which verdicts,
and what only Jon can decide. Where it cites drift, the citation was re-run on
this machine on 2026-06-11 at `main @ 366e69f` — not inherited from any single
lane's claim.

---

## §1 · Gate record (GM2 deploy-gate condition, as required in the PR body)

| Required report | Delivered | Channel | Transport-rule conformant |
|---|---|---|---|
| `2026-06-10_macbook-pro_claude-fable.md` | yes (authored by this lane under the GM2 stop procedure, then updated with the resolution) | local, consolidation lane | n/a (consolidator) |
| `2026-06-10_macbook-pro_codex.md` | yes | local file on the consolidation machine | no — written locally, never pushed to a fleet branch (harmless here: consolidation runs on this machine) |
| `2026-06-10_windows-nuc_claude-fable.md` | yes | fork branch `fleet/2026-06-10-windows-nuc-claude-fable` (`df6cbcd`) | yes (after the transport gap below) |
| `2026-06-10_windows-nuc_codex.md` | yes | fork branch `fleet/2026-06-10-windows-nuc-codex` (`7244847`) | yes (after the transport gap below) |
| `2026-06-10_macbook-air_claude-fable.md` (recommended) | yes | open PR #381, branch `codex/fleet-report-macbook-air-claude` | no — the transport rule says "open no PR"; the file is absorbed into this consolidation PR and #381 is closed with an absorption comment after merge |
| `2026-06-10_macbook-air_codex.md` (recommended) | not delivered | — | recommended-not-required; recorded as not delivered, not inferred-unavailable |
| `2026-06-10_surface_original-tauri-provenance.md` | yes (single combined report) | fork branch `fleet/2026-06-10-surface-original-tauri-provenance` (`a8f3297`) | yes — the only lane that followed the rule exactly |

**Gate verdict: MET, condition "all core reports present."** No
"Surface unavailable/deferred by Jon" note was needed — the Surface delivered.

**The transport gap, root-caused.** At first inventory (2026-06-11 ~09:30),
both NUC reports were absent from every prescribed channel (both remotes' full
branch lists, open/closed PRs, local `FLEET_REPORTS/`, Downloads/transfer
folders, Spotlight content search). Jon confirmed both NUC runs had completed.
The GitHub events feed for both accounts showed exactly three Garnet
deliveries in the fleet window — W-REBUILD pack (Jun 10 19:38Z), Air report
(22:50Z), Surface report (Jun 11 00:10Z) — and zero NUC pushes. Cause: the
GM4/GM5 lane prompts end at "create the report file → STOP"; the
push-the-fleet-branch transport rule lives in a separate kickoff section the
prompts never repeat. The MBP-Codex lane made the identical omission locally.
After Jon re-ran the push step (~11:45), both NUC branches arrived
conformantly. **Lesson (folded into the command center's consolidation
protocol): future lane prompts carry the transport step inline, so a lane
cannot truthfully complete without delivering.**

## §2 · Converged repo/release truth (live-verified this session)

- `origin/main = 366e69f` (`docs: add W-REBUILD foundation-rebuild workstream
  pack (#380)`); the W-REBUILD pack is tracked under
  `F_Project_Management/W_REBUILD/`.
- `v0.8.1`: annotated tag `4a6442d` → commit `8107c01`, local == origin,
  release published 2026-06-07T07:55:45Z, 9 assets (`.deb`, `.rpm`, two darwin
  tarballs, CycloneDX SBOM, `SHA256SUMS`, `SHA256SUMS.asc`, two stale
  `garnet-0.7.0-lsp-mvp-*.vsix`). **Wording rule (MBP-Codex P0): "signed"
  means signed checksums/assets/binaries — never a signed Git tag.
  `git tag -v v0.8.1` → `error: no signature found`.**
- Readiness: tracked plan 87/87 (100.0%); MIT/productization `active-partial`
  **92.8%** on current main. The NUC-Codex run reported 89.1% — measured on a
  checkout 76 commits behind (`c503866`, S83-era); the delta is checkout
  staleness, not a reporter disagreement. Both NUC lanes correctly declined to
  treat stale-tree results as v0.8.1 evidence.
- Windows truth (both NUC lanes agree): **no Windows release asset of any kind
  exists on v0.8.1**; local NUC binaries are 0.5.0-banner/May-22-era; the
  0.1.0 unsigned NSIS Studio installer + one pre-v0.8.1 clean-VM smoke are
  machine-local evidence only; 14 Windows audit findings (S1–S80, audited at
  `cc165e8`) are unverified against current main. "Garnet v0.8.1 works on
  Windows" **may not be claimed by anyone today.**
- Linux truth: WSL evidence is portability-grade only (both NUC lanes); the
  committed S108 Linux enforcement proof is UTM/CLI-scope from the Mac lane;
  no Linux desktop GUI (Tauri) proof exists on any lane. Seccomp remains
  Linux-only; no sandbox-enforcement claim may be sourced from WSL.
- Enforcement boundary (unchanged, no widening anywhere in the fleet): only
  `@caps` + `@max_depth` are enforced by deterministic traps (both backends);
  `@bounded` (Wasmtime fuel), memory, time, `@mailbox`, and macOS/Windows
  OS-sandbox remain declared-not-enforced. S114 stays **self-verified**
  (`CURRENT_STATE.md:15–16`) pending an actually independent re-verification.

## §3 · Consolidated drift map (the RB-0c pre-verified hit list)

Every row below was re-verified on this machine on 2026-06-11
(`rg -n` at `366e69f`; release assets via `gh release view`). RB-0c (and
RB-0b/RB-0d where noted) starts from this list, not from the punchlist's
citations — the Air lane documented punchlist A6's line-range and vocabulary
erratas, and this list supersedes them.

Ground truth for counts: `garnet-stdlib/src/registry.rs` has **80** primitive
rows (`rg -c '^\s*p\('`).

### Public-surface narrative drift (RB-0c scope)

| # | Location | Stale text (verbatim, trimmed) | Current truth |
|---|---|---|---|
| 1 | `README.md:145` | `garnet-stdlib \| 24 registry primitives with capability metadata` | 80 primitives |
| 2 | `README.md:193` | `Current main is post-v0.5.0 source; the latest tagged release is…` | main is post-v0.8.1 source (the sentence's tag half is already correct) |
| 3 | `README.md:212` | `✅ 24 stdlib registry primitives bridged through the interpreter` | 80 |
| 4 | `README.md:71` | `GARNET_VERSION=0.4.2 only when you intentionally want the previous release` | previous release is v0.8.0; v0.4.2 is several milestones back |
| 5 | `FAQ.md:55` | `v0.5.0 is research-grade and not production-complete` | v0.8.1 (still research-grade, not production-complete — keep the boundary language, fix the version) |
| 6 | `FAQ.md:57` | triple drift in one line: `24 bridged stdlib registry primitives` + `S1 LSP source surfaces` + `v0.5.0 … release assets` | 80 primitives; S16 LSP surface (hover, workspace symbols, CST-precise rename per `README.md:148`); v0.8.1 signed binaries |
| 7 | `FAQ.md:133` | `## What's coming after v0.5.0?` | post-v0.8.1 runway (S129+/W-REBUILD) |
| 8 | `docs/getting-started.html:40` | `The default installer now targets the published v0.5.0 release path; v0.4.2 remains available as the previous release.` | v0.8.1 / v0.8.0 — worst class: misleading-current on a first-time user's first page (Air finding, confirmed) |
| 9 | `docs/status.html` (whole page) | zero mentions of `v0.8` anywhere; only version-ish string is `v0.5 readiness reporters` (line 281) | the page that promises "exact current truth" is a frozen v0.5-era snapshot |

### Hardcoded site metrics (RB-0a/RB-0d scope — these become truth.json reads)

| # | Location | Stale value | Note |
|---|---|---|---|
| 10 | `docs/index.html:1450` | `1193` workspace tests | v0.5.0-era; no current doc-pinned replacement exists — re-derive from `cargo test --workspace` at stamping time |
| 11 | `docs/index.html:1451` | `136` security tests | markup splits "136" from "Security tests" — naive `rg "136 security"` misses it; pattern lesson recorded for the RB-0a checker |
| 12 | `docs/index.html:1455` | `0.93×` expressiveness | Unicode `×` (U+00D7) — ASCII `0.93x` greps miss it; same checker lesson |
| 13 | `docs/index.html:1460` + `docs/index.html:639` | `92.3%` (text) + `width: 92.3%` (CSS) | live value 92.8% and moving; the CSS width must move with the metric — two coupled edit points |
| 14 | `docs/index.html:1467` | `87/87 tracked slices` | v0.5-era ledger framing; S81–S130 landed since |
| 15 | `docs/status.html:87` + `docs/status.html:92` | `87/87` + `92.3%` | same class as #13/#14 on the status page |

### Release/extension asset drift (RB-0d scope, swap Jon-gated)

| # | Location | Drift | Note |
|---|---|---|---|
| 16 | v0.8.1 release assets | `garnet-0.7.0-lsp-mvp-darwin-arm64.vsix`, `garnet-0.7.0-lsp-mvp-linux-x64.vsix` | 0.7.0-named assets on the 0.8.1 release; no win32-x64 VSIX exists at all (NUC lanes to contribute that target per their S173 draft) |
| 17 | `editors/vscode/package.json:5,32` | `"version": "0.7.0"`, emits `garnet-0.7.0-lsp-precision.vsix` | **the root drift the release assets inherit** — RB-0d re-pack starts here, not at the release page |
| 18 | `CURRENT_STATE.md:120` | accurately mirrors the 0.7.0 packaging line | not itself wrong; will follow #17 automatically once the source changes |
| 19 | `editors/vscode/README.md:3,7,9,10,11` | `S1 LSP MVP` + "hover is not in this MVP" + "workspace symbols are deferred to S1.1" + "rename is deferred" | contradicts root `README.md:148` (S16: hover, workspace symbols, CST-precise rename shipped — the VSIX is literally named `lsp-precision`); blocks any LSP screenshot until reconciled (Air finding, confirmed) |

### Blog cadence drift (RB-0d scope, posting/dating Jon-gated)

| # | Location | Drift | Note |
|---|---|---|---|
| 20 | `docs/blog/index.html:65` | `Planned · 2026-06-01` @caps post | 10 days past its date, absent from `docs/blog/posts/` (only two 2026-05-20 entries exist) — a dated public promise currently broken |
| 21 | `docs/blog/index.html:80` | "monthly is the floor, not the ceiling" | promise currently unmet (no June post, no v0.8.0/v0.8.1 release posts) — fix by shipping/re-dating the draft (Jon), not by scrubbing the promise |

### Historical-exempt (MUST NOT be "fixed" — the RB-0a checker needs an exemption list)

- `docs/blog/index.html:60` — dated 2026-05-19 post summary citing 87/87 and
  55.8%: historical facts in a dated artifact.
- Dated release-post summaries (71.3%/55.8% class) and "Discussions live as of
  v0.5.0" wording: scrubbing these rewrites history; they are exempt, and the
  Part C / RB-0a guard must encode the exemption to avoid false positives.

## §4 · Machine-fingerprint reconciliation (NUC §10 vs the Surface branch)

The NUC-Claude report §10 records that a Surface-harvest goal mode was pasted
into the NUC session on 2026-06-10; the NUC checked hardware identity, refused
the misrouted prompt, and warned that no Surface report existed and none
should be trusted "unless authored there." A Surface report *was* subsequently
delivered (branch pushed Jun 11 00:10Z, after the NUC report was written).
Fingerprint comparison of the two reports' own evidence:

| Fingerprint | NUC (both NUC reports) | Surface report |
|---|---|---|
| Windows user profile | `C:\Users\IslandDevCrew\` | `C:\Users\jony0\` |
| Hostname/hardware | GMKtec NucBox M2Pro_S, `NUCBOX_M2PRO_S` | (not stated; Surface-era artifact trail) |
| Primary checkout | `Desktop\Garnet Opus 4.7 final\garnet` @ `c503866`, branch `main`, 76 behind | `Documents\Garnet` @ `c0cf2bf`, branch `codex/windows-clean-vm-proof`, dirty |
| WSL distros | Ubuntu (default) + docker-desktop | docker-desktop only |
| Artifact trail | 2026-05-31-era debug CLI, audit-windows/ | original-Tauri May-20–22 build outputs, `Garnet-work-preserve`, April research bundles |

Verdict: **two distinct machines; the Surface report is consistent with
genuine Surface authorship, and the NUC's warning was true at its writing
time** (pre-00:10Z) and correct in spirit — it refused to fabricate a Surface
harvest from the wrong hardware. Both behaviors are the calibrated-honesty
system working as designed. Jon, who dispatched the lanes, is the final
confirmer; flagged here rather than silently assumed.

## §5 · Artifact verdict roll-up + the needs-Jon decision table

Each fleet report carries its full per-artifact verdict table
(commit / archive / ignore / duplicate / unsafe / needs-Jon); this section
consolidates rather than repeats. Roll-up by machine:

- **MacBook Pro (Claude + Codex reports):** committed by this PR — the seven
  `FLEET_REPORTS/` files, the reassessment (+ Gap-6 appendix), this document,
  the updated command center, the intake + kickoff sheets. Archive-class:
  dogfood root (80 bundles, 803 MB, manifests verified on newest three, no
  unsafe patterns), audit-review bundle trio, captures, historical planning
  zips, windows-audit-opus handoff tarball (relocate to dogfood root +
  reseal). Ignore-class: `/private/tmp` scratch (4,449 items), Debian UTM
  infra, stale VSIX tmp copies. Duplicates: the three W_REBUILD Downloads
  drafts (byte-identical to tracked), extracted handoff dir, s45 double-seal
  pair (one of two).
- **Windows NUC (both reports):** nothing commit-class beyond the reports
  themselves; all local builds/installers/smoke bundles are machine-local
  archive-class, explicitly pre-v0.8.1; `audit-windows/` (untracked, S1–S80
  findings) needs a verdict → decision table. Recommended first NUC command
  next session: `git merge --ff-only origin/main` (clean tree, 0 ahead).
- **Surface:** report-only commit; original-Tauri build outputs hashed and
  archived in place; `source-included` advisory bundles marked **unsafe**
  (publication/provider-forwarding prohibited, local test evidence only);
  paused Studio-lane patch + parity report → decision table.
- **MacBook Air (Claude report):** read-only audit; its machine-local verdicts
  (local tag re-point, 25 safe-delete branches, s18/s19 review) → decision
  table.

**The one needs-Jon decision table** (consolidated from all six reports +
this lane's sweep; nothing below was acted on autonomously):

| # | Item | Machine | Decision needed |
|---|---|---|---|
| J1 | PR #381 disposition | GitHub | Absorbed by this PR; close #381 with absorption comment after merge (planned). Veto window: say so before/at merge. |
| J2 | `audit-windows/` (S1–S80 Windows audit, 14 open findings, untracked) | NUC | Commit as episodic audit record vs archive; findings re-run against current main is S166 either way |
| J3 | ECC-Prime pair (`ASSIMILATION` with historical re-cut prompt; `OVERLAY_RECOMMENDATION`) | MBP | Stay untracked (current state), commit with SUPERSEDED banner, or delete; intake banner documents the deliberate exclusion |
| J4 | Unrouted research docs: `TLDR analysis od Dual mode at v0.8.1.md`, `GARNET_HORIZON_RESEARCH_AND_TRAJECTORY.md`, `Garnet Creative Synth.zip`, kickoff addendum (Downloads) | MBP | Ride a follow-up `RESEARCH/` PR, archive, or drop |
| J5 | Branch hygiene: 76 patch-equivalent local branches (MBP) + 25 (Air); 12 cherry-positive (MBP) + `agent-mac-codex/s18,s19` (Air) need review first | MBP+Air | Approve a deletion pass after the cherry-positive review |
| J6 | Local tag drift on Air (`v0.4.2` local ≠ origin); two local-only `parked/*` tags on MBP | Air+MBP | Re-point Air's tag to origin; keep-or-push `parked/*` |
| J7 | Desktop loose screenshots + kickoff-day recording; `Garnet-ios` Xcode experiment; s45 double-seal pair | MBP | Keep/archive/prune |
| J8 | Surface paused-lane patch (`windows-studio-core-workflow-stale-local.patch`) + parity report + clean-VM contract reconciliation | Surface | Review before any promotion; never apply the stale patch wholesale |
| J9 | Splash-art candidates + Grok handoff zip + `Codex Garnet Windows.md` | Surface | Promote/archive/drop |
| J10 | VSIX 0.8.1 re-pack + release-asset swap (drift rows #16–17) | release | RB-0d prepares with checksums; **the swap on the published release is yours alone** |
| J11 | @caps blog post (drift rows #20–21) | site | Ship or honestly re-date; posting/dating yours |
| J12 | `2026-06-10_macbook-air_codex.md` never delivered (recommended lane) | Air | Run it late, or record as skipped |

## §6 · Runway registration + what this PR unblocks

The command center now carries (this PR): the 2026-06-11 baseline refresh, the
**W-REBUILD registration** (RB-0 ≡ S135–S140 front-door band; RB-1..RB-7 =
Foundation workstream between front-door and playground bands; trust band
S141–S150 parallel on other lanes) — completing the spec's outstanding P0
runway duty (the pack itself landed in #380) — and the **quarterly
competitive-watch standing slice** (reassessment Gap 7, first due 2026-09).

W-REBUILD deploy gate (spec §1) after this PR merges: condition 1 (core fleet
reports exist) **met**; condition 2 (consolidation PR merged) **met at
merge**; condition 3 (clean tree on current `origin/main`) verified at RB
kickoff. **RB execution still does not start until Jon lights the lane** —
this PR is consolidation, not implementation. The NUC reports' S166–S178
draft plan and this drift map give RB-0 and W-SHIP pre-verified starting
points.

## §7 · Claim boundaries

- This document proves: the gate record, the live-verified drift rows, the
  fingerprint comparison, and the verdict roll-up, each backed by the cited
  commands/files at `366e69f` on 2026-06-11.
- It does not prove: anything about v0.8.1 on Windows (no claim possible —
  see §2); Linux desktop GUI behavior; production or v1.0 readiness; a signed
  Git tag; independent S114 re-verification; enforcement beyond the `@caps` +
  `@max_depth` deterministic traps (seccomp Linux-only).
- Self-grading boundary: this consolidation was authored by the same operator
  family (Claude/Codex lanes) that produced five of the six inputs; the
  Surface/Air lanes are separate machines but not independent reviewers in
  the S114 sense. Nothing here counts toward the independent re-verification
  the trust band owes.
- Corpus-search rule honored throughout: a search miss is recorded as a miss
  on the stated channels, never as proof of absence.
