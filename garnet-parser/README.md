# garnet-parser

**Rung 2 of the Garnet engineering ladder.** A hand-rolled lexer and
recursive-descent parser for Mini-Spec v0.2 §2.1 (memory unit declarations)
and §4.1 (actor declarations with protocols and handlers).

> *"Where there is no vision, the people perish."* — Proverbs 29:18

## Status

- ✅ **Builds cleanly** on stable Rust (tested on 1.94.1).
- ✅ **35 tests passing** (`cargo test`), covering lexer, §2.1, §4.1, and
  round-trip parsing of every example file.
- ✅ **Mini-Spec v0.2 §7-compliant** for the parser MUST clauses on §2.1
  and §4.1.
- ⚠️ **§5.1–§5.3 is a structural no-op.** v0.2 defines normative MUST
  rules without surface syntax — see *v0.2 underspec note* below.

## Engineering ladder context

| Rung | Deliverable | Status |
|---|---|---|
| 1 | Mini-Spec v0.2 | ✅ complete |
| **2** | **`garnet-parser` crate** | ✅ **this crate** |
| 3 | Managed interpreter + REPL | ⬜ next |
| 4 | `@safe` lowering | ⬜ queued |
| 5 | Memory Core + Manager SDK | ⬜ queued |
| 6 | Harness Runtime | ⬜ queued |

## Quick start

```bash
cargo build
cargo test
```

```rust
use garnet_parser::parse_source;

let src = r#"
memory episodic conversations : Vector<Turn>

actor Greeter {
  protocol hello(name: String) -> String
  on hello(name) {
    let greeting = "hello, #{name}"
    greeting
  }
}
"#;

let module = parse_source(src).expect("parses cleanly");
assert_eq!(module.items.len(), 2);
```

Errors are span-attached `miette::Diagnostic` values — wrap in
`miette::Report::new(err).with_source_code(src.to_string())` to get
source-context rendering.

## Crate layout

```
garnet-parser/
├── Cargo.toml
├── README.md            (this file)
├── src/
│   ├── lib.rs           public API: parse_source, lex_source
│   ├── token.rs         Token, TokenKind, Span, StrPart
│   ├── lexer.rs         hand-rolled single-pass lexer
│   ├── ast.rs           AST nodes (Module, Item, MemoryDecl, ActorDef, …)
│   ├── parser.rs        recursive-descent cursor + helpers
│   ├── error.rs         span-attached ParseError variants
│   └── grammar/
│       ├── mod.rs       top-level item dispatch
│       ├── memory.rs    §2.1 — memory units + recursive Type parser
│       ├── actors.rs    §4.1 — actor / protocol / handler
│       └── expr.rs      handler-block interior (Pratt expressions)
├── tests/
│   ├── lex_tests.rs     11 lexer tests (8 happy + 4 error)
│   ├── parse_memory.rs   8 §2.1 tests   (4 happy + 4 error)
│   ├── parse_actors.rs  12 §4.1 tests   (8 happy + 4 error)
│   └── parse_examples.rs 3 round-trip tests for examples/*.garnet
└── examples/
    ├── memory_units.garnet
    ├── greeter_actor.garnet
    └── pingpong_actors.garnet
```

## What this crate parses

### §2.1 — memory unit declarations

```garnet
memory working   scratch       : SemanticCache
memory episodic  conversations : Vector<Turn>
memory semantic  knowledge     : Map<String, Vector<Embedding>>
memory procedural skills       : SkillBox
```

The four `memory-kind` keywords (`working`/`episodic`/`semantic`/`procedural`)
are tokenized as keywords, not identifiers. `store-type` is a recursive
generic-type grammar — `Map<String, Vector<Embedding>>` parses to a
`Type` tree of arity-2/arity-1 nesting.

