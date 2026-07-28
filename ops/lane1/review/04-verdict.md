# Lane 1 Phase 0 — Independent Review Verdict 04

reviewer: Codex GPT-5.6 Sol (independent, non-implementer)
reviewed_head: 72ae0246fb448ce33d689b1b80eb783497a7f215
reviewed_tree: 3c98ba05eb756377049325942842164f5d98910b
approved_head: 72ae0246fb448ce33d689b1b80eb783497a7f215
request: ops/lane1/review/04-request.md (02002c462c9dc9319c18e7abcf4339175fef792f)
request_addendum: ops/lane1/review/04-request-addendum-u33.md (7dd4846e8dc2796643626fec798f6a932b2e6035)
swept_at: 2026-07-27T21:39:48Z
machine: Hughs-MacBook-Pro.local; macOS 26.5; Darwin 25.5.0 arm64
model: Codex GPT-5.6 Sol
implementer_self_report: Claude Code Fable 5
verdict: APPROVE
verified_identity: head/tree/diffstat/digests reproduced: yes
lineage: merge commits from base: 0 | canonical review-record commits in reviewed range: 0

Request 04 and its U-33 addendum are approved together for the exact frozen
candidate `72ae0246fb448ce33d689b1b80eb783497a7f215`. F1 and the
ceremony-scoped F2 attribution defects are genuinely cured. The ceremony
seat's whole-page ruling is honored: the remaining pre-existing units are not
Lane-1 blockers and remain launch-blocking U-33 work in Lane 3. This approval
does not weaken U-31, U-32, U-33, or S-SEC-1.

The native-Windows NUC WV-6 acceptance must run at this exact approved head
and nowhere else.

## Boot, floor, and sweep

- Fresh clone with `core.autocrlf=false`; remote `origin` is
  `Island-Dev-Crew/garnet`; no `refs/pull/*` were fetched.
- `origin/main`:
  `68317ae258327aade47fc2c07b7b5b580ec7c6ea`, tree
  `29191aa0e17121c08b73fe12578ee4464559e2ba`.
- Main truth floor: lane-0 closeout PASS (22/22), MSRV PASS (1.95),
  frozen backlog PASS, and rolling-review v2 diagnostic PASS with
  base=head=`68317ae...`, `trust_kernel_touched=false`, and no problems.
- Fork sweep: 13 `mission/*` branches. Request 04 and its immutable addendum
  were the unanswered packet for this lane. Request 03 was already expressly
  disposed by Verdict 02.
- Fork tip at freeze:
  `7dd4846e8dc2796643626fec798f6a932b2e6035`.

## Identity, lineage, scope, and record law

The exact candidate reproduces:

```text
head  72ae0246fb448ce33d689b1b80eb783497a7f215
tree  3c98ba05eb756377049325942842164f5d98910b
parent 14a5e45628eec5210c23170211e94bacf087db3f
```

The candidate commit itself is:

```text
docs/why.html | 10 +++++-----
1 file changed, 5 insertions(+), 5 deletions(-)
```

- Merge-base with `68317ae...` is exactly `68317ae...`; zero merge commits
  occur in the range.
- `garnet-cli/src` has zero changed paths from the prior reviewed candidate
  `4f5ebb8` through `72ae024`.
- No Rust, Python, workflow, dependency, lockfile, sealed input, gate,
  reporter, capability implementation, or canonical `*.review.json` record
  changed in this cure.
- Request 04 was added exactly once by `02002c4...` and never modified.
  Its U-33 addendum was added exactly once by `7dd4846...` and never modified.
- The two commits above the frozen candidate are review artifacts only:

```text
02002c4: M ops/lane1/BLOCKED.md
         M ops/lane1/journal.md
         A ops/lane1/review/04-request.md
7dd4846: M ops/lane1/BLOCKED.md
         M ops/lane1/journal.md
         A ops/lane1/review/04-request-addendum-u33.md
```

The addendum did not amend, rebase, or move the candidate. The candidate
head/tree/digest remain the objects stated above.

## F1/F2 disposition and real CLI dispatch

All four literal `--evidence` mentions are now explicitly future work:

