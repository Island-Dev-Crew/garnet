# L1 re-acceptance redesign brief — cross-family review record (2026-08-30)

- Reviewing seat: Claude Fable 5, Pro reviewer lane, macOS, fresh clone per
  act; cross-family from the Codex implementer seat (L-15 satisfied).
- Verdict: **CONFIRM-WITH-FINDINGS, zero blockers**, bound to reviewed head
  `a47af84457f1e8b7ac4c35b483e88d437ee68e9c` (tree
  `90d8cf83844add1769198ff86029eaa1cf9b6175`), base
  `0607f7fe8770491bff3d16261628c27c570baa51` (the #528 squash, tip of main
  with zero later commits at review time). The verdict binds that exact head;
  this record commit is the sanctioned head move it anticipates.
- Scope: the design record only. No native check is re-executed, no product
  pair is promoted, and review coverage is not extended or backdated.

## Chain reviewed

- `f8af6cbae14d32df16f81aa19d4630cc66a66c81` — v1 design record.
- `530acd6982f752f5b9379f09106496d3b1bf1bb0` — v2 revision discharging all
  four v1 ratification blockers (terminal
  `garnet.wv_acceptance_effectiveness/v1` transcript with a
  no-receipt-for-the-receipt termination rule; Class A canonical
  `name:=X.Y.Z` exact-comparator predicate plus a machine-consumed
  expiry-event registry; Class C catalog predicate
  `CLASS_C_CATALOG_MATCH_V1` with construction-proof entry `C-RW-0001`; the
  R2 `approval_pending_only` machine state via a durable attempt-1 artifact
  with an exact `actions: read` permission delta) and nine folded
  non-blocking cures, including the two-clock per-edge/walk-scoped
  qualification rule that makes the #528 exhibit valid.
- `a47af84457f1e8b7ac4c35b483e88d437ee68e9c` — vocabulary cure; three
  substitutions; retired-word census at the tip is zero.

## Recomputed at the reviewed head

- Endpoint walk from base: exactly one path,
  `F_Project_Management/W_TRUST/REACCEPTANCE_REDESIGN_BRIEF_v2.md`
  (digest-excluded, record-class); neither frozen pair moves.
- WV-6 reporter output byte-identical to its output at main at every branch
  tip: `state=partial`, 5/5 required checks passing, findings exactly the raw
  pair `1d404df1…/1649` against the native-accepted pair `6f2d5f0b…/1646` at
  reviewed head `8426ca761c696c3556190be77cce3e340250b5c7` — a pre-existing
  condition the brief records truthfully and does not repair.
- Suites: focused WV 5/6 (the one pre-existing stale accepted-versus-partial
  expectation), content provenance 9/9, provenance seal chain 3/3, agent
  contracts 24.
- Document claims: base tree `50db668b…`; #528 PR-head
  `d9d6c163…` tree equals the squash tree; accepted head `8426ca76…` is its
  ancestor with a two-commit, fourteen-path, all-record-class walk; MSRV
  1.95; the cited rolling-review-contract and rulesets README regions carry
  the referenced law; both root toolchain files are absent as stated.
- Empirical, correcting the reviewer's own v1 finding: cargo-deny 0.19.6
  fixture shows `name@X.Y.Z` is one exact comparator (a deny row at a lower
  patch version does not match a later resolved version; the exact-version
  control bans; the canonical colon form `name:=X.Y.Z` parses and matches
  exactly). The v1 caret-range premise was false; the implementer's
  recomputation was right.
- #528 run `32543270060` verified live: attempt 1 exposed seven job rows;
  attempt 2 all nine expanded jobs with success; the truth job took fresh
  identity `96957159289` → `96959969901`, proving a full re-run; the
  attempt-2 `triggering_actor.id` equals reviewer id `306739987`, so the
  brief's statement that #528 fails the proposed `r2_role_separation_v1`
  predicate is accurate.

## Rolling-gate posture and record format

The rolling trust-kernel review status at base..reviewed-head reports
`touched_paths: []`, `content_digest: null`, `trust_kernel_touched: false`,
`changed_count: 1`, `problems: []`, exit 0 — this branch touches no
trust-kernel trigger and requires no `*.review.json` record.

Recorded mechanical fact: an attempt to carry this verdict as a
`garnet.trust_kernel_review_record/v2` file under the `*.review.json`
trigger suffix was rejected by the gate's own record validator with exactly:
`touched_paths must not be empty`; `content_digest must be sha256 followed
by 64 lowercase hex digits`; `pull request id must be a positive integer`;
`pull request number must be a positive integer`; `authenticated GitHub
review transport is required for a trust-kernel change`. That schema is
reserved for trust-kernel changes with pull-request transport; no sanctioned
structured format exists today for a records-class cross-family verdict.
This document is that verdict's record, following the Lane 2A / Lane 0
markdown review-record precedent, and the format gap is noted as a register
candidate for the next sweep.

## Findings, none blocking

1. Transport deviation: the branch was staged on `origin` after the fork
   rejected the push with `permission denied` — a deviation from the
   fork-staging topology; registered fleet item.
2. All branch commits author as the origin-write credential
   `jon-isaac@islanddevcrew.com` rather than a Tier-1 seat identity —
   registered fleet items pending the sweep-branch merge (U-64, U-65, U-69).
3. Four conservation predicates (`r1_review_scope_exact_v1`,
   `r2_role_separation_v1`, `r1_reporter_constant_projection_v1`,
   `r1_strict_equal_blob_identity_v1`) are `OPEN-UNTIL-IMPLEMENTED` by the
   brief's own declaration — activation blockers, not merge blockers, for
   this non-operative design record; the brief's conservation rule makes
   `OPEN-UNTIL-IMPLEMENTED` ineligible for activation, which is the correct
   fail-closed posture.

Ratification preconditions remain reserved to Jon in the brief's DECISION
POINTS, including DP7's requirement that the standing deny-policy row be
removed or re-ruled before Class A may be ratified. Implementation,
activation, approval, and merge are separate acts, none performed by the
reviewing seat.
