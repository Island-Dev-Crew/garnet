## Summary

Repairs the two Lane 0 durability defects that made downstream lane boot depend
on platform checkout behavior and ambient Git objects.

- U-20: fences every `ops/**/evidence/**` path byte-exact so Windows
  `core.autocrlf=true` cannot invalidate sealed hashes.
- U-21: keeps the pre-squash reviewed head as provenance but derives changed
  paths only from main-reachable `231aefa..aa681ba`.
- Records the legitimate 86-to-87 delta as the post-review Lane 0 W_TRUST
  companion, with no candidate-only path and no content divergence.

No check is skipped, weakened, excluded, or made advisory. Launch remains HOLD,
the global audit remains band 3/5, and this PR is Jon-merge-only.

## Root cause

`ops/lane0/evidence/` had no effective `.gitattributes` fence, so Windows
checkout conversion changed all 22 sealed evidence files. Separately, the
PR-body validator still ran `cat-file`, ancestry, and diff operations against a
discarded pre-squash commit even after the broader review marker became
squash-durable in #508.

## Dogfood Readiness

### Current truth

- [x] Fresh repair base is upstream `origin/main`
  `8535e6d3f9023cabc57476991024354dd2741dc1`.
- [x] U-20 and U-21 are logged beside U-19 and resolved by deterministic traps.
- [x] Exactly four denominators remain: S114 100.0%, truth pulse 93.1%,
  launch-critical 50.0%, launch ledger 37.5%; launch is HOLD.
- [x] No `refs/pull/*` was fetched in the implementation or acceptance clones.

### Local verification

- [x] `python3 -I scripts/test_garnet_lane0_closeout_status.py` — 36 tests
  green, one Windows symlink skip.
- [x] `python3 scripts/test_garnet_trust_kernel_review_status.py` — 14 green.
- [x] `python3 scripts/garnet_trust_kernel_review_status.py --gate --format json`
  — `ok: true`, `problems: []`.
- [x] `python3 scripts/check-agent-contracts.py` and its six-test suite.
- [x] `cargo run -q -p xtask -- truth --check`.
- [x] Frozen-backlog and MSRV gates, `cargo fmt --all -- --check`, and
  `git diff --check`.

### Remote verification

- [x] Fresh Windows default-autocrlf clone: 23/23 object/worktree byte equality,
  candidate absent, no pull refs, closeout PASS.
- [x] Fresh Ubuntu/WSL clone: 23/23 byte equality, candidate absent, no pull
  refs, closeout PASS.
- [ ] GitHub required checks must pass on the terminal PR head before Jon merges.

### Evidence bundle

- [x] `ops/lane0-repair2/evidence/MANIFEST.sha256` seals baseline RED,
  discrepancy, TDD, object-history, Windows, Linux, and local-gate evidence.
- [x] `F_Project_Management/W_TRUST/LANE0_EVIDENCE_DURABILITY_REVIEW_2026-07-16.md`
  is the paired trust-kernel companion.
- [x] `ops/lane0-repair2/state.json`, `ledger.jsonl`, and `journal.md` close the
  bounded ARCHIPELAGO repair at S6.

### Deferred / out of scope

- Jon owns review, merge, FIRE, tags, publishing, promo QA, and the 31-to-32
  ceremony.
- No Lane 1, Lane 2A, Lane 2B, workflow, ruleset, language-surface, Studio,
  registry, provider-LLM, mobile, or promotional work is included.
- After merge, each dependent lane must re-run the Lane 0 gate from fresh
  upstream main before resuming.
