# Garnet main-branch governance contract

`garnet-main.json` is the exact active ruleset expected on the public
`Island-Dev-Crew/garnet` default branch. `repository-settings.json` contains the
merge and Actions settings that complete that contract. This first slice is a
declarative mirror only. Once the prepared validators land, candidate PR CI
performs strict static validation and never receives an admin credential. The
separate human ceremony then runs the admin-authenticated live comparison; a
public API read, prose promise, or locally green branch does not count as
no-bypass proof.

The repository is presently maintained by one organization member. Requiring
an approval or a code-owner approval would make that maintainer unable to merge
their own PR, so the bootstrap profile deliberately requires **zero approvals**.
The prepared machine-status model therefore exposes independent reviewer identity as
`configured: false`, `verified: false`, and an explicit limitation. It does not
turn an otherwise valid solo-profile PR red, and it never re-labels an in-thread
or self-authored v2 review record as independent GitHub actor approval.

Apart from that explicitly unresolved identity boundary, every update must
still arrive through a PR, all review threads must be resolved, the registered
GitHub-Actions checks must pass on the latest base, auto-merge is disabled, no
actor has bypass rights, and a human must perform the final squash or rebase
merge.

The required ledger includes the formerly path-scoped agentic-dogfood,
macOS-Studio, web/PWA, and pull-request parser-fuzz workflows plus a Windows
Studio trust job. They run on every PR so GitHub cannot leave a required context
pending after a path-filter skip or make a platform trust lane merely advisory.

PR #488 is the point-in-time bootstrap proof that all 31 names now emit on an
unrelated workflow-only PR. This declarative mirror alone cannot prevent a
future producer from becoming absent, filtered, conditional, or ambiguous. A
separate protected producer-policy contract and validator must map every
required context exactly once to an unconditional PR-to-main workflow/job.
Until that gate lands, producer availability remains an explicit residual risk.

The pre-activation ledger intentionally contains 31 contexts and intentionally
does **not** include `Base-controlled trust policy`. The separately prepared
`pull_request_target` workflow is not live or required in this baseline. Once
bootstrapped on `main`, it executes only policy code from the base checkout; the
candidate checkout has no persisted credentials or submodules and is never
executed. The base validator then checks candidate governance, v2 review-record
scope/digest, and byte identity of the protected workflow. The candidate cannot
change that protected workflow in-band because the old-base validator requires
byte identity. Other policy evolution remains subject to the exact-head v2
rolling review. A separately typed policy-upgrade record is not implemented by
this bootstrap and is therefore planned, not an enforced mechanism.

GitHub binds an ordinary required Actions check to its job context and the
GitHub Actions integration, **not** to a workflow file, matrix, or event. The
prepared trusted validator therefore scans every candidate workflow as inert
UTF-8 data, requires canonical job mappings, rejects unapproved dynamic job
names, and proves `Base-controlled trust policy` occurs exactly once at the
protected workflow's `policy` job. It also reads the Actions API and requires
that exact default-branch workflow path/name to remain `active`. This removes
accidental or in-repository context collisions, but it is not workflow-ID
binding. After bootstrap, an organization-level required-workflow rule is the
stronger upgrade if the organization plan exposes it; until then, this platform
limitation remains an explicit residual risk rather than an
independent-approval claim.

If the owner later authorizes a second accountable maintainer, that person must
accept repository write access and complete one shadow-review cycle before this
pre-registered upgrade is allowed:

1. set `required_approving_review_count` to `1`;
2. set `require_code_owner_review` and `require_last_push_approval` to `true`;
3. apply the live ruleset change and update the checked-in contract together.

### Bootstrap order for `Base-controlled trust policy`

This ordering is mandatory because `pull_request_target` reads workflows from
the default branch:

1. merge the workflow, trusted scripts, and tests in a human-only bootstrap PR
   while `Base-controlled trust policy` is not a live required context;
