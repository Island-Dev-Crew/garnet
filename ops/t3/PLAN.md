# T3 checked acceptance and T-L shadow policy

Approved direction: Jon's 2026-09-18 task instruction, following the local
Garnet direction report. Baseline upstream main: f6d448173b349a649212170b95fbc2987d73b2f2.

## Active review unit

1. Refuse a seal before output when edition-aware `garnet check` would fail.
2. Make the new in-toto subject identify normalized source, so changing only
   `@caps` changes artifact identity. Version the new contract, preserve the
   distinction between content integrity and external signatures.
3. Independently recompute source/build/capability bindings when verifying a
   new seal. Do not accept unknown/legacy generic seal formats as current proof.
4. Make an explicit `--caps-baseline` fail on widening, missing/malformed/empty
   evidence or an incomplete directory walk. Apply to both verify routes;
   preserve existing signed deterministic-manifest verification.
5. Introduce an opt-in shadow policy with immutable Git input coordinates,
   conservative unknown handling and `merge_authorized: false` unconditionally.
   No external labels, PR actions, required context, credentials or signing.

Negative controls: capability-only mutation; malformed seal; invalid source;
invalid baseline; directory omissions; duplicate fields; changed source;
unsupported schemas; changed symlink/executable/submodule; unknown policy path.
Positive controls: same source/caps; LF/CRLF equivalent source; canonical seal
round-trip; existing deterministic manifest/signature tests; historical pinned
Minimum Shelf package (explicit legacy compatibility only).

## Separate build and approval rounds

Playground Phase 1 is on `codex/playground-phase1`: readable/raw results, an
illustrative declared-capability card, annotated declaration table, safe
fragment links, refusal presets, editor comfort, digest badge and compact
embedding. It must pass real browser journeys, mobile/iframe and offline
checks, then bind W-PLAY proof to final content. This work does not change
Rust or WASM semantics.

Build in parallel; stage exactly one review record round at a time. The second
round starts from then-current upstream main with a fresh record and required
recaptures. Jon's #591 agent-merge exception is not inherited.

## Subsequent T3 semantics and T4

The original T3 includes additional runtime step-budget, network/socket and
converter work. They remain open, not silently promoted by this acceptance
unit. Step counting and nested exhaustion need a separate red/green contract;
per-host network policy is distinct from socket bridging; converter parse
success is distinct from semantic preservation. T4 reconciliation remains with
Jon/Claude as requested. Studios consolidation is deferred while existing
native Linux Studio launch obligations remain in force.

## Gates and delivery boundary

- `cargo test -p garnet-cli --test checked_seal_acceptance`
- `cargo test -p garnet-cli`
- `cargo fmt --all -- --check`
- `cargo test --workspace --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `python3 -I scripts/check-agent-contracts.py`
- `python3 -I scripts/test_check_agent_contracts.py`
- `python3 -I scripts/test_garnet_shadow_lanes.py` plus real binary integration
- wasm/playground readiness and exact final browser proof for the UI round

Cross-family review at an exact clean head, approval, CI and owner merge remain
separate from implementation. Same-family development QA is never substituted
for that review. Local logs live in the sibling dogfood packet; their absence
from source is not a declaration that they were committed evidence.
