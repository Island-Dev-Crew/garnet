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

- [x] **Step 1: Activate partial B1/B2/B4 borrow-rule conformance**

Acceptance:

```sh
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Evidence: direct use-after-move through `own` parameters and direct
`mut`+`borrow` aliasing are rejected by `garnet check`; managed `def` code
remains ARC-governed rather than affine.

Remaining: full place-granular B1-B5 beyond same-call overlap, method-call
ownership, B3 lifetime containment, broader drop discipline, two-phase borrows,
and NLL are still pending.

- [x] **Step 1B: Track unambiguous method receiver ownership**

Acceptance:

```sh
cargo test -p garnet-check --test borrow
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Evidence: unambiguous same-module `own self` method receivers now move the
receiver binding, receiver `mut` plus borrowed arguments trigger aliasing, and
conflicting same-named method signatures are skipped until type resolution can
disambiguate them.

Remaining: type-resolved impl dispatch and full receiver/field/place-granular
borrows are still pending.

- [x] **Step 1C: Disambiguate simple typed method receivers**

Acceptance:

```sh
cargo test -p garnet-check --test borrow
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Evidence: simple declared receiver types now select the matching impl method
for same-named methods across different receiver types, and typed receivers do
not fall back to another type's method when no matching impl exists.

Remaining: generic receiver types, trait impl dispatch, inferred local types,
and full receiver/field/place-granular borrows are still pending.

- [x] **Step 1D: Track simple field places for aliasing and moves**

Acceptance:

```sh
cargo test -p garnet-check --test borrow
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Evidence: simple field projections now participate in B1/B2 aliasing and B4
move tracking. The checker rejects same-field `mut`+`borrow` aliasing,
parent/child aliasing, and same-field use-after-move while allowing distinct
sibling fields to remain usable.

Remaining: index/dynamic places, generic receiver types, trait impl dispatch,
inferred local types, drop discipline, two-phase borrows, and NLL are still
pending.

- [x] **Step 1E: Track indexed places conservatively**

Acceptance:

```sh
cargo test -p garnet-check --test borrow
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Evidence: `root[index]` projections now participate in B1/B2 aliasing and B4
move tracking as wildcard index sub-places. Indexes under the same receiver
conflict conservatively; nested index receiver operands are still checked; and
indexes under distinct sibling fields remain usable.

Remaining: dynamic places, generic receiver types, trait impl dispatch,
inferred local types, drop discipline, two-phase borrows, and NLL are still
pending.

- [x] **Step 2: Implement a conservative NLL subset**

Acceptance:

```sh
cargo test -p garnet-check --test extended return_ref
cargo test -p garnet-cli --test conformance_skeleton deferred_nll_lifetime_inference
```

Evidence: Phase 4F activates a conservative Mini-Spec §8.5.2
lifetime-elision subset for reference returns. No-input and
multiple-borrowed-input reference returns reject; one borrowed input is
accepted. Full CFG NLL, closure capture lifetimes, variance, dynamic places,
drop discipline beyond same-call overlap, and two-phase borrows remain pending.

- [x] **Step 2B: Reject same-call double-own drop hazards**

Acceptance:

```sh
cargo test -p garnet-check --test borrow double_own
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4G activates a conservative B5 drop-discipline slice. Calls
with overlapping places passed to multiple `own` parameters now reject, covering
same-binding and parent/child double drops while allowing distinct sibling
fields.

Remaining: full CFG NLL, closure capture lifetimes, variance, dynamic places,
generic receiver types, trait impl dispatch, broader scope/branch drop
elaboration, and two-phase borrows remain pending.

- [x] **Step 2C: Add direct-returning branch liveness**

Acceptance:

```sh
cargo test -p garnet-check --test borrow returning
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4H checks `if`/`elsif`/`else` branches independently and only
merges moves from branch bodies that can continue. Values moved inside a
direct-returning branch can be borrowed on later continuing paths, while moves
inside continuing branches and condition expressions remain conservative.

Remaining: full CFG NLL, nested/non-local terminators, loops, closure capture
lifetimes, variance, dynamic places, generic receiver types, trait impl
dispatch, broader scope/branch drop elaboration, and two-phase borrows remain
pending.

- [x] **Step 2D: Stop borrow scans at direct returns and returning loop bodies**

Acceptance:

```sh
cargo test -p garnet-check --test borrow return
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4I reuses the branch-outcome liveness helper for function
bodies and loop bodies. Direct `return` terminates block scanning, and values
moved in direct-returning `while`/`loop` bodies can be borrowed after the loop
on paths where the loop body does not execute.

