# Lane 0 trust-kernel review — 2026-07-16

This is the rolling trust-kernel review companion for
[PR #507](https://github.com/Island-Dev-Crew/garnet/pull/507),
`mission/l0-truth-freeze`.

- Integration base: `231aefa91985e5a0520c493c7f0fc3e54d74efc8`.
- Reviewed technical range:
  `231aefa91985e5a0520c493c7f0fc3e54d74efc8..3124ba5ecfa88aa6f2c2c289313860670673cdec`.
- Sealed PR head before this companion:
  `aa14368bde83391506775d835ace8985bb7bc1ed`.
- Trigger: 14 paths in the rolling S114 trust-kernel set.
- Review result: cleared with zero open Critical or Important findings.

This companion records review of the actual changes. It does not bypass or
weaken `scripts/garnet_trust_kernel_review_status.py`, relabel S114
independence, promote any frozen backlog claim, turn launch HOLD into GO, or
authorize a merge. Jon remains the merge authority.

## Independent review lineage

The reviewers below were separate Codex reviewer agents from the implementing
agents. “Independent” here describes the Lane 0 review process; it does not
alter the repository's scoped S114 verdict or Jon-only acceptance authority.
The durable summary is
`ops/lane0/evidence/25-independent-review.md`, sealed by
`ops/lane0/evidence/MANIFEST.sha256`.

| Review scope | Independent reviewer | Reviewed range/head | Reviewed at | Result |
|---|---|---|---|---|
| Archive, U-18, and truth-freeze gate | Codex independent final reviewer | `231aefa..d424a7d` | `2026-07-16T11:31:22Z` | Approved; 0 Critical, 0 Important |
| MSRV, workflow, governance, and WV contracts | Codex independent final reviewer | through `70bfd68` | `2026-07-16T12:34:15Z` | Approved; 0 Critical, 0 Important |
| Frozen backlog and quarterly watch | Codex independent final reviewer | through `9a74521` | `2026-07-16T12:32:30Z` | Approved; 0 Critical, 0 Important |
| Integrated branch review | Codex independent integrated final reviewer | `231aefa..0ba5227` | `2026-07-16T13:31:19Z` | Approved; 0 Critical, 0 Important |
| Final closeout-parser delta | Codex independent final parser reviewer | `0ba5227..3124ba5` | `2026-07-16T13:35:05Z` | Approved; 0 Critical, 0 Important |
| Composite final verdict | Codex composite independent integrated and final-delta review | `231aefa..3124ba5` | `2026-07-16T13:35:05Z` | Approved; 0 Critical, 0 Important |

## The 14 trust-kernel paths

| Path | What changed | Why Lane 0 required it | Independent clearance |
|---|---|---|---|
| `.github/workflows/ci.yml` | Added the MSRV reporter suite and gate plus the WV acceptance suite to the existing agent-contract job. In the existing three-OS cargo-test matrix, Linux installs Rust `1.95.0` and runs the locked all-target/all-feature workspace check before the ordinary stable tests. | End the `1.75+` versus `1.95+` split and enforce one floor without inventing a new required context. | Task 2 reviewer and composite integrated review. |
| `.github/workflows/macos-studio.yml` | The Windows Studio job installs Rust `1.95.0` and checks the locked Tauri backend under that exact toolchain before its existing format, test, web-build, and Playwright steps. | Apply the same MSRV to the active Studio manifest that is excluded from the root workspace. | Task 2 reviewer and composite integrated review; the fresh Windows job passed. |
| `scripts/garnet_frozen_backlog_status.py` | Added a bounded, strict-JSON backlog gate that rejects duplicate keys, unknown IDs/states, invalid partial/planned structures, non-ancestral SHAs, missing code/evidence blobs, unsafe paths, symlinks, and future destinations presented as current evidence. | Freeze only claims tied to current code and evidence; preserve partial/planned states when proof is absent. | Task 3 reviewer and composite integrated review. |
| `scripts/garnet_lane0_closeout_status.py` | Added the fail-closed Lane 0 closeout verifier for the exact 22-file evidence set, 43-command inventory, 37-event hash/chronology/state-bound ledger, four reporter-derived denominators, launch HOLD, band 3, S6 advisory, WV pending semantics, and singular approved review record. | Make runnable evidence semantics—not prose or freshly rehashed false bytes—the final arbiter of Lane 0 completion. | Integrated final reviewer, final parser reviewer, and composite final review. |
| `scripts/garnet_lane0_truth_freeze_status.py` | Added a local-Git gate deriving the 34 first-parent PR/SHA mappings through `1fe7489`, the #499/`231aefa` checkpoint, P7/P7-T1..T4 U-18 integrity, supported statuses, locked P0 commands, and unresolved adversarial status. | Verify the reconciliation archive and prevent the mission state from referring to nonexistent P8/P9/P10 phases. | Task 1 reviewer and composite integrated review. |
| `scripts/garnet_msrv_status.py` | Added a deterministic, standard-library-only gate over all 18 active manifests, the Rust `1.95` public/procedural surfaces, stable tracking, and exact existing-job CI checks. Its conservative workflow projection rejects commented, disabled, wrong-job, ambiguous-YAML, shell/default/working-directory, and toolchain-override false positives. | Settle and enforce one Rust MSRV everywhere it is declared. | Task 2 reviewer and composite integrated review. |
| `scripts/garnet_quarterly_competitive_watch_status.py` | Added a cadence gate for required categories, report naming/completeness/contiguity, due dates, and the “search miss is not evidence of absence” rule. Current truth remains `planned`, zero reports, first due `2026-09-30`. | Activate the standing research slice without falsely claiming that its first quarterly run already happened. | Task 3 reviewer and composite integrated review. |
| `scripts/garnet_wv_acceptance_status.py` | Added bounded, strict, hash-verified WV-6/WV-7 acceptance reporting tied to the exact base, destination, candidate ancestry, required checks, and regular evidence artifacts. Both gates remain honestly pending until their Windows evidence manifests exist. | Freeze WV authority, commands, destinations, and Jon-only boundaries without promoting planned work. | Task 2 reviewer and composite integrated review. |
| `scripts/test_garnet_frozen_backlog_status.py` | Added 16 positive and adversarial tests for backlog state, ancestry, code/evidence provenance, authority anchors, path safety, and future-evidence separation. | Pin the claim-freeze boundary against silent promotion or provenance drift. | Task 3 reviewer and composite integrated review. |
| `scripts/test_garnet_lane0_closeout_status.py` | Added 18 adversarial tests covering false GO/MIT/WV evidence, command truncation, manifest drift, symlinks/traversal, ledger backdating or contradiction, state-binding drift, and contradictory review records. | Demonstrate that a resealed contradiction cannot false-green the closeout gate. | Integrated final reviewer, final parser reviewer, and composite final review. |
| `scripts/test_garnet_lane0_truth_freeze_status.py` | Added 14 archive, state, status, copied-checker, SOTU, corrupt-SHA, stale-phase, and platform-Python-selection regressions. | Pin U-18, Git-derived archive truth, and cross-platform SOTU regeneration. | Task 1 reviewer and composite integrated review. |
| `scripts/test_garnet_msrv_status.py` | Added 25 mutations proving that comments, disabled/wrong jobs, ambiguous YAML, unsafe workflow execution overrides, unlisted manifests, or stale public surfaces cannot satisfy the MSRV gate. | Prove Rust `1.95` is structurally enforced rather than merely mentioned. | Task 2 reviewer and composite integrated review. |
| `scripts/test_garnet_quarterly_competitive_watch_status.py` | Added six tests for honest planned state, overdue behavior, incomplete placeholders, required corpus/directive structure, and byte-preserved June-source migration. | Prevent an activated cadence from being mistaken for a completed research run. | Task 3 reviewer and composite integrated review. |
| `scripts/test_garnet_wv_acceptance_status.py` | Added six tests preserving WV meanings, the current pending-red state, strict candidate/check/hash binding, malformed partial evidence, and a complete positive fixture. | Keep WV-6/WV-7 fail closed until exact-candidate evidence exists. | Task 2 reviewer and composite integrated review. |

## Fresh evidence

### Tracked Lane 0 evidence files

These files were captured on 2026-07-16 and are bound to the reviewed
candidate by `ops/lane0/evidence/COMMANDS.json` and the exact
`ops/lane0/evidence/MANIFEST.sha256`:

- `ops/lane0/evidence/00-environment.json` — Darwin arm64 host and exact tool
  versions. This is explicitly local evidence, not a cross-OS claim.
- `ops/lane0/evidence/05-msrv.json` — reporter `ok: true`, all 18 active
  manifests accounted for, exact workflow contexts present, and moving stable
  preserved.
- `ops/lane0/evidence/20-python-tests.txt` — the reporter and adversarial
  suites, including 16 backlog, 18 closeout, 14 truth-freeze, 25 MSRV, six
  quarterly-watch, and six WV tests.
- `ops/lane0/evidence/21-rust-msrv-checks.txt` — exact Rust `1.95.0` locked
  workspace check/test and excluded Studio check.
- `ops/lane0/evidence/22-workspace-tests.txt` — formatting, template tests,
  and full current-stable workspace tests.
- `ops/lane0/evidence/25-independent-review.md` — the approved review ranges,
  roles, times, and zero-finding counts.
- `ops/lane0/evidence/11-truth-check.txt` and
  `ops/lane0/evidence/12-repository-evidence-integrity.json` — machine truth
  green and 38/38 evidence bundles verified.
- `ops/lane0/evidence/13-wv6-pending.json` and
  `ops/lane0/evidence/14-wv7-pending.json` — fresh expected-red acceptance
  results. They prove the contracts remain pending; they do not claim Windows
  acceptance.
- `ops/lane0/evidence/COMMANDS.json` — 43/43 commands with expected and actual
  exits and explicit candidate binding.
- `ops/lane0/evidence/MANIFEST.sha256` — exact sorted 22/22 evidence coverage.

### Fresh cross-OS reproduction for PR #507

The fresh cross-OS results are GitHub Actions logs bound to PR head
`aa14368bde83391506775d835ace8985bb7bc1ed`. They are listed separately from
the tracked Darwin files so the evidence boundary stays explicit:

- [Windows Studio build + test](https://github.com/Island-Dev-Crew/garnet/actions/runs/29503635271/job/87638647792)
  — passed, including exact Rust `1.95.0`, locked Tauri backend check, backend
  tests, web build, and Playwright.
- [macOS Studio build + test](https://github.com/Island-Dev-Crew/garnet/actions/runs/29503635271/job/87638647865)
  — passed Swift build/test and shell contracts.
- [Ubuntu deterministic build](https://github.com/Island-Dev-Crew/garnet/actions/runs/29503634784/job/87639203511)
  — passed.
- [macOS deterministic build](https://github.com/Island-Dev-Crew/garnet/actions/runs/29503634784/job/87639203443)
  — passed.
- [Cross-OS determinism comparison](https://github.com/Island-Dev-Crew/garnet/actions/runs/29503634784/job/87639883745)
  — passed the Ubuntu/macOS hash comparison.
- [Ubuntu agent-contract job](https://github.com/Island-Dev-Crew/garnet/actions/runs/29503637598/job/87638656003)
  — the new MSRV tests, MSRV gate, and WV tests passed before the rolling
  trust-kernel gate correctly stopped the job for this missing companion.

The determinism run produced two fresh evidence files:

- Actions artifact
  `determinism-hash-ubuntu-latest/manifest-hash-ubuntu-latest.txt`
- Actions artifact
  `determinism-hash-macos-latest/manifest-hash-macos-latest.txt`

Both artifact files have SHA-256
`1ad9d1531de7819ba2532ea28aea121b2e82fe1ec8801290493a7cd7d0581f12`
and both contain the same manifest hash
`a6e51071d99b3b87740c2a6f95d727952b44dce60101f64c726b2db242680484`.

The main three-OS cargo-test matrix did not run on that head because its
upstream agent-contract dependency was correctly red. It must rerun after this
companion is pushed; this document does not pre-claim that result.

Pre-existing durable cross-OS trust evidence remains available at:

- `proofs/windows/enforcement/windows-enforcement-proof.json`
- `proofs/mac/matrix/mac-cross-os-matrix-20260604-201153/garnet-mac-cross-os-matrix.json`
- `proofs/linux/enforcement/utm-linux-enforcement-20260604-s108/garnet-linux-cross-os-enforcement-proof.json`
- `proofs/cross-os/matrix/cross-os-trap-parity-20260604-s109/garnet-cross-os-trap-parity-matrix.json`
- `proofs/linux/studio/utm-native-20260701/garnet-studio-native-linux.json`
- `F_Project_Management/VALIDATION_REPORTS/2026-06-28_windows_codex_lane2_s114_review.md`
- `proofs/independent/s114/codex-verdict-20260625/`

Those older bundles are baseline context, not substitutes for the fresh PR
jobs above.

## Review conclusion and required rerun

The 14 changes are bounded governance, evidence, and CI-enforcement changes.
They do not add language/runtime authority, weaken a denial, expand a public
enforcement claim, or perform a Jon-only action. The independent reviews
cleared the reviewed technical range with zero open Critical or Important
findings.

Required local and remote checks after adding this companion:

```text
python3 scripts/test_garnet_trust_kernel_review_status.py
python3 scripts/garnet_trust_kernel_review_status.py --gate --format json
python3 scripts/check-agent-contracts.py
python3 scripts/test_check_agent_contracts.py
python3 -I -S scripts/garnet_lane0_closeout_status.py --gate
cargo run -q -p xtask -- truth --check
```

The PR remains launch HOLD and Jon-merged only.
