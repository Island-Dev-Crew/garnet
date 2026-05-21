# S11 — v0.6 Slice Contract Scaffold — Implementation Plan

Date: 2026-05-20 (post v0.5.0 + v0.5.1 close)
Contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` is the v0.5
ledger; S11 introduces the v0.6 successor and is logged in
`F_Project_Management/GARNET_v0_6_SLICE_DOGFOOD.md` once this PR lands.
State: not-started → **planned** (this plan file commits the transition)
Reviewer: Jon (Island Development Crew)

> S11 mirrors S0 from the v0.5 cycle: it lands the contract surface a
> downstream slice will need, without claiming any runtime delta. PR title:
> `S11: v0.6 slice contract scaffold + readiness skill v0.5 pulse refresh`.

---

## 1. Why S11 is needed before any v0.6 implementation slice

Every v0.5 slice (S0–S10) reached `review-ready` by citing
`F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` § S\<N\> in its plan and
PR body. The CI grep for the dogfood-readiness headings is independent, but
the project-management discipline that kept eleven concurrent slices honest is
the contract file itself. Without a v0.6 analog on disk:

- No single source of truth for which v0.6 slices exist.
- No state-machine ledger ("not-started → planned → in-progress → review-ready
  → dogfood-passing → merged") that downstream PRs can update in the same
  commit as the work.
- No release-gate definition for what would make v0.6.0 tag-eligible.
- The dogfood-readiness skill on disk still cites v0.4.2 / 86 slices / PR #73,
  predating v0.5 entirely.

S11 lands the contract + the roadmap + the skill refresh so S12–S16 each have
the gravity their PRs need.

## 2. Scope (in)

- `F_Project_Management/GARNET_v0_6_SLICE_DOGFOOD.md` (new). Mirrors the v0.5
  contract: state machine, common verification primitives (unchanged — the
  CI gates already enforce them), cross-slice gates (unchanged), slice
  contracts for S11–S16, v0.6.0 release gate, PR body template (unchanged),
  integration with existing status reporters, honesty anchors (carry
  forward from v0.5).
- `F_Project_Management/ROADMAPS/GARNET_v0_6_LANGUAGE_RUNTIME_ROADMAP.md`
  (new). High-level v0.6 thesis ("v0.5 shipped scaffolds; v0.6 makes them
  load-bearing"), the five confirmed slices, what's explicitly deferred to
  v0.7+, honesty anchors, target lane delta.
- `F_Project_Management/DOGFOOD/GARNET_DOGFOOD_READINESS_SKILL.md` (refresh
  in place). Update the `Live run snapshot` block from PR #73 / `6e945d6` /
  v0.4.2 / 86 slices to v0.5.x / `e43d378` / `v0.5.0` / 87 slices via
  implementation-plan + 71.9 % / 21 lanes / 12 verified via lane reporter.
  Keep the methodology and bundle-minimum sections intact.
- `CHANGELOG.md`: open a new `## [Unreleased] — v0.6.0 in flight` block
  above the (now-closed) v0.5.1 block; entry under that for S11.
  Also resolves live `<<<<<<< HEAD / ======= / >>>>>>> 407e6ec`
  merge-conflict markers (lines 14/30/44 in current `origin/main`) between
  the S7 and S3 entries — both entries are legitimate; the conflict slipped
  through PR #211's merge against the already-merged PR #213. Resolution:
  drop markers, keep both entries in merge order (S7 first, then S3). No
  content changes to either entry. Scope-creep is small and matches S11's
  housekeeping theme.
- `.claude/plans/S11-plan.md` (this file).

## 3. Scope (out)

- **NOT** adding any new lanes to `scripts/garnet_mit_readiness_status.py`.
  Lanes get added by the slices that ship the work, matching the v0.5 S0
  pattern (S0 created the no-regression flag and baseline; the four new
  lanes since were each added by their respective slices). The four v0.6
  lanes (`pkg_resolver_v0_2`, `registry_stub_v0_1`,
  `vm_function_call_lowering`, `parser_cst_layer`) are scoped in their
  contract sections but not yet created in the reporter.
- **NOT** regenerating `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json`.
  No lane percentage moves in S11; the baseline stays current.
