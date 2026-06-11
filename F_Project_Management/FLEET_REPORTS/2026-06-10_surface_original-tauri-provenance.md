# Garnet Surface Original-Tauri Provenance Harvest

Date: 2026-06-10
Lane: Surface original-Tauri provenance harvest
Mode: read-only inventory plus report-only fleet branch
Report branch: `fleet/2026-06-10-surface-original-tauri-provenance`

## Scope and hard stops observed

- No tags were pushed.
- No release was cut or re-cut.
- No ECC install or hook change was attempted.
- No tracked repo files were modified outside this report file.
- No binaries, archives, VSIX files, screenshots, proof bundles, or package artifacts were staged for commit.
- Secret-bearing files were not opened. `gh auth status` output was recorded only as account/scope truth, with the token value omitted.

## Repo and GitHub truth

Commands run from `C:\Users\jony0\Documents\Garnet` unless noted:

| Command | Result |
| --- | --- |
| `gh auth status` | Authenticated to GitHub as `Navigata1`; HTTPS git protocol; scopes include `repo`, `workflow`, `read:org`, `gist`; token value redacted by the CLI output and not copied here. |
| `git fetch origin main --tags --prune` | Succeeded against `https://github.com/Island-Dev-Crew/garnet.git`; `origin/main` at `366e69f28812732df2f18fa6b80e1581dd7ac2d9`. |
| `git status --short --branch` | Primary checkout is dirty on `codex/windows-clean-vm-proof...fork/codex/windows-clean-vm-proof`. |
| `git rev-parse HEAD origin/main` | Primary checkout `HEAD=c0cf2bf62c791d9678711afdf501c1f8a6745ffc`; `origin/main=366e69f28812732df2f18fa6b80e1581dd7ac2d9`. |
| `gh pr list --repo Island-Dev-Crew/garnet --state open --limit 20` | One open PR found: `#381 Fleet report: MacBook Air / Claude Fable - GM6 independent audit (S131-S200 runway input)`, branch `Navigata1:codex/fleet-report-macbook-air-claude`. |
| `gh release view v0.8.1 --repo Island-Dev-Crew/garnet` | Release `v0.8.1` is published, not draft, not prerelease; created `2026-06-07T07:53:00Z`, published `2026-06-07T07:55:45Z`; assets include `.deb`, `.rpm`, macOS tarballs, VSIX packages, SBOM, `SHA256SUMS`, and `SHA256SUMS.asc`. |

## Local Garnet checkouts and worktrees

| Path | Kind | Branch / status | HEAD / origin-main | Verdict | Notes |
| --- | --- | --- | --- | --- | --- |
| `C:\Users\jony0\Documents\Garnet` | Git worktree, primary Surface checkout | `codex/windows-clean-vm-proof`; dirty with tracked modifications and one untracked handoff doc | `HEAD=c0cf2bf62c791d9678711afdf501c1f8a6745ffc`; `origin/main=366e69f28812732df2f18fa6b80e1581dd7ac2d9` | needs Jon | Stale relative to S130+ main. Contains historical original-Tauri build outputs under `target\release`; do not reset or delete. |
| `C:\Users\jony0\Documents\Garnet-windows-studio-pr` | Git worktree created during interrupted Windows Studio core-workflow lane | `codex/windows-studio-core-workflow-shell`; dirty with 8 Studio/status files modified | `HEAD=366e69f28812732df2f18fa6b80e1581dd7ac2d9`; `origin/main=366e69f28812732df2f18fa6b80e1581dd7ac2d9` | needs Jon | Contains unpushed implementation work from the paused Studio PR lane. This fleet lane did not stage or push it. |
| `C:\Users\jony0\Documents\Garnet-work-preserve` | Non-git preservation folder | n/a | n/a | archive / needs Jon | Holds patch/report copied before fresh Studio worktree creation. Hashes listed below. |
| `C:\Users\jony0\Downloads\Garnet` | Non-git historical research/handoff folder | n/a | n/a | archive | Contains April 2026 research decks, papers, redline packages, and handoff docs. It is not a repo checkout. |
| `C:\Users\jony0\Documents\Garnet-surface-fleet-report` | Git worktree for this report | `fleet/2026-06-10-surface-original-tauri-provenance` | `HEAD=366e69f28812732df2f18fa6b80e1581dd7ac2d9` before this report commit | commit | Dedicated report-only branch/worktree. |

Known local branch names visible from the two Git worktrees:

- `codex/tauri-v2-studio-shell`
- `codex/windows-clean-vm-proof`
- `codex/windows-linux-studio-mvp`
- `codex/windows-studio-core-workflow-shell`
- `codex/windows-studio-v05-parity`
- `main`
- `fleet/2026-06-10-surface-original-tauri-provenance`

## Original Tauri / Studio build artifacts

These are local build outputs only. They are useful provenance, but they are not commit candidates.

| Artifact | Size | Modified UTC | SHA256 | Verdict | Safety note |
| --- | ---: | --- | --- | --- | --- |
| `C:\Users\jony0\Documents\Garnet\target\release\garnet-studio.exe` | 8,894,976 | `2026-05-20T23:02:31.8440182Z` | `554C3019DC44C6D2F3D63E9EB1369EE620BBB8789A9412E58C78D5BB1FFD274D` | archive | Local unsigned Windows Studio executable; hash only, never commit binary. |
| `C:\Users\jony0\Documents\Garnet\target\release\bundle\nsis\Garnet Studio_0.1.0_x64-setup.exe` | 1,904,300 | `2026-05-20T22:40:15.3201083Z` | `5381882C089F4B5356B75661B1D0048982F69DDEE42623999D1A432D61890B88` | archive | Unsigned NSIS installer; useful for provenance only, not a signed MSI/winget claim. |
| `C:\Users\jony0\Documents\Garnet\target\release\garnet_studio_lib.dll` | 116,224 | `2026-05-20T23:21:20.4196734Z` | `61AC9017188BC91013606103B7FE7D5A3141509E5DC5439CFE0815BC29FF9A6E` | archive | Build output; hash only. |
| `C:\Users\jony0\Documents\Garnet\target\release\garnet_studio_lib.lib` | 89,440,506 | `2026-05-20T23:20:58.8404326Z` | `490891EF93741904453F19B1E785036F5B236E57E05877C9636FA1944C51C855` | archive | Build output; large binary/static library, do not commit. |
| `C:\Users\jony0\Documents\Garnet\target\release\garnet_studio.pdb` | 5,058,560 | `2026-05-20T23:02:31.9421375Z` | `E76B5435D8A64BFE84205D789E5BBC9A94A5ABB6B19579B35B9A3195B50052B5` | archive | Debug symbol file, do not commit. |

Origin/main comparison: `git ls-files target apps/garnet-studio/target` returned `0`, so these build outputs are not tracked on origin/main. Keep as local provenance only.

## Desktop dogfood and clean-VM evidence

Top-level `C:\Users\jony0\Desktop\dogfood` bundle inventory:

| Path | Modified UTC | Files | Manifest SHA256 | Verdict |
| --- | --- | ---: | --- | --- |
| `C:\Users\jony0\Desktop\dogfood\garnet-agentic-dogfood-20260519-021039` | `2026-05-19T07:23:17.0288903Z` | 434 | `44436FD0E95D27979B826B80FD192D60E6AA59CC8248A17090C6E1FDA9801950` | archive |
| `C:\Users\jony0\Desktop\dogfood\garnet-agentic-dogfood-20260519-025527` | `2026-05-19T08:09:25.5506367Z` | 435 | `EB736D36BE2976833670E909FE337CA6C9B28B6C5D578916B34738D0D502339C` | archive |
| `C:\Users\jony0\Desktop\dogfood\garnet-agentic-dogfood-20260519-033135` | `2026-05-19T08:41:08.8736175Z` | 435 | `0C87C53CC837FB225C747A79E94263AB18065F9082001B06A5F9AA55A2039008` | archive |
| `C:\Users\jony0\Desktop\dogfood\garnet-agentic-dogfood-20260519-051823` | `2026-05-19T10:30:51.9659982Z` | 435 | `16EDF49272E2D556868E0C46C258821A145B947EAB06E2A0745F8B826225650C` | archive |
| `C:\Users\jony0\Desktop\dogfood\garnet-agentic-dogfood-20260520-191025` | `2026-05-21T00:50:13.8203098Z` | 444 | `65822F6DE1E0EADA1CA823CF7E5F8BE79ED362D2603AC531B167A40B66581391` | archive |
| `C:\Users\jony0\Desktop\dogfood\garnet-agentic-dogfood-20260520-200322` | `2026-05-21T01:34:45.8585336Z` | 439 | `9B7242477353E54CA8EA2455824249FC04891211FAFB47B228C9B14DECC252ED` | archive |
| `C:\Users\jony0\Desktop\dogfood\garnet-agentic-dogfood-20260520-210552` | `2026-05-21T02:38:10.1964480Z` | 438 | `39C16AD5E3A56EC21B7B1D3FD5337582C1184CD4BC2864281D6E5146F772353F` | archive |
| `C:\Users\jony0\Desktop\dogfood\garnet-agentic-dogfood-20260520-220348` | `2026-05-21T03:34:09.4154591Z` | 439 | `333AA75843F5B8FB548EB5008A3A8B19FBD8BE93CEAF9F1768BB644421B961ED` | archive |
| `C:\Users\jony0\Desktop\dogfood\garnet-agentic-dogfood-20260520-224238` | `2026-05-21T04:18:51.1004892Z` | 439 | `54C847EEAB23FF4D68000ECB899A48DF67529C55E32316FD9BD6DCF049E04F63` | archive |
| `C:\Users\jony0\Desktop\dogfood\garnet-agentic-dogfood-20260521-010033` | `2026-05-21T06:32:57.5090232Z` | 439 | `7A478659242C4012E4502FCD13037FF38F4A355EA7E0C3F5267FAAEC4CF50962` | archive |
| `C:\Users\jony0\Desktop\dogfood\matrix-20260522-002737` | `2026-05-22T05:56:18.7174187Z` | 445 | `62DC980D5CB58E1CE4ED66D9FACFD82571E4834BEE19A8223D57F01583AC4D8B` | archive |
| `C:\Users\jony0\Desktop\dogfood\garnet-studio-windows-clean-vm` | `2026-05-22T06:04:28.3311101Z` | 449 | no top-level manifest | archive / needs Jon |
| `C:\Users\jony0\Desktop\dogfood\garnet-studio-windows-linux` | `2026-05-20T23:22:25.0279714Z` | 42 | no top-level manifest | archive / needs Jon |
| `C:\Users\jony0\Desktop\dogfood\assets` | `2026-05-22T10:52:25.6000226Z` | 4 | no top-level manifest | archive |

Clean-VM/studio-relevant files:

| Artifact | Size | Modified UTC | SHA256 | Verdict | Safety note |
| --- | ---: | --- | --- | --- | --- |
| `C:\Users\jony0\Desktop\dogfood\garnet-studio-windows-clean-vm\browser-ui\release-readiness-windows-vm-installer.png` | 73,471 | `2026-05-22T06:04:29.2590878Z` | `EF585B392A26854009480962CE8A0AD704AF02727CF86E666F680E663A00DC64` | archive | Screenshot evidence, not a binary. Duplicate of Downloads/Desktop asset. |
| `C:\Users\jony0\Desktop\dogfood\garnet-studio-windows-clean-vm\contract-20260522-002605\windows-clean-vm-installer-status.json` | 5,306 | `2026-05-22T05:26:11.4130255Z` | `8125CC3D31EDF1B61E47D2849F4E39A68FF4A85222C42F507720E8161C351554` | archive / needs Jon | Clean-VM contract/status evidence. Needs comparison to committed Windows clean-VM proof before any copy claim changes. |
| `C:\Users\jony0\Desktop\dogfood\garnet-studio-windows-clean-vm\contract-20260522-002605\windows-clean-vm-installer-status.md` | 3,251 | `2026-05-22T05:26:11.4156256Z` | `E9FBB6C73C7808D1E8CB743B1336EAEFE91CA7814E83501F180B1593BC02C0C4` | archive / needs Jon | Same as above. |
| `C:\Users\jony0\Desktop\dogfood\garnet-studio-windows-clean-vm\contract-20260522-002605\MANIFEST.sha256` | 210 | `2026-05-22T05:26:11.8015419Z` | `257A31AFB530D5382A73B8BDFE216E9A690B3FE7172CA9C46F0FFB3A54971511` | archive | Manifest only. |
| `C:\Users\jony0\Desktop\dogfood\garnet-studio-windows-clean-vm\matrix-20260522-002737\MANIFEST.sha256` | 49,322 | `2026-05-22T05:56:18.7123623Z` | `62DC980D5CB58E1CE4ED66D9FACFD82571E4834BEE19A8223D57F01583AC4D8B` | archive | Dogfood matrix manifest. |
| `C:\Users\jony0\Desktop\dogfood\garnet-studio-windows-clean-vm\matrix-20260522-002737\dogfood-readiness-data.json` | 271,619 | `2026-05-22T05:55:34.7112135Z` | `1AE88717430FB6B33C882B73D4488C750C949891ADCF4AD633CB820B3A3B0E57` | archive | Report data, not a binary. |
| `C:\Users\jony0\Desktop\dogfood\garnet-studio-windows-clean-vm\matrix-20260522-002737\dogfood-readiness-matrix.md` | 41,735 | `2026-05-22T05:55:34.7331167Z` | `052805C1A9ACA378B8F3800D50629E5F389B6F58C5609469D52392EED11A1E5D` | archive | Report output. |