2. add the base-controlled context to the checked-in ledger in a separate
   activation PR, verify that the already-landed workflow emits the exact job
   name, is API-reported `active`, and owns the only candidate occurrence of
   that context; then add it to the live required-check ledger while that PR is
   open and rerun the authenticated governance drift gate before the human
   merge.

The solo activation mechanically proves base-owned static validation, not
independent actor approval. If a second reviewer is later authorized, use two
more PRs: first land the exact two-owner CODEOWNERS matrix only after API-confirmed
`write` or `admin` access (GitHub maps `maintain` to `write`); then switch to the
complete multi-reviewer ruleset profile with that CODEOWNER's review and an
admin-authenticated live readback. A multi-reviewer profile can never downgrade
to the solo profile in band.

The base-owned validator accepts exactly three ledger states: trusted 31 to
candidate 31 before activation, trusted 31 to candidate 32 for the one explicit
activation delta, and trusted 32 to candidate 32 thereafter. It rejects 32 to
31. Never activate the 32nd context before step 1: doing so creates a
missing-context deadlock. Never claim independent reviewer identity is verified
merely because a JSON record names a different string.

The repository-settings contract also pins default Actions workflow permission
to `read` and `can_approve_pull_request_reviews` to `false`. The live drift gate
counts only an admin-authenticated read as authoritative for the empty bypass
ledger; anonymous/public output is diagnostic only.

### External Action pin updates

Every external workflow action is bound to a full 40-character commit in
`external-action-pins.json`; mutable tag or branch references are forbidden,
including release publishing. The manifest records the reviewed upstream ref
and peeled commit, while CI remains network-free and verifies exact equality
between all workflow occurrences and that manifest.

An update is an explicit human-reviewed digest change: resolve the upstream
tag/branch with authoritative Git refs, peel annotated tags to commits, update
the manifest and every occurrence together, run the workflow file/YAML/schema,
Node-runtime, MSRV, and action-integrity suites, attach a W_TRUST companion with
fresh Linux/macOS/Windows evidence, and leave the PR for Jon. Do not schedule an
automatic updater, trust a version comment, or weaken the gate when a ref cannot
be resolved. The `stable` and `nightly` Rust channels remain runtime channel
choices even though the action implementation that selects them is immutable.
Because `dtolnay/rust-toolchain` normally derives the channel from its action
ref, the full-SHA form is always paired with an explicit `with.toolchain`
input; omitting that input is a policy error.

### Required-context semantic fingerprints

Producer identity is not satisfied by a workflow path, job id, and emitted
name alone. Each row in `required-context-producers.json` carries a lowercase
SHA-256 over the immutable projection of the workflow-level trigger and policy,
the selected job plus its complete transitive dependency jobs, their ordered
steps, and the selected matrix member. The pre-activation gate separately pins
the ordered aggregate of all 31 active fingerprints, so a candidate cannot
coordinate a workflow behavior change with a freshly invented digest.

The schema gate also rejects the exact vacuous forms `run: "true"`,
`if: "false"`, and `if: "${{ false }}"` before a producer can be projected.
The semantic digest is style-independent but byte-stable over the resulting
canonical AST; ordinary YAML formatting is not treated as behavior, while a
command, action, condition, environment, permission, trigger, dependency, or
matrix-member change is. Updating a producer's behavior therefore requires the
same reviewed trust-kernel path as updating the workflow itself.

### Workflow update maintenance window

The base-controlled workflow is byte-immutable while its context is required.
To change it, a human administrator declares a maintenance window, freezes
merges, removes only that required context from the live and checked-in ledgers,
confirms the remaining protections by authenticated readback, lands the reviewed
workflow change through a human-only PR, observes the new context from `main`,
then performs the explicit 31-to-32 reactivation and records a second
authenticated readback. Policy-upgrade JSON cannot waive this ceremony.

Verified commit signatures are a separate follow-up. They should become a rule
only after every active authoring lane has a tested signing setup; turning the
rule on first would strand already-open unsigned branches and create an admin
disable ritual rather than durable governance.
