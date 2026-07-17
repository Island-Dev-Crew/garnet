# Lane 1 Item 2 — rolling-review v2 failure design and TDD evidence

Date: 2026-07-17
Scope: Lane 1 Item 2 / U-04 and U-19 only
Merge authority: Jon only

This design was converted into adversarial tests before the v2 production
implementation. The control has two honest states: a content-bound premerge
review record and a separate squash-durable landed marker. A premerge record
never invents a merge commit; a landed marker never invents or rewrites the
premerge reviewer/content claim.

## Failure semantics fixed before implementation

| Boundary | Required RED |
|---|---|
| Ref discovery | missing `origin/main`; head/base that is not a commit; command failure or timeout; empty, malformed, or partial merge-base output |
| Diff enumeration | command failure or timeout; missing NUL terminator; incomplete raw header/path pair; invalid UTF-8 path; duplicate or unsafe path; unsupported status |
| Independent object enumeration | raw/name presentations disagree with commit/tree-object traversal; malformed/unreadable commit or tree; graph/tree bound exceeded |
| Add/change/delete identity | status inconsistent with old/new mode or object ID. Exact deletion is allowed only as an old-blob tombstone with zero new mode/object; ambiguous deletion is RED |
| Worktree state | malformed/failed porcelain-v2 enumeration or any staged, unstaged, or untracked path in gate mode |
| Enumeration override | `--changed-file`, explicit `--base`, or non-`HEAD` `--head` cannot satisfy `--gate`; a legitimate authoritative empty Git diff remains GREEN |
| Companion discovery | no `*.review.json`; deleted record; multiple records; Markdown/trailer only; non-JSON or noncanonical JSON |
| Record history/type | modification, deletion, restoration, or type change of a record on any raw commit/tree edge; new record not a regular non-executable `100644` blob; endpoint restoration never erases an intermediate RED |
| JSON shape | duplicate/unknown/missing keys, invalid schema/state, malformed OIDs/digest/identity, unsorted/duplicate/empty path or commit-principal list |
| Exact scope | record path set has any missing or extra trust path; discovered base differs; content digest differs |
| Independence | reviewer is malformed or overlaps a derived email author or an authenticated GitHub author/committer principal; verdict is not `pass`; blocking findings are nonempty |
| Authenticated review | missing explicit transport; unreachable/403/malformed response; partial pagination; a full terminal page without a completeness signal; duplicate review ID; non-integer/boolean/nonpositive PR or repository IDs; upstream-base/fork-head repository or PR/reviewer mismatch; latest decisive same-reviewer state is not `APPROVED`; approval not bound to the exact current candidate head |
| Authenticated commits | partial/duplicate/malformed PR commit collection; mismatch with local raw commit graph; missing or malformed GitHub author/committer role identity; mismatch between the legacy `author_ids` field and the exact immutable author/committer ID union |
| Author enumeration | `git rev-list base..reviewed_head` fails, times out, is empty/truncated/malformed, or any commit author lookup is missing/partial/malformed; self-declared authors differ from the derived canonical email set |
| Review provenance | reviewed head is missing/non-commit, does not follow base, is not an ancestor of the current premerge head, or its tree differs |
| Post-review mutation | any reviewed-lineage commit after `reviewed_head` touches trust content, including edit/revert and merge-only resolutions |
| Landed first-parent proof | missing/non-commit merged commit, missing authoritative main, commit absent from first-parent, merge ordered before/at base, malformed first-parent edge, or replacement-ref substitution |
| Landed content proof | merged tree mismatch; exact first-parent landing-edge trust path/digest mismatch; canonical premerge record absent from that edge, missing at merge, noncanonical, or raw-byte SHA-256 mismatch |
| Landed claim binding | any author, reviewer, verdict, finding, path, digest, reviewed-head/tree, scope, or base field differs from the exact committed premerge record |
| Production marker registry | missing/malformed registry; path alias; listed/discovered marker mismatch; non-`100644` registry/marker; any registered marker fails landed verification; any commit transition deletes/replaces a marker or removes a registry entry |
| Git subprocess isolation | checkout persists a credential; shell exports a token into the reporter; any child Git process inherits credentials, unrelated ambient state, or Git repository/index/worktree/config/object/replace/namespace controls; a dirty real checkout appears clean through ambient redirection |

## Deterministic trust-change digest

Each trust entry is sorted by path and hashes status, path, old/new modes,
old/new Git object IDs, and independently recomputed SHA-256 identities of both
blob byte strings. Additions use an absent-old sentinel; deletions retain the
old blob SHA-256 and use an absent-new sentinel. This makes a deletion
reviewable without converting a valid deletion into a permanent gate failure.

## Two-state squash contract

### State A — premerge

Schema `garnet.trust_kernel_review_record/v2`, state `premerge`:

- binds the authoritative merge-base, exact trust paths/digest, raw-commit
  author emails, the exact authenticated GitHub author/committer identity union
  (retained under the schema-compatible `author_ids` key), immutable
  upstream-base and fork-head repository IDs, PR identity, designated approved
  reviewer, pass verdict, zero blocking findings, and trust-content reviewed
  head/tree;
