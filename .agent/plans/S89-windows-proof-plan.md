# S89 Windows Proof Plan

Slot: win-codex
Branch: codex/s89-windows-proof
Slice: S89 Windows proof/accounting only

## Scope

S89 already landed on `origin/main` as a Mac-authored runtime-enforcement slice.
This Windows lane does not re-author `garnet-interp-v0.3`, the CLI, or the
kernel. It proves the landed `@max_depth(N)` trap on Windows and records the
result in the audit ledger.

## Proof Commands

- `python -B scripts\test_garnet_bounded_enforcement_status.py`
- `python -B scripts\garnet_bounded_enforcement_status.py --gate --format json`
- `cargo test -p garnet-cli --test bounded_enforcement -- --nocapture`
- Direct Windows trap fixture:
  `cargo run -q -p garnet-cli --bin garnet -- run --interp <over_ceiling.garnet>`

## Expected Result

- The reporter tests pass 5/5.
- The status gate exits 0 with `ok=true`.
- The Rust integration test passes 4/4.
- The direct fixture exits non-zero and stderr contains
  `@max_depth(4) exceeded for `deep``.

## Honest Scope

This proves the one S89 enforced ceiling on Windows: `@max_depth(N)` recursion
in the interpreter. It does not prove Wasmtime fuel, memory, time, mailbox,
the VM backend, or undeclared-capability runtime traps.
