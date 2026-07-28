# Lane 1 Phase 0 — Independent Review Verdict 01

reviewer: Codex GPT-5.6 Sol (independent, non-implementer)
reviewed_head: d7430c285fa8620dcf1f0c1cd94e5cc44b98d180
reviewed_tree: 2e8ce5dedfe88d67e8cc4bfa2527591ffcc5f3a8
request: ops/lane1/review/01-request.md (request commit 742d00e885eb0d8700fa171634425b54ffe8ab01)
swept_at: 2026-07-27T19:18:05Z
machine: Hughs-MacBook-Pro.local; macOS 26.5; Darwin 25.5.0 arm64
model: Codex GPT-5.6 Sol
implementer_self_report: Claude Code Opus 4.8
verdict: APPROVE-WITH-BLOCKERS
verified_identity: head/tree/diffstat/digests reproduced: yes
lineage: merge commits from base: 0 | record commits in reviewed range: 0

The candidate is identity-clean and the two landed-review markers reproduce.
The review does not approve the new public prose as written. Two bounded
documentation cures are required: remove or make truthful the nonexistent
`garnet build --evidence` shipping claim, and re-scope every new enforcement,
attenuation, or guarantee statement to evidence the repository actually
supports. No implementation cure is authorized by this verdict.

## Boot and sweep

- Fresh clone with `core.autocrlf=false`; remote `origin` is
  `Island-Dev-Crew/garnet`; no `refs/pull/*` were fetched.
- `origin/main` at boot and close:
  `68317ae258327aade47fc2c07b7b5b580ec7c6ea`, tree
  `29191aa0e17121c08b73fe12578ee4464559e2ba`.
- Main truth floor: lane-0 closeout PASS (22/22), MSRV PASS (1.95),
  frozen backlog PASS, authenticated rolling-review v2 diagnostic PASS.
- Fork sweep: 13 `mission/*` branches. The only unanswered request was this
  packet on `mission/l1-reconcile-post-activation`.
- Fork tip at sweep:
  `6d160a19fc8c6f357a7aa000a75d2611b3ebfe34`. The commits above the
  candidate are review artifacts only: `742d00e` adds this request;
  `6d160a1` adds `ops/lane1/BLOCKED.md` and one journal line.

## Identity, lineage, scope, and record law

Against base `68317ae258327aade47fc2c07b7b5b580ec7c6ea`:

```text
5 files changed, 449 insertions(+), 11 deletions(-)
M F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json
A F_Project_Management/W_TRUST/landed/LANE1_GOVERNANCE_ACTIVATION.landed-review.json
A F_Project_Management/W_TRUST/landed/LANE2B_MINIMUM_SHELF_MCP.landed-review.json
M docs/why.html
A ops/lane1/evidence/90-reconcile-baseline-red.md
```

- Merge-base is exactly the stated base. The three-commit reviewed range is
  linear (`7db0f45`, `a55a4e0`, `d7430c2`) with zero merge commits.
- No canonical `*.review.json` record was added or modified in the reviewed
  range. The eventual review record therefore remains subject to the
  add-once/final-commit rule.
- No path exists outside the declared five-path scope. `git diff --check`
  passes. The new lane evidence resolves `text: unset` under
  `git check-attr`.
- No Rust, Python, workflow, dependency, `Cargo.lock`, sealed input, gate, or
  reporter logic changed.

## Marker provenance — independently re-derived

The registry is a sorted, unique path list and exactly enumerates the two
canonical `F_Project_Management/W_TRUST/landed/*.landed-review.json` blobs in
the candidate tree.

For `LANE1_GOVERNANCE_ACTIVATION`:

- landing `68317ae...` is on `origin/main` first-parent immediately after
  recorded base `41d6ced...`; Git tree `29191aa0...` matches;
- landed trust-content digest independently recomputes to
  `sha256:b697d2fec3a610f212616f3a790660bcd16061e4b42fa5f53e5417ae82035a6a`;
- record blob SHA-256 independently recomputes to
  `f62d19dd9e2b0dab4f907c79ae4da3c1ff6a958a0e72b8ccf796aa4289323040`;
- the record is status `A` from mode `000000`/zero OID to `100644`;
- all shared claims equal the committed premerge record; reviewed head/tree
  objects reproduce; `touched_paths` exactly equals the landing-edge paths
  selected by the module's `is_trust_kernel` predicate.

For `LANE2B_MINIMUM_SHELF_MCP`:

- landing `41d6ced...` is on `origin/main` first-parent immediately after
  recorded base `cede73c...`; Git tree `e3c914b...` matches;
- landed trust-content digest independently recomputes to
  `sha256:6d08be44a7ecaaed8ab9a7b93e8caaee5279c70aa5848a9dabe80e510b085331`;
- record blob SHA-256 independently recomputes to
  `fbfe0310950c13f42609b9da3c22292a525294316d07344964da47a0404ec86f`;
- the record is status `A` from mode `000000`/zero OID to `100644`;
- all shared claims equal the committed premerge record; reviewed head/tree
  objects reproduce; `touched_paths` exactly equals the landing-edge paths
  selected by `is_trust_kernel`.