- intentionally does not embed a future review ID or its own commit SHA. The
  live gate selects the authenticated approval from the named reviewer at the
  exact current candidate head, avoiding an unsatisfiable self-reference under
  stale-approval dismissal;
- proves every authenticated PR commit is present and every post-review commit
  is free of trust touches, not merely that endpoint content is identical;
- permits the record/companion commit after the trust-content `reviewed_head`,
  catches every later trust-path edit, and requires the final authenticated
  approval to cover the exact record-containing candidate head;
- has no `merged_commit` field.

### State B — landed

Schema `garnet.trust_kernel_review_marker/v2`, state `landed`:

- binds the exact premerge record path and raw-byte SHA-256 at the landed commit;
- requires every premerge claim field to equal that committed record;
- excludes `review_id` and `approval_head`, because postmerge verification has
  no authenticated review transport and cannot honestly prove those claims;
- proves `merged_commit` is on upstream main's first-parent history after its
  base and its replacement-ref-resistant tree equals `merged_tree`;
- recomputes exact trust paths/digest on the immediate
  first-parent-to-`merged_commit` landing edge and requires that same edge to
  carry the canonical premerge record; the base stays bound earlier on main,
  while unrelated main advances before a legitimate squash remain valid;
- deliberately does not require pre-squash reviewed-head ancestry and does not
  backdate review coverage;
- makes the registry and marker directory trust-kernel paths, requires regular
  `100644` blobs, and inspects every candidate commit transition so registered
  history can only be retained or appended, never deleted, replaced, or reset.

## Recorded RED

Command run before production implementation:

```text
python3 -I scripts/test_garnet_trust_kernel_review_status.py -v
```

Initial v2 RED observed: exit `1`; 45 tests ran, with 2 explicit old-v1
false-greens and 40 missing-v2 errors. The false-greens proved both forbidden
bypasses:

- `--changed-file README.md --gate` returned zero despite not proving complete
  enumeration;
- `--assume-trailer` returned zero for a trust-path change with no content-bound
  record.

The independent audit then required authenticated GitHub object enumeration,
raw commit/tree traversal, edit/revert and merge traps, clean-worktree parsing,
append-only records, and production landed-marker registry wiring. The expanded
RED was recorded before those mechanisms: exit `1`; 83 tests; 9 failures and 49
errors.

A final integration audit found two additional unsatisfiable assumptions and
recorded both as RED before repair: the approval pointed at the content head
before the record commit even though Garnet stale-dismisses approvals on push,
and head/base repository identity assumed a same-repository PR despite the
required `Navigata1/garnet` fork -> `Island-Dev-Crew/garnet` path. CI also
defaulted to the synthetic merge ref. Dedicated tests now require separate
immutable fork/base identities, exact-current-head approval, and exact fork-head
checkout for the review job.

The first independent final review then rejected five concrete false-greens,
all recorded before repair: a 100-row terminal API page with its `Link` header
removed; an older approval surviving a later same-reviewer adverse decision;
erasable landed history plus unauthenticated landed approval fields; checkout,
shell, and Git-child credential inheritance; and backslash aliases accepted as
exact Git paths. Each now has a dedicated regression trap. The production
mechanism fails closed on ambiguous full pages, selects the newest decisive
same-reviewer event by immutable reviewer ID even after a login rename, treats landed history as commit-by-commit append-only,
removes the unprovable landed fields, isolates credentials before Python and
Git, and requires canonical slash-delimited path bytes.

The integrated freeze review found that two direct fail-closed dependencies,
`garnet-cli/src/cmd/add.rs` and `garnet-cli/src/bound_source.rs`, were changed in
this lane but absent from the trigger surface. A classification regression was
added first and failed on `cmd/add.rs`; both exact files are now machine
trust-kernel paths. Future parsing or retained-handle changes cannot bypass the
v2 record merely because the public `run`/`test` call sites stay unchanged.

The same integrated review found that `.github/CODEOWNERS` is the enforcement
artifact for U-16 but was absent from the trigger surface. A regression was
added first and failed with `AssertionError: False is not true :
.github/CODEOWNERS`; the exact file is now machine-classified. Future removal
of the `scripts/garnet_github_*` Jon-only ownership rule therefore requires the
same structured rolling review as the governed scripts.

The integrated dependency audit also found four direct Item 6 dependencies
outside the exact trigger set: `garnet-cli/src/lib.rs` exports the retained
source module, `garnet-cli/src/cmd/mod.rs` owns dispatch to the reviewed
`run`/`test`/`add` modules, `garnet-cli/Cargo.toml` declares the pinned parsing
and file-identity dependencies, and `Cargo.lock` fixes their resolved graph. The
classification regression failed first on `garnet-cli/src/lib.rs`; all three
initial dependencies and then the dispatch module failed before repair. All
four are now exact trust-kernel files, without broadening the surface to
unrelated CLI or workspace content.

