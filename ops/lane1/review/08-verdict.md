# Lane 1 Phase 0 — Independent Cross-Family Review Verdict 08

request: Request 07 — U-35 cure
reviewer: Codex GPT-5.6 Sol (`gpt-5.6-sol`)
review_family: OpenAI Codex — cross-family from the Claude implementer
reviewed_head: `7ad43855115103fdf2c08dddcb21cd6fd001334e`
reviewed_tree: `ad4335a036578e6e0e1d3577614091d88a261cef`
request_07: `ops/lane1/review/07-request.md` at `484f4620ce10657b946b2f567bc63f3432610600`
red_before_cure: `2f2377dff9a911b1e1b757e976794bb2930a9130` (evidence 92)
authorization: Verdict 05 (`db6ab65`) — AUTHORIZE-WITH-CONSTRAINTS
same_family_corroboration: Verdict 07 (`173e822`) — Claude Fable 5
swept_at: `2026-07-28T02:00:04Z`
machine: `Pulses-MacBook-Air.local`; Darwin 25.5.0; arm64
verdict: **APPROVE — the U-35 cure at exact head `7ad4385` is correct,
in-scope, security-reviewed, and merge-durable**
approved_head_for_nuc: `7ad43855115103fdf2c08dddcb21cd6fd001334e`
verdict_of_record: **YES — this is the cross-family verdict of record;
Verdict 07 remains same-family corroborating evidence**

## Executive ruling

U-35 is cured at the squash boundary. The candidate adds exactly the
Verdict-05-authorized literal `b"ops/lane1/"` exclusion, binds the independently
re-derived product pair `e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f`
/ `1544` in the same commit, preserves the historical reviewed-tree anchors,
passes all five required traps, and introduces no reportable security finding.

**APPROVED HEAD FOR THE NUC:
`7ad43855115103fdf2c08dddcb21cd6fd001334e`.**

This approval names the cure commit, not the later review-artifact tip. The
later Request 07 and Verdict 07 commits are product-digest-inert by the behavior
this review independently proved.

## Reviewer identity and independence

- This review was performed by Codex GPT-5.6 Sol, satisfying the standing
  cross-family reviewer requirement relative to the Claude Code implementer.
- The fresh review clone used organization `Island-Dev-Crew/garnet` as
  `origin` and `Navigata1/garnet` as `fork`; no pull-request refs were fetched.
- `core.autocrlf=false` was set globally before the cold clone.
- Commit authorship for the reviewed series is Jon Isaac
  `<Navigata1@gmail.com>`. No reviewed commit is authored or committed by
  `IDC-Trust-Review`.
- Sabbath fence was not active: the review began Monday 2026-07-27 in
  America/Chicago.
- Verdict 07 was not read until this reviewer had completed legs 1–7 and
  reached an independent conclusion.

## Boot and truth floor

Cold-start facts:

- `origin/main`: `68317ae258327aade47fc2c07b7b5b580ec7c6ea`
- branch review tip before this verdict: `173e822ac00f6291b56addb28cd3a30a8b277d69`
- reviewed cure head: `7ad43855115103fdf2c08dddcb21cd6fd001334e`
- reviewed cure tree: `ad4335a036578e6e0e1d3577614091d88a261cef`
- merge-base with `origin/main`: exactly `68317ae258327aade47fc2c07b7b5b580ec7c6ea`
- merge commits in the candidate range: zero

Truth-floor results on landed main:

- Lane 0 closeout: PASS; 22/22 evidence rows, ledger 37, denominator 4/4;
  launch remains HOLD.
- MSRV: PASS at Rust 1.95; 16/16 workspace members inherit the floor.
- Frozen backlog: PASS; eight entries.
- Trust-kernel rolling status: PASS on unchanged main.

These results establish review prerequisites. They do not convert this verdict
into launch authority.

## Leg 1 — Exact authorized scope

The cure commit is four files, `+61/-4`:

