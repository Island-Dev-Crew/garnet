# win-opus — S29 Plan: `@stability` error-level enforcement (opt-in)

**Slot:** win-opus · **Slice:** S29 (Jon-directed; the named v0.8 "@stability error-level" deferral) ·
**Branch:** `agent-win-opus/s29-stability-errors` (off `origin/main` `980c65b`, S28 merged)
**Baseline:** readiness 85.5% / 40 lanes, `--check-no-regression` exit 0.

## What's left

S17 made `@stability` enforcement warning-level; the Layer Policy §4 + S17 lane name
"error-level enforcement is v0.8". This slice ships that as an **opt-in**, fully within
garnet-check's stability surface (win-opus-owned), so the default stays warning-level and
existing programs/CI stay green.

## Scope (garnet-check stability surface only)

- `lib.rs`: add a FATAL `CheckError::StabilityError(String)` variant and list it in
  `CheckReport::ok()` (the explicit fatal allowlist). `garnet-cli` already exits on
  `report.ok()`, so this works end-to-end with **no garnet-cli change**.
- `stability.rs`: `stability_error_mode()` reads `GARNET_STABILITY_ERRORS` (`1`/`true`);
  `advise(..., error_mode)` emits `StabilityError` ("stability error: …") for
  experimental/deprecated under error mode, `StabilityAdvice` otherwise. Frozen stays an
  info advisory; `stable` silent. Default-mode messages are byte-for-byte the v0.7 wording.
- `stability_error_enforcement` readiness lane; baseline surgically extended.

## Test proportion (~60/40)
"Code" = the variant + ok() entry + env read + advise/advisory plumbing. "Test" = stability
unit tests (experimental/deprecated→fatal error, frozen→info, default→warning, all via the
`advise(error_mode)` core — no env races) + a lib.rs `ok()` classification test. End-to-end
CLI behavior verified: `GARNET_STABILITY_ERRORS=1 garnet check <experimental program>` exits 1.

## Honest scope
- Error mode is process-global via env var; per-source `@uses(experimental)` opt-out still
  needs the parser annotation variants (S17 win-opus → mac-opus handoff), unchanged.
- Default stays warning-level (opt-in), so CI and existing programs are unaffected.

## Dogfood block
```bash
cargo build -p garnet-cli
cargo test -p garnet-check stability --no-fail-fast
garnet check examples/novel_04_dispatched_stdlib_pipeline.garnet                 # warns, exit 0
GARNET_STABILITY_ERRORS=1 garnet check examples/novel_04_dispatched_stdlib_pipeline.garnet  # "stability error:", exit 1
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/run_agentic_dogfood_matrix.py    # unchanged (default warning-level)
```
