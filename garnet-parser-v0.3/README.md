# garnet-parser v0.3

**Rung 2.1 of the Garnet engineering ladder.**
A hand-rolled lexer and recursive-descent parser covering all 90 EBNF productions from the Garnet v0.3 Mini-Spec.

> *"Where there is no vision, the people perish." — Proverbs 29:18*

---

## What this crate is

This is the canonical v0.3 parser for the Garnet programming language. It consumes UTF-8 source text and produces a typed AST (`Module`) suitable for consumption by the managed-mode interpreter (Rung 3), the safe-mode type checker (Rung 4), and the name resolver / boundary validator (Compiler Architecture Spec Phases 3–5).

### Covered productions

All 90 productions from `GARNET_v0_3_Formal_Grammar_EBNF.md`:

| Category | Productions | Mini-Spec §  |
|----------|-------------|--------------|
| Top-level | 3 | §1 |
| Modules / imports | 4 | §3 |
| Memory units | 3 | §4 |
| Functions & closures | 10 | §5 |
| Types | 9 | §11 |
| User-defined types | 9 | §11.3 |
| Actors | 4 | §9 |
| Statements | 13 | §6 |
| Expressions (Pratt tower, 11 levels) | 16 | §§5–7 |
| Control flow | 6 | §6.2, §6.3, §7 |
| Error handling | 3 | §7 |
| Spawn / messaging | 2 | §9 |
| Lexical | 8 | §2 |

### v0.2 → v0.3 reconciliation

The v0.2 parser (historical, preserved in `Garnet-Gemini-Claude/`) covered ~20 productions against an earlier spec stub. v0.3 is a ground-up rebuild of the full parser from scratch, honoring the lessons in the v2.6 handoff (read the spec, not references to the spec) and adopting the design decisions recorded in `quirky-tinkering-meadow.md`.

| Subsystem | v0.2 coverage | v0.3 coverage |
|---|---|---|
| Memory declarations | ✓ (§2.1) | ✓ (§4.1, identical syntax) |
| Actor declarations | ✓ (§4.1) | ✓ (§9, extended with actor-scoped memory + let) |
| Expressions | Pratt tower, 6 levels | Pratt tower, 11 levels (added pipeline, or/and/not, range) |
| Functions | — | ✓ `def` (managed) + `fn` (safe) + closures |
| Modules | — | ✓ `module`, `use Path::{A,B}`, `use Path::*` |
| Control flow | — | ✓ `if`/`elsif`/`else`, `while`, `for`, `loop`, `match`, `try` |
| Pattern matching | — | ✓ 6 pattern kinds |
| Error handling | — | ✓ `try`/`rescue`/`ensure`, `raise`, `?` |
| User types | — | ✓ `struct`, `enum`, `trait`, `impl` |
| Annotations | — | ✓ `@max_depth(N)`, `@fan_out(K)`, `@require_metadata`, `@safe`, `@dynamic` |

---

## Using the crate

```rust
use garnet_parser::parse_source;

let src = r#"
@safe
module Crypto {
    fn hash(own data: Bytes) -> Hash {
        let mut state = State::new()
        state.update(borrow data)
        state.finalize()
    }
}
"#;

let module = parse_source(src).expect("parse failed");
assert!(module.safe);
assert_eq!(module.items.len(), 1);
```

### Error rendering

Parse errors are `miette`-compatible, producing rich diagnostic output with source context:

```rust
use garnet_parser::parse_source;
use miette::Report;

match parse_source(src) {
    Ok(module) => println!("parsed {} items", module.items.len()),
    Err(e) => {
        let report = Report::new(e).with_source_code(src.to_string());
        eprintln!("{report:?}");
    }
}
```

---

## Crate structure

```
garnet-parser-v0.3/
├── Cargo.toml              # edition 2021, deps: miette 7.6, thiserror 2.0
├── README.md               # (you are here)
├── src/
│   ├── lib.rs              # public API: parse_source, lex_source
│   ├── token.rs            # TokenKind (~80 variants), Span, StrPart
│   ├── lexer.rs            # hand-rolled single-pass lexer
│   ├── ast.rs              # all AST node types
│   ├── parser.rs           # cursor: peek, bump, eat, expect, skip_separators
│   ├── error.rs            # 6 ParseError variants with miette Diagnostic
│   └── grammar/
│       ├── mod.rs          # top-level item dispatch
│       ├── memory.rs       # §4 memory units
│       ├── actors.rs       # §9 actors, protocols, handlers
│       ├── types.rs        # §11 type expressions
│       ├── functions.rs    # §5 def/fn/closures + annotations
│       ├── modules.rs      # §3 module-decl, use-decl
│       ├── stmts.rs        # §6 blocks, let/var/const, while/for/loop, break/continue/return/raise
│       ├── expr.rs         # §§5–7 expressions (11-level Pratt tower)
│       ├── control_flow.rs # §6.2, §6.3, §7 if/match/try
│       ├── patterns.rs     # §6.3 pattern matching
│       └── user_types.rs   # §11.3 struct, enum, trait, impl
├── tests/
│   ├── lex_tests.rs
│   ├── parse_memory.rs, parse_actors.rs, parse_expr.rs, parse_stmts.rs
│   ├── parse_functions.rs, parse_modules.rs, parse_user_types.rs
│   ├── parse_control_flow.rs, parse_patterns.rs
│   └── parse_examples.rs
└── examples/
    ├── memory_units.garnet
    ├── greeter_actor.garnet
    ├── build_agent.garnet
    ├── safe_module.garnet
    ├── control_flow.garnet
    └── error_handling.garnet
```

