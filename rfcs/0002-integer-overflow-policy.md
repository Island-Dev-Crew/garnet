# RFC-0002: Integer Overflow Policy — Checked by Default

- **Status:** Accepted <!-- design ruling (Jon, 2026-06-12); implementation is a separate W-REBUILD slice, not yet scheduled -->
- **Author(s):** Jon Isaac (ruling) · Claude Code (drafting)
- **Created:** 2026-06-12
- **Tracking PR:** the J-queue rulings docs PR (RB-band stop-report decision J5)
- **Origin:** W-REBUILD RB-2 named-deferred decision J5 (`F_Project_Management/W_REBUILD/RB_BAND_STOP_REPORT_2026-06-12.md`).
- **Scope:** `i64` arithmetic in the interpreter and the bytecode VM.

## Decision

Garnet integer arithmetic is **checked by default**: an operation whose true
result is not representable in the value type produces a runtime **diagnostic**
(a `RuntimeError` on the interpreter, the byte-identical `VmError::Runtime` on
the VM), never a silent wrap and never an uncontrolled process abort. Where a
program genuinely wants modular arithmetic, it uses **explicit wrapping
operations** (surface form TBD in the implementing slice — e.g. a
`wrapping_add`-style intrinsic or a `@wrapping` block), so the wrap is a
visible, intentional act rather than a build-profile accident.

## Why

Today's behavior is the worst of both worlds and the central reason this is a
ruling rather than a quiet default: **release builds silently wrap** (wrong
answers, no signal) while **debug builds abort** — arithmetic semantics differ
by build profile, which directly contradicts the trust kernel's claim that the
seal attests what the program actually does. Checked-by-default makes
arithmetic **loud, deterministic, and profile-independent**, and matches the
precedent already shipped for the all-profile `i64::MIN / -1` and `% 0` cases
(RB-2 [#389](https://github.com/Island-Dev-Crew/garnet/pull/389),
[#390](https://github.com/Island-Dev-Crew/garnet/pull/390)): those div/rem
overflows are already checked diagnostics with identical messages on both
backends. This RFC extends the same discipline to `+`/`-`/`*`.

## Boundary / non-goals

- This is an **interim** representation decision, not a commitment against a
  future bigint (Ruby-velocity) direction: checked-diagnostic **reserves** the
  overflow behavior rather than baking in wrapping, so a later move to
  arbitrary-precision `Int` is not foreclosed.
- The implementing slice must: land red→green per-backend tests; keep the
  interp/VM messages byte-identical (cross-backend parity, per the RB-2
  pattern); record a perf note (checked arithmetic in the VM hot path);
  and not be merged under a gate it modifies.
- No "enforced" claim beyond what a deterministic trap test proves.
