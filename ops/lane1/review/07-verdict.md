# Lane 1 Phase 0 — Independent Review Verdict 07 (Request 07: U-35 cure)

reviewer: Claude Fable 5 (`claude-fable-5`) — the seat this prompt designates
  as "Codex GPT-5.6 Sol" was executed by a Claude-family model this round.
  See IDENTITY DISCLOSURE below; this is recorded unsoftened.
reviewed_head: 7ad43855115103fdf2c08dddcb21cd6fd001334e
reviewed_tree: ad4335a036578e6e0e1d3577614091d88a261cef
request_07: ops/lane1/review/07-request.md (484f4620ce10657b946b2f567bc63f3432610600)
request_tip_tree: 1dac71dcb2230db5a096d11b9c0bcd2d140c849c
red_before_cure: 2f2377dff9a911b1e1b757e976794bb2930a9130 (evidence/92)
authorization: verdict 05 (db6ab65) — U-35 AUTHORIZE-WITH-CONSTRAINTS
swept_at: 2026-07-27T23:47:51Z
machine: Pulse's MacBook Air; Apple M5; macOS 26.5.1; Darwin arm64 (fanless —
  functional and byte-level legs only; no timing claims made or validated)
model: Claude Fable 5 (claude-fable-5)
implementer_self_report: Claude Code Opus 4.8 (request 07)
verdict: **APPROVE — U-35 cure at exact head 7ad4385 is correct, in-scope,
  and merge-durable**
approved_head_for_nuc: 7ad43855115103fdf2c08dddcb21cd6fd001334e
verified_identity: head/tree/diff/digests independently reproduced: yes
lineage: merge-base with origin/main (68317ae) is 68317ae; merges in range: 0;
  chain db6ab65 → 2f2377d → 7ad4385 → 484f462, all authored/committed
  Jon Isaac <Navigata1@gmail.com> (U-30 clean: no IDC-Trust-Review authorship)
differential: python battery 140 scripts at candidate AND merge-base — sole
  real delta is the standing WV-6 freeze red; cargo workspace 2199 passed /
  0 failed at rustc 1.95.0; zero Rust/Cargo bytes differ in range
security: APPLICABLE — examined (see SECURITY); no new exposure; S-SEC-1 carries

## IDENTITY DISCLOSURE (process caveat, stated before the ruling)

This verdict was produced by Claude Fable 5, not by Codex GPT-5.6 Sol. The
implementer of the cure is Claude Code (Opus 4.8). The standing rationale for
this reviewer seat — uncorrelated blind spots from a different model family —
is therefore NOT satisfied for this round. Everything below was re-derived
from the tree with reviewer-owned tooling (including a from-spec
reimplementation of the digest construction validated against five historical
anchors), and no implementer-reported number was accepted without independent
reproduction. The content ruling stands on that evidence. Jon (or the ceremony
seat) may nevertheless void this verdict and re-run request 07 through a true
cross-family reviewer at their discretion; if they do, this file records what
a same-family independent re-derivation found in the meantime. No prior
verdict file was modified; verdicts 01–05 carry their original attributions.

## Boot and truth floor

- Sabbath fence: not active (Monday 2026-07-27, America/Chicago).
- Fresh clone (no warm reuse), space-free path, non-sync-managed;
  `core.autocrlf=false` set globally BEFORE cloning.
- Remote order correct (U-34): `origin=Island-Dev-Crew/garnet`, then
  `fork=Navigata1/garnet`. No `refs/pull/*` fetched (count 0).
- `origin/main` = `68317ae258327aade47fc2c07b7b5b580ec7c6ea`. Boot UTC
  2026-07-27T23:32:26Z; Python 3.14.5.
- Truth floor on main, all PASS: Lane 0 closeout (22/22, ledger 37, 4/4);
  MSRV 1.95 (ok, 16/16 inheriting); frozen backlog (8 entries, findings []);
  trust-kernel review v2 (ok, base=head=68317ae, touched=false, problems []).

## Leg 1 — SCOPE: exactly the five authorized items, byte-verified

Whole-range diff `db6ab65..484f462` excluding `ops/lane1/` touches exactly
four files; per-file zero-context hunks are exactly:

1. `scripts/garnet_content_provenance.py`: ONE added line, `b"ops/lane1/",`
   appended to `FROZEN_MUTABLE_PREFIXES`. Literal prefix; the denied general
   predicate is absent (verified at source and adversarially, leg 2d).
2. `scripts/test_garnet_minimum_shelf_provenance.py`: pure additions
   (zero deleted lines) — the three U-35 traps plus the `cp` module load, in
   the existing provenance test surface.
3. `scripts/smoke_garnet_minimum_shelf.py`: ONLY
   `EXPECTED_PRODUCT_CONTENT_SHA256` → `e89cb299…` and
   `EXPECTED_PRODUCT_PATH_COUNT` → 1544.
