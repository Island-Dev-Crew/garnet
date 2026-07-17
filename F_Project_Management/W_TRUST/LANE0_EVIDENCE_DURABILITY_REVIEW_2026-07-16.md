# Lane 0 evidence durability trust-kernel review — 2026-07-16

This is the paired W_TRUST companion for bounded Lane 0 repair #2 on
`mission/l0-evidence-durability`. It covers U-20 checkout fidelity and U-21
squash-durable changed-path validation. It does not authorize merge; Jon is the
only merge authority.

## Frozen boundary

- Fresh integration base: `8535e6d3f9023cabc57476991024354dd2741dc1`
  from `Island-Dev-Crew/garnet` `origin/main`.
- Lane 0 base: `231aefa91985e5a0520c493c7f0fc3e54d74efc8`.
- Independently reviewed pre-squash head, retained as provenance only:
  `3124ba5ecfa88aa6f2c2c289313860670673cdec`.
- Landed Lane 0 commit on upstream main first-parent:
  `aa681bacd2e437bfde3cea0ffc1ca75bdb134aac`.
- No `refs/pull/*` ref may be fetched or required.
- Launch remains HOLD. The Lane 0 audit remains band 3/5 and S6 advisory.

## U-20 — checkout fidelity

Before repair, `ops/lane0/evidence/COMMANDS.json` resolved to
`text: unspecified, eol: unspecified`. A fresh Windows checkout with
`core.autocrlf=true` changed the committed 11,497-byte LF blob into an
11,846-byte CRLF worktree file. Its 349 inserted CR bytes exactly matched its
349 LF bytes, and all 22 manifest hashes failed.

The repair adds one final-precedence `.gitattributes` rule:

```text
ops/**/evidence/** -text
```

This mirrors the existing byte-exact `proofs/** -text` fence and protects every
future lane evidence directory, not just Lane 0. Files already committed with
LF are unchanged in the object store; the rule repairs checkout fidelity.

A reachable-object scan inspected all 45 unique UTF-8 text evidence blobs under
`ops/**/evidence/**` and found zero committed CRLF text blobs. One PNG contains
one CRLF byte pair as binary image data; it is not a text line ending and is
preserved byte-exact. No evidence object was silently renormalized.

## U-21 — main-reachable changed paths

The old PR-body verifier parsed the reviewed candidate and required three local
Git operations against it: `cat-file`, `merge-base --is-ancestor`, and
`diff base...candidate`. The candidate is a discarded pre-squash object and is
absent from a fresh main-only clone, so the result depended on ambient PR refs.

The repair keeps the candidate SHA as immutable review provenance but performs
no Git operation against it. Content proof now requires both the base and
landed commit on `refs/remotes/origin/main` first-parent, ignores replacement
refs, and derives the exact changed set from:

```text
231aefa91985e5a0520c493c7f0fc3e54d74efc8..aa681bacd2e437bfde3cea0ffc1ca75bdb134aac
```

The landed range has 87 paths. The reviewed candidate had 86. An exact set
comparison found one and only one additional landed path:

```text
F_Project_Management/W_TRUST/LANE0_TRUST_KERNEL_REVIEW_2026-07-16.md
```

That companion is explicitly recorded as post-review. Three-dot versus two-dot
semantics are not the cause because the reviewed candidate descends from the
base. The gate pins the 87-path landed count and the exact one-path post-review
delta, then checks the archived 86-path transcript. A missing origin/main,
off-first-parent base or merge, wrong landed count, missing companion path, or
wrong reviewed-head provenance remains RED.

## Recorded RED and local GREEN

The focused RED ran before production implementation:

```text
python3 -I scripts/test_garnet_lane0_closeout_status.py -v
```

It exited 1 with 34 tests: two attribute failures (`text: unspecified`) and two
missing-proof errors. The production implementation then passed that suite.
After adding missing-origin and wrong-landed-count negatives, the final focused
suite is 36 tests green, with the one Windows symlink test skipped as designed.

The exact logs and discrepancy proof are under
`ops/lane0-repair2/evidence/`.

## Fresh clean-clone GREEN

Both acceptance runs are bound to implementation commit
`ab6b91eae4fbdd9c85f44131483de12051bb82f5`. Each clone fetched only the named
fork repair branch and upstream `refs/heads/main`; neither fetched fork main or
any `refs/pull/*` ref.

- Windows default `core.autocrlf=true`: `text: unset`, 23 tracked Lane 0
  evidence files checked, zero object/worktree hash mismatches, pre-squash
  candidate object absent with exit 128, closeout gate PASS.
- Ubuntu/WSL with `core.autocrlf` unset: the same 23/23 byte proof, candidate
  absent with exit 128, closeout gate PASS.

The captured transcripts are
`ops/lane0-repair2/evidence/04-windows-clean-clone-green.txt` and
`05-linux-clean-clone-green.txt`; `MANIFEST.sha256` seals the repair evidence.

## Review status and authority

This companion records the bounded mechanism, falsifiers, and evidence. It is
not an independent-review claim and does not backdate review over this repair.
The rolling trust-kernel gate must report `ok: true` and `problems: []`; the PR
must remain human-merged. Jon must merge only after the clean-clone evidence and
live required checks are green.
