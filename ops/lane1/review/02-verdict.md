# Lane 1 Phase 0 — Independent Review Verdict 02

reviewer: Codex GPT-5.6 Sol (independent, non-implementer)
reviewed_head: 4f5ebb83d5772598e37a658ed4dce78208ea86fa
reviewed_tree: 46b289097fa281036e8b00e5e10a3e9a6b87a175
request: ops/lane1/review/02-request.md (request commit c6298ab05f9d3d6045bb7a8b434b9a5e9fcac7ad); ops/lane1/review/03-request.md (request commit 533f8a925ffa8f549caa351e630f93ec17878ab1)
swept_at: 2026-07-27T20:47:26Z
machine: Hughs-MacBook-Pro.local; macOS 26.5; Darwin 25.5.0 arm64
model: Codex GPT-5.6 Sol
implementer_self_report: Claude Code Fable 5 for both cure rounds (request 02 records earlier round-1 slices as Opus 4.8)
verdict: APPROVE-WITH-BLOCKERS
verified_identity: head/tree/diffstat/digests reproduced: yes
lineage: merge commits from base: 0 | canonical review-record commits in reviewed range: 0

This single verdict disposes of both Request 02 and Request 03. The exact
candidate, its linear lineage, the two-line ceremony-authorized extension, the
pinned Gate-1 claims, and its product digest all reproduce. The cure is not
approved for the Windows acceptance run: F1 remains incompletely cured, and
the whole-page adversarial read plus executable CLI probes find remaining
shipping/enforcement/attestation claims that outrun the repository. No exact
head is approved; the NUC must not run WV-6 acceptance at `4f5ebb83...`.

## Boot and sweep

- Fresh clone with `core.autocrlf=false`; remote `origin` is
  `Island-Dev-Crew/garnet`; no `refs/pull/*` were fetched.
- `origin/main` at boot:
  `68317ae258327aade47fc2c07b7b5b580ec7c6ea`, tree
  `29191aa0e17121c08b73fe12578ee4464559e2ba`.
- Main truth floor: lane-0 closeout PASS (22/22), MSRV PASS (1.95),
  frozen backlog PASS, authenticated rolling-review v2 diagnostic PASS.
- Fork tip at review freeze:
  `533f8a925ffa8f549caa351e630f93ec17878ab1`. That commit adds Request 03
  above the exact content candidate. Repeated `ls-remote` reproduced the tip
  before the verdict was written.

## Identity, cumulative scope, lineage, and record law

The cumulative delta requested for review, `d7430c2..4f5ebb8`, is:

```text
6 files changed, 738 insertions(+), 25 deletions(-)
M docs/why.html
A ops/lane1/BLOCKED.md
M ops/lane1/journal.md
A ops/lane1/review/01-request.md
A ops/lane1/review/01-verdict.md
A ops/lane1/review/02-request.md
```

- Candidate head and tree reproduce exactly. Merge-base with
  `68317ae258327aade47fc2c07b7b5b580ec7c6ea` is that exact base; the range
  contains zero merge commits.
- No Rust, Python, workflow, dependency, lockfile, sealed input, gate,
  reporter, capability implementation, or canonical `*.review.json` record
  changed.
- Requests 01, 02, and 03 and Verdict 01 were each introduced by one commit
  and were never modified. Request 03 is above the content candidate and is
  covered by this verdict. Nothing was pushed after its record commit before
  this verdict.
- The cumulative request packet is not whitespace-clean:
  `git diff --check 68317ae..4f5ebb8` reports trailing whitespace in
  `ops/lane1/review/02-request.md` at lines 221, 225, and 235. This is the
  non-blocking S-WS-1 advisory below, not implementation ride-along.

## Request 03 scope-extension ruling

The ceremony authorization in Request 03 expressly permits the reason #2 and
reason #10 edits. Independent inspection of implementation commit
`4f5ebb83...` against parent `afc00c6...` gives:

```text
docs/why.html | 4 ++--
1 file changed, 2 insertions(+), 2 deletions(-)
```

