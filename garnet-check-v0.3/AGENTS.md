# AGENTS.md — Safe-Mode Checker Contract

## Scope

Owns safe-mode validation, CapCaps propagation, borrow/safety checks, and dependency audit helpers used by the CLI.

## Stable Contracts

- Safe mode must fail closed.
- CapCaps propagation must remain transitive: callers inherit or declare authority needed by callees.
- Transitivity holds across call-graph cycles (U-117, cured 2026-09-18).
  `caps_graph::transitive_caps` is an iterative Tarjan-SCC walk with
  DeRemer–Pennello propagation: every member of a strongly connected
  component receives the union of the component's authority plus everything
  reachable from it, and recursion depth never grows with graph depth. A
  cycle is not a propagation boundary; `caps_graph_cycle_tests.rs` holds the
  reachability oracle and deep-chain regression tests that keep it that way.
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
- Memory declarations charge `mem` (D-04b, cured 2026-09-18). `CapsGraph::build`
  attributes every `memory <kind> ..` declaration (top-level, module, actor)
  to the program entry `main` as a `memory::<kind>` primitive callee, so
  `garnet check` reports `caps coverage` for an entry that declares a tier
  without `@caps(mem)`, matching the runtime pre-pass. Library modules
  without `main` are not charged; the runtime still gates them at load.
- Enum variant construction is checked in safe functions (D-107, cured
  2026-09-18). `match_coverage::check_variant_construction` runs on every
  `Enum::Variant(args...)` call and bare `Enum::Variant` path the safe-mode
  walk reaches, resolving the enum exactly as match arms do (modules, `use`
  aliases). An unknown variant, a unit variant given a payload, a payload
  variant used bare, or a payload variant with the wrong field count is a
  `SafeModeViolation`; a name that is an associated fn of an `impl` on that
  type is a call, not a variant, and is not judged. Paths that do not resolve
  to exactly one user enum (structs, module fns, prelude `Ok`/`Some`) are left
  alone. Managed `def` bodies are outside this walk, like the rest of the
  safe-mode contract; `tests/variant_construction.rs` pins that scope.
- Static bounded-loop verification is conservative: in `fn`, `@safe`, or
  `@bounded(...)` functions, uncheckable loops fail closed; only explicitly
  proven literal finite loops, literal counter `while` loops, and
  immediate-exit loop bodies are accepted. Do not describe this as Wasmtime fuel,
  runtime loop metering, VM enforcement, or OS sandbox enforcement.
- Diagnostics should identify the missing or malformed safety surface directly.
  A `caps coverage` violation's `via` names the qualified gated primitive
  that requires the missing capability, then the named path that reaches
  it when the call is not direct: `fs::write_file (via b → a)`,
  `fs::write_file (via helper → .go() → A::go)`. The path is the first
  reaching callee at every hop in `BTreeSet` order, each function entered
  once (so a cycle is walked, not re-walked), and elided to its first and
  last three hops past six. `caps_graph_cycle_tests.rs` pins the format;
  a bare `via` such as `(via a)` that names only a hop is a regression.
- Do not weaken safety checks to make examples pass; fix the examples or specs.

## Required Checks

```sh
cargo test -p garnet-check
cargo test -p garnet-cli
```

Cargo accepts one positional test filter, so `cargo test -p garnet-cli check
build verify` is rejected; run the crate's tests whole (the `check`, `build`
and `verify` acceptance tests live in `conformance_skeleton.rs` and
`verify_gate_acceptance.rs`).