Remaining: full CFG NLL, nested/non-local terminators, general loop fixed-point
analysis, for-loop fixed-point liveness, closure capture lifetimes, variance,
dynamic places, generic receiver types, trait impl dispatch, broader
scope/branch drop elaboration, and two-phase borrows remain pending.

- [x] **Step 2E: Scope `for` loop variables and returning for bodies**

Acceptance:

```sh
cargo test -p garnet-check --test borrow for_
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4J extends direct-return loop-body liveness to `for` bodies
and checks loop variables in a body-local environment. A direct-returning `for`
body no longer poisons after-loop paths, and a loop variable shadowing a moved
outer binding no longer clears that outer moved state.

Remaining: full CFG NLL, nested/non-local terminators, general loop fixed-point
analysis, closure capture lifetimes, variance, dynamic places, generic receiver
types, trait impl dispatch, broader scope/branch drop elaboration, and
two-phase borrows remain pending.

- [x] **Step 2F: Scope `match` pattern bindings before arm move merging**

Acceptance:

```sh
cargo test -p garnet-check --test borrow match_
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4K records match-arm pattern bindings, checks each arm against
an arm-local environment, and restores those pattern names from the pre-match
snapshot before merging arm move state. A pattern-local move no longer poisons
a same-named outer binding, while a real outer move inside a match arm still
propagates after the match.

Remaining: full CFG NLL, nested/non-local terminators, general loop fixed-point
analysis, closure capture lifetimes, variance, dynamic places, generic receiver
types, trait impl dispatch, broader scope/branch drop elaboration, and
two-phase borrows remain pending.

- [x] **Step 2G: Preserve `match` arm block statements before arm tail values**

Acceptance:

```sh
cargo test -p garnet-parser --test parse_control_flow parses_match_arm_block_with_statements_and_tail
cargo test -p garnet-interp --test eval_control match_arm_block_preserves_statements_before_tail
cargo test -p garnet-check --test borrow match_arm_block_statement_move_still_propagates_after_match
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4L stores each match-arm body as a full `Block`, wraps
expression arms as tail-only blocks, evaluates matched arm blocks with normal
block semantics, walks arm blocks for capability/safe-mode inventory, and
checks arm block statements for moves before merging arm state. Statements
before a match-arm tail now execute and produce borrow diagnostics, and guard
moves still merge when the guard can fail before a returning arm body runs.

Remaining: full CFG NLL, nested/non-local terminators, general loop fixed-point
analysis, closure capture lifetimes, variance, dynamic places, generic receiver
types, trait impl dispatch, broader scope/branch drop elaboration, and
two-phase borrows remain pending.

- [x] **Step 2H: Add finite-domain match exhaustiveness and reachability**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4M adds a scoped safe-mode `match_coverage` pass for finite
domains. `Bool` matches must cover `true` and `false`; same-module enum matches
must cover each variant unless an unguarded catch-all appears; guarded arms do
not count as exhaustive coverage; duplicate covered arms and arms after an
unguarded catch-all are rejected as unreachable.

- [x] **Step 2I: Add finite nested-constructor match coverage**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage safe_nested_enum_match
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4N extends `match_coverage` so finite nested constructor
payloads are enumerated. `Outer::Wrap(Inner::Left)` and
`Outer::Wrap(Inner::Right)` are distinct coverage cases, missing nested payload
cases are rejected, and `Outer::Wrap(_)` covers the nested finite payload
domain without claiming open-domain literal or recursive payload reasoning.

- [x] **Step 2J: Add imported enum alias match coverage**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage safe_imported_
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4O adds scoped import resolution for the safe-mode
`match_coverage` pass. Named imports (`use Types::{Status}`), glob imports
(`use Types::*`), module-qualified aliases, and module-relative imports are
resolved to the finite enum domain, while pattern coverage accepts the source
alias prefix such as `Status::Ready` in addition to the canonical
`Types::Status::Ready`.

- [x] **Step 2K: Add literal guard match coverage reasoning**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage guard
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4P treats literal `if true` match guards as safe coverage and
literal `if false` guards as statically unreachable/non-covering. Non-literal
guards still remain non-covering until a later predicate-proof pass exists.

Remaining: cross-file/package imports, recursive/open payload reasoning, richer
type inference, open-domain exhaustiveness/range reasoning, and non-literal guard reasoning
remain pending.

- [x] **Step 2L: Add open-domain literal match reachability**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage safe_open_
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4Q rejects duplicate open-domain literal arms and arms after
catch-all patterns in safe-mode `match` expressions while preserving unknown
guard conservatism (`1 if ok` does not cover a later `1` arm).

Remaining: cross-file/package imports, recursive/open payload reasoning, richer
type inference, open-domain exhaustiveness/range reasoning, and non-literal
guard reasoning remain pending.

- [x] **Step 2M: Infer immutable local finite match domains from initializers**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage safe_match_uses_local_
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4R seeds safe-mode match coverage from immutable local boolean
literal initializers and enum variant constructor/path initializers, so `let
flag = true` and `let status = Status::Ready()` can trigger finite-domain
non-exhaustiveness diagnostics without explicit local type annotations.

Remaining: direct mutable-local assignment tracking is covered in Step 2N
below; cross-file/package imports, recursive/open payload reasoning, richer type
inference, open-domain exhaustiveness/range reasoning, and non-literal guard
reasoning remain pending.

- [x] **Step 2N: Track direct mutable-local match-domain assignments**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage mutable_
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4S adds assignment-sensitive direct `let mut` match-domain
tracking: finite boolean/enum assignments seed safe-mode match coverage, and
non-finite assignments invalidate inferred finite-domain state before later
matches.

Remaining: direct `if`/`elsif`/`else` branch-merged assignment flow is covered
in Step 2O below; compound-assignment invalidation is covered in Step 2Q
below; cross-file/package
imports, recursive/open payload reasoning, richer type inference, open-domain
exhaustiveness/range reasoning, and non-literal guard reasoning remain
pending.

- [x] **Step 2O: Join branch-local match-domain assignments**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage if_else_assignments
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4T carries direct mutable-local match-domain evidence through
conservative `if`/`elsif`/`else` joins only when every possible branch preserves
the same finite domain. Bool and enum branch assignments now drive
non-exhaustiveness diagnostics after the conditional, while mixed
finite/non-finite branches clear stale finite-domain state before later
matches.

Remaining: nested all-path `if` branch assignment flow is covered in Step 2P
below; compound-assignment invalidation is covered in Step 2Q below;
loop-body invalidation is covered in Step 2R below; `try`/`ensure`
invalidation and uninvoked closure-definition boundaries are covered in Step
2S below; direct closure-literal invocation invalidation is covered in Step
2T below; direct local closure-literal binding call invalidation is covered in
Step 2U below; branch-joined local closure-literal binding call invalidation
is covered in Step 2V below; branch-rebound local closure-literal binding call
invalidation is covered in Step 2W below; direct local closure-alias binding
call invalidation is covered in Step 2X below; branch-joined local closure-alias
call invalidation is covered in Step 2Y below; direct branch-selected closure
expression call invalidation is covered in Step 2Z below; immutable local
boolean guard constants are covered in Step 2ZA below; same-module top-level
boolean guard constants are covered in Step 2ZB below; scoped named/glob
imported top-level boolean guard constants are covered in Step 2ZC below;
path-qualified top-level boolean guard constants are covered in Step 2ZD below;
narrow boolean const aliases are covered in Step 2ZE below;
basic boolean const expressions are covered in Step 2ZF below;
short-circuit boolean const expressions are covered in Step 2ZG below;
boolean const equality/inequality expressions are covered in Step 2ZH below;
direct boolean match guard expressions are covered in Step 2ZI below;
integer const equality/inequality expressions are covered in Step 2ZJ below;
integer const relational expressions are covered in Step 2ZK below;
integer const arithmetic expressions are covered in Step 2ZL below;
loop fixed-point and broader mutable/escaped/general higher-order closure
invocation/call-effect flow, cross-file/package imports, recursive/open payload
reasoning, richer type inference, non-integer/broader comparison,
function-call, and broader const expression evaluation, open-domain exhaustiveness/range
reasoning, and broader non-literal guard reasoning remain pending.

- [x] **Step 2P: Join nested if assignment domains inside branch bodies**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage nested_if_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4U carries match-domain evidence through nested `if` /
`elsif` / `else` expressions inside branch bodies only when every nested path
definitely assigns the outer subject. Missing nested `else` paths remain
open-domain, and branch-local bindings remain ineligible for post-branch
finite-domain evidence.

- [x] **Step 2Q: Make compound assignments an explicit invalidation boundary**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage compound_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4V proves direct compound assignments and all-branch
compound assignments clear finite `Bool`/enum match-domain evidence before a
later match, so operator/type-dependent updates cannot reuse stale finite
domains.

- [x] **Step 2R: Invalidate domains after possible loop-body assignments**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage loop_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4W clears finite match-domain evidence when an outer binding
may be assigned in a `while`, `for`, or `loop` body, including conditional
assignments inside a loop body, while ordered shadowing tests preserve outer
domains when a loop-local binding merely shadows the same name and still clear
outer evidence when an assignment appears before the local shadow.

- [x] **Step 2S: Invalidate try-flow domains and isolate closure definitions**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage try_
cargo test -p garnet-check --test match_coverage uninvoked_closure
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4X clears stale finite match-domain evidence after
`try`/`rescue`/`ensure` writes while preserving the existing safe-mode
`try`/`rescue` rejection, and treats uninvoked closure literals as definition
boundaries so their body assignments do not merge into enclosing flow.

- [x] **Step 2T: Invalidate direct closure-literal invocation domains**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage immediate_closure
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4Y clears finite match-domain evidence after directly invoked
closure literals whose block or expression bodies assign the match subject,
without claiming broader stored closure invocation/call-effect analysis.

- [x] **Step 2U: Invalidate direct local closure-literal binding calls**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage local_closure_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4Z tracks the conservative outer-write effect of local closure
literals bound in the current block and clears finite match-domain evidence
when that local binding is called directly. Escaped closures, general higher-order calls, and broader mutable closure flow remain
deferred.

- [x] **Step 2V: Invalidate branch-joined local closure binding calls**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage branch_joined_local_closure_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AA joins the conservative outer-write effects from every
closure literal returned by an `if` / `elsif` / `else` expression assigned to a
local binding, then clears finite match-domain evidence when that binding is
called directly. Escaped closures, general higher-order calls, and broader mutable closure flow remain
deferred.

- [x] **Step 2W: Invalidate branch-rebound local closure binding calls**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage branch_rebound_local_closure_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AB joins local closure-effect maps after all-path branch
rebinding of a local closure binding to known closure literals, then clears
finite match-domain evidence when that binding is called directly. Escaped
closures, higher-order calls, and broader mutable closure flow remain deferred.

- [x] **Step 2X: Invalidate direct local closure-alias calls**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage local_closure_alias_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AC copies the conservative outer-write effect of a known local
closure binding through a direct local alias, then clears finite match-domain
evidence when that alias is called directly. Escaped closures, general
higher-order calls, and broader mutable closure flow remain deferred.

- [x] **Step 2Y: Invalidate branch-joined local closure-alias calls**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage branch_joined_local_closure_alias_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AD carries known local closure effects through all-path
branch-selected direct aliases and clears finite match-domain evidence when that
alias is called directly, while preserving unknown behavior when a branch-local
shadowed tail is not a known closure. Escaped closures, general higher-order
calls, and broader mutable closure flow remain deferred.

- [x] **Step 2Z: Invalidate direct branch-selected closure expression calls**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage direct_branch
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AE reuses the known local closure-effect extractor for direct
calls whose callee is an all-path branch expression. It clears finite
match-domain evidence for branch-selected closure writes while preserving
unknown behavior when a branch-local shadowed tail is not a known closure.

- [x] **Step 2ZA: Recognize immutable local boolean guard constants**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage bool_guard
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AF tracks immutable local boolean guard constants separately
from finite-domain subject inference. `let always = true` guards count as
coverage, `let never = false` guards are statically false and non-covering,
and `let mut always = true` remains unknown so mutable guard locals cannot
produce stale coverage.

- [x] **Step 2ZB: Recognize same-module top-level boolean guard constants**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage const_bool_guard
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AG seeds guard facts from same-module top-level boolean
`const` items. `const ALWAYS = true` guards count as coverage,
`const NEVER = false` guards are statically false and non-covering, and
function parameters with the same name shadow the const fact so parameterized
guards remain conservative.

- [x] **Step 2ZC: Recognize imported top-level boolean guard constants**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage imported
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AH resolves scoped named and glob imports of top-level boolean
`const` guard facts. Named-imported and module-relative imported true const
guards count as coverage, glob-imported false const guards are statically false
and non-covering, and function parameters with the same name shadow the
imported const fact so parameterized guards remain conservative.

- [x] **Step 2ZD: Recognize path-qualified boolean guard constants**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage path_qualified
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AI resolves path-qualified top-level boolean `const` guard
expressions through the same scoped const-fact index. `Flags::ALWAYS` guards
count as coverage, `Flags::NEVER` guards are statically false and
non-covering, and ambiguous or non-constant paths remain conservative. Broad
const expression evaluation remains deferred.

- [x] **Step 2ZE: Resolve narrow boolean const guard aliases**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage const_bool_alias
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AJ resolves direct boolean const aliases through the scoped
const-fact index without evaluating arbitrary expressions. Path-valued aliases
such as `Flags::ALWAYS = Core::RAW` count as coverage when they resolve to
`true`, resolve to statically false/non-covering guards when they resolve to
`false`, and leave arithmetic, comparison, function-call, and broader const
expression evaluation deferred.

- [x] **Step 2ZF: Fold basic boolean const guard expressions**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage boolean_const_expression
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AK folds basic boolean `not`, `and`, and `or` const
expressions over already-resolved boolean facts. Expressions such as
`Core::RAW and not false` count as coverage when they resolve to `true`;
expressions such as `not Core::RAW or false` resolve to statically
false/non-covering guards when they resolve to `false`. Arithmetic,
comparison, function-call, recursive, and broader const expression evaluation
remain deferred.

- [x] **Step 2ZG: Honor short-circuit boolean const guard expressions**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage short_circuit
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AL honors decisive left operands for boolean const
`or`/`and` expressions. `true or Missing::VALUE` counts as coverage without
resolving the right operand, and `false and Missing::VALUE` is statically
false/non-covering without resolving the right operand. Arithmetic, comparison,
function-call, recursive, cross-file/package, and broader const expression
evaluation remain deferred.

- [x] **Step 2ZH: Fold boolean const equality/inequality guard expressions**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage boolean_const_equality
cargo test -p garnet-check --test match_coverage boolean_const_inequality
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AM folds boolean `==` and `!=` const expressions over
already-resolved boolean facts. `Core::RAW == true` counts as coverage, and
`Core::RAW != true` is statically false/non-covering. Arithmetic, relational
comparison, function-call, recursive, cross-file/package, and broader const
expression evaluation remain deferred.

- [x] **Step 2ZI: Fold direct boolean match guard expressions**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage direct_true_boolean_const_equality
cargo test -p garnet-check --test match_coverage direct_false_boolean_const_inequality
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AN applies conservative boolean fact folding directly to
match guard expressions. `Status::Ready if Core::RAW == true` counts as
coverage, and `Status::Ready if Core::RAW != true` is statically
false/non-covering without requiring an intermediate alias const. Arithmetic,
relational comparison, function-call, recursive, cross-file/package, and
broader const expression evaluation remain deferred.

- [x] **Step 2ZJ: Fold integer const equality/inequality guard facts**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage integer_const_equality
cargo test -p garnet-check --test match_coverage false_integer_const_inequality
cargo test -p garnet-check --test match_coverage direct_integer_const_equality
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AO extends the narrow guard fact domain to integer literals
for equality/inequality only. `Core::LIMIT == 2` counts as coverage, and
`Core::LIMIT != 2` is statically false/non-covering. Arithmetic, relational
comparison, function-call, recursive, cross-file/package, and broader const
expression evaluation remain deferred.

- [x] **Step 2ZK: Fold integer const relational guard facts**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage integer_const_less_than
cargo test -p garnet-check --test match_coverage false_integer_const_greater_than
cargo test -p garnet-check --test match_coverage direct_integer_const_greater_equal
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AP folds narrow integer `<`, `<=`, `>`, and `>=` const
comparisons over the existing guard fact domain. `Core::LIMIT < 3` and
`Core::LIMIT >= 2` count as coverage, while `Core::LIMIT > 3` is statically
false/non-covering. Arithmetic, non-integer/broader comparison, function-call,
recursive, cross-file/package, and broader const expression evaluation remain
deferred.

- [x] **Step 2ZL: Fold checked integer arithmetic guard facts**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage integer_const_arithmetic
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AQ folds checked integer `+`, `-`, `*`, `/`, `%`, and unary
`-` arithmetic inside the existing guard fact domain. `Core::LIMIT +
Core::OFFSET == 3` and `Core::LIMIT + 1 >= 3` count as coverage, while
`Core::LIMIT * 2 < 4` is statically false/non-covering. Non-integer/broader
comparison, function-call, recursive, cross-file/package, and broader const
expression evaluation remain deferred.

- [x] **Step 2ZM: Carry integer const facts through scoped guard identifiers**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage integer_const_identifiers
cargo test -p garnet-check --test match_coverage imported_glob_integer_const
cargo test -p garnet-check --test match_coverage function_parameter_shadows_imported_integer_const
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AR carries integer `ConstFact` values through same-module
bare guard identifiers and scoped named/glob imported top-level integer
`const` identifiers. `LIMIT + OFFSET == 3` and imported `use Core::{LIMIT,
OFFSET}` guard expressions count as coverage, glob-imported false integer
identifier guards are statically false/non-covering, and parameter-shadowed
imported integer identifiers remain unknown/non-covering. Cross-file/package
imports, non-integer comparison, function-call evaluation, recursion, and
broader const expression evaluation remain deferred.

- [x] **Step 2ZN: Fold literal symbol and string const equality facts**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage symbol_const_equality
cargo test -p garnet-check --test match_coverage false_string_const_inequality
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AS extends the narrow const guard fact domain to static
symbols and plain non-interpolated strings for equality/inequality checks.
`Core::MODE == :ready` counts as coverage, and `Core::LABEL != "ready"` is
statically false/non-covering. Interpolated strings, ordering comparisons,
function-call evaluation, recursion, cross-file/package imports, and broader
const expression evaluation remain deferred.

- [x] **Step 2ZO: Fold nil const equality facts**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage nil_const_equality
cargo test -p garnet-check --test match_coverage false_nil_const_inequality
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AT extends the narrow const guard fact domain to `nil`
for equality/inequality checks. `Core::EMPTY == nil` counts as coverage, and
`Core::EMPTY != nil` is statically false/non-covering. Broader const expression
evaluation remains deferred.

- [x] **Step 2ZP: Fold mixed literal const equality facts**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage mixed_literal_const_inequality
cargo test -p garnet-check --test match_coverage false_mixed_literal_const_equality
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AU applies the runtime equality rule for distinct known
literal kinds inside the narrow const guard fact domain. `Core::EMPTY != false`
counts as coverage, and `Core::EMPTY == false` is statically
false/non-covering. Float arithmetic, non-finite floats, interpolated strings, ordering comparisons,
function-call evaluation, recursion, cross-file/package imports, and broader
const expression evaluation remain deferred.

- [x] **Step 2ZQ: Fold finite float const equality facts**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage float_const
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AV extends the narrow const guard fact domain to finite
floats and runtime-aligned int-float equality. `Core::RATIO == 1.5` and
`Core::COUNT == 1.0` count as coverage, `Core::RATIO != 1.5` is statically
false/non-covering, and non-finite float facts remain unknown. Float
arithmetic, ordering comparisons, interpolated strings, function-call
evaluation, recursion, cross-file/package imports, and broader const expression
evaluation remain deferred.

- [x] **Step 2ZR: Fold finite float const relational facts**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage float_const_relational
cargo test -p garnet-check --test match_coverage non_finite_float_relational
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AW aligns relational `ConstFact` folding with the runtime
numeric comparison rule for finite `Float`/`Float`, `Int`/`Float`, and
`Float`/`Int` pairs. `Core::RATIO < 2.0` and `Core::COUNT <= 2.0` count as
coverage, `Core::RATIO > 2.0` is statically false/non-covering, and non-finite
float relational facts remain unknown. Float arithmetic, interpolated strings,
function-call evaluation, recursion, cross-file/package imports, broader
non-numeric comparison, and broader const expression evaluation remain
deferred.

- [x] **Step 2ZS: Fold finite float const arithmetic facts**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage float_const_arithmetic
cargo test -p garnet-check --test match_coverage non_finite_float_arithmetic
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AX aligns arithmetic `ConstFact` folding with checked integer
arithmetic plus runtime numeric arithmetic for finite `Float`/`Float`,
`Int`/`Float`, and `Float`/`Int` pairs. `Core::RATIO + 0.5 == 2.0` and
`Core::COUNT * 1.5 >= 3.0` count as coverage, `Core::RATIO * 2.0 < 3.0` is
statically false/non-covering, and overflow-to-infinity remains unknown.
Interpolated strings, function-call evaluation, recursion, cross-file/package
imports, broader non-numeric comparison, broader float edge-case reasoning, and
broader const expression evaluation remain deferred.

- [x] **Step 2ZT: Fold immutable local guard expression aliases**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage local_boolean_const_expression
cargo test -p garnet-check --test match_coverage local_integer_const_expression
cargo test -p garnet-check --test match_coverage mutable_local_expression_source
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AY carries the existing narrow `ConstFact` evaluator through
immutable local guard aliases. `let always = raw == true` and `let always =
limit + 1 == 3` now count as coverage, `let never = limit + 1 < 3` is
statically false/non-covering, and `let mut limit = 2` remains unknown when a
later guard alias depends on it. Path-qualified local alias expressions,
function-call evaluation, recursion, cross-file/package imports, broader
non-numeric comparison, broader float edge-case reasoning, and broader const
expression evaluation remain deferred.

- [x] **Step 2ZU: Resolve path-qualified consts in local guard expression aliases**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage local_path_integer_const_expression
cargo test -p garnet-check --test match_coverage local_
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AZ resolves path-qualified top-level constants inside
immutable local guard expression aliases. `let always = Core::LIMIT + 1 == 3`
now counts as coverage, and `let never = Core::LIMIT + 1 < 3` is statically
false/non-covering. Function-call evaluation, recursion, cross-file/package
imports, broader non-numeric comparison, broader float edge-case reasoning, and
broader const expression evaluation remain deferred.

- [x] **Step 2ZV: Fold static interpolated string const facts**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage interpolated_string
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4BA folds static interpolated string const facts whose
interpolation bodies already resolve through the same narrow `ConstFact`
evaluator. `"re#{"ad"}y" == "ready"` now counts as coverage, and
`"re#{"ad"}y" != "ready"` is statically false/non-covering. Function-call
interpolation, recursion, cross-file/package imports, broader non-numeric
comparison, broader float edge-case reasoning, and broader const expression
evaluation remain deferred.

- [x] **Step 2ZW: Fold static string relational const facts**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage string_const_relational
cargo test -p garnet-check --test match_coverage mixed_string_symbol_relational
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4BB folds runtime-aligned static string relational facts.
`Core::LABEL < "rust"` now counts as coverage, and `Core::LABEL > "ready"` is
statically false/non-covering. Mixed string/symbol relational facts,
function-call interpolation, recursion, cross-file/package imports, broader
non-string non-numeric comparison, broader float edge-case reasoning, and
broader const expression evaluation remain deferred.

- [x] **Step 2ZX: Fold static boolean relational const facts**

Acceptance:

```sh
cargo test -p garnet-check --test match_coverage boolean_const_relational
cargo test -p garnet-check --test match_coverage mixed_boolean_nil_relational
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4BC folds runtime-aligned static boolean relational facts.
`Core::RAW < true` now counts as coverage under the runtime's `false < true`
ordering, and `Core::RAW < false` is statically false/non-covering. Mixed
boolean/nil relational facts, function calls, recursion, cross-file/package
imports, broader non-boolean non-string non-numeric comparison, broader float
edge-case reasoning, and broader const expression evaluation remain deferred.

