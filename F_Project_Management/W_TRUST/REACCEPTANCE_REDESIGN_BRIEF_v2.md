# WV-6 re-acceptance redesign brief v2

**Class:** L1 design record; non-operative; no code or contract change<br>
**Binding base:** `0607f7fe8770491bff3d16261628c27c570baa51`<br>
**Base tree:** `50db668bb831a76d467f727044923953111f2460`<br>
**Date:** 2026-08-30<br>
**Scope:** WV-6 successor rebind, approval ordering, and bounded external-event re-acceptance

This brief answers a narrow economic problem: the fail-closed gates were right
at every Freeze-3 through Freeze-7b boundary, but the current model prices a
record-only squash successor and every bounded external-state reaction as a
new native-Windows ceremony. The redesign below changes the price without
changing what absence, ambiguity, incomplete transport, or unexplained drift
means. Every such condition remains RED.

This document is a proposal only. It does not amend
`WV6_WV7_ACCEPTANCE_CONTRACTS.json`, the WV reporters, the rolling-review
contract, venue law, a ruleset, a workflow, or an acceptance record. Air
review, cross-family review, implementation, activation, merge, and any
acceptance decision are separate acts.

## Changes from v1

The v1 brief at `f8af6cbae14d32df16f81aa19d4630cc66a66c81`
received a cross-family verdict of `CONFIRM-WITH-FINDINGS`. This v2 revises the
unlanded design record; it does not represent review or ratification of v2.
Each requested finding was recomputed against the binding tree before cure:

- **R-FIX-1 — Q-leg durability:** adds one establishment-time-only,
  squash-durable effectiveness transcript with an exact schema, predecessor-
  base producer, explicit trigger routing, a terminal non-recursion rule, and
  a stated residual trust surface.
- **R-FIX-2 — Class A exact pin:** corrects the finding's premise: cargo-deny
  0.19.6 and CI's pinned 0.20.2 both interpret `name@X.Y.Z` as exact, not caret.
  The proposed class nevertheless requires the visibly explicit
  `name:=X.Y.Z` form and a separately machine-evaluated expiry carrier; the
  standing prose-only row remains ineligible.
- **R-FIX-3 — Class C equivalence:** replaces unbounded behavioral prose with
  a predecessor-base catalog of construction proofs, beginning with the exact
  guarded `chunks_exact(2)` to `as_chunks::<2>().0` rewrite shape.
- **R-FIX-4 — R2 machine state:** makes a canonical attempt-1 artifact the
  durable channel, names the additional `actions: read` CI permission, requires
  jobs-API proof of a full attempt-2 replay, and states the irreducible
  readback-to-merge time-of-check/time-of-use window.
- **R-FIX-5 — #528 ordering:** separates strict per-edge fact collection from
  walk-scoped producer qualification after the terminal record and exact-head
  approval exist.
- **FOLD-1 — consumer census:** adds a predecessor-base exhaustive inventory
  of every machinery-consumed record matcher and operation; a prefix alone is
  never authority.
- **FOLD-2 — independent recomputation:** defines “twice” as two independent
  implementations with independently encoded enumeration and digest logic.
- **FOLD-3 — durable approval anchor:** converges with R-FIX-1 by making the
  in-repository effectiveness transcript, rather than permanent forge state,
  the ordinary-verification anchor.
- **FOLD-4 — Class B projection boundary:** records that the live top-level
  `RULESET_KEYS` projection is lossy and that the two existing leaf shapes
  exhaust the ratified variants; the class autonomously admits no future
  shape.
- **FOLD-5 — absent root toolchain file:** states that root
  `rust-toolchain.toml` does not exist, must remain absent, and must not be
  introduced by Class C.
- **FOLD-6 — graph authority:** pins closure traversal to the predecessor's
  base-controlled graph-of-record, never the candidate's graph.
- **FOLD-7 — prose-satisfiable cells:** names four mechanical predicates and
  marks each `OPEN-UNTIL-IMPLEMENTED` until executable evidence exists.
- **FOLD-8 — quiescent-base price:** states that `tree(R) == tree(B)` requires
  serialization or restart after any intervening main movement; R1 saves a
  native replay, not queueing or exact-head review.
- **FOLD-9 — registry-fork denial of service:** makes fork/duplicate RED an
  explicit integrity-over-availability choice and blocks activation until the
  presently absent suffix/registry routing is old-base controlled and green.

## Binding ruling and falsified hypothesis

The task's original `868f77f` prefix is void. It resolves to no object in any
advertised ref and is the U-71 chat-transport exhibit. Cold-clone readback binds
this brief to `0607f7fe8770491bff3d16261628c27c570baa51`, the #528 squash at
`refs/heads/main`, with zero later commits on main. The pending register sweep
independently corroborates that identity from its own fresh clone and the
GitHub PR #528 `mergeCommit` field. U-71's incident record is itself on the
sweep branch, records-class, pending merge.

The expected hypothesis—“the WV reporter at the tip stays accepted with the
pair unchanged”—is **false at the binding base**. A clean invocation of

```text
python3 -I scripts/garnet_wv_acceptance_status.py --wv WV-6 --gate
```

returns `state=partial`, exit 1. All five required WV-6 checks pass; the only
findings are the raw current pair versus the frozen pair:

| binding | SHA-256 | path count |
|---|---|---:|
| native-accepted pair at reviewed head `8426ca761c696c3556190be77cce3e340250b5c7` | `6f2d5f0b2dff0bd800955e0a55b81f6d6f784d71240fe3c906e58a6a3ca8eec6` | 1646 |
| raw observed pair at #528 squash main | `1d404df10aa6c2ebba60fd254e0c50125a7cde8fd3c18bc6241997e6b463c310` | 1649 |

The focused WV test suite exposes the same stale expectation: five of six
tests pass, while its current-repository assertion expects `accepted` and
observes `partial`. This brief records that pre-existing failure and does not
repair it.

The accepted tree is
`601a368414762646ec9e5ad29b53736e20628474`. The advertised #528 PR head is
`d9d6c163e083b667d3e7beaafcc2f3bb5bde061a`; its tree equals the #528 squash
tree byte-for-byte. The accepted head is an ancestor of that PR head, and its
two-commit walk to the PR head touches fourteen paths, all in the current
record classifier. Squashing then removes that ancestry from ordinary main.
The current reporter consequently falls back to hashing the containing tree
and observes the different raw pair. This is U-58's mechanism, amplified by
U-67: record tolerance and digest exclusion are independent axes.

The required deliverable path is itself safe for this records lane.
`F_Project_Management/W_TRUST/**` is already both digest-excluded and an
enumerated post-acceptance record prefix in
`scripts/garnet_content_provenance.py`. This brief therefore keeps the exact
requested path and creates no new `ops/<dir>/` namespace. Its addition does not
move either pair above.

## Evidence and issue boundary

The design was derived from the current WV-6 contract and reporter,
`scripts/garnet_content_provenance.py`, the rolling-review contract and gate,
`.github/workflows/ci.yml`, `ops/gate-topology/RULING-ORDERING.md`, and the
Freeze-3 through Freeze-7b ceremony and structured-review records.

The following identifiers are **allocated on the sweep branch, pending
merge**:

| ID | design relevance | lane |
|---|---|---|
| U-58 | squash-successor gap | L-1 |
| U-59 | approval-versus-CI ordering | L-5 |
| U-60 | re-acceptance economics | L1 |
| U-67 | digest-domain/drift-tolerance orthogonality | L-2 |
| U-68 | cure-and-proof atomicity | L-7 |
| U-39 | base-controlled adapter defect | L3 |

U-39 is evidence for the base-controlled rule in this proposal, not work
absorbed by it. Its cure remains a separate L3 trust-kernel change.
R1–R3 cannot activate through the defective adapter: the base-controlled
verification they consume must be independently green, but its L3 cure may not
ride this records commit.

The ceremony chain establishes these design facts:

| boundary | event and lesson | frozen pair |
|---|---|---|
| Freeze-3 | A squash orphaned the accepted branch lineage. Record-only drift was real, but no sanctioned successor existed. | `32f397… / 1637` |
| Freeze-4 | An internal REST projection omission required a new boundary. A missing projected fact must remain RED. | `056a… / 1640` |
| Freeze-5 | The first forge serialization additions were safe only at exact disabled/empty values; an early boundary was superseded. | `0513ed… / 1643` |
| Freeze-5 redo | The `arrayref@0.3.9` yank exception was exact and lock-preserving, but it was still a bounded policy weakening. | `573523… / 1643` |
| Freeze-6 | Registry-reversal prose and Clippy 1.98 activation shared a boundary. The syntax cure was behavior-neutral; the exact tool pin strengthened reproducibility. | `449ba9… / 1643` |
| Freeze-7 | A second forge leaf strengthened approval posture, but the carry omitted a downstream browser-proof refresh. The boundary was superseded before merge. | `87d520… / 1643` |
| Freeze-7b | Producer-derived closure restored the omitted proof. Three prior ceremony records were record-tolerated but digest-included, moving 1643 to 1646. | `6f2d5f… / 1646` |

The registry reversal did **not** itself strengthen policy: the exact ignore row
remained. It was neutral documentation while the earlier bounded weakening
continued. Strengthening occurs only when that row is removed.

## Model and terms

The redesign separates facts that the current implementation sometimes
collapses:

- **Native root:** the exact WV contract, platform, reviewed head and tree,
  five-check result, manifest, artifact set, proof mirrors, reporter projection,
  structured review, authenticated approval, and native-accepted pair produced
  by the terminal Windows ceremony.
- **Accepted pair:** the pair currently authorized for the product. It begins
  as the native-accepted pair. A record-only successor cannot change it. A
  qualifying R3 content event may establish a later accepted pair only through
  an explicit, reviewed delta certificate.
- **Raw observed pair:** a full recomputation over the containing candidate
  under the unchanged digest definition. It can move when a record-tolerated
  path is digest-included. It is accounting evidence, not native acceptance.
- **Content landing `B`:** the authoritative main first-parent squash whose
  tree equals the final reviewed PR tree carrying the source acceptance.
- **Certificate content head `C`:** the exact source or bounded-event boundary
  before its certificate and structured-review records are appended.
- **Certificate PR tip `Q`:** the exact approved PR head containing the
  certificate and its review record. `Q` is derived from authenticated PR and
  review objects rather than embedded self-referentially in the certificate.
  It is an establishment-time fact, not an ordinary-verification dependency.
- **Certificate landing `M`:** the authoritative main first-parent squash that
  introduces the certificate; `tree(Q)` must equal `tree(M)` exactly.
- **Effectiveness transcript:** the canonical
  `garnet.wv_acceptance_effectiveness/v1` terminal receipt captured
  after `M`. It durably anchors the authenticated `Q` review, `Q`/`M` tree
  bridge, complete pagination, and landing census without requiring a later
  clone to possess `Q` or query the forge.
- **Record class:** the base-controlled set of eligible paths. Prefix
  membership is necessary, never sufficient. Each mutation must also satisfy
  a producer-qualified operation subtype and appear in the predecessor's
  exhaustive machinery-consumer inventory.
- **Producer closure:** the fixed point obtained by starting with every changed
  source/input and following every committed producer, pin, mirror, manifest,
  and proof consumer until `closure_open=[]`. Its graph-of-record is the
  predecessor effective tip's base-controlled inventory, never candidate law.

All classifiers, digest rules, projections, and producer inventories are read
from the authoritative base, independently cross-checked, and hash-bound.
Candidate-controlled classification is never authority.

“Independently recomputed twice” means two separately implemented enumerators
and digestors whose source hashes are bound and whose parsing, ordering,
framing, and classification logic are independently encoded. Neither may
import, invoke, wrap, or consume the other or its output. Two invocations,
processes, wrappers, or hosts running one implementation count once.

The certificate schemas, registries, producer definitions, and every governed
policy surface admitted by an R3 class must themselves become base-controlled
trust-kernel triggers whose exact bytes enter the rolling-review digest. A path
being records-class or digest-excluded does not confer review coverage. No new
certificate can become effective until that fail-closed review routing exists.

## R1 — successor rebind

### Proposed mechanism

Introduce an append-only **acceptance-succession certificate** under the
existing `F_Project_Management/W_TRUST/**` record prefix, with a distinct
suffix and schema rather than overloading `*.review.json`. A sorted registry
selects exactly one linear tip per WV acceptance chain. Existing certificate
and registry bytes are immutable; the registry may only append.
The future trust-kernel classifier must explicitly trigger on that suffix and
registry and include their raw bytes in the reviewed content digest.

Establishment has seven mechanical stages:

1. **Bind the native root and final record tip.** Bind the exact WV id and
   contract blob, native reviewed head/tree/pair, evidence destination,
   manifest blob and SHA-256, exact artifact membership and hashes, proof
   mirrors, reporter blob and its permitted constant projection, structured
   review blob, authenticated PR/reviewer/review objects, and the final
   pre-squash record tip `R`.
2. **Census every edge, not only endpoint differences.** During certificate
   establishment, authenticated transport materializes the complete
   `H..R` PR graph. A base-controlled L-12 producer enumerates every commit
   edge and every merge parent, path, mode, and before/after object; an
   independent endpoint tree diff must agree. A transient non-record touch
   that was later reverted is still disqualifying.
3. **Apply producer-qualified operation law on two clocks.** Edge facts remain
   strictly per-edge: predecessor-base law records every touch when it occurs,
   and a later edge can never erase, reclassify, or excuse an earlier transient
   touch. Qualification is decided once, at establishment, over the complete
   walk after the unique terminal record and its exact-`R` authenticated
   approval exist. An earlier producer output may be qualified by that later
   record only when the record binds the output's after-commit, reviewed head,
   complete producer closure, and a post-review tail to `R` that is itself
   qualified record-only. “Already approved” means approved when establishment
   evaluates the whole walk, not approved before each producer output was
   committed. Ordinary historical records may only be new regular `100644`
   blobs; existing historical records may not be edited, renamed, replaced, or
   deleted. Reporter movement must satisfy
   `r1_reporter_constant_projection_v1`: only the literal values assigned to
   `REVIEWED_HEAD`, `REVIEWED_TREE`, `EXPECTED_PRODUCT_CONTENT_SHA256`, and
   `EXPECTED_PRODUCT_PATH_COUNT` may differ, while the independently parsed
   module, imports, executable tokens, and function bodies are identical. Any
   other reporter movement is content.
4. **Prove the squash bridge.** Require `tree(R) == tree(B)`, require `B` on the
   authoritative upstream main first-parent history, and bind repository and
   PR immutable identities. Establishment may fetch otherwise ephemeral PR
   objects; the certificate then materializes a canonical, self-contained
   graph projection: commit/tree identities, per-edge entries, relevant blob
   hashes and semantic projections, classifier/digest-law hashes, and both
   endpoint inventories. Ordinary later verification must not depend on a
   pull ref or fork branch remaining advertised. Equality is also a
   quiescent-base price: any intervening main byte absent from `R` makes the
   bridge RED. The lane must serialize the merge or restart at the current
   effective tip and obtain fresh exact-head review.
5. **Restate, never launder, the pair.** Recompute the native pair at `H` and
   raw observed pair at `B` with two independent implementations over the
   complete ordered `(path, blob OID)` streams. Record both.
   `native_accepted_pair` remains unchanged and remains the acceptance
   authority; `successor_observed_pair` is explicitly non-authoritative
   accounting. Every differing input entry must be explained by a qualified
   record operation. Only the path count has arithmetic; the SHA-256 values
   require full recomputation.
6. **Land and capture.** The premerge certificate binds `B`, avoiding direct
   self-reference. Authenticated PR/review transport derives the exact approved
   certificate tip `Q`, proves the complete `B..Q` walk is
   certificate/review-only, completely paginates reviews, and directly reads
   the selected decisive review. After squash, the predecessor-base producer
   authenticates the merge identity, derives `M`, requires
   `tree(Q) == tree(M)`, proves `B` and `M` are on authoritative main
   first-parent history, independently censuses the landing edge, and requires
   current `HEAD` to descend through that chain. `M` is
   `ESTABLISHED-BUT-INEFFECTIVE` until stage 7 lands.
