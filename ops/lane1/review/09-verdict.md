# Lane 1 Phase 0 — Independent Cross-Family Review Verdict 09

request: Request 09 — U-31 cure proposal
reviewer: Codex GPT-5.6 Sol (`gpt-5.6-sol`)
review_family: OpenAI Codex — cross-family from the Claude implementer and ceremony seat
reviewed_packet_head: `889bcd706da6fe303377301d466c1af61fd2cf7d`
reviewed_packet_tree: `9f0cc9bd55dd999d0972ff4e6fd94d9dac3a77b0`
red_before_cure: `5a06c293d73ace31b1024f2712947fe776d1a845`
slice_4_evidence_head: `f3876c5a78beb31d6cfe8cc5a115bf264af8008f`
origin_main: `68317ae258327aade47fc2c07b7b5b580ec7c6ea`
swept_at: `2026-07-28T08:44:02Z`
machine: `Pulses-MacBook-Air.local`; Darwin 25.5.0; arm64
verdict: **AUTHORIZE OPTION A WITH CONSTRAINTS — retain `source` and emit the
repo-relative POSIX producer path**
authorized_value: `scripts/garnet_launch_readiness_status.py`
approved_u31_cure_head: **NONE — Request 09 contains no implementation**
phase_0_denominators: **3/6 = 50.0% · 3/8 = 37.5%**
u36: **REGISTER — shelf-gate/WV-acceptance wiring is a separate reviewed
semantic change**
security: **APPLICABLE — current absolute-path emission is both an
information-disclosure surface and a determinism defect; S-SEC-1 carries**

## Executive ruling

**Option A is authorized.** The U-31 cure may change only the launch-readiness
reporter's `source` construction from the resolved host-absolute script path to:

```python
Path(__file__).resolve().relative_to(REPO_ROOT).as_posix()
```

The resulting serialized value must be exactly:

```text
scripts/garnet_launch_readiness_status.py
```

Option B (remove the field) and Option C (replace it with an opaque stable
identifier) are denied. The field truthfully names the producing source file;
its defect is host-relative representation, not its existence or meaning.

The packet's four trap categories are sufficient only with the exact
strengthening recorded below, including native-Windows proof of the POSIX
spelling. `ops/lane0/` remains product-digest-INCLUDED. Extending the U-35
exclusion to `ops/lane0/`, directly or indirectly, is denied.

The denominator expectation in the earlier tasking is falsified by the current
producer and consumer. Phase 0 must land the honest `3/6 = 50.0%` and
`3/8 = 37.5%`. I concur with registering U-36 for any later wiring that makes
WV-6 acceptance move the `minimum_sealed_shelf` gate. U-36 is not part of
U-31 and may not be smuggled into this cure.

## Reviewer identity, independence, and boot

- This review was performed by Codex GPT-5.6 Sol, not by a Claude model.
- The implementer and ceremony seats are Claude-family; this verdict supplies
  the required cross-model-family judgment.
- The review began Tuesday 2026-07-28 at 03:28 CDT. The Friday-sunset to
  Saturday-sunset America/Chicago Sabbath fence was not active.
- `git config --global core.autocrlf false` ran before cloning; the effective
  value was independently read back as `false`.
- A fresh non-synced clone with no spaces was created under `/tmp`.
  Organization `Island-Dev-Crew/garnet` is `origin`; `Navigata1/garnet` is
  `fork`. No `refs/pull/*` refspec was fetched.
- Boot UTC was `2026-07-28T08:28:58Z`; host
  `Pulses-MacBook-Air.local`; Darwin 25.5.0; arm64.
- The reviewed three-commit tail is authored and committed by Jon Isaac
  `<Navigata1@gmail.com>`. No verdict or packet commit is authored by
  `IDC-Trust-Review`.

## Truth floor on landed main

All four required prerequisites passed at exact `origin/main`
`68317ae258327aade47fc2c07b7b5b580ec7c6ea`:

- Lane 0 closeout: PASS; evidence 22/22, ledger 37, denominators 4/4,
  recommendation HOLD.
- MSRV: PASS at Rust 1.95; 16/16 workspace members and 18 active manifests.
- Frozen backlog: PASS; eight entries; no findings.
- Rolling-review v2: PASS; base and head both `68317ae`; trust kernel untouched;
  no problems.

These are review prerequisites, not launch approval.

## Leg 1 — Independent RED reproduction

I checked out packet head `889bcd7` into two fresh no-hardlink clones at
different absolute paths on the same machine and ran:

```text
python3 -I scripts/garnet_launch_readiness_status.py --format json
```

The outputs were not byte-identical:

```text
seat A source =
/private/tmp/garnet-u31-seatA.HcZBS9/repo/scripts/garnet_launch_readiness_status.py

seat B source =
/private/tmp/garnet-u31-seatB.scB1jx/repo/scripts/garnet_launch_readiness_status.py

seat A artifact sha256 =
cbd193999eb388208dadc47c03ea857d31cd539b5df75d5d3081c3e53ab3a55e

seat B artifact sha256 =
28f16a76a57163a591c83db85f0447a75a00acd71930524739112f364a385563
```

Deleting only `source` from the parsed objects made them equal. Substituting
each regenerated blob for digest-included
`ops/lane0/evidence/08-launch-readiness.json` produced two different certified
product pairs:

```text
seat A = 9892468be2d8ecea4b5c7fe71d370637c38a3bfc71f8dbdb48522002d43a6b84 / 1544
seat B = 92dd87979645f47c86a3132554e4bf76462a3f7112b0af2ca653b2ead14cbed0 / 1544
```

Those hashes differ from the packet's seat hashes because the independent
clone roots differ, which is the defect under review. Reconstructing the
packet's disclosed seat-A path exactly reproduced all three packet values:

```text
artifact sha256 = 3f902587e2f40cd819a74710da6ae56cfb6125392508e962a0c62968d21ae96d
blob oid        = 47a7295b6e318737d353f89abc4c5d2604b8691c
product digest  = 824e1e8faeb11d27c558d803b048012da66310ef503f7f754897b3458e622bad
count           = 1544
```

The packet redacts seat B's concrete scratch root as `<scratch>`, so its exact
`b232031b…` digest cannot be reconstructed from committed bytes alone.
That transcription is non-load-bearing: the independent second clone
reproduced the same one-field divergence and a second distinct product digest.

Normalizing both independently generated objects to the proposed source value
made the artifact bytes identical and independently reproduced the packet's
cure-shape simulation:

```text
artifact sha256 = e44eb5b22cdb4a85379d60a553b51b0c77fc270a18ccce24432fa3ea5b60203d
product digest  = 3aa7ecc6d7f4a2520235c4f80bada08afe98b4fc0a37c9a15edf297cfb043650
count           = 1544
```

The simulation is evidence for cure shape, not an approved Phase-0 product
pin.

The committed-vs-regenerated comparison has two independent axes, as the
packet states:

1. `source` changes with the clone root — U-31.
2. `live_wasm_playground.blockers` changes from three entries to none — real
   readiness-evidence drift since the truth freeze.

The second axis must continue to move the artifact; it must not be normalized
away with the first.

**Leg 1: PASS. U-31 is reproduced independently.**

## Leg 2 — Consumed versus emitted

### `08-launch-readiness.json`

`08.source` is emitted-only for gate semantics:

- `build_status()` serializes `source` through the status dataclass.
- JSON rendering emits the field; the human and Markdown renderers do not use
  it.
- `--gate` evaluates a freshly built status and never reads the committed
  artifact.
- Lane 0 closeout reads `08-launch-readiness.json` and requires its schema,
  `recommendation=HOLD`, `launch_ready=false`, exact eight-gate tuple, critical
  denominator, ledger denominator, and evidence totals. It never reads
  `source`.
- The sole test assertion on the field is
  `status.source.endswith("garnet_launch_readiness_status.py")`.
- A targeted `.github/` search found zero references to the artifact or its
  reporter.

Changing the field's representation while preserving its truthful file
identity therefore does not change gate semantics.

### `09-mit-readiness.json`

The contrast is real and load-bearing. Lane 0 closeout explicitly requires:

```text
source = committed-truth
```

It enforces that value in both closeout validation and denominator writing,
and the MIT reporter test asserts it exactly. The fact that `08.source` can
change shape does not authorize a corresponding change to `09-mit.source`.

**Leg 2: PASS. The packet's emitted-vs-consumed determination is correct on
both halves.**

## Leg 3 — Cure option ruling

### Option A — authorized

The reporter describes itself as a machine-independent promotion snapshot.
Within that semantic frame, `source` should name the repository producer, not
the host location of the checkout. A repo-relative path is the narrowest
truthful representation.

`relative_to(REPO_ROOT)` preserves source-file identity and fails closed if
the resolved producer ever escapes the repository root. `.as_posix()` is
load-bearing for the Windows seat. A direct Python lexical probe showed:

```text
str(PureWindowsPath(...).relative_to(repo)) = scripts\garnet_launch_readiness_status.py
as_posix()                                  = scripts/garnet_launch_readiness_status.py
```

Using `str()` would retain OS-dependent separators and leave a cross-platform
byte-determinism defect.

