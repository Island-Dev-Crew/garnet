# S41 Plan — async/concurrency contract (first v0.8 hardening slice)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S41 (band line
promoted to a detailed block this slice).
Map: reconciled plan — "Async/concurrency contract (Codex S41) … @bounded
includes mailbox size (actors); dual-mode implies a concurrency model."
Branch: `codex/s41-async-contract`.

## Goal (codify the EXISTING model — no new semantics)
Garnet's concurrency is **actors**, not async/await (`async` is reserved for a
future edition — S32). The model is already built in `garnet-actor-runtime`
(OS-thread + bounded mpsc mailbox; `@mailbox` override; Result-returning `ask`;
hot reload) and Mini-Spec §9. S41 makes it a **canonical contract** + a checkable
surface. This is honest codification, not strategy invention.

## Deliverables
- **`C_Language_Specification/GARNET_CONCURRENCY_CONTRACT.md`** — the canonical
  contract: actors, bounded mailboxes (no unbounded-mailbox DoS), ask vs tell
  protocols, spawn/`@fan_out`, `@bounded`, `@nonsendable` Sendable boundary, hot
  reload. Honest-scope section lists what is deferred (no async/await;
  `@nonsendable` enforcement deferred; fuel enforcement deferred; structured
  concurrency/cancellation future).
- **`garnet_check::concurrency_surface(module)`** → `Vec<ActorContract>`
  (per-actor protocols classified ask/tell via `return_ty.is_some()`, handler
  count; sorted, deterministic).
- **`garnet concurrency <file>`** — reports the contract surface + the model note.

## Load-bearing dogfood
- An actor with `protocol get() -> Int` (ask) and `protocol incr()` (tell)
  reports them correctly classified; a non-actor module reports "no actors".
- 2 unit + 2 integration tests; CLI smoke verified.

## Crates touched
- `garnet-check-v0.3`: new `concurrency.rs` + re-export.
- `garnet-cli`: `cmd/concurrency.rs` + dispatcher + help.
- New spec doc under `C_Language_Specification/`.

## End-state / gates
- No new readiness lane (the contract lane table does not mandate one for S41).
- fmt / clippy -D / test --workspace / doc -D / deny / --check-no-regression /
  conformance / python — all green. CHANGELOG + contract S41 block. Dogfood
  bundle → PR → CLI-merge → `s41` advance rides with the S42 PR.

## Honest scope / out of scope
- Documents what is BUILT; no new concurrency semantics are introduced.
- `@nonsendable` cross-boundary enforcement and `@bounded`/fuel runtime
  enforcement are deferred (declared + reported, not enforced — no faking).
- No async/await (reserved for a future edition); structured concurrency /
  cancellation beyond actor lifecycle + Result-ask is future work.
