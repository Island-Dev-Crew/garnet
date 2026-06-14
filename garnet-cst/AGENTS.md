# AGENTS.md — garnet-cst Contract

## Scope

Owns Garnet's trivia-preserving Concrete Syntax Tree (CST): the rowan
`SyntaxKind` / `GarnetLanguage` binding, the `SyntaxNode` / `SyntaxToken`
aliases, the `CstNode` trait, `parse_cst`, `Parse<T>`, `cst_to_source`, the
direct recursive-descent CST builder (`builder.rs`), the `cst_to_ast`
projection (`convert.rs`), the typed-node wrappers (`nodes.rs`), and the
LSP-facing token/span helpers (`tokens.rs`, `TokenInfo`, `token_infos`,
`identifier_spans`).

Built **cold** for the v0.7 build-both-then-compare A/B (slice S15),
independently of the in-parser CST merged in #221
(`garnet-parser-v0.3/src/cst.rs`). The S15-Compare checkpoint chose this rowan
crate as the canonical CST; #221's parser CST served as the temporary legacy
migration oracle and was retired at RB-4a (LSP rowan-backed and green). This crate
shares only two surfaces with the parser — the
trivia-preserving lexer (`garnet_parser::lex_source`) and the AST type
(`garnet_parser::ast::Module`, the target of `cst_to_ast`). It never reads or
extends #221's CST.

## Stable Contracts

- `parse_cst(&str) -> Parse<SyntaxNode>` is **trivia-preserving**: every source
  byte — whitespace and comments included — is emitted into the rowan green
  tree in source order, so `cst_to_source(parse_cst(s).syntax()) == s` is a
  byte-identical round-trip for inputs that lex. The builder also flushes any
  unconsumed tokens, so the round-trip holds even for grammatically invalid
  input (error recovery is best-effort on *structure*, never on round-trip).
- `cst_to_ast` projects the CST onto `garnet_parser::ast::Module`, **span-exact
  with `parse_source`** (RB-4b.1): `tests/substrate_fidelity.rs` proves the
  projected AST equals the parser's AST byte-for-byte INCLUDING spans across
  the corpus, and that `parse_cst` agrees with `parse_source` on the
  error-verdict (which inputs are rejected/over-budget). `span_of` trims trivia,
  skips item annotation/`pub` prefixes, and sees through transparent wrappers
  the parser strips from spans (`ParenExpr` → inner expr, `dyn` → keyword-
  excluded inner trait, parenthesized types → lowered-type end) to match the
  parser's token-joined spans; the CST tree shape is unchanged (round-trip +
  token-parity gate it). `cst_to_ast` is stack-safe to the default budget
  depth (256); a generous-`max_depth` caller must bound depth itself (known
  limitation). The older `tests/cst_to_ast_parity.rs` compares span-normalized
  and is now subsumed on spans. Existing AST consumers (interp, check, vm) keep using
  `parse_source` and are untouched; CST-first migration is the rest of RB-4b.
- `parse_cst_with_budget_and_edition` (RB-4b.1) applies the parser's fail-fast
  fences (source-bytes, token-nesting depth) error-TOLERANTLY — it records
  `SyntaxError`s but always builds a round-trippable tree, so the rowan path
  never silently accepts an input `parse_source` rejects. `parse_cst` is the
  default-budget+edition wrapper. Keep both fences in lockstep with
  `parse_source`; a new budget axis added to the parser must be mirrored here.
- `SyntaxError` carries a `span` (a range over the offending token), not just a
  start offset (RB-4b.2). Recovery errors anchor at the next SIGNIFICANT token
  (skip trivia), matching `parse_source`'s anchoring; budget/lex errors use
  `ParseError::span()`. This is the foundation for a future LSP single-parse —
  DEFERRED: `parse_cst`'s error recovery cascades (many errors per malformed
  input vs `parse_source`'s one), so the LSP keeps `parse_source` for
  diagnostics until recovery is de-noised. Do not drop `parse_source` from the
  LSP without first proving diagnostic quality is preserved-or-improved.
- Performance: the `parse_cst_vs_ast` Criterion bench keeps the CST path within
  1.5× the AST path (currently ≈1×). If a change pushes it over, document the
  ratio in `CHANGELOG.md` rather than blocking the slice.
- The `CstNode` trait (`syntax()`, `kind()`) is the load-bearing seam that S16
  (LSP precision) builds against. Evolve it only with a ledger note while the
  surface is `experimental`.
- `SyntaxKind` maps 1:1 from `garnet_parser::token::TokenKind` at the token
  level; composite node kinds follow Mini-Spec v1.0 §2–§11. Node kinds may be
  added additively in later PRs without breaking the trait surface.
- `tokens.rs` preserves #221's useful editor-facing token ergonomics on top of
  rowan: `TokenInfo` recovers `TokenKind` payloads, byte `Span`s, and exact
  token text. `tests/token_view_parity.rs` must keep proving that this token
  view matches `garnet_parser::lex_source` (the shared lexer surface) on the
  example corpus, excluding the zero-width EOF sentinel — the RB-4a successor
  to the retired legacy-oracle differential. Broader admission criterion
  (no parse-success requirement), exercised by explicit
  lexable-but-unparseable inline cases and guarded non-vacuous
  (`compared == corpus`).
- `u16` <-> `SyntaxKind` conversion is safe (no `mem::transmute`); this crate
  introduces no ambient `unsafe`.
- No OS authority: pure parsing, declares no `@caps`.
- Downstream consumers (interp, check, vm) stay on the AST path
  (`garnet_parser::parse_source`). `cst_to_ast` is an **additive** projection,
  not a replacement; CST-first migration of those consumers is v0.8 work.

## Stability

Canonical CST for v0.7 after the S15-Compare checkpoint. The Rust API remains
additive/experimental until S16 hardens the LSP-facing surfaces and S17 wires
compiler `@stability(experimental)` annotations.

## CST in v0.7 (spec note)

Parked here per the S15 plan so `GARNET_v1_0_Mini_Spec.md` stays under the
maintainer's hand:

> v0.7 adds a trivia-preserving CST as a first-class layer above the AST. The
> AST remains the semantic reference; the CST is a lossless syntactic
> projection used by editor tooling (rename, code actions, formatting). The
> S15-Compare checkpoint chose the rowan `garnet-cst` crate as canonical;
> #221's in-parser legacy oracle was deleted at RB-4a after its recorded
> precondition (rowan-backed LSP coverage green) was met. Round-trip is
> source-preserving for inputs that lex; recovery from malformed input is
> best-effort and may diverge.

## Required Checks

Run the crate's own suite after changes:

```sh
cargo test -p garnet-cst
```

Run the full workspace suite when the trait surface or `cst_to_ast` changes,
since S16 and the AST consumers depend on the seam:

```sh
cargo test --workspace --no-fail-fast
```