- **NOT** touching the v0.5 slice contract file. v0.5 is closed.
- **NOT** opening a v0.6.0 release-gate validation bundle. That's the v0.6.0
  tag-time slice, not S11.
- **NOT** flipping any S12–S16 state past `planned`. They stay
  `not-started` in the v0.6 contract until each one's own PR opens.
- **NOT** adding new dependencies.

## 4. Concrete tasks (ordered)

1. Write `F_Project_Management/GARNET_v0_6_SLICE_DOGFOOD.md` in full.
2. Write `F_Project_Management/ROADMAPS/GARNET_v0_6_LANGUAGE_RUNTIME_ROADMAP.md`.
3. Refresh
   `F_Project_Management/DOGFOOD/GARNET_DOGFOOD_READINESS_SKILL.md` in
   place. Touch only the dated snapshot + slice-count lines.
4. Add `## [Unreleased] — v0.6.0 in flight` block at the top of
   `CHANGELOG.md` with an `### Added` entry naming S11.
5. Run focused verification:
   - `python3 scripts/garnet_mit_readiness_status.py --check-no-regression`
     (must pass — no lane percentage delta).
   - `python3 scripts/garnet_readiness_status.py` (must still report 87/87
     since S11 doesn't add implementation-plan slices).
   - `python3 scripts/garnet_conformance_matrix_check.py` (must pass).
   - `python3 -m unittest discover scripts/ -p 'test_*.py'` (no script
     changes, must still pass).
6. Generate Desktop dogfood bundle at
   `/Users/IDC2.5/Desktop/dogfood/garnet-s11-v0-6-scaffold-<UTCstamp>/`
   with the standard artifact set (report, diff, verification logs, file
   list, sealed MANIFEST).
7. Commit, push to `Navigata1`, open PR against `Island-Dev-Crew/garnet:main`
   with the dogfood-readiness PR-body headings filled in.
8. After CI green, `gh auth switch --user IslandDevCrew` → `gh pr merge
   <num> --squash --delete-branch` → switch back → fast-forward `main`.
9. Mark task #2 completed; move to S12.

## 5. Honest doubts and risks

- **Lane delta deferred.** The contract describes four v0.6 lanes that don't
  yet exist in the reporter. That's intentional and matches v0.5's pattern,
  but it means the "v0.6.0 release gate" section can only define percent
  targets in qualitative terms ("higher AND more granular") until the
  slices that open the lanes land. Documented as a known property of the
  v0.6 contract.
- **Two readiness reporters.** `garnet_readiness_status.py` (implementation
  plan, 87/87 = 100 %) and `garnet_mit_readiness_status.py` (MIT lane,
  71.9 %) are distinct. The skill refresh has to make this distinction
  visible so future agents don't conflate them. Done in §3.
- **S15 (CST) gates S16 (LSP v0.2) and a future formatter v0.2.** The
  contract has to be explicit that S16 is `not-started` until S15 merges,
  otherwise a concurrent agent could open S16 against the trivia-dropping
  parser and waste a week.
- **`use <dep>::*` AST shape.** S12 depends on the parser already being
  able to parse `use foo::bar;` paths. The S11 contract should note that
  S12's plan must verify the parser surface before writing the resolver.
- **Phase-ID rule.** `CLAUDE.md` says "Allocate phase ids with
  `python3 scripts/garnet_phase_id.py` (never hand-pick)." Phase IDs are
  for memory-core / implementation phases, not for slice numbers (the
  v0.5 contract uses bare `S<N>`). S11 keeps `S<N>` numbering; if a
  future v0.6 slice touches Memory Core internals it will allocate a
  phase ID separately.

## 6. State-machine transitions logged here

| Transition | When | Evidence |
|---|---|---|
| not-started → planned | this file commits to `.claude/plans/S11-plan.md` | this file |
| planned → in-progress | draft PR `S11: v0.6 slice contract scaffold …` opens | PR URL |
| in-progress → review-ready | dogfood block green + PR body template filled + CI green | PR check status |
| review-ready → dogfood-passing | Jon review | PR review |
| dogfood-passing → merged | squash-merge + CHANGELOG.md updated | merge commit |

## 7. What I need from Jon before going in-progress

Nothing blocking. The PR opens in draft against `Island-Dev-Crew/garnet:main`
from the `Navigata1` fork, matching the slice discipline that worked through
every v0.5 PR.
