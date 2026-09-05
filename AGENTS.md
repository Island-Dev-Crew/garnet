# AGENTS.md — Garnet Runtime Documentation Contract

## Documentation First

Treat every `AGENTS.md` file as part of Garnet's runtime documentation contract, not as optional contributor notes. Garnet is an agent-native language platform; long-horizon agents must be able to recover local intent, invariants, and "what not to break" from files that live beside the code they govern.

This repo uses a documentation hierarchy:

- `/AGENTS.md` owns repo-wide rules, the contract index, and cross-cutting architecture.
- Crate-level `AGENTS.md` files own implementation contracts for each Rust crate.
- Spec and project-management `AGENTS.md` files distinguish normative language truth from episodic handoff history.
- Template docs under `garnet-cli/templates/` define what new Garnet projects should teach agents by default.

The closer the doc is to the code, the more concrete it should be. Parent docs explain boundaries and stable seams; child docs explain local behavior, invariants, tests, and update rules.

## Memory-Kind Mapping

Garnet's own memory taxonomy applies to the repository:

- Working memory: current task plans, local run notes, and active PR descriptions.
- Episodic memory: handoffs, verification logs, release notes, and dated project-state files.
- Semantic memory: language specs, architecture docs, research papers, and public README/FAQ material.
- Procedural memory: `AGENTS.md`, contribution rules, test ladders, commands, and repeatable workflows.

A stable workflow that changes agent behavior belongs in procedural memory, not only in a chat transcript or a maintainer's head.

## Lane 0 Truth-Freeze Gate

`python3 -I scripts/garnet_lane0_truth_freeze_status.py --gate` is the machine
authority for the Lane 0 first-parent archive and U-18 resume contract. It
derives the archived PR order and full squash-main SHAs from local Git history,
then checks the materialized P7/P7-T1..P7-T4 references and rejects stale
P8/P9/P10 resume pointers. Run
`python3 -I scripts/test_garnet_lane0_truth_freeze_status.py` after changing the
checkpoint, its locked P0 gates, or the mission SOTU renderer. This gate reads
only the local checkout; it must not read a fork's main branch or ambient
credentials.

## Rust MSRV Contract

Cargo `rust-version = "1.95"` is the single workspace MSRV. Every active
workspace member inherits that value; the excluded Studio backend and parser
fuzz workspace declare it directly. Ordinary CI jobs other than Clippy
continue to track moving stable, while the existing required CI and Studio
contexts also compile under exact Rust 1.95.0. The `clippy` job is pinned to
Rust 1.98.0 because it enforces `-D warnings`; changing that compiler is a
reviewed workflow-policy change rather than an ambient toolchain event. Do not
add a `rust-toolchain.toml` pin or raise the floor without updating every
active manifest, current public/contributor surface, the existing required
workflow checks, and this contract in one Jon-reviewed change.

Run `python3 -I scripts/test_garnet_msrv_status.py` and
`python3 -I scripts/garnet_msrv_status.py --gate` after changing a Rust
manifest, current MSRV wording, or the exact-floor workflow wiring. This gate
is stdlib-only: its narrow workflow projector accepts only the canonical
job/matrix/step shape needed by the MSRV contract and rejects ambiguous YAML
features or indentation. Comments, disabled steps, or commands in another job
do not satisfy it. The broader repository workflow-policy gates continue to
use their separately pinned typed-YAML boundary.

## Governance and Action Integrity

Every external `uses:` entry in `.github/workflows/` is pinned to a reviewed
full 40-character commit in
`.github/rulesets/external-action-pins.json`. Run
`python3 -I scripts/test_garnet_workflow_action_integrity_status.py` and
`python3 -I scripts/garnet_workflow_action_integrity_status.py --gate` after
changing a workflow or action pin. Pin updates are manual trust-kernel changes:
resolve the exact upstream tag or branch, record the peeled commit and source
ref in the manifest, update every use, preserve the immutable-action and Node
runtime gates, attach a W_TRUST review companion with fresh cross-OS evidence,
and leave the PR for Jon's merge. Comments and mutable tags are not authority.

The Jon-only governance exception covers the entire `scripts/garnet_github_*`
family, not a filename allowlist. Those transport, projection, and live-policy
scripts govern the merge boundary itself and must never auto-merge, inherit an
ambient credential, or print/persist a credential.

