# Garnet Error Policy (v0.8, S42)

Status: codifies the typed-`Result`-first error policy and the **over-catch
advisory** that enforces a piece of it. Descriptive of the shipped surface
(`core::result` combinators, S26; `try`/`rescue`/`ensure`/`raise`, Mini-Spec §7);
no new control-flow semantics are introduced.

## 1. Two error channels

Garnet has two error channels, used for different situations:

1. **Typed `Result` (preferred).** `core::result::{ok, err, map, and_then,
   or_else, …}` (S26) model expected, recoverable failures as **values**, composed
   railway-style. This is the default for any failure a caller is expected to
   handle.
2. **Exceptions (`raise` / `try`/`rescue`/`ensure`).** For *truly exceptional*,
   non-local conditions. `ask` across an actor boundary is Result-returning, so a
   handler failure surfaces as a value, not a panic crossing the boundary (S41).

## 2. The over-catch anti-pattern (Ronacher)

A **catch-all `rescue`** — a `rescue` clause with **no type** (`rescue { … }` or
`rescue e { … }`) — swallows *every* exception, including ones the code did not
anticipate. In agent-generated code this is a common, dangerous habit ("agents
over-catch exceptions"): real bugs get silently swallowed.

**Policy:** prefer a typed `Result` for expected failures; when you do use
`rescue`, **name the exception type** you intend to handle (`rescue e: IoError {
… }`) so unanticipated exceptions propagate.

### Enforcement (advisory)

`garnet check` emits a **non-fatal advisory** (`check.over_catch`) for each
catch-all `rescue` (a rescue with no type). It is an advisory, not an error: it
never changes the exit code (consistent with `@stability` advisories) — it steers
toward typed `Result` / typed rescues without breaking existing code.

`garnet_check::overcatch_sites(module)` is the reusable analyzer behind it.

## Honest scope (what this policy does NOT do)

- It does **not** ban exceptions or catch-all rescues — the over-catch check is an
  **advisory only** (no exit-code change, no auto-rewrite).
- It does **not** introduce a typed-exception hierarchy or checked-exceptions;
  exception *types* in `rescue` clauses are surface syntax the analyzer reads, not
  a new type system.
- Result combinator coverage is whatever `core::result` ships (S26); expanding it
  is a stdlib-promotion concern (later band).
