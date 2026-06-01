# S88 Release Tooling Status Plan

Slot: win-codex

Scope: author the Windows-required reporter for local external release tooling. This closes the audit gap only as an honest status lane: it never claims signing, SBOM, or fuel/epoch proof when the corresponding tool is absent.

Writable surface:
- `.agent/plans/S88-release-tooling-status-plan.md`
- `scripts/garnet_release_tooling_status.py`
- `scripts/test_garnet_release_tooling_status.py`
- `CHANGELOG.md` under `[Unreleased]`
- `F_Project_Management/WINDOWS_AUDIT_S1_S80.md` S88/accounting rows

Out of scope:
- no release tag
- no release credential setup
- no fake signed artifact claim
- no claim that Wasmtime fuel/epoch enforcement is integrated into Garnet runtime

Proof commands:
- `python -B scripts\test_garnet_release_tooling_status.py`
- `python -B scripts\garnet_release_tooling_status.py --gate --format json`
- local tool discovery/provision attempt for `cosign`, `syft`, `cyclonedx`, and `wasmtime`
- `python -B scripts\test_garnet_windows_audit_status.py`
- `python -B scripts\garnet_windows_audit_status.py --gate --format json`
- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo test --workspace --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`
