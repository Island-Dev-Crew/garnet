# Garnet safe subset — specification (S74)

Garnet is dual-mode: a managed, Ruby-like mode (`def`, `FnMode::Managed`) and a
safe, Rust-like mode (`fn`, `FnMode::Safe`). The **safe subset** is the language
you get when you restrict to the safe mode. This document specifies the safe
subset **as it exists today**, then specifies a **proposed** optional
linear/effect-typed rigor mode for high-assurance components.

## 1. The safe subset today (implemented)

A `fn` (safe-mode) function is the Rust-like tier:

- **Typed.** Typed parameters and an explicit `-> type` return (see the EBNF
  `safe-fn` production and `GARNET_v1_0_Mini_Spec.md` §4.5).
- **Ownership & references.** Safe mode carries `&` / `&mut` reference types and
  Rust-style ownership; **privacy-by-default** so external code cannot alias
  internal state.
- **Capability coverage.** Safe-mode functions have their own caps-coverage path
  in the checker (distinct from the managed `managed-fn-missing-caps` rule).

### The fn↔def boundary audit (the safe-subset trust story)

The real safe-subset guarantee today is **boundary legibility**, implemented in
`garnet-check-v0.3/src/audit.rs` (`ModeAuditLog`). Every `fn`↔`def` boundary
crossing is emitted as a structured log entry:

```text
<source-span> <caller-mode> -> <callee-mode> <callee-name>
examples/mvp_02_relational_db.garnet:32 managed -> safe BTree::compare
examples/mvp_02_relational_db.garnet:47 safe -> managed RelDb::trim
```

This **closes the "hidden safe→managed escalation" threat class**: a reviewer
reads one audit file shipped with the manifest and enumerates every trust-boundary
crossing — no grepping `fn ` vs `def ` across hundreds of modules. The
`warn_if_audit_log_grows_faster_than_source` lint flags when boundary crossings
grow faster than code size (a design smell that the dual-mode boundary is being
forgotten).

**This is what the safe subset buys today: a typed, ownership-disciplined tier
whose every crossing into managed (ambient-authority) code is machine-enumerated.**

## 2. Proposed: optional linear/effect-typed safe mode (graft — NOT implemented)

The trajectory research (compass report; reconciled-plan §172 graft) is candid
that `@caps` annotations are a *pragmatic* middle ground, less rigorous than:

- **Austral's linear capabilities** — capability values that are unforgeable,
  non-duplicable, and must be passed explicitly (authority cannot be acquired
  "out of thin air"); capability security falls out of linearity.
- **Koka's algebraic effects** — effects tracked in the type system with
  well-studied semantics; an effect row is more expressive and more checkable
  than an annotation.

The **proposed** high-assurance mode would offer, for a safe subset only:

1. **Linear capability values** — `@caps` authority carried as linear values that
   the type system forbids duplicating or forging, giving a precise answer to
   *what prevents authority laundering* via FFI, ambient imports, or
   `proc`-spawned subprocesses (Deno's documented escape hatch; Pony's
   "no forging without FFI" framing).
2. **Effect rows (optional)** — track the effect surface in types for components
   that want Koka/Effekt-grade checkability over annotation legibility.

This mode is **opt-in and high-assurance**: ordinary code keeps the legible
`@caps` annotations (legibility matters most for agent-authored code humans
*accept*); high-assurance components can pay for linear/effect rigor.

## 3. Honest scope (do not soften)

- §1 (the safe subset today) describes **implemented** behavior: `FnMode::Safe`
  exists in `garnet-parser-v0.3/src/ast.rs`; the boundary audit exists in
  `garnet-check-v0.3/src/audit.rs`. `scripts/garnet_safe_subset_status.py --gate`
  verifies these groundings so the spec cannot overclaim.
- §2 (linear/effect-typed mode) is a **PROPOSAL — NOT IMPLEMENTED.** No linear
  type system, no effect rows, and no soundness proof ship in this slice. It
  records the high-assurance direction and the precedents to borrow (Austral,
  Koka), per the research's "offer an optional linear/effect-typed safe mode"
  recommendation. This is a specification slice, not a type-system implementation.
