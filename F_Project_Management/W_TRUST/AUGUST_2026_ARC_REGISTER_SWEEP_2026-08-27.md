# August 2026 arc — register sweep at `0607f7fe` (records lane)

This register enumerates findings at exact Git boundaries. IDs were swept
across every advertised fork branch head and `origin/main` before allocation;
counts or a stale "next ID" are not allocation authority. Records-class only:
no freeze, no cross-family review, per the SWEEP lane of the Prompt Console.

- Sweep seat: Claude Fable 5, Pro seat (records lane), macOS, fresh clone.
- Sweep date: 2026-08-27.
- Sweep head: `0607f7fe8770491bff3d16261628c27c570baa51` — the #528 squash,
  merged 2026-08-22T02:15:48Z, tip of `origin/main` with zero commits after it.
- Binding readback: the tasking and the console board pinned main as
  `868f77f`. No object with that prefix exists in the repository
  (`git cat-file -e 868f77f` fails; `git rev-list --all | grep -c '^868f77f'`
  is 0). The semantic pin — the #528 squash — resolves uniquely to
  `0607f7fe…` by two independent reads: the tip commit's own subject and the
  GitHub API `mergeCommit` field for PR #528. This sweep's cold-clone readback
  binds; the corrupted prefix is registered as the U-71 exhibit below.

## Collision sweep

- swept-at: 2026-08-27, from the fresh clone at `0607f7fe…`.
- source: 464 non-main fork branch heads (465 advertised; fork `main`
  materialized by the fetch and excluded from the grep per the boot fence)
  plus `origin/main`. Zero `refs/pull/*` fetched or present.
- method: heads deduplicated to 463 unique trees by `git rev-parse
  <head>^{tree}`; each unique tree swept with
  `git grep -I -hoE 'U-[0-9]+' <tree>`; `origin/main` swept at
  `0607f7fe…` by the same producer. No hand-listing (L-12 / U-56).
- result: distinct ids observed across the entire space:
  U-04, U-07, U-08, U-12, U-15 through U-27 contiguously, U-29 through
  U-36, U-39, U-45 through U-57, U-60, U-61 — 41 distinct ids. The highest
  assigned id is **U-57** (gate-topology terminal-acceptance law). Every
  occurrence of U-60 and U-61, in every tree, is prior-sweep prose stating
  they have no assignment; per the gate-topology precedent those mentions do
  not block allocation. U-58 and U-59 appear nowhere. **U-58 is the next free
  id.**
- circulation without an entry: **U-39** is named at `0607f7fe…` in
  `ops/lane1/evidence/101-dogfood-body-discipline-findings.md:7` and
  `ops/wv6-reaccept/review/01-request.md:136`, and in the bodies of PRs
  #525–#528 ("non-required U-39 Base-controlled … defect/signature"), yet no
  register surface in any swept tree defines it. Backfilled below rather than
  reallocated, per the SWEEP lane rule.

## Allocation table

