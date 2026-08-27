# WV-6 re-acceptance redesign brief v1

**Class:** L1 design record; non-operative; no code or contract change<br>
**Binding base:** `0607f7fe8770491bff3d16261628c27c570baa51`<br>
**Base tree:** `50db668bb831a76d467f727044923953111f2460`<br>
**Date:** 2026-08-27<br>
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
- **Certificate landing `M`:** the authoritative main first-parent squash that
  introduces the certificate; `tree(Q)` must equal `tree(M)` exactly.
- **Record class:** the base-controlled set of eligible paths. Prefix
  membership is necessary, never sufficient. Each mutation must also satisfy
  a producer-qualified operation subtype.
- **Producer closure:** the fixed point obtained by starting with every changed
  source/input and following every committed producer, pin, mirror, manifest,
  and proof consumer until `closure_open=[]`.

All classifiers, digest rules, projections, and producer inventories are read
from the authoritative base, independently cross-checked, and hash-bound.
Candidate-controlled classification is never authority.

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

Establishment has six mechanical stages:

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
3. **Apply producer-qualified operation law.** Ordinary historical records may
   only be new regular `100644` blobs. Existing historical records may not be
   edited, renamed, replaced, or deleted. Terminal manifest, proof, status, and
   reporter movements are admitted only when the already-approved terminal
   record binds their exact producer closure. A reporter movement must be an
   exact allowed-constant projection with every executable token, import, and
   function body otherwise byte-identical. Any reporter logic change is
   content, irrespective of its path.
4. **Prove the squash bridge.** Require `tree(R) == tree(B)`, require `B` on the
   authoritative upstream main first-parent history, and bind repository and
   PR immutable identities. Establishment may fetch otherwise ephemeral PR
   objects; the certificate then materializes a canonical, self-contained
   graph projection: commit/tree identities, per-edge entries, relevant blob
   hashes and semantic projections, classifier/digest-law hashes, and both
   endpoint inventories. Ordinary later verification must not depend on a
   pull ref or fork branch remaining advertised.
5. **Restate, never launder, the pair.** Recompute the native pair at `H` and
   raw observed pair at `B` twice from the complete ordered `(path, blob OID)`
   streams. Record both. `native_accepted_pair` remains unchanged and remains
   the acceptance authority; `successor_observed_pair` is explicitly
   non-authoritative accounting. Every differing input entry must be explained
   by a qualified record operation. Only the path count has arithmetic; the
   SHA-256 values require full recomputation.
6. **Land and observe.** The premerge certificate binds `B`, avoiding direct
   self-reference. Authenticated PR/review transport derives the exact approved
   certificate tip `Q` and proves the complete `B..Q` walk is
   certificate/review-only. After squash, the reporter authenticates the merge
   identity, derives `M`, requires `tree(Q) == tree(M)`, and proves `B` and `M`
   are on authoritative main first-parent history. It also independently
   censuses the landing edge and requires current `HEAD` to descend through
   that chain. The certificate becomes effective only then.

For #528, a future implementation would therefore preserve
`6f2d5f… / 1646` as the accepted pair while recording
`1d404d… / 1649` as the raw observed pair at `0607f7fe…`. It would not claim
that the Windows manifest retroactively attests the latter. The current
reporter has no such certificate reader, so this is a design result, not a
present acceptance claim.

### Proposed contract text

