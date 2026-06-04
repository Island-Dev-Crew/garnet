# win-codex S117 Linux/Tauri Gate Replay Plan

Branch: `agent-win-codex/s117-linux-gate-replay`

Baseline before edits:

- `python scripts/garnet_mit_readiness_status.py` -> `92.1%` active-partial
- `cargo test --workspace --no-fail-fast` -> pass
- `cargo clippy --workspace --all-targets -- -D warnings` -> pass

## Scope

Record a consolidated S117 Linux/Tauri gate replay proof that replays the current committed WSL/WSLg package, runtime, shell-domain, and shell-readiness proof gates from one repo-owned command.

## Honest Boundaries

- WSL and WSLg rows remain execution/portability evidence only.
- This slice does not claim clean/non-WSL Linux desktop proof.
- This slice does not claim Linux seccomp, OS-sandbox enforcement, signed artifacts, production readiness, or v1.0 readiness.
- The existing individual gates remain the authority; the new proof verifies they all still replay together.

## Implementation

1. Add a focused failing test module for the replay verifier.
2. Add `scripts/smoke_garnet_studio_linux_gate_replay.py`.
3. Wire committed replay evidence into the Windows/Linux Studio status reporter.
4. Wire a committed replay lane into the MIT readiness reporter.
5. Update section-scoped docs and PR evidence copy.

## Validation

- `python scripts/test_smoke_garnet_studio_linux_gate_replay.py`
- `python scripts/smoke_garnet_studio_linux_gate_replay.py --record --format json`
- `python scripts/smoke_garnet_studio_linux_gate_replay.py --gate --format json`
- `python scripts/test_garnet_windows_linux_studio_status.py`
- `python scripts/test_garnet_mit_readiness_status.py`
- `python scripts/garnet_mit_readiness_status.py --check-no-regression`
- `cargo fmt --all -- --check`
- `cargo test --workspace --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
