# MacBook Pro Codex Verification Report - 2026-06-11

Lane: MacBook Pro Codex verification, report-only.
Branch: `aux/2026-06-11-macbook-pro-codex-verification`.
Base verified before report write: `origin/main @ 8482294e76a392d78f50162a632c484e3006184d`.
Local worktree status before report write: `## aux/2026-06-11-macbook-pro-codex-verification...origin/main`.
Hard stops honored: no merges, no tags, no releases, no branch deletion, no cherry-picks, no CI/gate edits, no ECC hooks, no edits outside this report file.

## Executive Verdict

GATE: MET for W-REBUILD deploy pre-check.

- Core fleet reports are present on main: MacBook Pro Claude, MacBook Pro Codex, Windows NUC Claude, Windows NUC Codex, MacBook Air Claude, Surface original-Tauri provenance, plus `TEMPLATE.md`.
- Consolidation and reassessment are merged: PR #382 (`1cfadda`, merged 2026-06-11T17:16:40Z) and PR #383 (`0cef67b`, merged 2026-06-11T17:27:13Z). Current main also includes RB-0a PR #384 (`3ccfd38`) and RB-0b PR #385 (`8482294`).
- Clean-tree start is reproducible: this verification ran from a separate worktree fast-forwarded to current `origin/main @ 8482294e76a392d78f50162a632c484e3006184d` before writing this report.

Findings ordered by severity:

1. LOW / timing delta: PR-A's drift map is historically reproducible at the commit it names (`main @ 366e69f`, see `F_Project_Management/GARNET_S131_S134_SOURCE_TRUTH_CONSOLIDATION.md:80-87`). On current `HEAD @ 8482294e76a392d78f50162a632c484e3006184d`, RB-0a/RB-0b have intentionally fixed part of that drift, especially the root README primitive/version rows. Current `HEAD` still reproduces the remaining stale FAQ, VSIX, CURRENT_STATE, and blog strings listed below. This is not a PR-A defect; it is an expected post-merge state change.
2. LOW / watch item: `F_Project_Management/W_REBUILD/README_PROPOSED.md` is now explicitly marked historical (`README_PROPOSED.md:1-5`). The authoritative root `README.md` landed in PR #385 and has tighter scoped wording, including the principle at `README.md:10` and honest status at `README.md:129-146`.
3. No blocker found in PR-B: the reassessment infusion carries the required principle, RB-1 machine verdict, RB-3 Core Ring, RB-4 typed-core-IR caps re-check and editions note, RB-6 linker doctrine plus measured 2-3x reopen threshold, and governance/FCP stub.
4. No calibrated-honesty leakage found in the checked surfaces: S114 remains self-verified; no production/1.0 claim was introduced; ECC language did not displace Garnet dogfood/readiness authority; `enforced` is limited to the deterministic trap-backed `@caps` + `@max_depth` kernel claim.

## Recon Snapshot

Commands run from `/Users/IDC2.5/Desktop/garnet-codex-verify` unless noted.

- `git fetch origin main --tags --prune`; fetched `fleet/*` and `aux/*` refs.
- `gh pr list --repo Island-Dev-Crew/garnet --state all --limit 10` observed PRs #380-#385, with #382, #383, #384, and #385 merged and no open PRs after #385.
- `python3 scripts/garnet_readiness_status.py --format json` -> 87/87 tracked slices, 100.0%.
- `python3 scripts/garnet_mit_readiness_status.py --format json` -> `active-partial`, 92.8%.
- `git rev-parse HEAD` == `git rev-parse origin/main` == `8482294e76a392d78f50162a632c484e3006184d` before report write.

## Task 1 - PR-A / PR-B Verification

### PR-A: S131-S134 source-truth consolidation (#382)

Verified. The consolidation document is scoped as docs/report-only and explicitly excludes code, CI/gate/threshold, release/tag/asset, README/site live edits, crate renames, and ECC hooks (`F_Project_Management/GARNET_S131_S134_SOURCE_TRUTH_CONSOLIDATION.md:3-6`).

