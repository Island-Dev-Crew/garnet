# Garnet v0.5 Dogfood Readiness Phase Log

Date: 2026-05-08

This log records whether the language-completion roadmap survives actual use.
It is not a replacement for dogfood-readiness reports; it is the phase ledger
that each report should consume.

| Phase | Dogfood gate | Current result | Evidence |
|---|---|---|---|
| 1 Parser parity | New Mini-Spec syntax parses without breaking MVPs | parser-stage `do...end` support landed and remains covered while Phase 2A builds runtime semantics on top | `cargo test -p garnet-parser --test parse_v1_parser_parity`; `cargo test -p garnet-parser`; `cargo test -p garnet-cli --test conformance_skeleton`; `cargo test -p garnet-cli --test dogfood_readiness_examples`; `cargo test -p garnet-cli --test examples` |
| 2 Managed runtime | Parser-stage features run in managed mode | Phase 2H active: syntactic `do...end` block invocation, `yield`, `next`, per-instance `@dynamic` method tables, static inherent impl fallback, `method_missing`, protocol-typed managed parameter checks, protocol method signature checks, runtime `as Protocol` casts, generic protocol substitution, core built-in typed method signatures, and `@dynamic impl` dispatch tables are active. Ordinary trait coherence and broader generic impl resolution remain deferred. | `cargo test -p garnet-cli --test conformance_skeleton`; `cargo test -p garnet-cli --test conformance_phase_gates`; `cargo test -p garnet-parser --test parse_v1_parser_parity` |
| 3 Actors + Sendable | `agent-orchestrator` uses actor syntax frictionlessly and nonsendable actor payloads are rejected | Phase 3B active: `@nonsendable` actor protocol/handler payload rejection plus synchronous managed `spawn Actor.handler(args)` dispatch. Full async actor-runtime address/mailbox bridge remains pending. | `cargo test -p garnet-check --test extended actor_protocol_rejects_nonsendable_payload_type`; `cargo test -p garnet-cli --test conformance_skeleton actor_sendable_rejects_nonsendable_protocol_payloads`; `cargo test -p garnet-cli --test examples multi_agent_builder_runs_with_managed_actor_bridge`; `cargo test -p garnet-interp c5_actor_handler_dispatches_via_spawn_bridge` |
| 4 Safe mode | unsafe ownership examples are rejected, valid examples pass | not started | `borrow.rs` skeleton |
| 5 Traits/generics | trait/generic examples check and run honestly | not started | parser-only today |
| 6 Memory Core | cycle and kind-aware memory fixtures pass | not started | Mnemos reference stores only |
| 7 Proof/release/empirics | release/proof/benchmark claims are separated and falsifiable | scaffold only | runbooks and research handoffs |
| Security/trust boundary | FS/net/db/converter/release authority surfaces are inventoried and probed | in progress | `GARNET_SECURITY_DOGFOOD_RUBRIC.md`; `cargo audit`; `cargo deny --all-features check`; source trust-boundary scan |

## Current Phase 1 Acceptance

- `cargo test -p garnet-parser --test parse_v1_parser_parity`
- `cargo test -p garnet-cli --test conformance_skeleton`
- `cargo test -p garnet-cli --test conformance_phase_gates`
- `cargo test -p garnet-cli --test dogfood_readiness_examples`
- `cargo test -p garnet-cli --test examples`
- canonical MVP parse/check/run loop
- `cargo audit`
- `cargo deny --all-features check`
- source trust-boundary scan for command execution, DB queries, filesystem/network authority, unsafe/eval/exec, secrets, and release-integrity surfaces
