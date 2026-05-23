# AGENTS.md — garnet-cst Contract

## Scope

Owns Garnet's trivia-preserving Concrete Syntax Tree (CST): the rowan
`SyntaxKind` / `GarnetLanguage` binding, the `SyntaxNode` / `SyntaxToken`
aliases, the `CstNode` trait, `parse_cst`, `Parse<T>`, `cst_to_source`, and
(from PR-2) the recursive-descent CST builder plus the `cst_to_ast` projection.

Built **cold** for the v0.7 build-both-then-compare A/B (slice S15):
independently of the in-parser CST merged in #221
(`garnet-parser-v0.3/src/cst.rs`), which is preserved untouched as the
S15-Compare baseline. This crate shares only two surfaces with the parser — the
trivia-preserving lexer (`garnet_parser::lex_source`) and the AST type
(`garnet_parser::ast::Module`, the target of `cst_to_ast`). It never reads or
extends #221's CST.

## Stable Contracts

- `parse_cst(&str) -> Parse<SyntaxNode>` is **trivia-preserving**: every source
  byte — whitespace and comments included — is emitted into the rowan green
  tree in source order, so `cst_to_source(parse_cst(s).syntax()) == s` is a
  byte-identical round-trip for inputs that lex. PR-1 ships an intentionally
  trivial impl (whole source as one trivia leaf); PR-2 ships the structural
  builder. The round-trip guarantee holds in both.
- The `CstNode` trait (`syntax()`, `kind()`) is the load-bearing seam that S16
  (LSP precision) builds against. Evolve it only with a ledger note while the
  surface is `experimental`.
- `SyntaxKind` maps 1:1 from `garnet_parser::token::TokenKind` at the token
  level; composite node kinds follow Mini-Spec v1.0 §2–§11. Node kinds may be
  added additively in later PRs without breaking the trait surface.
- `u16` <-> `SyntaxKind` conversion is safe (no `mem::transmute`); this crate
  introduces no ambient `unsafe`.
- No OS authority: pure parsing, declares no `@caps`.
- Downstream consumers (interp, check, vm) stay on the AST path
  (`garnet_parser::parse_source`). `cst_to_ast` is an **additive** projection,
  not a replacement; CST-first migration of those consumers is v0.8 work.

## Stability

`experimental` until the S15-Compare checkpoint (Jon, fresh eyes) records the
canonical CST. The compiler `@stability(experimental)` annotation is wired once
S17 ships it.

## CST in v0.7 (spec note)

Parked here per the S15 plan so `GARNET_v1_0_Mini_Spec.md` stays under the
maintainer's hand:

> v0.7 adds a trivia-preserving CST as a first-class layer above the AST. The
> AST remains the semantic reference; the CST is a lossless syntactic
> projection used by editor tooling (rename, code actions, formatting). Two CST
> implementations coexist during S15 by design — #221's in-parser CST and this
> rowan crate — until the S15-Compare checkpoint records the canonical choice.
> Round-trip is source-preserving for inputs that lex; recovery from malformed
> input is best-effort and may diverge.

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
