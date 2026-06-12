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
  of the row contract (RB-7 `?doc` consumes them).
- Do not add file, network, process, or time authority without updating CapCaps expectations and tests.
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
