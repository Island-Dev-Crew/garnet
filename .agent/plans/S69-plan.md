# S69 — LLM-suggest v0.2 / Paper VI Exp 1 prep

## Goal
Ready the compiler-as-agent advisory's LLM tier and prep Paper VI Experiment 1,
honestly. The rules tier (S10) is the ACTIVE baseline; the LLM tier is
**pending-infra** (no model wired in this environment). No new firing advisory.

## Why honest-partial
- No LLM provider is available in this environment; calling one would be a fake
  runtime claim (violates calibrated-honesty doctrine).
- Adding a *new* firing rule risks the corpus/count assertions `suggest.rs`
  warns about, and repeats the S42-on-`safe_io` over-catch lesson. The rules
  tier stays exactly as shipped in S10.

## Deliverables
- `scripts/garnet_llm_suggest_readiness.py` — reporter; verifies the 3 S10 rule
  IDs exist in `garnet-check-v0.3/src/suggest.rs`; documents the LLM tier as
  pending-infra; records the Exp 1 prep protocol; quotes the Paper VI scorecard
  verbatim. `--gate` gates the rules tier's presence (LLM tier NOT gated).
- `scripts/test_garnet_llm_suggest_readiness.py` — 5 unit tests.
- `C_Language_Specification/GARNET_LLM_SUGGEST.md` — the two-tier model + Exp 1.
- CI wiring: test + `--gate` in the `agent-contracts` job.
- CHANGELOG `[Unreleased]` entry; contract S69 block; this plan.
- Ledger `s68 → merged(5)` (rides with this PR).

## Verification
- `python3 scripts/test_garnet_llm_suggest_readiness.py` → 5 OK.
- `garnet_llm_suggest_readiness.py --gate` → rc 0 (rules tier present).
- Ladder: `cargo fmt --all -- --check`, `git diff --check`,
  `cargo test --workspace --no-fail-fast` (no Rust changed → 0 failed).

## Honest scope (do not soften)
LLM tier pending-infra — no model called, no provider bundled, no new advisory.
Rules tier unchanged. Paper VI scorecard ("…1 pending-infra") surfaced verbatim.
