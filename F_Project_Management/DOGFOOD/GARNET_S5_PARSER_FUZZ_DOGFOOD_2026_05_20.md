# Garnet S5 Parser Fuzz Harness Dogfood

Date: 2026-05-20
Branch: `codex/s5-parser-fuzz-harness`
Base: `69fcdcbb54ced4cd3af269ee068ca4cf40df9ded`
Slice: S5 — Parser Fuzz Harness

## Current Truth

- S5 adds a source-present `cargo-fuzz` harness for the active parser crate.
- The harness is seeded from 25 current Garnet example files.
- The local dogfood run passed for 60 seconds with no crash or hang.
- The nightly workflow is scheduled for one hour, but no accumulated nightly fuzz-hour evidence is claimed in this PR.
- Parser correctness, mechanized proof, and long-running adversarial-corpus coverage remain unclaimed.
- Daily-truth repair: S9 was already merged via PR #187, but the slice ledger still said `not-started`; this PR corrects that state line without changing S9 implementation.

## Dogfood Command

```bash
cd garnet-parser-v0.3
cargo +nightly fuzz run parse_input -- -max_total_time=60
```

## Dogfood Output Summary

```text
25 files found in garnet-parser-v0.3/fuzz/corpus/parse_input
seed corpus: files: 25 min: 269b max: 9921b total: 38384b rss: 36Mb
DONE   cov: 3053 ft: 13368 corp: 2462/2199Kb lim: 9921 exec/s: 41471 rss: 486Mb
Done 2529747 runs in 61 second(s)
```

Result: exit 0, no panic, no hang, no crash artifact.

## Status Reporter Delta

```text
Before: scripts/garnet_proof_benchmark_status.py tracked 4 Criterion harnesses, 0 fuzz harnesses.
After:  scripts/garnet_proof_benchmark_status.py tracks 4 Criterion harnesses, 1 fuzz harness.

Before: scripts/garnet_mit_readiness_status.py proof_empirics = 45.0%; overall = 61.1%.
After:  scripts/garnet_mit_readiness_status.py proof_empirics = 50.0%; overall = 61.1%.
```

Overall MIT/productization percent did not move in S5 because the objective reporter currently scores the proof lane as `active-partial`; the proof lane itself is more granular.

## Local Verification

- [x] `cargo +nightly fuzz run parse_input -- -max_total_time=60`
- [x] `cargo test -p garnet-parser`
- [x] `PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_garnet_proof_benchmark_status.py`
- [x] `PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_garnet_mit_readiness_status.py`
- [x] `cargo deny --manifest-path garnet-parser-v0.3/fuzz/Cargo.toml check`
- [x] `cargo fmt --all -- --check`
- [x] `cargo fmt --manifest-path garnet-parser-v0.3/fuzz/Cargo.toml -- --check`
- [x] `ruby -e "require 'yaml'; YAML.load_file('.github/workflows/fuzz-nightly.yml')"`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace --no-fail-fast`
- [x] `cargo deny check`
- [x] `PYTHONDONTWRITEBYTECODE=1 python3 scripts/garnet_mit_readiness_status.py --check-no-regression`
- [x] `PYTHONDONTWRITEBYTECODE=1 python3 scripts/garnet_conformance_matrix_check.py`
- [x] `PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_garnet_benchmark_no_run.py`
- [x] `python3 scripts/check-agent-contracts.py`
- [x] `git diff --check`

## Desktop Evidence Bundle

- [x] `/Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence/S5-dogfood.md`
- [x] `/Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence/machine-metadata.txt`
- [x] `/Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence/mit-readiness.json`
- [x] `/Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence/proof-benchmark-status/garnet-proof-benchmark-status.json`
- [x] `/Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence/proof-benchmark-status/garnet-proof-benchmark-status.md`
- [x] `/Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence/MANIFEST.sha256`
- [x] `shasum -a 256 -c /Users/idc2.0/Desktop/garnet-s5-parser-fuzz-evidence/MANIFEST.sha256`

## Cargo Deny / License Check

- Root `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`; inherited duplicate-version warnings remain.
- Fuzz manifest `cargo deny --manifest-path garnet-parser-v0.3/fuzz/Cargo.toml check`: `advisories ok, bans ok, licenses ok, sources ok`; duplicate `unicode-width` warning remains.
- `libfuzzer-sys` requires the permissive University of Illinois/NCSA Open Source License component. `deny.toml` now allows `NCSA` explicitly with a scoped comment for the S5 fuzz harness.

## Deferred / Out Of Scope

- No claim that scheduled one-hour nightly fuzz has already run after merge.
- No claim that the generated in-memory/libFuzzer corpus is a curated source corpus.
- No parser correctness proof.
- No mechanized proof.
- No external adversarial-corpus campaign.
