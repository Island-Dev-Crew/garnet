# Garnet Formal Proof Plan

Date: 2026-05-14
Status: planned

## Goal

Move formal assurance from aspirational claims to a small, auditable, runnable
artifact track with clear first milestones and explicit scope limits.

## Falsifiable first milestone

- One mechanized lemma for a tiny safe-mode core fragment (e.g., one direct
  own-move rejection statement).
- Keep the proof build reproducible and checked in CI evidence.

Acceptance command:

```sh
# replace <proof-target> with the active proof artifact command when available
cargo test --manifest-path proofs/Cargo.toml
```

Current target is scaffold-only; there is no active proof crate in this repository yet.

## Proof scope (current)

- Do not claim blanket soundness for safe-mode, borrow semantics, or runtime
  concurrency until each theorem is implemented, checked, and linked to a scope
  statement.
- Prefer small lemmas with direct correspondence to executable test slices.
- Track the first three proof artifacts independently so partial truth is explicit:
  parser-level semantics, checker-level obligations, runtime-level invariants.

## Milestone 1: Lemma scaffold

1. Create a proof workspace (or link a repo) for a constrained core fragment.
2. Prove one small safe-mode property and wire a local checker.
3. Document assumptions and non-modeled features in `CURRENT_STATE.md` and the
   conformance ledger.

## Milestone 2: Regression and coverage

1. Add one failing property-driven test/lemma pair for each major safe-mode slice
   that has mature behavior (for example direct-affine move ownership).
2. Keep theorem naming in sync with corresponding conformance test identifiers.
3. Include proof artifact checks in the dogfood evidence bundle.

## Milestone 3: Expansion and alignment

1. Expand the proven core by one language construct at a time.
2. Keep the executable/contract/lemma boundary explicit.
3. Require human + CI verification before any public claim of production-grade proof
   coverage.