Safety note: multiple dogfood bundles intentionally contain paths with `source-included` in the name, including:

- `...\converter-advisory-review-source-included-bundle\MANIFEST.sha256` with SHA256 `19A4A7AA04ABB0DB97276F3D7583CD866B3FF96CB7441094A191B3AD92091434`
- `...\converter-advisory-handoff-source-included-bundle\MANIFEST.sha256` with SHA256 `19A4A7AA04ABB0DB97276F3D7583CD866B3FF96CB7441094A191B3AD92091434`

Verdict for `source-included` bundles: `unsafe` for publication or provider-facing reuse. They can remain local test evidence only.

## Downloads, handoff docs, splash assets, and Grok packet

| Artifact | Size | Modified UTC | SHA256 | Verdict | Safety note |
| --- | ---: | --- | --- | --- | --- |
| `C:\Users\jony0\Downloads\files-b9b2e7bc.zip` | 15,472 | `2026-05-16T22:58:26.0314227Z` | `426710E608216A817E24DB0C2278799340C65A7E24B1067EA7F2CE9BA9716542` | archive / needs Jon | Handoff zip from Grok/xAI lane. Inventory only; do not commit archive. |
| `C:\Users\jony0\Downloads\Codex Garnet Windows.md` | 19,862 | `2026-05-22T12:03:06.4739167Z` | `7E33B1293B839ED74F66CD44714F2C54B37A5AC9C77009BE4F9A6484C2FA70E2` | archive / needs Jon | Windows-side notes; review before promoting durable truth. |
| `C:\Users\jony0\Downloads\garnet-release-readiness-report.pdf` | 572,594 | `2026-05-18T22:05:09.3974520Z` | `041BAAFC44CB077376D41E40530CA5159B70560B0EA990CD65463C0EB0CB6A0F` | archive | PDF report, not source truth unless reconciled with repo. |
| `C:\Users\jony0\Downloads\GARNET_KICKOFF_ADDENDUM_2026_06_10.md` | 9,553 | `2026-06-10T21:47:32.0984529Z` | `BB0FFDED3D370B8B71A0F39BEAE0C7B45A679602280B3D9C862E28F707A5D090` | needs Jon | Fresh handoff/addendum; likely human-authored or external-lane input. |
| `C:\Users\jony0\Downloads\Release readiness screenshot.png` | 73,471 | `2026-05-22T09:33:57.2725903Z` | `EF585B392A26854009480962CE8A0AD704AF02727CF86E666F680E663A00DC64` | duplicate | Same hash as Desktop dogfood screenshot. |
| `C:\Users\jony0\Desktop\dogfood\assets\Release readiness screenshot.png` | 73,471 | `2026-05-22T09:33:57.2725903Z` | `EF585B392A26854009480962CE8A0AD704AF02727CF86E666F680E663A00DC64` | duplicate | Same file as Downloads screenshot. |
| `C:\Users\jony0\Downloads\gpt-image-2_a_hyper_realistic_lush_surreal_raytraced_render_of_a_cinematic_photo_of_a_hyper--2.jpg` | 694,250 | `2026-05-27T10:29:55.7568233Z` | `D650F1F81E8EF2D7A15C72B966A5C8842402784CE1DDFD61412A59ED135AB79A` | needs Jon | Candidate splash/install art only. Not product truth or proof evidence. |
| `C:\Users\jony0\Downloads\gpt-image-2_a_hyper_realistic_lush_surreal_raytraced_render_of_a_cinematic_photo_of_a_hyper--3.jpg` | 788,101 | `2026-05-16T22:54:16.4110416Z` | `8191DEAFE930AADCFD5AB49C8076787C3E976016AEAE5CC45E7345C00681291F` | needs Jon | Candidate splash/install art only. |
| `C:\Users\jony0\Downloads\gpt-image-2_a_hyper_realistic_lush_surreal_raytraced_render_of_a_cinematic_photo_of_a_hyper--4.jpg` | 672,770 | `2026-05-16T22:53:54.7391526Z` | `44CEB139E126BE967532D42CF8E6115E643EB78C7EF5B07C02E1EBB0D9EDDCC7` | needs Jon | Candidate splash/install art only. |

