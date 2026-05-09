# Garnet Language Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn Garnet's original dual-mode, agent-native language ambition into a sequence of executable milestones that can be reviewed, falsified, and eventually presented as a stable MIT-grade language/toolchain.

**Architecture:** Complete Garnet by vertical language slices. A feature is not current truth until parser/AST support, checker or type semantics, interpreter/runtime behavior, conformance tests, dogfood examples, documentation status, and CI gates line up.

**Tech Stack:** Rust workspace, `garnet-parser-v0.3`, `garnet-check-v0.3`, `garnet-interp-v0.3`, `garnet-actor-runtime`, `garnet-memory-v0.3`, `garnet-cli`, Cargo integration tests, GitHub Actions, Markdown evidence ledgers, dogfood-readiness reports.

---

## Completion Ledger

This table is the current truth as of the v0.5 readiness-remediation branch. It separates what is already executable from work that is parsed-only, partial, or still future research.

| Ambition | Current status | Evidence | Next executable gate |
|---|---|---|---|
| 10 MVP app corpus | Complete for current v0.4.2 examples | `garnet-cli/tests/dogfood_readiness_examples.rs`; CI `canonical MVP examples` job | Keep `cargo test -p garnet-cli --test dogfood_readiness_examples` green |
| Current-state/reviewer guide | Complete first pass | `CURRENT_STATE.md`; `F_Project_Management/GARNET_CURRENT_VS_HISTORICAL_LEDGER.md` | Review before each public/MIT packaging pass |
| Repo IA truth separation | Complete first pass | `CURRENT_STATE.md`; `archive/history/`; roadmap index | Finish link-rewrite cleanup before a public main-page launch |
| v0.4.2 release assets | Fork release published; org release path requires browser/desktop-authorized publication | `Navigata1/garnet` release assets; CLI reports org `push: false`; browser session can open org release form | Publish org `v0.4.2` release from an org-authorized session and rerun installer smoke |
| Parser parity for old ambition | Partial, Phase 1 active | `protocol`, `dyn Trait`, `yield`, `next`, `@dynamic`, `@nonsendable`, and `do ... end` parser tests | Keep runtime gaps explicit and activate Phase 2 only with executable semantics |
| Blocks, `yield`, `next` runtime semantics | Phase 2A active | `do ... end` parses as a trailing closure argument; `deferred_blocks_and_yield` runs a managed-mode block/yield/next program | Keep `cargo test -p garnet-cli --test conformance_skeleton deferred_blocks_and_yield` green; add richer block edge cases later |
| Dynamic method dispatch tables | Partial Phase 2H | `deferred_dynamic_dispatch` covers per-instance method tables; `static_impl_dispatch_and_method_missing` covers static inherent impl fallback and `method_missing`; `dynamic_impl_dispatch_tables` covers `@dynamic impl Type for Protocol` registration and dispatch | Add richer dispatch precedence and ambiguity probes |
| Structural protocol satisfaction and runtime casts | Partial Phase 2H | `Item::Protocol` and `Expr::Cast` parse; `deferred_structural_protocols` checks protocol-typed managed parameters, runtime `as Protocol` casts, static/dynamic methods, mode/arity/parameter/return annotation mismatches, generic protocol substitution, core built-in typed method signatures, and `@dynamic impl` methods | Add broader trait/generic coherence |
| Actor protocol enforcement and `Sendable` | Partial Phase 3D | actor runtime crate exists; `actor_sendable_rejects_nonsendable_protocol_payloads` rejects `@nonsendable` actor protocol payloads before runtime; managed interpreter now registers actors, dispatches `spawn Actor.handler(args)` synchronously, creates `spawn Actor` addresses with persistent actor-local state, enforces bounded source mailboxes through `Actor.spawn(capacity)`, and ships a generated `agent-orchestrator` actor template that runs/tests through managed actor addresses; full async OS-thread bridge remains partial | Bridge generated actor projects to the full async `garnet-actor-runtime` OS-thread address/mailbox runtime |
| Rust-grade NLL and borrow rules | Partial Phase 4H | `garnet-check-v0.3/src/borrow.rs`; `garnet-check-v0.3/src/lib.rs`; `garnet-check-v0.3/tests/borrow.rs`; `garnet-check-v0.3/tests/extended.rs`; `partial_borrow_rule_suite` rejects direct use-after-move, direct mut-aliasing, `own self` method receiver moves, method receiver aliasing, simple typed receiver disambiguation, simple field-place aliasing/field use-after-move, and conservative index-place aliasing/index use-after-move while checking nested index operands; `deferred_full_borrow_rule_suite` now covers B5 same-call overlapping `own` drop discipline plus direct-returning branch liveness; `deferred_nll_lifetime_inference` covers conservative reference-return lifetime elision | Activate full CFG NLL, dynamic place tracking, generic/trait impl dispatch, broader drop elaboration, loops, and two-phase borrows |
| Trait coherence | Partial Phase 5C | `garnet-check-v0.3/src/coherence.rs`; `garnet-check-v0.3/tests/coherence.rs`; `deferred_trait_coherence` rejects exact duplicate trait impls, orphan-rule violations, simple generic blanket-vs-concrete overlaps, renamed generic blanket overlaps, and qualified external type short-name collisions while allowing local-trait, local-type, and qualified local-module impls | Activate specialization and imported-package coherence solving |
| Generic instantiation / monomorphization | Partial Phase 5B | `generic_instantiation_runs_without_monomorphization_claims` runs generic struct construction, a generic impl method, and a generic function through the managed interpreter | Keep native zero-cost monomorphization deferred until a compiler backend exists |
| Memory Core ARC/cycles and allocator integration | Partial Phase 6L | `garnet-memory-v0.3/src/{alloc,cycle,working,episodic,semantic,procedural}.rs`; `garnet-memory-v0.3/tests/{cycle,properties,persistence}.rs`; active `deferred_arc_cycle_detection`; `CycleAllocatorFixture` owns graph + root buffer for root/edge decrement scheduling; all four stores expose kind-aware allocator stats; policy-configured episodic/semantic stores evict lazily on read/search; `CycleAwareKindAllocator` observes store-root retain/release lifecycles on write, clear, eviction, replacement, and drop; `EpisodeStore::save_text` / `load_text` now prove versioned episodic text snapshot recovery, delimiter-safe payload encoding, malformed-file non-mutation, and cycle-aware root rehydration | Promote the bounded allocator-owned fixture model into production allocator-integrated ARC and broaden persistence/backend hardening beyond the reference episodic snapshot slice |
| Compiler-as-agent cache privacy/replay | Partial Phase 6I | `garnet-cli/src/{cache,cmd,provenance}.rs`; `garnet-cli/tests/cache_episodes.rs`; cache episode logs redact external absolute paths, collapse project-local absolute paths to stable relative labels, warn while ignoring same-cache foreign-key plus copied-cache replay episodes, bind verified episodes to a keyed source-tree identifier, quarantine copied/stale strategy rows whose provenance does not re-verify in the current source tree, and preserve bounded concurrent plus 16-writer soak appends; CacheHMAC and ProvenanceStrategy tests remain active | Add extended release-duration/cross-platform cache soak and keep production Memory Core ARC integration separate |
| Native compiler | Long-horizon scaffold only | no backend crate | Create backend design PR before claiming compiled language status |
| Formal RustBelt/Iris/Coq proof | Long-horizon scaffold only | Paper V theorem sketches | Open proof repo or `proofs/` workspace with checked theorem stubs |
| Signed cross-platform installers | Partial | Linux packages and checksums exist; macOS/Windows signing remains separate authority work | Signed/notarized macOS and Authenticode Windows install smokes |
| Empirical PLDI-grade validation | Long-horizon scaffold only | benchmarking and empirical protocols exist | Run pre-registered studies with archived datasets/scripts |