1. `scripts/garnet_content_provenance.py`
   - adds only `b"ops/lane1/",` to `FROZEN_MUTABLE_PREFIXES`;
   - does not add a general `ops/` or `ops/<lane>/` predicate;
   - leaves `REPORTER_PATH` and the three historical exclusions intact.
2. `scripts/test_garnet_minimum_shelf_provenance.py`
   - adds the three focused U-35 regression tests plus the provenance-module
     load needed by those tests;
   - contains no production behavior.
3. `scripts/smoke_garnet_minimum_shelf.py`
   - changes only `EXPECTED_PRODUCT_CONTENT_SHA256` and
     `EXPECTED_PRODUCT_PATH_COUNT`.
4. `proofs/minimum-shelf/lane2b/PROOF.json`
   - changes only the matching `productContentSha256` and
     `productPathCount` mirrors.

The RED evidence, request, journal, BLOCKED state, and earlier verdict are
confined to `ops/lane1/**`.

Byte-identity checks confirmed no changes to:

- `.github/` workflows, actions, or ruleset material;
- `REVIEWED_HEAD = 72ae0246fb448ce33d689b1b80eb783497a7f215`;
- `REVIEWED_TREE = 3c98ba05eb756377049325942842164f5d98910b`;
- `REVIEWED_TREE_PRODUCT_SHA256 =
  1e6692175ea8fe2dd5b04fad4a492dc8ce48767dd07d88fd11a0847ce96749d5`;
- `REVIEWED_TREE_PATH_COUNT = 1527`;
- the reporter self-path exclusion.

**Leg 1: PASS.**

## Leg 2 — All five required traps

The reviewer independently implemented the documented construction from
specification rather than trusting the repository module:

`sorted raw path bytes -> path + NUL + blob OID + LF -> SHA-256`

### Trap A — included product bytes still move the digest

A controlled included-product mutation moved the product digest to
`2148f914…` at the same path count, made the repository verifier red, and
made the Minimum Shelf pin gate exit non-zero.

**PASS.**

### Trap B — Lane 1 review-only changes are inert

In a throwaway clone, adding and modifying only `ops/lane1/**` left the
digest/count byte-identical at `e89cb299…/1544`; the Shelf gate remained
green.

**PASS.**

### Trap C — candidate and later review tip are identical

The independent construction produced the same pair at:

- cure head `7ad4385`: `e89cb299…/1544`;
- Request 07 commit `484f462`: `e89cb299…/1544`;
- same-family verdict tip `173e822`: `e89cb299…/1544`.

This is the corrected Verdict-05 formulation: compare the final cure/rebind
candidate with its later review-artifact tip.

**PASS.**

### Trap D — exact tuple; generalized predicate is rejected

The frozen tuple is exactly:

```text
b"ops/lane2b/"
b"proofs/"
b"F_Project_Management/W_TRUST/"
b"ops/lane1/"
```

The concrete behavioral probe confirmed that a general `ops/` predicate
would additionally remove 79 unauthorized paths, including
`ops/lane0/AUDIT.md`. A variant that retained the literal tuple but generalized
runtime behavior was also rejected by the sibling-lane assertion.

**PASS.**

### Trap E — final pin is independently re-derived

- independent candidate pair: `e89cb299…/1544`;
- repository module pair: `e89cb299…/1544`;
- reporter expected pair: `e89cb299…/1544`;
- both proof mirrors: `e89cb299…/1544`;
- old `5d3e7f72…/1581` pair: rejected;
- Minimum Shelf gate: exit 0, accepted, 5/5 checks, no findings;
- historical reviewed-tree pair `1e669217…/1527`: unchanged.

**PASS.**

## Leg 3 — Same-series requirement

The one behavioral exclusion line and the re-derived reporter/proof pair are
atomic in commit `7ad4385`. There is no implementation commit where the
exclusion exists without the matching pair or the pair exists without the
exclusion.

The parent is intentionally red because it contains the pre-cure U-35
disease. That pre-existing RED is not a cure-introduced reporter window.

**Leg 3: PASS.**

## Leg 4 — RED before cure

