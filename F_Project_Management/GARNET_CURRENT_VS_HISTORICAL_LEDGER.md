# Garnet Current vs Historical Ledger

Date: 2026-05-06

This ledger prevents historical handoffs from being misread as current
implementation truth.

## Reading Rule

When documents conflict, use this order:

1. Live command output from the current checkout.
2. `CURRENT_STATE.md`.
3. `README.md`.
4. `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md`.
5. Current CI workflow definitions.
6. Historical handoffs and archived milestone files.

## Current Proof Surfaces

| Claim surface | Current source | Verification |
|---|---|---|
| Workspace builds/tests | `Cargo.toml`, CI, command output | `cargo test --workspace --no-fail-fast` |
| Canonical app examples | `examples/mvp_*.garnet` | parse/check/run loop and `garnet-cli/tests/examples.rs` |
| First-user templates | `garnet-cli/templates/*` | `garnet new`, `garnet test`, `garnet run` |
| Language completeness | conformance matrix | implemented/partial/deferred rows |
| Release install path | README, installer, release workflow | source fallback works; native release assets require release publication |

## Historical or Descriptive Surfaces

| Surface | How to interpret it |
|---|---|
| `archive/history/_CANONICAL_DELIVERABLES_INDEX.md` | historical bundle index from the research/package era |
| `archive/history/GARNET_BUILD_INSTANTIATION_BRIEF.md` | early build-orchestration plan, not current repo setup |
| `archive/history/GARNET_PHASE7_COMPLETION_REPORT.md` | parser-phase milestone report, not current language-completeness evidence |
| `archive/examples/mvp-design-drafts/` | pre-remediation MVP application drafts, not current runtime proof |
| `F_Project_Management/GARNET_v3_2_HANDOFF.md` | historical snapshot; some example/check claims are superseded by current examples README and tests |
| `F_Project_Management/GARNET_v4_0_HANDOFF.md` | historical milestone; MVP scale claims must be checked against current runnable examples |
| `F_Project_Management/GARNET_v4_2_COMPLETE_PROJECT_STATE.md` | useful consolidated history, but path and release claims require live verification |

## Current Corrections From Dogfood Part 2

- The 10 canonical MVP examples are now compact executable smokes, not the old
  large design drafts.
- The three larger real-world examples remain parser-scale design drafts.
- The `agent-orchestrator` template now uses pure role functions so first run
  succeeds on the current interpreter.
- CI has an explicit canonical MVP example job in addition to the Rust test
  suite.

## Archive Boundary

`archive/` means "kept for audit trail only." Nothing under `archive/` should
be cited as current executable proof unless it is promoted back into the active
source tree with tests.

## Research Layout Note

The project already contains a proposed `research/` reorganization in
`F_Project_Management/GARNET_v4_2_GITHUB_REPO_LAYOUT.md`. That broad path move
should be done as a separate migration with link checking. This remediation pass
adds the current-state guide and historical ledger first so readers can navigate
the existing layout safely.
