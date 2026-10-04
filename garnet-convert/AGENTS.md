# AGENTS.md — Converter Contract

## Scope

Owns migration frontends and conversion helpers for lifting Rust, Ruby, Python, and Go source toward Garnet.

## Stable Contracts

- Conversion output must not overclaim: uncertain mappings should become explicit TODOs, not fake confidence.
- Keep sandboxing assumptions visible; do not execute source language code as part of conversion.
- Preserve provenance from source constructs to generated Garnet where possible.
- Every emitted file parses as Garnet (C1-18): the emitter parses its own output
  and refuses to emit text that does not parse.
- A statement the frontend cannot lower is kept whole as one `@migrate_todo`
  carrying its source lines; no fragment of it may stay active code. Each
  frontend reads a statement as a defined unit of its language, lexed in
  `src/frontends/lex.rs`: a Python logical line (and a compound statement's
  indented body and clauses), Go lines up to where Go inserts a semicolon (none
  after an operand-taking keyword such as `go` or `defer`), and for Ruby a
  lexical subset joined by brackets, keyword blocks, trailing operators,
  commas and modifiers, and a leading `.`. Only a statement on one line becomes
  code.
- What a frontend does not lex is refused with its line number, never guessed:
  Python tab or form-feed indentation, and f-strings or t-strings whose
  replacement fields hold their own quote, a comment or a backslash; Ruby
  outside the subset (heredocs and a `<<` with no space after it, percent
  literals, character literals, `=begin`, endless methods, multi-line
  strings). To support a construct, extend `lex.rs`
  and the unit, not one frontend's heuristic, and cover it in `tests/parses.rs`
  with a test that asserts its fragments are not active (`assert_inactive`).
- A definition or class-level statement the frontend does not read is kept as a
  to-do, never dropped: `__init__` stays a method, a Ruby definition whose `end`
  shares its line with more code is kept whole, an import is skipped only when
  nothing follows it on its line (a Ruby `require` only in its exact form), a Go
  function whose lexed header does not read is kept whole, and a Go struct whose
  fields are not one simple `Name Type [tag]` per line is kept whole.

## Required Checks

```sh
cargo test -p garnet-convert
cargo test -p garnet-cli convert
```