7. **Seal one terminal effectiveness transcript.** The base-controlled
   producer `garnet.wv_acceptance_effectiveness.producer/v1` emits exactly one
   canonical JSON object under
   `F_Project_Management/W_TRUST/effectiveness/*.wv-acceptance-effectiveness.json`
   and appends its path and blob SHA-256 once to
   `F_Project_Management/W_TRUST/WV_ACCEPTANCE_EFFECTIVENESS.json`.
   The transcript's exact top-level schema is:

   ```text
   schema, wv, certificate_kind, certificate_path, certificate_blob_sha256,
   predecessor_effective_tip, repository, repository_id,
   pull_request_number, pull_request_id, head_repository, head_repository_id,
   certificate_tip_q, certificate_tree, review_pages, selected_review,
   source_landing_b, certificate_landing_m, landing_parent, merged_tree,
   b_to_q_census, landing_edge_census, transport_receipts,
   record_consumer_inventory_sha256, classifier_sha256, digest_law_sha256,
   producer_identity, producer_sha256, verdict, blocking_findings
   ```

   `selected_review` binds immutable review and reviewer IDs, current login,
   `APPROVED`, `commit_id=Q`, timestamp, and decisive-event order.
   `review_pages` and `transport_receipts` bind bounded endpoint identities,
   status, terminal pagination, and normalized plus raw-body digests. Unknown
   keys, an extra archive member, incomplete pagination, or any conflict is
   RED. The transcript and registry are canonical, append-only regular
   `100644` blobs; the verifier derives their first-parent introduction commit
   instead of storing a self-reference.

The transcript suffix, registry, producer, verifier, and exact schema are
separate base-controlled trust-kernel triggers and rolling-review digest
inputs. The producer accepts a bounded credential only through stdin and runs
while `Q`, the selected review, and `M` remain obtainable; it performs complete
authenticated transport and two independent pair/inventory recomputations.
Ordinary verification thereafter uses the transcript, registry, and main Git
objects and must not require a surviving pull ref, `Q` object, or live review.

This receipt terminates rather than recurses. It is not a second succession or
event certificate, cannot move either pair, and may bind only `Q`/`M` facts
strictly preceding its own introduction. An effectiveness `Q_E`, `M_E`, or
receipt-for-the-receipt is forbidden. The schema, producer, verifier, and
single-purpose introduction law receive review in the separate activation act;
an instance is a deterministic terminal establishment output. Its residual
trust surface is explicit: GitHub's authenticity and completeness at capture,
the bounded transport and credential, predecessor-base producer correctness,
the independent review layer, Jon's merge, Git object integrity, and SHA-256.
A later clone can reverify the committed bytes and main relationships but
cannot prove retrospectively that GitHub's historical response was faithful.
Forge loss after a valid anchor is non-authoritative; omission, contradiction,
or loss before anchoring is RED.

For #528, a future implementation would therefore preserve
`6f2d5f… / 1646` as the accepted pair while recording
`1d404d… / 1649` as the raw observed pair at `0607f7fe…`. It would not claim
that the Windows manifest retroactively attests the latter. The current
reporter has no such certificate reader, so this is a design result, not a
present acceptance claim. The exhibit also fixes the ordering interpretation:
all thirteen producer/record movements are on `8426ca76…5d4e9525`, while
`5d4e9525…d9d6c163` adds only the terminal review record whose
`reviewed_head=5d4e9525…`. It qualifies only under the walk-scoped decision in
stage 3; every underlying touch remains a strict per-edge fact.

### Proposed contract text

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

### Reviewer attestation

The successor reviewer does not re-attest the five Windows executions. The
reviewer attests, in first person and at the exact record head, that:

- the native root and predecessor review/approval were independently
  recomputed and not rewritten;
- complete authenticated transport produced the full PR graph and the
  per-edge L-12 census, with zero transient or endpoint non-record touches;
- every record mutation matches its base-controlled producer subtype;
- `tree(R) == tree(B)`, `B` is exact and first-parent, the certificate has no
  self-reference or remote-ref durability dependency, the exact approval
  binds derived tip `Q`, and no acceptance is claimed until the post-landing
  verifier authenticates `M`, proves `tree(Q) == tree(M)`, and lands the
  terminal effectiveness transcript;
- the native-accepted pair is unchanged, two independent implementations
  recomputed the raw pair, and every raw-pair difference is record-explained;
- manifests, artifacts, proof mirrors, prior certificates, and historical
  records are preserved byte-for-byte; and
- the certificate chain is append-only, linear, unique, and has no blocking
  findings.

Free-form scope prose is not sufficient. Future
`r1_review_scope_exact_v1` evaluates exact structured fields across the
premerge record and effectiveness transcript:

```text
attestation_kind = mechanical_record_succession
native_checks_reexecuted = []
extends_native_coverage = false
reviewed_through = <certificate content head>
coverage_extension = []
source_h = H
source_r = R
source_b = B
approval_head_selector = exact-current-pr-head
landing_tree_requirement = tree(Q)==tree(M)
```

The later transcript supplies the derived `Q` and `M`; neither may be embedded
self-referentially in the premerge record. Unknown or missing fields, a
nonempty coverage extension, or disagreement with the authenticated objects is
RED. This predicate is `OPEN-UNTIL-IMPLEMENTED`; v2 does not claim the present
substring-based `review_scope` check satisfies it.

The reviewer explicitly states: `MECHANICAL RECORD SUCCESSION — no native
check is re-executed, no product pair is promoted, and review coverage is not
extended or backdated beyond the certified projection.`

### Failure modes

Succession is void, not partial credit, on any of the following:

- source reviewed head/tree/pair, manifest, artifact set, proof, reporter,
  review, approval, PR identity, or contract cannot be reproduced exactly;
- missing PR objects during establishment, partial pagination, ambiguous
  parents, a remote identity mismatch, or an unauthenticated squash bridge;
- any per-edge path outside the base record class, any later-reverted
  non-record touch, symlink/non-regular mode, rename ambiguity, deletion, or
  unclassified object; or a walk-scoped decision used to erase an edge fact;
- a candidate-controlled classifier, changed digest exclusion, changed
  record-prefix law, incomplete predecessor consumer inventory, unlisted
  machinery consumer, or movement in its base-controlled adapter;
- a reporter change beyond the exact constant projection, a manifest/proof
  refresh without its complete producer closure, or an old record altered in
  place;
- raw observed pair substituted for the native-accepted pair, a digest
  inferred from count arithmetic, one implementation run twice, endpoint
  inventory mismatch, or an unexplained pair-input delta;
- source PR-tip/`B` or certificate `Q`/`M` tree inequality; unauthenticated
  certificate PR or merge identity; `B` or `M` absent from authoritative main
  first-parent history; a non-record `B..Q` or landing-edge touch; or current
  head outside the certified chain;
- main movement that breaks the quiescent `tree(R) == tree(B)` bridge; a missing,
  duplicate, malformed, noncanonical, forked, or contradictory effectiveness
  transcript/registry entry; incomplete review pagination; unavailable `Q` or
  forge review before capture; a transcript that moves a pair or claims its own
  `Q_E`/`M_E`; or ordinary verification that still requires live forge state;
- duplicate/forked successors, registry rollback, record replacement,
  self-reference, an unavailable required artifact, blocking reviewer finding,
  self-review, stale approval, or head movement.

Any such void routes to a new bounded content event under R3 if and only if an
R3 class matches exactly; otherwise it routes to a full terminal freeze.

## R2 — ordering cure

### Proposed mechanism

Adopt one in-contract **same-run, same-head re-evaluation** as the sole U-59
exception to the U-66 one-firing law. The first CI attempt remains expected
RED until the record-containing head can receive its approval; it earns the
exception only when the gate emits a machine state of
`approval_pending_only` through a durable receipt. Exit status, log text, step
conclusion, and overall run conclusion are not eligibility evidence.

The sole channel is one canonical JSON member named `eligibility.json` in one
uniquely named Actions artifact:

```text
r2-approval-pending-<run_id>-attempt-1
```

