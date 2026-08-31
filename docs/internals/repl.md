# The Garnet REPL (RB-7)

`garnet repl [file.garnet]` starts an interactive session. Pass a file to
preload its top-level items first.

```sh
garnet repl                 # empty session
garnet repl examples/x.garnet   # preload a file, then drop into the prompt
```

## What you can type

| Input | Effect |
|-------|--------|
| `1 + 2 * 3` | evaluate an expression and print its value (`=> 7`) |
| `let x = 41` | a top-level binding |
| `def name(..) { .. }` | register a function (also `struct`/`enum`/`impl`/`actor`/…) |
| `@caps(fs)\ndef f() { .. }` | an annotated item — the annotation and its `def` are read as one block |
| `?doc <name>` | show a primitive's doc, arity, and required capabilities |
| `?<name>` | shorthand for `?doc <name>` (works for primitives and your own functions) |
| `:caps` | the session's declared + available authority surface |
| `:help` / `:h` | command help |
| `:quit` / `:q` / `Ctrl-D` | leave |

Composite values print with a type tag, e.g. `=> [1, 2, 3]  : Array`,
`=> <fn add>  : Fn`.

## `?doc` — primitive and function reference

`?doc` reads the live stdlib registry (`PrimMeta`): the module-qualified name,
arity, required `@caps`, layer, stability, and the one-line doc.

```text
garnet> ?doc read_file
fs::read_file  (1-arg primitive)
  caps: fs
  layer: Std · stability: Stable
  Read a UTF-8 file as String.
```

A bare name resolves to its qualified primitive when unambiguous (`?read_file` →
`fs::read_file`); a name you defined this session shows its arity and declared
caps.

## `:caps` — the authority surface (declared, not enforced)

`:caps` prints two evidence-scoped sections: the capabilities the **loaded functions
declare** (`@caps(...)`), and the **available primitives grouped by the
capability each requires**.

This is a *declared / available* surface, **not an enforced runtime budget**. A
bare call at the prompt holds no capability frame; `@caps` is enforced
per-function at entry (interpreter S90). The header in the output says so.

## Line editing

On a real terminal the REPL uses [`reedline`](https://crates.io/crates/reedline):

- **History** — Up/Down (and reverse-search).
- **Multiline** — an open `{`/`(`/`[`, or a leading annotation awaiting its
  item, continues the input on a `...>` line until it balances.
- **Tab completion** — over REPL commands, every stdlib primitive (bare and
  qualified), and the bindings live in your session.

When stdin is **not** a terminal (a pipe, CI, or a recorded demo), the REPL
falls back to a plain line reader that runs the *same* command dispatch, so
scripted transcripts are faithful.

## Architecture note (why reedline is CLI-only)

Reedline and the REPL ergonomics live in the `garnet-cli` crate, **not** in
`garnet-interp`. The interpreter crate is kept terminal-dependency-free so it
continues to compile to `wasm32-wasip1` (see the RB-6 backend/IR memo); a
terminal line editor would pull `crossterm` and break that portability. The CLI
is not a wasm target, so it is the right home for the joy.

The command dispatch (`dispatch`, `?doc`, `:caps`, completion candidates,
multiline detection) is a pure, unit-tested core; reedline is a thin input
layer over it.

## Evidence

A recorded session is at [`docs/demos/repl-session.txt`](../demos/repl-session.txt)
(Apple M5 Pro, macOS 26.5, rustc 1.95.0, plain/non-TTY mode). The interactive
line-editing behaviours (history, live Tab completion, in-editor multiline) are
TTY-only; their logic is covered by the unit tests in
`garnet-cli/src/cmd/repl.rs`. Cross-OS verification is delegated to the NUC lane
— see `F_Project_Management/W_REBUILD/RB7_NUC_HANDOFF.md`. **No cross-OS-complete
claim is made from this Mac-only proof.**
