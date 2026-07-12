# AGENTS.md — Interpreter Contract

## Scope

Owns managed-mode tree-walk execution, expression evaluation, stdlib bridging, and interpreter examples/tests.

## Stable Contracts

- Keep interpreter behavior aligned with parsed AST and Mini-Spec semantics.
- Do not bypass capability metadata when invoking stdlib or OS-facing operations.
- Source loaded for `garnet run` must register top-level `let`/`const`
  initializers under the selected program entry's `@caps` frame, before
  `main` is called. Load-time host authority is not allowed to execute outside
  a capability frame.
- Prefer explicit errors over silent no-ops for unsupported language features.
- Registry-derived dispatch (RB-3): `stdlib_bridge::install()` is ONE loop
  joining `garnet_stdlib::registry::all_prims()` (Binding/Guard/arity
  columns) against the `#[garnet_primitive]` adapter table — never add a
  hand-written registration row. Adapter bodies keep the literal
  `require_capability` backstops as grep-able source text (gate scripts
  parse this file). The four `memory::*` natives live in `BRIDGE_ONLY`;
  the registry-join trap tests + `guard_column_matches_runtime_backstop_behavior`
  make any registry/adapter drift a red test.
- Crash surface (RB-2): the crate carries
  `#![deny(clippy::unwrap_used, clippy::expect_used)]` (tests exempt via
  `cfg_attr`). Sanctioned escapes are in-line `// INVARIANT:` allows only.
  Integer arithmetic is checked by default for division, remainder,
  addition, subtraction, multiplication, and unary negation on interpreter
  and VM paths (RFC-0002). Overflow produces the byte-identical controlled
  diagnostic proven by `overflow_guards.rs` and `overflow_parity.rs` —
  never a process abort. Explicit wrapping operations remain deferred and
  must not be implied.
- `@max_depth(N)` is valid only for `1..=64`; the interpreter must reject
  invalid bounds at runtime too, not treat a checker failure as an oversized
  executable ceiling.
- Maintain compatibility with `garnet run`, `garnet eval`, and `garnet test` expectations.
- REPL introspection (RB-7): `Interpreter::live_binding_names` / `lookup_binding`
  and `Env::local_names` are **additive, read-only** accessors the CLI REPL uses
  for completion + `?doc`/`:caps`. They must stay read-only (no eval side
  effects) and pull no terminal/line-editor dependency — this crate must keep
  compiling to `wasm32-wasip1` (RB-6). Keep `repl.rs` here minimal and
  `std::io`-only; the rich REPL lives in `garnet-cli`.

## Required Checks

```sh
cargo test -p garnet-interp
cargo test -p garnet-cli run eval test
```

Run workspace tests when changing shared semantics.
