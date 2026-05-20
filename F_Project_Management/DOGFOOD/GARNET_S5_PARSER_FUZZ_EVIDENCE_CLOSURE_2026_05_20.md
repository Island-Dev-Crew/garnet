# Garnet S5 Parser Fuzz Evidence Closure

Date: 2026-05-20
Branch: `codex/s5-parser-fuzz-evidence-closure`
Base: `d12b4ad3a6de46c1aa348fae1b7bc073894d2520`
Slice: S5 — Parser Fuzz Harness

## Current Truth

- PR #189 merged the core S5 parser fuzz harness and the verified MIT readiness lane.
- This closure records the Mac-side 60-second cargo-fuzz dogfood run against that merged harness.
- The proof/benchmark reporter now inventories the parser fuzz harness named by the S5 contract.
- The fuzz sub-workspace has explicit package license metadata and a scoped cargo-deny check for `libfuzzer-sys`.
- The PR fuzz workflow installs the pinned `cargo-fuzz` release without its stale published lockfile, after CI showed that lockfile no longer builds on the current runner toolchain.
- Scheduled one-hour nightly fuzz history is still not claimed until GitHub Actions evidence accumulates after merge.
- Parser correctness, mechanized proof, interpreter/checker fuzzing, and OSS-Fuzz remain out of scope.

## Dogfood Command

```bash
cd garnet-parser-v0.3
cargo +nightly fuzz run parse_input -- -max_total_time=60
```

## Dogfood Output Summary

```text
8 files found in garnet-parser-v0.3/fuzz/corpus/parse_input
seed corpus: files: 8 min: 616b max: 9921b total: 22405b rss: 36Mb
DONE   cov: 2785 ft: 12262 corp: 2900/1094Kb lim: 8700 exec/s: 65746 rss: 493Mb
Done 4010506 runs in 61 second(s)
```

Result: exit 0, no panic, no hang, no crash artifact.

## Status Reporter Delta

```text
Before: scripts/garnet_proof_benchmark_status.py tracked 4 Criterion harnesses, 0 fuzz harnesses.
After:  scripts/garnet_proof_benchmark_status.py tracks 4 Criterion harnesses, 1 fuzz harness.

Before: scripts/garnet_mit_readiness_status.py overall = 63.7%.
After:  scripts/garnet_mit_readiness_status.py overall = 63.7%.
```

Overall MIT/productization percent does not move in this closure; S5's verified lane already landed in PR #189. This PR makes the evidence breakdown more accurate without widening readiness claims.

## Local Verification

- [x] `cargo +nightly fuzz run parse_input -- -max_total_time=60`
- [x] `(cd garnet-parser-v0.3/fuzz && cargo +nightly fuzz run parse_input -- -max_total_time=1)`
- [x] `CARGO_HOME=/tmp/garnet-cargo-fuzz-ci-install-home CARGO_TARGET_DIR=/tmp/garnet-cargo-fuzz-ci-install-target cargo +nightly install cargo-fuzz --version 0.13.1 --root /tmp/garnet-cargo-fuzz-ci-install-root`
- [x] `cargo deny --manifest-path garnet-parser-v0.3/fuzz/Cargo.toml check`
- [x] `cargo fmt --all -- --check`
- [x] `cargo fmt --manifest-path garnet-parser-v0.3/fuzz/Cargo.toml -- --check`
- [x] `ruby -e "require 'yaml'; YAML.load_file('.github/workflows/fuzz-nightly.yml')"`
- [x] `PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_garnet_proof_benchmark_status.py`
- [x] `PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_garnet_mit_readiness_status.py`
- [x] `PYTHONDONTWRITEBYTECODE=1 python3 scripts/garnet_mit_readiness_status.py --check-no-regression`
- [x] `PYTHONDONTWRITEBYTECODE=1 python3 scripts/garnet_conformance_matrix_check.py`
- [x] `python3 scripts/check-agent-contracts.py`
- [x] `cargo deny check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --no-fail-fast`
- [x] `git diff --check`

## Cargo Deny / License Check

- Root `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`; inherited duplicate-version warnings remain, and `NCSA` is unmatched in the root workspace because the fuzz harness is an excluded sub-workspace.
- Fuzz manifest `cargo deny --manifest-path garnet-parser-v0.3/fuzz/Cargo.toml check`: `advisories ok, bans ok, licenses ok, sources ok`; duplicate `unicode-width` warning remains.
- `libfuzzer-sys` requires the permissive University of Illinois/NCSA Open Source License component. `deny.toml` now allows `NCSA` explicitly with a scoped S5 comment.

## Desktop Evidence Bundle

- [x] `/Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence-closure/S5-evidence-closure.md`
- [x] `/Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence-closure/machine-metadata.txt`
- [x] `/Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence-closure/mit-readiness.json`
- [x] `/Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence-closure/proof-benchmark-status/garnet-proof-benchmark-status.json`
- [x] `/Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence-closure/proof-benchmark-status/garnet-proof-benchmark-status.md`
- [x] `/Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence-closure/MANIFEST.sha256`
- [x] `shasum -a 256 -c /Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence-closure/MANIFEST.sha256`

## Deferred / Out Of Scope

- No claim that scheduled one-hour nightly fuzz has already accumulated after merge.
- No claim that the generated libFuzzer corpus is a curated source corpus.
- No interpreter/checker fuzz target.
- No OSS-Fuzz integration.
- No parser correctness proof.
- No mechanized proof.
