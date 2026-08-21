# Evidence 101 — dogfood PR-body discipline findings

- Boundary: Freeze-7 Part A5.
- Observation seat: native-Windows NUC.
- Scope: registration only. No checker, test, workflow, permission, event
  trigger, or network behavior is changed by this record.
- Route: Repair 3b, alongside the non-required U-39 Base-controlled
  signature work already assigned there.

## A5-DF-1 — evidence selection follows tuple order, not document order

- Status: **REGISTERED — DEFERRED**.
- Observation date: `2026-08-21`.
- Surface: `scripts/check_dogfood_pr_body.py`.
- Cause: `EVIDENCE_HEADINGS` lists `### Desktop dogfood bundle` before
  `### Evidence bundle`. `present_evidence_heading` iterates that tuple and
  returns the first tuple member found anywhere in the body. It does not
  select the heading that occurs first in document order.
- Effect: when both real headings are present, the checker validates
  `### Desktop dogfood bundle` even when `### Evidence bundle` appears
  earlier in the document.
- Reproduction: a body with checked `### Evidence bundle` followed by
  unchecked `### Desktop dogfood bundle` selected the Desktop section and
  emitted exactly:

  ```text
  evidence bundle section must include at least one checked evidence item
  ```

  The observed heading positions were Evidence `117` and Desktop `160`;
  the selected heading was nevertheless Desktop. Adding one checked item
  to both sections produced zero validation problems.
- Test gap: the existing suite accepts each heading independently but has no
  fixture containing both real headings in both document orders.
- Freeze-7 discipline: the PR body carries at least one checked item in both
  evidence sections and is validated locally before PR creation.
- Repair 3b target: bind validation to document order, or validate every
  evidence section actually present, with regression fixtures for both
  heading orders. No cure is implemented here.

## A5-DF-2 — workflow reruns replay the original event payload

- Status: **REGISTERED — DEFERRED**.
- Observation date: `2026-08-21`.
- Surface: `.github/workflows/dogfood-readiness.yml`.
- Cause: the body gate reads `${{ github.event.pull_request.body }}` from
  the triggering pull-request event.
- Effect: re-running the workflow replays the original event payload. It
  does not acquire a subsequently edited PR body, so a body-dependent failure
  cannot be cured by a workflow rerun.
- Freeze-7 discipline: the exact body is validated before the pull request
  is opened. Never close/reopen, never use “re-run all jobs,” and never use
  “re-run failed jobs.” If the U-17 readback window lapses, Jon must request
  a fresh record-class commit; that commit and its new event are the only
  sanctioned venue refresh.
- Repair 3b target: encode and test the event-freshness contract without
  silently adding GitHub API, token, permission, or network authority.
  No cure is implemented here.

## Boundary disposition

Both findings are registered and routed, not cured. Air confirmation, the
structured review record, carrier approval, U-17 readback, and merge remain
separate acts.
