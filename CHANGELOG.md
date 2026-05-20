# Changelog

All notable changes to Garnet are recorded here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This file is updated in the same PR as the work it tracks (per the v0.5 slice
contract). Lines added here are part of the calibrated-honesty record — if a
slice ships labeled "partial," its CHANGELOG entry says so explicitly.

## [Unreleased] — v0.5.1 in flight

### Added

<<<<<<< HEAD
- **S7 (Actor OS-thread bridge / `trust-report`):** new
  `garnet trust-report <file.garnet>` command (`garnet-cli/src/cmd/trust_report.rs`)
  produces a structural trust report including the literal line
  `actors: N / threads: N`, matching the contract's dogfood grep. The
  count is structural — `garnet-actor-runtime/src/runtime.rs` already
  spawns one OS thread per actor (its header documents the
  "Spawn-and-mailbox runtime" contract); S7 lands the CLI bridge that
  surfaces what the runtime does. New
  `examples/agent_orchestrator_3thread.garnet` is the three-actor
  fixture; `garnet-cli/tests/trust_report.rs` asserts the dogfood block
  on every `cargo test --workspace`. New "Actor OS-thread bridge" lane
  in `garnet_mit_readiness_status.py` (verified 100%). Honest deferred
  list documents that live-runtime instrumentation, mailbox/Sendable
  audit, and transitive caps aggregation are out of scope. Closes
  Paper VI Contribution 4's CLI-bridge surface gap.
=======
- **S3 (`garnet add` + Manifest Spec v0.1):** new `garnet-cli/src/cmd/add.rs`
  implements `garnet add [--name <id>] <path>` to vendor a local Garnet
  directory into `.garnet/vendor/<name>/`, update `Garnet.toml`'s
  `[dependencies]` table, and write `Garnet.lock` with BLAKE3-per-file
  hashes. Lockfile output is deterministic (alpha-sorted deps, lex-sorted
  files, lowercase hex). Format documented in
  `C_Language_Specification/GARNET_MANIFEST_v0_1.md`. New
  "Garnet manifest + vendored deps" lane in
  `garnet_mit_readiness_status.py` (verified 100%). Honest deferred list
  documents that the interpreter does NOT yet resolve `use <dep>::*` at
  `garnet run` time, remote sources / transitive deps / SemVer matching /
  workspace mode / `garnet verify-deps` are all out of scope until later
  slices.
>>>>>>> 407e6ec (S3: garnet add + Garnet Manifest v0.1 (vendored deps, content-addressed lockfile))
- **S6 (Memory eviction policy benchmarks):** `garnet-memory-v0.3/benches/eviction.rs`
  is a Criterion bench harness exercising `MemoryPolicy::score` +
  `should_retain` per Mnemos kind (working / episodic / semantic /
  procedural) against a naive FIFO baseline. `scripts/garnet_memory_eviction_status.py`
  inventories per-kind coverage; `scripts/test_garnet_memory_eviction_status.py`
  asserts the harness keeps all four kinds covered with both branches.
  New "Memory eviction policy benchmarks" lane in
  `garnet_mit_readiness_status.py` (verified 100%). Closes the S6 contract
  surface and half of Paper VI Contribution 3's production-allocator gap.
  Honest deferred list documents that a fresh Criterion measurement run,
  end-to-end store-throughput benches, and the production allocator path
  itself remain separate work.
- **S4 (Formatter idempotent baseline):** `garnet-cli/tests/fmt_idempotency.rs`
  proves that two passes of `garnet fmt --stdout` over every canonical
  `examples/{mvp_,det_}*.garnet` produce byte-identical output, and that
  three runs on the same input produce identical bytes. This makes the
  S4 contract goal (deterministic, idempotent source formatter) workspace-
  test-enforced. New "Formatter idempotent baseline" lane in
  `garnet_mit_readiness_status.py` (verified 100%). Honest deferred list
  documents that AST-driven semantic formatting, comment-preserving
  round-trip, and workspace-level fmt are NOT in scope until the parser
  grows a trivia-preserving CST.

## [0.5.0] — 2026-05-20

### Added

- **v0.5.0 organization release validation:** the `v0.5.0` GitHub Release now
  exists at `13a5805250dc0777ca9212f2214fff5d07247e7b` with Linux `.deb`/`.rpm`
  packages, macOS aarch64/x86_64 CLI tarballs, unified `SHA256SUMS`, and
  darwin-arm64/linux-x64 VSIX assets from green tag workflows. Release-only
  M5 evidence is sealed at
  `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-release-validation-20260520T142443Z`:
  `scripts/verify_org_release_smoke.sh` passed against the org release without
  source fallback, the installer honestly fell back from the unavailable `.pkg`
  to the aarch64 tarball, `garnet new --template cli` / `garnet test` /
  `garnet run` passed from the installed release binary, and the published
  darwin-arm64 VSIX produced the injected standalone VS Code diagnostic. This
  is still not Apple Developer ID notarization, a signed/notarized macOS `.pkg`,
  Marketplace/OpenVSX publication, or Windows/Linux target-runtime proof.
