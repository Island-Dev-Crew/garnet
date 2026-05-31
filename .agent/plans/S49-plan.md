# S49 Plan — AI-PR-review-collapse wedge demo (the launch narrative)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S49.
Map: reconciled plan §97/§151 (GRAFT, Opus) — the launch narrative; machine
capability review vs human review collapsing under AI volume.
Branch: `codex/s49-wedge`. Base: `origin/main` @ `49f0c90` (S48).

## Thesis (the wedge)
AI volume collapses human PR review; the slips are authority/dependency changes,
not logic bugs. Garnet catches them in one O(1) command: `garnet diff-caps`
(GAINED caps), `garnet sandbox` (egress consequence), over-catch (S42),
slopguard (S45).

## Deliverables
- `examples/wedge_pr_review/{before,after}.garnet`: before `@caps(fs)`, after
  silently `@caps(fs, net)`. Both `garnet check` clean; both run (=> 0).
- `garnet-cli/tests/pr_review_wedge.rs` (3 tests): the CI-gated proof — clean on
  both, diff-caps exit 1 + GAINED net + AUTHORITY EXPANDED, sandbox egress flip.
  Runs cross-OS via `cargo test --workspace`.
- `scripts/smoke_garnet_pr_review_wedge.py`: narrative report (`--format md|json`),
  newest-binary resolution (avoids stale-artifact footgun).
- `scripts/test_garnet_pr_review_wedge.py` (4 tests): pure-logic + skipUnless live.
- Wire the py test into ci.yml agent-contracts.
- `F_Project_Management/GARNET_PR_REVIEW_WEDGE.md`: the narrative + honest scope.

## Dogfood
- `smoke_garnet_pr_review_wedge.py` → wedge fires (all steps ✅, exit 0).
- Rust test: 3 pass. check clean on both; the escalation is invisible to the
  checker but caught by diff-caps.

## End-state / gates
- Full ladder green (workspace 0 failed incl. the new Rust test). CHANGELOG +
  contract S49 block + wedge doc. Ledger: `s48 → merged(5)` advanced this branch;
  `s49` advance rides with S50 (the v0.8 beta gate).

## Honest scope (do not soften)
- "Human review collapse" is the cited thesis, NOT measured here — no fabricated
  review-time numbers.
- Narrative composition of existing gates, not new enforcement; a PR that keeps
  its capability surface unchanged is out of reach.
- No new readiness lane.