A base-controlled classifier emits a receipt on every attempt-1 outcome, and
an `if: always()` step using the already pinned `actions/upload-artifact`
implementation uploads it even when the ordinary gate exits 1. Its exact-key
schema binds `schema`, repository ID, PR ID and number, base ref/SHA, candidate
head/tree, review-record path and raw-byte SHA-256, workflow ID/ref/SHA, run ID
and number, `run_attempt=1`, event, predecessor producer-inventory digest,
`state`, and normalized `finding_codes`. The only eligible tuple is:

```text
state = approval_pending_only
finding_codes = [approval-absent]
```

That tuple means every non-approval predicate was evaluated and passed. Any
other finding, short-circuit, missing or duplicate artifact, second ZIP member,
unsafe path, malformed schema, unavailable download, incomplete artifact
pagination, or API/body digest disagreement is RED.

After the designated reviewer approves the exact unchanged head, an
Actions-write carrier performs exactly one **Re-run all jobs** on that same CI
run. The carrier transports the re-evaluation and gains no review authority.
Attempt 2 treats the replayed event payload as stale venue coordinates only.
It freshly fetches the PR, base, commits, every review page, and the selected
review object, and downloads and verifies the attempt-1 receipt through
authenticated Actions transport.

This option is narrower than adding a new approval-aware workflow trigger: it
adds no new required context, CI secret, or write permission inside CI. It does
broaden the existing job token by exactly `actions: read`, in addition to the
current `contents: read` and `pull-requests: read`, so attempt 2 can authenticate
and download the receipt. The external transport-only carrier separately needs
fine-grained `Actions: write`; that credential must never enter a job. A
check-run channel is not selected because it would require `checks: write` and
is not reliably writable from a fork-origin PR job.

```yaml
permissions:
  actions: read
  contents: read
  pull-requests: read
```

A final authenticated readback immediately before Jon's merge **detects** a
review, head, base, governance, bypass, or context divergence at observation
time; it does not prevent a later divergence. The current ruleset requires zero
approving reviews, and no dismissal-triggered workflow creates an atomic lock.
A dismissal or adverse review after readback but before Jon's merge click is an
irreducible time-of-check/time-of-use race. Any delay or visible UI-state change
requires another readback, but only a separately authorized forge-enforced
approval rule or transactional merge primitive could close the race.

### Proposed contract text

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

The historical #528 run `32543270060` corroborates the all-jobs discriminator,
not the complete future mechanism. Attempt 1 exposed seven job rows; attempt 2
exposed all nine expanded CI jobs and all succeeded. The already-green truth
job received a new ID (`96957159289` to `96959969901`), proving a full rerun.
However, the attempt-2 `triggering_actor.id` and the terminal record's
`reviewer_id` are both `306739987`. #528 therefore fails the proposed
`r2_role_separation_v1` predicate and is not a role-separation precedent.
That predicate is `OPEN-UNTIL-IMPLEMENTED`.

### Venue-law changes required by a later implementation

The following text surfaces must change together, in a separately reviewed
contract act:

1. `C_Language_Specification/GARNET_TRUST_KERNEL_ROLLING_REVIEW.md`, especially
   the approval-selection law around lines 94–105, to define
   the receipt schema, `approval_pending_only`, stale-payload non-authority,
   attempt-2 live transport, role separation, the jobs-API proof, and the
   non-atomic final premerge readback.
2. The same contract's Usage section around lines 185–200, to prescribe the
   exact artifact, reviewer, carrier, rerun, readback, and merge sequence and
   forbid partial/debug/third reruns.
3. `.github/rulesets/README.md` around lines 20–24, to state that “latest
   checks” may be satisfied through this one same-run observation and no other
   rerun.
4. Repo procedural law in `AGENTS.md`, so the U-59 exception and final
   readback survive chat and operator turnover.
5. U-66's one-firing venue law, after its pending sweep allocation lands, by an
   append-only amendment or companion that names U-59 as its only mechanically
   checked exception. The landed sweep record must not be edited in place.
6. `.github/workflows/ci.yml` and its permission-locking tests, to add exactly
   `actions: read`, the always-run pinned artifact upload, and the attempt-2
   download/verification step; no CI write permission is permitted.
7. The rolling reporter, governance run census, focused fixtures, and CI
   operator procedure, to emit normalized finding codes, verify the exact
   artifact, paginate both attempt-specific jobs collections, authenticate
   `triggering_actor`, and fail closed on every mismatch.

### Failure modes

- Attempt 1 has any finding besides missing exact-head approval; the receipt is
  absent, duplicated, expired, malformed, short-circuited, or not the sole safe
  artifact member; or attempt-specific jobs prove anything short of all jobs.
- A body/title/base edit, retarget, close, draft conversion, new base SHA, new
  head/tree/record bytes, workflow movement, or other mutable fact is hidden by
  the replayed payload.
- Review enumeration is partial; direct-object equality fails; the latest
  decisive event is dismissed, changes-requested, or approved at another head;
  approval changes after green; or the final review/governance/bypass/context
  readback is absent or divergent.
- The reviewer, rerun carrier, and author roles are conflated; the carrier's
  Actions permission is treated as approval; `actor.id` is substituted for
  `triggering_actor.id`; or CI receives any scope beyond `actions: read`,
  `contents: read`, and `pull-requests: read`.
- There is a new run ID, close/reopen, dispatch, partial/job/debug rerun, a
  third CI attempt, or any second attempt for another producer.
- A diagnostic string rather than the complete machine predicate grants
  eligibility, or run accumulation is reduced to “latest attempt wins.”
- A readback is described as preventing dismissal or as an atomic merge lock;
  a delay or visible state change occurs without a repeated readback; or the
  residual readback-to-click race is concealed.

Every failure remains RED. It does not authorize an automatic refresh.

## R3 — external-event classes

### Shared bounded re-acceptance mechanism

A non-record content change ends R1 succession. It may avoid a full native
ceremony only by producing one append-only **event re-acceptance certificate**
whose class predicate matches exactly. The certificate binds the predecessor
accepted chain, exact certificate content head `C` and tree, old and new
accepted pairs, complete edge census, external observation, allowed source
delta, producer-closure fixed point, focused impact proof, preservation set,
direction statement, and independent review. Its record tail must be
producer-qualified; authenticated approval must derive exact PR tip `Q`; and
post-squash effectiveness requires an authenticated first-parent landing `M`,
`tree(Q) == tree(M)`, a complete landing-edge census, and the same terminal
`garnet.wv_acceptance_effectiveness/v1` anchor defined by R1 with
`certificate_kind=event`. Any record-inflated raw pair at `M` is restated
separately from the new accepted pair at `C`.

The old native root remains immutable. The new pair is authorized by the
transitive theorem “native root plus every accepted bounded delta,” not by a
claim that old Windows bytes directly attest the new tree. A machine-derived
impact map must address each of the five WV-6 criteria. If a changed input
reaches native behavior and the class's equivalence proof is incomplete, the
bounded path is unavailable and native replay is required.

Weakening is never smuggled through a neutral label. A `BOUNDED WEAKENING`
requires an explicit Jon decision, exact scope, expiry predicate, and proof
that all surrounding fail-closed machinery remains intact. Neutral and
strengthening events still require independent review; direction is not a
waiver of proof.

The event-certificate suffix, registry, class producer, and exact governed
policy surfaces—including the Class A deny-policy row—must be explicit
base-controlled rolling-review triggers whose raw bytes enter the review
digest. Digest exclusion or a certificate path alone is not review authority.

Shared proposed contract text:

```text
EVENT CERTIFICATE LANDING. Every event certificate, registry entry, class
producer, and governed policy byte MUST be in the base-controlled trust-kernel
trigger set and rolling-review digest. Exact-head approval MUST derive the
certificate PR tip Q. The certificate is ineffective until authenticated main
first-parent landing M exists, tree(Q) equals tree(M), the landing edge is
completely censused, exactly one terminal effectiveness transcript anchors the
establishment-time forge facts, and current HEAD descends through M. Ordinary
verification MUST NOT depend on a surviving Q or live forge review. Any record-
tail raw pair movement MUST be restated separately from the accepted pair at
content head C. Missing review routing or anchor, unequal trees, incomplete
landing transport, or ambiguous ancestry is RED.
```

