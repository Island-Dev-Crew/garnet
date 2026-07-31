# Lane 0 Repair 3 — Review Verdict 02: B1 disclosure + F1 checker cure

request: `ops/lane0-repair3/review/02-b1-f1-request.md`
reviewer: Claude Code on **Claude Fable 5** (`claude-fable-5`, Anthropic)
reviewer_machine: `Pulses-MacBook-Air.local`; Darwin 25.5.0; arm64 (Apple M5);
  fanless — functional and byte-level claims only, no timing claims
implementer_identity_as_found: OpenAI Codex, GPT-5-based agent (exact version
  unavailable), on `Hughs-MacBook-Pro.local` (macOS 26.5 / Darwin 25.5.0,
  arm64) — cross-family separation holds
branch: `mission/l0-repair3` (fork; explicit refspec; zero `refs/pull/*`)
reviewed_head: `608bae46bb4f554c7ffd455f01ef4cbf44faee3c`
  (tree `7913546b516d6ef9144bdc62c02eddf9c209055f` — both reproduced)
integration_base: `efd4f6bae8b3afaba74594e57944b2548142aeae` (still exact
  `origin/main`)
chain_since_verdict_01: `d99b1ca` → `cbcf0a1` (F1 RED) → `9d6baef` (F1 cure)
  → `a4ed09b` (B1 disclosure) → `2564b17` (request) → `608bae4` (ID
  amendment); five single-parent commits, all authored/committed
  `OpenAI Codex <codex@openai.com>`; `IDC-Trust-Review` appears nowhere as an
  author, committer, or trailer (its one occurrence in the range is the
  request document's own seat-description prose)
swept_at: `2026-07-31T16:06:53Z` boot; Friday midday America/Chicago — the
  Sabbath fence never armed during this review; the push was not held
scope: B1 and F1 (with F2 riding the same edit); Verdict 01's U-25 analysis
  is not reopened — but the CHECKER was re-audited as a new artifact, and
  this approval-of-record expressly covers its new bytes and tests
verdict: **BLOCKED on exactly one blocker (B1 — a one-clause provenance
  misattribution in the U-47 register row; records-only cure). Every other
  requested check passes: the F1 cure is verified empirically, F2 is cured
  exactly, no pin moved, and the B1 disclosure is mechanically exact at the
  final head.**

## Provenance discipline

As in Verdict 01, every item is marked INDEPENDENTLY FOUND or
INHERITED-AND-CONFIRMED by where THIS seat got it.

## F1 — the checker cure, re-audited as a new artifact (INHERITED-AND-CONFIRMED cure; residuals INDEPENDENTLY FOUND)

The checker and its test file are the only non-`ops/lane0-repair3/**` changes
since `d99b1ca` (+20/−2 and +20/−0). Full-file re-audit, not a diff skim:

- **The cure is exactly the cure for the channel Verdict 01 named.**
  `_git()` gains `attr_source`; the single attribute-consuming invocation
  (`git grep -I -l -z -e '\r' <commit> --`) passes
  `attr_source=commit` where `commit` is the `rev-parse --verify`-validated
  40-hex SHA of the scanned ref — so `GIT_ATTR_SOURCE` binds attribute
  lookup to the same object the scan reads. The two remaining git calls are
  `rev-parse` (no attribute consultation).
- **Empirically closed** (git 2.50.1, fixture repo): a committed CRLF blob
  hidden by a dirty worktree `bad.txt -diff` line is NAMED by the new
  checker and falsely-greened by the `d99b1ca` checker on the identical
  fixture — the RED/GREEN pair reproduces the exact Verdict 01 channel. A
  hostile caller environment presetting `GIT_ATTR_SOURCE` to a different
  commit is overridden by the checker's explicit binding.
- **RED discipline verified by execution, not by reading evidence**: at
  `cbcf0a1` the new test
  `test_exact_commit_scan_ignores_dirty_worktree_attributes` fails
  (`AssertionError: True is not false`; the checker there has zero
  `GIT_ATTR_SOURCE` occurrences); at `9d6baef` and at the reviewed head the
  same suite is 7/7 OK. The failing test FUNCTION is byte-identical between
  RED and cure — nothing was weakened. (Process note, LOW: the cure commit
  also adds one fixture line to a different test — the F2 pin — so the
  test-file diff is not zero; F2 had no separate RED of its own.)
- **Adjacent channels probed empirically**: committed `-text` (the fence's
  actual attribute) does NOT affect `grep -I`; textconv is not applied;
  `core.bigFileThreshold` does not blind the scan; `GIT_NO_REPLACE_OBJECTS`
  is set on every call; attributes override the NUL heuristic in both
  directions, and the heuristic itself depends only on commit-bound bytes.
- **No-count contract still holds** in the new bytes: every numeric literal
  is plumbing; no count field exists; the tests assert none appears.

### Residuals (both INDEPENDENTLY FOUND, neither blocking)

1. **git < 2.40 silent fallback.** `GIT_ATTR_SOURCE` was introduced in Git
   2.40; an older git never reads the variable, so the checker silently
   degrades to pre-cure behavior — no version check, canary, or
   effectiveness assertion exists, and the unit test detects the
   degradation only where the suite itself runs. Every fleet seat currently
   runs modern git, so this is latent. Recommended cure rides proposed
   U-48's Lane 3 remit (resolved-tool binding): fail closed below git 2.40
   or assert the binding took effect.
2. **Committed diff-family attributes remain an attribute-visible design
   boundary.** A `-diff`/`binary` attribute committed AT the scanned commit
   hides a blob from the scan (empirical), and a committed `diff=driver`
   line plus LOCAL `diff.<driver>.binary=true` config completes the same
   hide. The hiding line is part of the scanned commit's own reviewable
   `.gitattributes` — head has zero diff-family lines — so this is a
   documented boundary, not a defect; recorded so no future seat mistakes
   it for coverage.
3. (LOW, records) The `EXCLUSIONS` tuple and docstring still advertise
   `("ops/**/evidence/**", "proofs/**")`, narrower than the cured
   `_excluded` behavior which also fences literal `ops/**/evidence` blobs;
   the JSON output therefore under-describes the actual exclusion by one
   path class. One-line precision fix whenever the checker is next touched.

## F2 — cured exactly (INHERITED-AND-CONFIRMED)

`"evidence" in parts[1:-1]` → `"evidence" in parts[1:]`. Empirical fixture:
`ops/x/evidence` and `ops/evidence` (literal files) are now excluded;
`ops/x/evidence-notes/y`, `ops/x/foo.txt`, and top-level `evidence/z.txt`
remain in scope. That is precisely the union of the two pre-existing fence
lines (`ops/**/evidence -text`, `ops/**/evidence/** -text`) — the Verdict 01
divergence is closed with no over-exclusion, and `.gitattributes` itself is
unchanged across the cure.

## Pins and boundary immobility (INHERITED-AND-CONFIRMED)

- `scripts/smoke_garnet_minimum_shelf.py` is byte-identical to `d99b1ca`
  (same blob OID `5ec61b96…`), and line 51 still pins
  `b8b22a96534aa11b02d5d72e5baf2a6cc5dc9481ea5ad85a5441728ffa8d2e5f`. The
  entire B1 ruling rested on this; it held.
- The range changes exactly nine paths: seven `ops/lane0-repair3/**`
  records plus the two text-byte-policy scripts. Zero changes to
  `.github/`, `proofs/**` (the `windows-20260628-lane2` tree OID is
  identical, pinning every sealed blob), `.gitattributes`, `AGENTS.md`,
  the provenance module, the WV reporter, or any Rust/Cargo path. All
  candidate constants unchanged.

## B1 — disclosure is mechanically exact at the final head (INHERITED-AND-CONFIRMED; head pair recorded here)

Product pairs recomputed at every boundary by BOTH the repository provenance
function and an independent raw reconstruction (methods agree at all seven
revisions):

```text
5e5a24c  cd9c080ff62483721abd20aad19f666f30adb7c35c1c16b2fa08540193ac4263 / 1553
d99b1ca  e13d0775f2249c0ce44353fda699c8a3f519e15781be37557baea15ff99d503a / 1554
9d6baef  20830394be5d37a39c622fed252b30a213fe0c0420f2a026567b852c80a93707 / 1555
a4ed09b  1d27fa765adb6ac50af2ddb6edd028538217511e211d5cb79b813f7f6be35bae / 1558
2564b17  f7943a0a16fe7d174cac8d01e9584638b31d2b661ab285fd71789856bec527d6 / 1559
608bae4  bcbae1ea664542498e1b1308c167961486e0ecc096619c9ad9b3ee7836753196 / 1559  ← reviewed head, recorded HERE
```

Every pair the record discloses matches recomputation. The reviewed head's
own pair appears in no committed file — correctly, per the record's
no-self-SHA principle — and the disclosure states the MECHANISM (every
`ops/lane0-repair3/**` record is digest-included and moves the live pair;
boundaries disclosed, never collapsed) plus the named freeze/rebind and NUC
WV-6 re-acceptance successors. Nothing stale is presented as current; the
`2564b17` and `608bae4` pairs above make the record whole for the
freeze/rebind slice. The 14-path enumeration for `efd4f6b..5e5a24c`
reproduces exactly from the tree under the frozen exclusion tuple. The
expected-red tripwires reproduce at the reviewed head: shelf smoke gate
exit 1 reporting live `bcbae1ea…/1559` against pinned `ea38d354…/1544` and
the `.gitattributes` pin mismatch; WV-6 test fails `'partial' !=
'accepted'`, data-driven (reporter and test byte-identical to base).

## BLOCKER B1 — U-47 attribution clause misstates provenance (INDEPENDENTLY FOUND; records-only cure)

The amended U-47 register row reads: *"Independently found by this
implementer seat while executing the corrected battery instruction."* The
commit record contradicts the marking:

```sh
git log --format='%h %aI %s' d99b1ca cbcf0a1 --no-walk
# d99b1ca 2026-07-31T05:09:11-05:00  Verdict 01 — names BOTH standing base
#          failures (adoption-surface pointer; release-assets sha256 line)
# cbcf0a1 2026-07-31T07:32:40-05:00  first commit of the cure chain
```

Verdict 01 — the record this cure chain demonstrably executes against —
named both failures at 05:09; every implementer battery action postdates it
by hours. Under this lane's own marking law ("what matters is where you got
it," and a finding already in a record you have read cannot be marked
independent), the implementer seat's correct marking is
**inherited-and-confirmed from Verdict 01**, with the U-47 REGISTRATION and
Lane 3 routing remaining the implementer's genuine contribution. Evidence
10, to its credit, openly reconciles Verdict 01's counts — the record as a
whole tells the truth; the register row's marking does not. The row's
denial of CHAT-seat origin is accurate and unaffected; U-50's chat-seat
attribution is verified correct.

