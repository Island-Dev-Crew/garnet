# Fleet Report — Windows NUC — Claude Code (Fable 5) — 2026-06-10

**Lane:** Windows/Linux/Tauri evidence (NUC). **Machine identity (hardware-confirmed):**
GMKtec NucBox M2Pro_S, hostname `NUCBOX_M2PRO_S`, Windows 11 Pro 10.0.26200.
**Scope:** read-only recon of found state;
no checkout mutation, no builds, no pushes. Feeds the S131–S134 source-of-truth
consolidation (MacBook Pro lead) and seeds the W-SHIP (S166–S178) distribution band plan.

**Truth discipline:** every claim below is labeled one of —
**[committed]** (on `origin/main`), **[released]** (GitHub release asset),
**[machine-local]** (exists on this NUC only), **[not-proven]** (no evidence found).
Corpus-search-miss ≠ absence: "not found" means not found by this recon, flagged for the
consolidation PR, never asserted as nonexistent.

---

## 1 · Repo truth (as found, 2026-06-10)

| Fact | Value |
|---|---|
| Repo path | `C:\Users\IslandDevCrew\Desktop\Garnet Opus 4.7 final\garnet` |
| Branch | `main`, **76 commits behind** `origin/main`, zero local commits ahead |
| Local HEAD | `c503866` — S83 post-tag release-truth reconciliation (#311) |
| `origin/main` | `366e69f` — W-REBUILD foundation-rebuild workstream pack (#380) |
| Working tree | Clean except untracked `audit-windows/` (S1–S80 Windows audit reports + summary) |
| Tags | `v0.8.1` newly fetched this session |
| Open PRs | **0** (`gh pr list --repo Island-Dev-Crew/garnet --state open` → `[]`) |
| GitHub auth | `gh` logged in as `Navigata1` (keyring), scopes `gist, read:org, repo, workflow` — fork-only account; direct push/merge to `island-dev-crew/garnet` is permission-blocked (`push:false`); fork-PR + work-profile browser-merge is the known-good path |

The local checkout **predates the entire S107–S130 arc** (cross-OS matrix, kernel red-team,
truth-sync gate, v0.8.1 cut/re-cut, signing). Nothing run against this working tree proves
anything about v0.8.1 truth until the checkout is fast-forwarded.

**Recommended first command for the next NUC session (safe, reversible):**
`git merge --ff-only origin/main` (tree is clean; no local commits at risk).

## 2 · Toolchain inventory [machine-local]

| Tool | Version | Note |
|---|---|---|
| rustc / cargo | 1.95.0 (matches repo toolchain expectation) | OK |
| node | v22.22.2 | OK for Tauri frontend + .mjs smoke scripts |
| python3 | 3.14.4 | OK for `scripts/*.py` gates |
| Java | OpenJDK **1.8.0_492** (Java 8; rejects `--version`, needs `-version`) | No Garnet dependency on Java identified in-repo; recorded as fleet inventory only. Anything requiring a modern JDK would need a separate install. |
| WSL | WSL2; distros: **Ubuntu** (default, Stopped), docker-desktop (Stopped) | **No Debian distro exists on this machine** — the goal-mode's "WSL/Debian state" wording does not match found state. |

## 3 · WSL state — and what it can and cannot prove

- WSL2 Ubuntu is present and functional (stopped at recon time). [machine-local]
- WSL evidence is **portability evidence only**: "the Linux x86_64 binary runs under WSL2."
- WSL **cannot** prove: clean-Linux-distro install experience, desktop GUI (Tauri) behavior
  on a real Linux desktop session, or **any seccomp/landlock/OS-sandbox enforcement claim**.
  Per the standing constraint, no Linux sandbox-enforcement claim will ever be sourced from
  WSL on this lane. The S108 Linux UTM enforcement proof (committed, Mac lane) is the model:
  real VM, not WSL.
- Known open finding WIN-S71-001 [machine-local, audit S1–S80]: a reporter passes Windows
  absolute paths into WSL bash — Windows↔WSL path translation is a live defect class on
  this lane, unverified against current main.

## 4 · Tauri / Garnet Studio state

| Item | Evidence | Label |
|---|---|---|
| `apps/garnet-studio` (Vite + `src-tauri/`) | present, with built `dist/` and `node_modules/` | [committed] structure / [machine-local] build state |
| `target/release/garnet-studio.exe` | built 2026-05-22 | [machine-local] |
| NSIS installer `target/release/bundle/nsis/Garnet Studio_0.1.0_x64-setup.exe` (1.9 MB) | built 2026-05-22 | [machine-local] — version string **0.1.0**, unsigned, never published |
| Clean-VM smoke bundle `..\dogfood\garnet-studio-windows-clean-vm\` | dated 2026-05-22 | [machine-local] — pre-dates v0.8.1; runbook exists as `F_Project_Management/GARNET_WINDOWS_STUDIO_CLEAN_VM_SMOKE_2026_05_21.md` [committed] |
| Windows/Linux Studio bundle `..\dogfood\garnet-studio-windows-linux\` | dated 2026-05-22 | [machine-local] |
| macOS Studio packaging | `apps/garnet-studio-macos`, `scripts/package_garnet_studio_macos.sh`, notarization scripts | [committed] — Mac lane's surface, listed for contrast: Windows has no equivalent packaging script |

**Tauri verdict:** a Windows installer *can* be produced on this machine and a clean-VM smoke
*has* been run once (May 22) — but everything is pre-v0.8.1, version-stamped 0.1.0, unsigned,
and machine-local. No Windows Studio artifact has ever been attached to a release. [not-proven
at release grade]

## 5 · CLI / local proof artifacts [machine-local]

- `target/debug/garnet.exe` (built 2026-05-31) runs and self-reports
  **`garnet 0.5.0`** with the pre-cohesion component banner (parser 0.3.0, stdlib 0.4.0
  "22 bridged primitives", etc.). This is consistent with the checkout predating
  S123 (#372, CLI bump to 0.8.1 + version-surface cohesion) — and is a concrete
  demonstration of why nothing built from this tree proves v0.8.1 claims.
- `target/release/garnet-lsp.exe` (2026-05-22). **No release-profile `garnet.exe` exists.**
- `audit-windows/` (untracked): S1–S80 per-slice reports + `summary.md`/`summary.json`,
  audited at v0.8.0-era HEAD `cc165e8`. **14 open Windows findings**, headline classes:
  uppercase-`.GARNET` discovery misses (S33/S36/S37/S46), CRLF-vs-LF seal hash instability
  (S38), WSL path translation (S71), VM/interpreter parity divergence on deep recursion
  (S73), aggregate-READY masking direct binary failures (S80). None re-verified against
  current main. The consolidation PR needs a commit/archive/ignore verdict on this directory.
- `..\dogfood\` (sibling of repo, durable evidence per CLAUDE.md): ~10 agentic-dogfood
  bundles (2026-05-19 → 05-21), `matrix-20260522-002737`, the two Studio bundles above,
  and `Codex Garnet Windows.md`. All pre-v0.8.1.

## 6 · Package artifacts — v0.8.1 release truth [released]

Release `v0.8.1`, published 2026-06-07T07:55:45Z, target `main`
(<https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.8.1>). Assets:

```
garnet-0.8.1-aarch64-apple-darwin.tar.gz     garnet-0.7.0-lsp-mvp-darwin-arm64.vsix
garnet-0.8.1-x86_64-apple-darwin.tar.gz      garnet-0.7.0-lsp-mvp-linux-x64.vsix
garnet_0.8.1-1_amd64.deb                     garnet-sbom-cyclonedx.tgz
garnet-0.8.1-1.x86_64.rpm                    SHA256SUMS
                                             SHA256SUMS.asc
```

Gaps this lane owns:
1. **No Windows asset of any kind** — no CLI zip/installer, no Studio installer.
2. **VSIX drift** — assets are version-stamped `0.7.0-lsp-mvp` on an 0.8.1 release; no
   `win32-x64` VSIX exists at all. (W-REBUILD RB-0d already plans the 0.8.1 VSIX re-pack
   as PREPARED-with-checksums; the asset swap stays Jon-gated — this lane must not collide,
   only contribute the Windows target.)
3. No winget/scoop presence (prerequisite: a stable Windows release asset; see plan §8).

## 7 · What remains unproven (the honest list)

| # | Claim nobody may make yet | Why |
|---|---|---|
| 1 | "Garnet v0.8.1 works on Windows" | No Windows binary at v0.8.1 exists anywhere — not released, not even built locally (local builds are 0.5.0-banner debug / May-22 era). |
| 2 | "Studio installs/runs on Windows at current version" | Only a 0.1.0 unsigned NSIS installer + one pre-v0.8.1 clean-VM smoke, machine-local. |
| 3 | "Garnet works on Linux" (desktop/GUI sense) | WSL portability evidence only on this lane; the committed Linux enforcement proof (S108) is CLI-scope, UTM, Mac-lane. No Linux desktop GUI (Tauri) proof exists on any lane that this recon found. |
| 4 | Any Linux seccomp/landlock/OS-sandbox enforcement from this machine | WSL cannot host that proof; explicitly out of bounds per goal-mode constraint. |
| 5 | "The 14 Windows audit findings are fixed" | Audited at `cc165e8` (v0.8.0-era); not re-run against `366e69f`. |
| 6 | "Local NUC verification of v0.8.1" | Checkout is 76 behind; no gate (`check-agent-contracts`, readiness reporters, dogfood matrix) has been run on current main on this machine. Deliberately not run this session — running them on a stale tree produces evidence-shaped noise. |
| 7 | Production / v1.0 anything | Out of bounds, standing constraint. |

## 8 · W-SHIP draft — Windows/Linux package & smoke plan (S166–S178, NUC-led)

Per W_REBUILD_SPEC §5, this band executes **after** S131–S134 consolidation merges and is
parallel-safe with W-REBUILD (it never edits `garnet-check`/`garnet-interp`/`garnet-stdlib`/
`garnet-parser`/`garnet-cst`). Pattern inherited: deploy gate → one slice per PR → evidence
ladder → stop points. All publication acts (assets, tags, Marketplace, manifests submitted
upstream) are **Jon-gated**; this lane prepares artifacts + checksums + transcripts only.

**Deploy gate (S166 entry):** consolidation PR merged; NUC checkout clean on current
`origin/main`; `audit-windows/` has its verdict; this report superseded by a refreshed one
if the gap exceeds a week.

| Slice | Deliverable | Acceptance evidence |
|---|---|---|
| S166 | **Lane bring-up + Windows truth re-baseline.** FF to main; release-profile `garnet.exe`; re-run the S1–S80 Windows gate set against current main; triage the 14 findings into fixed/live/wontfix. | Version banner reads 0.8.1; findings table with per-finding re-run output; `cargo test --workspace` + clippy green on Windows. |
| S167 | **Windows CLI package.** Reproducible zip (`garnet-0.8.1-x86_64-pc-windows-msvc.zip`: garnet.exe, garnet-lsp.exe, LICENSE, README) + SHA256; build recipe committed as script so any machine reproduces it. | Byte-level checksum committed; second build on same machine reproduces hash (document if not, and why). |
| S168 | **Windows CLI smoke, clean machine.** Fresh Win11 VM (Hyper-V checkpoint), no toolchains: unzip → `garnet check examples/safe_io_layer.garnet` → "0 diagnostics"; `garnet run` smoke; LSP handshake via `smoke_garnet_lsp_protocol.py`. | Transcript + OS build number + checksum of artifact under test; clean-machine template (below) satisfied. |
| S169 | **Studio Windows installer re-cut.** Tauri bundle at correct version (sync `tauri.conf` version from truth — coordinate with RB-0a truth.json rather than inventing a second source); NSIS + MSI; signing **deferred/Jon** (record signtool feasibility only). | Installer version string matches CLI version; SHA256s committed; install→launch→open-example→run smoke on the clean VM per the 2026-05-21 runbook, re-validated. |
| S170 | **Linux proof taxonomy doc + WSL re-scope.** One committed page defining the three Linux evidence grades: (a) WSL portability, (b) clean-Linux-VM CLI, (c) Linux desktop GUI. Re-label all existing WSL evidence as grade (a). Fix WIN-S71-001 path translation if still live. | Grades table committed; no doc anywhere upgrades WSL evidence to (b)/(c); S71 gate green or finding re-confirmed. |
| S171 | **Clean-Linux CLI proof (grade b).** Hyper-V Ubuntu Desktop VM (real distro, not WSL): install `.deb` from the release → CLI smoke set. | Transcript + distro/kernel version + asset checksum verified against released SHA256SUMS (and `.asc` signature check). |
| S172 | **Linux desktop GUI proof (grade c) — the one nobody has.** Build Studio in the Linux VM (webkit2gtk deps documented), launch in a real desktop session, run the domain-matrix smoke subset. Explicitly NOT a sandbox-enforcement claim. | Screen recording or screenshot series + matrix subset transcript; deps list committed as the Linux packaging recipe. |
| S173 | **VSIX naming/publication cleanup (prepare-only).** Re-pack VSIX at 0.8.1 with sane naming (`garnet-lsp-0.8.1-<target>.vsix`) for darwin-arm64, linux-x64, **win32-x64** (new); checksums; coordinate with RB-0d (Mac lane prepares the swap — NUC contributes the win32 target, no duplicate re-pack). | Three VSIX + SHA256s staged in-repo or as PR artifacts; `code --install-extension` smoke on Windows; asset swap left to Jon. |
| S174 | **VSIX Marketplace feasibility memo.** Publisher account, namespace, signing, CI implications — memo only, no publication, no CI edits. | Committed memo with decision points escalated to Jon. |
| S175 | **scoop feasibility + draft manifest.** JSON manifest against the (future) Windows release asset; validate via local bucket install. Blocked-by: S167 asset existence — note explicitly. | Manifest committed under `installer/`; local `scoop install` transcript from a personal bucket; upstreaming = Jon. |
| S176 | **winget feasibility + draft manifest.** `winget validate` + `winget install --manifest` locally; document the signed-installer-or-hash question honestly (unsigned NSIS + SmartScreen reality). | Validated manifest committed; install/uninstall transcript; microsoft/winget-pkgs submission = Jon, listed as such. |
| S177 | **Clean-machine evidence template (cross-OS).** Promote the S168/S171 procedure to a committed template: required fields (OS build, artifact checksum vs released SHA256SUMS, signature verification step, transcript, screenshots for GUI), so Air/Mac lanes produce comparable evidence. | `FLEET_REPORTS/CLEAN_MACHINE_TEMPLATE.md` (or location per consolidation verdict) committed; one retroactive application to S168 evidence. |
| S178 | **W-SHIP band report.** Roll-up: what is now released-grade vs machine-local vs still unproven; refreshed NUC fleet report; handoff to W-REACH (S179+). | Band report committed; unproven list explicitly carried forward, never silently dropped. |

**Standing constraints for the band:** no tag pushes, no release creation/edits, no CI/gate
changes, no ECC hooks; WSL never upgraded past grade (a); no sandbox-enforcement claims from
WSL; no production/v1.0 language; calibrated honesty — evidence, never claims.

## 9 · Recommended next commands (for the orchestration lane, post-consolidation)

```powershell
git -C garnet merge --ff-only origin/main
python3 garnet/scripts/check-agent-contracts.py
python3 garnet/scripts/garnet_readiness_status.py --format json
python3 garnet/scripts/garnet_mit_readiness_status.py --format json
cargo build --release -p garnet-cli          # then: garnet.exe --version must read 0.8.1
python3 garnet/scripts/garnet_windows_audit_status.py
```

## 10 · Fleet routing note (for Jon / consolidation)

A "Surface original-Tauri provenance harvest" goal mode was pasted into this NUC session on
2026-06-10. Hardware identity was checked before any harvest action: this machine is the
GMKtec NucBox M2Pro_S (the NUC), **not** a Surface. The Surface harvest was therefore **not
executed** — no inventory, no report, no `fleet/...surface...` branch exists. It must be
re-routed to the actual Surface machine; no `2026-06-10_surface_original-tauri-provenance.md`
should be trusted unless authored there.

---

*Prepared by the Windows NUC Claude lane (Fable 5), 2026-06-10. Read-only session: no
checkout mutation, no builds, no pushes, no tags, no releases, no CI changes. Stopping here
for MacBook Pro consolidation per goal mode.*