## Preserved paused-lane patches

| Artifact | Size | Modified UTC | SHA256 | Verdict | Safety note |
| --- | ---: | --- | --- | --- | --- |
| `C:\Users\jony0\Documents\Garnet-work-preserve\windows-studio-core-workflow-stale-local.patch` | 67,845 | `2026-06-10T22:44:20.9838118Z` | `D34FA6115589410D4AF4401BEBE2DE608163D4CFDAD265C4F1347A57F4C527D9` | needs Jon | Patch of stale local Windows Studio work from before switching to report lane. Do not apply blindly to current main. |
| `C:\Users\jony0\Documents\Garnet-work-preserve\GARNET_WINDOWS_LINUX_STUDIO_CORE_WORKFLOW_PARITY_2026_06_09.md` | 4,022 | `2026-06-10T03:07:16.6748216Z` | `FBC4951C276FB8055F42C4E1A867A95BCDEDB1F3948E1DB38BD948E8A1C1370A` | needs Jon | Local parity report from paused lane; promote only after reconciling with current S130+ main. |

## Temp and WSL findings

- `$env:TEMP` contains many Garnet test scratch directories from local test runs, including `garnet-memory-*`, `garnet_conformance_*`, `garnet_cache_*`, `garnet_repro_*`, `garnet-convert-test-*`, and `garnet-pwa-browser-*`. Verdict: `ignore` for fleet truth unless a specific failed test needs local forensic review. They are scratch outputs, not release artifacts.
- `wsl.exe -l -q` returned only `docker-desktop`. No user Linux distribution or WSL home checkout was available to inspect from this Surface lane. Verdict: `needs Jon` only if a non-Docker WSL distro is expected.

## Origin/main and Windows NUC correlation

- A fresh `origin/main` worktree at `C:\Users\jony0\Documents\Garnet-surface-fleet-report` contains `721` tracked files under `proofs/`. The local proof files visible in `C:\Users\jony0\Documents\Garnet-windows-studio-pr\proofs\...` are therefore treated as `duplicate` of origin/main unless a later diff proves otherwise.
- The local `target\release` Studio executable, DLLs, PDBs, static libraries, and NSIS installer are not tracked on origin/main. Verdict: `archive`, not commit.
- Desktop dogfood bundles and downloaded PDFs/images/zips are not origin/main source truth. Verdict: `archive` or `needs Jon`, depending on whether the item could support a future narrative or product asset decision.
- The Windows NUC was not directly mounted, queried, or otherwise accessible in this lane. No claim is made that a Surface-local artifact is present on the NUC unless it also appears as committed proof on origin/main.

## Recommended artifact verdicts

| Verdict | Applies to | Action |
| --- | --- | --- |
| `commit` | This report file only | Commit and push on `fleet/2026-06-10-surface-original-tauri-provenance`. |
| `archive` | Local Tauri build outputs, Desktop dogfood bundles, release-readiness PDFs, historical research packages | Preserve locally or in external archival storage; do not commit binaries/bundles. |
| `ignore` | `$env:TEMP` scratch directories from tests | Do not commit; do not delete in this lane. |
| `duplicate` | Release readiness screenshot with SHA256 `EF585B392A26854009480962CE8A0AD704AF02727CF86E666F680E663A00DC64`; tracked proof files already on origin/main | Keep one reference in report; avoid duplicate artifact movement. |
| `unsafe` | Any `source-included` advisory bundle/review/handoff output | Do not publish or provider-forward. Local test evidence only. |
| `needs Jon` | Fresh handoff docs, splash art selection, paused Studio core workflow patches, Windows clean-VM status contract reconciliation | Human decision before promotion into source, docs, website, or release copy. |

## Next safe handoff

1. Use this report as inventory evidence only.
2. Do not push local binaries, zips, screenshots, proof bundles, or Desktop dogfood directories.
3. If Windows Studio work resumes, start from `origin/main`, read the current AGENTS contracts, and re-evaluate the paused `Garnet-windows-studio-pr` diff rather than applying the stale patch wholesale.
4. If clean-VM copy or MIT readiness copy changes are proposed, compare the local Surface clean-VM dogfood bundle against the committed origin/main proof/status reporters first.
