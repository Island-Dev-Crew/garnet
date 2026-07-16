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

PR #508's successful full remote run is bound to companion/evidence head
`e96b9a4b7b55019eb4f7870a8782fb9b80179516`. The two trust-kernel files are
byte-identical to the independently reviewed head `2870ea9`; the intervening
commits only completed this companion and recorded remote evidence.

Fresh successful jobs:

- [Ubuntu cargo test](https://github.com/Island-Dev-Crew/garnet/actions/runs/29532148134/job/87735002035)
- [macOS cargo test](https://github.com/Island-Dev-Crew/garnet/actions/runs/29532148134/job/87735002045)
- [Windows cargo test](https://github.com/Island-Dev-Crew/garnet/actions/runs/29532148134/job/87735002038)
- [Ubuntu agent documentation contracts and rolling trust gate](https://github.com/Island-Dev-Crew/garnet/actions/runs/29532148134/job/87734744024)
- [macOS Studio build + test](https://github.com/Island-Dev-Crew/garnet/actions/runs/29532148137/job/87734744190)
- [Windows Studio build + test](https://github.com/Island-Dev-Crew/garnet/actions/runs/29532148137/job/87734744231)
- [Ubuntu deterministic build](https://github.com/Island-Dev-Crew/garnet/actions/runs/29532148265/job/87735272758)
- [macOS deterministic build](https://github.com/Island-Dev-Crew/garnet/actions/runs/29532148265/job/87735272806)
- [Cross-OS determinism comparison](https://github.com/Island-Dev-Crew/garnet/actions/runs/29532148265/job/87735808303)
- [Agentic dogfood matrix](https://github.com/Island-Dev-Crew/garnet/actions/runs/29532148149/job/87734744460)

The determinism run produced these fresh evidence files:

- artifact `determinism-hash-ubuntu-latest` (ID `8389145923`, archive digest
  `sha256:4bd19b6f1b4e5e955eb962bfd076c9809231f5b2aff7a876e293e6ec5283e103`),
  containing `manifest-hash-ubuntu-latest.txt`;
- artifact `determinism-hash-macos-latest` (ID `8389139936`, archive digest
  `sha256:28ed0d249f2769ea4c0bfc834126cd463845dd37d050a3a30d31c717c05e2907`),
  containing `manifest-hash-macos-latest.txt`.

Both inner files have SHA-256
`376402892b09fd98329d42f04e809c74b22bafc270971e52a10f9636ee5f0d6a`
and both contain the same manifest line:
`16a8f1ef305c7b98ccb2ac3e089e824b51f88b6fc6f6ae709de2ec55243f6df1  examples/det_fixture_01.garnet.manifest.json`.

The three-OS cargo jobs are fresh repository regression evidence; the current
workflow does not execute `test_garnet_lane0_closeout_status.py` on those
runners. The focused squash-proof runtime evidence remains the local 31/31
suite recorded above. This companion does not relabel the cargo matrix as
cross-OS execution of the Python closeout verifier.

An earlier [macOS Studio attempt](https://github.com/Island-Dev-Crew/garnet/actions/runs/29531484580/job/87732574797)
on head `a04ed934f21b2dbe79199034b8a6d275908d661c`
was RED because the hosted `macos-26-arm64` runner returned filesystem
`NSPOSIXErrorDomain Code=5` / `EIO` while tests wrote temporary files. The
exact command passed locally on this Mac with 101/101 tests. No Studio file is
changed in this PR. The active fork credential could not rerun an upstream job
and no admin credential was inherited or used, so a companion-only evidence
commit retriggered the normal PR checks. The fresh macOS Studio job above then
passed all 101 tests plus the shell and package-script contracts, confirming
the earlier failure was infrastructure-only.

Cross-OS repair evidence is **COMPLETE** for the unchanged trust-kernel files.
This final companion-only seal must still receive the ordinary required checks
on its resulting PR head before the draft is marked ready for Jon.

## Final required-check retry

The first final-seal head
`42689dd1061152c75e10bfecc3a107fb8bba1467` passed the trust gate, all three
cargo-test jobs, both Studio jobs, determinism, agentic dogfood, and every
other code-executing required check. Its sole required failure was
[smoke-rpm](https://github.com/Island-Dev-Crew/garnet/actions/runs/29532789469/job/87737808508):
GitHub could not reach `codeload.github.com:443` to download the pinned
`actions/download-artifact` action after three retries. The job failed during
setup before repository code ran.

The active fork credential cannot rerun an upstream job, and no admin
credential was inherited or used. This documentation-only provenance update
therefore creates the normal final retry. It is the terminal companion state:
do not add another self-referential evidence commit merely to restate GitHub's
live result. The PR check state on the resulting head is the authority; Jon
must merge only if every required check is green.

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
Fresh Linux, macOS, and Windows evidence is **APPROVED** at the exact
companion/evidence head recorded above. Jon must not merge unless the live
required-check state is fully green on the terminal companion head.