The rolling trust-kernel review gate preserves exact reviewed bytes across
post-review merge topology. A merge parent outside the reviewed lineage is
admissible only when both the merge's complete trust-kernel snapshot and its
SHA-256 byte digest equal those at `reviewed_head`; an unequal snapshot or
digest remains red. This topology rule does not extend review coverage over
the outside parent and does not relax transient-touch checks on the reviewed
lineage. Run `python3 -I scripts/test_garnet_trust_kernel_review_status.py`
after changing the walk or its trust-snapshot identity.

## Dogfood PR-Body Evidence Contract

`scripts/check_dogfood_pr_body.py` accepts the legacy Desktop dogfood bundle
heading and the Evidence bundle heading. When both occur as real Markdown
headings, the first one in document order is authoritative; configuration
tuple order must not override document order. Keep regression fixtures for
both heading orders. Run `python3 -I scripts/test_check_dogfood_pr_body.py`
after changing the checker or this contract.

Three rules bind how the checker reads a body. A required heading matches only
when its heading text equals the contract heading exactly after trailing
whitespace is stripped (`### Current truth — none stated` is not
`### Current truth`). A section ends at the next heading of the same or higher
level, so a `## ` closes an open `### ` section and a checked item under a
later `## ` never counts for it. A checked item counts as evidence only when it
carries a token a reviewer can recompute: a command, path, or value in
backticks, a repo path, a 7–40 hex SHA, an `https://` URL, a `#PR` reference,
or a numeric result; Remote verification also accepts the named CI/PR check
with its expected status, and the evidence bundle also accepts a named artifact
and where it lives. The merged bodies of #545 and #546 live under
`scripts/fixtures/dogfood_pr_bodies/` as positive fixtures that must keep
passing. The `git diff` the checker runs is bounded at 30 s and fails closed.

## WV-6 / WV-7 Acceptance Gates

`F_Project_Management/LAUNCH/WV6_WV7_ACCEPTANCE_CONTRACTS.json` preserves the
established meanings: WV-6 is the native-Windows Core Ring Tier 1 + Minimum
Shelf/MCP proof, and WV-7 is the winget/Scoop dry-run + devcontainer/Docker +
installer happy-path distribution proof. Run
`python3 -I scripts/test_garnet_wv_acceptance_status.py` after changing either
contract or reporter. The two `--gate` commands are expected to exit nonzero
with `state=pending` until their exact-candidate, hash-verified Windows evidence
manifests exist; never turn absence into acceptance or perform a Jon-only
action from the reporter.

Post-acceptance U-35 drift does not redefine the product digest. The WV-6 and
Minimum Shelf reporters retain the pair at the exact frozen head only when
that head's tree and pair match the recorded boundary and every changed path
through the candidate tip is in the separately enumerated record class. That
class includes the established evidence/review surfaces and
`ops/wv6-reaccept/**`; any other changed path is RED and is named. Keep this
tolerance separate from `FROZEN_MUTABLE_PREFIXES`, which remains the exact
four-prefix digest definition.

Acceptance is the last content operation on a candidate. Any later merge that
changes a non-record byte supersedes the existing acceptance with preservation;
it never widens the record class or bends the verifier. Freeze the final merged
tree, perform native acceptance, and rebind its pins in one terminal ceremony.
Later reviewer records may ride above that boundary only through the proven
U-35 record-class tolerance. This is U-57, demonstrated by
`ops/gate-topology/evidence/11-third-merge-integration-red.txt`.

## Lane 0 Frozen Backlog

`ops/lane0/frozen-backlog.json` is the machine authority for the Lane 0
claim-state freeze. Only `implemented`, `partial`, `planned`, and `research`
are valid states. Implemented clauses must name tracked current code and
tracked executable evidence; partial entries must preserve both the implemented
and open clauses. A future destination is always labeled `future-not-evidence`
and cannot promote a claim.

Run `python3 -I scripts/test_garnet_frozen_backlog_status.py` and
`python3 -I scripts/garnet_frozen_backlog_status.py --gate` after changing the
backlog, its human rendering, or any evidence path it cites. Lane 2C remains
partial until a deterministic reporter verifies three exact-candidate stress
cases exceeding four minutes; ignored fixtures and the historical 0.03-second
run do not satisfy that contract.

## Lane 0 Closeout Gate