4. `proofs/minimum-shelf/lane2b/PROOF.json`: ONLY the
   `productContentSha256` / `productPathCount` mirrors.

Item 5 (`ops/lane1/**`): RED evidence (2f2377d), request + BLOCKED/journal
(484f462) — nothing else in those commits.

Byte-untouched, confirmed: reporter logic; `REVIEWED_HEAD` (72ae024…);
`REVIEWED_TREE` (3c98ba05…); `REVIEWED_TREE_PRODUCT_SHA256` /
`reviewedTreeProductSha256` (1e669217…); `REVIEWED_TREE_PATH_COUNT` /
`reviewedTreePathCount` (1527); every other exclusion; `REPORTER_PATH`
(self-path) — and `git diff` of `.github/` (workflows, rulesets, actions)
against BOTH db6ab65 and origin/main is empty (0 lines). Cure commit
diff-stat is exactly 4 files, +61/−4, and `git diff --check` clean.

## Leg 2 — ALL FIVE TRAPS, re-executed by this reviewer

Reviewer-owned reimplementation: I rebuilt the documented construction
(`git --no-replace-objects ls-tree -r -z`, blob records, exclusion filter,
sort by raw path bytes, SHA-256 over `path NUL blob-OID LF`) from the spec,
NOT by importing the repository module, and validated it against five
historical anchors from verdict 05 — all reproduced exactly:
`14a5e456→48c63f8d/1578`, `72ae024→99c3f270/1578`, `f1ec569→5d3e7f72/1581`,
`ea1dcf63→9f483ce9/1582`, `35cf33e→726005d3/1584` (old predicate).

- **(a) product blob moves, lane1 does not** — committed test
  `test_included_product_change_moves_while_lane1_does_not` passes at
  7ad4385 (suite 6/6 OK): product-blob mutation moves the digest AND trips
  `_verify_product_content`; an `ops/lane1/`-only change does neither. PASS.
- **(b) lane1-only changes are inert** — committed test
  `test_lane1_review_artifacts_do_not_move_the_digest` passes; digest and
  count byte-identical after add+modify of only `ops/lane1/` paths. PASS.
- **(c) cure candidate vs LATER review-request tip (as corrected)** — my
  implementation, new predicate:
  `7ad4385 → e89cb299…/1544`; tree `ad4335a0 → e89cb299…/1544`;
  request tip `484f462 → e89cb299…/1544`. Identical digest AND count at the
  candidate and the later tip, independently reproducing the ceremony seat's
  pair with a third implementation. PASS.