- **v0.5 macOS release tarball path:** `.github/workflows/linux-packages.yml`
  now stages macOS CLI tarballs for `aarch64-apple-darwin` and
  `x86_64-apple-darwin`, then composes one release-time `SHA256SUMS` covering
  Linux `.deb`/`.rpm` packages plus those tarballs. This closes the pre-tag
  workflow gap that would make an M5 Mac release-only installer smoke fail
  after publication. The tag-time publication and release-only smoke evidence
  are recorded in the v0.5.0 organization release validation entry above; this
  remains not a signed/notarized `.pkg`. Fresh local M5 file-backed release-mode
  evidence is sealed at
  `/Users/idc2.0/Desktop/dogfood/garnet-macos-cli-tarball-release-assets-20260520T135703Z`.
- **v0.5 release-backed VSIX path:** `scripts/package_garnet_vscode_extension.sh`
  now builds `garnet-lsp`, packages the VS Code extension with the bundled
  native server, writes host-labeled VSIX evidence, and can copy a sealed bundle
  to Desktop. `.github/workflows/vscode-extension.yml` builds those VSIX
  artifacts on PR/main/tag runs and publishes them as GitHub Release assets on
  `v*` tag pushes. `scripts/verify_org_release_smoke.sh` now fails the release
  smoke unless the matching release-backed VSIX asset exists and contains the
  extension entry point plus bundled server. The tag-time publication and
  release-backed diagnostic proof are recorded in the v0.5.0 organization
  release validation entry above; this remains not Marketplace publication or
  OpenVSX publication. Fresh M5 local evidence is sealed at
  `/Users/idc2.0/Desktop/dogfood/garnet-vscode-release-assets-20260520T133747Z`
  for `garnet-0.5.0-lsp-mvp-darwin-arm64.vsix`.
- **v0.5 release-gate evidence:** post-merge public installer source-fallback
  proof is recorded in
  `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-rc-merged-20260520T121820Z`, and
  Mac-local Cursor/VSIX diagnostic proof is recorded in
  `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-editor-gate-20260520T122611Z`.
  The latter includes the local `garnet-0.5.0-lsp-mvp.vsix`, installed
  `island-dev-crew.garnet@0.5.0` extension evidence, a screenshot showing
  `1 problem in this file` / `Errors: 1`, and protocol smoke JSON for
  diagnostics, hover, and go-to-definition. Clean standalone VS Code 1.121.0
  diagnostic proof is recorded in
  `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-standalone-vscode-gate-20260520T130303Z`:
  the locally packaged VSIX contains `extension/server/garnet-lsp`, installs
  into isolated user-data/extensions directories, launches without
  `garnet.lsp.path`, and shows the injected syntax-error diagnostic.
- **S8 (Signed hot-reload BLAKE3 demo):** `examples/mvp_11_signed_hotreload.garnet`
  and `examples/mvp_11_signed_hotreload_mismatch.garnet` are runnable
  managed-mode demonstrations of the BLAKE3 fingerprint check that drives the
  Rust-runtime `actor.reload_signed` path. The success example exits 0 with
  `reloaded successfully` on stdout; the mismatch example exits 1 with
  `BLAKE3 fingerprint mismatch` on stderr. New "Signed hot-reload BLAKE3 demo"
  lane in `garnet_mit_readiness_status.py` (verified 100%). Honest deferred
  list documents that managed-mode `actor.reload_signed` syntax is NOT
  exposed yet — the demos use `crypto::blake3` and `raise` to reproduce the
  fingerprint-mismatch behaviour at the program level. Closes Paper VI
  Contribution 5 surface gap.
- **S10 (Compiler advisory mode, rules-based):** `garnet-check-v0.3/src/suggest.rs`
  ships a deterministic, no-LLM suggestion engine with three rules today —
  `managed-fn-missing-caps`, `long-parameter-list`, and `empty-function-body`.
  `garnet check --suggest <file.garnet>` surfaces them prefixed with the
  literal `compiler suggested:` so downstream tooling can grep. Corpus test
  `garnet-check-v0.3/tests/suggest_corpus.rs` proves ≥ 3 distinct rules fire on
  3 fixture programs. New "Compiler advisory mode (rules-based)" lane in
  `garnet_mit_readiness_status.py` (verified 100%). Closes Paper VI
  Contribution 7 surface for the rules-based tier; the LLM tier remains
  pending-infra.
