# S76 — stdlib promotion wave

## Goal
Promote the foundational stdlib primitives out of the Experimental tier — the
first Rust-touching runway slice — resolving the core-only example stability
warnings honestly (without gaming the @stability contract).

## What ships
- **Rust:** `garnet-stdlib/src/registry.rs` — the whole `core::*` layer (30
  primitives: iter/result/option/cmp/math) Experimental → Stable. Two checker
  tests (`stability.rs`, `lib.rs`) repointed from `core::iter::map` (now Stable)
  to a still-experimental `std::json::parse`, preserving their intent (verify the
  experimental-warning machinery).
- `scripts/garnet_stdlib_promotion_status.py` (+ `--gate`, 5 tests) — parses the
  registry and gates that the wave stayed SCOPED: every core::* is Stable AND
  std::* still carries Experimental entries (so a blanket flip can't pass).
- Spec `C_Language_Specification/GARNET_STDLIB_PROMOTION.md`; CI agent-contracts;
  CHANGELOG; contract S76 block; this plan; ledger `s75 → merged`.

## Promotion criteria (ALL must hold)
1. core layer (RequiredCaps::none — no host authority);
2. frozen semantics (functional iterators / Result / Option / cmp / basic math);
3. test-covered + corpus-used.
std::* (env/process host authority; json/regex/uuid/base64/log evolving APIs) are
KEPT Experimental — their warnings are correct.

## Effect (honest)
- novel_07 (core-only) → 0 diagnostics.
- novel_04/05/06 → still warn (std::base64/json/regex/uuid/log experimental).
  This partially resolves the novel-composition follow-up; the residual warnings
  are accurate, not bugs.

## Verification
- `cargo build` + `cargo test --workspace` green (after repointing 2 tests).
- `python3 scripts/test_garnet_stdlib_promotion_status.py` → 5 OK; `--gate` rc 0.
- fmt/diff clean.

## Honest scope (do not soften)
A stability judgement, not warning-suppression. Only core::* promoted; std::*
stays Experimental until its API is settled.
