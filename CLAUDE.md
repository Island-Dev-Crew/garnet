# CLAUDE.md - Garnet Claude Code Operating Brief

Claude Code must treat this file as a bridge into Garnet's live project truth,
not as the primary source of truth. Git state, current docs, and fresh
verification output outrank every historical handoff.

## First Read

Before editing, read:

1. `F_Project_Management/GARNET_CLAUDE_CODE_RESUME_PACKAGE_2026_05_10.md`
2. `F_Project_Management/GARNET_AGENT_HANDOFF_2026_05_09.md`
3. `F_Project_Management/GARNET_VERTICAL_SLICE_TOOLING_GUIDE.md`
4. `F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md`
5. `CURRENT_STATE.md`
6. `F_Project_Management/ROADMAPS/GARNET_v0_5_LANGUAGE_COMPLETION_ROADMAP.md`
7. `F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md`
8. `C_Language_Specification/GARNET_v0_4_2_Conformance_Suite.md`
9. Root `AGENTS.md` and the nearest subsystem `AGENTS.md` before edits.

## Current Repository Truth

As of this helper refresh, `origin/main` contains:

- PR #68: Phase 4BD / Memory Core 6Q allocator-root lifecycle evidence.
- PR #70: Phase 4BE / Memory Core 6R buffered edge-removal collection evidence.

Current verified main tip:

```text
4e6a0df Merge pull request #70 from Navigata1/codex/phase4be-buffered-edge-removal-collection
```

PR #69 is a documentation/tooling helper slice only. It must not be used as a
runtime readiness claim.

## Boot Verification

Run this before any edit:

```sh
cd /private/tmp/garnet-phase2-block-yield-runtime || cd "/Users/idc2.0/Desktop/GARNET Opus 4.7 Final"
git fetch --prune origin
git fetch --prune fork
git status --short --branch
git remote -v
git log --oneline --decorate --max-count=12 origin/main
gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,isDraft,mergeStateStatus,url --limit 50
```

For any dogfood archive you depend on, verify its manifest before citing it.

## Work Selection

Do not hardcode the next implementation slice from this file. Choose the next
slice from the live implementation plan, roadmap, dogfood log, conformance
suite, and open PR state after the boot verification above.

Memory Core Phase 6Q and 6R are active partial passes. They prove allocator
wrapper evidence only. Production allocator-integrated ARC, interpreter-tied
runtime finalizer invocation, and native backend ARC lowering remain deferred
unless current source and tests prove otherwise.

## Mandatory Discipline

- Verify git state and manifests before edits.
- Red tests before behavior changes.
- Keep each PR narrow and evidence-backed.
- Preserve safe affine exclusion from ARC cycle collection.
- Update docs/conformance/dogfood ledgers when readiness changes.
- Run focused tests before full verification.
- Do not leave evidence only in `/tmp`; copy durable evidence to
  `/Users/idc2.0/Desktop/dogfood` and reseal manifests when a dogfood bundle is
  part of the deliverable.
- If anything fails, report the command, failure, and last known good state.

## Standard Verification Ladder

Focused:

```sh
cargo fmt --all -- --check
git diff --check
cargo test -p garnet-memory --test cycle -- --nocapture
cargo test -p garnet-memory --test properties cycle_aware -- --nocapture
cargo test -p garnet-cli --test conformance_skeleton deferred_arc_cycle_detection -- --nocapture
cargo test -p garnet-cli --test conformance_phase_gates -- --nocapture
```

Full:

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
cargo audit
cargo deny --all-features check
```