The two changed HTML lines are exactly reason #2 and reason #10. No other path
or line rode along. Reason #2 now binds its compiler statement to the bounded,
pinned Gate-1 surface; reason #10 now carries the same bound inside its own
reading unit. The authorized extension did not exceed its authorization.

## F1/F2 disposition and real CLI surface

The source dispatcher, not the page, was treated as authority:

```text
garnet-cli/src/bin/garnet.rs:52   "check" =>
garnet-cli/src/bin/garnet.rs:90   "caps" =>
garnet-cli/src/bin/garnet.rs:280  "diff-caps" =>
garnet-cli/src/bin/garnet.rs:303  "seal" =>
```

The built binary routes all four commands. Thus the page's references to
`caps`, `diff-caps`, `seal`, and `check` name real CLI surfaces.
`build --evidence` is not one:

```text
$ target/debug/garnet build --evidence \
    garnet-cli/tests/fixtures/malformed/caps_violation.garnet
unknown build flag: --evidence
rc=2
```

F1 is not cured. Three full-command references are expressly planned, but a
fourth raw occurrence remains at `docs/why.html:544`, where the page calls
Garnet “the `--evidence` layer” without a planned/roadmap qualifier. The
mandated condition that every `--evidence` mention be planned/roadmap is
therefore false.

The named F2 lines from Verdict 01 were materially narrowed, and the two
Request-03 lines were cured within authorization. The hostile whole-page pass
nevertheless finds additional self-contained present-tense guarantees. Direct
behavior also contradicts the page's remaining build/seal claims:

```text
$ target/debug/garnet check \
    garnet-cli/tests/fixtures/malformed/caps_violation.garnet
caps coverage: function `main` does not declare `fs` but transitively calls
`read_file` which requires it
rc=1

$ target/debug/garnet build \
    garnet-cli/tests/fixtures/malformed/caps_violation.garnet
built ... (1 items)
rc=0

$ target/debug/garnet seal \
    garnet-cli/tests/fixtures/malformed/caps_violation.garnet
... predicate emitted UNSIGNED ...
"capability_manifest":{"aggregate":[],"functions":[{"name":"main","caps":[]}]}
rc=0
```

The checker rejects the program, while build and seal accept it and the seal
records the empty declarations. Therefore “capability declarations checked at
build” and “the seal attests what the core proves” are not bounded descriptions
of the executable behavior.

## Adversarial whole-page prose read

The following remaining reading units make enforcement, proof, guarantee, or
shipping claims without bounding themselves to the two pinned Gate-1 claims,
or are directly contradicted by the probe above:

- lines 236-237, 275, and 278: “Enforcement by construction,” “A type cannot,”
  “undeclared = inexpressible,” and “the property is not [porous]”;
- lines 289 and 297-301: authority is “provable” to the accepting human/host;
  undeclared authority is structurally impossible/inexpressible; the authority
  delta is a typed, diffable, sealed property;
- line 317: whole classes of authority failure do not compile;
- lines 399 and 449: “no evidence without a build that proves it” and
  capability declarations are checked at build;
- lines 488, 490, and 493-494: a type makes overstep structurally impossible,
  the seal attests what the core proves, the safe thing is the only expressible
  thing, and an MCP host verifies the typed envelope before granting it;
- lines 496-497: the capability lattice can make delegation provably narrow,
  and the compiler holds the envelope for every junior and agent thereafter;
- lines 508-517: present-tense domain cards claim signing-key/network
  non-access, exfiltration compile errors, deployable per-invocation policy,
  sealed compiler-derived manifests, delegation attenuation, installed CLI
  manifests exposed by `brew info`, firmware authority verification, and
  exact-analysis sealed-run verification;
- line 544: the raw `--evidence` layer and swarm-delegation primitive.

The scope note at line 479 and later research-grade fences do not rewrite
these independent hero, reason-card, and domain-card reading units. More
decisively, the build/seal probe falsifies lines 449 and 490 with the
repository's own checker-invalid fixture. The findings therefore survive an
attempt to read the later boundary as a global qualification.

