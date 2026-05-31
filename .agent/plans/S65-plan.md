# S65 Plan — AI-authorship provenance

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S65 (v0.8.1 band).
Map: reconciled plan §165-166 — MCP/AI attestation.
Branch: `codex/s65-ai-provenance`. Base: `origin/main` @ `6d72ac1` (S64).

## Approach (declared, attestable — not detection)
Make AI-authorship a first-class declaration in the seal predicate (same posture
as @caps). `garnet seal --authored-by <provenance>` records an "authorship" field.

## Deliverables
- `garnet-cli/src/seal.rs`: `statement_json_with_authorship(...authorship: Option)`
  adds an optional `"authorship"` field; `statement_json` delegates (None) →
  default shape unchanged.
- `garnet-cli/src/cmd/seal.rs`: `--authored-by <provenance>` flag.
- `C_Language_Specification/GARNET_AI_PROVENANCE.md` — the model + honest
  "self-declared, not detection" boundary.
- `garnet-cli/tests/ai_provenance.rs` — 2 cross-OS tests (default has no
  authorship; --authored-by records it). Existing seal tests unchanged.

## Dogfood
- `seal hello.garnet` → no authorship field; `seal hello.garnet --authored-by
  ai:claude-opus-4-8` → predicate.authorship == that string.

## Honest scope (do not soften)
- Self-DECLARED fact, NOT AI-detection. Silence = no claim (not implicit human).
  Verifying accuracy is out of scope. No new readiness lane.

## Gates
- Rust tests + ladder (workspace 0 failed). Ledger: `s64 → merged(5)` advanced;
  `s65` rides with S66.