```text
RECORD-ONLY ACCEPTANCE SUCCESSION.

An accepted WV boundary MAY succeed across a squash without repeating native
platform execution only through one effective garnet.wv_acceptance_succession/v1
certificate in the append-only succession registry.

The certificate MUST bind the native root, the final reviewed and approved PR
record tip R, the authoritative content landing B, the complete producer-
censused H..R graph, the exact R-tree/B-tree equality, the base-controlled
record classifier and digest definition, preservation hashes, and a linear
predecessor certificate.

The certificate suffix, registry, classifier, and producer definitions MUST
be explicit base-controlled trust-kernel triggers. Their exact raw bytes MUST
enter the rolling-review content digest and receive decisive exact-head
approval. Record classification or digest exclusion alone is not review.

Record-path membership is necessary but not sufficient. Every edge operation
MUST satisfy a producer-qualified record subtype. Any transient or endpoint
non-record touch, reporter executable change, unexplained producer output,
historical-record mutation, or incomplete graph is content drift and is RED.

The certificate MUST restate both native_accepted_pair and
successor_observed_pair. Native_accepted_pair MUST equal the predecessor's
accepted pair exactly. Successor_observed_pair MUST be independently
recomputed at B and MUST NOT be represented as native evidence. Every pair-
input difference MUST be exhausted by qualified record operations under the
unchanged digest law.

The succession is effective only when authenticated transport derives the
exact decisively approved certificate PR tip Q and merge identity; the complete
B..Q walk is qualified record-only; the certificate-containing landing M is on
authoritative main first-parent history after B; tree(Q) equals tree(M); the
landing edge is independently censused; and current HEAD descends through the
unique linear succession tip. Ambiguity or absence is RED.
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
  verifier authenticates `M` and proves `tree(Q) == tree(M)`;
- the native-accepted pair is unchanged, the raw pair was recomputed twice,
  and every raw-pair difference is record-explained;
- manifests, artifacts, proof mirrors, prior certificates, and historical
  records are preserved byte-for-byte; and
- the certificate chain is append-only, linear, unique, and has no blocking
  findings.

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
  unclassified object;
- a candidate-controlled classifier, changed digest exclusion, changed
  record-prefix law, or movement in its base-controlled adapter;
- a reporter change beyond the exact constant projection, a manifest/proof
  refresh without its complete producer closure, or an old record altered in
  place;
- raw observed pair substituted for the native-accepted pair, a digest
  inferred from count arithmetic, endpoint inventory mismatch, or an
  unexplained pair-input delta;
- source PR-tip/`B` or certificate `Q`/`M` tree inequality; unauthenticated
  certificate PR or merge identity; `B` or `M` absent from authoritative main
  first-parent history; a non-record `B..Q` or landing-edge touch; or current
  head outside the certified chain;
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
`approval_pending_only`. A free-form log string is not eligibility evidence.

After the designated reviewer approves the exact unchanged head, an
Actions-write carrier performs exactly one **Re-run all jobs** on that same CI
run. The carrier transports the re-evaluation and gains no review authority.
Attempt 2 treats the replayed event payload as stale venue coordinates only.
It freshly fetches the PR, base, commits, every review page, and the selected
review object. A final authenticated readback immediately before Jon's merge
closes the post-green dismissal race and repeats the live governance
projection, bypass posture, and required-context posture rather than trusting
the green run alone.

This option is narrower than adding a new approval-aware workflow trigger: it
needs no new required context or token inside CI, and it matches #528's
observed one-attempt-2 history. It does require explicit venue-law amendments;
today's rerun is not a general refresh right.

### Proposed contract text

```text
POST-RECORD APPROVAL OBSERVATION; SOLE U-59 EXCEPTION TO U-66.

A candidate is eligible for one re-evaluation only when CI attempt 1 is the
unique CI run for the pull_request event at the exact record-containing head
and terminates in machine state approval_pending_only: every content,
provenance, succession, transport, pagination, PR-identity, base-currency, and
record predicate evaluated before the approval boundary passes, and the sole
finding is absence of the recorded reviewer's decisive approval. Contexts
structurally skipped behind that deliberate RED are not success evidence.

Attempt 1 MUST bind repository and PR immutable IDs, base ref and base SHA,
candidate head and tree, record path and raw-byte digest, workflow ref and SHA,
event, run ID, run number, and attempt number.

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
every other producer at attempt 1. Immediately before merge, authenticated
transport MUST repeat the PR/head/tree/base/record/latest-review readback and
the complete live governance projection, including bypass []/never and exact
required-context identity and posture.