`python3 -I scripts/garnet_lane0_closeout_status.py --gate` is the final
repository-local authority for the Lane 0 reconciliation archive. It requires
exact, sorted SHA-256 coverage of `ops/lane0/evidence/`, verifies the
ARCHIPELAGO ledger with the upstream JSON hash algorithm and zero-hash genesis,
derives all four denominators from the captured reporter payloads on every
run, and requires the exact command inventory, gate bindings, chronology, and
an independently approved final integrated review with zero open Critical or
Important findings. Because Garnet squash-merges PRs, the durable review marker
separates the exact pre-squash `reviewed_head` from landed-content proof:
`merged_commit` must exist on the authoritative upstream main first-parent
history and its tree must equal `reviewed_tree`. A reviewed-tree mismatch, a
missing merged commit, an unavailable authoritative main ref, or a merged
commit absent from that first-parent history is RED. Never require the
pre-squash reviewed head to be an ancestor of main, and never use the content
digest to backdate independent review over later commits. Content-proof Git
reads ignore `refs/replace`, and each lane must pin its exact reviewed and
landed boundary facts outside the two mutable state copies. It rejects
symlinks, path traversal, duplicate entries, extra files, missing files, stale
hashes, semantically contradictory reseals, a fifth readiness denominator, or
any launch state other than HOLD. The gate reads no fork branch, ambient
credential, or environment variable.

Run `python3 -I scripts/test_garnet_lane0_closeout_status.py` after changing the
Lane 0 audit, evidence manifest, ledger, closeout state, or four-denominator
record. The S6 verdict remains `advisory` at band 3/5 until the pending browser
runtime journey and current Lane 2C duration proof exist; an empty waiver list
does not turn advisory evidence into enforced governance.

Every `ops/**/evidence/**` path is byte-exact under `.gitattributes` (`-text`).
Do not normalize or silently reseal evidence to repair a platform checkout;
the worktree bytes must equal the committed blob bytes on Windows and Unix.
The archived Lane 0 PR-body candidate remains review provenance only. Its
changed-path check is reconstructed from the main-reachable
`base..merged_commit` range plus the exact disclosed post-review companion;
never require the discarded pre-squash candidate object or a `refs/pull/*`
fetch to make the closeout gate pass.

## Repository Text Byte Policy

`python3 -I scripts/garnet_text_byte_policy_status.py --gate` enumerates
CR-bearing text blobs at the exact current commit and tree. It is path-based:
never replace its violating-path output with a pinned repository-wide count.
Captured bytes under `proofs/**` and `ops/**/evidence/**` are excluded from
this text policy and remain governed by their byte-exact manifests and the
evidence-integrity gate.

Run `python3 -I scripts/test_garnet_text_byte_policy_status.py` after changing
the checker, its exclusions, or line-ending attributes. Generated Mission
Control HTML must be regenerated by its owning `render-sotu.mjs`, never
hand-edited; the renderer must reject CR-bearing inputs and output.

## Research Corpus and Competitive Watch

`research/README.md` is the canonical A–F corpus map. Legacy paths stay live
until a dedicated link-checked migration; do not move engineering crates or
import absent external corpus files during documentation cleanup. The former
June reassessment path is an explicit compatibility pointer to the canonical
`research/2026-06/` copy.

`research/QUARTERLY_COMPETITIVE_WATCH.md` owns the standing cadence and report
contract. Run
`python3 -I scripts/test_garnet_quarterly_competitive_watch_status.py` and
`python3 -I scripts/garnet_quarterly_competitive_watch_status.py --gate` after
changing it. The contract being active is not a completed watch report. Search
misses are coverage statements, never evidence that a competitor or standard
does not exist.

## Required Contract Index

Every path below is part of the current contract surface and must remain present unless the owning scope is removed or renamed.

