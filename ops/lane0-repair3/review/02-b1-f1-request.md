# Lane 0 Repair 3 — Review Request 02: B1 disclosure + F1 checker cure

## Seats and identity

- Actual implementer: OpenAI Codex, GPT-5-based agent; exact version
  unavailable, on `Hughs-MacBook-Pro.local` (macOS 26.5 / Darwin 25.5.0,
  arm64).
- Requested reviewer: Claude Code on Claude Fable 5 (Anthropic), on a
  different machine. Verdict 01 was produced on
  `Pulses-MacBook-Air.local` (Darwin 25.5.0, arm64 Apple M5).
- Review carrier: `IDC-Trust-Review` only.
- Merge authority: Jon (`IslandDevCrew`) only.
- The implementer is neither carrier nor merge authority and does not grade
  this packet or author its verdict.

## Exact boundary

- Integration base: `efd4f6bae8b3afaba74594e57944b2548142aeae`
  / tree `e9bce10421c1eac2a514291212b87d61a5289037`.
- Verdict 01 record commit: `d99b1ca9a9f0a6da0d0c38d57471b565b18f0120`
  / tree `68bf27813669b2a470014d2838f841e01a48fa29`.
- F1 RED commit: `cbcf0a11b80b5cacec1c914686d11ee5aac87c06`
  / tree `ea4f9813863192c88c6240cb194120738ae80721`.
- F1 implementation head: `9d6baef54971b3058648d6f9d8ee4f35f49bb0e9`
  / tree `7df00be6868f35cc3e91ddca69e5feed1bd52451`.
- B1 disclosure and findings head:
  `a4ed09b724dd0a14d4219febf6a50ae1c5166540` / tree
  `47d83822bd4f950f08751d8d306c1e79645516f5`.
- Prior Request 02 tip before the finding-ID collision amendment:
  `2564b17c92ce05a19bd04fba94ddabfb10b0169f` / tree
  `6dc91aa1fc919e84ffbb88e0abe7752cf5d5fb1e`.
- Trust-path digest at the disclosure head:
  `sha256:47b2676bafcaf41d42404bf421194a582a7dbeaf9c83a667d87fd83a9ac87775`.
- B1 evidence: `ops/lane0-repair3/evidence/07-b1-standing-red.txt`.
- F1 RED: `ops/lane0-repair3/evidence/08-f1-worktree-attribute-red.txt`.
- F1 GREEN: `ops/lane0-repair3/evidence/09-f1-git-attr-source-green.txt`.
- Battery differential:
  `ops/lane0-repair3/evidence/10-python-battery-differential.txt`.

This request file is itself an `ops/lane0-repair3/**` record and therefore
digest-included under the current frozen predicate. One records-only
finding-ID amendment commit succeeds `2564b17`; the reviewer must fetch and
bind that exact branch tip as well as the implementation/disclosure heads
above. A commit cannot contain its own SHA, so no self-referential SHA is
asserted inside the amendment. U-17 remains OPEN and governance freeze remains
not armed.

## B1 — standing-gate movement is now disclosed

No pin or acceptance state is changed in this cure. The failures below are
correct fail-closed tripwires.

At the Verdict 01 reviewed head `5e5a24c`, repository provenance recomputes:

```text
live product pair  cd9c080ff62483721abd20aad19f666f30adb7c35c1c16b2fa08540193ac4263 / 1553
pinned pair        ea38d3547eafa7f56141454df50eaf8084dbc66ee9573ebdd67623df8be97bbe / 1544
live .gitattributes SHA-256  b2a14050a850391f8ed1c788f9a6a66155a423ebceb3bb4722478dcaec97dd1b
pinned .gitattributes SHA-256 b8b22a96534aa11b02d5d72e5baf2a6cc5dc9481ea5ad85a5441728ffa8d2e5f
```

The 14 digest-included changes in `efd4f6b..5e5a24c` are:

1. `.dogfood/windows-audit-goal.json`
2. `.dogfood/windows-core-audit.json`
3. `.gitattributes`
4. `AGENTS.md`
5. `D_Executive_and_Presentation/garnet-website.html`
6. `ops/lane0-repair3/FINDINGS.md`
7. `ops/lane0-repair3/U25_SCOPE.json`
8. `ops/lane0-repair3/evidence/05-u25-scope-red.txt`
9. `ops/lane0-repair3/evidence/06-u25-green.txt`
10. `ops/lane0-repair3/journal.md`
11. `ops/lane0-repair3/review/01-u25-request.md`
12. `scripts/garnet_text_byte_policy_status.py`
13. `scripts/test_garnet_lane2b_sotu_bytes.py`
14. `scripts/test_garnet_text_byte_policy_status.py`

Later records are also digest-included and are disclosed by boundary rather
than folded into that original 14-path statement:

```text
d99b1ca reviewer record  e13d0775f2249c0ce44353fda699c8a3f519e15781be37557baea15ff99d503a / 1554
9d6baef F1 head          20830394be5d37a39c622fed252b30a213fe0c0420f2a026567b852c80a93707 / 1555
a4ed09b disclosure head  1d27fa765adb6ac50af2ddb6edd028538217511e211d5cb79b813f7f6be35bae / 1558
```

At `a4ed09b`, `smoke_garnet_minimum_shelf.py --gate` exits 1 with the
`1d27fa76…/1558` versus `ea38d354…/1544` mismatch and the same
`.gitattributes` pin mismatch. The WV status test exits 1 only for WV-6:
`test_current_repository_tracks_wv6_acceptance_and_wv7_pending` reports
`'partial' != 'accepted'`.

Named successor, not performed or authorized here: **Lane 0 Repair 3
post-record freeze/rebind and NUC WV-6 re-acceptance**. The freeze/rebind slice
updates the four candidate constants and the `.gitattributes` per-file pin
under its own review. The NUC then re-accepts WV-6 at that exact reviewed
candidate under a separate review. This request performs neither action.

## F1 — checker changed after Verdict 01

The pre-fix fixture commits `bad.txt` with CRLF bytes, resolves that commit,
then dirties only the fixture worktree `.gitattributes` with `bad.txt -diff`.
Before the cure, the exact-commit scan false-greens and the new assertion fails:

```text
AssertionError: True is not false
Ran 1 test
FAILED (failures=1)
```

At `9d6baef`, the checker passes the resolved commit through
`GIT_ATTR_SOURCE=<commit>` for `git grep`. The same fixture then names
`bad.txt`; 7/7 checker tests pass, the real-tree gate passes, and 4/4 Lane 2B
renderer byte tests pass.

Verdict 01 verified the earlier checker. It does not cover the F1 checker
bytes. Request 02 explicitly asks the reviewer to re-audit the checker,
environment scoping, exact-commit attribute binding, and fixture.

## Battery — exact machine-bound differential

Both fresh detached clones used Python 3.14.5 in the same isolated venv with
PyYAML 6.0.3 and jsonschema 4.26.0, in symmetric no-build state.

Implementer machine resolved toolchain:

```text
rustc 1.95.0 (59807616e 2026-04-14)
cargo 1.95.0 (f2d3ce0bd 2026-03-21)
```

Base `efd4f6b`: 1,130 tests, 2 failures, 3 skipped. Both are **pre-existing
red on origin/main**, not introduced by this branch:

1. `test_repo_and_site_point_to_the_adoption_surface_reporter`
2. `test_tag_release_publishes_unified_checksummed_assets`

Head `9d6baef`: 1,141 tests, 3 failures, 3 skipped. The shared set is exactly
the two above; base-only/predecessor-only is empty. The head-only/successor-only
set is exactly:

1. `test_current_repository_tracks_wv6_acceptance_and_wv7_pending (WV-6)`

That one branch-introduced failure is the expected-red B1 disclosure above.