## Done Means Executable

Do not advance public status for any row unless all applicable evidence lands in the same PR:

1. A failing conformance or dogfood test that names the missing behavior.
2. Parser/AST support when syntax is involved.
3. Checker/type-system support when the spec promises safety or conformance.
4. Runtime/interpreter support when the feature is user-visible.
5. A canonical example, template, or MVP app smoke when users or agents touch it.
6. Updated conformance matrix and current-state docs.
7. Local verification plus GitHub Actions verification before merge.

## Milestone 1: Parser Parity Baseline

**Purpose:** Accept the old design syntax in a controlled parser-stage form without claiming runtime semantics.

**Files:**

- Modify: `garnet-parser-v0.3/src/token.rs`
- Modify: `garnet-parser-v0.3/src/ast.rs`
- Modify: `garnet-parser-v0.3/src/grammar/mod.rs`
- Modify: `garnet-parser-v0.3/src/grammar/user_types.rs`
- Modify: `garnet-parser-v0.3/src/grammar/types.rs`
- Modify: `garnet-parser-v0.3/src/grammar/stmts.rs`
- Modify: `garnet-parser-v0.3/src/grammar/expr.rs`
- Test: `garnet-parser-v0.3/tests/parse_v1_parser_parity.rs`
- Test: `garnet-parser-v0.3/tests/properties.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`
- Docs: `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md`

