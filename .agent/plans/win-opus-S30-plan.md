# win-opus — S30 Plan: Functional-core composition capstone

**Slot:** win-opus · **Slice:** S30 (Jon-directed; capstone of the S26-S28 functional-core arc) ·
**Branch:** `agent-win-opus/s30-functional-capstone` (off `origin/main` `d301bed`, S29 merged)
**Baseline:** readiness 85.9% / 41 lanes, `--check-no-regression` exit 0.

## Goal

With the full functional `core::` surface now interpreter-dispatched (S26 result, S27
option, S28 iter), prove they **compose** into railway-oriented pipelines from Garnet
source — and tell the story. Additive composition + story slice (the S25 capstone shape).

## Scope (additive only)

- `examples/novel_07_functional_core_pipeline.garnet` — deterministic, cross-platform
  `@caps()` program: `core::iter` (collect/map/fold/zip) → `core::result` (ok/map/and_then/
  unwrap_or) → `core::option` (some/map/unwrap_or), final `80`. Joins the novel-composition
  harness (now 7/7).
- `garnet-interp-v0.3/tests/functional_core_composition.rs` — integration test exercising
  BOTH tracks: iter fold (20); Result Ok-railway (40) AND Err-railway recovered via
  `or_else` (0); Option Some (80) AND None default (7) → `[20,40,0,80,7]`.
- Story: `C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md` novel_07 section.
- `functional_core_composition` readiness lane; baseline surgically extended.
- **No parser/CST/owned-crate source change** — only a new example + test + additive docs.

## Test proportion (~60/40)
"Code" = the novel_07 program. "Test" = the integration test (both success + failure
tracks, asserted `[20,40,0,80,7]`) + the novel-harness exact-output case (`novel_07 final: 80`).

## Novel discovery
The full functional core (result/option/iter) is now runnable and composable from managed
Garnet — railway-oriented error/optional handling + iterator pipelines, pure compute, no
host effects, byte-stable output. Complements the S25 host-effect capstone.

## Honest scope
- Pure managed-mode compute (no host effects); the host-effect composition is S25.

## Dogfood block
```bash
cargo build -p garnet-cli
cargo test -p garnet-interp --test functional_core_composition --no-fail-fast
garnet run examples/novel_07_functional_core_pipeline.garnet     # novel_07 final: 80
python3 scripts/smoke_garnet_novel_compositions.py               # 7/7
python3 -m unittest scripts.test_garnet_novel_compositions
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```
