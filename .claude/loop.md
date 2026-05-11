Continue only the already-authorized Garnet readiness task visible in the
current conversation and live repository state.

Before doing anything each loop:

1. Run `git status --short --branch`.
2. Fetch `origin` and `fork` when remote truth matters.
3. Check open PRs and remote checks before creating or merging more work.
4. If tests, CI, or a PR are pending, diagnose those first.
5. If a focused implementation step is in progress, continue the next unchecked
   item only.

Current baseline after this helper refresh:

- PR #68 is merged into `origin/main`.
- PR #70 is merged into `origin/main`.
- Phase 6Q and Phase 6R remain active partial Memory Core evidence, not
  production ARC completion.
- PR #69 is documentation/tooling only.

Rules:

- Read `CLAUDE.md`, root `AGENTS.md`, and nearest subsystem `AGENTS.md` before
  edits.
- Do not claim production ARC complete.
- Red tests before implementation.
- Keep safe affine allocations excluded from ARC cycle collection.
- Update current-state, roadmap, conformance, and dogfood docs when readiness
  changes.
- Run focused verification before full ladder.
- Preserve Desktop dogfood checkpoints under `/Users/idc2.0/Desktop/dogfood`
  and reseal manifests when a dogfood bundle is part of the deliverable.
- Never silently swallow failures. Report command, failure, and last known good
  state.
- Do not start LSP/CST/release work unless Jon explicitly redirects or live
  planning docs make that the current approved lane.