- `/AGENTS.md`
- `/C_Language_Specification/AGENTS.md`
- `/F_Project_Management/AGENTS.md`
- `/garnet-parser-v0.3/AGENTS.md`
- `/garnet-parser-v0.3/fuzz/AGENTS.md`
- `/garnet-interp-v0.3/AGENTS.md`
- `/garnet-check-v0.3/AGENTS.md`
- `/garnet-memory-v0.3/AGENTS.md`
- `/garnet-actor-runtime/AGENTS.md`
- `/garnet-stdlib/AGENTS.md`
- `/garnet-cli/AGENTS.md`
- `/garnet-cli/templates/AGENTS.md`
- `/garnet-convert/AGENTS.md`
- `/garnet-cst/AGENTS.md`
- `/garnet-prim-macros/AGENTS.md`
- `/garnet-lsp/AGENTS.md`
- `/garnet-suggest-llm/AGENTS.md`
- `/garnet-vm/AGENTS.md`
- `/garnet-wasm/AGENTS.md`
- `/garnet-registry-stub/AGENTS.md`
- `/apps/garnet-studio/src-tauri/AGENTS.md`
- `/apps/garnet-studio-macos/AGENTS.md`
- `/examples/AGENTS.md`
- `/xtask/AGENTS.md`

Run `python3 scripts/check-agent-contracts.py` after changing this index or any `AGENTS.md` file.

## Change Rules

Before editing a subsystem, read the closest owning `AGENTS.md` plus this root file.

When a code change alters behavior, ownership, invariants, public commands, template shape, or required tests, update the closest owning `AGENTS.md` in the same change. Update parent docs too when the higher-level architecture or boundary changes.

Do not let handoff files become the only source of current truth. If a handoff records a durable rule, promote that rule into the relevant spec or `AGENTS.md` file.

Do not add hidden compatibility seams, generated artifacts, or ad hoc scratch directories as tracked content unless the owning contract says they are durable project state.

## Rolling Trust Record Succession

Structured `F_Project_Management/W_TRUST/*.review.json` records are append-only.
When a candidate range contains more than one record, their introduction
commits and `reviewed_head` values must form the same strict linear ancestry.
Only the uniquely tip-most record binds the full-range `touched_paths`, content
digest, and authenticated GitHub review transport. Earlier records remain
preserved provenance: modification, deletion, forked succession, duplicate
heads, or a later record that backdates `reviewed_head` is RED.

Run `python3 -I scripts/test_garnet_trust_kernel_review_status.py` after changing
record discovery, succession selection, content binding, or review transport.

## Rolling Review Venue, Readback, and Canonical Bytes

U-66 remains the default venue law: one CI firing per exact readback head, and a
close/reopen or rerun is not an idempotent refresh. U-59 is its only exception.
It permits one same-run, same-head **Re-run all jobs** only after attempt 1 emits
the canonical `approval_pending_only/[approval-absent]` eligibility receipt and
the designated reviewer approves the exact unchanged record-containing head.
The carrier is a distinct authenticated Actions-write identity and transports
only the rerun; it gains no review authority. No partial, job-only, debug,
dispatch, close/reopen, new-run, or third-attempt path is equivalent. Until the
carrier exists and `r2_role_separation_v1` is executable and green, the
exception is contract law but is ineligible for activation.

Attempt 2 treats the replayed event payload as stale coordinates, never current
authority. It uses fresh bounded transport for the PR, base, commits, reviews,
selected review, attempt-1 artifact, workflow-run object, both attempt-specific
jobs endpoints, governance, bypass, and required contexts. The all-jobs proof
compares the fully expanded predecessor-owned job multiset and requires a new
positive job ID for every row, including jobs that passed on attempt 1.

The final premerge readback is dual. The reporter emits the authenticated
PR/head/tree/base/record/latest-review and governance projection; Jon reads that
emission immediately before the merge click. These are detection acts, never
prevention or an atomic merge lock. Any delay or visible UI-state movement
requires a fresh reporter emission and a fresh Jon readback. This procedure
does not delegate Jon's merge authority to the reporter, reviewer, or carrier.

U-74 is the rebase-propagation doctrine. CI evaluates the advertised PR head,
not a synthetic merge ref. A cure that lands on main reaches an existing PR
only after that branch rebases onto the new base and emits a fresh event; when a
sibling merges under the strict up-to-date policy, the other siblings become
`BEHIND`. A rerun cannot propagate new base bytes. The U-59 exception may
re-observe one unchanged eligible head, but it cannot substitute for a U-74
rebase.

Canonical structured records use the byte contract implemented at
`scripts/garnet_trust_kernel_review_status.py:921`:

```python
(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()
```

That means UTF-8, literal non-ASCII characters, lexicographically sorted object
keys, two-space indentation, LF line endings, and exactly one trailing LF. The
reporter compares the supplied bytes to this serialization; duplicate keys,
unknown keys, aliases, alternate escaping, reordered keys, CRLF, missing final
LF, or extra trailing bytes are RED.