The §2.2 *uniqueness* rule ("two memory units of the same kind and ident in
the same module MUST be a compile-time error") is a **validator-pass
concern**, not parsing. The parser accepts duplicates; rung 3+ will catch
them.

### §4.1 — actor declarations

```garnet
actor Greeter {
  protocol hello(name: String) -> String

  on hello(name) {
    let greeting = "hi"
    greeting
  }
}
```

- **Brace-delimited** actor body — matches the spec's
  `"actor" ident "{" protocol-decl* handler-decl* "}"` production.
- **Protocols and handlers may interleave** in any order — pairing
  enforcement (§4.2 "every declared protocol MUST have a handler") is
  validator territory, not parsing.
- **Protocol parameters MUST be typed** (`name: String`); handler
  parameters MAY be untyped (`name`). This is a small ergonomic delta
  beyond the spec's strict reading of `param-list`, justified by the spec's
  silence on whether handler params share the protocol param vocabulary.

### Handler-block interior — *v0.2 underspec note*

Mini-Spec v0.2 §4.1 defines `handler-decl := "on" ident "(" param-list ")" block`
**but never defines `block`**. The parser cannot consume handler bodies
without *some* interpretation of `block`, so this crate provides the
smallest one that lets useful programs parse:

```text
block := "{" stmt* "}"
stmt  := "let" ident "=" expr
       | "return" expr?
       | expr
```

The expression grammar inside is Pratt-style precedence
(equality < comparison < add < mul < unary < postfix < primary), with
postfix `.field` / `.method(args)` / `expr(args)` and primary
literals / identifiers / `Path::segments` / parens / strings (including
`#{...}` interpolation, which re-lexes the inner source).

**When v0.3 specifies `block`, `src/grammar/expr.rs` is the file to revise.**
The disclaimer at the top of that file calls out the gap.

### §5.1–§5.3 — recursion guardrails (intentional no-op)

Mini-Spec v0.2 §5 defines normative MUST rules for recursion-depth limits,
fan-out caps, and metadata validation, but **does not define a surface
syntax**. The parser is therefore a structural no-op for §5: there is no
`recurse` keyword, no `@max_depth(N)` attribute, no fan-out annotation.

Per §5.4 ("§5 constrains program structure, not program semantics"), this
is correct for v0.2. A v0.3 stub will need to specify the surface for
recursion annotations before the parser can enforce §5.1–§5.3 statically.

## Protocol reconciliation note

The original `GARNET_v2_2_Master_Execution_Protocol.md` Priority 4 brief
listed the parser's targets as "§2.1, §4.1, §5.1–§5.3 grammars" — but a
prior session (v2.5 → v2.6 attempt) misread the protocol's section numbers
as referring to *managed mode*, *`@safe` mode*, and *typed actors*
respectively. Reading the actual Mini-Spec v0.2 reveals:

| Brief reference | What it actually is in v0.2 |
|---|---|
| §2.1 | **Memory unit declarations** (not managed-mode `def`/`end`) |
| §4.1 | **Actor declarations with `{...}` braces** (not `@safe` mode) |
| §5.1–§5.3 | **Recursion guardrails** (no concrete grammar in v0.2) |

`@safe` is a **module-level annotation** per §3.3, not a per-function
attribute, and §3 is **not** in the parser's MUST-parse mandate per §7.
This crate targets the actual v0.2 spec, not the misread reconstruction.
A v0.3 stub that adds a `block` grammar and a recursion-annotation surface
will let rung-2.1 work absorb both.

## Diagnostics

`ParseError` is a `thiserror::Error + miette::Diagnostic` enum with five
variants:

- `UnexpectedChar` — lexer hit a byte it can't classify
- `UnterminatedString` — string opened but not closed before EOF/newline
- `InvalidInt` — integer literal didn't fit in `i64`
- `UnexpectedToken` — parser expected one production, got another
- `UnexpectedEof` — input ended where the parser still wanted tokens

All variants carry `SourceSpan` labels. To render with source context:

```rust
use miette::Report;

if let Err(e) = parse_source(src) {
    let report = Report::new(e).with_source_code(src.to_string());
    eprintln!("{report:?}");
}
```

## Anchors

> *"In the multitude of counsellors there is safety."* — Proverbs 11:14
>
> *"The plans of the diligent lead surely to abundance."* — Proverbs 21:5
>
> *"Where there is no vision, the people perish."* — Proverbs 29:18

**Iron Canvas.** Garnet · `#9B1B30` · safe by default · fast when needed · joyful always.

## License

MIT OR Apache-2.0