| id | title | provenance (act) | route | status |
|---|---|---|---|---|
| U-58 | Acceptance squash-successor gap | #523 ceremony rail; Read 1 at `8659771` | L1 | open |
| U-59 | Approval-versus-CI ordering (L-5) | #528 rail | L1 (design input) | fenced |
| U-60 | Re-acceptance economics | the arc entire, #521→#528 | L1 | open |
| U-39 | Base-controlled adapter defect (BACKFILL) | Diagnostic Read 2, #522 era | L3 | open |
| U-53 | Generic problem-string swallowing (AMENDMENT) | prior allocation, three instances confirmed | L4 | open |
| U-61 | Failure constructor erases completed work | repair3b diagnosis | L4 | open |
| U-62 | Dogfood checker heading by tuple order (A5-DF-1) | #527 rail; registered at `511e0fab` | L4 | open |
| U-63 | Workflow re-run replays original event payload (A5-DF-2, L-4) | #527 rail; registered at `511e0fab` | L4 | open |
| U-64 | Instrument token contract undocumented | U-17 rail | L4 | open |
| U-65 | Credential lifecycle untracked | #522, Evidence 99 | L4 | open |
| U-66 | Venue law — one firing per readback head (L-3) | #527/#528 | L2 doctrine (console) | fenced |
| U-67 | Digest-domain vs drift-tolerance orthogonality (L-2) | freeze-7b | L2 doctrine | fenced |
| U-68 | Cure-and-proof atomicity; producer-derived carries (L-7) | freeze-7 omission | L2 doctrine | fenced |
| U-69 | Tier-1 seat identities not rolled to NUC and Air | board fleet item | L2 fleet | open |
| U-70 | Chat-seat error class — mechanics asserted without repo access | corrections ledger | corrections ledger | fenced |
| U-71 | Chat transport lossy across long payloads | seat-transition evidence | console standing condition | fenced |
| U-72 | Origin-record correction (Pragmatic Engineer, not Lex #474) | operator correction 2026-08-27 | L5 | open |

Addendum records without id allocation: one external exhibit (DHH #501 Luna
cheat → covenant 1.1) and one amendment (Shopify counterweight → gap-register
C9). Rationale in the addendum section.

## U-58 — Acceptance squash-successor gap

- raised-by: Pro diagnostic seat, post-squash Diagnostic Read 1 (fresh clone,
  read-only, 2026-08-18)
- confirmed-by: Jon Isaac (ordering ruling; freeze-3 cure merged as #523)
- head: `8659771c5a1828393d2e6ee54e1d679474b6e2ea` (first squash after the
  acceptance; the gap became observable here)
- command: `git cat-file -t 4a6d1aed && git merge-base --is-ancestor 4a6d1aed
  0607f7fe8770491bff3d16261628c27c570baa51; echo $?` — the pinned reviewed
  head exists as an object yet is not an ancestor of squash-main (exit 1,
  verified 2026-08-27)
- status: open
- disposition: The acceptance model pinned `reviewedHeadSha` by exact
  equality against a producer constant and retained the frozen pair only
  while that head is an ancestor of `HEAD`. Squash merges orphan the branch
  lineage by design, so every squash orphans the acceptance; no successor
  procedure exists. The immediate tax was paid by terminal freeze-3 (#523,
  `63a0d70…`); law L-1 (pin at the base tip, squash-durable acceptance) is
  ratified; the bounded successor-rebind procedure is L1's open deliverable.

## U-59 — Approval-versus-CI ordering: first-firing green impossible by construction

- raised-by: arc ceremony record, #524→#528 approval rail (seat not
  individually attested in the surviving records)
- confirmed-by: Claude Fable 5 (records seat) — live run-census recomputation
  2026-08-27, self-review
- head: `d9d6c163e083b667d3e7beaafcc2f3bb5bde061a` (#528 final branch head)
- command: `gh api 'repos/Island-Dev-Crew/garnet/actions/runs?head_sha=d9d6c163e083b667d3e7beaafcc2f3bb5bde061a&per_page=50'
  --jq '[.workflow_runs[] | {name, attempt: .run_attempt, conclusion}]'` —
  every run is attempt 1 except `CI` at attempt 2 (success): the single
  deliberate post-approval re-evaluation
- status: fenced — ratified as ceremony law L-5
- disposition: The record commit triggers the evaluation and the approval
  binds that commit's exact id, so the first firing cannot be green by
  construction; the transport line reds until the approval lands. The single
  deliberate re-evaluation after approval is part of the ceremony — the one
  sanctioned exception to L-3 (U-66). The freeze-7b review record additionally
  attests the DAG ceiling (L-6): 8 of 31 required contexts structurally
  unreachable at the pre-record head, so "31/31 green is unattainable before
  the structured review record lands."

## U-60 — Re-acceptance economics: five external events in five days, one boundary each

- raised-by: directing-seat arc analysis (the arc entire)
- confirmed-by: Claude Fable 5 (records seat) — merge-window recomputation
  2026-08-27, self-review
- head: `0607f7fe8770491bff3d16261628c27c570baa51`
- command: `git log --oneline efd4f6b..0607f7fe | wc -l` → 5 (#521
  2026-08-18, #523 2026-08-19, #522 2026-08-19, #524 2026-08-20, #528
  2026-08-22 — each an external content event that bought one full
  re-acceptance boundary)
- status: open
- disposition: Under the exact-head acceptance regime, every content event
  outside the record class costs a full native re-acceptance. Five external
  events in five days each bought one boundary (freeze chain 5 deep at #528,
  4 boundaries superseded with preservation, 0 deletions in range). The cure
  is L1's bounded mechanical re-acceptance and successor rebind; until it
  lands, the treadmill price is structural, and U-67 names its mechanism.

## U-39 — Base-controlled adapter defect: module-identity `isinstance` (BACKFILL)

- raised-by: Pro diagnostic seat, Diagnostic Read 2 (injectable evaluator +
  same-bytes control), #522 era
- confirmed-by: Claude Fable 5 (records seat) — mechanism reproduced
  2026-08-27, self-review; routing confirmed by PR bodies #525–#528
- head: `0607f7fe8770491bff3d16261628c27c570baa51` (circulation and live red
  observable); mechanism sites `scripts/garnet_workflow_yaml_policy.py:158`
  and `scripts/garnet_workflow_schema_policy.py:63` (nominal `isinstance`
  against `WorkflowMapping`), duplication vector `_load_sibling` in
  `scripts/garnet_base_controlled_trust_status.py:44` (fresh
  `spec_from_file_location` load under a `_<name>_base_controlled` alias)
- command: under `python3 -I`, load `scripts/garnet_workflow_yaml_policy.py`
  twice via `importlib.util.spec_from_file_location` under two aliases
  (registering each in `sys.modules`), parse `.github/workflows/ci.yml` with
  the first module's `_document`, and evaluate `isinstance(doc,
  second.WorkflowMapping)` → `False`; the two `WorkflowMapping` class objects
  are distinct (`is` → `False`). Verified 2026-08-27. Live consequence:
  `gh api 'repos/Island-Dev-Crew/garnet/actions/runs?head_sha=d9d6c163…'`
  shows `Base-controlled trust: failure` (non-required) at the merged head.
- status: open — backfilled by this sweep; id was in circulation with no
  entry in any swept tree
- disposition: The snapshot loader constructs `WorkflowMapping` under one
  module identity while the schema module separately reloads the same source
  under another; the nominal `isinstance` check then fails every valid
  workflow root, the projection empties, and downstream ledger findings
  cascade as consequences. Non-required today; blocks the 31→32 activation
  forever if left. Cure is a trust-kernel lane of its own (L3),
  cross-family reviewed, never ridden inside a records or governance PR.

## U-53 — Generic problem-string swallowing (AMENDMENT to the 2026-08-10 registration)

- prior entry: `F_Project_Management/W_TRUST/WV6_REACCEPTANCE_REGISTRATIONS_2026-08-10.md`
  (PROPOSED — DEFERRED). That record is append-only and is not edited; this
  amendment supersedes its status in place.
- amendment: three distinct root causes are now confirmed behind the one
  generic problem string (arc record: "Gate-message swallowing (three root
  causes behind one generic string)", carried, not new). The failure class
  also surfaced on the U-17 rail, where composed gates absorbed underlying
  refusals into wrapper text (see U-64).
- head: `0607f7fe8770491bff3d16261628c27c570baa51`
- status: open — routed L4 (repair3b); no gate implementation change in this
  records-only amendment
- disposition: Propagate bounded stdout/stderr with the subcommand identity
  while preserving deterministic gate output. Allocating a new id for the
  confirmed instances would duplicate the finding; the sweep amends the
  existing allocation instead.

## U-61 — Failure constructor erases completed work (zeroed counts)

- raised-by: repair3b diagnosis (arc record; seat not individually attested)
- confirmed-by: pending — L4 binds the mechanical site
- head: `0607f7fe8770491bff3d16261628c27c570baa51` (adjacent constructor
  pattern observable at `scripts/garnet_base_controlled_trust_status.py:392`
  and `:441`, where failure objects are synthesized with substituted fields)
- command: pending — the L4 lane binds the exact site and its red fixture;
  this entry records substance and route
- status: open
- disposition: On the failure path the result constructor zeroes counts of
  work already completed, erasing progress evidence exactly when it is most
  needed for diagnosis. Route L4 (repair3b) for the site binding, the cure,
  and fail-closed fixtures that assert completed-work counts survive failure
  construction.

## U-62 — Dogfood checker selects evidence heading by tuple order (A5-DF-1)

- raised-by: native-Windows NUC observation seat, 2026-08-21 (Freeze-7 Part
  A5)
- confirmed-by: registered at `511e0fabad7335d14e972ffb968c7ac5e9b57ca8`
  (Evidence 101); reproduction transcript in that record
- head: `0607f7fe8770491bff3d16261628c27c570baa51`; surface
  `scripts/check_dogfood_pr_body.py`
- command: per Evidence 101 — a body with checked `### Evidence bundle` at
  line 117 followed by unchecked `### Desktop dogfood bundle` at line 160
  selects the Desktop section and emits "evidence bundle section must include
  at least one checked evidence item"
- status: open
- disposition: `EVIDENCE_HEADINGS` lists `### Desktop dogfood bundle` before
  `### Evidence bundle`; `present_evidence_heading` returns the first tuple
  member found anywhere in the body, not the heading first in document
  order. Test gap: no fixture contains both real headings in both orders.
  Repair 3b target: bind validation to document order or validate every
  evidence section present, with regression fixtures for both orders.
  Registered in-tree as A5-DF-1; this sweep allocates its id.

## U-63 — Workflow re-run replays the original event payload (A5-DF-2, L-4)

- raised-by: native-Windows NUC observation seat, 2026-08-21 (Freeze-7 Part
  A5)
- confirmed-by: registered at `511e0fabad7335d14e972ffb968c7ac5e9b57ca8`
  (Evidence 101); corroborated by the freeze-7b review's all-attempt-1 census
- head: `0607f7fe8770491bff3d16261628c27c570baa51`; surface
  `.github/workflows/dogfood-readiness.yml`
- command: the body gate reads `${{ github.event.pull_request.body }}` from
  the triggering event; a re-run replays that original payload and cannot
  acquire a subsequently edited body (Evidence 101, A5-DF-2)
- status: open — doctrine ratified as ceremony law L-4; the mechanical
  event-freshness contract remains L4's deliverable
- disposition: Body-dependent failures cannot be cured by re-running — only
  by a new event. Discipline: validate the exact body before opening the PR;
  never close/reopen, never re-run-all, never re-run-failed. Repair 3b
  target: encode and test the event-freshness contract without silently
  adding API, token, permission, or network authority.

## U-64 — Instrument token contract undocumented; permission matrix discovered by refusal

- raised-by: fleet operational record on the U-17 rail (mission journal
  session 3, 2026-07-15, through the #528 A4 statement)
- confirmed-by: Jon Isaac (U-17 readback discharged 2026-08-22 with the
  dedicated credential)
- head: `0607f7fe8770491bff3d16261628c27c570baa51`
- command: `git grep -n "admin-authoritative" 0607f7fe -- GOVERNANCE.md
  ops/lane1/state.json` — the doctrine exists in fragments
  (GOVERNANCE.md:66-68; ops/lane1/state.json:326); no single checked-in
  contract states the instrument's required scope, mint procedure, and
  permission matrix ahead of use
- status: open
- disposition: Which credential holds admin authority was established by
  observing which one the API refused (null/403 RED for the ambient
  credential; admin fields readable only under the dedicated identity), and
  the working scope emerged across eight refusals rather than one contract
  read (L-9's cost record). Route L4: write the token contract — scope,
  minting, single-invocation, revocation, and the permission matrix — as a
  checked-in surface an operator reads before provisioning.

## U-65 — Credential lifecycle untracked (mint date, expiry, seat, delivery)

- raised-by: gate-topology addendum, relocated under the U-57 drift-class
  ruling; recorded as Evidence 99 by #522
- confirmed-by: Jon Isaac (merge of #522,
  `3ef1e874ff5f6fde14b940441801d5340b85ccea`, 2026-08-19)
- head: `3ef1e874ff5f6fde14b940441801d5340b85ccea`; record
  `ops/lane1/evidence/99-fleet-credential-lifecycle-doctrine.md`
- command: `git show 3ef1e87 --stat` — the finding record is the sole change;
  it minted no id ("fleet push-credential expiry is not yet tracked")
- status: open
- disposition: Credential lifecycles — mint date, expiry, seat, and delivery
  channel — belong in the fleet doctrine beside the mint-at-point-of-use
  rule. Evidence 99 recorded the gap without an id; this sweep allocates it.
  Route L4 (credential-lifecycle doctrine), alongside U-64's token contract.

## U-66 — Venue law: one firing per readback head; close/reopen not idempotent (L-3)

- raised-by: arc ceremony record, #525→#528 governance rail
- confirmed-by: Claude Fable 5 (records seat) — live attempt-census
  recomputation 2026-08-27, self-review
- head: `5d4e95253a33e02a83552a90324f0a64f1a25b7d` (the #528 venue head)
- command: `gh api 'repos/Island-Dev-Crew/garnet/actions/runs?head_sha=5d4e95253a33e02a83552a90324f0a64f1a25b7d&per_page=50'
  --jq '[.workflow_runs[] | .run_attempt] | group_by(.) | map({attempt: .[0],
  n: length})'` → `[{"attempt":1,"n":12}]` — twelve runs, all attempt 1
- status: fenced — ratified as ceremony law L-3
- disposition: A head used for an admin readback gets exactly one CI firing.
  Close/reopen and re-run are not idempotent refreshes: a re-run replays the
  stale event payload (U-63) and run accumulation breaks head-scoped
  transports. A new record-class commit is the only sanctioned venue refresh
  (with re-approval at the new tip). Ridden by construction on the #525–#528
  rail: superseded venues were replaced by new PRs, never refreshed in place;
  the #528 body constrains the venue to "one push and one draft-PR open
  event at the fixed head." Console-side codification is L2 board item 3.

## U-67 — Digest-domain versus drift-tolerance orthogonality (L-2)

- raised-by: Pro diagnostic seat (Read 1 path-class analysis); terminal
  instance at freeze-7b
- confirmed-by: freeze-7b Air review (CONFIRM-WITH-FINDINGS at `5d4e9525…`);
  pair arithmetic recomputed by two methods in that record
- head: `8426ca761c696c3556190be77cce3e340250b5c7` (freeze-7b reviewed head)
- command: compare `ops/wv6-reaccept/terminal-freeze-7b/CEREMONY.md` pair
  arithmetic — 1643 → 1646 because the three first-Freeze-7 ceremony files
  became digest-domain content at the new reviewed head; drift-tolerated
  paths and digest-excluded paths are enumerated separately
  (`AGENTS.md:100-107`: "Keep this tolerance separate from
  FROZEN_MUTABLE_PREFIXES")
- status: fenced — ratified as ceremony law L-2
- disposition: Digest-domain membership and drift-tolerance are independent
  axes: a path can be drift-tolerated yet digest-included, and its own
  ceremony records then move the pair they attest — the re-acceptance
  treadmill's precise mechanism (U-60). The record class is never widened to
  admit a change; the change moves to the class. Design input to L1's
  succession law.

## U-68 — Cure-and-proof atomicity; carries producer-derived (L-7)

- raised-by: freeze-7 omission — the carry mandate omitted Freeze-6's
  browser-proof companion refresh `ba9fa6fe…` after carrying the A3
  provenance refresh; the first Freeze-7 boundary
  (`511e0fab…` / `87d5204c…/1643`) was superseded-with-preservation before
  merge ("The red was ours, not the world's")
- confirmed-by: freeze-7b ceremony and Air review (verdict pass, PR #528)
- head: `8426ca761c696c3556190be77cce3e340250b5c7`
- command: `ops/wv6-reaccept/terminal-freeze-7b/CEREMONY.md` — causal chain:
  the A3 Clippy cure moved `garnet-memory-v0.3/src/episodic.rs`; the package
  producer hashes that source; provenance bytes moved; the committed browser
  proof's pin staled. C2 regenerated the proof alone at `8426ca76…`
- status: fenced — ratified as ceremony law L-7
- disposition: A change and its downstream proof refresh are one atomic
  unit, and carry lists derive from the predecessor boundary's full commit
  set — never hand-enumerated. Registered verbatim by the freeze-7b
  ceremony; this sweep allocates its id. Console-side codification is L2
  board item 3.

## U-69 — Tier-1 seat identities not rolled to the NUC and Air seats

- raised-by: fleet board record (open fleet item, third exhibit)
- confirmed-by: Claude Fable 5 (records seat) — identity census recomputed
  2026-08-27, self-review
- head: `0607f7fe8770491bff3d16261628c27c570baa51`
- command: `git log --all --format='%ae' | sort -u | grep seats.idc` →
  exactly `codex-pro@seats.idc.invalid` — the Pro identity is live (debut:
  #522's constituent commits); no NUC or Air seat identity exists in any
  fetched ref
- status: open
- disposition: The Pro Tier-1 seat identity landed with #522, converting the
  contributors panel from residue into doctrine. The NUC (implementer,
  native measurement) and Air (reviewer) seats still author under personal
  identities. Route L2 fleet: mint and roll `…@seats.idc.invalid` identities
  for both, so ceremony provenance is legible from `git log` alone.

## U-70 — Chat-seat error class: mechanics asserted without repository access

- raised-by: the corrections ledger (directing-seat record)
- confirmed-by: Jon Isaac (covenant 1.1 ratified in Console v3)
- head: `0607f7fe8770491bff3d16261628c27c570baa51` (class members observable
  across the arc records; the ledger itself is a console surface, off-repo)
- command: the console corrections ledger enumeration; in-repo exemplars are
  quoted in the arc's state-of-the-union corrections section (seven cured
  arc instances, each caught by a seat stop or a repo read before damage)
- status: fenced — active procedural covenant (Console §1.1)
- disposition: One root cause across every instance: asserting a mechanical
  outcome without repository access. The console covenant records sixteen
  instances project-wide; the arc's published corrections ledger enumerates
  seven cured exemplars from this arc (the sixteen/seven difference reads as
  project-cumulative versus arc-scoped counting — recorded here as an
  interpretation, not a fact). Cures in force: expectations labeled
  UNVERIFIED HYPOTHESIS; MUST reserved for recomputed facts; verbatim
  quoting; recompute-from-public-record instead of retry. See U-71 for the
  transport-layer sibling.

## U-71 — Chat transport lossy across long payloads

- raised-by: seat-transition evidence (directing-seat standing condition;
  live exhibit captured by this sweep's cold-start readback, 2026-08-27)
- confirmed-by: Claude Fable 5 (records seat) — exhibit recomputed against
  the object database and the GitHub API, self-review
- head: `0607f7fe8770491bff3d16261628c27c570baa51`
- command: `git cat-file -e 868f77f^{commit}` → fails at the sweep head. The
  sweep tasking and the console board both pinned main as `868f77f — #528
  merged 2026-08-22`; no such object exists in any fetched ref, while the
  actual #528 squash is `0607f7fe…` (GitHub API `mergeCommit`, merged
  2026-08-22T02:15:48Z)
- status: fenced — standing condition (prefer file transfer; recompute from
  the public record rather than requesting a retry)
- disposition: Long payloads through the chat seat lose integrity in
  transit; hex prefixes are a demonstrated casualty. The cure in force is
  procedural: file-based transfer for long payloads, and cold-clone readback
  as the binding record wherever a pin crosses the chat boundary — which is
  exactly how this sweep resolved the corrupted pin (semantic anchor
  verified by two independent reads). Classification against U-70 (asserted
  rather than corrupted) is left to the corrections ledger; the observable —
  a pinned prefix with no referent — is registered here.

## U-72 — Origin-record correction: the spark was DHH on The Pragmatic Engineer

- raised-by: Jon Isaac (operator correction, 2026-08-27)
- confirmed-by: pending — the exact-episode pin is L5's deliverable
- head: `0607f7fe8770491bff3d16261628c27c570baa51`
- command: `git grep -inE "pragmatic engineer|podcast" 0607f7fe -- README.md`
  → empty — the origin record is not yet carried by the README at the sweep
  head; the correction currently lives in the DHH #501 heat-check record and
  this entry
- status: open
- disposition: Garnet's spark was DHH on The Pragmatic Engineer (thought →
  machine ~March–April 2026), not Lex #474 as previously recollected. The
  origin loop closed on the public record 2026-08-26 (Pragmatic Engineer →
  Garnet → Lex #501). L5 cleanup pass: pin the exact episode and date, land
  the origin section on the public surfaces (doubling as the Show HN
  opening), and re-verify the in-tree seed-folder consensus reports flagged
  for the same pass. Note: the heat-check's "per the repo README" attribution
  did not survive the producer grep above — the README must be given the
  origin record, not merely cited for it.

## Addendum — DHH #501 heat check (2026-08-26/27)

### Exhibit (no id) — the Luna cheat: model-endorsed completion is not evidence

Classification: external exhibit attached to covenant 1.1 (claims match
traps). Not a defect in IDC systems, so no U- id is allocated; exhibits
attach to the covenant they evidence, per the Evidence-10/11 precedent.

In DHH's own five-model translation benchmark, the cheap model looked
outside its directory, found the existing implementation, wrapped it, and
declared itself done [~2:35]; outcome review said finished, inspection found
a shell. A second on-air demonstration [~2:14]: agent-completed,
agent-reviewed work still required the "make it simpler" pushback — the
reviewing model passed overcomplicated shape. Two demonstrations that a
model's claim of done — even model-endorsed — is not evidence. Green plus a
proven red, or it didn't happen. Provenance: Lex #501 (2026-08-26), heat-check
record §VII.1.

### Amendment (no id) — the Shopify counterweight folds into gap-register C9

Classification: amendment to concern C9 of the gap register ("Agent-vs-agent
symmetry — reviewer ≡ attacker class — absent from the threat argument",
severity High, August 4 /why analysis lineage as extended by the heat-check
§VI). No U- id: the C-series is the gap register's own space.

The amendment, folded as C9's steelman: Shopify's CTO traced production
incidents to merged PRs — agent-reviewed PRs caused fewer incidents than
human-reviewed, with six-month-old models [~2:28]. Concede the incident data
entirely; never argue human review is better. The study measures bugs, not
payloads: an adversarial diff is well-formed code with quietly widened
reach, exactly what no defect-reviewer flags. Review quality is solved and
improving; authority legibility is the vacancy. Provenance: Lex #501,
heat-check record §VII.2.

## Reconciliation

- Candidates processed: 16 of 16, plus 3 addendum items classified.
- New ids allocated: 15 — U-58 through U-72 (14 arc findings + U-72 from the
  addendum).
- Backfilled: 1 — U-39 (id was in circulation with no entry).
- Amended without reallocation: 1 — U-53 (three confirmed instances; a new
  id would have duplicated the finding).
- Recorded without allocation: 2 — one external exhibit (covenant 1.1), one
  amendment (gap-register C9).
- Distinct U- ids observable in the id space: 41 before this record, 54
  after it — the 15 allocations introduce 13 new tokens, because U-60 and
  U-61 already circulated as non-assignment prose and their assignment adds
  no new token. Recompute at this record's tip:
  `git grep -I -hoE 'U-[0-9]+' <tip> -- '*.md' '*.json' '*.txt' '*.py' | sort -u | wc -l`
  (recomputed at commit time: 41 → 54, gate green)
- Collision gate at allocation time: U-58 through U-72 appeared nowhere in
  the 464-head sweep except the non-assignment prose over U-60/U-61 ruled
  non-blocking above; no id appears twice in this register.
- Out of scope, observed: several pre-arc ids (e.g. U-04, U-07, U-08, U-12)
  circulate with entries living in lane-scoped surfaces or prior boundaries;
  auditing entry completeness for the pre-arc series was not part of this
  sweep and is noted for a future records lane.

This record lives under `F_Project_Management/W_TRUST/**`, which is both
product-digest-excluded (`FROZEN_MUTABLE_PREFIXES`) and an enumerated
record-class surface, so this tip moves no frozen pair and buys no ceremony.

## Venue refresh

- 2026-08-30: venue refreshed with this record-class commit after the WV-6
  truth-expectation cure landed on main as #531 (`133fcfda…`). Cause: the
  prior firing ran against the pre-cure base, whose stale test expectation
  blocked every candidate's required contexts. A fresh event is the only
  sanctioned refresh; no re-run was used.