The reviewer Air's recorded default resolves to `rustc 1.94.1` / `cargo
1.94.1`, below the repository MSRV. Its Verdict 01 battery therefore has
three additional focused-cargo failures at both base and head; those three
are green under explicit `+1.95.0`. Hugh's machine runs 1.95.0 natively.
Neither result is generalized or represented as wrong.

## F2–F5 and new proposed findings

- F2 is cured in the same checker/test edit at zero additional ceremony cost:
  a literal `ops/.../evidence` file now follows the existing evidence fence;
  non-evidence ops paths remain checked.
- F3 is not fixed. Pre-existing renderer mojibake is proposed U-49, addressed
  to the Lane 3 ops/documentation sweep.
- F4 is recorded without changing sealed bytes: three sealed paths retain
  inert `eol: lf` while `text` is unset; CR must be stripped from the CRLF
  manifest stream only when invoking `shasum -c`.
- F5 is recorded without rewriting historical evidence: `ref: HEAD` is
  cosmetic because commit/tree fields bind evidence 05/06.
- Proposed U-47 routes the two standing origin/main battery failures to Lane 3
  integration-baseline reconciliation; this lane does not cure them. They
  were independently found by this implementer seat during the corrected
  battery run, not by the chat seat.
- Proposed U-48 routes resolved-toolchain binding and explicit `+1.95.0` cargo
  gate invocation to Lane 3 procedural/gate hardening; no workflow changes
  occur here.
- Proposed U-50 records the chat-seat-found U-46 collision and routes a
  cross-lane ID allocator to the Lane 3 governance surface. Lane 2C retains
  U-46; U-49 and U-50 had no other assignments across the 15 current
  non-main `mission/*` fork heads checked before this amendment.

## Fresh gates at the disclosure head

All five mandatory truth floors pass at `a4ed09b`:

```text
lane0 closeout: PASS; evidence 22/22; ledger 37; denominators 4/4; HOLD; band 3
MSRV: ok=true; Rust 1.95; 18 active manifests
frozen backlog: ok=true; 8 entries
capability scope: ok=true; 2/2 bounded claims
evidence integrity: ok=true; 38/38 bundles
```

Additional results:

```text
text-byte policy: PASS at a4ed09b / tree 47d83822
text-byte checker tests: 7/7
Lane 2B renderer byte tests: 4/4
agent contracts: 24/24; contract tests 6/6
cargo +1.95.0 fmt --all -- --check: PASS
git diff --check origin/main..HEAD: PASS
trust-kernel gate: REVIEW REQUIRED; structured review record missing
```

## Requested reviewer checks

1. Recompute the product pair at `5e5a24c`, verify the 14-path enumeration,
   and reproduce the later disclosed pairs without treating any as accepted.
2. Confirm no pin, candidate constant, WV acceptance state, workflow, or gate
   merge rule changed in this cure.
3. Reproduce F1 RED at `cbcf0a1`, then audit and reproduce F1 GREEN at
   `9d6baef`, including a dirty worktree attribute override against an exact
   commit.
4. Confirm F2 aligns only the pre-existing evidence fence and does not exclude
   neighboring non-evidence ops paths.
5. Reproduce or audit the machine/toolchain-bound battery differential and
   confirm WV-6 is the sole head-only failure on the 1.95.0 implementer seat.
6. Confirm Lane 2C retains U-46; U-47/U-48/U-49/U-50 remain
   proposed/deferred; the U-47/U-50 attributions are correct; and F3–F5
   received no out-of-scope byte cure.
7. Audit the exact fetched request tip. The checker itself changed after
   Verdict 01, so an approval must expressly cover its new bytes and tests.

Verdict destination:
`ops/lane0-repair3/review/02-b1-f1-verdict.md`

## STOP

The implementer stops after pushing Request 02. No PR, merge, tag, release,
acceptance emission, pin rebind, or later slice is performed.
