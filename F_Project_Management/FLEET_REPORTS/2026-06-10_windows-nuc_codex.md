# Fleet Report - Windows NUC - Codex - 2026-06-10

**Lane:** Windows NUC cross-OS verifier.  
**Scope:** independent read/proof pass from the Windows NUC. No packaging changes, no tags, no release edits, no PRs.  
**Active checkout inspected:** `C:\Users\IslandDevCrew\Desktop\Garnet Opus 4.7 final\garnet`.

## Bottom Line

This NUC can reproduce important Windows-local evidence right now: agent contracts, tracked readiness, clean-VM status reporting, the full 20-case Windows domain matrix, and a no-GUI Garnet Studio smoke. It cannot honestly prove current `v0.8.1` Windows readiness from this checkout because the checkout is 76 commits behind `origin/main` and the local CLI still self-reports `garnet 0.5.0`.

Use this report as a fleet fact sheet, not as a release-completion claim.

## Repo State

| Fact | Evidence |
|---|---|
| Repo path | `C:\Users\IslandDevCrew\Desktop\Garnet Opus 4.7 final\garnet` |
| Local branch | `main` |
| Local HEAD | `c503866a4d8623bf9bdc042e7a77e041b4de6f5d` (`v0.8.0-7-gc503866`) |
| Remote main after fetch | `366e69f28812732df2f18fa6b80e1581dd7ac2d9` |
| Divergence | `main...origin/main [behind 76]`, no local commits ahead |
| Local untracked paths | `F_Project_Management/FLEET_REPORTS/`, `audit-windows/` |
| `v0.8.1` peeled commit | `8107c01a59a3f84925e9a4880a29a07a32b408eb` |
| `v0.8.1` tag subject | `S129: publish release-signing public key + fingerprint (private key set as secret) (#378)` |
| HEAD relation | local HEAD is an ancestor of `origin/main` |

I read the root `AGENTS.md` plus the relevant child contracts:

- `F_Project_Management/AGENTS.md`
- `apps/garnet-studio/src-tauri/AGENTS.md`
- `garnet-cli/AGENTS.md`
- `examples/AGENTS.md`

Contract implications: project-management reports preserve evidence and gaps; Windows/Linux Studio remains a thin Tauri wrapper; CLI output must stay truthful; examples should not imply production readiness.

## Release State

`gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json tagName,publishedAt,url,targetCommitish,assets` succeeded after host-network escalation.

| Fact | Evidence |
|---|---|
| Release | `v0.8.1` |
| Published | `2026-06-07T07:55:45Z` |
| URL | `https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.8.1` |
| targetCommitish | `main` |

Release assets found:

- `garnet-0.8.1-1.x86_64.rpm`
- `garnet_0.8.1-1_amd64.deb`
- `garnet-0.8.1-aarch64-apple-darwin.tar.gz`
- `garnet-0.8.1-x86_64-apple-darwin.tar.gz`
- `garnet-sbom-cyclonedx.tgz`
- `SHA256SUMS`
- `SHA256SUMS.asc`
- `garnet-0.7.0-lsp-mvp-darwin-arm64.vsix`
- `garnet-0.7.0-lsp-mvp-linux-x64.vsix`

No Windows release asset was found in `v0.8.1`. The VSIX assets are still named `0.7.0-lsp-mvp` on a `v0.8.1` release.

## Machine Inventory

