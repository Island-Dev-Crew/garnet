# S82 Windows proof - seal source-hash determinism

## Goal
Record the Windows lane proof for the already-merged S82 seal determinism fix
without changing seal or manifest logic.

## Scope
- Prove `scripts/garnet_seal_determinism_status.py --gate` passes on Windows.
- Prove an LF `.garnet` source and a CRLF `.garnet` source seal to the same
  `predicate.source_blake3` value on Windows.
- Record the exact hash in `F_Project_Management/WINDOWS_AUDIT_S1_S80.md`.

## Out of scope
- Re-authoring `garnet-cli/src/manifest.rs`.
- Claiming a fresh Mac runtime proof from the Windows machine.
- Updating S81, S84, S85, S89, or S90 proof rows.

## Verification commands
- `python -B scripts\test_garnet_seal_determinism_status.py`
- `python -B scripts\garnet_seal_determinism_status.py --gate --format json`
- Windows LF/CRLF seal proof:
  - `cargo run -q -p garnet-cli --bin garnet -- seal <lf.garnet> --out <lf.json>`
  - `cargo run -q -p garnet-cli --bin garnet -- seal <crlf.garnet> --out <crlf.json>`
  - compare `predicate.source_blake3`
- `python -B scripts\test_garnet_windows_audit_status.py`
- `python -B scripts\garnet_windows_audit_status.py --gate --format json`
- `cargo fmt --all -- --check`
- `git diff --check --cached`
- `cargo test --workspace --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`

## Recorded proof
The Windows LF/CRLF seal proof produced identical `source_blake3`:

`096cb946361fbf2d821452449578fd8f5af3f2a70c3546e763e43d4374d168ad`
