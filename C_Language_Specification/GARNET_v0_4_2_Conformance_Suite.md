# Garnet v0.4.2 Conformance Suite Skeleton

Date: 2026-05-06
Source matrix: `GARNET_v0_4_2_Conformance_Matrix.md`
Executable skeleton: `garnet-cli/tests/conformance_skeleton.rs`

## Purpose

The Mini-Spec matrix is the descriptive truth about what Garnet implements
today. The conformance suite skeleton turns that matrix into test handles so
language-completeness work can move from prose to executable evidence.

## Current Active Tests

| Matrix area | Phase | Readiness gate | Test | Expected status |
|---|---|---|---|---|
| §6 control flow + interpreter | v0.4.2 baseline | Conformance Gate | `implemented_control_flow_and_interpreter_smoke_runs` | active pass |
| §4 memory declarations | v0.4.2 baseline | Conformance Gate | `implemented_memory_declaration_parses` | active pass |
| §8 CapCaps authority checking | v0.4.2 baseline | Security Gate | `implemented_capcaps_rejects_missing_fs_authority` | active pass |
| §16 deterministic manifest tooling | v0.4.2 baseline | Release Gate | `implemented_reproducible_manifest_smoke_builds` | active pass |
| §9.4 actor Sendable boundary | v0.5 Phase 3A | Security Gate | `actor_sendable_rejects_nonsendable_protocol_payloads` | active pass |
| §9.1 actor source-to-runtime bridge | v0.5 Phase 3B | Runtime Gate | `multi_agent_builder_runs_with_managed_actor_bridge`; `c5_actor_handler_dispatches_via_spawn_bridge` | active pass |
| §9.1/§9.2 managed actor address + bounded mailbox | v0.5 Phase 3C | Runtime Gate | `parses_spawn_keyword_as_member_method_name`; `c5_spawn_actor_returns_address_with_persistent_state`; `c5_actor_address_enforces_bounded_mailbox`; `c5_actor_address_tell_reports_full_mailbox`; `c5_actor_spawn_rejects_extra_capacity_args` | active pass |
| §16.2 generated actor project | v0.5 Phase 3D | Tooling/Runtime Gate | `new_agent_orchestrator_template_runs_and_tests` | active pass |
| §11.6/§11.8 parser parity | v0.5 Phase 1 | Parser-Parity Gate | `parser_parity_top_level_protocol_and_dyn_trait_parse` | active pass |
| §5.4/§11.7 parser parity | v0.5 Phase 1 | Parser-Parity Gate | `parser_parity_yield_next_dynamic_and_nonsendable_parse` | active pass |
| §5.4.1 parser parity | v0.5 Phase 1 | Parser-Parity Gate | `parses_do_end_block_argument` | active pass |
| §5.4 block/yield/next runtime | v0.5 Phase 2A | Runtime Gate | `deferred_blocks_and_yield` | active pass |
| §5.4 block-vs-closure boundary | v0.5 Phase 2A | Runtime Gate | `explicit_closure_argument_does_not_become_implicit_block` | active pass |
| §11.7 dynamic dispatch runtime semantics | v0.5 Phase 2B | Runtime Gate | `deferred_dynamic_dispatch` | active pass |
| §11.7 static impl fallback + method_missing | v0.5 Phase 2D | Runtime Gate | `static_impl_dispatch_and_method_missing` | active pass |
| §11.7 `@dynamic impl` dispatch tables | v0.5 Phase 2H | Runtime Gate | `dynamic_impl_dispatch_tables` | active pass |
| §11.8 structural protocol semantics | v0.5 Phase 2C/2E/2F/2G/2H | Runtime/Checker Gate | `deferred_structural_protocols`; `parses_protocol_cast_expression`; `dynamic_impl_dispatch_tables` | active pass, including arity, parameter-type, return-type, mode, `as Protocol` cast, generic protocol substitution, typed built-in method signature negatives, and `@dynamic impl` protocol satisfaction |
| §8.6 partial borrow rules | v0.5 Phase 4C | Safe-Mode Gate | `partial_borrow_rule_suite` | active pass for B1/B2 direct mut-alias rejection, method receiver aliasing, B4 direct function use-after-move, unambiguous `own self` method receiver use-after-move, and simple typed receiver disambiguation for same-named impl methods |

## Deferred or Partial Test Handles

These tests are intentionally `#[ignore]` in v0.4.2. They should be activated
when the corresponding Mini-Spec row becomes implemented.

| Matrix row | Phase | Readiness gate | Test handle |
|---|---|---|---|
| §4.5 ARC + Bacon-Rajan cycle detection | v0.5 Phase 6 | Memory Core Gate | `deferred_arc_cycle_detection` |
| §8.5 NLL/lifetime inference | v0.5 Phase 4 | Safe-Mode Gate | `deferred_nll_lifetime_inference` |
| §8.6 full place-granular borrow rules B1-B5 | v0.5 Phase 4 | Safe-Mode Gate | `deferred_full_borrow_rule_suite` |
| §11.5 trait coherence | v0.5 Phase 5 | Type-System Gate | `deferred_trait_coherence` |
| §11.6 monomorphization | v0.5 Phase 5 | Type-System Gate | `parsed_only_monomorphization` |

## Activation Rule

When an ignored conformance test is made active:

1. Implement the runtime/checker/parser behavior.
2. Replace the placeholder body with an assertion that would fail on the old
   behavior.
3. Update `GARNET_v0_4_2_Conformance_Matrix.md` in the same commit.
4. Run `cargo test -p garnet-cli --test conformance_skeleton`.
