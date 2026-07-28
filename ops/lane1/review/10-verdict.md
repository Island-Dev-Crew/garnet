# Lane 1 Phase 0 — Independent Cross-Family Review Verdict 10

request: Request 10 — U-31 cure implementation
reviewer: Codex GPT-5.6 Sol (`gpt-5.6-sol`)
review_family: OpenAI Codex — cross-family from the Claude Code implementer and ceremony seat
reviewed_head: `c3dc53ee4169ae879647fcb74e7bb524488653ed`
reviewed_tree: `20ddbf9e4529b6481d9fa0a548c24366f5b41c18`
packet_tip: `8f224e9bafb49598a7db91663e75c71cdfb468f6`
packet_tree: `73e9d2a14ff1e9b3921cc45c681a7cb7a027e209`
red_before_cure: `657f22a8731344ec2d135a56dfb816616f11d1f9`
authorization: Verdict 09 at `ef6d21b96a3f8ea0b63603dbc11b763ddaf46f40`
origin_main: `68317ae258327aade47fc2c07b7b5b580ec7c6ea`
swept_at: `2026-07-28T11:06:07Z`
machine: `Pulses-MacBook-Air.local`; macOS 26.5.1 (25F80); Darwin 25.5.0; arm64
model: Codex GPT-5.6 Sol
implementer_self_report: Claude Code Opus 5 (`claude-opus-5`)
verdict: **APPROVE — the bounded U-31 cure is correct at exact head
`c3dc53ee4169ae879647fcb74e7bb524488653ed`**
verified_identity: **PASS — head, tree, parentage, scope, product pairs, and
author/committer identity independently reproduced**
lineage: **PASS — merge-base is exact `origin/main`; zero merge commits**
differential: **PASS — Python delta is only the standing WV-6 pin state;
Cargo is exact parity**
security: **APPLICABLE — the cure closes the host-path disclosure; bounded
Codex Security diff scan complete with zero reportable findings**
scope: **PASS — only the authorized reporter expression, its test file, and
`ops/lane1/**` artifacts**
weakening: **NONE — `source` remains, all `ops/lane0/` bytes remain included,
and the frozen digest predicate is byte-identical**
provenance: **PASS — reviewed commits are authored and committed by Jon Isaac
`<Navigata1@gmail.com>`; `IDC-Trust-Review` is absent from the union**
not_verified: **native-Windows emission of the exact POSIX spelling; it is a
separate NUC prerequisite before slice 5 consumes a Windows regeneration**

## Executive ruling

The implementation matches Verdict 09 exactly. The sole production change is:

```python
Path(__file__).resolve().relative_to(REPO_ROOT).as_posix()
```

The serialized key is retained and its value is exactly:

```text
scripts/garnet_launch_readiness_status.py
```

All four mandatory traps pass independently. The RED commit proves the
clone-path trap failed before the cure while the three standing guards passed.
No product-digest exclusion was added or broadened. The product pair lawfully
moves to:

```text
0b6239c204f893ed627f59c937d83b6cd5c56832bc0260e31a63775d8d568158 / 1544
```

at both the cure and packet tips because the changed reporter and test are
digest-included `scripts/` files while the later packet is confined to
digest-excluded `ops/lane1/`.

Native-Windows evidence is **not required before this bounded cure may be
approved**. It is required as a separate NUC leg at the approved head before
slice 5 consumes a Windows regeneration. This preserves Verdict 09's
platform-proof prerequisite without making the NUC prove an unapproved
revision.

**APPROVED HEAD FOR THE NUC:
`c3dc53ee4169ae879647fcb74e7bb524488653ed`.**

## Reviewer identity, boot, and truth floor

- This review was performed by Codex GPT-5.6 Sol, not by a Claude model.
- The review began Tuesday 2026-07-28 in America/Chicago. The Friday-sunset
  through Saturday-sunset Sabbath fence was not active.
- `git config --global core.autocrlf false` ran before cloning and read back as
  `false`.
- A fresh no-space clone was created under `/tmp`, outside a sync-managed
  directory. Organization `Island-Dev-Crew/garnet` is `origin`;
  `Navigata1/garnet` is the second remote `fork`. No `refs/pull/*` refspec was
  fetched.
- Boot UTC was `2026-07-28T10:28:08Z`. The seat is a fanless MacBook Air;
  this verdict makes functional and byte-level claims only.
