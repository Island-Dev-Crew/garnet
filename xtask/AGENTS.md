# AGENTS.md — xtask Contract

## Scope

Owns repository automation that is too project-specific for ordinary cargo commands.

## Stable Contracts

- Keep xtask commands deterministic and CI-friendly.
- Prefer explicit named tasks over hidden side effects.
- If an xtask becomes a user-facing product command, promote it into `garnet-cli` with tests.

## Required Checks

```sh
cargo test -p xtask
```