### Class A — registry yank and reversal

**May move:** one canonically explicit exact package ignore row written
`name:=X.Y.Z`; one append to the base-controlled yank-exception event registry;
focused fresh-registry fixtures; the event certificate; and deterministic
pins/mirrors derived from those records. On expiry, the row must be deleted and
one terminal expiry event appended in the same candidate.

**May never move:** `Cargo.lock`; the resolved source/checksum; dependency
requirements or graph; Cargo manifests; vendored/package bytes; global
`yanked = "deny"`; unrelated advisory policy; or a name-only, range, wildcard,
or multi-package exception.

Evidence must reproduce the event with a fresh `CARGO_HOME` and corroborate it
through the registry API and sparse index. An ambient cache is not evidence.
Absence of a RustSec advisory is not safety proof, because an advisory database
may predate the yank.

The finding's claimed parser fact does not reproduce. Both the locally
installed cargo-deny 0.19.6 and CI's pinned cargo-deny 0.20.2 interpret
`name@X.Y.Z` as one exact comparator and do not match a later patch version.
Thus the standing `arrayref@0.3.9` row is semantically exact, not caret. The
future class nevertheless chooses the visibly unambiguous canonical spelling
`arrayref:=0.3.9` and checks the parsed object: exactly one `Exact` comparator,
all major/minor/patch components present, and no wildcard, range, or additional
comparator. Parser/action pin movement or parser disagreement is RED.

Cargo-deny's extended ignore object has only reason prose, so expiry authority
must live in a separate base-controlled,
rolling-review-digested `garnet.wv_registry_yank_exception_events/v1`
registry. Each linear append binds `exception_id`, predecessor event,
`action=activate|expire`, deny-policy path and blob SHA-256, exact package
name/version/source/checksum, resolved depender edge, global `yanked=deny`
posture, and a `valid_while` predicate. `valid_while` is true only when (a) a
fresh API and sparse-index census agree that this exact version is yanked and
(b) the exact lock tuple and resolved depender edge are unchanged. Unknown or
unavailable transport is RED. If either clause becomes false, an active row is
RED and must be removed. Only registry reversal while the lock/source/checksum/
edge remain unchanged is a bounded Class A expiry. Movement in those bound
dependency facts also invalidates the row and requires its deletion, but makes
the candidate out of Class A and routes to native replay unless another already-
ratified class matches. Reason text is explanatory only and is never evaluated
as expiry law.

Proposed class contract:

```text
REGISTRY-YANK ADDITION is BOUNDED WEAKENING. It is eligible only for one
canonical name:=X.Y.Z row whose predecessor-pinned parser yields one complete
Exact comparator, plus one matching machine-consumed activation event. The
event binds one exact locked name/version/source/checksum and resolved depender,
with Cargo.lock, manifests, dependency graph, source bytes, and global
yanked-deny byte-identical. Every other yank remains denied. The valid_while
predicate is evaluated on every run. Jon's explicit weakening approval is
required before Class A may be ratified.

REGISTRY-YANK EXPIRY is STRENGTHENING only when the exact ignore row is deleted
and the matching expiry event is appended in the same candidate, restoring
unconditional global yank denial after a fresh registry reversal while the
lock/source/checksum and resolved edge remain byte-identical. Movement in any
of those dependency facts makes the exception invalid but is not a Class A
expiry event. If the row remains, the candidate is RED; prose about inertness
cannot keep it eligible.
```

Required reviewer statements:

```text
BOUNDED WEAKENING — exactly one locked package/checksum is excepted; every
other yank remains denied; no dependency or lock byte moved; the parsed
requirement is one complete Exact comparator; and the machine valid_while
predicate is presently true.

STRENGTHENING — fresh API/index evidence shows reversal while every bound
dependency fact is unchanged; the exact exception is deleted, its expiry event
is appended, and unconditional global yank denial is restored.
```

The standing `arrayref@0.3.9` row does not satisfy Class A as-is. Although it is
semantically exact under the pinned parser, it has no machine-consumed event
entry and its own prose records an observed reversal, making the proposed
`valid_while` predicate false. Class A is therefore **NOT-RATIFIABLE** until a
separate authorized act removes that row, or Jon explicitly rules a replacement
predicate. Decision Point 7 is an entry precondition, not later cleanup.

Class failure modes include parser/action pin movement; comparator count,
operator, or complete-triplet mismatch; deny-row/registry digest mismatch;
duplicate, missing, forked, or candidate-defined events; stale cache;
API/index disagreement or unavailability; false `valid_while` with the row
retained; source/checksum/lock/edge movement; advisory absence used as proof;
or any unrelated policy movement.

### Class B — forge-API ruleset serialization drift

**May move:** only the exact typed leaf set comprising the complete observed
serialization divergence at its exact already-checked ruleset path; canonical
31/32 document pins derived from that leaf set; value-specific positive and
negative fixtures; the event certificate; and no wildcard projection surface.
For the first historical evolution, the set is exactly
`dismissal_restriction={enabled:false, allowed_actors:[]}` and
`required_reviewers=[]`. For the second, it is exactly
`require_extra_approval_for_unattributed_changes=true`.
Observation alone never admits a third shape: a new typed variant and its
direction require a prior Jon ruling or a separately governed contract act.

**May never move:** `_strict_equal`; the top-level `RULESET_KEYS` and transport
projection sets; bypass actors or bypass result;
target/ref/enforcement/conditions; rule identity/order;
required-context identity, row order, count, producer integration, or semantic
pins except their mechanically derived canonical document hashes; transport,
pagination, permissions, or fail-closed direction.

**Live-projection limit.** The current comparison is not whole-object equality.
Transport projects the live ruleset and `_policy_equality_problems` then builds
only `{key: live_ruleset.get(key) for key in RULESET_KEYS}` before
`_strict_equal` (`scripts/garnet_github_governance_gate.py:602`). The exact
top-level-key check at line 560 in `_checked_contract_problems` applies only to
the checked-in document. An extra live top-level server field is discarded
before equality; it is invisible, not eligible Class B drift. Nested values
retained under `rules` remain strict.

Both ratified leaf variants are already absorbed in the checked contract: the
exact disabled/empty `dismissal_restriction` plus `required_reviewers` shape,
and `require_extra_approval_for_unattributed_changes=true`. As presently
ratified, Class B admits **zero autonomous future events**. Any future field,
path, type, value, or top-level server fact is RED until a prior Jon ruling and
separate contract act expands the predecessor-owned live projection, adds one
typed variant, refreshes exact digests, and adds positive and fail-closed
negative fixtures. Live observation cannot bootstrap that authority.

Proposed class contract:

```text
FORGE SERIALIZATION DRIFT is eligible only when complete authenticated
projection yields exactly the certificate's finite typed leaf set and zero
other divergence; each leaf occupies its exact existing rule path; transport
is complete; _strict_equal and all executable comparison logic are
byte-identical; bypass is []/never; required-context rows remain exact; and
fixtures fail closed for absence, weaker values, wrong types, extra values,
and path displacement. The leaf shape MUST be an already-ratified class
variant; live observation cannot widen the class.

Disabled/empty dismissal_restriction and required_reviewers values are
EFFECTIVE-POSTURE NEUTRAL only at those exact values. A true
require_extra_approval_for_unattributed_changes leaf is STRENGTHENING. An
absent or false value is RED. No updated_at value proves why or when a server
field appeared.
```

Required reviewer statement:

```text
The complete projected diff is exactly the enumerated typed leaf set;
_strict_equal is byte-identical; bypass is []/never; all required-context rows
remain exact; absent or weaker values fail closed. Observed posture is
[NEUTRAL|STRENGTHENING]; no server-side cause is claimed as proven.
```

Class failure modes include incomplete transport, an additional divergent
leaf, treating an unprojected live top-level field as harmless or class-
eligible, claiming the current projector rejects unknown live top-level keys,
ambiguous path/rule identity, reorder, bypass or required-context movement,
false/absent strengthening value, a candidate-controlled projection, or
inference from `updated_at` alone.

### Class C — toolchain lint activation

