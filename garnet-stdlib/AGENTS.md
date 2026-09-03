# AGENTS.md — Standard Library Contract

## Scope

Owns Garnet stdlib primitives and their capability metadata.

## Stable Contracts

- Every OS-facing primitive must declare accurate capability metadata.
- The registry row is THE dispatch declaration (RB-3): `Binding` (bare /
  qualified / unbridged) and `Guard` (declared / gate / gate+entry) columns
  drive the interpreter's derived `install()`; rows keep the textual
  `p("module", "name", arity, caps, Layer::X, Stability::Y, ...)` shape the
  layer-gate and promotion-gate scripts regex-parse. Doc strings are part
  of the row contract (RB-7 `?doc` will be their first consumer).
- Do not add file, network, process, or time authority without updating CapCaps expectations and tests.
- Every host-authority row carries `Guard::GateEntry`, not `Guard::Gate`
  (U-91). `Gate` alone lets a non-entry frame supply the capability, which
  launders the entry's budget through any call edge the checker cannot see.
  The `Gate` variant remains defined and currently has zero members. The test
  that holds that to zero is `gate_count_matches_the_audited_runtime_backstop`,
  which asserts the pair `(gate_count, gate_entry_count) == (0, 15)`; adding any
  `Gate` row turns it red. `entry_gates_are_the_whole_gated_surface` does NOT
  catch that case — it only compares the `GateEntry` names to the expected
  fifteen, so a new `Gate` row leaves it green.
  Adding a row with `Gate` requires stating in the same change why the entry
  budget must not bound it.
- Keep primitives small and predictable; richer behavior belongs in higher-level libraries or examples.
- Crash surface (RB-2): the crate carries
  `#![deny(clippy::unwrap_used, clippy::expect_used)]` (tests exempt via
  `cfg_attr`). The single allowlisted site is `crypto.rs::hmac_sha256`
  (`// INVARIANT:` — Hmac accepts any key length per RFC 2104); converting
  it to a `Result` would be a public-API change, keep the allow.

## Required Checks

```sh
cargo test -p garnet-stdlib
cargo test -p garnet-check
cargo test -p garnet-interp
```
