# Garnet trust-kernel rolling review v2

**Status: normative operational policy.** S114's independent red-team is a
recurring control, not a one-time ceremony. A material trust-kernel change is
mergeable only when Git completely enumerates it and one canonical review
record binds the exact changed trust paths, bytes, reviewed content head,
authors, and designated independent reviewer; the live gate separately proves
that reviewer approved the exact current pull-request head.

The executable policy is
`scripts/garnet_trust_kernel_review_status.py` (status schema
`garnet.trust_kernel_review/v2`). Any disagreement is resolved by fresh gate
output on the current tree.

## Trigger surface

The gate's `TRUST_KERNEL_PREFIXES` and `TRUST_KERNEL_FILES` constants are the
machine authority. They cover the checker, interpreter, VM, stdlib registry,
Wasm runner, CLI authority entry points, governance/readiness policy scripts
and tests, GitHub workflows/actions/rulesets, and the named public enforcement
claims. The constants are deliberately conservative: an extra review is safer
than an unreviewed trust-spine change.

## Discovery is part of the proof

Gate mode derives the base from `merge-base(HEAD, origin/main)`. It resolves
both refs as commits with replacement refs disabled, then reads a full-index,
NUL-delimited, no-rename raw diff. It independently parses the base and head
commit/tree object bytes and requires the two enumerations to agree exactly. A
legitimate empty diff is distinct from an enumeration failure.

Every discovery failure is RED, including:

- missing or non-commit `HEAD` / `origin/main`, empty or malformed merge-base;
- command failure or timeout;
- a non-NUL-terminated, partial, invalid-UTF-8, duplicate-path, or unsupported
  raw status stream;
- an addition, change, type change, or deletion whose modes and old/new object
  identities do not match its status.
- identically truncated raw/name presentations that disagree with independent
  commit/tree-object traversal.

Gate mode also requires a clean porcelain-v2, NUL-delimited worktree. Staged,
unstaged, or untracked content and malformed/failed status enumeration are RED;
the reviewed commit, not mutable checkout state, is the proof target.

An exact deletion is not forbidden. It is represented as a tombstone with its
old mode/object/blob identity and zero new mode/object. Rename detection is
disabled, so a rename becomes independently reviewable delete-plus-add entries.

`--base`, non-`HEAD` `--head`, and `--changed-file` remain diagnostic aids but
cannot satisfy `--gate`; otherwise an incomplete caller-supplied list could
bypass enumeration. A `Trust-Kernel-Review:` trailer and `--assume-trailer`
also cannot satisfy v2.

## Canonical premerge record

Exactly one changed file whose name matches
`F_Project_Management/W_TRUST/**/*.review.json` must accompany a trust-kernel
change. Human-readable W_TRUST Markdown remains encouraged, but only the JSON
record satisfies the machine gate.

Review records are append-only. An existing record may not be modified,
deleted, or type-changed. A new record must be a regular, non-executable
`100644` blob; symlinks, executables, and gitlinks are RED.

The record schema is `garnet.trust_kernel_review_record/v2`, state `premerge`,
and has exactly these keys:

- `author_emails` — sorted canonical emails derived from raw commit objects in
  `base_commit..reviewed_head`;
- `author_ids` — sorted immutable GitHub user IDs derived from every commit in
  the authenticated pull-request commit collection;
- `base_commit` — the exact discovered merge-base;
- `blocking_findings` — an empty list;
- `content_digest` — the deterministic trust-change digest below;
- `repository` / `repository_id` — the authenticated upstream base repository;
- `head_repository` / `head_repository_id` — the authenticated fork head
  repository (it may equal the base repository, but is never assumed to);
- `pull_request_number` / `pull_request_id` — the authenticated immutable PR;
- `review_state` (`APPROVED`), `reviewer_id`, and `reviewer_login` — the exact
  required state and designated independent reviewer;
- `review_scope` — states that content proof does not extend or backdate
  `reviewed_head` coverage;
- `reviewed_head` and `reviewed_tree` — exact premerge provenance;
- `schema`, `state`, `touched_paths`, and `verdict` (`pass`).

The file must be UTF-8 canonical JSON: sorted keys, two-space indentation, LF,
and one final newline. Duplicate keys, unknown keys, missing keys, multiple
records, a deleted/missing record, prose-only companions, malformed identities,
self-review, non-pass verdicts, blocking findings, and noncanonical path aliases
(including backslashes for slash-delimited Git paths) are RED.

`review_id` is intentionally absent from a premerge record. The record must be
committed before the final approval exists; embedding the future review ID or
the record-containing commit SHA would be self-referential, while adding either
after approval would immediately stale-dismiss that approval under Garnet's
ruleset. Instead, the gate selects the authenticated `APPROVED` review from the
recorded reviewer whose `commit_id` equals the exact current PR head, then reads
that selected review object directly. Reviews are grouped by the immutable
reviewer ID; the selected row's current login must still match the record. The
newest decisive review from that identity wins: a later `CHANGES_REQUESTED` or `DISMISSED` event invalidates an
older approval, and a later approval at any head other than the exact candidate
is RED. The record's `review_state` is a required condition, not a claim that
author-supplied bytes can prove.