Fleet report delivery is backed by the PR-A gate table (`...SOURCE_TRUTH_CONSOLIDATION.md:16-29`). The transport gap is root-caused rather than hidden (`...:31-44`). The converged release/repo truth separates signed assets from unsigned tag, Windows release absence, WSL-only Linux portability, and enforcement boundaries (`...:46-76`).

Artifact verdict backing is present in the underlying reports, not invented in the roll-up:

- MacBook Pro Claude report defines the verdict taxonomy and records concrete needs-Jon/archive/duplicate/ignore verdicts (`F_Project_Management/FLEET_REPORTS/2026-06-10_macbook-pro_claude-fable.md:129-221`).
- MacBook Pro Codex report records the W-REBUILD pack, v0.8.1/tag truth, and report verdict (`F_Project_Management/FLEET_REPORTS/2026-06-10_macbook-pro_codex.md:16-23`, `...:97`, `...:150-188`, `...:317-340`).
- Windows NUC Claude report separates committed/released/machine-local/not-proven evidence and marks Windows/Tauri/Linux package state honestly (`F_Project_Management/FLEET_REPORTS/2026-06-10_windows-nuc_claude-fable.md:10-11`, `...:64-71`, `...:121-153`, `...:172-181`).
- Windows NUC Codex report explicitly says the local checkout is 76 commits behind and cannot prove current v0.8.1 Windows readiness (`F_Project_Management/FLEET_REPORTS/2026-06-10_windows-nuc_codex.md:9`, `...:93-108`, `...:186-223`).
- MacBook Air Claude report records tag drift, 25 safe-delete branches, s18/s19 review, local artifacts, and the formerly missing command-center file (`F_Project_Management/FLEET_REPORTS/2026-06-10_macbook-air_claude-fable.md:16-25`, `...:113-121`).
- Surface report records original-Tauri provenance, local build outputs as archive, source-included bundles as unsafe, scratch dirs as ignore, and recommended artifact verdicts (`F_Project_Management/FLEET_REPORTS/2026-06-10_surface_original-tauri-provenance.md:32-38`, `...:54-60`, `...:68-82`, `...:102`, `...:137-143`).

Historical drift reproduction:

- At `366e69f`, `git grep` reproduced the stale `24` stdlib rows, `post-v0.5.0`, `v0.5.0 is research-grade`, `S1 LSP`, `0.7.0-lsp`, and `monthly is the floor` strings called out by PR-A.
- At current `HEAD @ 8482294e76a392d78f50162a632c484e3006184d`, the root README primitive rows are no longer stale after RB-0a/RB-0b. Remaining current hits are:
  - `CURRENT_STATE.md:120` -> `garnet-0.7.0-lsp-precision.vsix`.
  - `FAQ.md:55` -> `v0.5.0 is research-grade and not production-complete`.
  - `FAQ.md:57` -> current primitive marker is 80, but the line still carries `S1 LSP source surfaces` and `v0.5.0 Linux/macOS CLI release assets`.
  - `docs/blog/index.html:80` -> `monthly is the floor`.
  - `editors/vscode/README.md:3` -> `S1 LSP MVP extension`.
  - `editors/vscode/package.json:32` -> `garnet-0.7.0-lsp-precision.vsix`.

### PR-B: reassessment infusion (#383)

Verified. Required infusion points are present:

- One-sentence principle at W-REBUILD spec top: `F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md:1-4`.
- Historical README proposal now marked landed/historical: `F_Project_Management/W_REBUILD/README_PROPOSED.md:1-5`; root README authoritative after PR #385 with principle at `README.md:10`.
- RB-1 structured `--machine` verdict: `F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md:193-204`.
- RB-3 names Core Ring and defers it to W-SHIP/W-LAUNCH: `F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md:217-235`.
- RB-4 typed core IR carries capabilities and re-checks after every lowering pass; editions are the surface-collapse vehicle: `F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md:237-258`.
- RB-6 memo carries Stroustrup linker doctrine, per-pass caps re-check as a hard backend constraint, and an integrate-lean recommendation with measured 2-3x reopen threshold: `F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md:271-288`.
- Governance stub exists, records honest single-maintainer status, FCP, edition invariants, and no false steering body: `GOVERNANCE.md:1-6`, `GOVERNANCE.md:35-52`, `GOVERNANCE.md:67-72`.
- Source reassessment file supports the infusion path: `F_Project_Management/RESEARCH/GARNET_REASSESSMENT_2026-06-11.md:157-165`, `...:173-192`, `...:196-203`.

No evidence found that PR-B self-grades S114 as independent, widens enforcement beyond trap-backed claims, claims production/v1.0, or lets ECC become authoritative over Garnet dogfood/readiness.

## Task 2 - W-REBUILD Deploy-Gate Pre-Check

GATE: MET.

| Condition | Evidence | Verdict |
|---|---|---:|
| Core fleet reports exist on main | `F_Project_Management/FLEET_REPORTS/` contains MBP Claude, MBP Codex, Windows NUC Claude, Windows NUC Codex, MacBook Air Claude, Surface, and template. PR-A gate table records same (`...SOURCE_TRUTH_CONSOLIDATION.md:18-29`). | MET |
| Consolidation + infusion merged | PR #382 merged 2026-06-11T17:16:40Z; PR #383 merged 2026-06-11T17:27:13Z. Latest PR list also shows #384 and #385 merged after them. | MET |
| Clean-tree start possible | Separate worktree `/Users/IDC2.5/Desktop/garnet-codex-verify` was fast-forwarded to `origin/main @ 8482294e76a392d78f50162a632c484e3006184d` and clean before writing this report. | MET |

The deploy gate being MET does not itself authorize tags, releases, destructive branch cleanup, CI/gate edits, ECC hooks, or semantic changes. It only means a lead W-REBUILD lane can start from current `origin/main` without building on unreconciled fleet drift.

## Task 3 - Branch-Hygiene Verdict Table

Scope: the original MBP consolidation snapshot: 76 patch-equivalent local branches plus 12 `git cherry` positive branches. Jon executes any cleanup; this lane deleted nothing and cherry-picked nothing.

Base command family: `git log --format=%h origin/main..<branch>` and `git cherry -v origin/main <branch>` against `origin/main @ 8482294e76a392d78f50162a632c484e3006184d`.

Counts:

- Candidate branches in the requested table: 88.
- `safe-delete`: 76. These have no `+` patch IDs from `git cherry` versus current `origin/main`.
- `cherry-pick-first`: 12. These have at least one `+` patch ID and need review before deletion.
- Non-candidates intentionally excluded: `main`, this report branch, the just-merged `codex/rb0b-readme-replacement`, and new post-snapshot local branch `codex/rb0c-version-narrative` (zero ahead at audit time; not part of the 88 promised set).

