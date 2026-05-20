---
title: "Garnet v0.5: substance over surface"
description: "v0.5 ships six release gates — LSP MVP, bytecode VM scaffold, parser fuzz harness, signed hot-reload demo, cross-machine determinism CI, and rules-based compiler advisory mode. The MIT readiness pulse moved from 54.2% / 12 lanes to 65.3% / 17 lanes — higher AND more granular, on purpose."
date: 2026-05-20
canonical: https://garnet-lang.org/blog/2026-Q2-garnet-v0-5.html
---

# Garnet v0.5: substance over surface

**v0.5 is the first release where we stopped letting the percentage do the talking.**

v0.4.2 landed a 100% tracked-implementation ledger (87/87 slices) and an honest MIT-readiness pulse that *went down* from 58.6% to 55.8% — because we added more sub-gates than we shipped completions in that cycle. We wrote about that in [v0.4.2 and the case for honest accounting](/blog/2026-05-19-v0-4-2-and-honest-accounting.html).

v0.5 is the inverse cycle. We shipped six release gates this time. The pulse moved from **54.2% / 12 lanes** at v0.4.2 release to **65.3% / 17 lanes** at the v0.5 tag. *Both* numbers moved in the right direction — more lanes surface more substance, and the average rose because every shipped lane verifies at 100% with calibrated-honest deferred lists attached.

## The six gates

Each of these is a contract from `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md`. Each one is reproducible from a clean clone via its own dogfood block. None of them is marketing.

### S1 — LSP MVP

A new `garnet-lsp` crate built on `tower-lsp` provides `textDocument/publishDiagnostics` (real `garnet-parser` + `garnet-check` errors on every keystroke), `textDocument/hover` (markdown surfacing `@caps(…)` and function mode), and `textDocument/definition` (parameters + locals + top-level items). The companion VSCode extension under `editors/vscode/` packages with `vsce` and installs via `code --install-extension`.

**Honest partial:** safe-mode hover, workspace symbols, rename, code actions, and CST-grade incremental precision are NOT in this MVP. The lane reports as `source-present` at 60% — the next slice will earn the upgrade.

### S2 — Bytecode VM scaffold

A new `garnet-vm` crate ships a deterministic bytecode serializer, ~15 native opcode families, function-level tree-walk fallback, and a `garnet run --vm | --interp` dispatch flag. A Criterion harness compares the two paths on `examples/mvp_01..05`.

**Honest partial:** This is a **scaffold**, not a production VM. Function calls fall back to the tree-walk interpreter at boundaries. There is no stable bytecode ABI yet. No production native compiler proof is implied.

### S5 — Parser fuzz harness

`garnet-parser-v0.3/fuzz/` is a `cargo-fuzz` sub-workspace with a single `parse_input` target that wraps every parse in a strict `ParseBudget` (4096-byte source cap, 1024 tokens, 32 depth, 512-byte literals). The new `.github/workflows/fuzz-nightly.yml` runs ≥ 1 hour every night and uploads crash inputs as artifacts. Seed corpus is the canonical `examples/*.garnet` set.

**Honest partial:** Interpreter and checker fuzz targets, OSS-Fuzz integration, and coverage-guided corpus minimization are explicitly deferred.

### S8 — Signed hot-reload BLAKE3 demo

Two `examples/mvp_11_signed_hotreload{,_mismatch}.garnet` files demonstrate the BLAKE3 fingerprint check that drives the Rust-runtime `actor.reload_signed` path. The success demo prints `reloaded successfully` and exits 0; the mismatch demo raises with the literal `BLAKE3 fingerprint mismatch` and exits 1. CI is now aware that `*_mismatch.garnet` is an expected-failure example.

**Honest partial:** This is a managed-mode *reproduction* of the fingerprint check using `crypto::blake3` and `raise`. The full Rust-runtime `actor.reload_signed` API exists and is tested in `garnet-actor-runtime/tests/reload.rs`; exposing it as managed-mode syntax is a separate slice.

### S9 — Determinism CI cross-machine

`.github/workflows/determinism.yml` runs three jobs: `prepare-key` generates one ed25519 keypair and uploads it as a 1-day artifact; `build` runs `garnet build --deterministic --sign det.key examples/det_fixture_01.garnet` on `ubuntu-latest` and `macos-latest`; `compare` diffs the per-OS SHA-256 manifest hashes and fails CI with `::error::` annotation if they differ. They didn't differ on the first run, or the second, or the third.

**Honest partial:** Windows and Linux aarch64 are NOT in the cross-OS matrix today. No native-binary determinism (the manifest is the artifact). This closes Paper VI Contribution 6's verification gap.

