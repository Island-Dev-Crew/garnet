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
Wasm runner, the whole `garnet-cli/src/` tree, governance/readiness policy
scripts and tests, GitHub workflows/actions/rulesets, every script named
literally in a required-context workflow as its producer, and the named public
enforcement claims. The
constants are deliberately conservative: an extra review is safer than an
unreviewed trust-spine change.

Prefer a prefix over a file list. Until H3-02 (2026-09-03) `garnet-cli` was
enumerated file by file, and three independent reviews found the same holes in
that enumeration (only the Codex hardening pass carries an in-repo identifier): the capability walk (`cap_manifest.rs`, `cmd/diff_caps.rs`,
`cmd/verify_gate.rs`), the manifest and Ed25519 signature verifier
(`manifest.rs`, `cmd/verify.rs`), and the script that produces the required
`PR dogfood evidence` context. The peer enforcement crates were already
whole-`src/` prefixes; `garnet-cli` was the anomaly. The script entries that
remain enumerated are checked against a derivation from
`.github/rulesets/garnet-main.json` and `required-context-producers.json` from
the paths named literally in workflow text (not the import or wrapper closure
of a producer), so a new required-context producer that escapes the `scripts/garnet_` naming prefix
fails `TrustSurfaceCoverageTests` instead of merging unreviewed.

## The surface is versioned

A landed marker binds the `touched_paths` and `content_digest` of the trust
subset of its landing edge. Classify that edge with a wider surface and the
sealed marker reports missing paths and a digest mismatch — so before H3-02 the
surface could not be widened at all without invalidating sealed history, and
the append-only rule correctly forbids editing a marker to say otherwise. That
was found by running the gate on the H3-02 widening, not by reading the code.