The RED evidence commit `2f2377d` precedes the cure. With the old predicate,
a Lane-1-only change moves the product pair from `8296cb9…/1586` to
`229e…/1587`, and the three new U-35 traps fail. With the cure predicate,
those traps pass.

**Leg 4: PASS.**

## Leg 5 — Differential verification

### Focused and gate results

- provenance suite: 6/6 PASS;
- Minimum Shelf gate: PASS, 5/5 checks, no findings;
- WV acceptance suite: 5/6, with only the standing WV-6
  accepted-versus-partial freeze assertion red;
- WV-6 gate: expected `partial`, all 5 checks present, exactly four stale
  native-manifest findings;
- trust-kernel unit suite: 110/110 PASS;
- trust-kernel gate at candidate: expected pre-record red because the
  structured cross-family review record did not yet exist.

### Full Python differential

Using a disposable, hash-pinned PyYAML 6.0.3 environment:

- merge-base: 1123 tests; 6 failures; 0 errors; 5 skipped;
- candidate: 1126 tests; 7 failures; 0 errors; 5 skipped.

The six baseline failures are identical. The sole candidate-only failure is
the standing WV-6 freeze red. The three added U-35 traps pass.

An initial system-Python run produced four equal PyYAML import errors on both
sides; the pinned disposable environment removed that environmental noise.

### Rust differential

At Rust 1.95 with `cargo test --workspace --no-fail-fast`:

- merge-base: 2199 passed, 0 failed, 6 ignored;
- candidate: 2199 passed, 0 failed, 6 ignored.

No Rust or Cargo bytes changed in the reviewed range.

**Leg 5: PASS.**

## Leg 6 — Merge-time proof

The cure is content-bound rather than commit-ID-bound:

1. the branch is linear atop current `origin/main`;
2. a squash lands the branch-tip tree content;
3. all later review-only bytes live under the newly authorized literal
   `ops/lane1/` exclusion;
4. the independently computed product pair is identical at the cure,
   Request 07, and Verdict 07 tips;
5. the reporter and proof mirrors bind that same pair;
6. any later included product-byte change still moves the digest and makes
   the gate red.

Therefore later review artifacts cannot recreate the U-35 pin treadmill, and
the squash-tip content satisfies the frozen pair.

**Leg 6: PASS — the motivating regression is cured at the merge boundary.**

## Leg 7 — Security

Security review was treated as applicable because the change narrows integrity
coverage at a trust/provenance boundary.

A formal Codex Security diff scan covered all four changed files in full:

- scan ID: `a86896f6-2e02-4ac0-b016-438df1033183`;
- range: `2f2377d…7ad4385`;
- immutable diff snapshot:
  `codex-security-snapshot/v1:sha256:119093a50a62f33f06494be227befcee4b79cbf52c768ba69acd932be89b91df`;
- coverage: complete, four of four worklist rows closed;
- reportable findings: zero;
- sealed at: `2026-07-28T01:59:25.351274Z`.

The scan specifically examined exclusion broadening, path parsing, blob and
symlink handling, verifier/pin bypass, reporter self-path integrity, and
workflow/ruleset byte identity. No plausible candidate survived discovery.

The independent included-set comparison found 42 newly excluded paths, every
one under `ops/lane1/`, with zero reverse-direction changes. No product,
workflow, ruleset, or trust-kernel code is hidden by the new prefix.

**Leg 7: PASS. S-SEC-1 carries forward unchanged as the broader pre-Lane-4
security sweep; it is not evidence of a defect in this cure.**

## Findings

### F1 — stale path-count prose (NOTE, non-blocking)

Request 07 says 40 `ops/lane1/` paths leave the included set. At the exact
cure tree the actual old-to-new delta is **42**, all under `ops/lane1/`.
The extra two are later Lane-1 artifacts that entered after the earlier
40-path measurement.

The load-bearing digest and included-path count are correct:
`e89cb299…/1544`. This is stale explanatory prose, not a cure defect.

### F2 — trust-kernel missing-record red (NOTE, expected state)

