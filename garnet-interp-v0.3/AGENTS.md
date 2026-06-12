# AGENTS.md — Interpreter Contract

## Scope

Owns managed-mode tree-walk execution, expression evaluation, stdlib bridging, and interpreter examples/tests.

## Stable Contracts

- Keep interpreter behavior aligned with parsed AST and Mini-Spec semantics.
- Do not bypass capability metadata when invoking stdlib or OS-facing operations.
- Prefer explicit errors over silent no-ops for unsupported language features.
- Crash surface (RB-2): the crate carries
  `#![deny(clippy::unwrap_used, clippy::expect_used)]` (tests exempt via
  `cfg_attr`). Sanctioned escapes are in-line `// INVARIANT:` allows only.
  Checked integer division/remainder overflow (`i64::MIN / -1`, `% -1`) is a
  `RuntimeError::Overflow` diagnostic with the SAME message as the VM —
  never a process abort. Add/sub/mul overflow policy (wraps in release,
  aborts in debug) is an open language decision, named-deferred — do not
  silently change it.
- Maintain compatibility with `garnet run`, `garnet eval`, and `garnet test` expectations.

## Required Checks

```sh
cargo test -p garnet-interp
cargo test -p garnet-cli run eval test
```

Run workspace tests when changing shared semantics.