`TRUST_SURFACES` therefore names each surface version, and
`verify_landed_review_marker` classifies a marker's landing edge under the
surface that was **in force when it landed**. That is a question of provenance,
never of reading the landing's copy of the gate script. Each version after v1
has an **era stone**, a canonical JSON file
`F_Project_Management/W_TRUST/eras/<vN>.era.json` laid by the change that
introduced the version. A landing's surface is the latest version whose stone
was introduced at or before that landing on main's first-parent history, which
is append-only under the ruleset; a landing before every stone is v1 (that is
what the two pre-versioning markers, PRs #514 and #517, are). Stone history
must itself be append-only — a modified or deleted stone anywhere on the line
is a finding — so presence is monotone and the boundary is found by bisection.
Every registered version after v1 must have a stone, a stone for an
unregistered version is a finding, and a marker may carry `trust_surface`
only in agreement with the era in force (an explicit non-version value,
`null` included, is a finding; a `merged_commit` registered twice is a
finding). Every failure resolves to the current, widest surface.

The **runtime entry consistency boundary** runs inside every gate invocation:
the surface this process applies to live candidates (`CURRENT_TRUST_SURFACE`,
as bound at the entry point) must equal the latest era stone in the candidate
tree, or the repository verification is red. A copy that widens the constant
anywhere — its `__main__` block included — without laying its stone cannot run
green, and `--print-trust-surface` reports what the actual entry applies.

Six earlier designs are recorded here so they are not reintroduced, each
rejected by cross-family review with a reproduced construction.
Declaration-first let a new marker declare the old surface and hide the newly
covered paths on its own landing edge. Commit-only pinning let a second marker
replay a pinned `merged_commit`. A closed pin map bound to marker paths
preserved exactly the two pre-versioning markers and nothing sealed after them:
the first v2-to-v3 widening turned a valid v2 marker red with zero repository
changes. A regular expression over the constant in the landing's copy chose the
narrower era on a docstring, a single quote, an indent or a duplicate. A parse
of the constant's binding was defeated by a walrus, a `from … import` and a
tuple target — no inventory of Python binders is sound — and inferred the
pre-versioning era from the absence of a symbol. A seal comment beside the
constant was defeated by a copy that rebound the constant at its `__main__`
entry while carrying the seal. The common failure: any static reading of a copy
can be made to disagree with what that copy does. The ledger reads no copy; the
regression that proves it lands a v2 marker, lays a v3 stone with the live
surface widened, verifies the registry green, and shows a v3-era landing that
omits a v3-only path reported.

Live candidates are always judged by the current surface; only sealed history is
judged by its own.

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

### Post-record approval observation; sole U-59 exception to U-66

The ordinary venue law remains one CI firing per readback head. U-59 creates
one exception: an attempt-1 result whose only finding is absent exact-head
approval may receive one same-run, same-head **Re-run all jobs** after that
approval exists. The receipt is the exact-key
`garnet.trust_kernel_review_eligibility/v1` object defined in
`GARNET_WV_ACCEPTANCE_SUCCESSION_CONTRACT.md`, carried as the sole member named
`eligibility.json` in the uniquely named artifact
`r2-approval-pending-<run_id>-attempt-1`. The only eligible tuple is
`state=approval_pending_only` and `finding_codes=[approval-absent]`.

The following adopted contract text is transcribed verbatim from
`F_Project_Management/W_TRUST/REACCEPTANCE_REDESIGN_BRIEF_v2.md`:

```text
POST-RECORD APPROVAL OBSERVATION; SOLE U-59 EXCEPTION TO U-66.

A candidate is eligible for one re-evaluation only when CI attempt 1 is the
unique CI run for the pull_request event at the exact record-containing head
and its unique, single-member, canonical Actions artifact states
approval_pending_only/[approval-absent]: every content, provenance, succession,
transport, pagination, PR-identity, base-currency, and record predicate
evaluated before the approval boundary passes, and the sole finding is absence
of the recorded reviewer's decisive approval. Contexts structurally skipped
behind that deliberate RED are not success evidence.

Attempt 1 MUST bind repository and PR immutable IDs, base ref and base SHA,
candidate head and tree, record path and raw-byte digest, workflow ref and SHA,
event, run ID, run number, attempt number, producer-inventory digest, artifact
identity, exact receipt schema, and normalized finding codes. Attempt 2 MUST
retrieve that receipt by complete authenticated artifact enumeration and exact
raw-body digest under a job token whose sole permission delta is actions: read.

After the designated reviewer submits APPROVED for that exact unchanged head,
an Actions-write carrier MAY invoke exactly one Re-run all jobs on that same CI
run. Carrier identity and rerun privilege confer no review authority. A
close/reopen, push, dispatch, new run ID, re-run-failed-only, job-only rerun,
debug rerun, or non-CI producer rerun is not this exception.

Attempt 2 is valid only when run ID, run number, event, GITHUB_SHA, GITHUB_REF,
workflow ref/SHA, candidate head/tree/record digest, PR identity, open and
non-draft state, base-main identity/SHA, and producer census equal attempt 1;
run_attempt is exactly 2; fresh bounded transport re-enumerates the PR, every
commit, every review page, and the selected direct review object; the latest
decisive review is exact-head APPROVED; and every required context succeeds.

A fresh head-scoped run census MUST show only this CI workflow at attempt 2 and
every other producer at attempt 1. The verifier MUST completely paginate both
attempt-specific jobs endpoints:

  /actions/runs/<run_id>/attempts/1/jobs
  /actions/runs/<run_id>/attempts/2/jobs

The attempt-2 job-name multiset MUST equal the fully expanded inventory derived
from the predecessor's base-controlled workflow and matrices. Every expected
job MUST occur exactly once, bind the same run and head, have a positive job ID
absent from attempt 1, have nonempty start/completion timestamps, and conclude
completed/success. In particular, every job that succeeded in attempt 1 MUST
have a fresh attempt-2 identity; this distinguishes Re-run all jobs from Re-run
failed jobs. A default/latest jobs endpoint, unexpanded matrix placeholder,
skip, neutral, cancellation, duplicate, missing row, incomplete page, reused
job identity, or direct-object disagreement is RED.

r2_role_separation_v1 MUST authenticate immutable numeric identities. Let
REVIEWER_ID be the selected decisive reviewer's user ID; COMMIT_PRINCIPALS be
the union of every PR commit author.id and committer.id; and CARRIER_ID be
triggering_actor.id on the attempt-2 workflow-run object, not actor.id from the
initial event. REVIEWER_ID, CARRIER_ID, and every COMMIT_PRINCIPAL MUST be
pairwise disjoint, positive, and login-consistent across directly read objects.
Missing or malformed identity is RED.

Immediately before merge, authenticated transport MUST repeat the
PR/head/tree/base/record/latest-review readback and the complete live governance
projection, including bypass []/never and exact required-context identity and
posture. This proves only the instant read; it is not an atomic merge lock.

Any other attempt-1 finding, mutable-field reliance on the replayed event,
movement, incomplete transport, additional run or attempt, partial rerun,
attempt-2 failure, identity overlap, or final-readback failure voids the
exception. There is no attempt 3. The cure is a new linear record successor and
a new approval venue.
```

DP10 composes additively with the block's singular `REVIEWER_ID` shorthand.
When the selected review carries supplemental decisive reviews, the mechanical
reviewer set is the designated primary reviewer plus every selected
supplemental reviewer. Every member is positive and login-consistent, is
pairwise disjoint from every other reviewer, `CARRIER_ID`, and every
`COMMIT_PRINCIPAL`, and no supplemental reviewer can replace the designated
primary.

For attempt 2, the replayed event payload supplies stale venue coordinates and
no current authority. The reporter re-reads live PR, base, commit, review,
artifact, jobs, workflow-run, governance, bypass, and required-context state
through bounded authenticated transport. `r2_role_separation_v1` remains
`OPEN-UNTIL-IMPLEMENTED`; until its executable proof and the distinct carrier
identity exist, this adopted law cannot activate.

DP5 assigns the final premerge readback to both actors: the reporter emits the
authenticated readback, and Jon reads it immediately before the merge click.
Both acts detect divergence at their observation instant; neither prevents a
later review, head, base, governance, bypass, or context change. A delay or
visible UI-state change requires another reporter emission and another Jon
readback.

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

Branch-currency merge commits created by **Update branch** are forbidden on
trust-kernel pull requests. Rolling review v2 rejects non-linear reviewed
lineage; the lawful cure is always a linear successor based on current main.

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

## WV acceptance succession and the two-pair model

WV acceptance carries two distinct pairs. `native_accepted_pair` is immutable
through record-only succession. `successor_observed_pair` is a non-authoritative
accounting recomputation at the successor boundary and cannot be represented as
native evidence. Only a closed, bounded R3 event certificate may establish a
later accepted pair. The exact certificate and terminal-transcript schemas are
defined in `GARNET_WV_ACCEPTANCE_SUCCESSION_CONTRACT.md`.

The following adopted contract text is transcribed verbatim from
`F_Project_Management/W_TRUST/REACCEPTANCE_REDESIGN_BRIEF_v2.md`:

```text
RECORD-ONLY ACCEPTANCE SUCCESSION.

An accepted WV boundary MAY succeed across a squash without repeating native
platform execution only through one effective garnet.wv_acceptance_succession/v1
certificate in the append-only succession registry.

The certificate MUST bind the native root, the final reviewed and approved PR
record tip R, the authoritative content landing B, the complete producer-
censused H..R graph, the exact R-tree/B-tree equality, the base-controlled
record classifier and digest definition, the predecessor's exhaustive
record_consumer_inventory, preservation hashes, and a linear predecessor
certificate.

The certificate suffix, registry, classifier, and producer definitions MUST
be explicit base-controlled trust-kernel triggers. Their exact raw bytes MUST
enter the rolling-review content digest and receive decisive exact-head
approval. Record classification or digest exclusion alone is not review.

Record-path membership is necessary but not sufficient. Every edge operation
MUST be collected under predecessor law when it occurs. Producer qualification
MAY be decided only once over the complete walk after the terminal record and
its exact-head approval exist. A later record MUST NOT erase or reclassify an
earlier touch. Any transient or endpoint non-record touch, operation absent
from the predecessor consumer inventory, reporter movement outside
r1_reporter_constant_projection_v1, unexplained producer output, historical-
record mutation, or incomplete graph is content drift and is RED.

The certificate MUST restate both native_accepted_pair and
successor_observed_pair. Native_accepted_pair MUST equal the predecessor's
accepted pair exactly. Successor_observed_pair MUST be independently
recomputed at B by two independent implementations and MUST NOT be represented
as native evidence. Every pair-input difference MUST be exhausted by qualified
record operations under the unchanged digest law.

The succession is effective only when establishment-time authenticated
transport derives the exact decisively approved certificate PR tip Q and merge
identity; the complete B..Q walk is qualified record-only; the certificate-
containing landing M is on authoritative main first-parent history after B;
tree(Q) equals tree(M); the landing edge is independently censused; and exactly
one canonical garnet.wv_acceptance_effectiveness/v1 transcript and
registry append durably bind those facts. The effectiveness receipt is terminal:
it MUST NOT move a pair, claim its own Q/M bridge, or require another receipt.
Ordinary verification MUST use the receipt and main objects, not live forge
state. Current HEAD MUST descend through the unique linear effective tip.
Ambiguity, absence, duplicate receipt, or registry fork is RED.
```

The verbatim native-pair equality sentence above is exact: R1 certificates form
only the native-rooted succession prefix and cannot follow an R3 event. After
an effective R3 event, `native_accepted_pair` remains the immutable root pair;
later record-only observation preserves the event's current accepted pair while
recomputing raw movement separately. The type-order and equality tests in
`GARNET_WV_ACCEPTANCE_SUCCESSION_CONTRACT.md` prevent a post-event pair from
being relabeled as native evidence.

The effectiveness transcript is a terminal receipt. It has no effectiveness
`Q_E` or `M_E`, cannot move either pair, and cannot be the subject of another
receipt. Its later verifier uses the committed transcript, its registry entry,
and authoritative-main Git objects; live forge loss after a valid anchor is
non-authoritative, while omission, contradiction, or forge loss before capture
is RED. The transcript records the brief's residual trust surface rather than
claiming to eliminate it: capture-time GitHub authenticity and completeness,
bounded credential transport, predecessor-base producer correctness,
independent review, Jon's merge, Git object integrity, and SHA-256.

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

The sole U-59 sequence is mechanically ordered:

1. Attempt 1 emits and uploads exactly one canonical `eligibility.json` receipt
   even though the ordinary gate exits RED.
2. The designated reviewer submits a decisive `APPROVED` review for the exact
   unchanged record-containing head.
3. A distinct Actions-write carrier invokes exactly one **Re-run all jobs** on
   the same run; there is no close/reopen, new run, partial/job/debug rerun, or
   attempt 3.
4. Attempt 2 treats the event payload as stale coordinates, downloads and
   verifies the attempt-1 receipt through `actions: read`, performs fresh live
   transport, and proves through both attempt-specific jobs APIs that the fully
   expanded job-name multiset reran with new job IDs and completed successfully.
5. The reporter emits the final authenticated PR/head/tree/base/record/review
   and governance readback. Jon reads that emission immediately before the
   merge click. The readback detects only its observation instant; delay or a
   visible state change restarts this readback step.

The structured record binds what an identified reviewer attested. It does not
cryptographically prove that a human or agent performed an adequate review;
the same-PR W_TRUST companion, independent-review evidence, required checks,
and Jon-only merge boundary remain mandatory governance layers.
