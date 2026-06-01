# AGENTS.md — Safe-Mode Checker Contract

## Scope

Owns safe-mode validation, CapCaps propagation, borrow/safety checks, and dependency audit helpers used by the CLI.

## Stable Contracts

- Safe mode must fail closed.
- CapCaps propagation must remain transitive: callers inherit or declare authority needed by callees.
- Static bounded-loop verification is conservative: in `fn`, `@safe`, or
  `@bounded(...)` functions, uncheckable loops fail closed; only explicitly
  proven literal finite loops, literal counter `while` loops, and
  immediate-exit loop bodies are accepted. Do not describe this as Wasmtime fuel,
  runtime loop metering, VM enforcement, or OS sandbox enforcement.
- Diagnostics should identify the missing or malformed safety surface directly.
- Do not weaken safety checks to make examples pass; fix the examples or specs.

## Required Checks

```sh
cargo test -p garnet-check
cargo test -p garnet-cli check build verify
```
