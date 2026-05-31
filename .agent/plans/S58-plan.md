# S58 Plan — benchmark campaign

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S58.
Map: reconciled plan §157 — benchmark campaign.
Branch: `codex/s58-benchmark`. Base: `origin/main` @ `0db5089` (S57).

## Landscape
6 Criterion benches exist (parser parse, cst parse_cst_vs_ast, interp eval, vm
parse_compile_execute, memory vector + eviction). `garnet_benchmark_no_run.py`
proves they compile. S58 adds the campaign inventory + anti-rot gate.

## Deliverables
- `scripts/garnet_benchmark_campaign.py`: inventory the 6 benches (crate, name,
  measures, run command); verify each bench file + Cargo `[[bench]]` entry
  exists; `--gate` fails on a missing/undeclared bench. `--format md|json`.
- `scripts/test_garnet_benchmark_campaign.py`: 5 unit tests.
- Wire test + `--gate` into ci.yml agent-contracts.
- CHANGELOG + contract S58 block.

## Dogfood
- `garnet_benchmark_campaign.py --format md` → all 6 present + declared; `--gate` 0.

## Honest scope (do not soften)
- Inventories + verifies harnesses EXIST; does NOT run them; reports NO
  measurements (env-specific; recorded by an explicit campaign run). No new lane.

## Gates
- reporter + tests + ladder (zero Rust changed; workspace 0 failed). Ledger:
  `s57 → merged(5)` advanced this branch; `s58` rides with S59.
