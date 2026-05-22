# Workspace Plan: S15 — Trivia-preserving CST in `garnet-parser-v0.3`

Advance the Garnet compiler towards the `v0.6.0` release gate by building a source-faithful Concrete Syntax Tree (CST).

## Core Architecture Design
To preserve whitespace and comments without risking regression of standard EBNF grammar rules in the 11 `grammar/*.rs` files, we isolate trivia handling:
1. **Lexer Extension**: `lexer.rs` produces `TokenKind::Whitespace(String)` and `TokenKind::Comment(String)`.
2. **AST Parser Cursor Isolation**: `parse_source` filters out these trivia tokens from the flat stream *before* starting AST recursive-descent parsing.
3. **CST Construction**: Build a lightweight tree mapping AST node spans to the complete flat raw token stream. Since AST nodes are nested, each token (including whitespace and comments) belongs inside the most specific AST node span containing it.

## Verification
- CST round-trip test covering all `examples/*.garnet` files byte-identically.
- MIT readiness script addition (`parser_cst_layer`).