- [x] **Step 1: Land parser-stage support for `protocol`, `dyn Trait`, `yield`, `next`, `@dynamic`, and `@nonsendable`**

Run:

```sh
cargo test -p garnet-parser --test parse_v1_parser_parity
cargo test -p garnet-cli --test conformance_skeleton
```

Expected: parser-parity tests pass; deferred runtime/type-system handles remain ignored.

- [x] **Step 2: Add failing parser test for `do ... end` block arguments**

Add this test to `garnet-parser-v0.3/tests/parse_v1_parser_parity.rs`:

```rust
#[test]
fn parses_do_end_block_argument() {
    let src = r#"
def main() {
  each([1, 2, 3]) do |x|
    yield x + 1
  end
}
"#;
    parse_ok(src);
}
```

Run:

```sh
cargo test -p garnet-parser --test parse_v1_parser_parity parses_do_end_block_argument
```

Observed before implementation: failed with `UnexpectedToken` at the newline
after the `do |x|` header because block arguments were not parsed.

- [x] **Step 3: Implement `do ... end` parser support**

Parse `do ... end` as an `Expr::Closure` appended to the call or method-call
argument list. Phase 2A now tags syntactic `do...end` closures so runtime
block dispatch can distinguish them from ordinary first-class closure
arguments.

Run:

```sh
cargo test -p garnet-parser --test parse_v1_parser_parity
cargo test -p garnet-cli --test conformance_phase_gates
```

Expected after implementation: parser test passes; phase gates still prove runtime rows are not silently marked complete.

## Milestone 2: Managed Runtime Semantics

**Purpose:** Make parser-stage surfaces run in managed mode with explicit behavior.

**Files:**

- Modify: `garnet-interp-v0.3/src/value.rs`
- Modify: `garnet-interp-v0.3/src/env.rs`
- Modify: `garnet-interp-v0.3/src/eval.rs`
- Modify: `garnet-interp-v0.3/src/stmt.rs`
- Modify: `garnet-interp-v0.3/src/control.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`
- Example: `examples/mvp_06_multi_agent.garnet`

- [x] **Step 1: Replace the ignored block/yield placeholder with a failing executable test**

Change `deferred_blocks_and_yield` so it runs a Garnet program that passes a block and yields a value through it.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_blocks_and_yield
```

Observed before implementation: failed with `arity mismatch: expected 0, got 1` because the trailing block was still treated as a normal argument.

- [x] **Step 2: Add runtime support for callable block objects**

Use the existing closure-backed `Value::Fn` representation as the block object, bind a trailing closure into the call frame, and route `yield`/`next` through `stmt.rs`.
Only syntactic `do...end` closures may become implicit blocks; ordinary closure
arguments continue to go through normal arity checks.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_blocks_and_yield
```

Expected after implementation: pass without `#[ignore]`.

- [x] **Step 3: Add managed-mode dynamic dispatch tables**

Change `deferred_dynamic_dispatch` into an active test that constructs a dynamic receiver and dispatches through the method table.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_dynamic_dispatch
```

Observed before implementation: failed at runtime with `struct method dispatch for 'def_method' requires Rung 4 impl resolution`.

Expected after implementation: pass without `#[ignore]` for the per-instance dynamic method-table slice. Static `impl` fallback and `method_missing` remain deferred.

- [x] **Step 3D: Add static impl fallback and method_missing**

Add an active dispatch-order test proving static inherent impl methods resolve after per-instance dynamic methods and `method_missing` handles unresolved calls.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton static_impl_dispatch_and_method_missing
```

Observed before implementation: failed at runtime with `struct method dispatch for 'label' requires Rung 4 impl resolution`.

Expected after implementation: pass with static inherent impl method registration and `method_missing` fallback in managed mode. `@dynamic impl` tables remain follow-up work.

- [x] **Step 4: Add structural protocol satisfaction and casts**

Change `deferred_structural_protocols` into a test that proves a struct satisfies a protocol by method shape and rejects a missing method.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_structural_protocols
```

Observed before implementation: failed because protocol-typed parameters were accepted without checking required methods.

Expected after implementation: pass without `#[ignore]` for protocol-typed managed parameter checks, including static inherent impl-backed satisfaction.