Cure (one records commit): amend the U-47 disposition clause to read that
the two standing failures were first recorded in Verdict 01
(`d99b1ca`), independently reproduced and registered by the implementer
seat. Nothing else in the register requires change.

This is held to the same bar Verdict 01's B1 set: a governance register
under active review does not cross a gate carrying a marking its own
chronology refutes.

## Battery — toolchain-recorded (INHERITED-AND-CONFIRMED numbers; one artifact INDEPENDENTLY FOUND and root-caused)

Seat toolchain, recorded per proposed U-48: default `rustc/cargo 1.94.1`
(below MSRV 1.95), `+1.95.0` available and used for all cargo; battery under
the pinned venv (Python 3.14.5, PyYAML 6.0.3, jsonschema 4.26.0), symmetric
no-build state.

```text
base efd4f6b: 1,130 tests · 5 failures · 5 skipped   [corrected — see artifact note]
head 608bae4: 1,141 tests · 6 failures · 5 skipped   (+11 tests = the two new test files)
predecessor-only: EMPTY
successor-only:  exactly test_current_repository_tracks_wv6_acceptance_and_wv7_pending
                 (WV-6, 'partial' != 'accepted') — the disclosed B1 tripwire
```

Shared failures on this seat: three focused-cargo gate tests (bare `cargo`
below MSRV — green under `+1.95.0`; the U-48 divergence, applied as
instructed without treating the implementer's 2 → 3 as a discrepancy) plus
the two standing origin/main failures (U-47). Artifact note, in the open:
one intermediate run showed a sixth shared failure
(`test_current_backlog_gate_passes`, an ancestry assertion). Root cause was
this seat's own verification tooling — depth-limited fetches during the
fork-head ID sweep left 15 graft entries in the audit clone's
`.git/shallow`, clipping ancestry walks. After repairing the clone state the
same suite passes 16/16 at the same base commit. The artifact appeared
identically at both revisions and never touched the differential; it is
disclosed because a battery number was reported to this seat with it
embedded. Implementer-seat numbers (2 → 3 on native 1.95.0) reconcile
exactly under the machine-bound reading.

## Records and finding IDs (INHERITED-AND-CONFIRMED, with scope caveat)

- The ID amendment (`608bae4`) touches three record files, appends-forward
  in substance (inline SUPERSEDED marker with original claim preserved;
  prior tip `2564b17`/tree `6dc91aa1…` pinned inside the amended request),
  and rewrites no history — `2564b17` remains the parent.
- Register verified at head: U-25 IMPLEMENTED—REVIEW REQUIRED; U-45
  PROPOSED; U-47/U-48/U-49/U-50 all PROPOSED—DEFERRED with Lane 3
  addresses; Lane 2C retains U-46 (harness placement); U-49 = renderer
  mojibake (ex-F3); U-50 = allocator finding, chat-seat-found — attribution
  verified correct.
- Independent collision sweep: the fork holds exactly 15 `mission/*` heads
  (of 461 heads total); all 15 fetched and grepped — U-49/U-50 assignments
  exist only on this branch, and Lane 2C's U-46 is the doctrine finding.
  Scope caveat (recorded, non-blocking): the register row's "fifteen
  current `mission/*` fork heads" wording is accurately scoped, but the 446
  non-mission heads were never swept by anyone; the U-50 allocator is the
  systemic cure and this caveat belongs in its eventual Lane 3 design.
- F3/F4/F5 received no out-of-scope byte cure: the renderer and its
  mojibake bytes are untouched at head, sealed paths untouched, evidence
  05/06 not rewritten.

## Gates run at the reviewed head

lane0 closeout · MSRV · frozen backlog · capability scope · evidence
integrity · text-byte policy (PASS at all five later heads; four-path RED
reproduced at base) — all exit 0; trust-kernel rolling gate correctly
fail-closed REVIEW REQUIRED, its trust-path digest
`sha256:47b2676b…` matching the request AND independently recomputed from
raw git objects (footnote for future seats: the digest covers only the
trust-kernel-touched paths — 3 here — not all 22 changed paths);
agent-contracts 24/24 + tests 6/6; `cargo +1.95.0 fmt --all -- --check`
clean; `git diff --check` clean over the full range; checker tests 7/7;
Lane 2B renderer byte tests 4/4; shelf smoke and WV-6 in their disclosed
expected-red states. Zero `.rs`/`Cargo.*` changes in the range.

## Scope and not-verified

- This seat fixed nothing, performed no PR, approval, merge, acceptance,
  rebind, or NUC action; the writes are this verdict and one journal line.
- The implementer did not author this verdict.
- git < 2.40 behavior was reasoned from the documented introduction of
  `GIT_ATTR_SOURCE` (no pre-2.40 binary exists on this seat to execute);
  every other channel claim above was demonstrated by execution.
- No timing claims (fanless seat).

## Consequence

**BLOCKED on the single U-47 attribution clause; records-only cure.** On
its landing: B1 (Verdict 01) is CURED — the disclosure is mechanically
exact, with this verdict recording the final-head pair
`bcbae1ea…/1559` to make the record whole; F1 is CURED at `9d6baef` with
the channel closed empirically and two residuals routed to U-48's remit;
F2 is CURED exactly; no pin, constant, workflow, or acceptance state moved.
The freeze/rebind and NUC WV-6 re-acceptance remain the named, separately
reviewed successors. Approval then authorizes only Jon's merge decision.

## Reviewer stdout summary

Cross-family Verdict 02 (Claude Fable 5, Anthropic, MacBook Air; implementer
Codex GPT-5-based on Hughs-MacBook-Pro) verifies the whole cure package by
execution: the GIT_ATTR_SOURCE binding closes exactly the Verdict 01 F1
channel (the old checker false-greens the fixture the new one names), RED at
`cbcf0a1` and GREEN at `9d6baef` reproduce with the failing test function
byte-identical, F2's exclusion now matches the fence union precisely, the
shelf reporter is OID-identical with line 51 still pinning `b8b22a96…`, and
all six disclosed product pairs verify by two agreeing methods with the
reviewed head's own pair — `bcbae1ea…/1559` — computed and recorded here.
The battery is toolchain-bound per U-48 (5→6 on this 1.94.1-default seat,
2→3 on the implementer's 1.95.0 seat, successor-only exactly the disclosed
WV-6 tripwire), with one self-inflicted clone-state artifact found,
root-caused, repaired, and disclosed. One blocker only, independently
found and records-only: the U-47 register row marks as
implementer-independently-found two standing failures that Verdict 01 had
named hours earlier in the very record the cure chain executes against —
the marking law this fleet runs on requires the row to say
first-recorded-in-Verdict-01, and the register should not cross a gate
otherwise. Residual findings: no git-version floor under the new
GIT_ATTR_SOURCE binding (latent, routed to U-48), the committed-attribute
design boundary, the narrower-than-behavior advertised EXCLUSIONS, the
F2-without-RED process note, and the mission-heads-only scope of the U-50
collision sweep. Verdict authored under this seat's own identity.
