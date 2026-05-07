# Garnet v0.5 Dogfood Readiness Phase Log

Date: 2026-05-07

This log records whether the language-completion roadmap survives actual use.
It is not a replacement for dogfood-readiness reports; it is the phase ledger
that each report should consume.

| Phase | Dogfood gate | Current result | Evidence |
|---|---|---|---|
| 1 Parser parity | New Mini-Spec syntax parses without breaking MVPs | in progress | `parse_v1_parser_parity`, `conformance_skeleton`, MVP run probe |
| 2 Managed runtime | Parser-stage features run in managed mode | not started | deferred conformance handles |
| 3 Actors + Sendable | `agent-orchestrator` uses actor syntax frictionlessly | not started | current template uses pure role functions |
| 4 Safe mode | unsafe ownership examples are rejected, valid examples pass | not started | `borrow.rs` skeleton |
| 5 Traits/generics | trait/generic examples check and run honestly | not started | parser-only today |
| 6 Memory Core | cycle and kind-aware memory fixtures pass | not started | Mnemos reference stores only |
| 7 Proof/release/empirics | release/proof/benchmark claims are separated and falsifiable | scaffold only | runbooks and research handoffs |

## Current Phase 1 Acceptance

- `cargo test -p garnet-parser --test parse_v1_parser_parity`
- `cargo test -p garnet-cli --test conformance_skeleton`
- `cargo test -p garnet-cli --test conformance_phase_gates`
- `cargo test -p garnet-cli --test dogfood_readiness_examples`
- `cargo test -p garnet-cli --test examples`
- canonical MVP parse/check/run loop