## Pinned integrity

```text
$ python3 -I scripts/garnet_capability_scope_status.py --gate
ok=true
enforced_claim_count=2
enforced_claim_expected=2
enforced_claim_hashes_match=true
current_truth_missing=[]
cited_anchors_missing=[]
forbidden_hits=[]
stale_truth_hits=[]
```

- The two complete `<b>enforced:</b>` lines extracted from the candidate are
  byte-identical to `d7430c2`; count is exactly two.
- `test_entry_authority` and `scope_shadowing_parity` each occur once.
- Both canonical snippets remain verbatim.
- The focused reporter suite ran 10 tests and passed.
- `66.7%`, `50.0%`, `83.3%`, `62.5%`, and `37.5%` have zero hits anywhere on
  the page.

Mechanical pin integrity passes. It does not validate the unpinned prose
listed above.

## Differential

The full Python battery ran serially at candidate and base from the same
disposable isolated environment with exact `PyYAML==6.0.3`:

```text
PATH=<review-venv>/bin:$PATH <review-venv>/bin/python -I \
  -m unittest discover -s scripts -p 'test_*.py'

base 68317ae:      Ran 1123 tests; 4 failures; 0 errors; exit 1
candidate 4f5ebb8: Ran 1123 tests; 5 failures; 0 errors; exit 1
```

The same four inherited failures occur at both heads:
`test_repo_and_site_point_to_the_adoption_surface_reporter`,
`test_tracked_ledger_matches_renderer_byte_for_byte`,
`test_all_novel_programs_check_and_run`, and
`test_tag_release_publishes_unified_checksummed_assets`.

The only new-vs-base failure is the expected WV-6 freeze binding:

```text
test_current_repository_tracks_wv6_acceptance_and_wv7_pending
AssertionError: 'partial' != 'accepted'
```

The Cargo differential is exact parity:

```text
rustup run 1.95.0 cargo test --workspace --no-fail-fast
base:      182 suites; 2199 passed; 0 failed; 6 ignored; exit 0
candidate: 182 suites; 2199 passed; 0 failed; 6 ignored; exit 0
```

## Freeze facts

Independent construction from the candidate Git tree reproduced:

```text
product_content_sha256 =
9f2bbe761b0cd6762190d2e14d795f4a02e203cc3ede53fc6c4326af5cc6c925
product path count = 1576
```

The repository module independently returns the same values. WV-6 remains
5/5 checks with all five evidence artifacts; its only finding is:

```text
product content digest mismatch
(9f2bbe761b0cd6762190d2e14d795f4a02e203cc3ede53fc6c4326af5cc6c925
 != 2f8c9ad860bd9c6dbd1e005b0c82af0288dd3a736bb416a3978708c12e6fa1fd)
```

No Shelf/WV script, contract, proof, or artifact byte changed from `d7430c2`
to this candidate. The partial state and sole new Python failure are caused
only by moving off the certified product digest.

## Security

security: NOT APPLICABLE — no capability or authority implementation surface
in the cumulative six-path diff:

```text
docs/why.html
ops/lane1/BLOCKED.md
ops/lane1/journal.md
ops/lane1/review/01-request.md
ops/lane1/review/01-verdict.md
ops/lane1/review/02-request.md
```

This is a scoping decision, not a waiver. Deep code review was not triggered:
there is no security/concurrency/unsafe/lifetime surface and the content cure
itself is far under the size threshold. Source-dispatch inspection and CLI
execution above are claim verification, not a broad security scan. No Codex
Security workspace was retried.

## Findings

### F1 (BLOCKER) — the `--evidence` cure is incomplete

Reproduction:

```text
$ rg -n -- '--evidence' docs/why.html
391: ... planned ... not in the CLI today ...
398: The planned ... no such build flag exists ...
495: The planned ... flag ...
544: ... the --evidence layer, the swarm-delegation primitive ...
```

Line 544 is still present-tense product identity, not planned/roadmap text.
The real dispatcher rejects the flag with exit 2. This fails the explicit
Request-02 cure condition.