Run `python3 -I scripts/test_garnet_trust_kernel_review_status.py` and the agent
contract checks after changing this venue, readback, propagation, or canonical-
record procedure.

The U-59 wiring (L1 act 2) lives in `.github/workflows/ci.yml` and two
scripts. `scripts/garnet_trust_kernel_review_eligibility.py emit` runs at
attempt 1 after the reporter, classifies the reporter's `--status-out` problem
list through a fixed table (only the exact "authenticated decisive review from
the recorded independent reviewer is absent" maps to `approval-absent`; every
other finding maps to its own code and makes the receipt `ineligible`), and
writes the canonical `garnet.trust_kernel_review_eligibility/v1` receipt that
the pinned `actions/upload-artifact` step carries as the sole member
`eligibility.json` of `r2-approval-pending-<run_id>-attempt-1`. A record-less
candidate emits no receipt. `verify` runs at attempt 2 before the reporter:
complete artifact enumeration, one-hop authenticated download through
`scripts/garnet_actions_artifact_transport.py` (Authorization is stripped on
the blob hop; the hop may name only a subdomain of `.blob.core.windows.net` or
`.actions.githubusercontent.com` on the default port, never an IP literal;
`application/zip`/`application/octet-stream` only; 8 MiB cap),
single-member ZIP parse without extraction, canonical receipt equality, live
PR/base/head/tree/record/workflow/run/event/producer-inventory equality, and
`run_attempt == 2`; the reporter consumes that verdict through
`--eligibility-verdict` and fails closed on attempt 3 or later for every
candidate. The verdict carries the carrier — the run's `triggering_actor` as
the API reports it — proven neither an author nor the reviewer of the record,
and the reporter re-checks the carrier at the seam. `expected_job_multiset`
and `verify_jobs_and_census` are the act-4 readback callables; act 2 tests
them and does not wire them into CI; attempt 1 may carry an unexpanded matrix
row, attempt 2 must carry the exact expanded multiset with no reused job
identity. The `actions: read` workflow permission is the sole permission
delta; no CI job may hold a write permission. **Acceptance is construction-
only:** `R2_ACTIVATION_AUTHORIZED = False` in the reporter adds a named finding
to every record-bearing attempt-2 result even with a valid verdict, so U-59
grants no eligibility as a machine-enforced fact; flipping it is the activation
act, a change to the gate that merges under it (Integrity Rule 1), and it
lands only with the complete `r2_role_separation_v1` proof.

Run `python3 -I scripts/test_garnet_trust_kernel_review_eligibility.py`,
`python3 -I scripts/test_garnet_trust_kernel_review_status.py`, the workflow
policy suites, and `python3 -I scripts/test_garnet_required_context_contract.py`
after changing the receipt schema, the classifier table, the archive transport,
the attempt law, or the CI receipt/verify wiring. Changing any `ci.yml` job
moves every `ci.yml` producer fingerprint in
`.github/rulesets/required-context-producers.json` and the aggregate constants
in `scripts/garnet_required_context_contract.py` and
`scripts/garnet_github_governance_gate.py`; regenerate them from
`producer_semantic_sha256`, never by hand.

## Phase ID Allocation

Phase identifiers (e.g. `Phase 6BT`) are a single shared global counter. With
multiple concurrent agents, hand-picking a letter causes collisions (PR #74 and
PR #75 both shipped as "Phase 4BI"). Before choosing a phase id:

1. Run `python3 scripts/garnet_phase_id.py` and use the id it prints.
2. Never hand-pick or reuse a letter from memory or a stale handoff.
3. CI and agents may run `python3 scripts/garnet_phase_id.py --check <ID>`;
   it exits non-zero if `<ID>` already appears in the implementation plan,
   the phase ownership register, or recent git history.

This rule is procedural memory: it changes agent behavior, so it lives here,
not only in a transcript.

## Verification Ladder

For documentation-contract changes, run:

1. `python3 scripts/check-agent-contracts.py`
2. `python3 scripts/test_check_agent_contracts.py`
3. `cargo fmt --all -- --check`
4. `cargo test -p garnet-cli new_cmd`
5. `cargo test --workspace --no-fail-fast` when Rust behavior changed.

For release-impacting work, follow the latest verification ladder in `F_Project_Management/`.
