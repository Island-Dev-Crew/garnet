# S42 Plan — typed Result / error policy

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S42 (detailed
block authored this slice).
Map: reconciled plan — "typed Result / error policy (Ronacher: agents over-catch
exceptions)."
Branch: `codex/s42-error-policy`.

## Goal (codify + enforce one rule; no new control-flow semantics)
`core::result` combinators (S26) and `try`/`rescue`/`ensure`/`raise` already
exist. S42 codifies the typed-`Result`-first policy and enforces the **over-catch**
guard: a catch-all `rescue` (no exception type) is a non-fatal advisory.

## Deliverables
- `C_Language_Specification/GARNET_ERROR_POLICY.md` — the two error channels +
  the over-catch anti-pattern + the advisory's honest scope.
- `garnet_check::overcatch_sites(module)` — compiler-exhaustive AST walk
  collecting catch-all `rescue` clauses (`ty.is_none()`).
- `CheckError::OverCatch` — NON-FATAL advisory (excluded from `CheckReport::ok`),
  emitted by `check_module`; surfaced in `garnet check` human + JSON
  (`check.over_catch`, severity info via S34 diagnostics). A typed rescue
  (`rescue e: T`) is not flagged.

## Dogfood
- `try { } rescue e { }` → advisory present, exit 0; JSON `check.over_catch`,
  `ok:true`. `rescue e: IoError { }` → not flagged. 3 unit + 3 integration tests.

## End-state / gates
- No new readiness lane (not mandated). Full ladder green; CHANGELOG + contract
  S42 block. Dogfood → PR → CLI-merge → `s42` advance rides with the S43 PR.

## Honest scope
- Advisory only: no exit-code change, no auto-rewrite, no ban on rescues.
- No typed-exception hierarchy / checked-exceptions; exception types in `rescue`
  are surface syntax the analyzer reads, not a new type system.