- [x] **Step 4E: Tighten protocol method signature compatibility**

Add negative probes to `deferred_structural_protocols` that prove name-only method presence is insufficient.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_structural_protocols
```

Observed before implementation: a method with the required name but incompatible arity was accepted.

Expected after implementation: protocol satisfaction checks method mode, receiver-adjusted arity, annotated parameter types, and required return types. Runtime `as Protocol` casts, generic protocol substitution, and built-in typed signatures remain follow-up work.

- [x] **Step 4F: Execute runtime `as Protocol` casts**

Add parser and runtime probes that prove `value as Protocol` is not parsed as a stray identifier and that cast success/failure uses the same structural protocol gate as protocol-typed parameters.

Run:

```sh
cargo test -p garnet-parser --test parse_v1_parser_parity parses_protocol_cast_expression
cargo test -p garnet-cli --test conformance_skeleton deferred_structural_protocols
```

Observed before implementation: the compatible cast path failed at runtime with `undefined variable: as`.

Expected after implementation: `Expr::Cast` parses, structurally compatible values pass through unchanged, and incompatible values fail with `does not satisfy protocol ...`. Generic protocol substitution and built-in typed signatures remain follow-up work.

- [x] **Step 4G: Substitute protocol type parameters and type core built-in signatures**

Add positive and negative probes to `deferred_structural_protocols` proving `Protocol<T>` signatures are instantiated before method compatibility checks and that core built-in methods can satisfy typed protocol signatures.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton deferred_structural_protocols
```

Observed before implementation: `BoxLike<String>` rejected a `TextBox.value() -> String` method with `does not satisfy protocol BoxLike` because the required return type remained unresolved as `T`.

Expected after implementation: generic protocol type arguments substitute into required method signatures, incompatible concrete methods still fail, and core built-in methods such as `String#len`, `String#upcase`, and `String#starts_with` satisfy compatible typed protocol signatures while rejecting incompatible return types.

- [x] **Step 4H: Register `@dynamic impl` dispatch tables**

Add a positive dispatch and protocol-satisfaction probe plus a negative probe proving ordinary trait impls remain deferred.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton dynamic_impl_dispatch_tables
```

Observed before implementation: `@dynamic impl TraitWidget for Renderable` was parsed but did not satisfy a `Renderable` parameter, failing with `Struct does not satisfy protocol Renderable: missing method render`.

Expected after implementation: `@dynamic impl` methods satisfy protocol-typed managed parameters and dispatch after per-instance dynamic methods but before static inherent impl fallback. Ordinary non-dynamic trait impl coherence remains deferred.

## Milestone 3: Actor Runtime Bridge And Sendable

**Purpose:** Make the agent-native story executable from Garnet source instead of Rust-only actor-runtime tests.

**Files:**

- Modify: `garnet-parser-v0.3/src/grammar/actors.rs`
- Modify: `garnet-check-v0.3/src/lib.rs`
- Modify: `garnet-check-v0.3/src/borrow.rs`
- Modify: `garnet-actor-runtime/src/runtime.rs`
- Modify: `garnet-interp-v0.3/src/eval.rs`
- Modify: `garnet-cli/templates/agent-orchestrator/src/main.garnet`
- Test: `garnet-cli/tests/new_cmd.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [x] **Step 1: Add a failing managed actor bridge smoke**

Extend executable example coverage so source actor/protocol syntax must survive `garnet run`, not only parsing.

Run:

```sh
cargo test -p garnet-cli --test examples multi_agent_builder_runs_with_managed_actor_bridge
cargo test -p garnet-interp c5_actor_handler_dispatches_via_spawn_bridge
```

Observed before implementation: `garnet run examples/multi_agent_builder.garnet` failed with `runtime error: undefined variable: Planner`.

Expected after implementation: actor declarations register as managed runtime actor values and `spawn Actor.handler(args)` dispatches the matching handler synchronously. Full `garnet-actor-runtime` address/mailbox bridging remains a follow-up.

- [x] **Step 2: Enforce `Sendable` at actor message boundaries**