Direct production validation at the candidate:

```text
verify_landed_review_marker(lane1) = []
verify_landed_review_marker(lane2b) = []
verify_repository_landed_markers(root, ref="HEAD",
  main_ref="refs/remotes/origin/main") = []
```

Adversarial extra not present in the packet: replacing the Lane-1 marker's
`content_digest` in memory with all zeroes yields:

```text
['exact first-parent landing edge content digest mismatch',
 'landed marker claim does not match committed premerge record: content_digest']
```

Marker provenance is therefore verified, including fail-closed tamper
behavior.

## `docs/why.html` triple-bound result

Command:

```text
python3 -I scripts/garnet_capability_scope_status.py --gate
```

Result:

```text
ok=true
enforced_claim_count=2
enforced_claim_expected=2
enforced_claim_hashes_match=true
current_truth_missing=[]
cited_anchors_missing=[]
forbidden_hits=[]
stale_truth_hits=[]
```

Both pinned claim hashes, both test anchors, and both canonical snippets
remain present verbatim. No launch denominator is printed on the page:
`66.7%`, `50.0%`, `83.3%`, and `62.5%` all have zero hits. The other
percentages on the page are sourced market statistics, not launch
denominators.

The mechanical gate is green, but the adversarial semantic read is not. See
F1 and F2.

## Differential

The Python battery was run serially at candidate and base from the same
disposable isolated Python environment with exact `PyYAML==6.0.3`:

```text
PATH=<review-venv>/bin:$PATH <review-venv>/bin/python -I \
  -m unittest discover -s scripts -p 'test_*.py'

base 68317ae:      Ran 1123 tests; 4 failures; 0 errors; exit 1
candidate d7430c2: Ran 1123 tests; 5 failures; 0 errors; exit 1
```

The four base failures reproduce unchanged at the candidate:
`test_repo_and_site_point_to_the_adoption_surface_reporter`,
`test_tracked_ledger_matches_renderer_byte_for_byte`,
`test_all_novel_programs_check_and_run`, and
`test_tag_release_publishes_unified_checksummed_assets`.

The sole new-vs-base failure is:

```text
test_current_repository_tracks_wv6_acceptance_and_wv7_pending
AssertionError: 'partial' != 'accepted'
```

It is charged and recorded in F3. The full Cargo differential is exact parity:

```text
rustup run 1.95.0 cargo test --workspace --no-fail-fast
base:      2199 passed; 0 failed; 6 ignored; exit 0
candidate: 2199 passed; 0 failed; 6 ignored; exit 0
```

## Product digest, WV-6, and freeze

Independent invocation of the frozen product-digest construction over the two
committed trees reproduced:

```text
candidate c4b3cf7cea369a4003336b62b97a30a369be8063002cf4634c320bd6e027cb64 1572
base      2f8c9ad860bd9c6dbd1e005b0c82af0288dd3a736bb416a3978708c12e6fa1fd 1571
```

All three `W_TRUST` paths are excluded by the frozen construction. The product
delta is `docs/why.html` plus the tracked `ops/lane1/evidence` record. WV-6 is
accepted 5/5 with the same five artifacts at the base and partial 5/5 at the
candidate with one finding only:

```text
product content digest mismatch
(c4b3cf7c... != 2f8c9ad8...)
```

No Shelf/WV script, contract, proof, or artifact byte changed. The partial
state and the sole new Python failure are caused only by moving off the
certified product digest, not by a shelf-artifact change.

## Denominator falsification and deferral ruling

The sanctioned launch-readiness producer has eight gates. At both base and
candidate only `foundation_integrity`, `native_linux`, and
`s114_acceptance` are accepted. The committed lane-0 denominator producer
therefore correctly yields:

```text
launch_critical = 3/6 = 50.0%
launch_ledger   = 3/8 = 37.5%
```

Landed main already has WV-6 acceptance. Once the post-Windows slice refreshes
the product-bound WV-6 evidence and slice 5 regenerates the reporter/SOTU, the
supported reconciled values are `4/6 = 66.7%` and `4/8 = 50.0%`. The prompt's
`83.3% / 62.5%` would require one additional accepted launch gate and is not
supported by the current producer.

Ruling: deferring the reporter/SOTU/denominator refresh to slice 5 is correct.
Slice 5 must publish `66.7% / 50.0%` unless another gate is independently
closed before that refresh. It must first cure F4's machine-path
nondeterminism.

## Security

security: NOT APPLICABLE — no capability or authority surface in diff; scope
justified by the five-path list above; deep code review not triggered (no
security surface, diff far under the size threshold).

This is a scoping decision, not a waiver. No Codex Security workspace or broad
security scan was retried.

## Findings

### F1 (BLOCKER) — the page advertises a nonexistent shipping CLI flag

New lines 392, 399, 405, 450, and 453 state that
`garnet build --evidence` emits a versioned Verifiable Evidence Bundle with
the artifact and regenerates it in CI.

Reproduction:

```text
$ cargo run -q -p garnet-cli -- build --evidence \
    examples/mvp_04_numerical_solver.garnet
unknown build flag: --evidence
exit 2
```

