# S85 Windows proof - interpreter deep recursion and parity

## Goal
Record the Windows lane proof for the already-merged S85 interpreter large-stack
fix without changing interpreter, VM, or runtime logic.

## Scope
- Prove `garnet run --interp examples/mvp_function_call_demo.garnet` exits 0 on
  Windows and prints `=> 7105`.
- Prove `scripts/garnet_interp_stack_status.py --gate` passes on Windows.
- Prove `scripts/garnet_vm_interp_parity.py --gate` runs dynamically with the
  Windows `garnet.exe` binary and reports 33/33 parity.
- Record the exact evidence in `F_Project_Management/WINDOWS_AUDIT_S1_S80.md`.

## Out of scope
- Re-authoring `garnet-cli/src/cmd/run.rs`.
- Claiming unbounded recursion safety; S89 owns bounded enforcement.
- Updating S81, S82, S84, S89, or S90 proof rows.

## Verification commands
- `cargo run -q -p garnet-cli --bin garnet -- run --interp .\examples\mvp_function_call_demo.garnet`
- `python -B scripts\test_garnet_interp_stack_status.py`
- `python -B scripts\garnet_interp_stack_status.py --gate --format json`
- `python -B scripts\test_garnet_vm_interp_parity.py`
- `python -B scripts\garnet_vm_interp_parity.py --gate --format json`
- `python -B scripts\test_garnet_windows_audit_status.py`
- `python -B scripts\garnet_windows_audit_status.py --gate --format json`
- `cargo fmt --all -- --check`
- `git diff --check --cached`
- `cargo test --workspace --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`

## Recorded proof
- Direct interpreter run exited 0 and printed `=> 7105`.
- VM/interpreter parity gate reported `binary_available=true`, `parity_ok=33`,
  `corpus_size=33`, and `divergent=[]`.
