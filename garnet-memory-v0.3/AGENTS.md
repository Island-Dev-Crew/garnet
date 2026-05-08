# AGENTS.md — Memory Core Contract

## Scope

Owns the reference implementation for Garnet's working, episodic, semantic, and procedural memory abstractions.

## Stable Contracts

- Preserve the four memory-kind boundary: working, episodic, semantic, procedural.
- Keep the cycle-collection reference path honest: `cycle.rs` is an
  observable fixture/model for Mini-Spec §4.5 behavior, not the production
  allocator-integrated ARC collector.
- Treat `AGENTS.md` and workflow contracts as procedural-memory analogs when designing future tooling.
- Never hide sink, persistence, or machine-key failures; memory failures must be observable.
- Keep tests isolated from machine-local key races and cache state.

## Required Checks

```sh
cargo test -p garnet-memory --test cycle
cargo test -p garnet-memory
cargo test -p garnet-cli cache
```

Run workspace tests if cache or machine-key behavior changes.