- `origin/main` was
  `68317ae258327aade47fc2c07b7b5b580ec7c6ea`, tree
  `29191aa0e17121c08b73fe12578ee4464559e2ba`.

All four truth-floor prerequisites passed on exact `origin/main`:

- Lane 0 closeout: PASS; evidence 22/22, ledger 37, denominators 4/4,
  recommendation HOLD.
- MSRV: PASS at Rust 1.95; 18 active manifests and all 16 workspace members.
- Frozen backlog: PASS; eight entries and no findings.
- Authenticated rolling-review v2: PASS with base=head=`68317ae`,
  `trust_kernel_touched=false`, and no problems.

These are review prerequisites, not launch or merge authority.

## Leg 1 — Exact authorized scope

The reviewed series is linear:

```text
ef6d21b (Verdict 09)
  -> 657f22a (RED traps)
  -> c3dc53e (cure)
  -> 8f224e9 (Request 10 packet)
```

Object identity reproduces:

```text
RED  head 657f22a8731344ec2d135a56dfb816616f11d1f9
     tree f69af6b6e97c9a09397580dc5b800b09361c5dcb
cure head c3dc53ee4169ae879647fcb74e7bb524488653ed
     tree 20ddbf9e4529b6481d9fa0a548c24366f5b41c18
tip  head 8f224e9bafb49598a7db91663e75c71cdfb468f6
     tree 73e9d2a14ff1e9b3921cc45c681a7cb7a027e209
```

From Verdict 09 through the packet tip, the changed paths are exactly:

```text
scripts/garnet_launch_readiness_status.py
scripts/test_garnet_launch_readiness_status.py
ops/lane1/BLOCKED.md
ops/lane1/journal.md
ops/lane1/evidence/94-u31-cure-red.md
ops/lane1/evidence/95-u31-cure-green.md
ops/lane1/evidence/96-u31-cross-clone-digest.md
ops/lane1/review/10-request.md
```

The reporter diff is one value-construction line, `+1/-1`. The `source` field
remains the second dataclass/JSON key. The test diff adds only the four
authorized traps and the `asdict` import they use. The RED, GREEN, cross-clone,
request, blocker, and heartbeat material is confined to `ops/lane1/**`.

There is no changed workflow, ruleset, `ops/mission/state.json`, other
`scripts/garnet_*` file, digest predicate, Shelf reporter, trust-kernel path,
Rust file, Cargo file, dependency, or lockfile. `git diff --check` is clean
across the complete Verdict-09-to-packet series.

**Leg 1: PASS. One byte beyond the authorized scope was not found.**

## Leg 2 — Denial honored

At the cure head, `FROZEN_MUTABLE_PREFIXES` is byte-identical to Verdict 09:

```text
b"ops/lane2b/"
b"proofs/"
b"F_Project_Management/W_TRUST/"
b"ops/lane1/"
```

The only reporter self-path remains:

```text
b"scripts/smoke_garnet_minimum_shelf.py"
```

The reviewer enumerated all tracked `ops/lane0/` paths through Git:

```text
tracked ops/lane0 paths = 31
matched by mutable predicate = 0
```

There is no generalized `ops/` or `ops/<lane>/` predicate, no sibling-lane
pattern, and no equivalent runtime bypass. The digest remains sensitive to
every tracked Lane-0 evidence byte.

**Leg 2: PASS. The advance denial was honored exactly.**

## Leg 3 — All four traps

### Trap 1 — clone-path determinism

Two fresh no-hardlink clones at distinct absolute roots, both checked out at
`c3dc53e`, generated byte-identical JSON:

```text
artifact sha256 =
e44eb5b22cdb4a85379d60a553b51b0c77fc270a18ccce24432fa3ea5b60203d
bytes = 5268
source = scripts/garnet_launch_readiness_status.py
absolute = false
backslash present = false
```

The parsed objects and raw bytes are equal.

### Trap 2 — real state sensitivity

Changing a real WASM readiness blocker through the existing immutable
`Dependencies` test seam moved the JSON while leaving `source` fixed:

```text
baseline sha256 =
e44eb5b22cdb4a85379d60a553b51b0c77fc270a18ccce24432fa3ea5b60203d
mutated sha256 =
9d42f4f205f4328d5e7f019c79b29631c614eebd3589c3f353b17bde108d4e2b
source equal = true
restored baseline equal = true
```

