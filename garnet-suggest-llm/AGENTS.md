# AGENTS.md — Garnet LLM Suggestion Contract

## Scope

This crate owns the feature-gated compiler-as-agent LLM advisory tier for S19.
It is intentionally additive to `garnet-check-v0.3::suggest`: the deterministic
rules remain authoritative, and this crate may only add clearly labeled
non-deterministic suggestions.

## Stable Contracts

- The `llm` Cargo feature gates provider-facing code. Default workspace builds
  must not contact providers, require credentials, or change deterministic
  `garnet check --suggest` behavior.
- `LlmClient::complete` and `LlmTransport::send` are the explicit authority
  boundary. They are documented as `@caps(net)` surfaces; future process-backed
  transports must also declare `@caps(proc)` in their Garnet-facing wrapper.
- Prompt construction must run the deterministic rules tier first and include
  those findings as ground truth. LLM output is advisory only and must carry
  `@stability(non-deterministic)`.
- Reproducibility logs are evidence, not determinism claims. They record prompt
  hashes, model identity, temperature, raw response text, emitted suggestions,
  timestamp, and warnings without writing API keys.
- Provider code is request/response compatible with Anthropic Messages, OpenAI
  Chat Completions, and Ollama Generate. Streaming, tools/function calling,
  vision, and provider-specific edge features are out of scope for v0.7.

## Required Checks

Run these before claiming S19 crate readiness:

```sh
cargo fmt --all -- --check
cargo test -p garnet-suggest-llm --features llm
cargo build -p garnet-suggest-llm --features llm --release
python3 scripts/check_determinism_no_llm.py
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
```
