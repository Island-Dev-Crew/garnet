# Garnet v0.5 Language Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move Garnet from a research-grade prototype toward the original dual-mode, agent-native language/toolchain ambition with executable conformance, dogfood, and release gates.

**Architecture:** Work in vertical language slices. Every feature must progress through parser/AST, checker/type semantics, interpreter/runtime behavior, conformance tests, dogfood examples, docs, and CI before the public status changes.

**Tech Stack:** Rust workspace, hand-rolled Garnet parser/interpreter/checker, Cargo integration tests, GitHub Actions, Markdown readiness ledgers.

---

## Phase 1: Parser-Parity Baseline

**Intent:** Make Mini-Spec v1.0 syntax visible in the AST before pretending the runtime can execute it.

**Status:** complete for parser parity; Phase 2A now builds on it.

**Implemented in current branch:**

- `protocol` top-level declarations parse as `Item::Protocol`.
- `dyn Trait` parses as `TypeExpr::Dyn`.
- `yield` and `next` parse as staged statements.
- `@dynamic` and `@nonsendable` annotations are preserved on structs; `@dynamic` is preserved on impl blocks.
- Active test handles:
  - `parser_parity_top_level_protocol_and_dyn_trait_parse`
  - `parser_parity_yield_next_dynamic_and_nonsendable_parse`

**Files:**

- Modify: `garnet-parser-v0.3/src/token.rs`
- Modify: `garnet-parser-v0.3/src/ast.rs`
- Modify: `garnet-parser-v0.3/src/grammar/mod.rs`
- Modify: `garnet-parser-v0.3/src/grammar/user_types.rs`
- Modify: `garnet-parser-v0.3/src/grammar/types.rs`
- Modify: `garnet-parser-v0.3/src/grammar/stmts.rs`
- Test: `garnet-parser-v0.3/tests/parse_v1_parser_parity.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [x] **Step 1: Add active parser-parity tests**

Run:

```sh
cargo test -p garnet-parser --test parse_v1_parser_parity
cargo test -p garnet-cli --test conformance_skeleton
```

Expected: parser-parity tests pass; deferred runtime/type-system handles remain ignored.

- [x] **Step 2: Add parse support for `do ... end` block arguments**

Add parser tests before implementation:

```rust
#[test]
fn parses_do_end_block_argument() {
    parse_ok("def main() { each([1, 2]) do |x| x + 1 end }");
}
```

Expected first run before implementation: parser rejects `do`.

- [x] **Step 3: Update matrix rows**

Rows moved after Phase 1:

- `§5.4 Blocks + yield`: from deferred to partial/parser-stage, then Phase 2A
  managed-runtime evidence for block invocation, `yield`, and `next`.
- `§11.6 dyn Trait syntax`: parsed-only.
- `§11.7 @dynamic method dispatch`: parsed-only metadata.
- `§11.8 Structural protocols`: parsed-only top-level declarations.

## Phase 2: Managed Runtime Semantics

**Intent:** Turn parser-stage syntax into managed-mode behavior.

**Files:**

- Modify: `garnet-interp-v0.3/src/eval.rs`
- Modify: `garnet-interp-v0.3/src/stmt.rs`
- Modify: `garnet-interp-v0.3/src/value.rs`
- Modify: `garnet-interp-v0.3/src/env.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`
- Test: `examples/mvp_06_multi_agent.garnet`

- [x] **Step 1: Implement block/yield/next runtime semantics**

Acceptance:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_blocks_and_yield
```

Expected after implementation: pass without `#[ignore]`, with the
`explicit_closure_argument_does_not_become_implicit_block` regression proving
ordinary closure arguments are not silently consumed as implicit blocks.

- [x] **Step 2: Implement dynamic method table for managed mode**

Acceptance:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_dynamic_dispatch
```

Expected after implementation: pass without `#[ignore]` for per-instance `@dynamic` method tables. Static `impl` fallback and `method_missing` remain follow-up work.

- [x] **Step 2D: Implement static impl fallback and method_missing**

Acceptance:

```sh
cargo test -p garnet-cli --test conformance_skeleton static_impl_dispatch_and_method_missing
```

Expected after implementation: pass with static inherent impl methods resolving after per-instance dynamic methods and `method_missing` resolving unresolved calls.

- [x] **Step 3: Implement structural protocol compatibility checks**

Acceptance:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_structural_protocols
```

Expected after implementation: pass without `#[ignore]` for protocol-typed managed parameter checks, including static inherent impl-backed method presence.

- [x] **Step 3E: Tighten structural protocol signature checks**

Acceptance:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_structural_protocols
```

Expected after implementation: `deferred_structural_protocols` rejects arity and required return-type mismatches in addition to missing methods.

- [x] **Step 3F: Implement runtime `as Protocol` casts**

Acceptance:

```sh
cargo test -p garnet-parser --test parse_v1_parser_parity parses_protocol_cast_expression
cargo test -p garnet-cli --test conformance_skeleton deferred_structural_protocols
```

Expected after implementation: `value as Protocol` parses as `Expr::Cast`, structurally compatible values pass through unchanged, and incompatible values fail at runtime with the structural protocol diagnostic. Generic protocol substitution and built-in typed signatures remain follow-up work.

- [x] **Step 3G: Substitute generic protocol types and add typed built-in signatures**

Acceptance:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_structural_protocols
```

Observed before implementation: `BoxLike<String>` still compared required method returns against the unresolved type parameter `T`, so a `TextBox.value() -> String` method was rejected. Built-in `String` methods also satisfied protocols only when signatures had no parameter or return-type requirements.

