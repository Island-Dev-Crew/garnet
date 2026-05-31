# S74 — safe-subset spec (+ linear/effect-typed mode graft)

## Goal
Specify Garnet's safe subset (the `fn` safe mode) and fold in the Opus graft
(reconciliation §172): an optional linear/effect-typed rigor mode for
high-assurance components, per the compass trajectory report.

## What ships
- `C_Language_Specification/GARNET_SAFE_SUBSET.md`:
  - §1 the safe subset **today (implemented)**: typed, ownership-disciplined `fn`
    (`FnMode::Safe`), privacy-by-default, safe-mode caps coverage, and the fn↔def
    boundary audit (`audit.rs` `ModeAuditLog`) that closes the "hidden
    safe→managed escalation" threat class + the audit-growth lint.
  - §2 **proposed** optional linear/effect-typed mode (Austral linear
    capabilities / Koka effects) — high-assurance, opt-in; gives a soundness
    story for what stops authority laundering via FFI/ambient/proc. NOT
    IMPLEMENTED.
- `scripts/garnet_safe_subset_status.py` (+ `--gate`) — static anti-overclaim
  gate: spec present + anchored + the "implemented today" claims grounded in real
  source (`FnMode::Safe` in AST; boundary audit in checker).
- `scripts/test_garnet_safe_subset_status.py` — 5 unit tests.
- CI: agent-contracts (static gate + test). CHANGELOG; contract S74 block; this
  plan; ledger `s73 → merged`.

## Why this scope
The safe subset already exists (dual-mode `fn`/`def`); S74 consolidates it into
one spec and records the high-assurance rigor direction the research recommends —
without building a type system (that is research-grade future work). The gate is
purely static (reads spec + source), so it runs in the python-only agent-contracts
job; no binary needed.

## Verification
- `python3 scripts/test_garnet_safe_subset_status.py` → 5 OK.
- `garnet_safe_subset_status.py --gate` → rc 0 (spec present + grounded).
- Ladder: fmt/diff clean; `cargo test --workspace` 0 failed (no Rust changed).

## Honest scope (do not soften)
The linear/effect-typed mode is a PROPOSAL — NOT IMPLEMENTED. No linear type
system, effect rows, or soundness proof. A specification slice only.
