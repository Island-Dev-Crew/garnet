# Landing arc — register sweep at `7f77c0d6` (records lane)

This register enumerates findings at exact Git boundaries. IDs were swept
across every advertised fork branch head and `origin/main` before allocation;
counts or a stale "next ID" are not allocation authority. Records-class only:
no freeze, no cross-family review, per the SWEEP lane of the Prompt Console.

- Sweep seat: Claude Fable 5, Pro seat (records lane), macOS, fresh clone.
- Sweep date: 2026-08-31.
- Sweep head: `7f77c0d66621630911ac73914816c3b9a7933abd` — the #529 squash,
  tip of `origin/main` by remote readback, closing the landing arc
  (#531 WV-6 truth cure → #530 L1 brief and review record → #529 register
  sweep).

## Collision sweep

- swept-at: 2026-08-31, from the fresh clone at `7f77c0d6…`.
- source: 465 non-main fork branch heads (fork `main` excluded from the grep
  per the boot fence) deduplicated to 464 unique trees, each swept with
  `git grep -I -hoE 'U-[0-9]+' <tree>`, plus `origin/main` at `7f77c0d6…`
  by the same producer. Zero `refs/pull/*`. No hand-listing.
- result: the distinct id space runs U-04 through U-72 with the historical
  gaps; the previous sweep's allocations are landed and authoritative; no
  occurrence at or above U-73 exists in any swept tree. **U-73 is the next
  free id.** Distinct ids at the sweep head: 54.

## Allocation table

| id | title | provenance (act) | route | status |
|---|---|---|---|---|
| U-73 | No sanctioned structured record format for records-class cross-family verdicts | #530 landing act | L1 design input | open |
| U-74 | CI checks out the PR head SHA, not the merge ref | L1 brief adversarial review; #529/#530 refresh cycle | L4 doctrine | fenced |
| U-75 | Rolling gate hangs locally without credentials once a record and PR bind | #531 record act | L4 | open |
| U-76 | Governance gate's live-ruleset comparison is lossy at the top level | L1 brief adversarial review | L4 (cure is a Jon-ruled contract act) | open |

Amendment without reallocation: U-70 gains a confirmed reviewer-seat
instance (below). One seat-procedure lesson is routed to the corrections
ledger, not the register (see observations).

## U-73 — No sanctioned structured record format for records-class cross-family verdicts

