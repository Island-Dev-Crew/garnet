# S90 Windows Proof Plan

Slot: win-codex

Scope: prove the already-merged Mac-authored S90 `@caps` host-authority runtime enforcement seed on Windows. This branch is proof/accounting only.

Writable surface:
- `.agent/plans/S90-windows-proof-plan.md`
- `CHANGELOG.md` under `[Unreleased]`
- `F_Project_Management/WINDOWS_AUDIT_S1_S80.md` S90 row

Out of scope:
- no edits to `garnet-interp-v0.3/src/eval.rs`
- no edits to `garnet-interp-v0.3/src/stdlib_bridge.rs`
- no VM `@caps` enforcement claim

Proof commands:
- `python -B scripts\test_garnet_caps_enforcement_status.py`
- `python -B scripts\garnet_caps_enforcement_status.py --gate --format json`
- `cargo test -p garnet-cli --test caps_enforcement -- --nocapture`
- direct undeclared-capability `garnet run --interp` fixture exits non-zero with `requires @caps(env)`
- `python -B scripts\test_garnet_windows_audit_status.py`
- `python -B scripts\garnet_windows_audit_status.py --gate --format json`
- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo test --workspace --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`