Any other attempt-1 finding, mutable-field reliance on the replayed event,
movement, incomplete transport, additional run or attempt, partial rerun,
attempt-2 failure, or final-readback failure voids the exception. There is no
attempt 3. The cure is a new linear record successor and a new approval venue.
```

### Venue-law changes required by a later implementation

The following text surfaces must change together, in a separately reviewed
contract act:

1. `C_Language_Specification/GARNET_TRUST_KERNEL_ROLLING_REVIEW.md`, especially
   the approval-selection law around lines 94–105, to define
   `approval_pending_only`, stale-payload non-authority, attempt-2 live
   transport, and the final premerge readback.
2. The same contract's Usage section around lines 185–200, to prescribe the
   exact operator sequence and forbid partial/debug/third reruns.
3. `.github/rulesets/README.md` around lines 20–24, to state that “latest
   checks” may be satisfied through this one same-run observation and no other
   rerun.
4. Repo procedural law in `AGENTS.md`, so the U-59 exception and final
   readback survive chat and operator turnover.
5. U-66's one-firing venue law, after its pending sweep allocation lands, by an
   append-only amendment or companion that names U-59 as its only mechanically
   checked exception. The landed sweep record must not be edited in place.
6. The rolling reporter, governance run census, focused fixtures, and CI
   operator procedure. No workflow trigger or broader job token is required
   by this selected design.

### Failure modes

- Attempt 1 has any finding besides missing exact-head approval, or downstream
  jobs were never fully reevaluated by “all jobs.”
- A body/title/base edit, retarget, close, draft conversion, new base SHA, new
  head/tree/record bytes, workflow movement, or other mutable fact is hidden by
  the replayed payload.
- Review enumeration is partial; direct-object equality fails; the latest
  decisive event is dismissed, changes-requested, or approved at another head;
  approval changes after green; or the final review/governance/bypass/context
  readback is absent or divergent.
- The reviewer, rerun carrier, and author roles are conflated; the carrier's
  Actions permission is treated as approval; or CI inherits a broader token.
- There is a new run ID, close/reopen, dispatch, partial/job/debug rerun, a
  third CI attempt, or any second attempt for another producer.
- A diagnostic string rather than the complete machine predicate grants
  eligibility, or run accumulation is reduced to “latest attempt wins.”

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
`tree(Q) == tree(M)`, and a complete landing-edge census. Any record-inflated
raw pair at `M` is restated separately from the new accepted pair at `C`.

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
completely censused, and current HEAD descends through M. Any record-tail raw
pair movement MUST be restated separately from the accepted pair at content
head C. Missing review routing, unequal trees, incomplete landing transport,
or ambiguous ancestry is RED.
```

### Class A — registry yank and reversal

**May move:** one exact package-name-and-version ignore row; its reason and
machine-checkable expiry/removal condition; focused fresh-registry fixtures;
the event certificate; and deterministic pins/mirrors derived from those
records.

**May never move:** `Cargo.lock`; the resolved source/checksum; dependency
requirements or graph; Cargo manifests; vendored/package bytes; global
`yanked = "deny"`; unrelated advisory policy; or a name-only, range, wildcard,
or multi-package exception.

Evidence must reproduce the event with a fresh `CARGO_HOME` and corroborate it
through the registry API and sparse index. An ambient cache is not evidence.
Absence of a RustSec advisory is not safety proof, because an advisory database
may predate the yank.

Proposed class contract:

```text
REGISTRY-YANK ADDITION is BOUNDED WEAKENING. It is eligible only for one exact
locked name@version and checksum, with Cargo.lock, manifests, dependency graph,
source bytes, and global yanked-deny byte-identical. Every other yank remains
denied. The exception expires on registry reversal, dependency movement, or a
different checksum. Jon's explicit weakening approval is required.

REGISTRY-YANK REVERSAL is STRENGTHENING only when the exact ignore row is
deleted in the same atomic event and unconditional global yank denial is
restored. If a fresh API+index+cold-cache census reports reversal while the row
remains, the event is NEUTRAL DOCUMENTATION WITH STANDING WEAKENING and is RED
for closure pending deletion or a new Jon ruling.
```

Required reviewer statements:

```text
BOUNDED WEAKENING — exactly one locked package/checksum is excepted; every
other yank remains denied; no dependency or lock byte moved; the exception
expires on reversal or dependency movement.

STRENGTHENING — the exact exception is deleted and unconditional global yank
denial is restored.
```

Class failure modes include stale cache, API/index disagreement, checksum or
lock movement, a broadened exception, a missing expiry, advisory absence used
as proof, exception retention after verified reversal without a new ruling,
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

**May never move:** `_strict_equal`; the top-level `RULESET_KEYS` projection
set; unknown-field rejection; bypass actors or bypass result;
target/ref/enforcement/conditions; rule identity/order;
required-context identity, row order, count, producer integration, or semantic
pins except their mechanically derived canonical document hashes; transport,
pagination, permissions, or fail-closed direction.

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
leaf, generic unknown-field tolerance, ambiguous path/rule identity, reorder,
bypass or required-context movement, false/absent strengthening value, a
candidate-controlled projection, or inference from `updated_at` alone.

### Class C — toolchain lint activation

