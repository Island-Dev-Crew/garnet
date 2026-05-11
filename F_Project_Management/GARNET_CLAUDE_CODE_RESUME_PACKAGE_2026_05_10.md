# Garnet Claude Code Continuity Bridge - Updated After Phase 4BD/4BE

**Created:** 2026-05-10
**Refreshed:** 2026-05-11
**Purpose:** keep Claude Code useful for Garnet continuation without carrying
stale branch, PR, or readiness claims forward.

This file supersedes the original 2026-05-10 one-shot Claude resume package.
The original package was useful when Phase 4BD was still the next unchecked
slice. That is no longer true.

## Current Verified Truth

Repository:

```text
Island-Dev-Crew/garnet
origin = https://github.com/Island-Dev-Crew/garnet.git
fork   = https://github.com/Navigata1/garnet.git
```

Primary active worktree:

```sh
/private/tmp/garnet-phase2-block-yield-runtime
```

Current verified `origin/main` after the Phase 4BD/4BE merge train:

```text
4e6a0df Merge pull request #70 from Navigata1/codex/phase4be-buffered-edge-removal-collection
```

Recent merged PRs:

```text
PR #68 - Phase 4BD / Memory Core 6Q allocator-root lifecycle evidence
PR #70 - Phase 4BE / Memory Core 6R buffered edge-removal collection evidence
```

Current helper PR:

```text
PR #69 - Claude Code continuity helpers only
```

PR #69 is not a runtime readiness slice. It exists to make future Claude Code
continuation less likely to drift, overclaim, or re-run stale work.

## What Is Proven

The readiness train through PR #70 gives Garnet bounded evidence for:

- Phase 4BC static boolean relational guard folding aligned with runtime
  boolean ordering.
- Phase 6Q allocator-owned root lifecycle evidence through
  `CycleAwareKindAllocator`.
- Phase 6R allocator-facing buffered edge-removal collection evidence through
  `CycleAwareKindAllocator::remove_edge`.

Phase 6Q and Phase 6R are active partial passes. They are valuable allocator
wrapper evidence, but they are not production ARC completion.

## What Is Not Proven

Do not claim these without new source and verification evidence:

- production allocator-integrated ARC is complete,
- runtime finalizer invocation is wired through the interpreter loop,
- native backend ARC lowering is complete,
- Memory Core Tier 1 is complete,
- LSP/CST/release readiness is complete.

## Boot Sequence

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

Verify any Desktop dogfood bundle before citing it:

```sh
cd /Users/idc2.0/Desktop/dogfood/<bundle>
shasum -a 256 -c MANIFEST.sha256
```

The known Phase 4BD and 4BE bundle handles are:

```text
/Users/idc2.0/Desktop/dogfood/dogfood-readiness-garnet-phase4bd-allocator-root-lifecycle-20260510-001418
/Users/idc2.0/Desktop/dogfood/dogfood-readiness-garnet-phase4be-buffered-edge-removal-20260510-080210
```

## Canonical Reading Order

Read these first. Older handoffs are history unless a current file points to
them for provenance.

1. `CLAUDE.md`
2. `.claude/loop.md`
3. `.claude/agents/garnet-readiness-reviewer.md`
4. `.claude/agents/garnet-memory-core-implementer.md`
5. `F_Project_Management/GARNET_AGENT_HANDOFF_2026_05_09.md`
6. `F_Project_Management/GARNET_VERTICAL_SLICE_TOOLING_GUIDE.md`
7. `F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md`
8. `CURRENT_STATE.md`
9. `F_Project_Management/GARNET_CURRENT_VS_HISTORICAL_LEDGER.md`
10. `F_Project_Management/ROADMAPS/GARNET_v0_5_LANGUAGE_COMPLETION_ROADMAP.md`
11. `F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md`
12. `C_Language_Specification/GARNET_v0_4_2_Conformance_Suite.md`
13. `garnet-memory-v0.3/AGENTS.md` before touching Memory Core.

## Selecting The Next Slice

Do not reuse the old "Phase 4BD next slice" prompt. That work has landed.

The next implementation slice must be selected from live repository evidence:

- current `origin/main`,
- current open PR list,
- implementation plan checkbox state,
- roadmap and dogfood ledgers,
- conformance suite deferred/pass rows,
- subsystem `AGENTS.md` contracts.

Use this framing for Memory Core work unless newer docs supersede it:

```text
Continue one bounded Memory Core readiness slice at a time.
Preserve safe-affine exclusion.
Write red tests first.
Do not claim production ARC completion.
Keep docs and dogfood evidence honest.
```

## Verification Ladder

Focused first:

```sh
cargo fmt --all -- --check
git diff --check
cargo test -p garnet-memory --test cycle -- --nocapture
cargo test -p garnet-memory --test properties cycle_aware -- --nocapture
cargo test -p garnet-cli --test conformance_skeleton deferred_arc_cycle_detection -- --nocapture
cargo test -p garnet-cli --test conformance_phase_gates -- --nocapture
```

Full before PR completion:

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
cargo audit
cargo deny --all-features check
```

Known acceptable `cargo deny` condition:

- duplicate-version warnings for `cpufeatures` and `unicode-width` are
  acceptable only if exit code is zero and advisories, bans, licenses, and
  sources are OK.

## Claude Helper Files

This PR maintains these Claude-specific helpers:

```text
CLAUDE.md
.claude/loop.md
.claude/agents/garnet-readiness-reviewer.md
.claude/agents/garnet-memory-core-implementer.md
```

They are operational guardrails, not canonical project truth. If they conflict
with live docs, source, tests, or remote PR state, fix the helper files.

## One-Shot Claude Prompt

```text
You are Claude Code continuing Garnet. Start by reading CLAUDE.md and
F_Project_Management/GARNET_CLAUDE_CODE_RESUME_PACKAGE_2026_05_10.md.

Then run the boot verification commands exactly:
git fetch for origin/fork, git status, remote/log inspection, and open PR list.
Verify any Desktop dogfood bundle before citing it.

Summarize current truth in 10 bullets max. Choose the next implementation slice
only from live origin/main, open PRs, the implementation plan, roadmap, dogfood
log, conformance suite, and subsystem AGENTS.md contracts.

Do not implement until the current lane is clear. When implementing, write red
tests first, preserve safe-affine exclusion, keep PRs narrow, and never claim
production ARC completion unless source and verification genuinely prove it.
```

## Non-Negotiables

- Verify before claiming completion.
- Keep PRs narrow and evidence-backed.
- Red tests before behavior changes.
- Do not widen runtime semantics beyond interpreter/spec proof.
- Do not mark unknown behavior as false, safe, covered, or complete.
- Do not claim production ARC completion from allocator-wrapper evidence.
- Update conformance/current-state/roadmap/dogfood docs when readiness changes.
- Preserve Desktop dogfood checkpoints when producing dogfood evidence.
- Do not merge or release unless authority and side effects are intentional.
- If a command fails, report the command, failure, and last known good state.