```text
391 planned flag, not in the CLI today
398 planned flag; no such build flag exists in the shipping CLI today
495 planned flag
544 planned --evidence layer
```

No `--evidence` occurrence reads as shipping. The compiled binary confirms
the boundary:

```text
$ garnet build --evidence probe.garnet
unknown build flag: --evidence
exit 2
```

Every CLI surface cited by the page exists in the real dispatcher and appears
in compiled `garnet --help`: `build`, `caps`, `check`, `diff-caps`, `run`,
`seal`, and `test`.

Both build-attribution lines named by Verdict 02 are cured:

- line 399 now says “no evidence without a check that proves it”;
- line 449 now attributes checking to `garnet check` and expressly says it is
  not yet wired into `garnet build`.

Reasons #2 and #3 agree: proof is on the check path; seal attests the declared
surface without re-running the checker; binding seal to a passing check is
unshipped work.

The direct fixture probe matches those statements:

```text
garnet check caps_violation.garnet  -> exit 1, undeclared transitive fs
garnet build caps_violation.garnet  -> exit 0
garnet seal caps_violation.garnet   -> exit 0, aggregate=[], main caps=[]
```

F1 and the authorized F2 attribution cure therefore pass. The behavior itself
remains U-32, not a falsely restored shipping claim.

## Findings accuracy: U-32 and U-33

U-32 is recorded without softening:

- it owns a separate lane and requires full ceremony;
- it must land before the Lane 4 frozen candidate;
- build must route through the checker and seal must require a passing check,
  with a committed three-command trap;
- no public build-time enforcement claim may be restored until U-32 is cured.

U-33 is recorded without softening:

- it belongs to Lane 3/public truth as one coherent claim-ledger pass, not
  piecemeal wording changes;
- it remains launch-blocking;
- it must land before launch and before the CRA positioning push;
- it also preserves the prohibition on restoring build-time enforcement
  anywhere before U-32 is cured.

The seat's factual basis reproduces against landed main: the exact
case-sensitive counts for “Enforcement by construction” are 4/4 and for
“A type cannot” are 3/3 at main/candidate; the unqualified hero/Turn/reason
units are already live. The candidate changes only the five scoped
attribution lines and strictly improves the serving page. Per the explicit
ceremony ruling, the remaining whole-page units are deferred U-33 findings,
not blockers in this verdict.

## Pinned integrity

```text
python3 -I scripts/garnet_capability_scope_status.py --gate
ok=true
enforced_claim_count=2
enforced_claim_expected=2
enforced_claim_hashes_match=true
current_truth_missing=[]
cited_anchors_missing=[]
forbidden_hits=[]
stale_truth_hits=[]
```

- The two complete `<b>enforced:</b>` lines are byte-identical to `d7430c2`.
- `test_entry_authority` and `scope_shadowing_parity` each occur once.
- Both canonical snippets remain verbatim.
- The focused suite ran 10 tests and passed.
- `66.7%`, `50.0%`, `83.3%`, `62.5%`, and `37.5%` have zero hits on the page.

## Freeze facts and differential

An independent raw `git --no-replace-objects ls-tree -r -z` construction,
using the frozen exclusions, reproduced:

```text
product_content_sha256 =
99c3f2701f0a19b25f2f56e5cca8f59e9f719c8dd03b1bc5f14401cebeb0c3ab
product path count = 1578
```

The repository provenance module independently returns the same pair.

The full Python battery ran serially at candidate and base from the same
disposable environment with exact `PyYAML==6.0.3`:

```text
base 68317ae:      Ran 1123 tests; 3 failures; 3 skipped; exit 1
candidate 72ae024: Ran 1123 tests; 4 failures; 3 skipped; exit 1
```

The three base failures reproduce unchanged:
`test_repo_and_site_point_to_the_adoption_surface_reporter`,
`test_tracked_ledger_matches_renderer_byte_for_byte`, and
`test_tag_release_publishes_unified_checksummed_assets`.

The sole candidate-only failure is the expected exact-product WV-6 freeze:

```text
test_current_repository_tracks_wv6_acceptance_and_wv7_pending
AssertionError: 'partial' != 'accepted'
```

