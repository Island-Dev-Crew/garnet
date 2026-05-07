# AGENTS.md — Interpreter Contract

## Scope

Owns managed-mode tree-walk execution, expression evaluation, stdlib bridging, and interpreter examples/tests.

## Stable Contracts

- Keep interpreter behavior aligned with parsed AST and Mini-Spec semantics.
- Do not bypass capability metadata when invoking stdlib or OS-facing operations.
- Prefer explicit errors over silent no-ops for unsupported language features.
- Maintain compatibility with `garnet run`, `garnet eval`, and `garnet test` expectations.

## Required Checks

```sh
cargo test -p garnet-interp
cargo test -p garnet-cli run eval test
```

Run workspace tests when changing shared semantics.
