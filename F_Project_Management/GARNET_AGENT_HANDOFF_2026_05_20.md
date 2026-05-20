# Garnet Agent Handoff — 2026-05-20

**Audience:** the next assistant or contributor opening this repo cold after the v0.5.0 tag and the v0.5.1 sweep.
**Status:** ALL eleven `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` slice contracts (S0 housekeeping + S1–S10) are in `merged` state on `origin/main`. v0.5.0 is tagged and published as a GitHub Release. v0.5.1 follow-on slices (S3, S4, S6, S7) merged today.

---

## 0. Read this first

1. `CLAUDE.md` — the harness contract. The First Read list there is still authoritative for the boot sequence.
2. `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` — single source of truth for every v0.5 PR. State machine, dogfood blocks, honesty anchors, PR body template. **Do not invent slice statuses; flip rows in the same commit as the work.**
3. `CHANGELOG.md` (repo root) — Keep-a-Changelog format. The `[Unreleased] — v0.5.1 in flight` block names every slice merged after the v0.5.0 tag.
4. This file plus the matching memory files: `~/.claude/projects/-Users-IDC2-5-Desktop-Garnet/memory/garnet-slice-discipline.md` and `garnet-paper-vi-anchors.md`.

---

## 1. Current repo truth

- **`origin/main` tip:** post-S3 (`e43d378`), post-S7 (`0540b09`), post-S6 (`0cb4c22`), post-S4 (`#208`). Exact tip drifts hourly via website + supporting PRs; always run `git log --oneline --decorate --max-count=12 origin/main` after `git fetch`.
- **Latest published release:** `v0.5.0` (`13a5805`) — Linux `.deb`/`.rpm`, macOS aarch64/x86_64 CLI tarballs, unified `SHA256SUMS`, darwin-arm64/linux-x64 VSIX assets. Verified via `scripts/verify_org_release_smoke.sh`; M5 evidence under `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-release-validation-20260520T142443Z`.
- **MIT readiness pulse:** **71.9 % / 21 lanes / 12 lanes verified at 100 %** (was 65.3 % / 17 lanes at v0.5.0 tag; +6.6 pp from today's v0.5.1 sweep).
- **Paper VI scorecard (unchanged):** *4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra.*
- **Open PRs upstream:** none directly authored by this session as of writing. Verify with `gh pr list --repo Island-Dev-Crew/garnet --state open`.

```sh
# Boot verification (run before any edit):
cd /Users/IDC2.5/Desktop/Garnet
git fetch --prune origin
git fetch --prune fork
git status --short --branch
git remote -v
git log --oneline --decorate --max-count=12 origin/main
gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,isDraft,mergeStateStatus,url --limit 50
python3 scripts/garnet_mit_readiness_status.py | head -8
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```

---

## 2. What this session landed

Eleven slice contracts merged, in order:

| # | PR | Slice | Lane status |
|---|---|---|---|
| S0 | [#186](https://github.com/Island-Dev-Crew/garnet/pull/186) | Housekeeping (conformance matrix check + readiness no-regression flag + baseline + slice contract on disk) | (no lane — it is the gate infrastructure) |
| S1 | [#185](https://github.com/Island-Dev-Crew/garnet/pull/185) | LSP MVP (tower-lsp diagnostics + hover + go-to-def + VSCode extension) | `editor_lsp_adoption` (`source-present`, 60 %) |
| S2 | [#188](https://github.com/Island-Dev-Crew/garnet/pull/188) | Bytecode VM scaffold + `garnet run --vm/--interp` | `proof_empirics` evidence row expanded |
| S5 | [#189](https://github.com/Island-Dev-Crew/garnet/pull/189) | Parser fuzz harness (`cargo-fuzz` sub-workspace + nightly CI) | `parser_fuzz_harness` (verified 100 %) |
| S9 | [#187](https://github.com/Island-Dev-Crew/garnet/pull/187) | Determinism CI cross-machine (ubuntu+macos byte-identical manifest) | `determinism_ci_cross_machine` (verified 100 %) |
| S10 | [#191](https://github.com/Island-Dev-Crew/garnet/pull/191) | Compiler advisory mode (rules-based) — `garnet check --suggest` | `compiler_advisory_rules_based` (verified 100 %) |
| S8 | [#193](https://github.com/Island-Dev-Crew/garnet/pull/193) | Signed hot-reload BLAKE3 demo (`mvp_11_*`) | `signed_hot_reload_demo` (verified 100 %) |
| v0.5.0 release | tag + #202 | Tag, release-asset workflows, blog post, clean-machine repro | n/a |
| S4 | [#208](https://github.com/Island-Dev-Crew/garnet/pull/208) | Formatter idempotent baseline (workspace test) | `formatter_idempotent_baseline` (verified 100 %) |
| S6 | [#209](https://github.com/Island-Dev-Crew/garnet/pull/209) | Memory eviction policy benchmarks (per-kind Criterion + reporter) | `memory_eviction_benchmarks` (verified 100 %) |
| S3 | [#211](https://github.com/Island-Dev-Crew/garnet/pull/211) | `garnet add` + Manifest v0.1 (vendored deps + content-addressed lockfile) | `garnet_add_manifest` (verified 100 %) |
| S7 | [#213](https://github.com/Island-Dev-Crew/garnet/pull/213) | Actor OS-thread bridge — `garnet trust-report` + 3-actor fixture | `actor_trust_report_bridge` (verified 100 %) |

Per-slice plan files live under `.claude/plans/S<N>-plan.md`. Per-slice Desktop dogfood bundles live under `/Users/IDC2.5/Desktop/dogfood/garnet-<slug>-<UTCstamp>/` with verified `MANIFEST.sha256`.

---

## 3. What's explicitly NOT in v0.5.x today (honest partials per slice)

Every lane keeps a `deferred` list — those are the receipts the next slice cashes. Compact summary:

- **S1 LSP:** safe-mode hover, workspace symbols, rename, code actions, CST-grade incremental precision. Gates on a trivia-preserving CST in `garnet-parser-v0.3`.
- **S2 VM:** function calls fall back to tree-walk; bytecode ABI is not stable; no production native compiler proof.
- **S3 `garnet add`:** **interpreter does NOT yet resolve `use <dep>::*` to vendored sources at `garnet run` time.** Vendor + lockfile half ships; resolver contract is a v0.5.x follow-on slice. Remote sources, transitive deps, SemVer matching, workspace mode, and `garnet verify-deps` all out of scope.
- **S4 formatter:** whitespace + line-ending + terminal-newline normalization only. AST-driven semantic formatting, comment-preserving round-trip, pretty-printer for malformed input, and `garnet fmt --workspace` all gate on the CST work.
- **S5 fuzz:** parser only. Interpreter / checker fuzz targets, OSS-Fuzz upstream integration, differential fuzzing against the archived v0.2 parser, and coverage-guided corpus minimization are deferred.
- **S6 memory benches:** policy-cost measurement, not production allocator. Fresh Criterion measurement run captured separately as Desktop evidence, not embedded in the lane. End-to-end store-throughput benches and the Tier 1 allocator path remain separate.
- **S7 trust-report:** STRUCTURAL count of `actor` declarations. Does NOT spawn the runtime, measure live thread counts, or audit mailbox sizes. Mailbox + Sendable audit beyond `garnet check`, transitive caps aggregation, and cross-actor message-graph visualization are deferred.
- **S8 signed hot-reload:** managed-mode demo of the BLAKE3 fingerprint via `crypto::blake3` + `raise`. Managed-mode `actor.reload_signed` syntax, real signed-bytes payload, and Ed25519 signature verification at the program level are deferred. The Rust runtime API itself ships and is tested in `garnet-actor-runtime/tests/reload.rs`.
- **S9 determinism CI:** Linux x86_64 + macOS aarch64 only. Windows runner, Linux aarch64, multi-key rotation, and native-binary determinism deferred.
- **S10 compiler advisory:** three rules ship (`managed-fn-missing-caps`, `long-parameter-list`, `empty-function-body`). LLM tier remains pending-infra (Paper VI Exp 1 budget). Auto-apply/quick-fix LSP wiring, cross-module suggestions, and per-project rule-severity config deferred.

Bigger items still living in MIT readiness as `active-partial`, `blocked`, or `planned`:

- `developer_id_notarization` (blocked — APPLE_DEV_ID_APP + APPLE_NOTARY_PROFILE missing).
- `macos_studio_dmg` (75 %).
- `windows_linux_distribution` (50 %).
- `promo_video` (composition-ready 50 %).
- `llm_assist` (40 %, secure advisory implementation + provider/runtime boundary + dogfood gate).
- `broad_converter_frontends` (planned, every non-Rust/Ruby/Python/Go language).
- `proof_empirics` (45 %, mechanized proof + external user study still unclaimed).
- `mobile_distribution` (planned 0 %).

---

## 4. Slice discipline that worked through this session

This is the operational layer the rest of the team has to inherit; deviating from it caused every collision and every CI failure we hit.

- **One slice per PR.** Title: `S<N>: <short>`. Never bundle.
- **Branch from a fresh `origin/main` for every slice:** `git switch -c codex/s<N>-<slug> origin/main`. Never reuse a branch across slices.
- **Plan before code.** Drop `.claude/plans/S<N>-plan.md` referencing the contract section before opening the PR.
- **Dogfood bundle is required.** Path: `/Users/IDC2.5/Desktop/dogfood/garnet-<slug>-<UTCstamp>/`. Contents: `dogfood-readiness-report.md`, `change-diff.patch`, `verification-log.txt`, `artifact-files.txt`, `MANIFEST.sha256`, `manifest-verify.log`. Seal MANIFEST with `shasum -a 256` over the other artifacts; verify with `shasum -a 256 -c MANIFEST.sha256` (must be all OK). Note: CLAUDE.md still references `/Users/idc2.0/Desktop/` — the active home today is `/Users/IDC2.5/`; both casings appear in the repo and that is OK.
- **PR body MUST use the dogfood-readiness headings, not just the contract template.** CI's `.github/workflows/dogfood-readiness.yml` greps for `## Dogfood Readiness` plus `### Current truth`, `### Local verification`, `### Remote verification`, `### Desktop dogfood bundle`, `### Deferred / out of scope`. Each subsection needs at least one `- [x] ...` checkbox.
- **Avoid the "production ARC complete" phrase** in PR bodies unless it's already negated by one of the recognized patterns (`not production ARC complete`, `production ARC … deferred`, etc.). Easiest path: don't mention ARC unless the slice actually touches it.
- **Node-24 action minimums** (`scripts/test_github_actions_node24_readiness.py`): `actions/checkout@v6`, `actions/setup-python@v6`, `actions/cache@v5`, `actions/upload-artifact@v6`, `actions/download-artifact@v8`, `github/codeql-action/*@v4`. Older pins fail the gate.
- **`rustfmt` runs `cargo fmt --all -- --check` on every PR.** Always run `cargo fmt -p <package>` locally before pushing; S3 hit a CI rustfmt failure that needed a `--force-with-lease` amend round-trip.
- **MIT-readiness baseline** at `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json` must be regenerated whenever a slice adds a lane. Command: `python3 scripts/garnet_mit_readiness_status.py --format json > F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json`. The S0 `--check-no-regression` gate enforces this on every PR.
- **Push to fork (Navigata1) → PR against `Island-Dev-Crew/garnet`:**
  ```sh
  git push -u fork codex/<slug>
  gh pr create --repo Island-Dev-Crew/garnet --base main \
    --head Navigata1:codex/<slug> --no-maintainer-edit \
    --title "S<N>: <short>" --body-file /tmp/sN-pr-body.md
  ```
- **Merging needs IslandDevCrew** (Navigata1 lacks org merge perms):
  ```sh
  gh auth switch --user IslandDevCrew
  gh pr merge <n> --repo Island-Dev-Crew/garnet --squash --delete-branch
  gh auth switch --user Navigata1
  git checkout main && git fetch origin --quiet && git merge --ff-only origin/main
  ```
- **Concurrent agents own different surfaces.** Before opening a PR, always run `gh pr list --repo Island-Dev-Crew/garnet --state open` AND `git ls-remote fork | grep "codex/s"` to avoid duplicating somebody else's slice. Two PRs landing the same lane will collide on the MIT readiness script and the baseline file; rebasing is mechanical but tedious.
- **`*_mismatch.garnet` examples are expected-failure** in `examples/`. The canonical-examples CI step special-cases them. Any new "this exits 1 by design" example should follow that convention.
- **`@caps()` on `main`** is required by the checker even for purely-computational programs. New `.garnet` examples need it or `garnet check` will diagnose.

---

## 5. Active surface map (where the work lives)

```
garnet-parser-v0.3/     # Mini-Spec v1.0 lex + parse. Trivia-dropping; CST is future work.
garnet-parser-v0.3/fuzz # S5 cargo-fuzz sub-workspace. Separate [workspace] block.
garnet-interp-v0.3/     # Managed-mode tree-walk interpreter.
garnet-check-v0.3/      # Safe-mode + CapCaps + suggest engine (S10).
garnet-check-v0.3/tests/suggest_corpus*     # S10 fixtures + corpus test.
garnet-memory-v0.3/     # Mnemos reference Memory Core.
garnet-memory-v0.3/benches/eviction.rs      # S6 bench harness.
garnet-actor-runtime/   # Bounded mailboxes + signed hot reload + OS-thread spawn.
garnet-stdlib/          # OS-I/O primitives + capability metadata.
garnet-cli/             # `garnet` binary + cmd::* per subcommand.
garnet-cli/src/cmd/add.rs           # S3 garnet add.
garnet-cli/src/cmd/check.rs         # S10 --suggest flag.
garnet-cli/src/cmd/fmt.rs           # S4 idempotent baseline; AST work deferred.
garnet-cli/src/cmd/trust_report.rs  # S7 garnet trust-report.
garnet-cli/tests/fmt_idempotency.rs # S4 workspace test.
garnet-cli/tests/trust_report.rs    # S7 workspace test.
garnet-convert/         # Stylized Rust/Ruby/Python/Go converters; not a transpiler.
garnet-lsp/             # S1 tower-lsp + helpers.
garnet-vm/              # S2 bytecode VM scaffold + benches/vm_vs_interp.rs.
editors/vscode/         # S1 VSCode extension. `npm run package` → vsix.
examples/               # Canonical .garnet examples. mvp_01..mvp_11, det_*.
.github/workflows/      # CI surface; new files this session:
                        # - determinism.yml (S9)
                        # - fuzz-nightly.yml (S5)
scripts/                # Status reporters + tests. New this session:
                        # - garnet_conformance_matrix_check.py
                        # - garnet_lsp_status.py
                        # - garnet_memory_eviction_status.py
                        # - test_garnet_memory_eviction_status.py
                        # plus extended garnet_mit_readiness_status.py
                        # + tests.
F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md  # the contract (do not soften).
F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json # the no-regression baseline.
C_Language_Specification/GARNET_MANIFEST_v0_1.md   # S3 manifest spec.
C_Language_Specification/GARNET_BYTECODE_v0_1.md   # S2 bytecode spec.
docs/blog/2026-Q2-garnet-v0-5.md   # v0.5 launch post.
```

---

## 6. Suggested next slices

In rough priority order. None of these is on the v0.5.0 release-gate critical path (that's done), but each closes an honest-partial line surfaced in the lanes.

1. **Resolver contract (S3 follow-on).** Wire `garnet run` to read `Garnet.toml`'s `[dependencies]`, resolve `use <dep>::<symbol>` through `.garnet/vendor/<dep>/`, and execute the deferred half of S3's contract dogfood line. Touches `garnet-parser-v0.3` (use-path parsing already exists), `garnet-cli/src/cmd/run.rs`, and the interpreter's module resolution. The lane evidence is ready to flip from `deferred` to `verified` once `garnet run` actually consumes the vendored bytes.
2. **CST layer in `garnet-parser-v0.3`.** Unblocks S1 (hover ranges + go-to-def precision), S4 (real AST-driven formatter), and S7 (mailbox-size + caps spans). Bigger lift; plan first.
3. **Determinism matrix expansion (S9 follow-on).** Add `windows-latest` and Linux aarch64 to `.github/workflows/determinism.yml`'s matrix. Most of the work is GitHub runner availability, not Rust.
4. **Live trust-report (S7 follow-on).** Add an opt-in `--runtime` flag that spawns the actor runtime, instruments thread spawn, and prints the live count — preserve the structural default since it has no fork/exec cost.
5. **Compiler advisory mode v2 (S10 follow-on).** Wire suggestions into LSP code-actions (S1 mesh). Add the LLM tier once Paper VI Exp 1 budget lands; the seam is already in `suggest.rs`.
6. **Conformance matrix repair.** S0's `garnet_conformance_matrix_check.py` is advisory because the matrix has 9 path-shorthand findings. A dedicated slice can fix those and flip the gate to `--strict`.
7. **Apple Developer ID notarization.** Still blocked on `APPLE_DEV_ID_APP` + `APPLE_NOTARY_PROFILE` secrets. Credential-blocked, not technical.
8. **Promo video.** `composition-ready` at 50 %. Render + visual QA + website embed are the remaining steps.
9. **Mobile distribution + broad converter frontends.** Planned/0 % lanes. Each one is its own slice plan.

For every slice above, the first move is still: drop `.claude/plans/S<N>-plan.md`, reference the lane and the contract, then code.

---

## 7. Things NOT to do (preserved from earlier handoffs)

- **Do not** soften the honesty anchors. See `garnet-paper-vi-anchors.md` and the contract's § "Honesty Anchors (do not soften)" section. If a slice would weaken one, surface that in the PR body so Jon decides.
- **Do not** rename Mnemos / Memory Core. Naming is locked across six places.
- **Do not** add new dependencies without `cargo deny check` clean. Install with `cargo install cargo-deny --locked` if it isn't on the host (we did that in S0).
- **Do not** force-push to `origin/main`. Revert via a new commit if needed.
- **Do not** treat the dogfood bundle path as decoration — CI greps the PR body for `### Desktop dogfood bundle` and the dogfood-evidence checker fails the PR without it.

---

## 8. Standard sanity ladder before any commit on `main`

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo deny check
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/garnet_conformance_matrix_check.py
```

For S4 / S5 / S6 / S7 specifically the workspace tests already include the new gates (`fmt_idempotency`, `suggest_corpus`, `trust_report`, the eviction reporter test). Run `python3 -m unittest discover scripts/ -p 'test_*.py'` once before any readiness reporter edit.

---

## 9. Cross-session memory the next agent inherits

Saved to `~/.claude/projects/-Users-IDC2-5-Desktop-Garnet/memory/`:

- `garnet-slice-discipline.md` — the per-slice ritual (branch, dogfood bundle, PR body format, auth switching, action-pin minimums, concurrent-agent coordination).
- `garnet-paper-vi-anchors.md` — verbatim phrases that must not soften, plus the Paper VI contribution status (5, 6, 7 closed; 1–4 unchanged).
- `MEMORY.md` — index.

Future sessions should read those before opening any v0.5.x slice.

---

## 10. Status reporter delta across the whole session

```
Start of session (2026-05-20 morning):
  54.2 % overall / 12 lanes / 4 verified
                ↓ +S0 (housekeeping)
                ↓ +S1 (editor_lsp_adoption · source-present 60 %)
                ↓ +S9 (determinism_ci_cross_machine · verified)
                ↓ +S2 (proof_empirics evidence expanded)
                ↓ +S5 (parser_fuzz_harness · verified)
                ↓ +S10 (compiler_advisory_rules_based · verified)
                ↓ +S8 (signed_hot_reload_demo · verified)
                ↓ v0.5.0 tag + release + blog
                ↓ +S4 (formatter_idempotent_baseline · verified)
                ↓ +S6 (memory_eviction_benchmarks · verified)
                ↓ +S3 (garnet_add_manifest · verified)
                ↓ +S7 (actor_trust_report_bridge · verified)
End of session (2026-05-20 night):
  71.9 % overall / 21 lanes / 12 verified
```

The lane count grew (12 → 21) faster than the headline percent because every shipped lane carries a calibrated-honest `deferred` list. That ratio is the load-bearing signal Garnet trusts.

— end handoff —
