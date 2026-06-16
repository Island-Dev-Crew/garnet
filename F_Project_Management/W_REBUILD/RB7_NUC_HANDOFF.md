# RB-7 REPL — cross-OS proof handoff to the NUC lane (2026-06-16)

The RB-7 "joy" REPL (`garnet repl`) was built and demonstrated on **Apple M5 Pro
· macOS 26.5 · rustc 1.95.0**. This note hands the **cross-OS verification** to
the Windows NUC lane. **No cross-OS-complete claim is made from the Mac-only
proof** — that is exactly what this handoff is for.

## What is proven on Mac (this slice)

- `cargo build -p garnet-cli` + `cargo test -p garnet-cli repl` green (19 REPL
  unit tests: dispatch, `?doc`, `:caps`, completion candidates, multiline /
  dangling-annotation / unterminated-string-and-EOF detection, the plain-loop
  scripted session).
- `cargo clippy -D warnings` clean; full workspace **2030/0**.
- A recorded plain-mode session: `docs/demos/repl-session.txt`.
- `garnet-interp` still compiles to `wasm32-wasip1` (reedline is CLI-only, so the
  RB-6 wasm portability is preserved).

## What the NUC lane should verify on Windows

1. **Interactive TTY behaviour** (not exercised by the piped Mac transcript):
   - reedline starts on a Windows terminal (PowerShell + Windows Terminal);
   - history (Up/Down + reverse-search) works;
   - Tab completion shows the menu over commands / primitives / live bindings;
   - in-editor multiline (`{`/`(`/`[` and a leading `@caps` continue on `...>`);
   - `Ctrl-C` abandons the current line, `Ctrl-D` exits.
2. **Plain (non-TTY) parity:** piping `docs/demos/repl-session.txt` into
   `garnet repl` on Windows produces the same dispatch output (CRLF aside).
3. **`?doc` / `:caps`** render correctly (no terminal-encoding breakage on the
   `·` / box-drawing characters; substitute ASCII if a Windows console mangles
   them — flag it rather than asserting parity).

## Handoff status

- Owner: NUC lane (Windows × Claude/Codex).
- Inputs: this note + `docs/internals/repl.md` + `docs/demos/repl-session.txt`.
- Expected artifact: a fleet-report entry (or a short cross-OS note) recording
  the Windows TTY result. Until that lands, the REPL's cross-OS status is
  **Mac-proven only**.