### Option B — denied

Removing the field would discard useful producer provenance and change the
serialized schema shape unnecessarily. Emitted-only does not mean meaningless.

### Option C — denied

An opaque stable identifier would be deterministic but less truthful and less
auditable than the actual repository path. No indirection is needed.

**Leg 3: PASS. Option A is the authorized cure.**

## Leg 4 — Mandatory trap set

The packet's four categories are the right categories. They are mandatory with
the following exact requirements:

1. **Clone-path determinism**
   - Regenerate from two distinct absolute clone roots at the same commit and
     readiness state.
   - Require byte-identical JSON, exact
     `source == "scripts/garnet_launch_readiness_status.py"`, no absolute-root
     prefix, and no backslash.
   - Before slice 5 consumes a Windows regeneration, record native-Windows
     evidence that the emitted value has that exact POSIX spelling. This is a
     strengthening of trap (a), not a new semantic category.
2. **Real state sensitivity**
   - Change a real readiness dependency through the reporter's existing test
     seams.
   - Require the serialized artifact to move for that non-source change and
     restore the original state cleanly.
3. **No collateral reporter semantics**
   - At identical dependency state, compare the pre-cure and post-cure parsed
     status field-by-field.
   - The only value change may be `source`; the key remains present, schema and
     ordering remain unchanged, and human/Markdown behavior remains unchanged.
4. **Digest determinism without exclusion**
   - Recompute the tracked product pair after lawful regeneration from both
     clone roots and require equality.
   - Prove all 31 tracked `ops/lane0/` paths remain included.
   - Require the frozen exclusion tuple to remain exactly:
     `ops/lane2b/`, `proofs/`, `F_Project_Management/W_TRUST/`,
     `ops/lane1/`, plus the existing Shelf reporter self-path.
   - An `ops/lane0/` exclusion, generalized `ops/` predicate, or equivalent
     bypass is an automatic failure.

No fifth semantic trap category is required. The Windows lexical requirement,
exact-value assertion, and field-by-field comparison above close the packet's
only under-specified edges.

**Leg 4: PASS WITH STRENGTHENING. The packet honors the advance denial of an
`ops/lane0/` exclusion.**

## Leg 5 — Denominator truth and U-36

The current producer independently regenerates:

```text
launch critical = 3/6 = 50.0%
launch ledger   = 3/8 = 37.5%
```

The three accepted states are `foundation_integrity`, `native_linux`, and
`s114_acceptance`. `minimum_sealed_shelf` remains hardcoded
`manual-deferred` with the explicit statement that it is never
reporter-derived machine truth.

WV-6 itself is accepted 5/5 and bound to product pair `e89cb299…/1544`, but
`garnet_launch_readiness_status.py` consumes no WV-acceptance state. Lane 0
closeout independently pins and derives exactly 3/6 and 3/8 in both validation
and denominator generation.

Therefore `4/6 = 66.7%` and `4/8 = 50.0%` cannot honestly be emitted by today's
producer. Reaching those values requires a new shelf-gate semantic connection,
not reconciliation toward an expected number.

**I concur with the ceremony ruling: Phase 0 lands at 3/6 · 3/8, and U-36 is
registered for shelf-gate/WV-acceptance wiring as its own reviewed change.**
That ruling supersedes the earlier expectation wherever the current executable
producer falsifies it.

**Leg 5: PASS.**

## Leg 6 — Scope, lineage, and product pin

Request 09 implemented nothing:

- `f3876c5` changes only two native-Windows proof files under `proofs/**`.
- `5a06c29` adds only
  `ops/lane1/evidence/93-u31-machine-path-red.md`.
- `889bcd7` changes only `ops/lane1/BLOCKED.md`,
  `ops/lane1/journal.md`, and `ops/lane1/review/09-request.md`.
- The diff from `f3876c5` to packet tip contains four `ops/lane1/**` paths and
  zero `scripts/garnet_*` paths.
- Merge-base with `origin/main` is exactly `68317ae`; the range contains zero
  merge commits.
- All three commits are authored and committed by Jon Isaac
  `<Navigata1@gmail.com>`.

The tracked product pair was independently recomputed at each revision:

```text
f3876c5 = e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f / 1544
5a06c29 = e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f / 1544
889bcd7 = e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f / 1544
```

All 31 tracked paths under `ops/lane0/` remain included; zero match the mutable
predicate.

**Leg 6: PASS. Nothing is implemented, lineage is linear, and
`e89cb299…/1544` is unmoved.**

## Exact bounded implementation authority

This verdict authorizes one future U-31 cure series with only:

1. `scripts/garnet_launch_readiness_status.py`
   - replace only the `source` value construction with the authorized Option A
     expression;
   - do not change schema, gates, denominators, dependencies, rendering, or
     recommendation logic.
2. `scripts/test_garnet_launch_readiness_status.py`
   - add or tighten only tests needed to enforce the trap set above.
3. `ops/lane1/**`
   - record GREEN evidence, request the implementation verdict, and append the
     mission heartbeat.

The cure series may not modify:

- any other `scripts/garnet_*` file;
- `FROZEN_MUTABLE_PREFIXES`, `REPORTER_PATH`, or any product-digest predicate;
- `.github/**`, rulesets, trust-kernel code, or `ops/mission/state.json`;
- `ops/lane0/**`, the canonical launch ledger, SOTU, or denominator artifacts;
- the shelf gate, WV acceptance semantics, or U-36;
- implementation unrelated to the exact `source` representation.

The legitimate readiness regeneration and Phase-0 artifact refresh occur only
after the U-31 cure receives its own implementation verdict.

## Leg 7 — Security

Security gating applies. The current field can commit `/Users/<name>/...`,
drive names, checkout topology, and temporary-directory structure into a
certified, shareable tree. That is a bounded host-information disclosure in
addition to the integrity/determinism failure.

The scoped security review covered:

- the current `Path(__file__).resolve()` source assignment;
- JSON, human, and Markdown emission paths;
- the committed `08-launch-readiness.json`;
- every closeout read of 08 and 09;
- reporter tests and targeted `.github/` references;
- the proposed `relative_to(REPO_ROOT).as_posix()` expression;
- the product-digest inclusion predicate for `ops/lane0/`.

No formal cure-diff scanner was run because Request 09 contains no cure diff.
The exact future implementation diff remains subject to review. Option A
removes host topology while preserving truthful producer provenance and
introduces no new external input or authority surface.

S-SEC-1 remains in full force: the broad capability/authority sweep is still
owed before the Lane 4 frozen candidate and final red-team.

**Leg 7: PASS for the proposed shape; implementation security remains to be
verified at the cure head.**

## Findings

### F1 — U-31 remains open until an implementation verdict (BLOCKER)

This document authorizes a cure; it does not approve one. No U-31 cure head
exists. Slice 5 may not regenerate or commit `08-launch-readiness.json` until
the exact bounded implementation passes the trap set and receives a new
independent verdict.

### F2 — packet seat-B exact digest is not reconstructible (NOTE)

The packet records the second clone path as `<scratch>`, so the exact
`b232031b…` value cannot be rebuilt from committed evidence. Independent
two-root reproduction, exact packet seat-A reconstruction, and exact normalized
cure-shape reconstruction establish the load-bearing claim. This note does not
block the ruling.

### F3 — canonical Markdown ledger currently trails real readiness state (NOTE)

The reporter suite ran 37 tests with one failure:
`test_tracked_ledger_matches_renderer_byte_for_byte`. The exact diff removes
only three stale `live_wasm_playground` blocker lines from the regenerated
Markdown. Lane 0 closeout still passes.

This is the packet's disclosed real-state drift, not U-31. The cure must record
it as an unchanged baseline failure and may not absorb a ledger regeneration.
The later Phase-0 refresh is the proper scope.

## Remaining gates and consequence

**APPROVED U-31 CURE HEAD: NONE.**

The implementer may now land only the bounded Option A cure and its tests,
record the strengthened traps, and return for an independent implementation
verdict. This ruling does not approve a PR, merge, GitHub approval, launch,
Phase-0 artifact refresh, U-36 implementation, or later ceremony slice.

No implementation code, workflow, ruleset, trust state, PR, or GitHub approval
was changed or handled by this reviewer. No credential material was read,
printed, or modified.

## Reviewer stdout summary

Cross-family Verdict 09 AUTHORIZES U-31 Option A only: retain `08.source` and
emit the exact repo-relative POSIX path
`scripts/garnet_launch_readiness_status.py`. Two fresh clone roots reproduced
one-field artifact divergence and distinct digest-included product trees;
normalization reproduced `e44eb5b2…` and `3aa7ecc6…/1544`. `08.source` is
emitted-only while `09-mit.source=committed-truth` is load-bearing. The four
trap categories are mandatory with exact-value, field-diff, digest-inclusion,
and native-Windows strengthening. Phase 0 must land the honest
`3/6 = 50.0% · 3/8 = 37.5%`; U-36 owns any later shelf/WV semantic wiring.
Request 09 implemented nothing, `e89cb299…/1544` is unchanged, security applies,
and S-SEC-1 carries. Approved U-31 cure head: NONE.