| Branch | Verdict | `git log --format=%h origin/main..<branch>` | `git cherry -v origin/main <branch>` |
|---|---:|---:|---:|
| `agent-mac-opus/s15-compare-addendum` | `cherry-pick-first` | `edf576b`<br>`7f01b62` | +1/-1: `-7f01b62e4b96`<br>`+edf576b04835` |
| `agent-mac-opus/s15-cst-rowan` | `cherry-pick-first` | `64391d1`<br>`a8efe2f`<br>`0452fc4` | +3/-0: `+0452fc44c0e1`<br>`+a8efe2f58278`<br>`+64391d1a1dc0` |
| `agent-mac-opus/s15-cst-trait-stub` | `cherry-pick-first` | `755371a`<br>`edc5f88`<br>`af5200c` | +3/-0: `+af5200c61fea`<br>`+edc5f8824d5f`<br>`+755371aa9d39` |
| `chore/tie-up-parked-docs` | `safe-delete` | `81ab860` | +0/-1: `-81ab860d6578` |
| `codex/handoff-2026-05-20` | `safe-delete` | `4d6b159` | +0/-1: `-4d6b159cb297` |
| `codex/p0-windows-audit` | `safe-delete` | `545b7e8` | +0/-1: `-545b7e833043` |
| `codex/s0-housekeeping` | `cherry-pick-first` | `6a70147`<br>`fa0bbe6`<br>`a0662bd` | +2/-1: `-a0662bd90ae0`<br>`+fa0bbe65e897`<br>`+6a7014790b15` |
| `codex/s1-lsp-mvp-archive-claude` | `cherry-pick-first` | `602b56f` | +1/-0: `+602b56fd53e2` |
| `codex/s107-mac-domain-reattest` | `safe-delete` | `7f0fadc` | +0/-1: `-7f0fadc13297` |
| `codex/s107-mac-native-domain-studio-proof` | `safe-delete` | `fce3825` | +0/-1: `-fce38255db7d` |
| `codex/s107-mac-studio-ui-reattest` | `safe-delete` | `3c84e5d` | +0/-1: `-3c84e5d9f993` |
| `codex/s108-linux-utm-enforcement-proof` | `cherry-pick-first` | `9b85696`<br>`bf93688` | +2/-0: `+bf936884db6a`<br>`+9b85696ed2df` |
| `codex/s109-full-cross-os-closeout` | `safe-delete` | `cc8182b` | +0/-1: `-cc8182b6664b` |
| `codex/s109-full-cross-os-consolidation` | `safe-delete` | `a6fb9c0` | +0/-1: `-a6fb9c0d5d1a` |
| `codex/s109-mac-cross-os-matrix` | `cherry-pick-first` | `85436a4`<br>`c14846c` | +2/-0: `+c14846c231a6`<br>`+85436a46bd14` |
| `codex/s109-merge-ledger-closeout` | `safe-delete` | `2f3bb23` | +0/-1: `-2f3bb2353944` |
| `codex/s11-v0-6-scaffold` | `safe-delete` | `94600f2` | +0/-1: `-94600f28b052` |
| `codex/s12-resolver-contract` | `safe-delete` | `bf3bcdc` | +0/-1: `-bf3bcdcba510` |
| `codex/s13-registry-stub` | `safe-delete` | `7a6a413` | +0/-1: `-7a6a413edd05` |
| `codex/s14-vm-function-call-lowering` | `safe-delete` | `3d8fdab` | +0/-1: `-3d8fdab1586b` |
| `codex/s15-cst-compare-rowan-canonical` | `safe-delete` | `d5bbbf5`<br>`a88d220` | +0/-2: `-a88d22015303`<br>`-d5bbbf589779` |
| `codex/s3-garnet-add` | `safe-delete` | `d90148c` | +0/-1: `-d90148c6cd5f` |
| `codex/s31-pr2-reporter-determinism` | `safe-delete` | `76cd242` | +0/-1: `-76cd24269b6c` |
| `codex/s31-v0-8-foundation` | `safe-delete` | `fc01582` | +0/-1: `-fc01582d6b94` |
| `codex/s32-editions-compat` | `cherry-pick-first` | `dd48cd4`<br>`190e8e6` | +2/-0: `+190e8e6fedc2`<br>`+dd48cd4e3c3e` |
| `codex/s33-garnet-verify` | `safe-delete` | `fa49128` | +0/-1: `-fa49128672e1` |
| `codex/s34-diagnostics` | `safe-delete` | `410c761` | +0/-1: `-410c7614a062` |
| `codex/s35-caps-surface` | `safe-delete` | `586b9e5` | +0/-1: `-586b9e58ae1f` |
| `codex/s36-capability-manifest` | `safe-delete` | `963ff3c` | +0/-1: `-963ff3c782a1` |
| `codex/s37-diff-caps` | `safe-delete` | `0d5a516` | +0/-1: `-0d5a51622e27` |
| `codex/s38-seal` | `safe-delete` | `6264cc7` | +0/-1: `-6264cc73ad34` |
| `codex/s39-bounded` | `safe-delete` | `dc22eeb` | +0/-1: `-dc22eeb47a57` |
| `codex/s4-fmt-idempotency` | `safe-delete` | `a8829e2` | +0/-1: `-a8829e26e663` |
| `codex/s40-explosive-ops` | `safe-delete` | `7fe983e` | +0/-1: `-7fe983ecb4ac` |
| `codex/s41-async-contract` | `safe-delete` | `f4f4696` | +0/-1: `-f4f46960ee4a` |
| `codex/s42-error-policy` | `cherry-pick-first` | `34491c2`<br>`b34bd08` | +2/-0: `+b34bd08844e2`<br>`+34491c2be3bf` |
| `codex/s43-doctest` | `cherry-pick-first` | `63269ec`<br>`b635814` | +2/-0: `+b635814c5e7f`<br>`+63269ecc984b` |
| `codex/s44-lsp-precision` | `safe-delete` | `05bad0e` | +0/-1: `-05bad0e99dc6` |
| `codex/s45-slopguard` | `safe-delete` | `200d863` | +0/-1: `-200d863facb3` |
| `codex/s46-sandbox` | `safe-delete` | `d0b99eb` | +0/-1: `-d0b99eb341a6` |
| `codex/s47-build-proof` | `safe-delete` | `f1c19f4` | +0/-1: `-f1c19f43405f` |
| `codex/s48-proof-matrix` | `safe-delete` | `f5d724b` | +0/-1: `-f5d724b2e28f` |
| `codex/s49-wedge` | `safe-delete` | `e76bb26` | +0/-1: `-e76bb262ac69` |
| `codex/s5-parser-fuzz` | `cherry-pick-first` | `ed97a25`<br>`a75a9e7` | +2/-0: `+a75a9e714063`<br>`+ed97a25c95dd` |
| `codex/s50-beta-gate` | `safe-delete` | `678f3a9` | +0/-1: `-678f3a95f165` |
| `codex/s51-signed-release` | `safe-delete` | `2807250` | +0/-1: `-2807250bead3` |
| `codex/s52-install-check` | `safe-delete` | `27674e6` | +0/-1: `-27674e6b6c8f` |
| `codex/s53-tree-sitter` | `safe-delete` | `6715969` | +0/-1: `-671596945ddd` |
| `codex/s54-vscode-publish` | `safe-delete` | `9bed836` | +0/-1: `-9bed8364b5fd` |
| `codex/s55-wasm` | `safe-delete` | `0cb31a9` | +0/-1: `-0cb31a9967c4` |
| `codex/s56-playground` | `safe-delete` | `b8bc42f` | +0/-1: `-b8bc42fdc366` |
| `codex/s57-corpus` | `safe-delete` | `a1d124f` | +0/-1: `-a1d124f485a1` |
| `codex/s58-benchmark` | `safe-delete` | `e94dea0` | +0/-1: `-e94dea03e6cd` |
| `codex/s59-fuzz` | `safe-delete` | `729de8b` | +0/-1: `-729de8b1036b` |
| `codex/s6-memory-eviction-benches` | `safe-delete` | `af4a774` | +0/-1: `-af4a7747689d` |
| `codex/s60-release-readiness` | `cherry-pick-first` | `ca92ff4`<br>`405a7e6` | +2/-0: `+405a7e6bdac3`<br>`+ca92ff409f7b` |
| `codex/s61-ffi-authority` | `safe-delete` | `a9de2be` | +0/-1: `-a9de2be4be3b` |
| `codex/s62-rust-ffi` | `safe-delete` | `0ef6fdd` | +0/-1: `-0ef6fddc246c` |
| `codex/s63-c-abi` | `safe-delete` | `688b86a` | +0/-1: `-688b86a842f5` |
| `codex/s64-wasi` | `safe-delete` | `0850a8a` | +0/-1: `-0850a8aa6d3d` |
| `codex/s65-ai-provenance` | `safe-delete` | `a4285b2` | +0/-1: `-a4285b22fa59` |
| `codex/s66-attestation` | `safe-delete` | `911146d` | +0/-1: `-911146d2a376` |
| `codex/s67-mcp-caps` | `safe-delete` | `d4693dd` | +0/-1: `-d4693dd0a9f3` |
| `codex/s68-transparency-log` | `safe-delete` | `04d6f4c` | +0/-1: `-04d6f4c0ed11` |
| `codex/s69-llm-suggest` | `safe-delete` | `ef897c7` | +0/-1: `-ef897c767e5f` |
| `codex/s7-actor-trust-report` | `safe-delete` | `ce7d23a` | +0/-1: `-ce7d23aa99d1` |
| `codex/s70-version-map` | `safe-delete` | `01e03de` | +0/-1: `-01e03dede0eb` |
| `codex/s71-paper-vi-exp3` | `safe-delete` | `512c7f5` | +0/-1: `-512c7f583d9d` |
| `codex/s72-self-hosted-parser` | `safe-delete` | `0bb1246` | +0/-1: `-0bb124666361` |
| `codex/s73-vm-parity` | `safe-delete` | `f4ff6f2` | +0/-1: `-f4ff6f27c82f` |
| `codex/s74-safe-subset` | `safe-delete` | `7ef56c1` | +0/-1: `-7ef56c1fc57b` |
| `codex/s75-formal-verification` | `safe-delete` | `1588a69` | +0/-1: `-1588a699543c` |
| `codex/s76-stdlib-promotion` | `safe-delete` | `146af53` | +0/-1: `-146af5313f63` |
| `codex/s77-external-packages` | `safe-delete` | `7cf747a` | +0/-1: `-7cf747a845a4` |
| `codex/s78-governance-rfc` | `safe-delete` | `4cc33d3` | +0/-1: `-4cc33d3410f7` |
| `codex/s79-website-reframe` | `safe-delete` | `2b3971a` | +0/-1: `-2b3971a1530f` |
| `codex/s8-hotreload-demo` | `safe-delete` | `5637ca4` | +0/-1: `-5637ca4ba4c4` |
| `codex/s80-v0-8-0-cut` | `safe-delete` | `147faf0` | +0/-1: `-147faf02ddf0` |
| `codex/s81-garnet-case` | `safe-delete` | `afd3237` | +0/-1: `-afd32375f8c0` |
| `codex/s82-seal-determinism` | `safe-delete` | `456ac5f` | +0/-1: `-456ac5ff2b13` |
| `codex/s83-release-truth` | `safe-delete` | `8fb6a7b` | +0/-1: `-8fb6a7b933f4` |
| `codex/s85-interp-stack` | `safe-delete` | `8b95065` | +0/-1: `-8b9506507a4e` |
| `codex/s89-bounded-enforce` | `safe-delete` | `8247f72` | +0/-1: `-8247f72cad1c` |
| `codex/s9-determinism-ci` | `safe-delete` | `a9bf7d8` | +0/-1: `-a9bf7d8a3374` |
| `codex/s90-caps-enforce` | `safe-delete` | `5a16035` | +0/-1: `-5a16035648b4` |
| `codex/v0-5-release-blog` | `safe-delete` | `1e5cb6d` | +0/-1: `-1e5cb6d2e4f6` |
| `docs/v0.7-coordination-bundle` | `safe-delete` | `800b4b3` | +0/-1: `-800b4b3c18d2` |
| `docs/v0.7-cst-build-compare-reconcile` | `safe-delete` | `0778d05` | +0/-1: `-0778d05bded4` |

## Stop State

Report file only was created by this lane. Push confirmation is recorded outside this file after commit/push by the transport rule.