Reject `@nonsendable` message payloads before runtime dispatch.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton
```

Expected after implementation: the actor/sendable test is active and rejects the bad case.

Observed before implementation: `garnet-check` returned no errors for an actor protocol carrying an `@nonsendable` payload type.

Expected after implementation: actor protocol and handler parameters reject `@nonsendable` named or nested payload types before runtime while ordinary sendable payload structs remain accepted. Phase 3B adds the first managed source-to-runtime actor handler dispatch; async runtime bridging remains the next step.

- [x] **Step 3: Add managed actor addresses and bounded mailbox calls**

Make source-level actor state more tangible without overstating the async runtime bridge.

Run:

```sh
cargo test -p garnet-parser parses_spawn_keyword_as_member_method_name
cargo test -p garnet-interp c5_spawn_actor_returns_address_with_persistent_state
cargo test -p garnet-interp c5_actor_address_enforces_bounded_mailbox
cargo test -p garnet-interp c5_actor_address_tell_reports_full_mailbox
cargo test -p garnet-interp c5_actor_spawn_rejects_extra_capacity_args
```

Observed before implementation: `spawn Counter` evaluated to an actor type bridge only, so `counter.tell(:incr, 1)` failed with `actor Counter has no handler 'tell'`; `Counter.spawn(1)` also failed to parse because `spawn` was reserved after `.`.

Expected after implementation: managed actor addresses preserve actor-local `let` and `memory` state, `ask` dispatches immediately, `tell` enqueues or reports a full mailbox, `try_tell` returns false on backpressure, `drain()` processes queued messages and returns the drained count, and `Actor.spawn(capacity)` enforces bounded mailbox capacity. Full `garnet-actor-runtime` OS-thread execution remains the next actor milestone because managed `Value` is still `Rc`/`RefCell`-backed rather than `Send + 'static`.

- [x] **Step 4: Make the generated agent-orchestrator template use actor syntax**

Acceptance: `garnet new --template agent-orchestrator` emits a project whose
`src/main.garnet` declares Researcher / Synthesizer / Reviewer actors, uses
`spawn Actor`, `Actor.spawn(capacity)`, `ask`, `try_tell`, `mailbox_size`, and
`drain`, and whose generated tests pass through actor-local
episodic/semantic/procedural memory.

Evidence: `cargo test -p garnet-cli --test cli_smoke new_agent_orchestrator_template_runs_and_tests`; fresh `garnet new --template agent-orchestrator` smoke with `garnet run` returning `=> 25` and `garnet test` reporting 3 passed.

## Milestone 4: Safe-Mode Ownership Hardening

**Purpose:** Move safe mode from a useful skeleton toward conservative language law.

**Files:**

- Modify: `garnet-check-v0.3/src/borrow.rs`
- Modify: `garnet-check-v0.3/src/lib.rs`
- Test: `garnet-check-v0.3/tests/borrow.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [x] **Step 1: Activate the partial B1/B2/B4 borrow-rule suite**

Replace `partial_borrow_rule_suite` with concrete CLI-level cases for direct
use-after-move through `own` parameters, direct mutable aliasing, and managed
ARC behavior that must remain outside affine checking.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Remaining: full place-granular B1-B5, method-call ownership, B3 lifetime
containment, B5 drop discipline, and two-phase borrows are still deferred.

- [x] **Step 1B: Add unambiguous method receiver ownership**

Extend the partial borrow suite so same-module methods with an unambiguous
`self` ownership contract participate in direct move and alias checks, while
same-named methods with conflicting receiver signatures remain deferred until
type resolution exists.

Run:

```sh
cargo test -p garnet-check --test borrow
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Remaining: type-resolved impl dispatch and full place-granular receiver/field
borrows are still deferred.

- [x] **Step 1C: Disambiguate simple typed method receivers**

Use simple declared receiver types from safe parameters and annotated locals to
select the matching same-module impl method before falling back to unambiguous
method names for untyped receivers.

Run:

```sh
cargo test -p garnet-check --test borrow
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Remaining: generic receiver types, trait impl dispatch, inference for untyped
locals, and full place-granular receiver/field borrows are still deferred.

- [x] **Step 1D: Track simple field places for aliasing and moves**

Teach the borrow checker to treat `root.field` chains as places for direct
ownership and alias checks. The slice rejects same-field `mut`+`borrow`
aliasing, parent/child aliasing, and same-field use-after-move while preserving
valid sibling-field use.

Run:

```sh
cargo test -p garnet-check --test borrow
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Remaining: index/dynamic places, generic receiver types, trait impl dispatch,
inference for untyped locals, drop discipline, two-phase borrows, and NLL are
still deferred.

- [x] **Step 1E: Track indexed places conservatively**