**May move directly:** one exact CI compiler/Clippy pin, one lint-triggered
source site matching one predecessor-catalog rewrite, and the closest
procedural contract required by repository law. Every other moved pin,
manifest, proof, or evidence output must be discovered as a descendant in the
predecessor's machine-derived producer/pin closure; it is not admitted by a
hand-written path list.

**May never move:** root `rust-toolchain.toml` and root `rust-toolchain`, which
do not exist and must remain absent at predecessor and candidate; MSRV 1.95;
Cargo manifests, lock, dependencies, public API, or serialized behavior;
`allow`, `expect`, or Clippy suppression; lint flags; workflow job/context
names, permissions, `needs`, conditions, or required-context count; external
action `uses:` pins; more than the single declared lint cure; or an unrelated
cleanup. Introducing either root toolchain file is out of class.

For predecessor acceptance `P`, closure graph `G(P)` is the exact sorted
producer/consumer inventory read and hash-bound from `P`. The class producer
runs `G(P)` over the candidate delta; it never loads candidate selectors,
edges, glob rules, or consumers to decide the candidate's closure. A graph
change is itself an unratified policy change and cannot authorize the same
event. The historical A3 chain demonstrates the rule:
`episodic.rs -> Wasm source census/provenance.json -> browser runtime-input
aggregate -> W_PLAY_BROWSER_PROOF.json`. Identical JS/Wasm or screenshot bytes
alone did not close that chain.

`CLASS_C_CATALOG_MATCH_V1` replaces an undecidable general equivalence claim.
The candidate must name exactly one rewrite ID from the predecessor's
base-controlled catalog, bind the catalog and verifier blob hashes, match the
entry's exact typed old shape, and equal its exact emitted new shape after
masking only the declared source span. A catalog miss, verifier ambiguity,
candidate catalog movement, second source site, or open producer descendant
routes to native replay.

The first entry is `C-RW-0001/rust-u8-chunks2-guarded-v1`, derived from A3. Let
`S` be one unchanged `&str` receiver (`S=hex` in A3). The entry admits only:

- the same `S.as_bytes()` expression producing an immutable `&[u8]` receiver;
- constant `N=2` and an unchanged dominating
  `if !S.len().is_multiple_of(2) { return Err(the unchanged value) }`;
- replacement of `for p in receiver.chunks_exact(2)` by
  `for p in receiver.as_chunks::<2>().0`;
- an otherwise byte-identical loop body in which `p` is used only at indices
  `0` and `1`; and
- no remainder use, iterator-specific API, type-sensitive dispatch, unsafe
  code, guard, error, allocation, public API, or serialized-output movement.

Its proof is by construction, not testing over an unbounded domain: odd
lengths take the identical pre-loop error, empty input yields zero chunks, and
every even length yields the same ordered disjoint byte pairs; nibble evaluation
and first-error position are therefore identical. `as_chunks` is available at
the workspace MSRV (stable since Rust 1.88; the MSRV is 1.95). The catalog
predicate is `OPEN-UNTIL-IMPLEMENTED`.

Proposed class contract:

```text
TOOLCHAIN LINT ACTIVATION is eligible only for one exact compiler/Clippy pin
and one exact CLASS_C_CATALOG_MATCH_V1 rewrite. The predecessor catalog and
verifier MUST match byte-for-byte, the typed preconditions and emitted source
MUST match exactly, and the candidate MUST pass at workspace MSRV and the pinned
lint compiler. The compiler pin is
REPRODUCIBILITY-STRENGTHENING; the source rewrite is BEHAVIOR-NEUTRAL. These
dimensions MUST be stated separately.

Starting from the predecessor's base-controlled graph G(P), the producer census
MUST compute a fixed-point closure from every changed input through every
semantic pin, provenance file, manifest, mirror, runtime-input aggregate, and
proof without consulting candidate graph law.
Every dependent output MUST be regenerated or independently verified at the
exact candidate. Acceptance between cure and closure is void; a proof captured
at an older tree cannot close the current candidate. closure_open MUST equal
[]. A catalog miss or any obligation outside one exact entry requires native
replay.
```

Required reviewer statement:

```text
BEHAVIOR-NEUTRAL source rewrite; REPRODUCIBILITY-STRENGTHENING exact compiler
pin. Rewrite C-RW-0001 matches every predecessor-catalog precondition and its
construction proof; no general semantic-equivalence claim is made. The
predecessor input -> producer -> output graph, tool versions, lock equality,
exact output delta, reproducibility checks, and closure_open=[] have been
independently verified.
```

Class failure modes include a catalog or verifier hash mismatch; typed-shape,
guard, body, receiver, index, or emitter mismatch; candidate-defined graph law;
a second source span; any suppression or API/behavior change; failure under
either compiler; MSRV/lock/dependency movement; introduction of a root toolchain
file; workflow policy movement beyond the exact pin; unavailable or
non-reproducible producer; self-reference; stale runtime-input dictionary;
proof from an older tree; or any open/stale dependent. Every miss routes to
native replay, not a reviewer assertion of equivalence.

### Shared R3 failure modes

An event is out of class if two classes are needed to explain it, the external
observation is ambiguous, the full edge delta or producer closure is
incomplete, the impact map does not discharge all five WV criteria, the
direction is mislabeled, preservation is not exact, or the reviewer finds an
unbounded semantic effect. A missing, forked, or conflicting terminal
effectiveness anchor also voids the event. Unknown is RED. Combining independent
events to save a boundary is not mechanical economy; it is a new review scope.

## R4 — no weakening

### Proposed mechanism: conservation matrix

| property currently proved | R1 record succession | R2 ordering cure | R3 bounded events |
|---|---|---|---|
| **Bypass emptiness** | Not mutable by any record subtype; base law is hash-bound. | Same ruleset and same head; the mandatory live final governance readback repeats `[]/never`. | Forge class requires `[]/never`; other classes cannot touch bypass. |
| **Head and tree binding** | Binds native `H/T`, final source PR tip `R`, source landing `B`, exact approved certificate tip `Q`, equal-tree landing `M`, terminal effectiveness transcript, and current descendant chain. | Both attempts, artifact, attempt-specific job census, and final readback bind the identical head/tree/record/base. | Each event binds content head `C`, approved certificate tip `Q`, equal-tree first-parent landing `M`, terminal effectiveness transcript, and current exact pair; no backdating. |
| **Transport completeness** | Establishment authenticates the complete PR/review graph and anchors it in-repo; later forge availability is not authority. | Attempt 2 freshly paginates artifact, jobs, PR, commits, reviews, and the selected review object. | Each external source has a bounded complete transport; disagreement or partial data is RED. |
| **Drift census** | L-12 enumerates every edge and parent plus endpoint diff under the predecessor's exhaustive consumer inventory; walk-scoped qualification cannot hide an edge. | Candidate bytes cannot move between attempts. Attempt-specific job and head-scoped run censuses prevent venue laundering. | Full edge census plus predecessor class predicate and producer fixed point; combining classes is forbidden. |
| **Preservation** | Native manifests, artifacts, proofs, reviews, and old certificates remain byte-exact. | No repository bytes change; the approval is external state bound to the same record. | Predecessor root and every superseded boundary remain append-only; new pair is linked, never substituted backward. |
| **Append-only records** | Distinct certificate and terminal-effectiveness schemas, immutable registry entries, unique linear tip, and derived introduction commits. | The existing record is not edited after approval; failure requires a new successor. | One new immutable event certificate plus its terminal receipt; prior event, receipt, and evidence bytes cannot be resealed. |
| **`_strict_equal` and fail-closed direction** | `r1_strict_equal_blob_identity_v1` forbids comparator movement; `r1_reporter_constant_projection_v1` admits only four derived literals. Both remain open below. | Only observation time and the authenticated artifact channel change; equality predicates do not. | Forge equality is byte-identical within the acknowledged projection; registry weakening is explicit and exact; toolchain suppression is forbidden. |
| **Approval and reviewer independence** | `r1_review_scope_exact_v1` binds non-extension structurally and the effectiveness transcript anchors the selected approval. | `r2_role_separation_v1` requires authenticated reviewer/carrier/commit-principal disjointness; latest decisive review is re-read before merge. | Direction and delta proof receive independent review, its selected approval is durably anchored, and weakening additionally needs Jon. |
| **Cure/proof atomicity** | Per-edge collection plus walk-scoped qualification admits a terminal refresh only with its exact full closure. | Attempt 1 cannot emit the sole eligible receipt with an open closure finding. | `closure_open=[]` is a precondition, strengthening U-68 from carry prose to a machine state. |

