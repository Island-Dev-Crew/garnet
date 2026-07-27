# Lane 1 · Phase 0 — Reconciliation Unblock · Review Request 01

- Date: 2026-07-27 (UTC ~13:00Z)
- Implementer: Claude Code, **Opus 4.8** — macOS Darwin arm64 (Hughs-MacBook-Pro), fresh `autocrlf=false` clone
- Independent reviewer sought: **Codex GPT-5.6 Sol** (different machine, different model family)
- Merge authority: Jon (IslandDevCrew) only. Review carrier: IDC-Trust-Review only. Implementer is neither.

## Frozen reviewed candidate

| field | value |
|-------|-------|
| **frozen head** | `d7430c285fa8620dcf1f0c1cd94e5cc44b98d180` |
| **frozen tree** | `2e8ce5dedfe88d67e8cc4bfa2527591ffcc5f3a8` |
| reviewed base (origin/main) | `68317ae258327aade47fc2c07b7b5b580ec7c6ea` (#517 atop #514 `41d6ced`) |
| **product_content_sha256** | `c4b3cf7cea369a4003336b62b97a30a369be8063002cf4634c320bd6e027cb64` |
| product path count | 1572 |
| certified (pristine) digest | `2f8c9ad860bd9c6dbd1e005b0c82af0288dd3a736bb416a3978708c12e6fa1fd` (moved — expected, U-29) |

This request commit is a **review-artifact above the frozen candidate** `d7430c2`. The candidate for review, WV-6 acceptance, and the eventual record is `d7430c2` — not this commit.

## Diffstat vs `68317ae` (5 paths)

```
F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json          (registry: [] -> 2 sorted marker paths)
F_Project_Management/W_TRUST/landed/LANE1_GOVERNANCE_ACTIVATION.landed-review.json   (new marker, #517)
F_Project_Management/W_TRUST/landed/LANE2B_MINIMUM_SHELF_MCP.landed-review.json      (new marker, #514)
docs/why.html                                                   (+ Five Theses section, Jon-authorized)
ops/lane1/evidence/90-reconcile-baseline-red.md                 (RED baseline evidence)
```

## What each slice did

- **RED baseline** (`7db0f45`): recorded the pre-cure state — registry empty (0 markers), truth floor green, WV-6 accepted at `2f8c9ad8`.
- **Slice 1** (`a55a4e0`): registered #514 and #517 landed markers. Each marker is its own file under `F_Project_Management/W_TRUST/landed/*.landed-review.json`; `LANDED_REVIEW_MARKERS.json` enumerates their paths as a sorted list that exactly matches the tree. Each marker is carried verbatim from its recorded independent review (`*.review.json`, reviewer IDC-Trust-Review id 306739987) and validates via `verify_repository_landed_markers` + `verify_landed_review_marker` against `refs/remotes/origin/main`. All under `W_TRUST/` → product-digest-excluded.
- **Slice 1b** (`d7430c2`): grafted the canonical Five Theses (+ Layer Map + Honest Objection) into `docs/why.html`, adapted to the page's own design system (Instrument Serif / DM Sans, garnet/gold). Capability-scope gate stays green (2 `<b>enforced:</b>` lines with matching pinned hashes, both test anchors, both canonical snippets preserved in the untouched BOUNDARY section).

## Fresh gate outputs at `d7430c2`

| gate | result |
|------|--------|
| `verify_repository_landed_markers` | **clean** (both markers valid) |
| `garnet_capability_scope_status --gate` | **PASS** (enforced_claim_count 2, hashes_match true) |
| `garnet_lane0_closeout_status --gate` | **PASS** |
| `garnet_msrv_status --gate` | **PASS** |
| `garnet_frozen_backlog_status --gate` | **PASS** |
| `garnet_trust_kernel_review_status --gate` | **RED (expected)** — `trust_kernel_touched:true`, only problem = "structured review record is missing". The one review record is added once at slice 7 after your approval. |
| `garnet_wv_acceptance_status --wv WV-6` | **partial (expected, U-29)** — `product content digest mismatch (c4b3cf7c != 2f8c9ad8)`. Cured by the native-Windows WV-6 refresh (slice 4) at this frozen head. |

## Expectations the repository falsified (reported, not worked around)

1. **Platform (F-1).** The cold-start prompt's slice 4 requires the WV-6 refresh to run on the platform the acceptance manifest names — **native Windows** (`proofs/windows/…`). The implementer machine is **macOS Darwin arm64**. Slice 4 cannot run here; it is deferred to a Windows-capable seat at this reviewed head.
2. **Denominators (F-2).** The prompt states the reconciliation records launch denominators **83.3% / 62.5%** (→ 66.7% / 50.0%). The sanctioned producers on this pre-Windows tree yield **launch_critical 3/6 = 50.0%** and **launch_ledger 3/8 = 37.5%** — unchanged from pristine. Only 3 launch gates pass here (`foundation_integrity`, `native_linux`, `s114_acceptance`); `minimum_sealed_shelf` needs WV-6 Windows acceptance, `live_wasm_playground`/`static_playground` are remaining/partial. The 83.3/62.5 are a **post-Windows** state — the prompt's own slice 5 reports the *reconciled* denominators at the post-Windows evidence head. **Per Jon's explicit decision, the reporter/SOTU/denominator refresh is deferred to slice 5**; it is not committed here (also because `garnet_launch_readiness_status.py` embeds an absolute machine path in `08.source`, which would stamp a machine-specific path into the product-digest-tracked tree).
3. **`ops/lane1/` digest asymmetry (F-3, note).** Unlike `ops/lane2b/` (in `FROZEN_MUTABLE_PREFIXES`, digest-excluded), `ops/lane1/` is **inside** the product digest. So this review-request commit and the RED-baseline evidence are product-digest-affecting. The frozen candidate `d7430c2` (digest `c4b3cf7c`) is unaffected by commits placed *above* it; flagging for the record/merge phase so the reviewed candidate, not the tip, is bound.

## Findings block (U-23 .. U-30 relevant)

- **U-30** (register): the heartbeat reviewer's scoped credential must **not** author verdict commits as IDC-Trust-Review — any commit that account authors enters the author/committer union and disqualifies it as approver (the #513 failure). Verdicts push under the fleet-fork author identity (`Jon Isaac <Navigata1@gmail.com>`), as used for every implementer commit on this branch.
- **U-29** (live): committing the reconciliation moved product bytes (`docs/why.html`, `ops/lane1/`), flipping WV-6 → partial. Lawful order honored: satisfy acceptance as written at the reviewed candidate first (slice 4 Windows), land, *then* redesign acceptance (Lane 0 repair #3, out of scope here). No digest weakening, exclusion, or rebind was performed to escape the deadlock.

## Specific questions for the reviewer

1. Are the two landed markers' provenance bindings exact — `merged_commit`/`merged_tree` on origin/main first-parent, `reviewed_head`/`reviewed_tree` from the recorded reviews, `content_digest` matching landed trust content, `review_record_path`+sha256 correct, `touched_paths` = the trust-kernel landing-edge?
2. Is the registry shape correct — `markers` a sorted path list exactly enumerating the two `*.landed-review.json` files, all under `W_TRUST/landed/`?
3. Does the `docs/why.html` graft preserve the capability-scope invariants (exactly 2 hash-pinned `<b>enforced:</b>` lines, both test anchors, both canonical snippets) with no new "enforced" language asserting an unproven trap?
4. Is the deferral of the reporter/SOTU/denominator leg to slice 5 (post-Windows) the correct call, given F-2/F-3?
5. Is linear lineage intact (no merge commit, no Update-branch)?

## Stop

Implementer STOPS here for the independent delta review. Verdict lands as `ops/lane1/review/NN-verdict.md` under the fleet-fork identity (never as IDC-Trust-Review). Nothing further is pushed after the eventual record commit; nobody touches Update branch.