- **S5 (Parser fuzz harness):** `garnet-parser-v0.3/fuzz/` cargo-fuzz
  sub-workspace with a single `parse_input` target wrapping every call to
  `garnet_parser::parse_source_with_budget` in a strict `ParseBudget`
  (4096-byte source cap, 1024-token cap, 32-depth cap, 512-byte literal
  cap). New `.github/workflows/fuzz-nightly.yml` runs `cargo +nightly
  fuzz run parse_input -- -max_total_time=3600` nightly + on-demand;
  crashes upload as artifacts for triage. Seed corpus is populated from
  canonical `examples/*.garnet` files. New "Parser fuzz harness
  (nightly)" lane in `garnet_mit_readiness_status.py` (`verified` 100%).
  `scripts/garnet_proof_benchmark_status.py` also inventories the fuzz
  harness as evidence while keeping accumulated nightly fuzz hours unclaimed.
  The fuzz sub-workspace carries explicit license metadata and a scoped
  `cargo deny --manifest-path garnet-parser-v0.3/fuzz/Cargo.toml check`
  record for `libfuzzer-sys`'s permissive NCSA component. Honest deferred list
  documents that the interpreter, checker, and archived v0.2 parser are NOT in
  scope today.
- **S9 (Determinism CI):** `.github/workflows/determinism.yml` builds
  `examples/det_fixture_01.garnet` with `garnet build --deterministic --sign
  <key>` on a matrix of ubuntu-latest and macos-latest. A `prepare-key` job
  generates a single short-lived ed25519 signing key and uploads it as an
  artifact so both OSs sign with identical key bytes; the `compare` job
  diffs the resulting per-OS SHA-256 manifest hashes and fails CI with an
  `::error::` annotation on divergence. Closes Paper VI Contribution 6
  verification gap. New "Determinism CI cross-machine" lane in
  `garnet_mit_readiness_status.py` (`verified` 100%); honest deferred list
  documents that Windows runner and Linux aarch64 are not yet in the
  cross-OS matrix.
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
- **S1 (LSP MVP):** source-present `garnet-lsp/` language server and
  `editors/vscode/` extension launcher for diagnostics, hover, and basic
  go-to-definition. `scripts/smoke_garnet_lsp_protocol.py` proves those paths
  over stdio; local VSIX packaging now bundles `server/garnet-lsp` from the
  release build, and local install smoke passed in Cursor plus standalone VS
  Code 1.121.0 on this Mac.
- **S2 (Bytecode VM scaffold):** source-present `garnet-vm/` crate with a
  deterministic bytecode serializer, 15 native opcode families, function-level
  tree-walk fallback, `garnet run --vm` / `--interp` dispatch, a bounded
  Criterion VM/interpreter comparison harness, and
  `C_Language_Specification/GARNET_BYTECODE_v0_1.md`. The proof/benchmark
  reporter now inventories the VM harness, and the MIT reporter's proof lane is
  more granular while the overall objective remains active-partial.

### Honest partials

- The `v0.5.0` tag and GitHub Release exist with release-backed installer and
  darwin-arm64 VSIX diagnostic evidence, but that is not proof of Apple
  Developer ID notarization, a signed/notarized macOS `.pkg`,
  Marketplace/OpenVSX publication, or Windows/Linux target-runtime evidence.
- The current Mac has Cursor as `/usr/local/bin/code`, not the standalone VS
  Code CLI. Clean standalone VS Code diagnostic proof exists through an
  isolated downloaded VS Code 1.121.0 app, including the release-backed
  darwin-arm64 VSIX installed from the GitHub Release.
- The S1 LSP slice is source-present until Marketplace/OpenVSX publication and
  full manual VSCode hover/go-to-definition screenshots are attached to later
  review/release evidence. Safe-mode hover, workspace symbols, rename, and
  CST-grade incremental precision remain deferred.
- The S2 VM is a scaffold, not a production VM. It covers 15 opcode families for
  the MVP fixtures, falls back to the tree-walk interpreter at unsupported
  function boundaries, and does not claim a stable bytecode ABI, production
  native compiler proof, full safe-mode lowering, or standing benchmark
  measurements in the status reporter.
- The S5 fuzz harness is source-present with local 60-second dogfood evidence
  and scheduled nightly coverage, not a claim that one-hour nightly fuzz has
  already accumulated or that parser correctness is proven.

### Known Advisory Gates (inherited, not yet fixed)

- Conformance matrix shorthand: 9 path-like references in
  `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md` do not resolve
  to files on disk today. The new check surfaces these as advisory findings; a
  future slice will fix the matrix and flip the gate to strict.

## Historical record

For the v0.4.2 (research-grade) verification ledger and earlier phase logs, see
`F_Project_Management/GARNET_v4_2_HANDOFF.md` and the dated `GARNET_v*_HANDOFF`
files. Pre-CHANGELOG history was tracked in those handoff documents; from v0.5
onward this file is the canonical entry point.