Treat `root[index]` as a wildcard index sub-place for direct ownership and
alias checks. Indexes under the same receiver now conflict, matching the
Mini-Spec prefix rule for undecidable `i = j`, while indexes under distinct
sibling fields remain distinct. Nested index receiver operands are still
evaluated so moved index expressions cannot hide inside a recognized place.

Run:

```sh
cargo test -p garnet-check --test borrow
cargo test -p garnet-cli --test conformance_skeleton partial_borrow_rule_suite
```

Remaining: dynamic places, generic receiver types, trait impl dispatch,
inference for untyped locals, drop discipline, two-phase borrows, and NLL are
still deferred.

- [x] **Step 2: Implement a conservative NLL subset**

Implement the first conservative lifetime-elision gate before full CFG region
solving. Over-reject ambiguous reference returns; do not under-reject unsafe
cases.

Run:

```sh
cargo test -p garnet-check --test extended return_ref
cargo test -p garnet-cli --test conformance_skeleton deferred_nll_lifetime_inference
```

Evidence: Phase 4F implements the conservative Mini-Spec §8.5.2 lifetime
elision subset for reference returns. A safe function returning a reference
must tie the output to exactly one borrowed input lifetime, or to borrowed
`self`; no-input and multiple-borrowed-input reference returns reject until
explicit lifetime syntax and full region solving exist.

Remaining: full CFG region solving, closure capture lifetimes, variance,
dynamic places, generic receiver types, trait impl dispatch, drop discipline,
and two-phase borrows are still deferred.

- [x] **Step 2B: Reject same-call double-own drop hazards**

Implement the first B5 drop-discipline gate by rejecting calls where overlapping
places are passed to more than one `own` parameter in the same expression. This
prevents the checker from accepting an expression that would drop the same
binding, parent/child place, or conservative index family twice.

Run:

```sh
cargo test -p garnet-check --test borrow double_own
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4G rejects `consume_pair(b, b)` and `consume_pair(p, p.left)`
while allowing distinct sibling fields such as `consume_pair(p.left, p.right)`.

Remaining: full CFG region solving, closure capture lifetimes, variance,
dynamic places, generic receiver types, trait impl dispatch, broader drop
elaboration at scope/branch boundaries, and two-phase borrows are still
deferred.

- [x] **Step 2C: Add direct-returning branch liveness**

Implement the first CFG-liveness gate by checking each `if`/`elsif`/`else`
branch against the same pre-branch snapshot and only merging moves from branch
bodies that can continue past the `if`. Preserve moves that happen while
evaluating conditions.

Run:

```sh
cargo test -p garnet-check --test borrow returning
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4H allows a value moved inside a direct-returning branch to be
borrowed on later paths that still continue, while continuing branches still
merge moved state conservatively.

Remaining: full CFG region solving, nested/non-local terminators, loops,
closure capture lifetimes, variance, dynamic places, generic receiver types,
trait impl dispatch, broader drop elaboration, and two-phase borrows are still
deferred.

## Milestone 5: Traits, Coherence, And Generic Instantiation

**Purpose:** Make the Rust-rigor side credible without claiming native zero-cost compilation.

**Files:**

- Modify: `garnet-check-v0.3/src/lib.rs`
- Create: `garnet-check-v0.3/src/coherence.rs`
- Create: `garnet-check-v0.3/src/generics.rs`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [x] **Step 1: Add a conservative orphan-rule coherence checker**

Reject conflicting trait impls and impls where neither the trait nor the type is local.

Run:

```sh
cargo test -p garnet-check --test coherence
cargo test -p garnet-cli --test conformance_skeleton deferred_trait_coherence
```

Evidence: Phase 5C rejects exact duplicate trait impls, orphan impls where
neither the trait nor the type is local, simple generic blanket-vs-concrete
overlaps, renamed generic blanket overlaps, and qualified external type
short-name collisions. It preserves the Rust-compatible positive cases where
either the trait or the type is defined locally, plus qualified local-module
type impls.

Remaining: specialization, imported-package coherence, and native
monomorphization remain deferred.

- [x] **Step 2: Add interpreter-level generic instantiation evidence**

Treat generic instantiation as runtime/interpreter evidence only. Do not claim monomorphized zero-cost behavior until a compiler backend exists.

Run:

```sh
cargo test -p garnet-cli --test conformance_skeleton generic_instantiation_runs_without_monomorphization_claims
```

Evidence: Phase 5B runs a generic `Box<T>` struct, a generic `impl<T>
Box<T>` method, and a generic `identity<T>` function through `garnet parse`,
`garnet check`, and `garnet run`, returning `=> 43`.

