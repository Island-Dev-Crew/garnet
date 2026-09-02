# Landing arc 2 — register sweep at `080da696` (records lane)

This register enumerates findings at exact Git boundaries. IDs were swept
across every advertised fork branch head, every origin branch head, and
`origin/main` before allocation; counts or a stale "next ID" are not
allocation authority. Records-class only: no freeze, no cross-family review,
per the SWEEP lane of the Prompt Console.

- Sweep seat: Claude Fable 5.1, Pro seat (records lane), macOS, fresh clone.
- Sweep date: 2026-09-01.
- Sweep head: `080da696e22d14780283560f007856fc69f0c642` — the #539 squash,
  tip of `origin/main` by remote readback, closing the second landing arc
  (#535 DECISION POINTS ruling → #536 U-62 cure → #537 and #539 public-copy
  precision → #538 L1 act 1 contract law).
- Authority for this act: Jon's standing delegation of 2026-09-01 ("full
  permissions and approvals to proceed and continue"), recorded here; every
  autonomous act performed under it is attributed in its own commit and
  merge record (agent, model, gate version) per the repository's integrity
  rules.

## Collision sweep

- swept-at: 2026-09-01, from the fresh clone at `080da696…`.
- source: 466 non-main fork branch heads (fork `main` excluded per the boot
  fence) deduplicated to 465 unique trees; every `origin` branch head; and
  `origin/main` at `080da696…` — each swept with
  `git grep -I -hoE 'U-[0-9]+' <tree>`. Zero `refs/pull/*`. No hand-listing.
- result: the distinct id space runs U-04 through U-76 with the historical
  gaps; no occurrence at or above U-77 exists in any swept tree. **U-77 is
  the next free id.** Distinct ids at the sweep head: 58.

## Allocation table

| id | title | provenance (act) | route | status |
|---|---|---|---|---|
| U-77 | WV acceptance evidence is manifest-trust, not re-execution | L6 adversarial packet; #537 review | gate-hardening (contract act) | open |
| U-78 | Unmapped Tier-1 seat identities cannot satisfy the v2 record's author binding | #534 record act | L2 fleet (identity rollout) | open |
| U-79 | Canonical-JSON record contract has no dedicated local check mode | #534 record act | L4 | open |
| U-80 | Parallel seats share one `gh` auth state per machine | #538 / #539 record acts | L2 fleet (credential isolation) | open |
| U-81 | R2 eligibility-artifact channel is not producer-authenticated | #538 cross-family review | L1 acts 2 and 4 | open |
| U-82 | Base-controlled composite green is unobservable pre-merge by construction | #534 / #536 / #538 firings | L9 evidence ruling (contract act) | open |
| U-83 | Four memory natives are caps-invisible | enforcement scope; L6 packet; #537 review; #539 | L8 product (stdlib rows) | open |

Amendments without reallocation: U-39, U-62, U-72 transition to `fixed`;
U-58, U-59, U-60, U-66, U-67, U-68 gain landed contract law; U-74 gains its
doctrine surface; U-75's mechanism is corrected against the verifier's
measurement; U-70 gains three instances. Every entry and amendment in this
record was recomputed by an independent verifier before push; four factual
discrepancies and one laundered premise were corrected on that evidence.

## U-77 — WV acceptance evidence is manifest-trust, not re-execution

- raised-by: L6 adversarial packet (Bravo and Recon seats, arbiter rank
  2026-09-01: "evidence replay never runs check.command"; "`--gate` can
  pass on a wrong state via re-pin")
- confirmed-by: Claude Fable 5, reviewer seat — code-level read 2026-09-01
- head: `080da696e22d14780283560f007856fc69f0c642`; surface
  `scripts/garnet_wv_acceptance_status.py:205` (`_validate_evidence`)
- command: `sed -n '205,393p' scripts/garnet_wv_acceptance_status.py |
  grep -cE 'subprocess|run\('` → 0 — the validator shape-checks the manifest
  (exact keys, `status == "passed"`, command is a bounded string, pins
  match) and never executes a check; a re-executing validator would
  introduce a subprocess call and turn this count nonzero
- status: open
- disposition: The native-Windows manifest is trusted, not replayed. The
  reporter binds `REVIEWED_HEAD` / `EXPECTED_PRODUCT_*` from
  `scripts/smoke_garnet_minimum_shelf.py` — which is NOT a trust-kernel
  trigger (it is `REPORTER_PATH`, mutable and digest-excluded in
  `scripts/garnet_content_provenance.py:32`, with no CODEOWNERS row). The
  defense in depth is the required-CI test
  `scripts/test_garnet_wv_acceptance_status.py` (`scripts/test_garnet_`
  prefix, `ci.yml:54`): it asserts WV-6 `partial` / `ok: False` / a digest
  mismatch, so flipping the gate green by re-pinning reds that test, and
  changing the test trips the distinct-principal review gate
  (`garnet_trust_kernel_review_status.py:1064`; cross-family symmetry is
  DP10 policy on top of it). Not a silent bypass, but a single-layer one:
  the pins file itself can move without review. The L6 arbiter's `--gate`
  re-pin experiment remains the next smallest test for the gate-hardening
  lane. Cure shape: add the reporter pins file to the trust-kernel file
  list, plus a re-pin fixture that reds when pins move without a native
  transcript.

## U-78 — Unmapped Tier-1 seat identities cannot satisfy the v2 record's author binding

- raised-by: Claude Fable 5, records seat (the #534 record act, 2026-08-31)
- confirmed-by: GitHub API — both commit roles on the original #534 lineage
  mapped to no principal
- head: `080da696e22d14780283560f007856fc69f0c642`; validator
  `scripts/garnet_trust_kernel_review_status.py:1029-1058`
- command: the validator requires `author_emails` to equal the walk-derived
  set (line 1726) AND `author_ids` to be a nonempty sorted list of
  authenticated principal ids (lines 1048-1056); for a lineage authored as
  `codex-pro@seats.idc.invalid` the derived email set is unmapped, so no
  record can satisfy both — the #534 lineage was re-authored to a mapped
  principal (trees byte-identical) as the cure of record
- status: open
- disposition: The Tier-1 seat-identity scheme (U-69) and the record schema
  are incompatible as designed: seat identities must be GitHub-mapped
  accounts, or the schema needs a ruled amendment. Route L2 fleet — the
  identity rollout should mint mapped accounts for the NUC and Air seats and
  the distinct Actions-write carrier that `r2_role_separation_v1` requires.

## U-79 — Canonical-JSON record contract has no dedicated local check mode

- raised-by: Claude Fable 5, records seat (the #534 record act — a record
  serialized with ASCII escapes burned one CI firing with "review record
  must use canonical JSON")
- confirmed-by: the validator's own construction at
  `scripts/garnet_trust_kernel_review_status.py:921`
  (`json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"`)
- head: `080da696e22d14780283560f007856fc69f0c642`
- command: `sed -n '921p' scripts/garnet_trust_kernel_review_status.py` —
  the contract lives inside the gate; the diagnostic `--base/--head` mode
  does surface "review record must use canonical JSON" locally, but only
  for a record already committed (`_load_tip_review_record` reads blobs
  from the head commit, line 1899); `main()` (lines 2642-2666) offers no
  per-file `--check-record <path>` mode and no pre-commit hint exists
- status: open
- disposition: Every record authored since has been hand-verified against a
  reimplementation of line 921, or by committing first and running the
  diagnostic mode. Route L4: a per-file `--check-record <path>` mode on the
  gate (AGENTS.md now carries the line-921 one-liner as prose, added by
  #538) so an uncommitted record is checkable before a firing is spent.

## U-80 — Parallel seats share one `gh` auth state per machine

- raised-by: Claude Fable 5, records seat; corroborated by the Codex seat
- confirmed-by: three exhibits on 2026-09-01 — the #538 record push denied
  as `Navigata1` (403) after a parallel seat switched the active account;
  the #539 v2 record push denied identically from the Codex seat, which
  correctly refused to switch credentials and stranded its record locally;
  a second flip during the #539 record transport
- head: `080da696e22d14780283560f007856fc69f0c642` (fleet condition, not a
  tree fact)
- command: `gh auth status` on the shared machine lists both accounts; the
  active account is a single mutable global that any seat's
  `gh auth switch` changes for every other seat
- status: open
- disposition: One machine, one `gh` active account, many seats — every
  auth switch is a race against every other seat's next push. Route L2
  fleet: per-seat credential isolation (separate `GH_CONFIG_DIR` / token
  env per seat, or mapped seat accounts with their own tokens), which the
  identity rollout (U-69, U-78) should carry.

## U-81 — R2 eligibility-artifact channel is not producer-authenticated

- raised-by: the #538 law skeptic (adversarial review, 2026-09-01);
  recorded non-blocking in
  `F_Project_Management/W_TRUST/L1_ARC1_CONTRACT_LAW.review.json`
- confirmed-by: Claude Fable 5, reviewer seat
- head: `080da696e22d14780283560f007856fc69f0c642`; surface
  `C_Language_Specification/GARNET_WV_ACCEPTANCE_SUCCESSION_CONTRACT.md`
  (eligibility receipt channel) and the R2 block in
  `GARNET_TRUST_KERNEL_ROLLING_REVIEW.md`
- command: artifact upload inside a workflow run uses the run-scoped
  runtime token, which every attempt-1 job holds — including jobs that
  execute candidate test code; a forged `approval_pending_only` receipt (or
  a name-squat that makes the real gate's upload fail) passes the
  exactly-one-artifact check
- status: open
- disposition: It cannot mint a false green (attempt 2 is a complete fresh
  re-evaluation) but it can widen rerun eligibility beyond the
  approval-absent confinement. Route L1 acts 2 and 4: bind the receipt to
  its producing job via the jobs API (job id and step) and reject receipts
  from any other job.

## U-82 — Base-controlled composite green is unobservable pre-merge by construction

- raised-by: Claude Fable 5, reviewer seat — recorded in the review_scope of
  `F_Project_Management/W_TRUST/L5_PUBLIC_TRUTH_BATCH.review.json` (#534)
- confirmed-by: nine Base-controlled firings since #533 landed — four on
  #534 (run numbers 41-44), one on #536, four on #538 (runs 49, 51, 52, 54)
  — every one with `candidate_policy_ok: true` and
  `rolling_review_ok: false`. The approval-timing exhibits are #534 and
  #538 (record and approval landed after the last firing); #536's red is
  record absence (non-trust-kernel, no record or approval at all), which
  evidences the policy-layer half only
- head: `080da696e22d14780283560f007856fc69f0c642`; surface
  `.github/workflows/base-controlled-trust.yml` (`pull_request_target`,
  checkout at `github.event.pull_request.base.sha`)
- command: `gh api 'repos/Island-Dev-Crew/garnet/actions/runs?head_sha=<record
  tip>'` filtered to the Base-controlled workflow → its last run predates
  the carrier approval and nothing re-fires it afterward; the composite
  therefore reports the pre-approval red on every ceremonially clean trust
  PR
- status: open — Jon's activation-evidence ruling
- disposition: The U-39 cure is observed at the policy layer on all nine
  firings; on trust-kernel PRs the composite's rolling-review axis is
  approval-timing by design. L9's
  activation evidence must either be defined as policy-layer green plus the
  separately verified record/approval chain, or the workflow must gain a
  post-approval trigger in a contract act. Neither is a defect in the cure.

## U-83 — Four memory natives are caps-invisible

- raised-by: the enforcement scope's own caps-invisible row
  (`C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md`:
  "Host-visible natives with no capability row at all. Any 'all authority
  is capability-tagged' claim is false until these earn rows"); surfaced
  as a public-copy constraint by the L6 packet and the #537/#539 reviews
- confirmed-by: OpenAI Codex (the #537 review grounded its why.html
  297-298 kill on this row); Claude Fable 5 (the #539 cures scoped every
  universal to it)
- head: `080da696e22d14780283560f007856fc69f0c642`; surface `BRIDGE_ONLY`
  at `garnet-interp-v0.3/src/stdlib_bridge.rs:81` (the registry source of
  truth `garnet-stdlib/src/registry.rs` has no `memory` row)
- command: `grep -n "Caps-invisible" C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md`
  and `grep -n "BRIDGE_ONLY" garnet-interp-v0.3/src/stdlib_bridge.rs` →
  lines 81-86 list exactly the four `memory::*` keys, and the test
  `bridge_only_list_is_exact` (same file, ~1493) pins `len() == 4`; the
  four natives `memory::working` / `episodic` / `semantic` / `procedural`
  reach host authority with no capability row
- status: open
- disposition: The public surface now names this boundary instead of
  claiming past it (#539). The product cure is capability rows for the four
  natives, after which the copy's "registered surface" qualifiers can
  narrow again. Route L8 (stdlib / shelf work).

## Amendments

### U-39 — status: open → fixed

- fixed at `f6d3285aa54a4961e38d82d22cfe98ab4c631b22` (#533): identity-stable
  loading — the schema module's own `yaml_policy` object is passed into
  snapshot construction; every nominal predicate unchanged; cross-family
  CONFIRM, record `U39_ADAPTER_IDENTITY_CURE.review.json`.
- observed: `candidate_policy_ok: true` on the base-owned reporter's firings
  for #534 (twice), #536, and #538 — the twelve-workflow-root cascade is
  gone on every candidate since.
- command: `gh api 'repos/Island-Dev-Crew/garnet/actions/runs?head_sha=<any
  post-#533 PR head>'` → Base-controlled payload `candidate_policy_ok` true.

### U-62 — status: open → fixed

- fixed at `a8a66fcb4483ed38a2198ebe041a88ed65dd30e1` (#536):
  `present_evidence_heading` returns the supported heading at the lowest
  document offset; both heading-order fixtures added; invariant recorded in
  `AGENTS.md`. Non-trust-kernel (the checker does not match the
  `scripts/garnet_` prefix), merged on the required greens with no ceremony.

### U-72 — status: open → fixed

- fixed at `90fdaa1235a4ed7201b27cd4256e3677c41e6aa2` (#534): the README
  carries the origin record — DHH on The Pragmatic Engineer, "DHH's new way
  of writing code", 2026-04-08; loop closed via Lex Fridman #501,
  2026-08-26 — with the register deep link.
- command: `git grep -c "Pragmatic Engineer" HEAD -- README.md` → nonzero
  (the entry's original grep, now green).

### U-58, U-59, U-60, U-66, U-67, U-68 — contract law landed

- `006eee51f62ac8461b3f85865c8181ae2bd11275` (#538, L1 act 1 of 5) lands
  the succession contract law that these findings demanded: eight adopted
  blocks byte-exact to the brief, six interpretations ratified, the U-66
  companion naming U-59 as the sole exception. Statuses are unchanged (the
  law is text; every predicate remains OPEN-UNTIL-IMPLEMENTED until acts
  2 through 5), but the route for each now reads "L1 acts 2–5".

### U-74 — doctrine surface landed

- `AGENTS.md` (#538) now states the rebase-propagation doctrine in
  procedural law; the finding stays fenced (deliberate exact-candidate
  property). Exhibits since allocation: #538 and #539 each required a
  rebase after a sibling merge.

### U-75 — mechanism refined, and a laundered premise corrected

- The L4 lane's implementer diagnosis (Codex seat, 2026-09-01) asserted that
  the bare no-token invocation fails closed in about 0.09 s and that the
  unbounded surface is the authenticated transport sequence. This sweep's
  independent verifier falsified that: with no credential the gate constructs
  no transport at all (`_explicit_github_transport` returns none without
  `--github-token-stdin`, `scripts/garnet_trust_kernel_review_status.py:2606`;
  `_authenticated_review_findings` returns immediately, `:1450`), and the
  stall is local git-object traversal — one `git` subprocess per object
  (`_git_bytes` `:275`, `_raw_object` `:481`; about 8,548 `subprocess.run`
  calls at the #538 record tip at ~14 ms each), measured at 117.8 s to a
  fail-closed problem line, and 58–72 s even at a record-free tip. The
  records seat then reproduced the class itself before push rather than
  transcribe the verifier's figure: the two-record range
  `37dd4de2..080da696` on main, no credential, a PATH-shimmed `git` counter —
  75.6 s, 4,278 `git` spawns, fail-closed on `exactly one structured review
  record is required`, no transport constructed. The
  original premise (no output for over 100 s with no credential at a record
  tip) stands; the "0.09 s" correction did not reproduce and is retracted
  here before it could route the cure at the wrong layer. Cure target:
  batch the object reads (`git cat-file --batch`) or bound the traversal,
  with the transport timeout as a second, separate cure for the credentialed
  path. Status open; route L4 (its own single-purpose PR, since the gate is
  the surface).

### U-70 — three more instances, every seat role now represented

- Reviewer seat (Claude Fable 5): kept four public claims as defensible
  without applying the enforcement scope's caps-invisible row, corrected by
  the implementer's #537 review; predicted a Base-controlled "flip green"
  at the candidate without reading the workflow's base-checkout, corrected
  by the implementer's stop; wrote a composite-check stop instruction too
  coarse for a composite, corrected on the #534 firing.
- Adversarial seat (Grok arbiter, L6): over-ranked six defensible claims as
  kills, corrected by recomputation against the enforcement scope.
- The class's cure held every time: the opposite seat's recomputation
  caught each instance before any false record landed. Status unchanged
  (fenced — active covenant).

## Reconciliation

- Candidates processed: 7 new allocations (U-77 through U-83), 0 backfills,
  10 amendments (three status transitions to fixed, six law-landed notes,
  one doctrine-surface note, one mechanism correction, one class-evidence
  extension).
- Distinct U- ids observable in the id space: 58 before this record, 65
  after it. Recompute at this record's tip:
  `git grep -I -hoE 'U-[0-9]+' <tip> -- '*.md' '*.json' '*.txt' '*.py' | sort -u | wc -l`
- Collision gate at allocation time: no occurrence of U-77 or above in any
  of the 465 unique fork trees, any origin branch head, or `origin/main`;
  no id appears twice in this register.

This record lives under `F_Project_Management/W_TRUST/**`, which is both
product-digest-excluded and an enumerated record-class surface, so this tip
moves no frozen pair and buys no ceremony.