| Item | Result |
|---|---|
| `rustc --version` | `rustc 1.95.0 (59807616e 2026-04-14)` |
| `cargo --version` | `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| `node --version` | `v22.22.2` |
| `wsl --status` | default distribution `Ubuntu`, default version `2` |
| `wsl -l -v` | `Ubuntu` stopped, WSL2; `docker-desktop` stopped, WSL2 |

WSL commands required escalation. Inside the sandbox they returned `Wsl/EnumerateDistros/Service/E_ACCESSDENIED`; with host escalation they succeeded.

## Verification Commands Run

| Command | Result | Notes |
|---|---:|---|
| `git fetch origin main --tags --prune` | pass | fetched remote metadata only; no checkout mutation |
| `python3 -B scripts/check-agent-contracts.py` | pass | `agent-contracts: ok (21 contracts)` |
| `python3 -B scripts/garnet_readiness_status.py --format json` | pass | 87/87 tracked slices, 100.0% tracked implementation plan |
| `python3 -B scripts/garnet_mit_readiness_status.py --format json` | pass after temp-dir escalation | `overall_status=active-partial`, `completion_percent=89.1` on this stale checkout |
| `python3 -B scripts/garnet_windows_linux_studio_status.py --format json` | pass | status `tauri-v2-shell-v0-5-readiness-parity-windows-clean-vm-verified-linux-open` |
| `python3 -B scripts/garnet_windows_clean_vm_installer_status.py --format json` | pass | status `clean-vm-proof-verified` |
| `python3 -B scripts/garnet_v0_8_0_release_readiness.py --format json` | pass | `release_ready=true`; see stale-test caveat below |
| `python3 -B scripts/test_garnet_windows_linux_studio_status.py` | pass after temp-dir escalation | 9 tests OK |
| `python3 -B scripts/test_garnet_windows_clean_vm_installer_status.py` | pass after temp-dir escalation | 4 tests OK |
| `python3 -B scripts/test_garnet_windows_linux_studio_shell.py` | pass | 5 tests OK |
| `python3 -B scripts/test_garnet_v0_8_0_release_readiness.py` | fail | stale test expects `v0.8.0` not to exist, but tags now include `v0.8.0` and `v0.8.1` |

The first MIT readiness run and two status-test runs failed under sandboxed temp permissions. Re-running with host temp/write access converted the temp-related failures to passes. I did not treat the initial temp failures as code defects.

## Byte-Identical Committed Truth

Proven here only through committed reporters and remote release metadata:

- The tracked implementation plan reports 87/87 slices complete.
- `v0.8.1` exists as a published GitHub release.
- `origin/main` is much newer than the local checkout and includes the post-release truth-sync work through `366e69f`.
- Release assets include Linux `.deb`, Linux `.rpm`, macOS tarballs, SBOM, checksums, and signature file.

Not proven by this NUC run:

- Byte-identical deterministic build on this Windows NUC.
- A local current-main `v0.8.1` binary.
- Windows release artifact parity with the published release.

## Machine-Local Evidence

These are real on this NUC, but they are not the same as current release truth:

- `target\debug\garnet.exe` exists and ran the domain matrix. Its Studio smoke health payload reports `garnet 0.5.0`, matching the stale checkout.
- `target\release\garnet-studio.exe` exists and ran `--studio-smoke`.
- A fresh no-GUI Studio smoke bundle was written at:
  `C:\Users\IslandDevCrew\Desktop\dogfood\garnet-studio-windows-linux\garnet-studio-windows-linux-smoke-20260610-165729`
- The smoke JSON reports:
  - `status=passed`
  - `mode=studio-smoke`
  - `source_included=false`
  - `provider_api_called=false`
  - `repo_path=C:\Users\IslandDevCrew\Desktop\Garnet Opus 4.7 final\garnet`
  - `cli_path=...\target\debug\garnet.exe`
- The smoke manifest includes `studio-smoke.json`, `evidence-contract.json`, and command stdout/stderr hashes.

## Windows Proof

Fresh Codex-run Windows domain proof:

```text
python3 -B scripts\smoke_garnet_studio_domain_matrix.py --suite all --garnet target\debug\garnet.exe --output-dir C:\Users\IslandDevCrew\Documents\Codex\2026-05-22\chrome-plugin-chrome-openai-bundled-github\tmp\fleet-domain-matrix --format json
```

Result:

- `status=passed`
- `platform=windows`
- `arch=AMD64`
- `case_count=20`
- `passed_cases=20`
- `failed_cases=0`
- `command_count=60`
- `passed_commands=60`
- `failed_commands=0`
- `source_included=false`
- `provider_api_called=false`
- output bundle:
  `C:\Users\IslandDevCrew\Documents\Codex\2026-05-22\chrome-plugin-chrome-openai-bundled-github\tmp\fleet-domain-matrix`

This proves that the stale local Windows checkout can parse/check/run the 20-domain matrix with its local debug CLI. It does not prove the same matrix against current `origin/main` or release `v0.8.1`.

Clean-VM Windows installer status from reporter:

- `clean_vm_verified=true`
- proof root:
  `C:\Users\IslandDevCrew\Desktop\dogfood\garnet-studio-windows-clean-vm`
- latest proof created `2026-05-23T02:46:34.076634+00:00`
- installer:
  `...\clean-vm-sandbox-20260522-212438\sandbox-share\Garnet Studio_0.1.0_x64-setup.exe`
- installer SHA-256:
  `e0dd3a16abf1695604e4e0460098bf6edf2d7ef167714ad43469b0c8be7b14a7`
- guest:
  `Microsoft Windows 11 Enterprise 10.0.26100 build 26100`, `AMD64`
- proof includes install log, `studio-smoke.json`, screenshot, and claim-boundary gate.

This is valid Windows x64 unsigned NSIS clean-VM proof for the local 0.1.0 Studio artifact. It is not a signed MSI, winget, Windows ARM64, or `v0.8.1` release proof.

## WSL Portability Only

WSL state is present:

- default distro: `Ubuntu`
- default WSL version: `2`
- `Ubuntu` stopped
- `docker-desktop` stopped

No WSL Garnet smoke was run in this Codex pass because this active checkout does not include the later S117 WSL/Tauri replay scripts that exist on newer `origin/main`.

This lane may use WSL as execution/portability evidence only. It cannot prove:

- Linux seccomp enforcement
- Linux OS-sandbox enforcement
- clean Linux install behavior
- non-WSL desktop GUI behavior
- Linux Tauri packaging completion

## Clean-Linux Proof

No clean non-WSL Linux proof was produced on this NUC during this pass.

The release has Linux `.deb` and `.rpm` assets, but this NUC run did not install them in a clean Linux VM or launch a Linux desktop Tauri shell. Therefore clean-Linux remains **not proven by this machine**.

## Not-Proven Items

- Current-main local verification: not proven because the active checkout is 76 commits behind.
- Local `v0.8.1` Windows CLI build: not proven; local CLI self-reports `0.5.0`.
- Windows release artifact: not found in the `v0.8.1` release assets.
- Windows signed MSI/AuthentiCode: not proven.
- winget or scoop install path: not proven.
- Windows ARM64: not proven.
- WSL enforcement: not a valid claim.
- Clean non-WSL Linux desktop GUI/Tauri launch: not proven.
- Linux package install smoke on this NUC: not run.
- Production or v1.0 readiness: not claimed.

## Recommended Next Commands For Orchestration

Run these from the active Garnet checkout after deciding how to handle the untracked local evidence directories:

```powershell
git status --short --branch
git merge --ff-only origin/main
python3 -B scripts/check-agent-contracts.py
python3 -B scripts/garnet_readiness_status.py --format json
python3 -B scripts/garnet_mit_readiness_status.py --format json
cargo build --release -p garnet-cli
.\target\release\garnet.exe version
python3 -B scripts\smoke_garnet_studio_domain_matrix.py --suite all --garnet target\release\garnet.exe --output-dir C:\Users\IslandDevCrew\Desktop\dogfood\garnet-studio-domain-matrix --format json
.\target\release\garnet-studio.exe --studio-smoke
```

After the fast-forward, rerun any newer S117/S120+ Windows/Linux/Tauri replay scripts that appear on `origin/main`; this stale checkout cannot see those script names.

## Report Verdict

Windows NUC is useful and ready for a follow-up verification run, but the active checkout is stale. The strongest current facts are:

1. The machine has the right Rust/Node/WSL basics.
2. The old local checkout still passes a full 20-case Windows domain matrix.
3. The old local Studio binary can write no-GUI smoke evidence.
4. The clean-VM unsigned x64 Studio proof is present and verified by its reporter.
5. None of that upgrades to a `v0.8.1` Windows/Linux/Tauri release claim until the checkout is fast-forwarded and the current scripts/artifacts are rerun.