### F2 (BLOCKER) — remaining page-wide guarantees outrun executable evidence

The named Verdict-01 lines and Request-03 reason lines were narrowed, but the
whole-page reading units enumerated above still promise unbounded
construction, build enforcement, seal attestation, host verification,
delegation attenuation, and shipping domain integrations.

Exact executable reproduction:

```text
target/debug/garnet check garnet-cli/tests/fixtures/malformed/caps_violation.garnet
# rc=1, undeclared transitive fs
target/debug/garnet build garnet-cli/tests/fixtures/malformed/caps_violation.garnet
# rc=0
target/debug/garnet seal garnet-cli/tests/fixtures/malformed/caps_violation.garnet
# rc=0, aggregate=[], main caps=[]
```

At minimum lines 449 and 490 are directly false for this repository-owned
fixture. The broader lines are not explicitly bounded to the two pinned
Gate-1 claims and have no committed traps or shipping integration evidence.
Marketing that outruns evidence is blocking under the review instructions.

### F3 (NOTE) — Request 03 stayed exactly inside ceremony authorization

`git show --stat --oneline 4f5ebb83` reports one file and two replaced lines.
The patch contains only reason #2 and reason #10. Both cures are materially
bounded and no path or assertion rode along.

### F4 (NOTE) — expected WV-6 digest movement remains the sole new differential red

`garnet_wv_acceptance_status.py --wv WV-6` returns `partial`, 5/5 checks,
five artifacts, and only the product-digest mismatch. This is a real
head-vs-base failure and must remain fail-closed until Windows evidence is
rebound to a subsequently approved exact head.

### F5 (NOTE; blocking prerequisite for slice 5) — U-31 remains reproducible

Two independent checkouts of the exact candidate emitted different
launch-readiness JSON bytes:

```text
checkout A sha256 c81cc113ee4cc64571f6bdb8f146811f45709cf94228af16f0367bff35082f39
checkout B sha256 dc7ba4a6074b02a8dadf3e33bfe376333415324f1c05afc3b337c72c8462c737
A source=/private/tmp/.../candidate/scripts/garnet_launch_readiness_status.py
B source=/private/tmp/.../candidate2/scripts/garnet_launch_readiness_status.py
equal_without_source=true
```

The absolute checkout path in `source` remains the sole byte difference. It
does not change this docs-only cure ruling, but it still blocks slice 5 from
committing regenerated readiness/SOTU outputs until normalized.

## Style and standing advisories

S-SEC-1 (ADVISORY, non-blocking): carry forward the broad security sweep
across capability/authority surfaces before the Lane 4 frozen candidate and
final red-team. It must also cover any such surface touched by Lane 0 repair
#3.

S-WS-1 (ADVISORY, non-blocking): Request 02 contains three blank lines with
trailing whitespace, so its packet-wide `git diff --check: clean` claim is not
true for the cumulative range. Do not create a standalone formatting round;
clean it only if a later authorized record-writing step touches that file.

## Weakening, provenance, and not verified

- No implementation assertion, trap, gate, reporter, workflow, lockfile,
  sealed input, dependency, or capability surface was removed or loosened.
- The F1/F2 prose cure changed only `docs/why.html`; Request 03 stayed inside
  its exact two-line authorization.
- Native-Windows WV-6 evidence was not run on this macOS reviewer machine.
- No broad security scan was run under the explicit scope ruling.
- External market-statistic and legal sources were not re-researched; this
  review concerns Garnet's shipping/enforcement/proof claims.
- No PR, merge, GitHub approval event, workflow, ruleset, implementation file,
  or `ops/mission/state.json` was created or modified by the reviewer.

Final ruling: Requests 02 and 03 are both disposed by this verdict. Identity,
lineage, pinned integrity, scope-extension discipline, digest, and
differential freeze behavior are sound. The exact candidate
`4f5ebb83d5772598e37a658ed4dce78208ea86fa` is not approved for the NUC run
because F1 and F2 remain blocking.
