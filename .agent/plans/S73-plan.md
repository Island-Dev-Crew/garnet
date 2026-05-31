# S73 — VM / interpreter parity campaign

## Goal
Garnet has two execution backends (interpreter `--interp`, bytecode VM `--vm`).
Make backend parity a gated, reproducible differential campaign on the validation
runway — they must not silently diverge.

## What ships
- `scripts/garnet_vm_interp_parity.py` (+ `--gate`) — runs every
  `examples/*.garnet` through both backends; parity = same stdout + same exit
  code. Result today: **33/33 parity, 0 divergences**.
- `scripts/test_garnet_vm_interp_parity.py` — 7 unit tests (pure predicate +
  static gate + live full-parity check).
- CI: canonical-examples runs the binary-backed campaign; agent-contracts runs
  the static gate (`--no-run`) + tests.
- Doc `F_Project_Management/GARNET_VM_INTERP_PARITY.md`; CHANGELOG; contract S73
  block; this plan; ledger `s72 → merged`.

## Design notes (findings)
- Compare **stdout + exit code only**. stderr is NOT compared because (a) the VM
  wraps runtime errors with a cosmetic `vm error:` prefix the interpreter omits
  (verified on `mvp_11_signed_hotreload_mismatch`: both exit 1, same BLAKE3
  exception, different wrapper); (b) the episodic cache emits run-to-run stderr
  notes. **stdout is deterministic across cache state** → the sound parity signal.
- A naive stdout+stderr comparison run interp-then-vm in the same cwd produced 3
  false "divergences" from cache-strategy notes; neutralized by comparing stdout.

## Verification
- `python3 scripts/test_garnet_vm_interp_parity.py` → 7 OK.
- `garnet_vm_interp_parity.py --gate` → rc 0 (33/33). `--gate --no-run` → rc 0.
- Ladder: fmt/diff clean; `cargo test --workspace` 0 failed (no Rust changed).

## Honest scope (do not soften)
Corpus-based parity over the shipped examples, NOT a proof of total backend
equivalence. Divergences are reported, not hidden. The stderr wrapper-prefix is a
documented cosmetic difference, not masked.
