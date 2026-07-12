# AGENTS.md — Primitive-Dispatch Macros Contract

## Scope

Owns the RB-3 registry-derived dispatch proc-macros: `#[garnet_primitive]`
(one attribute per native adapter) and `#[garnet_primitive_module]` (the
per-module collector that emits `entries()` for the interpreter's derived
`install()`).

## Stable Contracts

- The macros generate registration only, never semantics: adapter bodies,
  `Value` conversions, and the literal `require_capability` capability
  backstops stay as ordinary source text in `stdlib_bridge.rs` (the
  caps-enforcement gate greps that file's literal text).
- Generated code references `crate::value::NativeFn` — expansion sites must
  be inside `garnet-interp`.
- Dispatch metadata (binding mode, arity, caps, layer, stability, doc) is
  NOT declared here: it lives in `garnet_stdlib::registry` PrimMeta rows;
  an adapter declares only its qualified key and body. A key without a
  registry row (or a row without an adapter) must fail the interpreter's
  registry-join tests deterministically — never silently bind or skip.
- Duplicate keys inside a module are a compile error.
- No new external dependencies beyond the already-locked proc-macro stack
  (proc-macro2/quote/syn). Consequence, recorded honestly: compile-error
  paths through the proc-macro entry points are reasoned + helper-tested,
  not yet trybuild-exercised (adding trybuild = a new external dep). A
  trybuild compile-fail suite is required before materially widening the
  Core Ring macro surface — do not grow the macro grammar on
  reasoned-only compile-error coverage.
- One adapter binds exactly ONE key: a fn carrying multiple
  `#[garnet_primitive]` attributes is a compile error (fail closed — a
  dual-key adapter would silently alias dispatch; found by adversarial
  probe). Qualified attribute paths are collected, never skipped.

## Required Checks

```sh
cargo test -p garnet-prim-macros
cargo test -p garnet-interp
```