## Phase 5: Traits, Coherence, And Monomorphization

**Intent:** Make the Rust-rigor side credible without overclaiming zero-cost guarantees.

**Files:**

- Modify: `garnet-check-v0.3/src/lib.rs`
- Create: `garnet-check-v0.3/src/coherence.rs`
- Create: `garnet-check-v0.3/src/monomorph.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [x] **Step 1: Enforce trait coherence with a conservative orphan-rule checker**

Acceptance:

```sh
cargo test -p garnet-check --test coherence
cargo test -p garnet-cli --test conformance_skeleton deferred_trait_coherence
```

Evidence: Phase 5C rejects exact duplicate trait impls, orphan impls where
neither the trait nor the type is local, simple generic blanket-vs-concrete
overlaps, renamed generic blanket overlaps, and qualified external type
short-name collisions, while preserving local-trait, local-type, and qualified
local-module positive cases. Full specialization and imported-package
coherence solving remain pending.

- [x] **Step 1B: Add conservative generic-overlap and qualified-path coherence**

Acceptance:

```sh
cargo test -p garnet-check --test coherence
cargo test -p garnet-cli --test conformance_skeleton deferred_trait_coherence
```

- [x] **Step 2: Add interpreter-level generic instantiation evidence**

Evidence: Phase 5B activates
`generic_instantiation_runs_without_monomorphization_claims`, proving a generic
struct, generic impl method, and generic function through the managed
interpreter while keeping native zero-cost claims out of scope.

- [x] **Step 3: Defer native zero-cost claims until a compiler backend exists**

## Phase 6: Memory Core Productization

**Intent:** Move Mnemos from reference memory stores toward the Memory Core ambition.

**Files:**

- Modify: `garnet-memory-v0.3/src/`
- Modify: `C_Language_Specification/MEMORY_CORE_ROADMAP.md`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [x] **Step 1: Implement observable ARC cycle fixtures**

Phase 6A added `garnet-memory-v0.3/src/cycle.rs` and
`garnet-memory-v0.3/tests/cycle.rs`, then activates
`deferred_arc_cycle_detection` as a bounded reference-model gate for retained
roots, collectable unrooted cycles, unrooted acyclic retention, and
kind-scheduled cross-kind collection.

- [x] **Step 2: Implement bounded Bacon-Rajan-style trial deletion reference path**

Phase 6B exposes trial candidates and scan-black retained candidates, then runs
a bounded mark-gray / scan / collect-white pass over the deterministic cycle
graph. This is still not production allocator-integrated ARC.

- [x] **Step 3: Add finalization-order and safe-mode interaction reference tests**

Phase 6C exposes deterministic collect-white finalization order and models
safe-mode affine nodes as retained non-ARC allocations excluded from trial
candidates.

- [x] **Step 4: Add bounded root-buffer/decrement-event reference path**

Phase 6D adds `CycleRootBuffer`, threshold-triggered collection, and buffered
candidate scans so the reference model no longer relies only on whole-graph
unrooted candidate discovery.

- [x] **Step 5: Add allocator-owned root/edge decrement fixture**

Phase 6E adds `CycleAllocatorFixture`, proving that the allocator-facing
surface can own the graph plus root buffer and route root releases and ARC edge
removals through buffered trial-deletion scheduling.

- [x] **Step 6: Add kind-aware allocator surface and policy-configured lazy eviction**

Phase 6J adds an object-safe `KindAllocator` / `HeapKindAllocator` surface with
allocator stats across all four Memory Core stores. `EpisodeStore::with_policy`
and `VectorIndex::with_policy` now lazily compact on read/search using
`MemoryPolicy::score`, `should_retain`, and `compaction_high_water` while
default constructors preserve the original unbounded reference behavior.

Cache-security sidecar: Phase 6I adds keyed source-tree binding for
compiler-as-agent episode records, quarantines copied same-machine strategy
rows whose replayed justifications do not verify in the current source tree,
and preserves valid NDJSON/all verified records under a 16-writer bounded
append soak.

- [x] **Step 7: Connect store root lifecycles to the cycle-aware allocator adapter**

Phase 6K adds `CycleAwareKindAllocator`, `AllocRootStats`, and object-safe root
hooks on `KindAllocator`. The four Memory Core stores now retain observable
roots on write and release them on clear, policy eviction, workflow replacement,
and drop. This proves store-root lifecycle wiring through the bounded cycle
fixture, not the final production ARC backend.

- [x] **Step 8: Add fenced episodic text snapshot persistence**

Phase 6L adds `EpisodeStore::save_text` / `load_text` for versioned episodic
text snapshots. Payloads are hex-encoded, writes go through a sibling temp file
before rename, malformed files fail before mutating the live store, and loaded
episodes retain fresh cycle-aware roots. This is reference-store recovery
evidence, not a pluggable production persistence backend.

- [x] **Step 8b: Add guarded append-style episodic text log commits**

Phase 6M adds `EpisodeStore::append_text` for dependency-free incremental
text-log commits. Existing logs are size-bounded and parsed as the store value
type before extension, corrupt, empty, type-invalid, or oversized logs are not
carried forward, projected oversize commits are rejected before file creation,
accepted record data is synced through a temp-file rewrite and rename, and the
live store changes only after the on-disk commit succeeds. This is still a
guardrail on the reference text format, not a broad pluggable persistence
backend.

- [x] **Step 8c: Bind episodic text commits to the default typed cache backend**

Phase 6N adds `episodic_cache_log_path_for` plus
`EpisodeStore::append_cache_text` / `load_cache_text` for fixed per-project
`.garnet-cache/episodic/episodes.mnemos` storage. The backend canonicalizes the
project root, creates private cache directories, rejects symlinked or
non-regular targets, refuses oversized loads before allocation, serializes
rewrite-based access with an OS-backed lockfile on Unix/Windows, anchors Unix
backend file operations to the validated episodic directory handle, keeps Unix
backend files private from creation time, and preserves corrupt/type-invalid
non-mutation and cycle-aware root rehydration.
This is the typed Mnemos backend boundary, not the CLI signed NDJSON
advisory-cache trust model, not trusted compiler input, and not a broad
pluggable persistence layer.

- [x] **Step 8d: Sync accepted text commit directories on Unix**

Phase 6O closes the first durability gap after atomic text commit renames.
Accepted `save_text`, `append_text`, and prepared typed cache commits sync the
containing directory after the temp file is renamed into place. The default
typed cache backend performs this through the validated episodic directory
handle rather than reopening the mutable path. Non-Unix platforms keep the
existing file-data sync behavior until a platform-specific directory-sync
contract is added.

- [x] **Step 8e: Bind typed episodic cache files to the source tree**

Phase 6P adds a dependency-free `source-tree` binding line to the default typed
cache backend format. `EpisodeStore::append_cache_text` validates that binding
before extending an existing `.garnet-cache/episodic/episodes.mnemos`, and
`load_cache_text` rejects copied typed cache files from another canonical
project root before replacing the live store. This is same-machine replay
hardening for the typed Mnemos backend; a cryptographic MAC and trusted
compiler-advice contract remain future work.

- [ ] **Step 9: Promote root-buffer/finalizer/safe-mode interaction into production allocator tests**

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
