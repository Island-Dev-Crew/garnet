# S40 Plan — explosive-operation / default-ceiling analysis

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S40 (closes the
v0.8 foundation band, S31–S40).
Branch: `codex/s40-explosive-ops`.

## Goal
Static identification of unbounded / explosive operations + a default-ceiling
policy. Closes the foundation band.

## Design (bounded, honest)
- **`garnet-check-v0.3/src/explosive.rs`** (new): a **compiler-exhaustive**
  AST visitor (`Block`/`Stmt`/`Expr`, recursing through every variant — match
  arms are exhaustive so no nested site is silently missed) that flags two
  unambiguous explosive constructs:
  - `Stmt::Loop` — an **unconditional loop** (unbounded iteration; static
    termination is undecidable, so every `loop` is flagged regardless of an
    internal `break`).
  - `Expr::Spawn` — actor **fan-out** (unbounded unless governed).
  - `explosive_ops(module) -> Vec<FnExplosiveReport>` (sorted by fn name): per
    function, the ops (kind + span) plus `has_bounded` / `has_fan_out` (does the
    function declare the governing annotation?).
  - **Default-ceiling policy** as documented constants: `DEFAULT_LOOP_CEILING`,
    `DEFAULT_SPAWN_FANOUT` — what applies when the op is not governed by a
    declared bound (`@bounded` for loops/fuel, `@fan_out` for spawn).
- **`garnet ceilings <file>`** (`cmd/ceilings.rs`): reports each function's
  explosive ops, whether each is governed by a declared bound or falls back to
  the default ceiling, and the honest deferral note.

## Load-bearing dogfood
- `loop { ... }` → flagged `UnconditionalLoop`; `spawn g()` → flagged `Spawn`;
  both detected even when nested (inside `if` / a call arg) — the visitor is
  exhaustive.
- A `@bounded`-annotated function with a loop reports "governed by @bounded";
  an un-annotated one reports "default loop ceiling N applies (declare @bounded)".
- `garnet ceilings <clean-file>` → "no explosive operations".

## Crates touched
- `garnet-check-v0.3`: new `explosive.rs` + re-export.
- `garnet-cli`: `cmd/ceilings.rs` + dispatcher + help, new tests.
- Reuses S39 `@bounded` and the existing `@fan_out` annotations as the governing
  declarations.

## End-state / gates
- No new readiness lane (the contract lane table does not mandate one for S40).
- fmt / clippy -D / test --workspace / doc -D / deny / --check-no-regression /
  conformance / python — all green. CHANGELOG + contract S40 state. **Mark the
  phaseA ledger complete (S40 → merged).**
- Dogfood bundle → PR → CLI-merge → ledger: s40 advance rides with the next
  band's first PR (or a closing commit).

## Honest scope / out of scope
- Static IDENTIFICATION + default-ceiling POLICY only. Runtime ENFORCEMENT of
  ceilings lowers to the S39 `@bounded` / Wasmtime-fuel path — **deferred**
  (wasmtime absent); no ceiling is faked.
- Explosive set = unconditional `loop` + `spawn`. Recursion is already addressed
  by `@max_depth` + the caps call graph; unbounded collection growth and other
  patterns are follow-ups (documented coverage, not a silent gap).
