# S59 Plan — fuzz campaign

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S59.
Map: reconciled plan §157 — fuzz campaign.
Branch: `codex/s59-fuzz`. Base: `origin/main` @ `86d4cd2` (S58).

## Landscape
One libFuzzer target: `garnet-parser-v0.3/fuzz/fuzz_targets/parse_input.rs`
(fuzzes parse_source_with_budget), run ≥1h nightly by `fuzz-nightly.yml`. 8 S20
corpus seeds. cargo-fuzz ABSENT locally.

## Deliverables
- 5 new corpus seeds (hello, typed_errors, state_machine, documented_math,
  safe_io_layer) → cover the S42–S57 grammar. Just input files (no cargo-fuzz).
- `scripts/garnet_fuzz_campaign.py`: inventory target + crate + nightly protocol
  + seed count; `--gate` fails if target file / Cargo [[bin]] / workflow wiring /
  seeds missing. `--format md|json`.
- `scripts/test_garnet_fuzz_campaign.py`: 5 unit tests.
- Wire test + `--gate` into ci.yml agent-contracts.
- CHANGELOG + contract S59 block.

## Dogfood
- `garnet_fuzz_campaign.py --format md` → harness wired + 13 seeds; `--gate` 0.

## Honest scope (do not soften)
- Verifies the harness EXISTS + is wired; does NOT run the fuzzer; NO bug-found
  (or bug-free) claim. cargo-fuzz absent → structural verification only. No lane.

## Gates
- seeds + reporter + tests + ladder (zero Rust changed; workspace 0 failed).
  Ledger: `s58 → merged(5)` advanced this branch; `s59` rides with S60.

## NOTE — S60 next is the v0.8.0 TAG → ESCALATE to Jon (release-truth decision).
