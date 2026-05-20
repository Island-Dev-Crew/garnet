# S0 — Housekeeping (Conformance matrix check + readiness no-regression flag) — Implementation Plan

Date: 2026-05-20
Contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md`
   · § **Common Verification Primitives** (names `garnet_conformance_matrix_check.py` as "add in S0 housekeeping")
   · § **v0.5.0 Release Gate** (depends on `garnet_mit_readiness_status.py` reporting a higher AND more granular %)

State: not-started → **planned** (this plan file commits the transition)
Reviewer: Jon (Island Development Crew)

> S0 is not a contracted v0.5.0 gate, but the contract's verification primitives reference scripts and flags that don't exist yet. Landing S0 first unblocks the CI for every later slice. Slice discipline: one slice per PR.
> PR title: `S0: Housekeeping — conformance matrix check + readiness no-regression flag`.

---

## 1. Why S0 is needed before any other slice's review-ready transition

The contract's Common Verification Primitives block lists these as CI gates:

```bash
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/garnet_conformance_matrix_check.py  # add in S0 housekeeping
```

Neither exists today:
- `scripts/garnet_mit_readiness_status.py` has no `--check-no-regression` flag (verified by `python3 scripts/garnet_mit_readiness_status.py --help` showing no such option; the script's argparse currently accepts only `--lane`, `--copy-to-desktop`, etc.).
- `scripts/garnet_conformance_matrix_check.py` does not exist at all (`ls scripts/ | grep conformance` returns nothing).

Until both exist, any later slice's CI run dies on a missing-script error, so no slice can legitimately move from `in-progress → review-ready`. S0 unblocks the train.

## 2. Scope (in)

- Add `scripts/garnet_conformance_matrix_check.py`: parses `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md`, verifies every row references a real test/handle and a real implementation file (best-effort grep), prints a deterministic report, and exits 1 on any unresolved row. Manifest-backed: no claims beyond evidence.
- Add `scripts/test_garnet_conformance_matrix_check.py`: at minimum a synthetic-matrix round-trip test (parse a small fake matrix, assert the script's output).
- Add `--check-no-regression` flag to `scripts/garnet_mit_readiness_status.py`: re-reads a baseline `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json` (committed snapshot of current 12-lane state), runs the live computation, and exits 1 if any lane's percent dropped vs. baseline OR if a lane is missing vs. baseline. Lanes that didn't exist in baseline never trigger regression (only present-in-baseline lanes can regress).
- Add `scripts/test_garnet_mit_readiness_status.py::test_check_no_regression_flag`: synthetic-baseline round-trip test.
- Commit a starter baseline `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json` from the **current** output of `garnet_mit_readiness_status.py` (54.2 %, 12 lanes — captured 2026-05-20).
- Update `AGENTS.md` "Required CI surfaces" (or equivalent) to list both new commands so a future agent doesn't drop them.
- `CHANGELOG.md` entry under an `## [Unreleased]` block: "S0: housekeeping — conformance matrix check + readiness no-regression flag."

## 3. Scope (out)

- **NOT** fixing the 7 `clippy::single_char_add_str` errors in `garnet-cli/src/formatter.rs`. That's S4's surface; per Jon's direction those get flagged in S1's PR body but not fixed in S0.
- **NOT** running `cargo deny check` against the new (untracked) `sha2` dep. That's S3's responsibility when S3 opens.
- **NOT** touching the slice contract file's S<N> rows. The state-machine flips happen in each slice's own PR.
- **NOT** adding new dependencies. S0 is pure Python + Markdown.

## 4. Concrete tasks (ordered, TDD style)

1. Write `scripts/test_garnet_conformance_matrix_check.py` first (failing). Cases:
   - Empty matrix → exits 0, prints "0 rows checked."
   - Synthetic matrix with one row pointing at a non-existent test handle → exits 1, prints the row name.
   - Synthetic matrix with one row that resolves cleanly → exits 0.
