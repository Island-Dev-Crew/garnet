# Phase 0 — Reconciliation baseline (RED before cure)

- Recorded: 2026-07-27T12:18:45Z
- Branch: `mission/l1-reconcile-post-activation` at `68317ae258327aade47fc2c07b7b5b580ec7c6ea`
- Base landed main: `68317ae` (#517 atop #514 `41d6ced`)
- Implementer: Claude Code, Opus 4.8 — macOS Darwin arm64 (Hughs-MacBook-Pro), fresh `autocrlf=false` clone
- Independent reviewer: Codex GPT-5.6 Sol (different machine/model) — verdicts land under the fleet-fork identity, never as IDC-Trust-Review (U-30)

## Failing state this reconciliation cures

The trust-kernel landed-review registry
`F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json` is an **empty list
(0 markers)**, created empty by #517. Two squashes have landed on `origin/main`
first-parent history without a landed marker:

| PR | merged_commit | review record (added on landing edge) |
|----|---------------|----------------------------------------|
| #514 (Lane 2B sealed Shelf + MCP) | `41d6ced…` | `LANE2B_MINIMUM_SHELF_MCP.review.json` |
| #517 (Lane 1 governance activation) | `68317ae…` | `LANE1_GOVERNANCE_ACTIVATION.review.json` |

Until both markers are registered, the post-squash reconciliation denominators
and the mission SOTU do not reflect the landed reality.

## Pre-reconciliation truth floor (all green on the pristine tree)

- `garnet_lane0_closeout_status --gate` → PASS · evidence 22/22 · ledger 37 · denominators 4/4 · launch HOLD · band 3
- `garnet_msrv_status --gate` → ok:true (MSRV 1.95)
- `garnet_frozen_backlog_status --gate` → ok:true
- `garnet_trust_kernel_review_status --gate` → ok:true (base==head, kernel untouched)
- `garnet_wv_acceptance_status --wv WV-6` → **accepted** 5/5, product digest `2f8c9ad860bd9c6dbd1e005b0c82af0288dd3a736bb416a3978708c12e6fa1fd`

## Expected post-cure state (U-29)

Committing the reconciliation changes product-tracked bytes, flipping WV-6 from
`accepted` to `partial` (digest moves off `2f8c9ad8…`) and dropping the
reconciliation denominators until the native-Windows WV-6 refresh re-accepts at
the reviewed frozen head. This is the lawful order — satisfy the acceptance law
as written at the reviewed candidate first; do not weaken, exclude, or rebind
the digest to escape it.