**May move directly:** one exact CI compiler/Clippy pin, one lint-triggered
source site with a total behavior-equivalence proof, and the closest procedural
contract required by repository law. Every other moved pin, manifest, proof,
or evidence output must be discovered as a descendant in the machine-derived
producer/pin closure; it is not admitted by a hand-written path list.

**May never move:** root `rust-toolchain.toml`; MSRV 1.95; Cargo manifests,
lock, dependencies, public API, or serialized behavior; `allow`, `expect`, or
Clippy suppression; lint flags; workflow job/context names, permissions,
`needs`, conditions, or required-context count; external action `uses:` pins;
more than the single declared lint cure; or an unrelated cleanup.

The historical A3 shape demonstrates the closure requirement: compiler input,
syntax site, procedural law, producer inventory pin, governance semantic and
binding pins, and required-context contract pins moved directly; the source
then moved package provenance, one of ten browser runtime-input pins, and the
browser proof. Identical JS/Wasm or screenshot bytes did not close that chain.

Proposed class contract:

```text
TOOLCHAIN LINT ACTIVATION is eligible only for one exact compiler/Clippy pin
and one exact lint cure. The source rewrite MUST be behavior-equivalent for the
entire input domain under unchanged guards and MUST pass at workspace MSRV and
the pinned lint compiler. The compiler pin is
REPRODUCIBILITY-STRENGTHENING; the source rewrite is BEHAVIOR-NEUTRAL. These
dimensions MUST be stated separately.

Starting from the predecessor's complete commit set, the producer census MUST
compute a fixed-point closure from every changed input through every semantic
pin, provenance file, manifest, mirror, runtime-input aggregate, and proof.
Every dependent output MUST be regenerated or independently verified at the
exact candidate. Acceptance between cure and closure is void; a proof captured
at an older tree cannot close the current candidate. closure_open MUST equal
[].
```

Required reviewer statement:

```text
BEHAVIOR-NEUTRAL source rewrite; REPRODUCIBILITY-STRENGTHENING exact compiler
pin. The unchanged guard covers the complete input domain at MSRV 1.95 and the
pinned lint compiler. The input -> producer -> output graph, tool versions,
lock equality, exact output delta, reproducibility checks, and closure_open=[]
have been independently verified.
```

Class failure modes include any suppression, guard/API/behavior change,
failure under either compiler, MSRV/root-toolchain/lock/dependency movement,
workflow policy movement beyond the exact pin, unavailable or
non-reproducible producer, self-reference, stale runtime-input dictionary,
proof from an older tree, or any open/stale dependent.

### Shared R3 failure modes

An event is out of class if two classes are needed to explain it, the external
observation is ambiguous, the full edge delta or producer closure is
incomplete, the impact map does not discharge all five WV criteria, the
direction is mislabeled, preservation is not exact, or the reviewer finds an
unbounded semantic effect. Unknown is RED. Combining independent events to
save a boundary is not mechanical economy; it is a new review scope.

## R4 — no weakening

### Proposed mechanism: conservation matrix

| property currently proved | R1 record succession | R2 ordering cure | R3 bounded events |
|---|---|---|---|
| **Bypass emptiness** | Not mutable by any record subtype; base law is hash-bound. | Same ruleset and same head; the mandatory live final governance readback repeats `[]/never`. | Forge class requires `[]/never`; other classes cannot touch bypass. |
| **Head and tree binding** | Binds native `H/T`, final source PR tip `R`, source landing `B`, exact approved certificate tip `Q`, equal-tree landing `M`, and current descendant chain. | Both attempts and final readback bind the identical head/tree/record/base. | Each event binds content head `C`, approved certificate tip `Q`, equal-tree first-parent landing `M`, and current exact pair; no backdating. |
| **Transport completeness** | Establishment authenticates the complete PR graph and becomes self-contained; missing objects are RED. | Attempt 2 freshly paginates PR/commits/reviews and reads the selected review directly. | Each external source has a bounded complete transport; disagreement or partial data is RED. |
| **Drift census** | L-12 enumerates every edge and parent plus endpoint diff; producer-qualified operations prevent path laundering. | Candidate bytes cannot move between attempts. A run census prevents venue laundering. | Full edge census plus class predicate and producer fixed point; combining classes is forbidden. |
| **Preservation** | Native manifests, artifacts, proofs, reviews, and old certificates remain byte-exact. | No repository bytes change; the approval is external state bound to the same record. | Predecessor root and every superseded boundary remain append-only; new pair is linked, never substituted backward. |
| **Append-only records** | Distinct certificate schema, immutable registry entries, unique linear tip, derived introduction commit. | The existing record is not edited after approval; failure requires a new successor. | One new immutable event certificate; prior event and evidence bytes cannot be resealed. |
| **`_strict_equal` and fail-closed direction** | Reporter executable movement is forbidden; base verifier/classifier is authority. | Only observation time changes; equality predicates do not. | Forge equality is byte-identical; registry weakening is explicit and exact; toolchain suppression is forbidden. |
| **Approval and reviewer independence** | Exact record head and reviewer object bind the certificate; scope explicitly does not extend/backdate. | Reviewer and rerun carrier are distinct authorities; latest decisive review is re-read before merge. | Direction and delta proof receive independent review; weakening additionally needs Jon. |
| **Cure/proof atomicity** | Terminal producer refresh is admitted only as its already-reviewed full closure. | Attempt 1 cannot become eligible with an open closure finding. | `closure_open=[]` is a precondition, strengthening U-68 from carry prose to a machine state. |

