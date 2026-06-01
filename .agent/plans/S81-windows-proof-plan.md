# S81 Windows Proof - uppercase .GARNET discovery

Contract: `F_Project_Management/WINDOWS_AUDIT_S1_S80.md` -> S81 Windows-proof row.

## Goal

Record the Windows proof for the already-merged Mac-authored S81 fix. The proof
must show that `garnet verify <dir>` discovers an uppercase `BAD.GARNET` file on
Windows, exits 1, and still reports the clean lowercase `main.garnet` as clean.

## Scope

- Update only proof/accounting artifacts for S81.
- Do not re-author the S81 collector fix.
- Keep S82 and later rows unchanged unless their own proof PR handles them.

## Dogfood

```powershell
python -B scripts\test_garnet_garnet_ext_discovery_status.py
python -B scripts\garnet_garnet_ext_discovery_status.py --gate --format json
cargo run -q -p garnet-cli --bin garnet -- verify <temp-dir-with-main.garnet-and-BAD.GARNET>
python -B scripts\test_garnet_windows_audit_status.py
python -B scripts\garnet_windows_audit_status.py --gate --format json
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
```

## Honest Scope

This proof records the Windows result for S81. It does not change collector
logic and does not claim S82/S84/S85/S89/S90 proof.