The changes are therefore economic, not evidentiary: R1 replaces a repeated
native execution with an exact projection theorem; R2 adds a second observation
of the same immutable candidate; R3 replaces a whole-platform replay only when
a closed, class-specific noninterference or direction proof explains the
entire content delta.

R1's saving is narrower than “no new ceremony.” `tree(R)==tree(B)` prices a
quiescent base: an intervening main change forces serialization or a restart
with fresh exact-head review, even if that change is records-class. R1 removes
the repeated native-Windows execution only; it does not remove queueing,
integration, CI, independent review, closeout, or Jon's merge boundary.

Four conservation predicates are explicit activation blockers rather than
prose claims:

| predicate | mechanical test | v2 status |
|---|---|---|
| `r1_review_scope_exact_v1` | Parse the exact structured non-extension fields listed in R1, then require authenticated `H/R/B/Q/M` agreement and `coverage_extension=[]`. | `OPEN-UNTIL-IMPLEMENTED` |
| `r2_role_separation_v1` | Require positive immutable reviewer and attempt-2 `triggering_actor` IDs to be pairwise disjoint from one another and every authenticated PR commit author/committer ID. | `OPEN-UNTIL-IMPLEMENTED` |
| `r1_reporter_constant_projection_v1` | In `scripts/smoke_garnet_minimum_shelf.py`, replace only the literal spans for `REVIEWED_HEAD`, `REVIEWED_TREE`, `EXPECTED_PRODUCT_CONTENT_SHA256`, and `EXPECTED_PRODUCT_PATH_COUNT` with typed sentinels; all remaining raw bytes must match and each replacement must be independently derived from bound Git/pair evidence. | `OPEN-UNTIL-IMPLEMENTED` |
| `r1_strict_equal_blob_identity_v1` | Require predecessor/candidate blob-OID equality for `scripts/garnet_github_governance_gate.py` and every predecessor-inventoried comparator consumer; relocation, wrapper, or import substitution is RED. The four reporter literals are not a `_strict_equal` carve-out. | `OPEN-UNTIL-IMPLEMENTED` |

The current substring review-scope check, existing reviewer/author test, generic
record classification of the reporter, and present comparator coverage do not
satisfy these future predicates. R1–R3 cannot activate while any row remains
open.

### Proposed contract text

```text
CONSERVATION RULE.

No succession or bounded re-acceptance MAY delete, relax, bypass, infer, or
silently substitute any fail-closed meta-property enumerated here. Bypass
emptiness, exact head/tree/pair binding, complete authenticated transport,
per-edge drift census, predecessor preservation, append-only linear records,
strict equality, independent decisive review, and producer-closure atomicity
MUST each be re-proved at the new observation boundary.

Every named conservation predicate MUST have executable predecessor-controlled
evidence. OPEN-UNTIL-IMPLEMENTED is ineligible, not an advisory pass.

An R3 certificate MAY alter an object-level policy only when an already-
ratified class names the exact movement and direction. The sole weakening in
this proposal is the Jon-approved, exact-name/version/checksum registry
exception with expiry; it MUST NOT relax any fail-closed meta-property above.

A mechanism that cannot name and recompute one conserved predicate is
ineligible, regardless of event direction or historical precedent. Unknown,
partial, unavailable, conflicting, or not-applicable-by-assertion is RED.
```

### Failure modes

R4 fails if any matrix cell is satisfied only by prose, endpoint-only diff,
candidate-controlled code, cached external state, a “latest wins” selector,
unreviewed widening, a deleted predecessor, or an equivalence claim over an
unbounded input domain. It also fails if a registry exception is labeled
neutral, a forge strengthening accepts false/absence, a toolchain cure uses a
suppression, or a raw record-inflated pair is promoted as native. R4 failure
disables R1–R3 and routes to full freeze; there is no waiver inside the
reporter.

## R5 — acceptance-succession law

### Proposed mechanism

Represent WV acceptance as an append-only chain rooted in native evidence:

```text
native root -> zero or more record-succession certificates
            -> one terminal effectiveness transcript per landed certificate
            -> zero or more bounded-event certificates
            -> current exact observation
```

A record certificate preserves the accepted pair and only changes where the
accepted theorem is observed. A bounded-event certificate changes the accepted
pair only through a complete R3 delta theorem. A new native terminal freeze
creates a new native root and supersedes the prior chain with preservation.
The reporter selects exactly one linear, effective tip; a fork, gap, duplicate,
or unverifiable predecessor is RED.

That unique-tip law intentionally trades availability for integrity. An
unauthorized fork, duplicate, or malformed registry append can force RED and
deny availability, but cannot produce false acceptance. Routing makes that
denial visible and merge-blocking; it cannot stop an authorized writer or merge
authority from deliberately denying service. Today the proposed suffix and
registry routes do not exist, so the mechanism is non-operative and cannot
claim prevention.

### Proposed contract text

```text
ACCEPTANCE SUCCESSION LAW.

1. BINDING. A native WV acceptance binds one WV contract and schema, platform
   and scope, exact reviewed head and tree, native-accepted content pair,
   evidence destination, canonical manifest, exact artifact membership and
   hashes, proof mirrors, reporter projection, all required-check results,
   structured review, independent reviewer identity, decisive exact-head
   approval, and preservation state. Acceptance is no broader than those facts.

2. SQUASH. A squash does not preserve branch ancestry and does not implicitly
   carry acceptance. Acceptance survives a squash only through an effective
   R1 certificate proving the complete qualified record projection from the
   accepted head through the final PR tip, exact PR-tip/squash-tree equality,
   authoritative source landing B, exact approved certificate tip Q,
   authenticated first-parent certificate landing M with tree(Q) equal to
   tree(M), complete landing-edge census, unchanged native-accepted pair, two
   independently implemented raw-pair recomputations, and one terminal
   effectiveness transcript anchoring the establishment-time forge facts. The
   source review is neither extended nor backdated. Ordinary verification MUST
   NOT require Q, a pull ref, or live forge review state after that anchor.

3. RECORD SUCCESSOR. Above an effective certificate, later commits preserve
   acceptance only after loading the predecessor effective tip's exhaustive
   record_consumer_inventory. Each sorted entry binds consumer ID, producer
   blob hash, exact path/prefix/suffix/registry matcher, schema, semantic role,
   and permitted operation for every gate or reporter that globs, suffix-
   matches, registry-loads, or parses record bytes, including landed markers
   and the succession, event, and effectiveness registries. Two independent
   inventory enumerators MUST agree. Candidate inventory is never authority.
   Every commit edge MUST be producer-censused, and every operation and
   machinery-consumed record path MUST match one inventory entry and its bound
   predicate. Prefix membership is necessary and never sufficient. An unlisted
   consumer, matcher drift, unmatched claimed record, or operation outside its
   predicate is content change. The accepted pair remains unchanged; any raw-
   pair movement is restated separately and fully explained. Existing records,
   evidence, and chain links remain immutable. A record-path logic or inventory
   change is content, not a record successor.

4. CONTENT CHANGE. The first non-record byte or non-qualified operation ends
   record succession immediately. The prior acceptance becomes
   SUPERSEDED-WITH-PRESERVATION; it is never rewritten or erased. An exact R3
   class MAY establish a new accepted pair through a reviewed, closed delta
   certificate only after its exact approved tip Q and equal-tree first-parent
   landing M are authenticated. If no one class matches, any impact remains
   open, any fail-closed meta-property moves, or weakening lacks Jon's explicit
   ruling, the candidate requires a full native terminal freeze.

5. OBSERVATION. Every current acceptance result binds the native root, complete
   certificate and effectiveness chain, exact current head/tree, current
   accepted pair, current raw observed pair, effective main landing, durably
   anchored decisive review, and fresh external observations still required by
   its classes. Missing, stale, ambiguous, conflicting, forked, partial, or
   unavailable evidence is RED.
```

### Failure modes

