# AGENTS.md — Standard Library Contract

## Scope

Owns Garnet stdlib primitives and their capability metadata.

## Stable Contracts

- Every OS-facing primitive must declare accurate capability metadata.
- Do not add file, network, process, or time authority without updating CapCaps expectations and tests.
- Keep primitives small and predictable; richer behavior belongs in higher-level libraries or examples.

## Required Checks

```sh
cargo test -p garnet-stdlib
cargo test -p garnet-check
cargo test -p garnet-interp
```
