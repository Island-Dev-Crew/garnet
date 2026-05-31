# S39 Plan — `@bounded` (resource bounds)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S39.
Branch: `codex/s39-bounded`.

## Goal ([GRAFT] wrap-don't-rebuild)
A bounded-resource annotation (CPU/mem/mailbox). Lower to **Wasmtime fuel**
metering. `@mailbox(N)` already exists (mailbox capacity); S39 adds the CPU/fuel
budget annotation `@bounded(N)` — N fuel units (Wasmtime-fuel steps) — and the
extraction + reporting. The Wasmtime-fuel *enforcement* is the wrap.

## Environment reality (honest)
`wasmtime` / `wasm-tools` are **absent** here. So S39 ships the **annotation +
its declared bound + a report** (real, doable) and honestly labels the
enforcement as deferred to the Wasmtime-fuel backend (declared, not yet
fuel-enforced). No fuel meter is faked.

## Design — `@bounded(N)` (single int; mirrors `@max_depth` / `@mailbox`)
Five additive sites for the new `Annotation::Bounded(i64, Span)`:
1. `garnet-parser-v0.3/src/ast.rs` — the enum variant.
2. `garnet-parser-v0.3/src/grammar/functions.rs` — the `"bounded"` parse arm.
3. `garnet-check-v0.3/src/lib.rs` — validation: `N > 0` (fuel budget must be
   positive); no upper bound (fuel budgets are legitimately large).
4. `garnet-cst/src/convert.rs` — the `"bounded"` CST→AST arm (rowan path; S15).
5. `garnet-cli/src/cmd/doc.rs` — add to the span-returning match.
Then:
- `garnet-check-v0.3`: `pub fn bounded_functions(module) -> Vec<(String, i64)>`
  (sorted by fn name) — reusable extraction.
- `garnet-cli/src/cmd/bounds.rs`: `garnet bounds <file>` — reports each function's
  declared fuel budget + the honest "Wasmtime-fuel enforcement deferred (wasmtime
  absent → declared, not fuel-enforced)" note.

## Load-bearing dogfood
- `@bounded(1000)\ndef f(){…}` parses; `bounded_functions` returns `[("f", 1000)]`.
- `@bounded(0)` / `@bounded(-1)` → a checker error (fuel budget must be positive).
- `@bounded(1000)` round-trips through the rowan CST (convert.rs arm).
- `garnet bounds <file>` lists the budgets + the honest enforcement note.

## Crates touched
- `garnet-parser-v0.3` (enum + parse), `garnet-check-v0.3` (validation +
  `bounded_functions`), `garnet-cst` (CST convert), `garnet-cli` (`cmd/bounds.rs`
  + dispatcher + help). All Annotation matches are compiler-checked.

## End-state / gates
- No new readiness lane (the contract lane table does not mandate one for S39).
- fmt / clippy -D / test --workspace / doc -D / deny / --check-no-regression /
  conformance / python — all green. CHANGELOG + contract S39 state. Dogfood
  bundle → PR → CLI-merge → `s39` advance rides with the S40 PR.

## Honest scope / out of scope
- `@bounded(N)` declares a CPU/fuel budget; **enforcement lowers to Wasmtime
  fuel** — a wrap that is deferred here (`wasmtime` absent). The bound is
  declared + reported, not yet runtime-enforced. No fuel meter is faked.
- Mem bounds and mailbox (`@mailbox`, already present) are out of scope for the
  `@bounded` CPU/fuel budget; a unified resource-bound syntax is a follow-up.
