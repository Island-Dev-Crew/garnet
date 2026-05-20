# Changelog

All notable changes to Garnet are recorded here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This file is updated in the same PR as the work it tracks (per the v0.5 slice
contract). Lines added here are part of the calibrated-honesty record — if a
slice ships labeled "partial," its CHANGELOG entry says so explicitly.

## [Unreleased] — v0.5.0 in flight

### Added

- **S0 (housekeeping):** `scripts/garnet_conformance_matrix_check.py` — file-existence
  check on the conformance matrix's evidence column. Advisory by default; `--strict`
  opts into CI-fail behavior. Lands the gate before the existing matrix shorthand
  is repaired so future drift is catchable.
- **S0 (housekeeping):** `--check-no-regression` flag on
  `scripts/garnet_mit_readiness_status.py`. Compares live lane percentages against
  a committed baseline at
  `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json` and exits 1 on any
  drop. Lanes absent from the live output (slice removed/renamed) also trigger
  failure.
- **S0 (housekeeping):** baseline snapshot
  `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json` captured at
  54.2 % overall / 12 lanes from the 2026-05-20 main tip.
- **v0.5 slice contract:** `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` —
  single source of truth for every v0.5 PR. State machine, dogfood blocks,
  honesty anchors, PR template.

### Known-RED gates (inherited, not yet fixed)

- Workspace clippy fails on the in-progress S4 formatter scaffold
  (`garnet-cli/src/formatter.rs`): 7 `clippy::single_char_add_str` errors. S4
  owner repairs these in the S4 PR; not blocking S0.
- Workspace tests have 2 failures in `garnet-cli/src/cmd/fmt.rs` exercising the
  S4 formatter scaffold. Same coordination as above.
- Conformance matrix shorthand: 9 path-like references in
  `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md` do not resolve
  to files on disk today. The new check surfaces these as advisory findings; a
  future slice will fix the matrix and flip the gate to strict.

## Historical record

For the v0.4.2 (research-grade) verification ledger and earlier phase logs, see
`F_Project_Management/GARNET_v4_2_HANDOFF.md` and the dated `GARNET_v*_HANDOFF`
files. Pre-CHANGELOG history was tracked in those handoff documents; from v0.5
onward this file is the canonical entry point.