- [x] **Step 3: Defer native zero-cost claims until a compiler backend exists**

Native Monomorphization and the zero-cost theorem remain future work. Phase 5B
claims only interpreter-level generic instantiation evidence.

## Milestone 6: Memory Core Productization

**Purpose:** Move Mnemos from reference stores toward the Memory Core and ARC/cycle ambitions in the Mini-Spec.

**Files:**

- Modify: `garnet-memory-v0.3/src/lib.rs`
- Create: `garnet-memory-v0.3/src/cycle.rs`
- Modify: `C_Language_Specification/MEMORY_CORE_ROADMAP.md`
- Test: `garnet-cli/tests/conformance_skeleton.rs`

- [x] **Step 1: Add observable cycle fixtures**

Evidence: Phase 6A adds a memory fixture with retained roots, an unrooted
collectable cycle, an unrooted acyclic component that remains available for
ordinary eviction, a self-cycle, and a kind-scheduled cross-kind cycle.

Run:

```sh
cargo test -p garnet-memory --test cycle
cargo test -p garnet-cli --test conformance_skeleton deferred_arc_cycle_detection
```

Observed before implementation: failed because `CycleGraph`, `CycleNodeId`, and
`CycleScan` did not exist. Expected after implementation: active pass.

- [x] **Step 2: Implement bounded Bacon-Rajan-style trial deletion reference path**

Evidence: Phase 6B exposes trial candidates and scan-black retained candidates,
then runs a bounded mark-gray / scan / collect-white pass over the deterministic
cycle graph. This still does not claim the production allocator root buffer.

Run:

```sh
cargo test -p garnet-memory
cargo test -p garnet-cli --test conformance_skeleton deferred_arc_cycle_detection
```

Expected after Phase 6B: cycle fixtures pass and the conformance handle is
active with trial-candidate assertions. Expected after full implementation:
allocator-integrated ARC cycle
collection has separate positive and negative tests for the Bacon-Rajan
root-buffer algorithm.

- [x] **Step 3: Add finalization-order and safe-mode interaction reference fixtures**

Evidence: Phase 6C exposes `finalization_order` from the bounded collect-white
pass and adds `CycleAllocationMode::SafeAffine` nodes that are retained while
excluded from ARC trial candidates.

Run:

```sh
cargo test -p garnet-memory --test cycle
cargo test -p garnet-cli --test conformance_skeleton deferred_arc_cycle_detection
```

Expected after Phase 6C: cycle fixtures pass for deterministic finalization
order and safe-mode non-ARC exclusion. Expected after full implementation:
allocator-integrated finalizer invocation and safe-mode boundary checks prove
the same invariants inside the runtime allocator path.

- [x] **Step 4: Add bounded root-buffer/decrement-event reference path**

Evidence: Phase 6D adds `CycleRootBuffer` and `release_root_to_buffer`, proving
that a root decrement can enqueue a still-referenced object and that collection
can scan buffered candidates without sweeping every unrooted graph node.

Run:

```sh
cargo test -p garnet-memory --test cycle
cargo test -p garnet-cli --test conformance_skeleton deferred_arc_cycle_detection
```

Expected after Phase 6D: buffered root release passes for threshold-triggered
collection and buffered-only scans.

- [x] **Step 5: Add allocator-owned root/edge decrement fixture**

Evidence: Phase 6E adds `CycleAllocatorFixture`, proving that the allocator
surface can own the graph plus root buffer and route root releases and ARC edge
decrements through the same buffered trial-deletion scheduling path.

- [x] **Step 6: Add kind-aware allocator surface and policy-configured store eviction**

Evidence: Phase 6J adds `KindAllocator`, `HeapKindAllocator`, `AllocRequest`,
and `AllocStats`, then threads the allocator surface through working,
episodic, semantic, and procedural stores without breaking existing `new()` /
`Default` callers. Policy-configured `EpisodeStore` and `VectorIndex` now
compact lazily at read/search time using `MemoryPolicy::score` and
`should_retain`.

Run:

```sh
cargo test -p garnet-memory --test properties
cargo test -p garnet-memory
cargo test -p garnet-cli cache
```

Expected after Phase 6J: allocator stats and policy eviction tests pass while
existing Memory Core and CLI cache callers remain compatible.

- [x] **Step 7: Connect store root lifecycles to the cycle-aware allocator adapter**

