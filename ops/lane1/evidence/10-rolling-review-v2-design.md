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
| Record history/type | modification, deletion, or type change of a historical record; new record not a regular non-executable `100644` blob |
| JSON shape | duplicate/unknown/missing keys, invalid schema/state, malformed OIDs/digest/identity, unsorted/duplicate/empty path or author list |
| Exact scope | record path set has any missing or extra trust path; discovered base differs; content digest differs |
| Independence | reviewer is malformed or overlaps a derived author; verdict is not `pass`; blocking findings are nonempty |
| Authenticated review | missing explicit transport; unreachable/403/malformed response; partial pagination; a full terminal page without a completeness signal; duplicate review ID; upstream-base/fork-head repository or PR/reviewer mismatch; latest decisive same-reviewer state is not `APPROVED`; approval not bound to the exact current candidate head |
| Authenticated commits | partial/duplicate/malformed PR commit collection; mismatch with local raw commit graph; missing/mismatched immutable GitHub author IDs |
| Author enumeration | `git rev-list base..reviewed_head` fails, times out, is empty/truncated/malformed, or any commit author lookup is missing/partial/malformed; self-declared authors differ from the derived canonical email set |
| Review provenance | reviewed head is missing/non-commit, does not follow base, is not an ancestor of the current premerge head, or its tree differs |
| Post-review mutation | any reviewed-lineage commit after `reviewed_head` touches trust content, including edit/revert and merge-only resolutions |
| Landed first-parent proof | missing/non-commit merged commit, missing authoritative main, commit absent from first-parent, merge ordered before/at base, malformed history, or replacement-ref substitution |
| Landed content proof | merged tree mismatch; landed trust path/digest mismatch; committed premerge record absent from landed range, missing at merge, noncanonical, or raw-byte SHA-256 mismatch |
| Landed claim binding | any author, reviewer, verdict, finding, path, digest, reviewed-head/tree, scope, or base field differs from the exact committed premerge record |
| Production marker registry | missing/malformed registry; path alias; listed/discovered marker mismatch; non-`100644` registry/marker; any registered marker fails landed verification; any commit transition deletes/replaces a marker or removes a registry entry |
| Credential isolation | checkout persists a credential; shell exports a token into the reporter; any child Git process inherits a token, secret, credential, password, cookie, authorization, or private-key environment name |

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
  author emails, authenticated GitHub author IDs, immutable upstream-base and
  fork-head repository IDs, PR identity, designated approved reviewer, pass
  verdict, zero blocking findings, and trust-content reviewed head/tree;
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
- recomputes exact landed trust paths/digest;
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

## Fresh focused GREEN

```text
python3 -I scripts/test_garnet_trust_kernel_review_status.py -v
```

Fresh grouped execution after implementation:

- premerge/deletion: 33/33 passed in 32.686 seconds;
- landed markers/registry: 12/12 passed in 9.787 seconds;
- discovery/object/worktree/append-only/CLI: 38/38 passed in 11.342 seconds.

Final integrated execution after the independent-review repairs: 92/92
focused tests passed in 85.552 seconds. The bounded authenticated transport also
passed 23/23 tests, and the strict Link-header parser passed 12/12 tests. The
added integration traps cover exact-current approval, later adverse review,
fork/base identity, canonical paths, append-only landed history, credential
isolation, and authenticated CI wiring without removing any earlier adversarial
case.

An independent adversarial reviewer approved the latest implementation after a
fresh 92/92 run and targeted edit/revert, pagination, path-alias,
credential-isolation, and U-19 squash-durability probes. That scoped approval is
not the authenticated GitHub approval required by the machine gate and is not
cross-OS evidence. The branch must still add its canonical structured record
and W_TRUST companion, obtain the recorded reviewer approval at the exact final
PR head, and pass the ordinary Linux/macOS/Windows required checks. Until then,
the branch-level gate remains correctly RED.