The cure normalizes only host representation; it does not normalize readiness
state.

### Trap 3 — no collateral semantics

At identical dependency state, the pre-cure and cured parsed statuses differ
only at `source`:

```text
changed fields = ["source"]
key present before/after = true
schema before/after = garnet.launch_readiness/v1
key order unchanged = true
pre-cure JSON sha256 =
eae5ee927b314c89004c49e36991ccc7af83793b7941eea6415200b319283887
cured JSON sha256 =
e44eb5b22cdb4a85379d60a553b51b0c77fc270a18ccce24432fa3ea5b60203d
human before/after sha256 =
376f4113190a1e9916dbb4bf53ccee622eb4fdb4c383e8c7f715b8ae535145fa
markdown before/after sha256 =
03e5a1fea80593afd7716a5e530d2cddfa0c46fc1575e3b9d7389f3dfd13b34f
```

The human and Markdown renderers never read `.source`.

### Trap 4 — digest determinism without exclusion

The ordinary cure-head pair is identical in both clone roots:

```text
0b6239c204f893ed627f59c937d83b6cd5c56832bc0260e31a63775d8d568158 / 1544
```

Lawfully regenerating `08-launch-readiness.json` in both roots produced the
same Git blob:

```text
44cae2519dc4eaf2fd70aa64817ef7eeb075e8ba
```

Substituting that blob into each tracked product set produced the same
simulated pair:

```text
6f8eb413d4672d6c6c3632ce5d7637a5cbd9682867a75bfbbec50d4cc6661b66 / 1544
```

The simulated pair is a determinism proof, not a slice-5 pin.

**Leg 3: PASS. All four traps were independently re-executed.**

## Leg 4 — RED before cure

At `657f22a`, the focused four-test trap suite exits nonzero:

```text
trap 1 exact repo-relative POSIX source = FAIL
trap 2 real state sensitivity          = PASS
trap 3 no collateral semantics         = PASS
trap 4 frozen exclusion/lane0 included = PASS
```

The failing value is the pre-cure host-absolute script path. At `c3dc53e`,
the same four tests all pass. The RED commit precedes the cure and does not
pretend that a standing guard must fail to prove value.

**Leg 4: PASS.**

## Leg 5 — Pin honesty

The reviewer reimplemented the frozen raw Git construction and compared it
with the repository module at each revision:

```text
ef6d21b =
e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f / 1544

657f22a =
26b0e1f5bc540f8776caa46ccd554257f3e0123d99ea11f083787c6937e2f0cb / 1544

c3dc53e =
0b6239c204f893ed627f59c937d83b6cd5c56832bc0260e31a63775d8d568158 / 1544

8f224e9 =
0b6239c204f893ed627f59c937d83b6cd5c56832bc0260e31a63775d8d568158 / 1544
```

Raw construction and repository module agree at every revision. The test file
and reporter are both digest-included; therefore both RED and cure bytes must
move the product digest. The path count remains 1544 because no included path
was added or removed. The later Request-10 packet is excluded by the literal
`ops/lane1/` rule, so cure and tip remain equal.

The existing Shelf/WV pin still binds `e89cb299…/1544`. At the cure head the
Shelf reporter and WV-6 reporter therefore fail closed on the exact mismatch:

```text
actual   = 0b6239c204f893ed627f59c937d83b6cd5c56832bc0260e31a63775d8d568158
expected = e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f
WV-6 artifacts/checks = 5/5
WV-6 state = partial
```

That state is the disclosed standing pin condition pending slice 5. Updating
the pin inside this one-line cure would have exceeded Verdict 09's authority.

**Leg 5: PASS. The movement is lawful, correctly explained, and honestly
fail-closed.**

## Leg 6 — F3 baseline

The cure-head reporter suite runs 41 tests with exactly one failure:

```text
test_tracked_ledger_matches_renderer_byte_for_byte
```

At RED, the same suite has that failure plus the intentional Trap-1 RED. The
tracked canonical ledger is byte-identical across RED and cure. The live
Markdown output is also byte-identical across the reporter change:

```text
tracked ledger sha256 =
a40f254f13c58d67b9f7b7b76b6e05d60aea9306cf8d40acf1e2919c084fb4f4
live renderer sha256 =
03e5a1fea80593afd7716a5e530d2cddfa0c46fc1575e3b9d7389f3dfd13b34f
```

