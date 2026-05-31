# S66 Plan — model/prompt/tool attestation in seal

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S66 (v0.8.1 band).
Map: reconciled plan §165-166 — MCP/AI attestation.
Branch: `codex/s66-attestation`. Base: `origin/main` @ `a8873f1` (S65).

## Approach
Extend S65's flat authorship string with a structured attestation block in the
same seal predicate. Self-declared, deterministic (sorted), additive.

## Deliverables
- `garnet-cli/src/seal.rs`: `statement_json_full(...authorship, attestation:
  &[(String,String)])`; `statement_json_with_authorship` delegates (empty
  attestation) → S65 signature + tests preserved.
- `garnet-cli/src/cmd/seal.rs`: `--attest <k>=<v>` (repeatable) → sorted
  attestation map.
- `C_Language_Specification/GARNET_ATTESTATION.md` — model/prompt/tool block +
  honest "self-declared, not verified" boundary.
- `garnet-cli/tests/seal_attestation_block.rs` — 4 cross-OS tests (sorted block;
  composes with --authored-by; default none; malformed rejected).

## Dogfood
- `seal --attest model=… --attest tool=…` → sorted attestation object; composes
  with --authored-by; default seal unchanged.

## Honest scope (do not soften)
- Self-DECLARED, NOT verified (the @caps posture). Garnet does not introspect the
  model / hash the live prompt / enumerate invoked tools. Absent --attest = no
  block. No new readiness lane.

## Gates
- Rust tests + ladder (workspace 0 failed). Ledger: `s65 → merged(5)` advanced;
  `s66` rides with S67.
