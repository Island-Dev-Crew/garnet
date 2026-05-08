# Garnet v0.5 Phase Ownership Register

Date: 2026-05-07

| Phase | Owner surface | Status | Main risk | Verification command | Handoff target |
|---|---|---|---|---|---|
| 1 Parser parity | `garnet-parser-v0.3`, `garnet-cli/tests/conformance_skeleton.rs` | started | parser accepts syntax the runtime cannot execute | `cargo test -p garnet-parser --test parse_v1_parser_parity` | conformance matrix parser-stage rows |
| 2 Managed runtime | `garnet-interp-v0.3` | not started | block/dynamic/protocol semantics become inconsistent | `cargo test -p garnet-cli --test conformance_skeleton deferred_blocks_and_yield -- --ignored` | runtime semantics addendum |
| 3 Actors + Sendable | `garnet-actor-runtime`, checker, interpreter bridge, CLI templates | Phase 3D active | full async actor-runtime OS-thread bridge remains detached from managed `Value` | `cargo test -p garnet-cli --test cli_smoke new_agent_orchestrator_template_runs_and_tests` plus actor conformance gates | async runtime bridge handoff |
| 4 Safe-mode ownership | `garnet-check-v0.3/src/borrow.rs`, `garnet-cli/tests/conformance_skeleton.rs` | Phase 4B active | full NLL/place-granular B1-B5 and type-resolved impl dispatch still pending | `cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite`; `cargo test -p garnet-check --test borrow` | full borrow/NLL checker handoff |
| 5 Traits/generics | `garnet-check-v0.3`, future lowering | not started | zero-cost claims outrun backend evidence | `cargo test -p garnet-cli --test conformance_skeleton parsed_only_monomorphization -- --ignored` | trait/generic implementation memo |
| 6 Memory Core | `garnet-memory-v0.3` | not started | ARC/cycle work is algorithmically subtle | `cargo test -p garnet-cli --test conformance_skeleton deferred_arc_cycle_detection -- --ignored` | Memory Core implementation addendum |
| 7 Proof/release/empirics | docs, release workflows, future proof repos | scaffold only | formal and empirical work cannot be honestly completed in one agent run | dogfood-readiness Part 1 after every phase | MIT/research handoff bundle |
