# Lane 0 closeout repair trust-kernel review — 2026-07-16

This is the rolling trust-kernel review companion for the bounded U-19
post-squash repair on `mission/l0-closeout-repair`.

- Integration base and landed Lane 0 commit:
  `aa681bacd2e437bfde3cea0ffc1ca75bdb134aac`.
- Landed Lane 0 tree:
  `98141597d17e13b02cfa228c03cdf0dc2119ad9f`.
- Matching PR #507 head and tree:
  `5680fbed4684d57fc3773a1f75f86868c44b7a95`,
  `98141597d17e13b02cfa228c03cdf0dc2119ad9f`.
- Independent review ended at:
  `3124ba5ecfa88aa6f2c2c289313860670673cdec`,
  tree `d2d3c735cf25b84ef69e0e385c8cfeb35e1af673`.
- Independently reviewed repair head:
  `2870ea9dab9a49ae08e899948b8334266221d1a7`.
- Independent reviewers: Codex independent mechanism reviewer and Codex
  independent provenance reviewer. Both cleared the exact repair head with
  zero open Critical, Important, or Minor findings.
- Merge authority: Jon only.

The content match above proves that squash preserved Lane 0's final PR tree.
It does not claim that the independent reviewer saw commits
`aa14368bde83391506775d835ace8985bb7bc1ed` or
`5680fbed4684d57fc3773a1f75f86868c44b7a95`; both are recorded as
`reviewed: false` in the state marker. No review coverage is backdated.

## Exact trust-kernel paths

The local rolling gate reports exactly two trust-kernel paths in this repair:

| Path | What changed | Why |
|---|---|---|
| `scripts/garnet_lane0_closeout_status.py` | Replaced the impossible reviewed-head-to-main ancestry test with a structured, squash-durable marker. The gate now requires a full lowercase `merged_commit`, requires that commit on authoritative upstream main's exact first-parent history, resolves its tree, and requires exact equality with `reviewed_tree`. Every content-proof Git command ignores replacement refs. The reusable verifier accepts an exact lane boundary; Lane 0 pins its reviewed head/tree, landed commit/tree, review-scope statement, and both post-review disclosures. | U-19 showed that the previous proof could never pass after this repository's documented squash merge. The replacement preserves fail-closed review proof rather than deleting or weakening it, while preventing replacement refs or two jointly edited state files from rebinding the proof. |
| `scripts/test_garnet_lane0_closeout_status.py` | Added structured-marker fixtures and real temporary-Git regressions for missing/nonexistent merged commits, commits absent from upstream-main first-parent, second-parent-only commits, tree mismatch, missing authoritative main, divergent state markers, evidence-head mismatch, `refs/replace` tree/history attacks, and erasure or rewriting of the exact Lane 0 boundary disclosures. It also proves a valid content marker does not require the pre-squash reviewed-head object. | Pin every required RED and prevent a future lineage-based, replacement-ref, mutable-marker, or warn-then-green regression. |

No workflow, ruleset, Lane 1 artifact, existing sealed Lane 0 evidence file, or
public claim is changed by this repair.

## Recorded RED before implementation

Focused TDD command:

```text
python3 -I -S scripts/test_garnet_lane0_closeout_status.py -v
```

Observed before production implementation: exit `1`, 26 tests run. The valid
structured-marker fixture failed because production still required the string
`approved`; the direct squash-durable tests errored because the new verifier
did not exist; and the marker-divergence/evidence-head mutations reached only
the old generic inconsistency findings.

Independent-review follow-up RED on repair head
`13c9307046b8e2613a5384f530e642934ac178ea`: the same command exited `1`
with 31 tests run. Both replacement-ref attacks false-greened before
hardening; mutations to the exact Lane 0 reviewed-head tree, landed commit,
landed tree, post-review list, or disclosure purpose were not yet bound to an
immutable lane boundary. The second-parent-only regression was already RED as
required.

Rolling trust-kernel companion RED:

```text
python3 scripts/garnet_trust_kernel_review_status.py \
  --changed-file scripts/garnet_lane0_closeout_status.py \
  --changed-file scripts/test_garnet_lane0_closeout_status.py \
  --gate --format json
```

Observed before this companion: exit `1`, `ok: false`,
`review_companion_present: false`, and both trust-kernel paths enumerated.

## Fresh local GREEN

```text
python3 -I -S scripts/test_garnet_lane0_closeout_status.py -v
```

Observed after implementation and independent-review hardening: exit `0`,
31/31 tests passed.

```text
python3 -I -S scripts/garnet_lane0_closeout_status.py \
  --seal --run-id lane0-20260716-3124ba5 --gate --format json
```

Observed after the post-squash markers and SOTU render: exit `0`, `ok: true`,
`findings: []`, evidence `22`, ledger entries `37`, denominators `4`, launch
`HOLD`, audit band `3`, and S6 `advisory`.

## Fresh cross-OS evidence

Fresh repair-head evidence is intentionally **pending** until this branch is
pushed and the successor PR's Linux, macOS, and Windows jobs finish. Do not
substitute PR #507's earlier jobs or durable historical bundles for proof of
this changed verifier.

The reviewer must replace this section with exact successful job URLs and
artifact filenames for the repair head, or record the missing platform as a
blocking finding. Until then, this companion does not claim cross-OS
clearance.

## Independent review verdict

| Reviewer | Exact reviewed head | Scope | Verdict |
|---|---|---|---|
| Codex independent mechanism reviewer | `2870ea9dab9a49ae08e899948b8334266221d1a7` | Both trust-kernel paths; fail-closed marker semantics; replacement-ref and first-parent attacks; generic boundary reuse | APPROVED — 0 Critical, 0 Important |
| Codex independent provenance reviewer | `2870ea9dab9a49ae08e899948b8334266221d1a7` | Both state markers; U-19 disclosure; journals/SOTU/ledger; procedural contract; bounded scope | APPROVED — 0 Critical, 0 Important, 0 Minor |

The mechanism review first found one Important replacement-ref bypass at
`13c9307046b8e2613a5384f530e642934ac178ea`. The repair recorded that RED,
added both attack regressions plus an exact Lane 0 boundary pin, and both
reviewers re-reviewed the final trust-kernel head above. The mechanism
reviewer also confirmed that both replacement-ref regressions fail against a
temporary mutant with the hardening removed.

Independent review of the changed trust-kernel paths is **APPROVED**.
PR-level merge readiness remains **PENDING** only on the fresh repair-head
Linux, macOS, and Windows evidence required above. Jon must not merge until
that section is complete and successful.