The base WV-6 reporter is accepted 5/5. At the candidate it is partial 5/5
with all five artifacts and exactly one finding:

```text
product content digest mismatch
(99c3f270... != 2f8c9ad8...)
```

No Shelf/WV script, contract, proof, or artifact changed. Native-Windows WV-6
must now rebind this exact approved product head.

Cargo is exact parity:

```text
rustup run 1.95.0 cargo test --workspace --no-fail-fast
base:      2199 passed; 0 failed; 6 ignored; exit 0
candidate: 2199 passed; 0 failed; 6 ignored; exit 0
```

## Marker provenance carry-forward

Round 1's independent marker result carries unchanged. Neither the candidate
nor either artifact commit above it changes the registry or landed markers.

- The registry remains a sorted, unique list exactly enumerating the two
  landed-marker files.
- Lane-1 landing `68317ae...` remains a first-parent main landing with matching
  tree, landing-edge added record, trust-content digest
  `sha256:b697d2fec3a610f212616f3a790660bcd16061e4b42fa5f53e5417ae82035a6a`,
  and `is_trust_kernel`-derived touched paths.
- Lane-2B landing `41d6ced...` remains a first-parent main landing with
  matching tree, landing-edge added record, trust-content digest
  `sha256:6d08be44a7ecaaed8ab9a7b93e8caaee5279c70aa5848a9dabe80e510b085331`,
  and `is_trust_kernel`-derived touched paths.

Fresh production validation at `72ae024`:

```text
verify_landed_review_marker(lane1) = []
verify_landed_review_marker(lane2b) = []
verify_repository_landed_markers(HEAD, origin/main) = []
```

## Security

security: NOT APPLICABLE — no capability or authority implementation surface
is present in this diff. The content cure changes only `docs/why.html`; the
two commits above it change only `ops/lane1/BLOCKED.md`,
`ops/lane1/journal.md`, Request 04, and its U-33 addendum.

This is a scoping decision, not a waiver. Deep code review was not triggered:
no security, concurrency, unsafe, lifetime, dependency, process/FFI,
path-authority, seal-implementation, or MCP-host surface changed. No Codex
Security workspace was run.

## Findings and advisories

F1 (NOTE): the nonexistent `--evidence` flag is now uniformly labeled
planned/non-shipping. The real binary still rejects it with exit 2.

F2 (NOTE): build/check/seal attribution is now accurate. The underlying
product gap is faithfully registered as U-32 with its own-lane, full-ceremony,
pre-Lane-4, no-restored-claim constraints.

F3 (NOTE): the wider pre-existing page claims are faithfully registered as
launch-blocking U-33 in Lane 3, requiring one coherent pre-launch/pre-CRA pass.
They are not re-raised as Lane-1 blockers under the ceremony ruling.

F4 (NOTE): the WV-6 candidate-only red is the expected digest movement and
remains fail-closed pending the native-Windows run at this approved head.

S-SEC-1 (ADVISORY, non-blocking): carry forward the broad security sweep
across capability/authority surfaces before the Lane 4 frozen candidate and
final red-team, including any such surface touched by Lane 0 repair #3.

S-WS-1 (ADVISORY, non-blocking): the cumulative historical packet still has
trailing whitespace in immutable Requests 02 and 03. The cure commit itself is
`git diff --check` clean. Do not create a standalone formatting round.

## Weakening, provenance, and not verified

- No implementation assertion, trap, gate, reporter, workflow, lockfile,
  sealed input, dependency, or capability surface was removed or loosened.
- Native-Windows WV-6 was not run on this macOS reviewer machine.
- U-31 remains a prerequisite for deterministic slice-5 generated outputs.
- U-32 remains required before Lane 4 and before any build-time enforcement
  claim is restored.
- U-33 remains required in Lane 3 before launch and the CRA positioning push.
- No broad security scan was run under the explicit scope ruling.
- No PR, merge, GitHub approval event, workflow, ruleset, implementation file,
  or `ops/mission/state.json` was created or modified by the reviewer.

Final ruling: APPROVE exact head
`72ae0246fb448ce33d689b1b80eb783497a7f215`. The NUC may run the native
Windows WV-6 acceptance only at that exact head.
