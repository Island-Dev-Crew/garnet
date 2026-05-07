# AGENTS.md — Memory Core Contract

## Scope

Owns the reference implementation for Garnet's working, episodic, semantic, and procedural memory abstractions.

## Stable Contracts

- Preserve the four memory-kind boundary: working, episodic, semantic, procedural.
- Treat `AGENTS.md` and workflow contracts as procedural-memory analogs when designing future tooling.
- Never hide sink, persistence, or machine-key failures; memory failures must be observable.
- Keep tests isolated from machine-local key races and cache state.

## Required Checks

```sh
cargo test -p garnet-memory
cargo test -p garnet-cli cache
```

Run workspace tests if cache or machine-key behavior changes.
