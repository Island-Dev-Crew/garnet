# S57 Plan — idiomatic open corpus (Lattner)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S57.
Map: reconciled plan §157 — idiomatic open corpus.
Branch: `codex/s57-corpus`. Base: `origin/main` @ `5c0fe51` (S56).

## Deliverables
- `examples/idiomatic/typed_errors.garnet` — typed `rescue e: AppError` (S42
  policy, not catch-all). `examples/idiomatic/state_machine.garnet` — exhaustive
  `match` over a finite enum + named `@caps`. Both `check` 0 diagnostics + run.
- `scripts/garnet_idiomatic_corpus.py` — harness: each must `check` to 0
  diagnostics (the idiomatic bar) + `run` to recorded output. md/json.
- `scripts/test_garnet_idiomatic_corpus.py` — 4 pure-logic + 1 skipUnless live.
- `examples/idiomatic/README.md` — names the idioms.
- Wire the test into ci.yml agent-contracts.

## Dogfood
- `garnet_idiomatic_corpus.py --format md` → 2/2 clean + running (exit 0).

## Honest scope (do not soften)
- A style/discipline corpus, not perf/coverage. "Idiomatic" = clean checker
  output (0 diagnostics) + hardening-band idioms, proven deterministically.
- No new readiness lane.

## Gates
- harness + tests + ladder (zero Rust changed; workspace 0 failed). Ledger:
  `s56 → merged(5)` advanced this branch; `s57` rides with S58.