A later committer-independence review found that authenticated commit rows
validated only GitHub's `author` role even though the API also exposes a
distinct `committer` principal. That allowed an approving reviewer who was the
authenticated committer, but not the author, to appear independent. Three
regressions were recorded before repair: reviewer/committer overlap falsely
passed, a missing committer object falsely passed, and an exact two-role
identity union was rejected. The gate now validates both immutable role
identities, fails closed on either malformed role, binds their exact sorted
union in `author_ids`, and rejects a reviewer present in that union.

The same review exposed Python's boolean/integer equality alias at
authenticated object boundaries: API `id: true` could compare equal to a
recorded integer `1`, and an integral float could compare equal to a PR number.
Four isolated regressions for PR number, PR ID, fork-head repository ID, and
upstream-base repository ID each falsely passed before repair. Every API value
must now have exact `int` type and be positive before equality is considered.

## Fresh focused GREEN

```text
python3 -I scripts/test_garnet_trust_kernel_review_status.py -v
```

Fresh grouped execution after implementation:

- premerge/deletion: 33/33 passed in 32.686 seconds;
- landed markers/registry: 12/12 passed in 9.787 seconds;
- discovery/object/worktree/append-only/CLI: 38/38 passed in 11.342 seconds.

Final integrated execution before the committer-independence repair: 92/92
focused tests passed in 85.552 seconds. The bounded authenticated transport also
passed 23/23 tests, and the strict Link-header parser passed 12/12 tests. The
added integration traps cover exact-current approval, later adverse review,
fork/base identity, canonical paths, append-only landed history, credential
isolation, and authenticated CI wiring without removing any earlier adversarial
case.

Combined execution after the committer, exact API identity-type, and initial
dependency-classification repairs, but before the U-19 exact-boundary repair:
99/99 focused tests passed in 97.074 seconds.

A final U-19 review showed that cumulative `base_commit..merged_commit` content
could let a later unrelated first-parent commit impersonate the original
squash. The regression advanced main after a valid squash, substituted that
later commit/tree in the marker, and observed a false GREEN. Landed verification
now derives the raw first parent of `merged_commit`, cross-checks it against
main's first-parent history, and requires the canonical review record plus the
exact reviewed trust paths/digest on that single landing edge. It does not
require that parent to equal the older reviewed base.

Combined execution before Git subprocess-environment hardening: 100/100 focused
tests passed in 98.381 seconds.

The final integrated audit found that `_git_bytes()` removed credential-shaped
names but inherited every other ambient variable. A dirty real checkout with
`GIT_DIR`, `GIT_WORK_TREE`, and `GIT_INDEX_FILE` redirected to a clean alternate
repository therefore returned a false GREEN. A second regression showed that
Git config, object-directory, alternate-object, graft, shallow, replace-base,
quarantine, common-directory, and namespace controls reached the child. Every
Git probe now uses one minimal allowlisted environment, controlled null global
and system configs, zero injected config entries, disabled replacement objects,
and no inherited repository control plane.

Combined execution before commit-by-commit review-record history enforcement:
102/102 focused tests passed in 105.142 seconds.

The next integrated audit found that premerge `*.review.json` append-only
checking inspected only the endpoint diff. Modifying or deleting an existing
record in one PR commit and restoring its original bytes at `HEAD` therefore
returned `ok: true`. Both false-greens were recorded before repair; a single new
regular `100644` record addition remained GREEN. The gate now traverses the raw
candidate commit graph and raw tree snapshots for every parent edge, permits
only first-time `100644` additions, and retains any intermediate record byte,
mode, or presence change as a permanent RED for that candidate history.

Combined execution after review-record history enforcement: 105/105 focused
tests passed in 125.606 seconds.

The final freeze audit then recorded four more concrete REDs before repair:

- the same record path could be introduced independently on two parallel
  branches because each parent edge appeared to be a first addition;
- a malformed record added in a record-only commit escaped parsing because it
  was outside the endpoint trust-path set;
- a landed marker could bind a pre-existing or modified record rather than a
  newly added regular `100644` record on the exact squash landing edge; and
- a user-level Git graft could alter commit enumeration because a graft file
  was not neutralized by the minimal subprocess environment.

The gate now requires exactly one raw-history introduction for every review
record path, parses and authenticates the canonical added record even when the
commit contains no other trust change, requires status `A` plus mode `100644`
for the record on the exact landed first-parent edge, and forces
`GIT_GRAFT_FILE` to the platform null device. Observed graft diagnostics are
also RED; the expected Git deprecation warning is suppressed at the command
boundary rather than ignored after enumeration.

Final exact-head execution after those four repairs: 110/110 focused tests
passed in 129.934 seconds. The final bounded authenticated transport suite also
passed 24/24. `python3 -m py_compile` and `git diff --check` passed for the
reporter, tests, and evidence changes.

The earlier independent adversarial approval covered that 92-test predecessor
and does not extend to the later repairs. A fresh exact-head re-review is
required against trust-content head `b9f8bc91dd0660f0988e711fea31a535c0aae8f5`.
No scoped agent approval substitutes for the authenticated GitHub approval
required by the machine gate or for cross-OS evidence. The branch must still
add its canonical structured record and W_TRUST companion, obtain the recorded
reviewer approval at the exact final PR head, and pass the ordinary
Linux/macOS/Windows required checks. Until then, the branch-level gate remains
correctly RED.