The changes are therefore economic, not evidentiary: R1 replaces a repeated
native execution with an exact projection theorem; R2 adds a second observation
of the same immutable candidate; R3 replaces a whole-platform replay only when
a closed, class-specific noninterference or direction proof explains the
entire content delta.

### Proposed contract text

```text
CONSERVATION RULE.

No succession or bounded re-acceptance MAY delete, relax, bypass, infer, or
silently substitute any fail-closed meta-property enumerated here. Bypass
emptiness, exact head/tree/pair binding, complete authenticated transport,
per-edge drift census, predecessor preservation, append-only linear records,
strict equality, independent decisive review, and producer-closure atomicity
MUST each be re-proved at the new observation boundary.

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
            -> zero or more bounded-event certificates
            -> current exact observation
```

A record certificate preserves the accepted pair and only changes where the
accepted theorem is observed. A bounded-event certificate changes the accepted
pair only through a complete R3 delta theorem. A new native terminal freeze
creates a new native root and supersedes the prior chain with preservation.
The reporter selects exactly one linear, effective tip; a fork, gap, duplicate,
or unverifiable predecessor is RED.

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
   tree(M), complete landing-edge census, unchanged native-accepted pair, and
   a separately recomputed raw observed pair. The source review is neither
   extended nor backdated.

3. RECORD SUCCESSOR. Above an effective certificate, later commits preserve
   acceptance only while every commit edge is producer-censused and every
   operation is qualified record-only. The accepted pair remains unchanged;
   any raw-pair movement is restated separately and fully explained. Existing
   records, evidence, and chain links remain immutable. A record-path logic
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
   certificate chain, exact current head/tree, current accepted pair, current
   raw observed pair, effective main landing, latest decisive review, and fresh
   external observations required by its classes. Missing, stale, ambiguous,
   conflicting, forked, partial, or unavailable evidence is RED.
