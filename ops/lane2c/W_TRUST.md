# Lane 2C W_TRUST Companion

This is an implementer provenance companion, not a review verdict or
acceptance record.

## Seats

- Implementer: OpenAI Codex, GPT-5-based model. The harness exposed no exact
  submodel or build version, so none is asserted.
- Implementer machine: `NUCBOX_M2PRO_S`, GMKtec NucBox_M2Pro_S, Windows 11 Pro
  `10.0.26200`; measurement guest Ubuntu WSL2, Linux
  `6.6.87.2-microsoft-standard-WSL2`, x86_64.
- Independent reviewer: not yet occupied. The required seat is a different
  model family on the MacBook Air. The reviewer must disclose the actual
  model/version and machine in `ops/lane2c/review/01-verdict.md`.
- Review carrier: IDC-Trust-Review only.
- Merge authority: Jon (IslandDevCrew) only.

No implementer-authored file in this branch is a verdict. The implementer must
not write `ops/lane2c/review/01-verdict.md`.

## Immutable product boundary

- Base: `efd4f6bae8b3afaba74594e57944b2548142aeae`
- Repair commit: `0649d796ac6b78b968d868398b517974838112f3`
- Product head: `5cd113617acd35307bb028463833a8da2bbd6ad2`
- Product tree: `85faad1de5a2c47cb632bedea78dfb89d209001a`
- Product diff: three files, 195 insertions, 5 deletions
- Product paths:
  - `garnet-memory-v0.3/src/cycle.rs`
  - `garnet-memory-v0.3/examples/lane2c_teardown_probe.rs`
  - `garnet-memory-v0.3/AGENTS.md`

All commits after the product head are restricted to `ops/lane2c/**` records.
They do not extend the product boundary or backdate review.

## Measurement boundary

- Counter: Callgrind `Ir`, not wall-clock duration.
- Sizes: 256, 512, 1,024.
- Cases: working clear, episodic drop, semantic drop.
- Filesystem: `/dev/sdd` ext4 inside WSL2; no `/mnt/c` measurement.
- Quiet window: `2026-07-29T08:34:09.8071444Z` through
  `2026-07-29T08:35:39.5943570Z`.
- Quiet state: OneDrive and its sync helper stopped; Ubuntu cron stopped; no
  concurrent build, sync, measurement, or spawned agent.
- Restore: cron and OneDrive active by `2026-07-29T08:35:50.7129927Z`; sync
  helper active by `2026-07-29T08:36:10.1339188Z`.
- WSLg: not used.
- Harness source SHA-256:
  `7baed24c356c262aa3d19388e6a9a72117bcb9aaeecd61879b05eccca0871040`
- Base probe binary SHA-256:
  `4577447bdfba5163467c48fc59d6444688a094c52df7a9360ffbeaa9f3f00a72`
- Product probe binary SHA-256:
  `0ca1e4e38471ba34ffe51274216a6de144910fb5d0c791a40be7a012bcdb9810`
- Root lockfile SHA-256 before and after:
  `01b8986b1cee0ef6a53ac439bd018b54fc1dca825a8f845a259ed8001e6715fa`
- Active manifests under `ops/lane2c/`: zero.

The evidence-local verifier must pass before review:

```sh
python3 -I ops/lane2c/verify_evidence.py --gate
```

## Authority boundary

The implementer records only the repair, commands, raw profiles, doctrine,
and test results. The independent reviewer decides whether the product and
evidence support closure. Jon alone decides merge. No tag, release, launch
promotion, acceptance decision, or global U-46 registration is authorized by
this companion.
