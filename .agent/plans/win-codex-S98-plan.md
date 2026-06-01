# S98 Capability Manifest Standard Plan

Slot: `win-codex`
Branch: `agent-win-codex/s98-capability-manifest-standard`
Base: `origin/main` at S97 merge `611366b`

## Goal

Advance the capability-manifest standard seed with a deterministic, reference-implementable schema profile and proof gate, while preserving calibrated honesty: this is intent plus a reference implementation seed, not an adopted standard.

## Baseline

- `python scripts/garnet_mit_readiness_status.py --format json` -> exit 0, readiness 90.0%.
- `cargo test --workspace --no-fail-fast` -> exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` -> exit 0.
- Logs: `C:\Users\ISLAND~1\AppData\Local\Temp\garnet-s98-baseline-20260601-070324`

## Implementation

1. Add failing tests for a language-neutral standard profile around the existing Garnet capability manifest.
2. Implement a deterministic standard JSON profile in `garnet-cli/src/cap_manifest.rs` without breaking the existing S36 manifest envelope.
3. Add CLI/test-vector coverage proving `garnet caps --standard-profile <path>` emits stable, sorted JSON that maps functions to capability grants.
4. Add a static status reporter and unit tests proving the docs, RFC, reference implementation, and vectors are present.
5. Wire a committed-truth readiness lane for S98 and update the changelog/plan/ledger with honest scope.
6. Build a desktop evidence bundle, open PR, run the dogfood PR body checker, wait for CI, merge through Chrome, then stop the S91-S98 goal lane.

## Honest Scope

- Reference seed only: no OWASP/LF adoption, no multi-language ecosystem, no public registry requirement.
- The standard profile reflects declared capability surfaces; it does not prove absence of undeclared authority.
- The VM capability-enforcement gap remains named-deferred.
