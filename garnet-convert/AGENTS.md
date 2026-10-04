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
  Python tab or form-feed indentation, a carriage return without a line feed,
  and f-strings or t-strings whose replacement fields hold their own quote, a
  comment, a backslash in the field's expression outside a nested string, a
  triple-quoted string or
  another f-string or t-string; Ruby outside the subset (heredocs and a `<<` with no
  space after it, regular expressions with interpolation, interpolation beyond
  plain expressions, an `alias` or `undef` with operands on another line,
  percent literals, character literals, `=begin`, endless
  methods, multi-line strings).
- Every line of a kept note or untranslatable reason is emitted as a comment;
  the text is split at `\n` and at a bare `\r`, so none of it lands outside a
  comment whatever a reader treats as a line end.
- No emitted code line holds `#{`: the emitter keeps any construct whose code
  (not its comments) contains it as a to-do, because source text that was inert
  would run as Garnet string interpolation. Likewise any construct that copies
  source text holding a quote character (`Cir::copies_quoted_text`): the source's
  quoting may tokenize differently in Garnet. Emit string values as
  `CirLit::Str`, which the emitter escapes, never as copied text. To support a construct, extend `lex.rs`
  and the unit, not one frontend's heuristic, and cover it in `tests/parses.rs`
  with a test that asserts its fragments are not active (`assert_inactive`).
- A definition or class-level statement the frontend does not read is kept as a
  to-do, never dropped: `__init__` stays a method, a Python or Ruby definition
  whose parameter list is not plain is kept whole (parameters are split at
  commas outside `[...]`; each must read as Python
  `[*|**]name[: annotation][= default]`, `/` or `*`, or Ruby `name[ = default]`;
  a default holding another `=`, or a Python `lambda`, is not plain), a Ruby
  definition whose `end`
  is followed by more of its statement (on the line or a `.` chain next) is kept
  whole, an import is skipped only when
  nothing follows it on its line (a Ruby `require` only in its exact form), an
  `eval` becomes an untranslatable note only as a single call with one string
  literal, a Go
  function whose lexed header does not read is kept whole (one without a body
  ends at its `;` or line end), and a Go struct whose fields are not each a
  simple `Name Type [tag]`, one per line or `;`-separated, is kept whole.

## Required Checks

```sh
cargo test -p garnet-convert
cargo test -p garnet-cli convert
```
