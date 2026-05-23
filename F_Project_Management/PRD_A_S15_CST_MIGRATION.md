# PRD A — S15: Trivia-Preserving CST Migration

| Field | Value |
|---|---|
| **Slot** | mac-opus (Claude Code Opus 4.7 1M Max, macOS) |
| **Slice** | S15 |
| **Status** | not-started → planned → in-progress → review-ready → dogfood-passing → merged |
| **PR count** | 2 (trait stub + substantive) |

---

## Goal

Migrate Garnet's parser from AST-only to a trivia-preserving CST (Concrete Syntax
Tree). This is the load-bearing foundation for LSP precision features (S16) and
for any future rename, refactor, or code-action work. Until this lands, LSP work
is constrained to MVP top-level features.

## v0.7 directive — build independently, then compare (READ FIRST)

A prior Codex PR (**#221, "Advance S15 CST and S16 LSP readiness"**) already
merged a **hand-rolled, in-parser CST** at `garnet-parser-v0.3/src/cst.rs`
(~510 lines, `CstNodeKind`) plus a roundtrip test and ~578 lines of
S16-adjacent LSP work in `garnet-lsp/src/lib.rs`.

Per Jon's directive, **do not extend, copy from, or override that work during
S15.** Instead:

1. **Build the rowan `garnet-cst` crate cold** — as if no CST existed — at full
   Opus 1M scope, straight from the Mini-Spec grammar. Independence is the
   point: two implementations built without reference to each other surface each
   other's blind spots.
2. **Build additively.** `garnet-cst` is a NEW workspace member; the parser
   gains an opt-in CST mode. **Do not modify or delete
   `garnet-parser-v0.3/src/cst.rs` (#221)** — it is preserved untouched as the
   comparison baseline.
3. **Do not reconcile during S15.** Extraction, diffing, and the
   keep/merge/discard decision happen in the separate **S15-Compare** checkpoint
   (see `GARNET_v0_7_SLICE_DOGFOOD.md`), a human-in-the-loop review by Jon with
   fresh eyes on both variations — not autonomous agent work.

Everything below describes the rowan crate to build. Treat #221 as out of view
while building.

## Why Mac Opus

Largest architectural slice in v0.7. Deep refactor of the parser's output surface.
Opus 4.7's 1M context window holds the parser + spec + downstream consumers
simultaneously. Goal-orientation fits because the end-state (parse → roundtrip
without trivia loss) is a single, clear, mechanical test.

## Owned crates (writable)

- `garnet-parser-v0.3` — add CST output mode behind an opt-in flag; preserve AST path. **Do not modify or delete the existing `src/cst.rs` (#221) — it is the S15-Compare baseline.**
- `garnet-cst` — **NEW crate**, primary deliverable

## Read-only crates

- `garnet-interp-v0.3` — consumes AST today; must keep working
- `garnet-check-v0.3` — consumes AST today; must keep working
- `garnet-lsp` — will consume CST after S16; not yet
- `garnet-vm` — consumes AST
- `garnet-stdlib` — no AST dep, but co-located
- `garnet-cli` — orchestrates; must keep working

## Dependencies

- **None upstream**. This slice can start immediately.

## Downstream blocks

- **S16 (LSP precision)** depends on the CST trait being published.
- **Trait-first protocol** — see "Trait Publication Protocol" below.

---

## Implementation Plan

### 1. Pre-work — read the prior art

Before writing any code, pull rust-analyzer's `rowan` and `syntax` crates as
reference material:

```bash
npx open-source rust-analyzer/rowan
npx open-source rust-analyzer/rust-analyzer
```

This puts them under `./repos/` so the model can reference real production
patterns instead of reinventing. Study how rust-analyzer wires Rowan into
`rust-analyzer-syntax`. Don't reinvent.

### 2. New crate `garnet-cst/`

With `rowan` as the dependency:

```rust
pub enum SyntaxKind {
    // Tokens
    Whitespace, Comment, Ident, IntLit, StrLit, // ... etc.
    // Composite nodes
    Module, FnDef, ParamList, Param, Block, ExprStmt, // ... etc.
    // The root
    Root,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GarnetLanguage {}
impl rowan::Language for GarnetLanguage {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind { /* ... */ }
    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind { /* ... */ }
}

pub type SyntaxNode  = rowan::SyntaxNode<GarnetLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<GarnetLanguage>;
```

### 3. Trait Publication PR (PR-1, small, opens FIRST)

This is the unblock-S16 PR. Ships with a STUB implementation that returns a
single CST node containing all source as trivia. The trait is real; the impl is
intentionally trivial.

```rust
/// The CST node trait — every typed wrapper around SyntaxNode implements this.
pub trait CstNode {
    fn syntax(&self) -> &SyntaxNode;
    fn kind(&self) -> SyntaxKind;
}

/// Parse a source string into a CST. Stub in PR-1; real impl in PR-2.
pub fn parse_cst(input: &str) -> Parse<SyntaxNode> { /* stub */ }

pub struct Parse<T> {
    pub root: T,
    pub errors: Vec<SyntaxError>,
}
```

Mark trait + `parse_cst` as `@stability(experimental)` (once S17 ships the
annotation) so downstream knows the surface may evolve.

**Target merge time for PR-1: 24 hours from open.** Coordinate via the ledger.

### 4. Parser CST output mode (PR-2)

- Add `--mode cst` flag to `garnet-parser-v0.3` (default remains AST for back-compat).
- Parser internally builds the CST via rowan's `GreenNodeBuilder`.
- Existing AST path stays via a CST→AST converter (single function in `garnet-cst`).

### 5. CST→AST converter (PR-2)

```rust
/// Lossy on trivia, lossless on structure. Existing AST consumers
/// (interp, check, vm) continue working unchanged.
pub fn cst_to_ast(node: &SyntaxNode) -> Module { /* ... */ }
```

### 6. Roundtrip property test (PR-2)

For any UTF-8 input that parses without errors:
`cst_to_source(parse_cst(input)) == input` (preserves trivia).

Use `proptest`. Run ≥1000 iterations in CI nightly.

### 7. Performance gate (PR-2)

New Criterion benchmark: `bench_parse_cst_vs_ast`.

Acceptable: CST path ≤ **1.5× AST path** on the 10 MVP examples.

If slower than 1.5×, ship anyway and document in CHANGELOG; don't block S15 on
perf optimization — that's v0.8 work.

### 8. Documentation (PR-2)

- `garnet-cst/AGENTS.md` — trait surface, CST→AST converter, stability tier.
- Update Mini-Spec §X.Y (grammar) with a one-paragraph note about CST in v0.7.

---

## Trait Publication Protocol

Open **TWO PRs** explicitly:

1. **PR-1 — `S15: garnet-cst trait surface + stub`** (small, fast, low-risk).
   Merge target: within 24h of opening.
2. **PR-2 — `S15: trivia-preserving CST via rowan`** (substantive).

Reason for two: unblock S16 (win-codex) immediately. They can begin coding against
the trait the moment PR-1 lands.

---

## Dogfood block (verification)

```bash
cargo build -p garnet-cst --release
cargo test -p garnet-cst -p garnet-parser-v0.3 --no-fail-fast
cargo bench -p garnet-cst --bench parse_cst_vs_ast
cargo test --workspace --no-fail-fast   # existing consumers still work
```

Expected:
- All workspace tests pass (no regression for interp/check/vm).
- Roundtrip property test: 1000/1000 inputs roundtrip cleanly.
- Bench: CST path ≤ 1.5× AST path; numbers committed to bench output.

---

## Out of scope

- LSP feature implementation (S16's job).
- Editor extension updates (S16's job).
- Migrating interp/check/vm to consume CST directly (v0.8 work).
- Performance optimization below the 1.5× bar (v0.8 optimization slice).
- **Reconciling with #221's in-parser CST — that is the separate S15-Compare checkpoint, not S15.** Build cold; do not read or extend `garnet-parser-v0.3/src/cst.rs`.

---

## Coordination

- Touch `garnet-check-v0.3` only via the existing AST surface. Do NOT modify
  check's internals.
- Touch `garnet-interp-v0.3` only to add CST-mode parse opt-in. Do NOT modify
  interp's AST-consumption path.
- If you find you must modify another agent's crate, STOP and append a Handoff
  Request entry in `AGENT_COORDINATION_LEDGER.md`.

---

## Honest accounting hooks

- "S15 ships the CST representation; downstream migration to CST-first is v0.8 work."
- "Downstream consumers (interp, check, vm) still operate on the AST projection;
  the AST path is preserved via `cst_to_ast()`."
- "CST parsing is approximately N× slower than AST parsing (committed bench
  numbers); optimization deferred to v0.8 unless reviewers flag this as a
  v0.7 blocker."

---

## Done criteria

- [ ] PR-1 (trait surface) merged with green CI.
- [ ] PR-2 (full impl) merged with green CI.
- [ ] `garnet-cst` crate published to workspace `Cargo.toml`.
- [ ] Roundtrip property test green at 1000+ iterations.
- [ ] `AGENT_COORDINATION_LEDGER.md` updated: mac-opus / S15 / MERGED.
- [ ] `CHANGELOG.md` updated under `[Unreleased]`.
- [ ] Readiness reporter shows a new lane for "CST migration".
