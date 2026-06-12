# AGENTS.md — Safe-Mode Checker Contract

## Scope

Owns safe-mode validation, CapCaps propagation, borrow/safety checks, and dependency audit helpers used by the CLI.

## Stable Contracts

- Safe mode must fail closed.
- CapCaps propagation must remain transitive: callers inherit or declare authority needed by callees.
- The propagator's capability representation is `capset::CapSet` — a `Copy`
  `u16` bitset over the closed cap set (RB-1). Propagation is bitwise OR,
  subset is `required & !declared == 0`, the diff-caps delta is XOR. Bit
  order is lexicographic-by-name so diagnostics keep `BTreeSet` iteration
  order. Unknown declared cap names survive only as the `OTHER` presence
  bit; their identity stays at the surface/audit layers (string-typed).
  Adding a capability name to the stdlib registry requires a matching
  `CapSet` bit — the `registry_caps_all_canonical` trap test fails closed
  otherwise.
- `capability_surface`/`caps_diff` keep full string fidelity (including
  unknown and wildcard names): a gained unknown capability must still gate
  as authority expansion in diff-caps.
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
