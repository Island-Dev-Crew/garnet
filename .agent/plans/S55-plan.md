# S55 Plan — WASM hello-world

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S55.
Map: reconciled plan §77, §156 — WASM hello-world (adoption driver; S56 enabler).
Branch: `codex/s55-wasm`. Base: `origin/main` @ `706b5a0` (S54).

## Environment reality → honest-partial
wasm32 target NOT installed; wasm-pack/wasmtime ABSENT; garnet-interp pulls
`miette` `fancy` (wasm-portability blocker). Garnet has NO wasm backend (the
interpreter compiled to wasm is the in-browser model). ⇒ ship hello-world + a
readiness reporter that NAMES the blockers; build no wasm.

## Deliverables
- `examples/hello.garnet` — canonical hello-world (checks clean, runs).
- `scripts/garnet_wasm_readiness.py` — inventory the wasm path + name blockers
  (wasm32 target, wasm-pack, wasmtime, miette fancy); `--gate` guards owned bits
  (example + doc) only. `--format md|json`.
- `scripts/test_garnet_wasm_readiness.py` — 5 unit tests.
- Wire test + `--gate` into ci.yml agent-contracts.
- `F_Project_Management/GARNET_WASM_TARGET.md` — the build path + honest deferral.

## Dogfood
- `garnet run examples/hello.garnet` → "Hello from Garnet!". `garnet_wasm_readiness.py
  --format md` → owned bits ready, blockers named (incl. miette fancy); `--gate` 0.

## Honest scope (do not soften)
- NO wasm built, NO browser run claimed. wasm toolchain absent + interp needs a
  portability fix → deferred. The absent toolchain is an honest deferral, not a
  gated failure. No new readiness lane.

## Gates
- reporter + tests + ladder (zero Rust changed; workspace 0 failed). Ledger:
  `s54 → merged(5)` advanced this branch; `s55` rides with S56.