The exact renderer delta against the tracked ledger removes only these three
stale blocker lines:

```text
docs/playground/live.js browser adapter is not implemented
browser Wasm package is not present under docs/playground/pkg
W-PLAY Playwright browser proof is not recorded
```

This is Verdict-09 F3: pre-existing real-readiness drift owned by the slice-5
ledger regeneration. It is not a U-31 regression and was correctly left
unfixed.

**Leg 6: PASS.**

## Leg 7 — Native-Windows prerequisite

A lexical Windows-path control demonstrates why `.as_posix()` is load-bearing:

```text
str(relative) = scripts\garnet_launch_readiness_status.py
as_posix()    = scripts/garnet_launch_readiness_status.py
```

This is not native-Windows evidence and is not represented as such. The
implementation is nevertheless reviewable on this seat because the authorized
Python expression is exact, the key is retained, both clone roots converge,
state sensitivity is preserved, and the digest closes without an exclusion.

The native-Windows leg must now run at the exact approved head and prove that
the emitted JSON value is byte-for-byte:

```text
scripts/garnet_launch_readiness_status.py
```

That proof must exist before slice 5 consumes a Windows regeneration.

**Leg 7: PASS WITH OUTSTANDING NUC PREREQUISITE.**

## Leg 8 — Differential verification

The full Python batteries used the repository's hash-pinned PyYAML 6.0.3 in a
disposable environment:

```text
origin/main 68317ae: 1123 tests; 6 failures; 0 errors; 5 skipped
cure c3dc53e:        1130 tests; 7 failures; 0 errors; 5 skipped
```

The six base failures are identical:

```text
test_repo_and_site_point_to_the_adoption_surface_reporter
test_gate_cli_returns_zero
test_tracked_ledger_matches_renderer_byte_for_byte
test_gate_passes_on_real_repo [linear effect]
test_gate_passes_on_real_repo [provenance seal chain]
test_tag_release_publishes_unified_checksummed_assets
```

The only cure-side failure not present on base is:

```text
test_current_repository_tracks_wv6_acceptance_and_wv7_pending [WV-6]
```

Its exact cause is the disclosed product mismatch `0b6239c2… !=
e89cb299…`; the four new U-31 traps pass. This matches the standing WV state,
not a new semantic regression.

At exact Rust 1.95:

```text
rustup run 1.95.0 cargo test --workspace --no-fail-fast
origin/main: 2199 passed; 0 failed; 6 ignored
cure:       2199 passed; 0 failed; 6 ignored
```

No Rust, Cargo manifest, or lockfile byte changes between base and cure.

**Leg 8: PASS.**

## Leg 9 — Security

Security gating applies. Before the cure, a certified JSON tree could record a
username, drive name, temporary-directory topology, and checkout location.
That is a bounded information-disclosure surface and an integrity/determinism
defect.

A formal Codex Security diff scan covered the exact
`657f22a..c3dc53e` range:

```text
scan id =
f13009fe-001f-4e81-8df6-e8f7e4358d5e
snapshot =
codex-security-snapshot/v1:sha256:82cbcfbbb7d0112b85c1a5cd3e651f8244522ccd4122c5bbe562de9915d92992
manifest sha256 =
fbea26f8d9eeeb2dc65838ca705aa2cacb8cb0e0bcf410d433cf7f13301591f2
coverage = 1/1 source-like diff rows
reportable findings = 0
```

The full reporter file, resolved repository security policy, serialization,
and minimum consumer context were reviewed. The new expression emits only a
truthful repository path, `relative_to(REPO_ROOT)` fails closed if the producer
escapes the repository, `.as_posix()` removes separator variance, and no
attacker-controlled input, external authority, or dangerous sink is added.

The cure fully closes the U-31 host-path disclosure. S-SEC-1 remains the
separate broad capability/authority sweep before the Lane 4 frozen candidate
and final red-team.

**Leg 9: PASS.**

## Findings

### F1 — stale canonical Markdown ledger (NOTE, non-blocking)

Reproduction: the reporter suite has only
`test_tracked_ledger_matches_renderer_byte_for_byte` red after the cure;
pre/post U-31 Markdown is byte-identical at `03e5a1fe…`; the tracked ledger
remains `a40f254f…`; the exact difference is the three obsolete W-PLAY blocker
lines quoted in Leg 6.

