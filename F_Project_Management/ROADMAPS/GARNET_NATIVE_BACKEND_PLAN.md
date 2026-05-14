# Garnet Native Backend Roadmap

Date: 2026-05-14
Status: planned

## Goal

Create a native backend track that is independently testable and separate from the
managed/interpreter implementation, so Garnet can grow toward production-grade
backend behavior without obscuring current runtime truth.

## Falsifiable first milestone

- Parse, check, and run one tiny integer arithmetic program through backend output.
- Keep the check runnable from a reproducible command.

Acceptance command:

```sh
cargo test --workspace --test native_backend_smoke -- --nocapture
```

The command may fail until the backend crate and test harness exist; that failure is
an acceptable placeholder only for the pre-existing scaffold stage.

## Scope

- Keep the first milestone narrow: one language subset, one compiler pass path,
  one emitted backend artifact, one runtime execution check.
- Do not mix native backend claims into parser/managed/runtime readiness claims.
- Update this file with every milestone boundary crossed (IR, codegen, link/run).

## Milestone 1: Minimal parse/check/run backend

1. Add a native backend crate entrypoint and a tiny input source contract.
2. Add an end-to-end smoke test that parses and checks a small program and emits
   a backend artifact.
3. Add execution verification that consumes that artifact and returns expected
   output.
4. Report any runtime/path/ABI limitations in current-state-facing docs.

## Milestone 2: Artifact confidence and deterministic build

1. Add deterministic backend output checks (diff-stable or hash-stable where possible).
2. Add integration tests for two additional small programs.
3. Record the known subset boundaries next to the conformance matrix.

## Milestone 3: Scope expansion

1. Expand language subset coverage in increments tied to parser/checker milestones.
2. Add native code quality gates that do not claim full compiler production status.
3. Keep the managed runtime path default for broader behavior until native parity exists.