Expected after implementation: `Protocol<T>` annotations and casts instantiate required signatures before structural checks; incompatible concrete substitutions still fail; core built-in String/Array/Map/number method signatures can satisfy typed protocols without accepting incompatible return types.

- [x] **Step 3H: Register `@dynamic impl` dispatch tables**

Acceptance:

```sh
cargo test -p garnet-cli --test conformance_skeleton dynamic_impl_dispatch_tables
```

Observed before implementation: `@dynamic impl TraitWidget for Renderable` was preserved in the AST but did not participate in protocol satisfaction or method dispatch.

Expected after implementation: `@dynamic impl Type for Protocol` methods are registered in managed mode, satisfy protocol-typed parameters, appear in dynamic receiver introspection, and dispatch before static inherent impl fallback while per-instance dynamic methods still override them.

## Phase 3: Actor Runtime Bridge And Sendable

**Intent:** Make agent-native claims executable from Garnet code, not only Rust actor-runtime tests.

**Files:**

- Modify: `garnet-parser-v0.3/src/grammar/actors.rs`
- Modify: `garnet-check-v0.3/src/lib.rs`
- Modify: `garnet-actor-runtime/src/runtime.rs`
- Modify: `garnet-interp-v0.3/src/eval.rs`
- Modify: `garnet-cli/templates/agent-orchestrator/src/main.garnet`

- [x] **Step 1: Add declaration-site Sendable checks for actor protocols**

Acceptance: protocol and handler parameters using `@nonsendable` payload types are rejected before run.

- [x] **Step 2: Bridge Garnet `actor` declarations to managed runtime dispatch**

Acceptance: source actor syntax can run through `spawn Actor.handler(args)` in managed mode, not only parse as a deferred declaration.

Evidence: `multi_agent_builder_runs_with_managed_actor_bridge`; `c5_actor_handler_dispatches_via_spawn_bridge`.

- [x] **Step 3: Add managed actor addresses and bounded source mailboxes**

Acceptance: `spawn Actor` returns an address with persistent actor-local state, `Actor.spawn(capacity)` constructs a bounded mailbox, and managed code can `ask`, `tell`/`try_tell`, inspect `mailbox_size`, and `drain` queued protocol messages. Full mailboxes surface explicit failure through `tell` and non-throwing backpressure through `try_tell`.

Evidence: `parses_spawn_keyword_as_member_method_name`; `c5_spawn_actor_returns_address_with_persistent_state`; `c5_actor_address_enforces_bounded_mailbox`; `c5_actor_address_tell_reports_full_mailbox`; `c5_actor_spawn_rejects_extra_capacity_args`.

Remaining: the full `garnet-actor-runtime` OS-thread async address/mailbox bridge is still pending.

- [x] **Step 4: Move `agent-orchestrator` from roadmap prose to actor-mode starter**

Acceptance: a fresh `garnet new --template agent-orchestrator` project runs
Researcher / Synthesizer / Reviewer actor declarations through managed actor
addresses, bounded `try_tell` backpressure, and actor-local memory stores.

Evidence: `cargo test -p garnet-cli --test cli_smoke new_agent_orchestrator_template_runs_and_tests`; generated-project smoke with `garnet run` returning `=> 25` and `garnet test` reporting 3 passed.

## Phase 4: Safe-Mode Ownership Hardening

**Intent:** Convert the safe-mode checker from useful skeleton into conservative language law.

**Files:**

- Modify: `garnet-check-v0.3/src/borrow.rs`
- Modify: `garnet-check-v0.3/src/lib.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [ ] **Step 1: Complete borrow rules B1-B5 with over-rejecting diagnostics**

Acceptance:

```sh
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite -- --ignored
```

- [ ] **Step 2: Implement a conservative NLL subset**

Acceptance:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_nll_lifetime_inference -- --ignored
```

## Phase 5: Traits, Coherence, And Monomorphization

**Intent:** Make the Rust-rigor side credible without overclaiming zero-cost guarantees.

**Files:**

- Modify: `garnet-check-v0.3/src/lib.rs`
- Create: `garnet-check-v0.3/src/coherence.rs`
- Create: `garnet-check-v0.3/src/monomorph.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [ ] **Step 1: Enforce trait coherence with a conservative orphan-rule checker**

- [ ] **Step 2: Add interpreter-level generic instantiation evidence**

- [ ] **Step 3: Defer native zero-cost claims until a compiler backend exists**

## Phase 6: Memory Core Productization

**Intent:** Move Mnemos from reference memory stores toward the Memory Core ambition.

**Files:**

- Modify: `garnet-memory-v0.3/src/`
- Modify: `C_Language_Specification/MEMORY_CORE_ROADMAP.md`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [ ] **Step 1: Implement observable ARC cycle fixtures**

- [ ] **Step 2: Implement Bacon-Rajan trial deletion in a bounded reference path**

- [ ] **Step 3: Add kind-aware root partitioning and safe-mode interaction tests**

## Phase 7: Release, Research, And Repeated Falsification

**Intent:** Put the too-large ambitions into rigorous scaffolds instead of pretending they are done.

**Must remain handoff/scaffold until separately funded/reviewed:**

- formal RustBelt/Iris/Coq mechanization,
- production native compiler,
- true zero-cost monomorphization guarantees,
- signed cross-platform installers,
- empirical PLDI-grade validation studies.

**Acceptance:**

- each item has a tracked handoff file,
- each item has a falsifiable success criterion,
- dogfood readiness is rerun after every phase,
- `canonical_mvp_examples_emit_stable_results` remains green.
