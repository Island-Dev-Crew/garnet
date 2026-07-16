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
- Independent reviewer for this repair: **PENDING**. This branch is not ready
  for Jon's merge until an independent reviewer records the reviewed repair
  head and zero open Critical/Important findings here.
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
| `scripts/garnet_lane0_closeout_status.py` | Replaced the impossible reviewed-head-to-main ancestry test with a structured, squash-durable marker. The gate now requires a full lowercase `merged_commit`, requires that commit on authoritative upstream main's exact first-parent history, resolves its tree, and requires exact equality with `reviewed_tree`. The evidence `reviewed_head` remains independently checked but is not required to exist locally or be an ancestor after squash. | U-19 showed that the previous proof could never pass after this repository's documented squash merge. The replacement preserves fail-closed review proof rather than deleting or weakening it. |
| `scripts/test_garnet_lane0_closeout_status.py` | Added structured-marker fixtures and real temporary-Git regressions for missing/nonexistent merged commits, commits absent from upstream-main first-parent, tree mismatch, missing authoritative main, divergent state markers, and evidence-head mismatch. It also proves a valid content marker does not require the pre-squash reviewed-head object. | Pin every required RED and prevent a future lineage-based regression or warn-then-green fallback. |

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

Observed after implementation: exit `0`, 26/26 tests passed.

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

**PENDING.** The independent reviewer must record:

- exact reviewed repair head;
- reviewer identity or role;
- reviewed trust-kernel paths;
- fresh Linux, macOS, and Windows evidence files or a blocking gap;
- open Critical findings;
- open Important findings;
- final verdict.

Jon must not merge while this section remains pending.