- Treating squash-tree equality alone as review or acceptance; requiring an
  ephemeral branch ref forever; or assuming ancestry that squash discarded.
- Omitting the certificate PR tip, authenticated merge identity,
  `tree(Q) == tree(M)`, first-parent placement, landing-edge census, or terminal
  effectiveness transcript and thereby recreating U-58 for the certificate
  itself.
- Treating a record-prefix match as semantic harmlessness; omitting a glob,
  suffix reader, parser, or registry from the predecessor inventory; consulting
  candidate graph law; ignoring transient touches; or allowing reporter logic
  to certify itself.
- Mutating the accepted pair on record succession, claiming native evidence
  for a raw successor pair, or changing an R3 pair without a closed impact
  theorem.
- Backdating review over later bytes, selecting among multiple chain tips,
  skipping a predecessor, editing a historical certificate, or resealing old
  evidence.
- Concealing that a registry fork is a fail-closed availability lever, or
  activating before every suffix/registry route and fork fixture is enforced by
  old-base law.
- Continuing after a content change because the change is small, external,
  strengthening, already green elsewhere, or expensive to freeze.

Any failure kills the proposed succession at that edge. It does not kill the
historical acceptance record, which remains preserved as superseded evidence.

## Activation boundary

The current #528 state is the migration exhibit, not an activated instance of
this proposal. A later implementation would need, in one separately governed
act, canonical certificate/effectiveness schemas and registries,
base-controlled consumer inventory and producers, reporter support, adversarial
fixtures, venue-law amendments, and a reviewed initial certificate for the #528
topology. Before the first entry, old-base law must route every proposed suffix
and registry as a required trust trigger, census the activation base for zero
unregistered or ambiguous objects, and pass fork/duplicate negative fixtures.
The trigger/schema/producer activation and canonical empty/genesis registry
state must be atomic. That act necessarily changes
contracts and code and therefore cannot bootstrap itself through R1. Jon must
rule its entry ceremony. Until that happens, the current reporter's PARTIAL
result is correct and must not be rewritten as accepted by this brief.

## DECISION POINTS

### Choices reserved to Jon

1. Whether to adopt the two-pair model: immutable `native_accepted_pair` plus
   non-authoritative `successor_observed_pair`, with R3 alone permitted to
   advance the current accepted pair.
2. Whether the succession certificate, distinct suffix, append-only registry,
   unique-tip rule, derived `B`/`Q`/`M` landing model, generic terminal
   effectiveness transcript, and explicit rolling-review trigger/digest
   coverage become contract law, including the stated capture-time residual
   trust and no-receipt-for-the-receipt termination rule.
3. Whether #528 receives a one-time migration certificate and what entry
   ceremony authorizes the code/contract change that cannot self-bootstrap;
   `tree(R)==tree(B)` means that ceremony must serialize against main or restart
   on any intervening movement.
4. Whether U-59's same-run **Re-run all jobs** cure is adopted instead of a new
   approval-aware trigger; whether CI may add exactly `actions: read` for the
   canonical attempt-1 artifact; and which separate Actions-write carrier may
   perform the transport-only act while satisfying
   `r2_role_separation_v1`.
5. Whether the irreducible readback-to-merge-click race is acceptable for this
   bounded cure, or whether a separately authorized forge-enforced approval
   rule or transactional merge primitive is required first. The final readback
   may be assigned to the reporter, a Jon-only ceremony, or both, but may not be
   described as prevention or omitted.
6. Whether the three R3 event families and their exact impact-proof matrices are
   accepted, and when any native-impact reachability still mandates Windows
   replay.
7. Whether Class A may ever be ratified as `BOUNDED WEAKENING`, including the
   canonical `name:=X.Y.Z` predicate and machine expiry-event registry. This is
   an explicit ratification precondition: the standing row must first be
   removed in a separate authorized act, or Jon must rule a replacement
   `valid_while` predicate. Registry reversal with the row retained is not
   ratifiable closure.
8. Whether Class B remains a closed set with zero autonomous future events, and
   what separate contract ceremony is required before expanding the lossy live
   top-level projection or admitting another typed leaf.
9. Whether to adopt the predecessor-base Class C rewrite catalog and
   `C-RW-0001` as its first construction-proof entry; every catalog miss routes
   to native replay.
10. Who is the designated cross-family reviewer for succession and each event
   class, and whether a second reviewer is required for a weakening.
11. Whether the exhaustive predecessor record-consumer inventory and all four
    named `OPEN-UNTIL-IMPLEMENTED` conservation predicates are the required
    activation shape, or whether an equally bounded mechanical replacement is
    commissioned. Prose is not an alternative.
12. Whether the integrity-over-availability unique-tip law is accepted, what
    recovery authority exists after an authorized or accidentally landed
    registry fork, and whether the atomic empty/genesis activation census and
    fork fixtures are sufficient. This brief defines no recovery waiver.
13. When the pending sweep allocations become authoritative after merge. U-39
    remains an independent L3 cure and may not ride this L1 records lane.
14. Whether and when to authorize later contract, reporter, workflow, ruleset,
    or procedural-law changes. This brief authorizes none of them.
15. Merge, acceptance, launch state, FIRE/HOLD, and any release action remain
    Jon-only and outside this design record.

### Questions the cross-family reviewer must answer

1. Does R1 bind the complete native root, `H/R/B/Q/M`, exact source and landing
   trees, and current head without self-reference or backdating, while pricing
   the quiescent-base restriction plainly?
2. Does the effectiveness schema capture complete authenticated review and
   pagination evidence at establishment, route every byte through predecessor
   trust law, terminate without `Q_E/M_E`, and leave only the stated residual
   capture-time trust rather than a permanent forge dependency?
3. Does the L-12 producer collect every edge and merge parent strictly when it
   occurs, then apply qualification only once over the complete walk? Under
   that rule, is #528's producer edge followed by its sole review-record edge a
   valid exhibit without allowing a later record to erase a transient touch?
4. Is the native-accepted pair unchanged; do two independent implementations,
   rather than one implementation twice, recompute the observed pair; and is
   every differing digest input exhausted by a qualified operation?
5. Does the predecessor `record_consumer_inventory` exhaust every glob reader,
   suffix matcher, record parser, landed-marker registry, and proposed registry,
   with prefix membership never sufficient and candidate graph law never
   authoritative?
6. Does R2 derive eligibility only from the canonical attempt-1 artifact under
   exactly `actions: read`, ignore replayed mutable payload fields, paginate
   both attempt-specific jobs APIs, prove every expanded job re-executed, and
   authenticate a carrier distinct from reviewer and commit principals?
7. Does the final R2 readback detect all named divergences without claiming an
   atomic lock, and is the residual readback-to-click race surfaced for Jon's
   explicit disposition?
8. Does Class A parse one complete Exact comparator, bind one locked checksum
   and depender, enforce machine `valid_while`, preserve global denial, and
   refuse ratification while the standing row lacks a carrier and records a
   reversal?
9. Does Class B accurately acknowledge the lossy live top-level projection,
   preserve strict nested equality and `[]/never`, and admit zero future shapes
   without a prior contract act?
10. Does Class C match exactly one predecessor-catalog construction proof,
    preserve the absence of both root toolchain files, classify the compiler
    pin separately as reproducibility-strengthening, run predecessor `G(P)`,
    and route every catalog miss or `closure_open` entry to native replay?
11. Do all four named R4 predicates have executable predecessor-controlled
    evidence, with no `OPEN-UNTIL-IMPLEMENTED` row mislabeled as conserved?
12. Are all native manifests, artifact members, proof mirrors, reviews,
    preserved bundles, certificates, effectiveness receipts, and historical
    records byte-exact and append-only?
13. Does the unique-tip/registry law prevent false acceptance while plainly
    exposing the denial-of-service lever, and is suffix/registry routing live
    under old-base law before any first entry?
14. Does R5 preserve only what is proved across squash and record succession,
    end succession at the first content change, and retain every superseded
    boundary with preservation?
15. Does the final report state the current truth without repair: WV-6 remains
    PARTIAL at `0607f7fe…`, five of five checks pass, and the pair mismatch is
    unresolved until a separately authorized implementation and entry act?