---

## Building and testing

```bash
cd garnet-parser-v0.3
cargo build                    # compile
cargo test                     # run the full test suite (~129 tests)
cargo clippy -- -D warnings    # lint, zero warnings enforced
cargo doc --open               # generate API docs
```

**Rust version:** stable, tested on 1.94.1. No nightly features used.

### What the test suite covers

- `lex_tests.rs` — 20 lexer tests (literals, operators, keywords, float/range disambiguation, string interpolation, error paths)
- `parse_memory.rs` — 8 tests covering all 4 memory kinds with simple and nested generic stores
- `parse_expr.rs` — 15 tests covering all 11 Pratt precedence levels and postfix operators
- `parse_stmts.rs` — 13 tests covering let/var/const, while/for/loop, break/continue/return, assignment operators
- `parse_functions.rs` — 12 tests covering def/fn distinction, annotations, closures, generics
- `parse_modules.rs` — 8 tests covering module declarations and all three use-import forms
- `parse_actors.rs` — 12 tests covering actors with protocols, handlers, actor-scoped memory and let
- `parse_user_types.rs` — 14 tests covering struct/enum/trait/impl with generics
- `parse_control_flow.rs` — 13 tests covering if/elsif/else, match with 6 pattern types, try/rescue/ensure
- `parse_patterns.rs` — 8 tests covering all 6 pattern kinds in match arms
- `parse_examples.rs` — 6 round-trip tests, one per `examples/*.garnet` file

Target: ~129 tests, all passing on first execution.

---

## Key design decisions

Design decisions adopted from the Plan phase of this build and preserved across the codebase:

- **Pipeline `|>` at lowest binary precedence** (follows the EBNF spec and matches Elixir convention). `x + 1 |> f` parses as `(x + 1) |> f`.
- **`TypeExpr` enum** (Named / Fn / Tuple / Ref) replaces the v0.2 flat `Type` struct. Disambiguation: `&` leads to Ref; `(` leads to fn-or-tuple; everything else is a named path.
- **Single `FnDef` with `FnMode` discriminant** (Managed / Safe). Both `def` and `fn` produce the same AST shape; mode-specific rules (optional types for managed, required types + ownership for safe) are enforced during parsing.
- **`Block` carries `tail_expr: Option<Box<Expr>>`** — enables `{ stmts; [trailing_expr] }` without ambiguity. An expression followed by `}` with no separator becomes the tail; otherwise it wraps in `Stmt::Expr`.
- **`@` lexed as a standalone token.** The parser combines `At` + `Ident("safe")` etc. into `Annotation` values. This keeps the lexer simple.
- **Float vs range disambiguation:** two-character lookahead in `lex_number()`. `.` followed by a digit → float; `.` followed by `.` → stop and let the operator lexer emit `DotDot`/`DotDotDot`.
- **Block vs map-literal disambiguation:** map literals require `=>`. `{ x }` is always a block expression; `{ k => v }` is a map literal.
- **Fail-fast error reporting.** No multi-error recovery. The first error surfaces with rich context via miette. Multi-error recovery is future work for the IDE path.

---

## Relationship to the engineering ladder

| Rung | Status | Dependency |
|---|---|---|
| 1 — Mini-Spec v0.2 | ✓ shipped | — |
| 2 — v0.2 parser | ✓ shipped | Rung 1 |
| **2.1 — v0.3 parser (this crate)** | **✓ shipped** | Mini-Spec v0.3, Formal Grammar EBNF |
| 3 — Managed interpreter + REPL | ⬜ next | This crate + Mini-Spec v0.3 |
| 4 — Safe-mode type checker + LLVM lowering | ⬜ queued | Rung 3 |
| 5 — Memory Core + Manager SDK | ⬜ queued | Rung 4 |
| 6 — Harness Runtime + `garnet` CLI | ⬜ queued | Rung 5 |

The parser is the foundational gate: every rung above it consumes the `Module` AST this crate produces. Type checking, interpretation, and codegen all operate on these nodes.

---

## License

Dual-licensed under MIT OR Apache-2.0, matching Rust ecosystem convention. You may use this crate under the terms of either license.

---

*"In the multitude of counsellors there is safety." — Proverbs 11:14*
*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*

**garnet-parser v0.3 — Island Development Crew, April 2026**