Disposition: unchanged Verdict-09 F3, owned by slice 5, not a cure regression.

### F2 — native-Windows POSIX emission evidence (PREREQUISITE, non-blocking to cure)

Reproduction: the Windows lexical control yields backslashes from `str()` and
forward slashes from `.as_posix()`. No native-Windows run was performed or
claimed on this MacBook Air.

Disposition: does not block approval of the exact code cure; blocks slice 5
from consuming a Windows regeneration until the NUC records the exact emitted
value at `c3dc53e`.

### F3 — expected Shelf/WV pin mismatch (NOTE, non-blocking)

Reproduction: the cure-head product pair is `0b6239c2…/1544` while the
pre-cure Shelf/WV pin is `e89cb299…/1544`; WV-6 retains all five required
artifacts/checks and reports `partial`.

Disposition: correct fail-closed standing state. The reviewed cure was not
authorized to absorb the later pin/ledger/denominator regeneration.

## Style advisories

S-SEC-1 (ADVISORY, non-blocking): carry forward the broad capability and
authority sweep before the Lane 4 frozen candidate and final red-team.

S-WS-1 (ADVISORY, non-blocking): the complete historical branch diff retains
trailing whitespace in immutable Requests 02, 03, and 07. The entire U-31
series from Verdict 09 through Request 10 is `git diff --check` clean. Do not
create a formatting-only cure round or rewrite earlier review artifacts.

No style issue blocks U-31 approval.

## Scope, weakening, provenance, and not verified

- Scope is exact: one production expression, the four-trap test addition, and
  Lane-1 review artifacts.
- The `source` key, schema, key order, human render, Markdown render, gates,
  denominators, recommendation, dependency collection, and evidence-base logic
  are unchanged.
- No assertion, trap, product byte, Lane-0 path, workflow, ruleset, trust
  record, digest predicate, or historical Shelf provenance anchor was removed
  or loosened.
- Merge-base with `origin/main` is exact `68317ae`; zero merge commits occur in
  the candidate range.
- RED, cure, and packet commits are authored and committed by Jon Isaac
  `<Navigata1@gmail.com>`. No `IDC-Trust-Review` author or committer appears in
  the reviewed union.
- Native-Windows exact-POSIX emission remains not verified on this macOS seat.
- Slice-5 regeneration, pin rebind, shelf/WV semantic wiring, U-36, PR, merge,
  GitHub approval, launch, and later ceremony gates were not performed or
  approved.
- No timing or performance claim was produced or validated.
- No implementation code was modified by this reviewer. No workflow, ruleset,
  `ops/mission/state.json`, credential, PR, merge, or GitHub approval event was
  touched.

## NUC consequence

**APPROVED HEAD FOR THE NUC:
`c3dc53ee4169ae879647fcb74e7bb524488653ed`.**

The native-Windows seat must check out that exact commit and recompute the
product pair before generating evidence. It must STOP unless it obtains:

```text
productContentSha256 =
0b6239c204f893ed627f59c937d83b6cd5c56832bc0260e31a63775d8d568158
productPathCount = 1544
```

It must then prove the emitted JSON source value is exactly
`scripts/garnet_launch_readiness_status.py`, with forward slashes and no host
root. Only after that separate NUC leg may slice 5 consume a Windows
regeneration.

This approval names the cure commit, not the later Request-10 packet tip. The
packet tip has the same product pair because its added bytes are review
artifacts, but it is not substituted for the exact implementation head under
review.

## Reviewer stdout summary

Cross-family Verdict 10 APPROVES the bounded U-31 cure at exact NUC head
`c3dc53ee4169ae879647fcb74e7bb524488653ed`: the authorized one-line
repo-relative POSIX construction is exact, `source` is retained, all four traps
and the RED-before-cure pass, all 31 Lane-0 paths remain digest-included, the
lawful product pair is `0b6239c2…/1544` at cure and packet tip, F3 is an
unchanged slice-5 ledger baseline, Python isolates only the standing WV pin
state, Cargo is 2199/0 parity, and the bounded security scan reports zero
findings. Native-Windows exact-POSIX emission remains a separate mandatory NUC
leg at this approved head before slice 5 consumes a Windows regeneration;
S-SEC-1 carries.