- **(d) exact tuple; general predicate must FAIL the trap** — committed test
  asserts the tuple is exactly
  `(b"ops/lane2b/", b"proofs/", b"F_Project_Management/W_TRUST/",
  b"ops/lane1/")`, reporter self-path intact, `ops/lane1/…` excluded while
  `ops/lane3/…` is NOT. I verified the trap is behavioral, not merely a
  tuple transcription, by patching the test module's own `cp` reference
  (its private module copy — patching a separately loaded copy does NOT
  reach it) with two adversarial variants: V1 a full general
  `ops/<lane>/` implementation → trap FAILS (tuple + behavior asserts);
  V2 sneaky — literal 4-tuple kept but `_is_mutable` behaves generally →
  trap STILL FAILS on `assertFalse(_is_mutable(b"ops/lane3/note.txt"))`.
  Baseline passes before and after. Included-path set difference old→new at
  the cure tree: 42 paths, ALL under `ops/lane1/`, and zero paths in the
  other direction (see F1 for the request's "40"). PASS.
- **(e) final-pin rederivation** — recomputed at the exact committed cure
  tree: `e89cb299…/1544`; reporter `EXPECTED_PRODUCT_CONTENT_SHA256` /
  `EXPECTED_PRODUCT_PATH_COUNT` equal the pair; both PROOF.json mirrors
  equal the pair; the old `5d3e7f72…/1581` pair FAILS (the reporter's own
  `_verify_product_content` emits the digest-mismatch finding against
  `5d3e7f72…`, and 1581 ≠ 1544); Shelf gate at 7ad4385: exit 0, `ok:true`,
  state `accepted`, 5/5 checks, findings `[]`, digest/count exact;
  historical `1e669217…/1527` byte-identical in reporter AND proof. PASS.

## Leg 3 — Same-series mandate

The exclusion (`garnet_content_provenance.py`) and the rederived pair
(reporter + proof mirrors) are in ONE commit, 7ad4385. Shelf gate state per
commit in the series: db6ab65 RED, 2f2377d RED — that red is the standing
U-35 disease itself (pin `5d3e7f72…` vs review-inflated tree; the documented
condition of evidence/91/92, present since the artifacts above f1ec569
landed) — then 7ad4385 GREEN and 484f462 GREEN. The cure introduces NO new
red-reporter window: at no commit does the exclusion exist without the pair
or vice versa. SATISFIED.

## Leg 4 — RED before cure, re-executed

2f2377d (parent db6ab65) records evidence/92 BEFORE the cure. I re-executed
the red with my own hands: at the pre-cure predicate (2f2377d checkout) with
ONLY the cure's test bytes overlaid, the suite runs 6 tests with EXACTLY 3
failures, and they are precisely the three U-35 traps
(`…exactly_the_authorized_set`, `…moves_while_lane1_does_not`,
`…do_not_move_the_digest`). Post-cure the same suite is 6/6. VERIFIED.

## Leg 5 — Differential

Python battery: all 140 `scripts/test_*.py` run under `python3 -I` at BOTH
the candidate (7ad4385) and the merge-base (68317ae). Twelve failures are
IDENTICAL at both revisions (adoption_surface, cap_manifest_standard,
launch_readiness, linear_effect, provenance_seal_chain, release_assets,
required_context_evaluator, spawn_ffi_authority, and the four workflow_*
policy suites) — pre-existing on landed main, environment-dependent on this
seat, and cure-independent. Candidate-only deltas:

1. `test_garnet_wv_acceptance_status.py` 5/6 — the sole failure is the
   standing WV-6 accepted-vs-partial freeze red charged in verdicts
   02/04/05. This is the ONLY sanctioned new failure, and it is the only
   real one. WV-6 gate at 7ad4385: exit 1, `partial`, 5/5 checks, exactly
   the four expected stale-native-manifest findings (reviewedHead,
   reviewedTree, manifest digest, live digest vs old `2f8c9ad8…`) — cured
   only by the NUC run.
2. `test_garnet_trust_kernel_review_status.py` hit the batch runner's 180s
   timeout at the candidate. Re-run in isolation at the same checkout:
   110/110 OK in ~111s — a contention artifact of two parallel batteries
   (the suite spawns hundreds of git subprocesses), not a candidate defect.

Cargo parity: `git diff 68317ae..484f462` over `*.rs` and all Cargo
manifests/lockfile is EMPTY — parity holds at the byte level. Functional
confirmation: `cargo +1.95.0 test --workspace` at 7ad4385 — 2199 passed,
0 failed (host default rustc 1.94.1 is below the 1.95 MSRV and refuses the
build, which is the MSRV floor working as specified; the pinned 1.95.0
toolchain runs green).

Truth-floor gates at the candidate: Lane 0 closeout, MSRV, frozen backlog
all PASS. The trust-kernel review GATE is red at the candidate by design —
see F2.

## Leg 6 — Merge-time proof

**The regression that motivated U-35 is CURED at the squash boundary.** The
branch is linear atop 68317ae (= current origin/main head, merge-base
verified, zero merges), so Jon's squash lands the branch-TIP tree unchanged.
My independent digest of the tip tree (`1dac71dc…`) under the cured
predicate is `e89cb299…/1544` — byte-equal to the reporter pin and both
proof mirrors. The Shelf gate is green at the tip. Post-merge review
artifacts under `ops/lane1/` (including this verdict file and its heartbeat)
cannot move the digest — proven by trap (b) and by my own included-set delta
(zero non-lane1 paths differ between predicates). The squash-tip tree
therefore satisfies the pin, and WV-6's remaining `partial` is exclusively
the native-manifest staleness that only the NUC run can refresh. Boundary
condition, stated for the record: the pin binds tree CONTENT, so any future
product-byte commit on this branch (outside `ops/lane1/`) re-moves the
digest and re-opens the question — by design, not defect.

## Leg 7 — SECURITY (judged APPLICABLE, not defaulted)

`garnet_content_provenance.py` is a digest/authority-adjacent trust-kernel
surface and an exclusion change is a coverage-reduction change, so security
review applies. What I examined:

- Every cure hunk, manually (61 added lines; the only non-test production
  change is the one-line literal prefix and the constant/mirror rebinds).
- All 42 newly excluded paths, enumerated individually: every one is Lane 1
  bookkeeping (evidence, review requests/verdicts, journal, ledger, locks,
  state, BLOCKED) — no code, no workflow, no executable, no gate input.
- Consumers of `ops/lane1/` content outside the lane: exactly one,
  `garnet_governance_activation_ceremony.py`, whose use is a schema string
  comparison of an `evidence_destination` FIELD VALUE against a constant —
  it does not ingest lane1 bytes as trusted gate input. This matches the
  already-reviewed precedent of the shelf reporter reading excluded
  `ops/lane2b/` evidence (reported, not pinned).
- Importers of the provenance module: unchanged set (shelf reporter, WV
  acceptance status, the provenance test).
- The new trap tests operate exclusively on synthetic `tempfile` git
  fixtures with `--no-replace-objects`; no network, no repo mutation, no
  subprocess input from untrusted data.

Conclusion: the exclusion removes integrity coverage ONLY from lane
bookkeeping that no gate consumes as trusted input; digest coverage of all
product, gate, workflow, and kernel surfaces is unchanged. No injection,
tamper, or authority-escalation vector introduced. **S-SEC-1 carries
forward unchanged**: the broad security sweep across capability/authority
surfaces remains due before the Lane 4 frozen candidate and final red-team.

## Findings

- **F1 (NOTE, non-blocking):** Request 07 and the journal say "the 40
  `ops/lane1/` paths now leave the set." At the cure tree the old→new
  included-set delta is **42** paths (verdict 05 measured 40 at `35cf33e`;
  `05-verdict.md` at db6ab65 and `evidence/92` at 2f2377d joined the
  namespace afterward). Exact reproduction: reviewer set-difference of the
  two predicates at `7ad4385` — 42 paths, all `ops/lane1/`, reverse
  direction 0. The load-bearing claims (digest `e89cb299…`, count 1544) are
  exact; only the prose path-count is stale. No cure round warranted.
- **F2 (NOTE, non-blocking, expected-state):** the trust-kernel rolling
  review v2 diagnostic over `68317ae → 484f462` reports
  `trust_kernel_touched=true` (the branch touches
  `scripts/garnet_content_provenance.py`,
  `scripts/test_garnet_minimum_shelf_provenance.py`, W_TRUST markers,
  `docs/why.html`) with the structured review record missing, and the gate
  form is red at the candidate. This is the DESIGNED pre-record state: the
  ONE record commit and the IDC-Trust-Review approval at the record head are
  Jon-only slice 6–7 actions (BLOCKED.md). Stated here so the red is not
  misread as a cure defect. On main the same gate is green (base=head,
  untouched).

## Style advisories (non-blocking; no formatting round)

- **S-WS-2:** `ops/lane1/review/07-request.md` carries trailing whitespace
  on five lines (134, 138, 139, 144, 200). It is an immutable, digest-
  excluded review artifact; the cure commit itself is `git diff --check`
  clean. Same posture as S-WS-1: do not create a standalone formatting
  round.

## Scope, weakening, provenance

- Reviewed: the exact three-commit series 2f2377d → 7ad4385 → 484f462 atop
  verdict 05, against verdict 05's authorization, plus lineage to
  origin/main. Nothing beyond the authorized five items changed (leg 1).
- Weakening: the only coverage change is the authorized literal
  `b"ops/lane1/"` exclusion; the general predicate is absent and
  adversarially shown to fail trap (d). Product/gate/kernel coverage
  unchanged. This is the consistency correction verdict 05 authorized, not
  a new weakening.
- Provenance: raw reproduction used `git --no-replace-objects` throughout;
  no replacement refs, no pull refs; digests recomputed (never transcribed)
  with a reviewer-owned from-spec implementation cross-validated on five
  historical anchors; the shelf/WV/test/gate legs additionally exercised
  the repository's own implementation. Machine honesty: no timing claims.

## Not verified

- Native Windows WV-6 was not run; this macOS/arm64 Air is not the NUC.
  WV-6 remains `partial` with the four expected findings until the NUC
  regenerates evidence at the approved head.
- No timing or performance claim was made or validated (fanless seat).
- Clippy was not run; zero Rust bytes changed in range and the workspace
  test suite is green at 1.95.0.
- The twelve environment-dependent battery failures were not root-caused
  individually; they are byte-identical failures at merge-base and
  candidate and therefore outside this delta.
- No PR, merge, GitHub approval event, workflow, ruleset, implementation
  file, or `ops/mission/state.json` was created or modified by this
  reviewer. This verdict and one journal heartbeat line are the only bytes
  added, both under digest-excluded `ops/lane1/`.

## NUC consequence

**APPROVED HEAD FOR THE NUC:
`7ad43855115103fdf2c08dddcb21cd6fd001334e`** (tree `ad4335a0…`, product
digest `e89cb299…`, 1544 paths). BLOCKER #0 lifts. The Windows seat runs
slice 4 at exactly that head per the BLOCKED.md resume procedure, and the
digest STOP-check there remains mandatory: recompute via
`scripts/garnet_content_provenance.py` and STOP unless it reproduces
`e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f` / 1544.
Later `ops/lane1/`-only commits on the branch (the request, this verdict)
carry the identical product digest, but the approved checkout is the exact
head above. Slices 5–7 remain gated as recorded (U-31 before slice 5;
slices 6–7 Jon-only with the ONE record commit and IDC-Trust-Review
approval at the record head).
