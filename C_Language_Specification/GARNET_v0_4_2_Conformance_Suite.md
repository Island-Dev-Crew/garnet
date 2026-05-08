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
| §11.6/§11.8 parser parity | v0.5 Phase 1 | Parser-Parity Gate | `parser_parity_top_level_protocol_and_dyn_trait_parse` | active pass |
| §5.4/§11.7 parser parity | v0.5 Phase 1 | Parser-Parity Gate | `parser_parity_yield_next_dynamic_and_nonsendable_parse` | active pass |
| §5.4.1 parser parity | v0.5 Phase 1 | Parser-Parity Gate | `parses_do_end_block_argument` | active pass |
| §5.4 block/yield/next runtime | v0.5 Phase 2A | Runtime Gate | `deferred_blocks_and_yield` | active pass |
| §5.4 block-vs-closure boundary | v0.5 Phase 2A | Runtime Gate | `explicit_closure_argument_does_not_become_implicit_block` | active pass |

## Deferred or Partial Test Handles

These tests are intentionally `#[ignore]` in v0.4.2. They should be activated
when the corresponding Mini-Spec row becomes implemented.

| Matrix row | Phase | Readiness gate | Test handle |
|---|---|---|---|
| §4.5 ARC + Bacon-Rajan cycle detection | v0.5 Phase 6 | Memory Core Gate | `deferred_arc_cycle_detection` |
| §8.5 NLL/lifetime inference | v0.5 Phase 4 | Safe-Mode Gate | `deferred_nll_lifetime_inference` |
| §8.6 borrow rules B1-B5 | v0.5 Phase 4 | Safe-Mode Gate | `partial_borrow_rule_suite` |
| §11.5 trait coherence | v0.5 Phase 5 | Type-System Gate | `deferred_trait_coherence` |
| §11.6 monomorphization | v0.5 Phase 5 | Type-System Gate | `parsed_only_monomorphization` |
| §11.7 dynamic dispatch runtime semantics | v0.5 Phase 2 | Runtime Gate | `deferred_dynamic_dispatch` |
| §11.8 structural protocol semantics | v0.5 Phase 2 | Runtime/Checker Gate | `deferred_structural_protocols` |

## Activation Rule

When an ignored conformance test is made active:

1. Implement the runtime/checker/parser behavior.
2. Replace the placeholder body with an assertion that would fail on the old
   behavior.
3. Update `GARNET_v0_4_2_Conformance_Matrix.md` in the same commit.
4. Run `cargo test -p garnet-cli --test conformance_skeleton`.
