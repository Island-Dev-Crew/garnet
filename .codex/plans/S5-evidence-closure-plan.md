# S5 Parser Fuzz Evidence Closure Plan

Source contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md`, section `S5 — Parser Fuzz Harness`.

## Current Truth

- PR #189 merged the core S5 parser fuzz harness, nightly workflow, MIT readiness lane, and baseline regeneration.
- The slice contract still says S5 is `not-started` and shows `cargo fuzz` rather than the Mac-proven `cargo +nightly` dogfood command.
- `scripts/garnet_proof_benchmark_status.py`, named by the slice contract as the S5 reporter, still has no fuzz-harness inventory.
- The fuzz sub-workspace adds `libfuzzer-sys`, so the release record needs a scoped license check instead of relying only on the root workspace deny run.

## Scope

- Preserve PR #189's core harness design.
- Add a proof/benchmark reporter fuzz inventory for the existing `parse_input` target.
- Tighten fuzz manifest/license metadata enough for `cargo deny --manifest-path garnet-parser-v0.3/fuzz/Cargo.toml check`.
- Update the slice contract and project state to reflect merged S5 plus remaining deferred fuzz surfaces.
- Commit a dated S5 local dogfood evidence note and Desktop bundle manifest.

## Non-Claims

- No new parser fuzz targets.
- No interpreter/checker fuzz targets.
- No claim that scheduled one-hour nightly fuzz has accumulated post-merge history.
- No parser correctness proof or mechanized proof.

## Verification

```bash
cd garnet-parser-v0.3
cargo +nightly fuzz run parse_input -- -max_total_time=60
cargo deny --manifest-path garnet-parser-v0.3/fuzz/Cargo.toml check
python3 scripts/test_garnet_proof_benchmark_status.py
python3 scripts/test_garnet_mit_readiness_status.py
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/check-agent-contracts.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
cargo deny check
```

## PR Plan

- Branch: `codex/s5-parser-fuzz-evidence-closure`
- Title: `S5: Parser fuzz evidence closure`
- Base: `Island-Dev-Crew/garnet:main`
- Head: `Navigata1:codex/s5-parser-fuzz-evidence-closure`
