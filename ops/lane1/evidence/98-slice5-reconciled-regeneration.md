# Evidence 98 — Slice 5 reconciled regeneration, F3 flip, and final product freeze

- Date: 2026-07-28
- Wake tip: `8cfa5fdfa026a3d8ae718027980f494e527c8b73`
- Actual implementer identity: **Codex (OpenAI GPT-5-based agent)** on
  **Hugh's MacBook Pro** (`Mac17,8`, arm64, macOS 26.5 / Darwin 25.5.0).
- Harness transition: earlier Lane 1 artifacts name Claude Code Opus 5 and
  Claude Fable 5 seats. This Slice 5 execution is Codex, not Claude. The
  reviewer of record remains Codex GPT-5.6 Sol on a different machine;
  cross-family separation is therefore **not** claimed for this slice.
- Ceremony seat: Claude chat via Jon. Merge authority: Jon
  (`IslandDevCrew`) only. Review carrier: `IDC-Trust-Review` only. The
  implementer is neither.

## Boot and truth floor

- Global and clone-effective `core.autocrlf`: `false`.
- Working clone: `/private/tmp/garnet-l1-s5-20260728` (space-free,
  non-sync-managed); `origin` is `Island-Dev-Crew/garnet`, `fork` is
  `Navigata1/garnet`, and no `refs/pull/*` exist.
- Fresh main-only clone at `68317ae258327aade47fc2c07b7b5b580ec7c6ea`
  passed:
  - `cargo run -p xtask -- truth --check`
  - `python3 -I scripts/garnet_lane0_closeout_status.py --gate`
  - `python3 -I scripts/garnet_msrv_status.py --gate`
  - `python3 -I scripts/garnet_frozen_backlog_status.py --gate`
  - `python3 scripts/garnet_trust_kernel_review_status.py --gate`

The clean-main trust diagnostic returned `ok: true`, `problems: []`.

## Exact Part A producer commands

```text
python3 scripts/garnet_launch_readiness_status.py --format json > ops/lane0/evidence/08-launch-readiness.json
python3 scripts/garnet_mit_readiness_status.py --committed-only --format json > ops/lane0/evidence/09-mit-readiness.json
python3 -I scripts/garnet_lane0_closeout_status.py --write-denominators
python3 scripts/garnet_launch_readiness_status.py --format markdown > F_Project_Management/LAUNCH/LAUNCH_READINESS.md
python3 /private/tmp/garnet-l1-s5-state-producer.py /private/tmp/garnet-l1-s5-20260728
node ops/mission/render-sotu.mjs
python3 -I scripts/garnet_lane0_closeout_status.py --seal --run-id lane0-20260716-3124ba5
```

The state producer was a bounded deterministic transformation: it refused
unless Verdict 10 named the approved U-31 cure, evidence 97 contained the exact
native-Windows POSIX spelling, launch stayed `HOLD`, and the four denominator
tuples were exactly the admitted values. Its first dry execution changed
nothing because its display-text precondition expected double quotes while
evidence 97 prints the source with single quotes. The matcher alone was
corrected to the committed evidence spelling, then the producer emitted the
state. The complete resulting state diff is committed and reviewable.

No uncontrolled machine-dependent output was introduced. The explicit Codex /
Mac machine identity in mission state is required identity provenance, not a
readiness input. The regenerated `09` artifact is byte-identical to its prior
committed version; any historical absolute-path prose it already contains was
not introduced by this run.

## Producer results and exact Part A diff

`08.source`:

```text
scripts/garnet_launch_readiness_status.py
```

`09.source` is the load-bearing committed-truth value:

```text
committed-truth
```

The four denominators are:

```text
S114 bounded mission  19/19    100.0%
Truth pulse           65.2/70   93.1% (rounded)
Launch-critical       3/6       50.0%
Whole launch ledger   3/8       37.5%
Launch status                    HOLD
```

Part A commit:

```text
599f2a7da1c858951148dd7dd256d6c5b76f67a5
tree f8acebc286c920e49f04fe707035757876ca3c68
```

Diffstat from the wake tip:

```text
F_Project_Management/LAUNCH/LAUNCH_READINESS.md |   5 --
ops/lane0/evidence/08-launch-readiness.json     |   8 +-
ops/lane0/evidence/10-denominators.json         |   2 +-
ops/lane0/evidence/MANIFEST.sha256              |   4 +-
ops/lane0/ledger.jsonl                          |  70 ++++++++--------
ops/mission/state-of-the-union.html             |  27 ++++--
ops/mission/state.json                          | 105 +++++++++++++++++++++---
7 files changed, 152 insertions(+), 69 deletions(-)
```

`09-mit-readiness.json` is absent from the diff because its sanctioned
regeneration was byte-identical.