Repository search finds no implementation of that build flag. Existing
Studio/dogfood evidence bundles are separate products and do not make this
CLI command or emitted build artifact real. The prose explicitly labels the
flag “shipping”; this is marketing that outruns executable evidence.

Required cure: remove the shipping/emission claim or replace it with bounded
truth about an actually runnable command and artifact. Do not implement a new
feature under this review verdict.

### F2 (BLOCKER) — new enforcement/attenuation guarantees evade the two-claim gate

The candidate adds visible “Claim class · enforced by the language today” at
lines 332 and 357 and “shipping flag” at line 405 while the page itself says
at line 548 that the two public claim upgrades — and only those two — are the
hash-bound claims at lines 550–551.

The reporter counts only source lines containing the literal marker
`<b>enforced:</b>`:

```text
scripts/garnet_capability_scope_status.py:153
if ENFORCED_CLAIM_MARKER in line:
```

Consequently the gate remains `ok=true`, count 2, despite the added visible
claim labels. The new prose also states, without a matching trap or completed
proof:

- lines 325, 343, 350: delegated authority exceeding a capability fails to
  build and the compiler holds the line for every contribution, agent, and
  commit;
- lines 368, 375–376: delegated authority always attenuates and a fleet
  cannot misbehave outside the envelope, while line 381 concedes the
  machine-checked proof pipeline is still in progress;
- line 424: no model can pass a capability it was never granted;
- lines 450 and 453: attenuation is structural and evidence is emitted and
  regenerates with the artifact.

The repository's own research describes metered delegation as future work,
and current S114 records retain declared-not-enforced and host/OS-sandbox
fences. The later boundary paragraph does not cure unconditional thesis text
that is explicitly labeled enforced today.

Required cure for this slice: restrict each sentence and claim-class label to
the exact checker/runtime surface supported by committed traps, and label
future attenuation/evidence work as future. The alternate-wording blind spot
also needs a separately reviewed gate-hardening follow-up; this verdict does
not authorize reporter-logic work inside the five-path reconciliation slice.

### F3 (NOTE) — the expected WV-6 digest movement is a real differential red

Reproduction:

```text
$ python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-6
state="partial"
passed_check_count=5
required_check_count=5
findings=["product content digest mismatch (c4b3cf7c... != 2f8c9ad8...)"]
```

The base suite is green for the WV-6 state fixture and the candidate suite is
red, so this is recorded under the differential law. It is the named,
expected freeze consequence, not an unexplained Shelf regression. Native
Windows refresh remains required before WV-6 can accept this exact product
head.

### F4 (NOTE; blocking prerequisite for slice 5) — U-31 candidate: absolute path in `08.source`

Two same-commit checkouts produced different reporter bytes:

```text
checkout A sha256 ba3f5ed9259a3370ca698c4acb9f5841d5ad53b7166303b5b4a411b24cd2bac8
checkout B sha256 6885bf5532e22939e7946022ccba67aef41be75d7f652ab58dba540e97fb8c4c

A source=/private/tmp/.../garnet/scripts/garnet_launch_readiness_status.py
B source=/private/tmp/.../candidate2/scripts/garnet_launch_readiness_status.py
```

After deleting only `.source` and canonicalizing JSON, `cmp` returns 0.
Therefore the absolute checkout path is the sole byte difference. This does
not invalidate the present candidate because it commits no regenerated
readiness output, but it blocks slice 5 from committing a deterministic
reporter/SOTU/denominator refresh until the source field is normalized.

## Style and standing advisories

S-SEC-1 (ADVISORY, non-blocking): a broad security sweep across Garnet's
capability/authority surfaces is owed before the Lane 4 frozen candidate and
final red-team. It must also cover any capability/authority surface touched by
Lane 0 repair #3. This is not a request for a standalone reformat or a
security-workspace retry on this five-path review.

No other S-style findings.

## Weakening and provenance ruling

- No assertion, trap, gate, reporter, lockfile, sealed input, or dependency
  was removed or loosened in implementation.
- The candidate does expose a semantic coverage gap in the public-claim gate:
  alternate “enforced today” labels are not counted. That gap is F2, not an
  approved weakening.
- Marker registry shape, first-parent lineage, landing-edge additions,
  record-byte hashes, content digests, reviewed head/tree provenance, and
  `is_trust_kernel` touched paths all verify.
- The RED-before-implementation record exists at `7db0f45` and truthfully
  records the empty marker registry before slice 1.

## Not verified

- Native-Windows WV-6 evidence was not rerun on this macOS reviewer machine.
- No broad security scan was run, by the explicit five-path scope ruling.
- External market-statistic sources in `docs/why.html` were not re-researched;
  this verdict addresses the new Garnet enforcement/proof/guarantee claims.
- No PR, merge, GitHub approval event, workflow, ruleset, implementation file,
  or `ops/mission/state.json` was created or modified by the reviewer.

Final ruling: marker reconciliation, identity, lineage, and the denominator
deferral are sound. The branch may proceed only after F1 and F2 receive a new
bounded cure and a superseding immutable review request/verdict cycle. F4 must
be cured before the slice-5 generated readiness artifacts are committed.
