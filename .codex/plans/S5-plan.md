# S5 Parser Fuzz Harness Plan

Source contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md`, section `S5 — Parser Fuzz Harness`.

## Current Truth

- S1, S2, and S9 are merged on `origin/main`.
- S9 source/status reporter evidence is merged, but `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` still says S9 is `not-started`; repair that state line as daily truth, with the PR body calling it out as a doc-status correction rather than S5 implementation.
- S5 is `not-started`.
- `cargo fuzz` is not installed on this Mac yet; installing the developer tool is allowed for dogfood, but the repo change must remain reproducible from a clean clone.

## Scope

- Add `garnet-parser-v0.3/fuzz/` as a standard `cargo-fuzz` target package.
- Add a `parse_input` target that feeds UTF-8 Garnet source into `garnet_parser::parse_source_with_budget` with a tight fuzz budget.
- Seed the fuzz corpus from current Garnet examples.
- Add `.github/workflows/fuzz-nightly.yml` with a scheduled one-hour parser fuzz run and crash artifact upload.
- Update `scripts/garnet_proof_benchmark_status.py` and tests so S5 is visible as fuzz-harness evidence without claiming long-running fuzz hours unless evidence exists.
- Update `CHANGELOG.md`, `CURRENT_STATE.md`, and the S5 state in `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md`.
- Commit a dated S5 dogfood evidence note under `F_Project_Management/DOGFOOD/`.

## Non-Claims

- No parser correctness proof.
- No claim that nightly fuzz has already run for one hour until GitHub Actions evidence exists.
- No full adversarial corpus coverage beyond the seeded examples and local dogfood run.
- No S9 implementation changes; only the stale state line may be repaired to match already-merged PR #187.

## Verification

```bash
cd garnet-parser-v0.3
cargo +nightly fuzz run parse_input -- -max_total_time=60
cargo test -p garnet-parser
python3 scripts/test_garnet_proof_benchmark_status.py
python3 scripts/garnet_proof_benchmark_status.py --format json
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
cargo deny check
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/garnet_conformance_matrix_check.py
```

## PR Plan

- Branch: `codex/s5-parser-fuzz-harness`
- Title: `S5: Parser fuzz harness`
- Base: `Island-Dev-Crew/garnet:main`
- Head: `Navigata1:codex/s5-parser-fuzz-harness`
