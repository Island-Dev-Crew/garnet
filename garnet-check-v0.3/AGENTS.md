# AGENTS.md — Safe-Mode Checker Contract

## Scope

Owns safe-mode validation, CapCaps propagation, borrow/safety checks, and dependency audit helpers used by the CLI.

## Stable Contracts

- Safe mode must fail closed.
- CapCaps propagation must remain transitive: callers inherit or declare authority needed by callees.
- Diagnostics should identify the missing or malformed safety surface directly.
- Do not weaken safety checks to make examples pass; fix the examples or specs.

## Required Checks

```sh
cargo test -p garnet-check
cargo test -p garnet-cli check build verify
```