The gate does not trust either author list. It cross-checks `rev-list` plus its
count against an independent raw commit-object graph traversal, reads each raw
author email, and requires exact `author_emails`. Through the explicitly
injected bounded GOV-009 transport, it enumerates the exact PR, every review,
the selected review object, and every PR commit. Repository, PR, review,
reviewer, state, exact current approval head, commit set, and `author_ids` must
all match. Fork head and upstream base repository names and immutable IDs are
checked separately.
Unreachable/403 transport, malformed objects, duplicate IDs, partial or bad
pagination, missing authors, non-`APPROVED` state, or any mismatch is RED. The
reporter never reads a credential from the environment; checkout disables
credential persistence, the invoking shell removes token variables before
starting Python, and every child Git process receives an environment stripped
of credential-shaped names. A credential is accepted only through bounded
stdin and is never printed or persisted.

`reviewed_head` must name the trust-content commit after `base_commit`, must be
an ancestor of the current premerge head, and its resolved tree must equal
`reviewed_tree`. The final authenticated approval must instead name the exact
current head, including the committed record and companion.
The gate independently diffs both `base_commit..reviewed_head` and
`base_commit..current-head`; their trust-path sets and digests must equal the
record. It also traverses every raw commit object after `reviewed_head` and
compares each commit tree with every parent on the reviewed lineage. Therefore
an edit-then-revert and a merge-only trust resolution are RED even when endpoint
content returns to the reviewed bytes. A premerge record must not claim a
`merged_commit`.

## Change digest

For each trust entry sorted by path, the SHA-256 stream is framed as:

```text
garnet.trust_kernel.change/v2 NUL
status NUL path NUL old_mode NUL old_git_oid NUL old_blob_sha256_or_dash NUL
new_mode NUL new_git_oid NUL new_blob_sha256_or_dash NUL
```

Git object IDs bind repository identity; independently recomputed SHA-256 values
bind exact blob bytes. Modes and status bind executable/type semantics. An
addition uses an all-zero old identity; a deletion uses an all-zero new identity
and retains the old blob SHA-256 as its reviewable tombstone.

## Squash-durable landed marker

`verify_landed_review_marker` implements the postmerge state using schema
`garnet.trust_kernel_review_marker/v2`. The marker retains the premerge claim
and adds only squash-verifiable landed fields: `review_record_path`, the exact
committed record's raw-byte SHA-256, `merged_commit`, and `merged_tree`. It must
equal the committed premerge record for every reviewer/path/content/provenance
field. It intentionally does not repeat `review_id` or `approval_head`: the
postmerge verifier has no authenticated review transport, so accepting those
fields would turn unverified author input into a claim.

The verifier requires the record to be added or modified in the landed range,
the merged commit to be on `origin/main`'s replacement-ref-resistant
first-parent history after its base, the merged tree to match exactly, and the
landed base-to-merge trust paths/digest to match the record. It never requires
the pre-squash `reviewed_head` to be an ancestor of main or even to remain in the
object store. This is U-19's two-state contract: provenance is recorded before
merge; landed content is proven after squash without backdating review onto
unseen commits.

`F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json` is the production
registry. Every `landed/*.landed-review.json` path in the committed tree must be
listed exactly once, and every listed marker is loaded and verified during
ordinary status evaluation. A bad marker cannot remain dormant behind a
test-only helper. The registry and marker directory are themselves trust-kernel
paths. Every candidate commit transition is inspected: a marker may only be
added as a regular `100644` blob, existing marker bytes and modes are immutable,
and the registry may only retain or append entries. Deletion, replacement,
rollback to an empty registry, path aliases, or a malformed historical snapshot
is RED even if a later commit restores the endpoint bytes.

## Usage

```text
# Authoritative PR gate. The caller explicitly injects PR metadata and one
# bounded credential; the reporter does not inherit ambient credentials.
printf '%s\n' "$EXPLICIT_REVIEW_TOKEN" | python3 \
  scripts/garnet_trust_kernel_review_status.py --gate \
  --github-repo Island-Dev-Crew/garnet --github-pr "$PR_NUMBER" \
  --github-token-stdin

# Focused adversarial suite.
python3 -I scripts/test_garnet_trust_kernel_review_status.py -v

# Diagnostics only; never a merge proof.
python3 scripts/garnet_trust_kernel_review_status.py --changed-file README.md
```

The structured record binds what an identified reviewer attested. It does not
cryptographically prove that a human or agent performed an adequate review;
the same-PR W_TRUST companion, independent-review evidence, required checks,
and Jon-only merge boundary remain mandatory governance layers.
