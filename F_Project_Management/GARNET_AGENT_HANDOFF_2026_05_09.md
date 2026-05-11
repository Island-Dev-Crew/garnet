# Garnet Agent Handoff - 2026-05-09

Paste this into a fresh Codex or Claude Code session to continue the project
without replaying the full chat history.

## Objective

Advance Garnet from prototype toward MIT-presentable language readiness by
executing narrow, test-first vertical slices, producing dogfood/security
evidence, and publishing verified PRs. Do not claim complete-language status
unless the implementation, tests, docs, and remote checks prove it.

## Current Repository Truth

- Current working checkout used for this run:
  `/private/tmp/garnet-phase2-block-yield-runtime`
- Original desktop project family:
  `/Users/idc2.0/Desktop/GARNET Opus 4.7 Final`
- Primary repo: `Island-Dev-Crew/garnet`
- Primary remote: `origin = https://github.com/Island-Dev-Crew/garnet.git`
- Push fork: `fork = https://github.com/Navigata1/garnet.git`
- Current verified `origin/main`: `5b940d8`
- Latest merge: PR `#67`, merged 2026-05-09, merge commit `5b940d8`
- Open PRs after the merge: none
- Latest local branch created for the next slice:
  `codex/phase4bd-next-readiness-slice`, based on `origin/main`

PR `#67` was a cumulative integration train of 57 commits from Phase 6N through
Phase 4BC. It was marked ready, all remote checks passed, and it was merged
through the browser/Desktop GitHub session because CLI merge was blocked by org
permissions.

## First Commands In A New Session

```sh
cd /private/tmp/garnet-phase2-block-yield-runtime || cd "/Users/idc2.0/Desktop/GARNET Opus 4.7 Final"
git fetch --prune origin
git fetch --prune fork
git status --short --branch
git log --oneline --decorate --max-count=12 origin/main
gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,isDraft,mergeStateStatus,url --limit 50
```

If the `/private/tmp` worktree is gone, create a fresh branch from the desktop
checkout after fetching `origin/main`.

## Read These Files First

1. `F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md`
2. `F_Project_Management/GARNET_VERTICAL_SLICE_TOOLING_GUIDE.md`
3. `CURRENT_STATE.md`
4. `F_Project_Management/GARNET_CURRENT_VS_HISTORICAL_LEDGER.md`
5. `F_Project_Management/ROADMAPS/GARNET_v0_5_LANGUAGE_COMPLETION_ROADMAP.md`
6. `F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md`
7. `C_Language_Specification/GARNET_v0_4_2_Conformance_Suite.md`

Treat older handoffs as historical context. Current git state plus the files
above are the source of truth.

## Verified Baseline

The latest integrated work includes:

- typed Mnemos episodic cache hardening through Phase 6P
- trait/generic coherence through Phase 5C
- safe-mode borrow-checker slices through Phase 4L
- finite-domain and guard-aware match coverage through Phase 4BC
- static boolean relational guard folding aligned with runtime boolean ordering
- remote CI, security, CodeQL, docs, package smoke, and local verification
  green before merge

Remote checks that passed on PR `#67`:

- rustfmt
- clippy with `-D warnings`
- cargo test on Ubuntu, macOS, and Windows
- cargo doc
- canonical MVP examples
- agent documentation contracts
- cargo audit
- cargo-deny
- CycloneDX SBOM
- CodeQL
- Linux package builds
- deb/rpm smoke tests
- shellcheck installer

The release job was skipped in PR context as expected.

## Desktop Evidence Archive

Use:

```sh
/Users/idc2.0/Desktop/dogfood
```

Important recent checkpoint:

```sh
/Users/idc2.0/Desktop/dogfood/dogfood-readiness-garnet-phase4bc-boolean-relational-guards-20260509-154947
```

That bundle includes the Phase 4BC mutation proof, PR metadata, final remote
check table, deck screenshot, and SHA-256 manifests.

## Next Recommended Work

The next concrete unchecked implementation item in the plan is:

```text
Memory Core Step 9: Promote fixture-backed roots to production ARC allocator roots
```

Do not start by claiming production ARC is complete. Start with a smaller
test-first slice that moves one production-facing allocator behavior out of
pure fixture status. Good candidates:

- allocator-surface finalization reporting after root release collection
- root-buffer/decrement collection evidence through `CycleAwareKindAllocator`
- safe-mode allocation exclusion proven through the allocator-facing path
- store root lifecycle plus finalization-order evidence in one production-facing
  Memory Core test

Likely files:

- `garnet-memory-v0.3/src/alloc.rs`
- `garnet-memory-v0.3/src/cycle.rs`
- `garnet-memory-v0.3/src/lib.rs`
- `garnet-memory-v0.3/tests/cycle.rs`
- `garnet-memory-v0.3/tests/properties.rs`
- `garnet-cli/tests/conformance_skeleton.rs`
- `garnet-cli/tests/conformance_phase_gates.rs`
- current-state, roadmap, conformance, ownership, and dogfood docs

## Verification Ladder

Use focused tests first, then run the full ladder before PR publication:

```sh
cargo fmt --all -- --check
git diff --check
cargo test -p garnet-memory --test cycle -- --nocapture
cargo test -p garnet-memory --test properties cycle_aware -- --nocapture
cargo test -p garnet-cli --test conformance_skeleton deferred_arc_cycle_detection -- --nocapture
cargo test -p garnet-cli --test conformance_phase_gates -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
cargo audit
cargo deny --all-features check
```

Known acceptable `cargo deny` condition: duplicate-version warnings for
`cpufeatures` and `unicode-width` are acceptable only when exit code is zero
and advisories, bans, licenses, and sources are ok.

## GitHub Notes

Use `gh` for PR creation, PR body updates, metadata, and check inspection.
Use the browser/Desktop GitHub session for merges if CLI reports an org
permission error.

The exact failure already seen:

```text
GraphQL: Navigata1 does not have the correct permissions to execute MergePullRequest
```

The browser session was org-authorized enough to merge PR `#67`.

## Non-Negotiables

- Verify before claiming completion.
- Keep each PR narrow and evidence-backed.
- Use red tests before implementation when behavior changes.
- Do not widen runtime semantics beyond what the interpreter or spec proves.
- Keep unknown/incomparable behavior unknown rather than treating it as false.
- Update docs and dogfood artifacts with every readiness slice.
- Preserve Desktop dogfood checkpoints so progress survives compaction.
- Do not merge or publish releases unless the relevant authority path is
  available and the side effect is intentional.

## Current Paused Point

The previous implementation turn was interrupted after creating
`codex/phase4bd-next-readiness-slice` from `origin/main` and while inspecting
Memory Core Step 9. No Memory Core code changes had been made for that next
slice at the time this handoff was created. The only intended immediate docs
created afterward are this handoff and the vertical-slice tooling guide.