Evidence: Phase 6K adds `CycleAwareKindAllocator`, object-safe root hooks on
`KindAllocator`, and `AllocRootStats`. Working, episodic, semantic, and
procedural stores now retain observable roots when values are stored and release
them on clear, policy eviction, workflow replacement, and drop. This connects
store behavior to the bounded allocator-owned cycle fixture while still keeping
production ARC finalizers out of scope.

Run:

```sh
cargo test -p garnet-memory --test properties cycle_aware
cargo test -p garnet-memory --test properties dropping_stores_releases_cycle_aware_roots
cargo test -p garnet-memory
```

Expected after Phase 6K: cycle-aware root lifecycle tests pass while existing
Memory Core callers remain compatible.

- [x] **Step 8: Add fenced episodic text snapshot persistence**

Evidence: Phase 6L adds `EpisodePersistenceError` plus
`EpisodeStore::save_text` / `load_text` as a versioned text snapshot boundary
for episodic memory. The snapshot format hex-encodes payload text, writes
through a sibling temp file before rename, rejects malformed files before
touching the existing store, and rehydrates cycle-aware roots for recovered
episodes.

Run:

```sh
cargo test -p garnet-memory --test persistence
cargo test -p garnet-memory --test properties
cargo test -p garnet-memory
```

Expected after Phase 6L: episodic recovery survives delimiter-control
payloads, malformed persistence files are loud and non-mutating, and existing
Memory Core property tests remain compatible.

- [ ] **Step 9: Promote fixture-backed roots to production ARC allocator roots**

Wire the trial-deletion pass to production ARC roots, decrement events, and
runtime finalizer invocation inside the Memory Core allocator backend instead
of only the deterministic fixture graph and cycle-aware adapter.

## Milestone 7: Release, Proof, Native Backend, And Empirical Evidence

**Purpose:** Give the too-large ambitions their own rigorous tracks so they stop being confused with current runtime truth.

**Files:**

- Modify: `F_Project_Management/GARNET_v0_4_2_RELEASE_PUBLICATION_RUNBOOK.md`
- Modify: `C_Language_Specification/GARNET_v0_4_2_Installer_Release_Contract.md`
- Create: `F_Project_Management/ROADMAPS/GARNET_NATIVE_BACKEND_PLAN.md`
- Create: `F_Project_Management/ROADMAPS/GARNET_FORMAL_PROOF_PLAN.md`
- Create: `F_Project_Management/ROADMAPS/GARNET_EMPIRICAL_VALIDATION_PLAN.md`
- Test: release workflow, installer smoke, and dogfood-readiness Part 1

- [ ] **Step 1: Publish the org release using an org-authorized browser or desktop session**

Use the already-built v0.4.2 assets. The CLI token for `Navigata1` has `push: false` on `Island-Dev-Crew/garnet`, but the current browser session can open the org release form and shows `Publish release`.

Run after publication:

```sh
gh release view v0.4.2 --repo Island-Dev-Crew/garnet --json tagName,url,assets
```

Expected: org release exists and lists tarball/package/checksum assets.

- [ ] **Step 2: Rerun a network-backed installer smoke against the org release**

Run:

```sh
GARNET_VERSION=v0.4.2 sh installer/sh.garnet-lang.org/install.sh
garnet --version
```

Expected: installer uses release assets when available and reports `garnet 0.4.2`.

- [ ] **Step 3: Create native backend, proof, and empirical plans**

Each plan must define a falsifiable first milestone:

- native backend: parse/check/run one integer arithmetic program through backend output,
- formal proof: one checked mechanized lemma for a tiny safe-mode core,
- empirical validation: one archived pilot dataset with script-reproducible metrics.

Run:

```sh
cargo test --workspace --no-fail-fast
python3 -m json.tool /tmp/dogfood-readiness-*/dogfood-readiness-data.json
```

Expected: implementation tests stay green and readiness artifacts remain parseable.

## MIT Release Gate

Garnet is ready to be spoken about as an MIT-grade prototype when these are true:

1. The current-state guide is the first reviewer path.
2. Historical claims are separated from current executable truth.
3. The 10 MVP apps parse, check, run, and are wired into CI.
4. Starter templates scaffold, test, and run frictionlessly.
5. The conformance matrix marks parser-only and deferred rows honestly.
6. The org release has assets and checksums, not just a fork release.
7. A dogfood-readiness report/deck/data bundle exists for the current commit.

Garnet is ready to be called a complete language/toolchain only when, in addition to the prototype gate, the runtime, checker, actor, memory, trait/generic, release, and empirical/proof tracks above have executable evidence.