## F3 RED → GREEN and full-battery differential

Before Part A, at exact wake tip `8cfa5fd`:

```text
python3 scripts/test_garnet_launch_readiness_status.py -v
Ran 41 tests
FAILED (failures=1)
FAIL: test_tracked_ledger_matches_renderer_byte_for_byte
```

After the renderer-owned Markdown ledger regeneration:

```text
python3 scripts/test_garnet_launch_readiness_status.py -v
Ran 41 tests
OK
```

The first raw full-battery attempt under Homebrew Python 3.14 with `-I` was
invalid because isolation correctly hid the user-site PyYAML installation,
producing four import errors. That degradation was not skipped. An isolated
temporary venv was created with exact `PyYAML==6.0.3` and
`jsonschema==4.26.0`; both authoritative runs used the same interpreter:

```text
<venv>/bin/python -I -m unittest discover -s scripts -p 'test_*.py'
```

Baseline: 1,130 tests, five failures, zero errors:

```text
test_repo_and_site_point_to_the_adoption_surface_reporter
test_tracked_ledger_matches_renderer_byte_for_byte
test_all_novel_programs_check_and_run
test_tag_release_publishes_unified_checksummed_assets
test_current_repository_tracks_wv6_acceptance_and_wv7_pending (WV-6)
```

Part A: 1,130 tests, four failures, zero errors:

```text
test_repo_and_site_point_to_the_adoption_surface_reporter
test_all_novel_programs_check_and_run
test_tag_release_publishes_unified_checksummed_assets
test_current_repository_tracks_wv6_acceptance_and_wv7_pending (WV-6)
```

Successor-only failure set: **empty**. F3 is the only delta and flipped green.
The WV-6 failure remains the explicitly expected exact-candidate partial state.

Additional green checks:

```text
cargo fmt --all -- --check
cargo +1.95.0 test -p garnet-cli new_cmd --no-fail-fast  # 13 passed
cargo run -p xtask -- truth --check
python3 scripts/check-agent-contracts.py                  # 24 contracts
python3 scripts/test_check_agent_contracts.py             # 6 passed
python3 -I scripts/garnet_lane0_closeout_status.py --gate
python3 -I scripts/garnet_msrv_status.py --gate
python3 -I scripts/garnet_frozen_backlog_status.py --gate
```

## Part B frozen pair and exact rebind

The repository's own
`garnet_content_provenance.tracked_content_digest(root, "HEAD")` computed at
the last Part A commit:

```text
frozen head   599f2a7da1c858951148dd7dd256d6c5b76f67a5
frozen tree   f8acebc286c920e49f04fe707035757876ca3c68
product sha   ea38d3547eafa7f56141454df50eaf8084dbc66ee9573ebdd67623df8be97bbe
path count    1544
```

Rebind commit:

```text
48295e5281b270384f07fae9e414d110f275afab
tree 32804fa1c39ecf078fd9961cce8fe3c6e096e10f
```

Only the candidate mirror fields were rebound in
`scripts/smoke_garnet_minimum_shelf.py` and
`proofs/minimum-shelf/lane2b/PROOF.json`: reviewed head, reviewed tree,
product digest, and product path count. The count was already `1544`, so its
semantic mirror was verified but required no byte replacement. The historical
pair remains byte-identical:

```text
1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5 / 1527
```

The committed rebind diff is six line replacements per file? No: each file
has exactly three changed value lines (`reviewed head`, `reviewed tree`,
`product digest`); the fourth mirrored value (`1544`) was already exact and
is mechanically asserted in both files. No logic, threshold, exclusion, or
historical value changed.

After the rebind, the repository function returns the same frozen pair at the
index, Part A commit, and rebind commit:

```text
ea38d3547eafa7f56141454df50eaf8084dbc66ee9573ebdd67623df8be97bbe / 1544
```

`python3 -I scripts/smoke_garnet_minimum_shelf.py --gate` returns
`state:"accepted"`, `ok:true`, `findings:[]`, and all five checks true.
`python3 -I scripts/test_garnet_minimum_shelf_provenance.py -v` is 6/6.

WV-6 remains honestly `partial`: its old native-Windows acceptance manifest
still binds the slice-4 pair and therefore reports four exact-candidate
findings (reviewed head, reviewed tree, product digest, live digest). This is
the required pre-NUC state, not a regression to conceal.

## Digest-inert post-freeze law

- `scripts/smoke_garnet_minimum_shelf.py` is the exact excluded reporter path.
- `proofs/**` is a frozen mutable prefix.
- `ops/lane1/**` is a frozen mutable prefix.

Therefore the rebind, this evidence, request 11, `BLOCKED.md`, and the Lane 1
journal heartbeat are all digest-inert. No product-included path may change
after `599f2a7` without restarting the freeze.