```

### Failure modes

- Treating squash-tree equality alone as review or acceptance; requiring an
  ephemeral branch ref forever; or assuming ancestry that squash discarded.
- Omitting the certificate PR tip, authenticated merge identity,
  `tree(Q) == tree(M)`, first-parent placement, or landing-edge census and
  thereby recreating U-58 for the certificate itself.
- Treating a record-prefix match as semantic harmlessness, ignoring transient
  touches, or allowing reporter logic to certify itself.
- Mutating the accepted pair on record succession, claiming native evidence
  for a raw successor pair, or changing an R3 pair without a closed impact
  theorem.
- Backdating review over later bytes, selecting among multiple chain tips,
  skipping a predecessor, editing a historical certificate, or resealing old
  evidence.
- Continuing after a content change because the change is small, external,
  strengthening, already green elsewhere, or expensive to freeze.

Any failure kills the proposed succession at that edge. It does not kill the
historical acceptance record, which remains preserved as superseded evidence.

## Activation boundary

The current #528 state is the migration exhibit, not an activated instance of
this proposal. A later implementation would need, in one separately governed
act, canonical certificate schemas and registries, base-controlled producers,
reporter support, adversarial fixtures, venue-law amendments, and a reviewed
initial certificate for the #528 topology. That act necessarily changes
contracts and code and therefore cannot bootstrap itself through R1. Jon must
rule its entry ceremony. Until that happens, the current reporter's PARTIAL
result is correct and must not be rewritten as accepted by this brief.

## DECISION POINTS

### Choices reserved to Jon

1. Whether to adopt the two-pair model: immutable `native_accepted_pair` plus
   non-authoritative `successor_observed_pair`, with R3 alone permitted to
   advance the current accepted pair.
2. Whether the succession certificate, distinct suffix, append-only registry,
   unique-tip rule, derived `B`/`Q`/`M` landing model, and explicit
   rolling-review trigger/digest coverage become contract law.
3. Whether #528 receives a one-time migration certificate and what entry
   ceremony authorizes the code/contract change that cannot self-bootstrap.
4. Whether U-59's same-run **Re-run all jobs** cure is adopted instead of a new
   approval-aware trigger, and which Actions-write carrier may perform that
   transport-only act.
5. Whether the final authenticated premerge readback is performed by the
   reporter, a Jon-only merge ceremony, or both; it may not be omitted.
6. Whether the three R3 event classes and their exact impact-proof matrices are
   accepted, and when any native-impact reachability still mandates Windows
   replay.
7. Whether to authorize any exact registry yank exception as
   `BOUNDED WEAKENING`, including its expiry, and whether a verified reversal
   must delete the row or receive a new ruling.
8. Who is the designated cross-family reviewer for succession and each event
   class, and whether a second reviewer is required for a weakening.
9. Whether certificate establishment must persist the complete canonical
   inventories or another equally self-contained proof object; ordinary
   verification may not depend on a surviving pull ref.
10. When the pending sweep allocations become authoritative after merge. U-39
    remains an independent L3 cure and may not ride this L1 records lane.
11. Whether and when to authorize later contract, reporter, workflow, ruleset,
    or procedural-law changes. This brief authorizes none of them.
12. Merge, acceptance, launch state, FIRE/HOLD, and any release action remain
    Jon-only and outside this design record.

### Questions the cross-family reviewer must answer

1. Does the proposed R1 certificate bind the complete native root, final
   approved source PR record tip, source landing `B`, approved certificate tip
   `Q`, equal-tree landing `M`, and current head without self-reference,
   backdating, or reliance on an ephemeral ref?
2. Does the L-12 producer enumerate every commit edge and merge parent, detect
   transient reverted touches, use base-controlled law, and reject every
   reporter change beyond the exact constant projection?
3. Is the native-accepted pair unchanged, is the raw observed pair recomputed
   independently twice, and is every differing digest input exhausted by a
   qualified record operation rather than count arithmetic or assertion?
4. Are all native manifests, artifact members, proof mirrors, reviews,
   preserved bundles, prior certificates, and historical records byte-exact
   and append-only?
5. Can ordinary post-landing verification reproduce the certificate without
   the fork or pull ref, does the registry have exactly one immutable linear
   tip, and do the certificate, registry, producers, and governed policy bytes
   all enter a base-controlled rolling-review digest?
6. Does R2 grant attempt 2 only from machine state `approval_pending_only`,
   ignore mutable stale-payload claims, rerun all jobs, enforce the exact run
   census, and repeat latest-review/base/head plus live governance,
   bypass-emptiness, and required-context readback immediately before merge?
7. Is the registry addition labeled bounded weakening, exact to one locked
   checksum, globally fail-closed around it, and mechanically expired? If
   reversal was observed, was the row actually removed before claiming
   strengthening?
8. Is each forge change the complete finite typed leaf divergence, with
   `_strict_equal` byte-identical, bypass `[]/never`, required contexts exact,
   weaker values RED, and no causal claim inferred from `updated_at`?
9. Is the toolchain source rewrite total-domain behavior-neutral at both MSRV
   and the pinned lint compiler, is the pin separately classified as
   reproducibility-strengthening, and does the producer graph close at
   `closure_open=[]`?
10. Does the R4 conservation matrix hold by executable evidence for every
    mechanism, with no path laundering, pair laundering, cached transport,
    hidden weakening, stale dependent, or reviewer/carrier authority collapse?
11. Does R5 preserve only what is actually proved across squash and records,
    kill succession at the first content change, and retain every superseded
    boundary with preservation?
12. Does the final report state the current truth without repair: WV-6 remains
    PARTIAL at `0607f7fe…`, five of five checks pass, and the pair mismatch is
    unresolved until a separately authorized implementation and entry act?