The candidate touches a trust-kernel-adjacent provenance module, so the
rolling review gate correctly remains red until the structured cross-family
record exists at the record head. Before this Verdict 08 commit, the exact
diagnostic was that the structured review record was missing.

That is the designed pre-record state, not a failed U-35 leg. The ceremony
must still follow the recorded Jon-only sequence and bind any approval to the
correct record head.

## Reconciliation with Verdict 07

After locking the independent conclusions above, this reviewer read
`ops/lane1/review/07-verdict.md`.

### Agreements

Verdicts 07 and 08 agree on every load-bearing point:

- exact authorized four-file cure plus Lane-1 artifacts;
- exact literal prefix and rejection of a generalized `ops/` predicate;
- all five required traps;
- `e89cb299…/1544` at the cure and later review tips;
- 42, not 40, newly excluded Lane-1 paths;
- same-series atomicity;
- authentic RED before cure;
- sole candidate differential red is the standing WV-6 freeze;
- 2199/0 Cargo result at Rust 1.95;
- trust-kernel gate red is the designed missing-record state;
- no new security exposure;
- the squash-boundary regression is cured;
- exact NUC head is `7ad4385`.

### Differences

- Verdict 07 disclosed that its reviewer was Claude Fable 5, so it could not
  satisfy the requested cross-family diversity premise. Verdict 08 does.
- The two reviews used different Python battery harnesses and environment
  normalization. Their raw baseline-failure totals differ, but both
  independently reach the same differential conclusion: the only real
  candidate delta is the standing WV-6 freeze red, while the new traps pass.
- Verdict 08 adds a sealed Codex Security scan with a canonical diff snapshot
  and four full-file coverage receipts.

No evidentiary disagreement requires a cure round. Verdict 07 remains valuable
same-family corroboration; Verdict 08 is the cross-family verdict of record.

## Strengths to preserve

- The cure is one literal policy line, not a new abstraction.
- The predicate remains exact and auditable.
- The re-derived pair is atomic with the policy change.
- Tests prove both sides of the boundary rather than checking only tuple text.
- Historical reviewed-tree anchors remain immutable.
- Review artifacts can accumulate without weakening included product-byte
  sensitivity.
- The design continues to fail closed for later product changes.

## Not verified and remaining gates

- Native Windows WV-6 was not run on this macOS reviewer seat.
- The NUC refresh itself has not yet occurred.
- No timing or performance claim was made.
- The six shared Python baseline failures were differentially classified but
  not repaired or individually root-caused; they are outside this cure.
- No PR was opened or merged, no GitHub approval was submitted, no workflow or
  ruleset was edited, no credential was used, and
  `ops/mission/state.json` was not touched.
- S-SEC-1, U-31, U-32, U-33, and all later ceremony gates retain their
  previously recorded force.

## NUC consequence

**APPROVED HEAD FOR THE NUC:
`7ad43855115103fdf2c08dddcb21cd6fd001334e`**

BLOCKER #0 lifts for the native Windows slice at exactly that checkout. Before
producing refreshed evidence, the NUC must recompute and STOP unless it gets:

```text
productContentSha256 =
e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f
productPathCount = 1544
```

Approval of this head does not approve launch, later slices, a merge, or a
GitHub approval event. Those remain governed by the standing sequence and
their own evidence.

## Reviewer stdout summary

Cross-family Verdict 08 APPROVES U-35 at exact NUC head
`7ad43855115103fdf2c08dddcb21cd6fd001334e`: scope is exact, all five traps
pass, the final pair independently reproduces as `e89cb299…/1544` through
later review-only tips, differential testing isolates only the standing WV-6
freeze red, the sealed four-file security diff scan reports zero findings, and
the squash-boundary regression is cured. F1 corrects stale prose from 40 to
42 excluded Lane-1 paths; F2 records the expected pre-record trust-kernel red.
Verdict 07 agrees on all load-bearing evidence but is same-family; Verdict 08
is the cross-family verdict of record.