2. Run; expect ModuleNotFound / AssertionError.
3. Implement `scripts/garnet_conformance_matrix_check.py`. Keep it ≤ 200 LOC. Use the same deterministic-report style as the other status scripts: a Markdown-headed block to stdout, machine-readable JSON optionally via `--json`.
4. Re-run the failing test, expect PASS.
5. In `scripts/test_garnet_mit_readiness_status.py`, add `test_check_no_regression_flag`:
   - With a written-out fake baseline showing all lanes at 50 %, and live output computed at the current 54.2 % → exit 0.
   - With a baseline showing one lane at 99 % and live at 50 % → exit 1, print the regressing lane name.
6. Run that test; expect failure.
7. Add `--check-no-regression` to `scripts/garnet_mit_readiness_status.py` argparse. Implement comparison logic. Keep stdout output identical to today when the flag is absent; emit a one-line summary when present.
8. Re-run; expect PASS.
9. Generate the baseline:
   ```bash
   python3 scripts/garnet_mit_readiness_status.py --json > F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json
   ```
   (Adds `--json` if it's missing — small, deterministic.)
10. Run the full test suite: `python3 -m unittest discover scripts/ -p 'test_*.py'`.
11. Add CHANGELOG entry.
12. Run the contract's common verification primitives locally:
    - `cargo fmt --all -- --check` (S0 is Python-only, should be no-op)
    - `cargo clippy --workspace --all-targets -- -D warnings` (**expected RED on garnet-cli/formatter.rs — flag in PR body as inherited**)
    - `cargo test --workspace --no-fail-fast`
    - `cargo deny check` (only run if S0's PR is the place we want this surfaced; otherwise defer)
    - `python3 scripts/garnet_mit_readiness_status.py --check-no-regression` (should pass against the baseline we just generated)
    - `python3 scripts/garnet_conformance_matrix_check.py` (should pass on the live matrix; if it doesn't, fix the matrix or relax the check before S0 lands)
13. Open draft PR `S0: Housekeeping — conformance matrix check + readiness no-regression flag` with body using the contract's template. Honest partials: "matrix check is grep-based, not full AST cross-reference."
14. Squash-merge once review-ready. Update contract file's S0-related lines if any (probably none — S0 is not a contracted slice).

## 5. Honest doubts and risks

- **The conformance matrix is large**: writing a check that's strict enough to be useful without being noisy may itself be a small project. Recommendation: start with "every row mentions at least one file path that exists" and grow from there. Don't try to verify "the implementation matches the spec" — that's the whole project, not a script.
- **Baseline drift**: every time a status reporter updates its accounting, the baseline must move. If we land S0 today and then S1 adds an "Editor tooling / LSP" lane on Monday, the baseline must be regenerated as part of S1's PR. Document this in the script's `--help`.
- **`cargo deny check` is still RED inheriting from prior-agent S3 work**. S0 does not unblock that; only S3 owner does. Flag clearly in S0 PR body.
- **No prior `garnet_conformance_matrix_check.py` exists**, so there's no reference impl. I'm writing from scratch but in the same Python style as `garnet_mit_readiness_status.py` (deterministic, manifest-backed).

## 6. State-machine transitions logged here

| Transition | When | Evidence |
|---|---|---|
| not-started → planned | this file commits to `.claude/plans/S0-plan.md` | this file |
| planned → in-progress | draft PR `S0: Housekeeping …` opens | PR URL |
| in-progress → review-ready | dogfood block green + PR body template filled + CI green | PR check status |
| review-ready → dogfood-passing | PR-Agent ≥ 4/5 + Jon review | PR review |
| dogfood-passing → merged | squash-merge + CHANGELOG.md updated | merge commit |

## 7. What I need from Jon before going in-progress

1. **Confirm:** I'm cleared to open the S0 draft PR on the `Navigata1` fork (or whichever fork remote is configured here — `git remote -v` shows what's available).
2. **Confirm:** The baseline file's canonical path. `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json` matches the existing layout, but if you have a preferred location (e.g., `scripts/` or a new `baselines/` dir), say.
3. **Confirm:** S0 PR is OK to flag the existing inherited RED state on `garnet-cli/formatter.rs` clippy without fixing it, so S0 doesn't become "S0 + accidental S4 cleanup."
