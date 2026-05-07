# Garnet v0.4.2 Conformance Suite Skeleton

Date: 2026-05-06
Source matrix: `GARNET_v0_4_2_Conformance_Matrix.md`
Executable skeleton: `garnet-cli/tests/conformance_skeleton.rs`

## Purpose

The Mini-Spec matrix is the descriptive truth about what Garnet implements
today. The conformance suite skeleton turns that matrix into test handles so
language-completeness work can move from prose to executable evidence.

## Current Active Tests

| Matrix area | Test | Expected status |
|---|---|---|
| §6 control flow + interpreter | `implemented_control_flow_and_interpreter_smoke_runs` | active pass |
| §4 memory declarations | `implemented_memory_declaration_parses` | active pass |
| §8 CapCaps authority checking | `implemented_capcaps_rejects_missing_fs_authority` | active pass |
| §16 deterministic manifest tooling | `implemented_reproducible_manifest_smoke_builds` | active pass |

## Deferred or Partial Test Handles

These tests are intentionally `#[ignore]` in v0.4.2. They should be activated
when the corresponding Mini-Spec row becomes implemented.

| Matrix row | Test handle |
|---|---|
| §4.5 ARC + Bacon-Rajan cycle detection | `deferred_arc_cycle_detection` |
| §5.4 blocks/yield | `deferred_blocks_and_yield` |
| §8.5 NLL/lifetime inference | `deferred_nll_lifetime_inference` |
| §8.6 borrow rules B1-B5 | `partial_borrow_rule_suite` |
| §11.5 trait coherence | `deferred_trait_coherence` |
| §11.6 monomorphization | `parsed_only_monomorphization` |
| §11.7 dynamic dispatch | `deferred_dynamic_dispatch` |
| §11.8 structural protocols | `deferred_structural_protocols` |

## Activation Rule

When an ignored conformance test is made active:

1. Implement the runtime/checker/parser behavior.
2. Replace the placeholder body with an assertion that would fail on the old
   behavior.
3. Update `GARNET_v0_4_2_Conformance_Matrix.md` in the same commit.
4. Run `cargo test -p garnet-cli --test conformance_skeleton`.