### S10 — Compiler advisory mode (rules-based)

`garnet check --suggest <file>` produces deterministic, no-LLM suggestions prefixed with the literal `compiler suggested:`. The shipping rules are `managed-fn-missing-caps`, `long-parameter-list`, and `empty-function-body`. A corpus test (`garnet-check-v0.3/tests/suggest_corpus.rs`) asserts at least three distinct rules fire across three fixture programs.

**Honest partial:** The LLM tier remains pending-infra (Paper VI Exp 1 budget). Auto-apply/quick-fix wiring into LSP code-actions is deferred. Cross-module suggestions are out of scope.

## The numbers, side by side

| Metric | v0.4.2 release | v0.5 tag |
|---|---|---|
| Tracked-implementation ledger | 87/87 (100%) | 87/87 (100%) |
| MIT readiness lanes | 12 | 17 |
| Overall MIT readiness | 54.2% | 65.3% |
| Lanes at `verified` 100% | 4 | 9 |

The MIT readiness % moved 11.1 percentage points in 14 days, and **the lane count grew by 5**. That's the signal we cared about. A higher number with the same lane count would have been suspicious — a sign of overclaiming. A higher number *with* a longer lane list, every new lane at 100% with calibrated-honest deferred items attached, is the kind of progress we want to report.

## Paper VI scorecard, unchanged

The Paper VI scorecard remains **4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra**. S5, S8, and S9 close *surface gaps* — they did not require us to re-claim any contribution. The pending-infra item is Paper VI Exp 1 (LLM compiler-as-agent measurements). S10 ships the rules-based tier that establishes the seam where the LLM plugs in once Paper VI Exp 1 unblocks.

## What v0.5 is not

- It is not a production release. Garnet remains a research-grade prototype.
- It is not a 1.0. The Mini-Spec v1.0 is the language *target*, not the current implementation truth — see [the conformance matrix](/blob/main/C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md).
- It is not byte-for-byte deterministic at the *binary* level. S9 verifies manifest determinism. The native-backend determinism story rides on the bytecode VM and a later compiler ladder.
- It is not signed-hot-reload from managed-mode syntax. S8 demonstrates the BLAKE3 fingerprint at the program level; the Rust runtime is where production-grade hot-reload lives today.

These honest "is not" lines aren't apology copy. They're the reason the readiness lanes' `deferred` fields are non-empty: every gap is named, surfaced in the reporter, and recoverable by a future slice without needing to walk back a v0.5 claim.

## What v0.5 reaches for next

The contract carries four `v0.5.1 acceptable` slices: S3 (`garnet add` + manifest spec), S4 (`garnet fmt`), S6 (memory eviction benchmarks), and S7 (actor OS-thread bridge). Each one shifts a specific number on the MIT readiness reporter. None is in this tag. They are scoped, planned, and queued.

Beyond v0.5.x: the v1.0 spec is still the spec. The conformance matrix is still the receipt. The honest-accounting framing isn't a marketing posture — it's the only way we know how to keep the project from drifting into the gap between "shipped" and "demoed."

— Island Development Crew

---

**Reproduce v0.5 from a clean clone:**

```bash
rm -rf /tmp/garnet-v0-5-verify && mkdir /tmp/garnet-v0-5-verify && cd /tmp/garnet-v0-5-verify
git clone https://github.com/Island-Dev-Crew/garnet.git
cd garnet

# All six v0.5.0 gates land via these scripts/tests/workflows. Each gate
# is independently reproducible from this clone:
python3 scripts/garnet_mit_readiness_status.py | head -7              # 65.3% / 17 lanes
python3 scripts/garnet_mit_readiness_status.py --check-no-regression  # exit 0 vs committed baseline
python3 scripts/test_garnet_conformance_matrix_check.py               # S0: 7 tests OK
python3 scripts/test_garnet_mit_readiness_status.py                   # 14+ tests OK

cargo test -p garnet-lsp                                              # S1: smoke tests OK
cargo test -p garnet-vm                                               # S2: scaffold OK
cargo test -p garnet-check --test suggest_corpus                      # S10: 4 corpus assertions OK
cargo deny check                                                      # supply chain OK

cargo build -p garnet-cli --release
./target/release/garnet check --suggest examples/mvp_03_compiler_bootstrap.garnet  # S10: 3 advisories
./target/release/garnet run examples/mvp_11_signed_hotreload.garnet               # S8: exit 0
./target/release/garnet run examples/mvp_11_signed_hotreload_mismatch.garnet      # S8: exit 1

# S5 and S9 run in CI on every push to main. See
# .github/workflows/{fuzz-nightly,determinism}.yml for the exact contract.
```