- raised-by: Claude Fable 5, records seat (the #530 landing act, 2026-08-30)
- confirmed-by: the rolling gate's own validator output (mechanical), and
  the landed review record documenting it
- head: `7f77c0d66621630911ac73914816c3b9a7933abd` (the format gap and its
  documentation observable); demonstration lineage preserved on the
  superseded staging branch `codex/l1-reacceptance-redesign-brief` (tips
  `e639d525…` / `ac3bd1a3…`, never merged)
- command: commit a `garnet.trust_kernel_review_record/v2` file under the
  `F_Project_Management/W_TRUST/*.review.json` trigger suffix on a branch
  whose gate discovery reports `touched_paths: []` — the validator rejects
  it (empty touched set, null digest, null PR ids, transport required);
  removing it then trips "structured review record history is append-only",
  binding the lineage. Both outputs are quoted verbatim in
  `F_Project_Management/W_TRUST/L1_REACCEPTANCE_REDESIGN_BRIEF_REVIEW_2026-08-30.md`
  at this head.
- status: open
- disposition: The `.review.json` schema is reserved for trust-kernel
  changes with pull-request transport; a records-class cross-family verdict
  has no structured home and rides markdown per the Lane 2A / Lane 0
  precedent. Route L1 design input: the succession/effectiveness registry
  design is the natural place to define a records-class verdict record, and
  its adoption is a Jon-ruled contract act.

## U-74 — CI checks out the PR head SHA, not the merge ref

- raised-by: the L1 brief's ordering-cure adversarial review (2026-08-30);
  confirmed live by the landing arc
- confirmed-by: Claude Fable 5, records seat — landing exhibit recomputed
- head: `7f77c0d66621630911ac73914816c3b9a7933abd`; surface
  `.github/workflows/ci.yml` (checkout pins the event's head SHA)
- command: the landing exhibit — after the WV-6 truth cure landed on main
  as #531, the refreshed run at the sweep branch's pre-rebase tip
  (run `33350992456`, head `ae88164f…`) still failed the already-cured
  suite with `'partial' != 'accepted'`; rebasing the branch onto the cured
  main was the only propagation path, and both PRs then went green.
- status: fenced — deliberate exact-candidate property: CI tests the exact
  advertised head, never an ephemeral synthetic merge, pairing with the
  strict up-to-date policy that forces every candidate to contain the base
  it will land on.
- disposition: The consequence doctrine is now on record: a base-landed
  cure reaches a PR's CI only by rebasing the PR branch; merging any PR
  sets its siblings BEHIND under the strict policy, and the sanctioned
  refresh is rebase plus a fresh event, never a re-run. Route L4 doctrine:
  write this into the repository's procedural law surface.

## U-75 — Rolling gate hangs locally without credentials once a record and PR bind

- raised-by: Claude Fable 5, records seat (the #531 record act, 2026-08-30)
- confirmed-by: pending — L4 binds the exact stall site
- head: observable at any trust-kernel candidate with a committed review
  record and an open PR, run without a GitHub credential; observed twice at
  `53bc1644…` (the #531 record tip)
- command: `python3 -I scripts/garnet_trust_kernel_review_status.py
  --base <main> --head <record tip> --format json` with no credential
  available — no output and no exit within 100 seconds, terminated only by
  an external timeout; the same head in CI fails fast with a proper
  problems line. Before the record exists, the same invocation returns in
  seconds.
- status: open
- disposition: The authenticated-transport path lacks a bounded timeout and
  an offline fail-fast problem line, so the local diagnostic posture is a
  hang instead of a report — the exact failure mode the gate-message
  discipline (U-53's family) exists to prevent. Route L4: bounded transport
  timeout plus a deterministic offline problem line, with a red-capable
  fixture.

## U-76 — Governance gate's live-ruleset comparison is lossy at the top level

- raised-by: the L1 brief's class-boundary adversarial review (2026-08-30),
  verified in code during the cross-check
- confirmed-by: recorded in the landed brief's Class B "Live-projection
  limit" section at this head
- head: `7f77c0d66621630911ac73914816c3b9a7933abd`; surface
  `scripts/garnet_github_governance_gate.py` — the live object is projected
  to `RULESET_KEYS` before strict equality (line 602 at the reviewed
  boundary), while unknown-top-level-field rejection (line 560) applies
  only to the checked-in document
- command: inspect the two sites — a novel top-level field on the live
  ruleset object is discarded by the projection and produces zero
  divergence; nested values retained under `rules` remain strictly
  compared
- status: open
- disposition: A server-side serialization change expressed as a new
  top-level sibling key is invisible to the gate rather than RED.
  Exploitability depends on the forge; the fail-closed cure is to reject
  unknown live top-level keys or explicitly fence the projection as a
  documented boundary. The touched surface is `scripts/garnet_github_*`,
  which is human-merge-only by integrity rule: the cure is a Jon-ruled
  contract act, tracked under L4.

## U-70 — amendment: confirmed reviewer-seat instance

- prior entry: `F_Project_Management/W_TRUST/AUGUST_2026_ARC_REGISTER_SWEEP_2026-08-27.md`
  (fenced — active procedural covenant). That record is append-only and is
  not edited; this amendment extends its class evidence in place.
- amendment: the class "mechanics asserted without repository access" now
  has a confirmed cross-family-reviewer instance beyond the directing chat
  seat. During the L1 brief cross-check, the reviewing seat's adversarial
  pass asserted that cargo-deny parses `name@X.Y.Z` as a caret range,
  from general tool knowledge without recomputation. The premise was false:
  the implementing seat recomputed against the pinned parser, and the
  reviewing seat's own follow-up fixture confirmed exact-comparator
  semantics (a lower-version deny row does not match a later resolved
  version; the exact-match control bans; the canonical `name:=X.Y.Z`
  spelling parses and matches exactly). The correction is carried in the
  landed brief's Class A section and in the landing-arc review record.
- head: `7f77c0d66621630911ac73914816c3b9a7933abd`
- status: unchanged (fenced — active covenant); the cure in force —
  recompute before asserting — held: the false premise was caught before
  any record landed, by the opposite seat's recomputation.
- disposition: The register is corrected against the reviewer, on the
  record, per its own discipline. Cross-family review caught a
  cross-family reviewer: the symmetry is the system working.

## Reconciliation

- Candidates processed: 5 of 5 — 4 new allocations (U-73 through U-76),
  1 amendment (U-70), 0 backfills (no id found in circulation without an
  entry at this head).
- Distinct ids observable in the id space: 54 before this record, 58 after
  it. Recompute at this record's tip:
  `git grep -I -hoE 'U-[0-9]+' <tip> -- '*.md' '*.json' '*.txt' '*.py' | sort -u | wc -l`
- Collision gate at allocation time: no occurrence of U-73 or above in any
  of the 464 unique fork trees or at `origin/main`; no id appears twice in
  this register.
- Observations routed elsewhere, not registered: (a) a seat-procedure
  defect — a shell `set -e` chain failed to stop on a red gate before a
  push during the landing arc; cured procedurally by reading every gate
  output before any push in a separate step — belongs to the corrections
  ledger as a seat-error instance, not to the finding register; (b) a host
  environment permission lapse on the console directory mid-session is an
  operations note, not a repository finding.

This record lives under `F_Project_Management/W_TRUST/**`, which is both
product-digest-excluded and an enumerated record-class surface, so this tip
moves no frozen pair and buys no ceremony.
