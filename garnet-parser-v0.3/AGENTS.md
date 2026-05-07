# AGENTS.md — Parser Contract

## Scope

Owns Mini-Spec v1.0 lexing, parsing, AST shape, grammar examples, and parser tests.

## Stable Contracts

- Keep grammar behavior aligned with `C_Language_Specification/GARNET_v0_3_Formal_Grammar_EBNF.md` and Mini-Spec v1.0.
- Preserve diagnostic span quality when changing lexer or parser code.
- Do not silently accept syntax that the spec does not describe unless the spec is updated in the same change.
- Examples under this crate are parser fixtures, not product demos.

## Required Checks

Run parser-focused tests after changes:

```sh
cargo test -p garnet-parser
```

Run full workspace tests when grammar changes affect downstream crates.
