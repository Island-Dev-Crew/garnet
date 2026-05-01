# AGENTS.md — Examples Contract

## Scope

Owns public Garnet examples and MVP demonstration programs.

## Stable Contracts

- Examples should teach one clear concept each.
- Do not let examples claim production readiness beyond the current README/FAQ status.
- Novel agentic examples should show memory/actor/capability concepts explicitly rather than hiding them behind comments.
- Keep examples synchronized with parser/interpreter/checker behavior.

## Required Checks

Run relevant parser, interpreter, or CLI tests after changing examples. For runnable examples, prefer `garnet run` and `garnet check` smoke tests.
