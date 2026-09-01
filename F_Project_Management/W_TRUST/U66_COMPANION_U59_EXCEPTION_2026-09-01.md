# U-66 companion — sole U-59 same-run exception (2026-09-01)

**Class:** append-only venue-law companion; normative text, not implementation
evidence

**Authority:** DP4 and DP5 of
`L1_DECISION_POINTS_RULING_2026-09-01.md`

**Amends:** U-66 in
`AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md` without editing that landed
sweep record

**Activation state:** `OPEN-UNTIL-IMPLEMENTED`

## Prior law and append-only relationship

U-66 remains the default: an exact readback head receives one CI firing, and a
close/reopen or rerun is not an idempotent venue refresh. The landed sweep is
preserved byte-for-byte. This companion changes no historical exhibit and does
not reinterpret any earlier rerun as satisfying the new mechanical contract.

## Sole U-59 exception

U-59's same-run, same-head **Re-run all jobs** is U-66's only mechanically
checked exception. The complete contract is the verbatim
`POST-RECORD APPROVAL OBSERVATION; SOLE U-59 EXCEPTION TO U-66` block in
`C_Language_Specification/GARNET_TRUST_KERNEL_ROLLING_REVIEW.md`.

The exception exists only when conditions 1–8 and the reporter-emission half of
condition 9 are mechanically true; Jon's read in condition 9 is the separate
mandatory procedural residual and is never claimed as machine-authenticated:

1. Attempt 1 is the unique CI run for the `pull_request` event at the exact
   record-containing head.
2. Its uniquely named artifact
   `r2-approval-pending-<run_id>-attempt-1` contains exactly one safe member,
   `eligibility.json`, whose canonical exact-key receipt binds the immutable
   repository, PR, base, head, tree, record, workflow, run, event, and
   producer-inventory facts.
3. The receipt's sole eligible tuple is
   `state=approval_pending_only` with `finding_codes=[approval-absent]`; every
   non-approval predicate has completed and passed.
4. The designated reviewer later submits decisive `APPROVED` at that exact
   unchanged head.
5. A distinct authenticated Actions-write carrier invokes exactly one
   **Re-run all jobs** on the same run. Carrier authority is transport-only.
6. Attempt 2 uses `actions: read` and live authenticated transport rather than
   mutable fields from the replayed event. It completely paginates the artifact
   inventory, PR commits, PR reviews, and both attempt-specific jobs endpoints,
   and it directly reads the PR, selected-review, artifact archive, and
   workflow-run objects.
7. The predecessor-owned fully expanded job-name multiset occurs exactly once
   on attempt 2, every row has a new positive job ID absent from attempt 1, and
   every row completes successfully.
8. `r2_role_separation_v1` proves that every selected primary and supplemental
   reviewer ID, attempt-2 `triggering_actor.id`, and every authenticated PR
   commit author/committer ID are positive, login-consistent, and pairwise
   disjoint; DP10's additional weakening reviewer composes additively rather
   than replacing the designated primary reviewer.
9. The reporter emits the final authenticated PR/head/tree/base/record/review
   and governance readback, and Jon reads that emission immediately before the
   merge click.

The exception stays unavailable until later arc acts implement and test every
condition, including the distinct carrier identity. An absent or open predicate
falls back to ordinary U-66 law; prose cannot grant eligibility.

## Excluded refreshes

A close/reopen, push, dispatch, new run ID, re-run-failed-only, job-only rerun,
debug rerun, non-CI producer rerun, partial job inventory, or attempt 3 is not
the U-59 exception. Failure after attempt 1 is cured through a new linear record
successor and a new approval venue, not another replay.

U-74 remains orthogonal. A base-landed cure reaches an existing candidate only
through rebase plus a fresh event. The U-59 exception re-observes one unchanged
eligible head and cannot propagate new base bytes.

## Readback residual and non-effects

The reporter/Jon readback pair detects divergence only at observation time. It
does not prevent a later review dismissal, adverse review, head or base move,
governance change, bypass change, or context change, and it is not an atomic
merge lock. Delay or visible UI-state movement requires a new reporter emission
and a new Jon readback.

This companion creates no workflow permission, carrier credential, eligibility
artifact, producer, verifier, structured review record, approval, rerun, merge,
acceptance, launch-state change, or release action.
