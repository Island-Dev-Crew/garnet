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
| Rust-grade NLL and borrow rules | Partial Phase 4L | `garnet-check-v0.3/src/borrow.rs`; `garnet-check-v0.3/src/lib.rs`; `garnet-check-v0.3/tests/borrow.rs`; `garnet-check-v0.3/tests/extended.rs`; `partial_borrow_rule_suite` rejects direct use-after-move, direct mut-aliasing, `own self` method receiver moves, method receiver aliasing, simple typed receiver disambiguation, simple field-place aliasing/field use-after-move, and conservative index-place aliasing/index use-after-move while checking nested index operands; `deferred_full_borrow_rule_suite` now covers B5 same-call overlapping `own` drop discipline, direct-returning branch liveness, direct `return` block termination, direct-returning loop-body liveness, scoped `for` loop-variable liveness, scoped `match` pattern binding liveness, match guard move merging, and match-arm block statement preservation; `deferred_nll_lifetime_inference` covers conservative reference-return lifetime elision | Activate full CFG NLL, dynamic place tracking, generic/trait impl dispatch, broader drop elaboration, general loop fixed-point analysis, and two-phase borrows |
| Pattern match exhaustiveness/reachability | Partial Phase 4AL | `garnet-check-v0.3/src/match_coverage.rs`; `garnet-check-v0.3/tests/match_coverage.rs`; `deferred_match_exhaustiveness_and_reachability` rejects non-exhaustive safe-mode `Bool`, same-module enum, finite nested-constructor, and scoped named/glob/module-qualified imported enum alias matches, treats unknown guarded arms as non-covering, counts literal `if true` arms as coverage, rejects literal `if false` arms as statically unreachable, rejects duplicate finite covered arms, rejects open-domain duplicate literal arms plus arms after unguarded catch-all patterns, infers finite match domains from immutable local boolean/enum variant initializers, tracks direct mutable-local finite assignments plus non-finite assignment invalidation, joins finite match-domain evidence across conservative `if`/`elsif`/`else` assignment branches, carries nested `if` all-path assignment joins inside branch bodies, explicitly invalidates finite evidence after compound assignments, conservatively invalidates after possible loop-body assignments with ordered shadowing checks, invalidates after possible `try`/`rescue`/`ensure` writes, prevents uninvoked closure literal bodies from merging assignment domains into enclosing flow, invalidates after direct closure-literal invocations, invalidates after directly called local closure-literal bindings, invalidates after branch-joined local closure-literal binding calls, invalidates after all-path branch rebindings of local closure-literal bindings, invalidates after direct aliases of known local closure-literal bindings, invalidates after all-path branch-selected direct aliases of known local closure bindings, invalidates after direct calls to all-path branch-selected closure expressions, recognizes immutable local boolean guard constants while keeping mutable guard locals unknown, and recognizes same-module, scoped named/glob imported, and path-qualified top-level boolean `const` guard constants, narrow boolean const aliases, and basic boolean const expressions, including left-decisive short-circuit `and`/`or`, while preserving parameter shadowing | Add cross-file/package imports, recursive/open payload reasoning, arithmetic, comparison, function-call, and broader const expression evaluation, loop fixed-point domain inference, broader mutable/escaped/general higher-order closure invocation/call-effect analysis, broader expression/type inference, open-domain exhaustiveness/range reasoning, and richer non-literal guard-aware diagnostics |
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

- [x] **Step 2D: Stop borrow scans at direct returns and returning loop bodies**

Extend the first CFG-liveness gate so direct `return` terminates scanning of
the current block and loop bodies that move then immediately return do not
poison later paths that only exist when the loop body does not execute.

Run:

```sh
cargo test -p garnet-check --test borrow return
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4I routes function bodies and `while`/`loop` bodies through the
same branch-outcome helper used by Phase 4H. Unreachable statements after a
direct `return` are not borrow-checked, and values moved in a direct-returning
loop body can still be borrowed after the loop on paths where the body never
runs.

Remaining: full CFG region solving, nested/non-local terminators, general loop
fixed-point analysis, for-loop fixed-point liveness, closure capture lifetimes,
variance, dynamic places, generic receiver types, trait impl dispatch, broader
drop elaboration, and two-phase borrows are still deferred.

- [x] **Step 2E: Scope `for` loop variables and returning for bodies**

Extend the direct-return loop-body liveness gate to `for` loops and prevent a
loop variable from rebinding an outer safe-mode binding after the loop body has
been checked.

Run:

```sh
cargo test -p garnet-check --test borrow for_
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4J checks `for` bodies against a loop-local environment where
the loop variable is rebound only for the body. Direct-returning `for` bodies
do not poison later non-executed paths, and a loop variable with the same name
as a moved outer binding no longer erases that outer moved state.

Remaining: full CFG region solving, nested/non-local terminators, general loop
fixed-point analysis, closure capture lifetimes, variance, dynamic places,
generic receiver types, trait impl dispatch, broader drop elaboration, and
two-phase borrows are still deferred.

- [x] **Step 2F: Scope `match` pattern bindings before arm move merging**

Prevent moves of match-arm pattern-local bindings from poisoning same-named
outer bindings after the `match`, while preserving diagnostics for real moves
of outer bindings performed inside arms.

Run:

```sh
cargo test -p garnet-check --test borrow match_
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4K records identifiers introduced by each match arm pattern,
checks the guard/body in an arm-local environment, and restores those names
from the pre-match snapshot before merging arm moves back into the outer
environment. A moved pattern-local `item` no longer causes a later `read(item)`
of an outer binding to fail, while `_ => consume(item)` still reports
use-after-move for a real outer move.

Remaining: full CFG region solving, nested/non-local terminators, general loop
fixed-point analysis, closure capture lifetimes, variance, dynamic places,
generic receiver types, trait impl dispatch, broader drop elaboration, and
two-phase borrows are still deferred.

- [x] **Step 2G: Preserve `match` arm block statements before arm tail values**

Keep every `match` arm body as a full block in the parser and downstream
walkers instead of reducing `{ stmt* tail }` arms to only the tail expression.
This makes managed execution, capability inventory, and safe-mode borrow
checking observe statements before the arm tail.

Run:

```sh
cargo test -p garnet-parser --test parse_control_flow parses_match_arm_block_with_statements_and_tail
cargo test -p garnet-interp --test eval_control match_arm_block_preserves_statements_before_tail
cargo test -p garnet-check --test borrow match_arm_block_statement_move_still_propagates_after_match
cargo test -p garnet-cli --test conformance_skeleton deferred_full_borrow_rule_suite
```

Evidence: Phase 4L changes `MatchArm.body` to `Block`, wraps expression arms
as one-tail blocks, evaluates matched arm blocks with normal block semantics,
walks match-arm blocks for capability/safe-mode inventory, and routes
safe-mode arm bodies through branch-block checking. A `let` before the tail now
affects a matched arm result, a move statement inside a match-arm block now
propagates to later use-after-move diagnostics, and moves in guards are still
merged when a guard can fail before a returning arm body runs.

Remaining: full CFG region solving, nested/non-local terminators, general loop
fixed-point analysis, closure capture lifetimes, variance, dynamic places,
generic receiver types, trait impl dispatch, broader drop elaboration, and
two-phase borrows are still deferred.

- [x] **Step 2H: Add finite-domain match exhaustiveness and reachability**

Reject safe-mode `match` expressions over finite domains when they omit a
`Bool` case or same-module enum variant. Treat guarded arms as non-exhaustive
coverage, and reject duplicate covered arms plus arms after an unguarded
catch-all.

Run:

```sh
cargo test -p garnet-check --test match_coverage
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4M adds a dedicated `match_coverage` checker pass that uses
function parameter and local type annotations to identify `Bool` and
same-module enum subjects. It rejects a missing `false` arm, a missing enum
variant, a guarded enum arm that would otherwise hide missing coverage,
duplicate unguarded variant arms, and arms made unreachable by an unguarded
catch-all, while preserving complete enum matches.

- [x] **Step 2I: Add finite nested-constructor match coverage**

Reject safe-mode `match` expressions over finite nested constructor payloads
when they omit a nested finite case, while allowing wildcard payload patterns
to cover that nested finite domain.

Run:

```sh
cargo test -p garnet-check --test match_coverage safe_nested_enum_match
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4N enumerates finite nested constructor payload products in the
same scoped `match_coverage` pass. `Outer::Wrap(Inner::Left)` and
`Outer::Wrap(Inner::Right)` are tracked as distinct coverage cases; missing
nested payload cases are reported; and `Outer::Wrap(_)` covers the nested
finite payload domain without claiming imported enum, recursive/open payload,
or guard-proof completeness.

- [x] **Step 2J: Add imported enum alias match coverage**

Resolve named, glob, module-qualified, and module-relative enum imports for the
same scoped safe-mode match coverage pass.

Run:

```sh
cargo test -p garnet-check --test match_coverage safe_imported_
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4O adds a per-module import scope for `match_coverage`. Named
imports such as `use Types::{Status}` and glob imports such as `use Types::*`
resolve `Status` to `Types::Status`, including when `Types` is relative to the
current module, and pattern coverage accepts the source alias prefix
(`Status::Ready`) as coverage for the canonical `Types::Status::Ready` finite
case. This avoids the previous global short-name fallback and keeps ambiguous
or cross-file imports outside the hard-error gate.

- [x] **Step 2K: Add literal guard match coverage reasoning**

Treat literal `if true` and `if false` match guards as decidable in the
safe-mode match coverage pass while keeping non-literal guards conservative.

Run:

```sh
cargo test -p garnet-check --test match_coverage guard
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4P counts `Status::Ready if true` as coverage for the
`Status::Ready` finite-domain case, rejects `Status::Ready if false` as an
unreachable match arm with a statically false guard, and still reports the
false-guarded variant as missing coverage. Dynamic/non-literal guards remain
non-covering because the checker does not yet prove arbitrary guard predicates.

Remaining: cross-file/package imports, recursive/open payload reasoning,
open-domain exhaustiveness/range reasoning, and non-literal guard reasoning are still
deferred.

- [x] **Step 2L: Add open-domain literal match reachability**

Treat duplicate literal arms and arms after catch-all patterns as unreachable
in safe-mode matches even when the subject type is not a finite `Bool` or enum
domain. Keep unknown guarded literal arms non-covering so later unguarded
literal arms remain reachable.

Run:

```sh
cargo test -p garnet-check --test match_coverage safe_open_
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4Q rejects repeated open-domain literals such as two `1`
arms, rejects literal arms after `_`, and preserves conservative behavior for
`1 if ok` because a non-literal guard can fail.

Remaining: cross-file/package imports, recursive/open payload reasoning,
open-domain exhaustiveness/range reasoning, and non-literal guard reasoning are
still deferred.

- [x] **Step 2M: Infer immutable local finite match domains from initializers**

Use immutable local boolean literal initializers and enum variant
constructor/path initializers to seed the safe-mode match coverage environment
when a local binding does not carry an explicit type annotation.

Run:

```sh
cargo test -p garnet-check --test match_coverage safe_match_uses_local_
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4R rejects non-exhaustive matches over `let flag = true` and
`let status = Status::Ready()` locals, while preserving the existing explicit
type-annotation evidence.

Remaining: direct mutable-local assignment tracking is covered in Step 2N
below; cross-file/package imports, recursive/open payload reasoning, broader
expression/type inference, open-domain exhaustiveness/range reasoning, and
non-literal guard reasoning are still deferred.

- [x] **Step 2N: Track direct mutable-local match-domain assignments**

Use direct `let mut` assignment flow to seed or clear the safe-mode match
coverage environment. Finite boolean/enum assignments seed the subject domain;
non-finite assignments clear inferred finite-domain state so open-domain
matches do not receive false exhaustiveness errors.

Run:

```sh
cargo test -p garnet-check --test match_coverage mutable_
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4S rejects non-exhaustive matches after finite mutable
assignment (`flag = true`), keeps mutable enum initializers finite, and stops
reporting finite-domain missing cases after `flag = 1` invalidates the inferred
domain.

Remaining: direct `if`/`elsif`/`else` branch-merged assignment flow is covered
in Step 2O below; compound-assignment invalidation is covered in Step 2Q
below; cross-file/package
imports, recursive/open payload reasoning, broader expression/type inference,
open-domain exhaustiveness/range reasoning, and non-literal guard reasoning are
still deferred.

- [x] **Step 2O: Join branch-local match-domain assignments**

Use conservative `if`/`elsif`/`else` branch joins to carry direct mutable-local
match-domain evidence forward only when every possible branch preserves the
same finite domain. Mixed finite/non-finite branches and missing-else paths
clear inferred domains rather than reporting false finite-domain
exhaustiveness errors.

Run:

```sh
cargo test -p garnet-check --test match_coverage if_else_assignments
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4T rejects non-exhaustive matches after both branches assign a
finite Bool or enum domain, and it accepts an open-domain match after one
branch assigns a non-finite value.

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
expression call invalidation is covered in Step 2Z below; loop fixed-point and broader
mutable/escaped/general higher-order closure invocation/call-effect
flow, cross-file/package imports, recursive/open payload reasoning, broader
expression/type inference, open-domain exhaustiveness/range reasoning, and
non-literal guard reasoning are still deferred.

- [x] **Step 2P: Join nested if assignment domains inside branch bodies**

Extend the branch-join eligibility proof from only direct branch-body
assignments to nested `if` / `elsif` / `else` expressions when every nested
path definitely assigns the same outer match subject. Missing nested `else`
paths remain open-domain and branch-local bindings remain ineligible.

Run:

```sh
cargo test -p garnet-check --test match_coverage nested_if_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4U rejects a non-exhaustive match after a nested `if/else`
inside one outer branch assigns `true`/`false` on every nested path, while it
accepts the missing-nested-else variant as open-domain.

- [x] **Step 2Q: Make compound assignments an explicit invalidation boundary**

Document and test that `+=`, `-=`, `*=`, `/=`, and `%=`-style assignments do
not preserve finite match-domain evidence. Direct and all-branch compound
assignments clear stale `Bool`/enum domains before later matches because their
result depends on operator and type semantics outside the finite-domain proof.

Run:

```sh
cargo test -p garnet-check --test match_coverage compound_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4V accepts matches after direct compound assignment and after
compound assignments in every `if`/`else` branch without reporting stale
finite-domain non-exhaustiveness diagnostics.

- [x] **Step 2R: Invalidate domains after possible loop-body assignments**

Loops may execute and assign through only some iterations or nested branches.
Conservatively clear finite match-domain evidence for outer bindings assigned
inside `while`, `for`, or `loop` bodies instead of preserving stale pre-loop
domains. Loop-local bindings remain excluded so shadowing does not erase the
outer domain, while assignments before a later loop-local shadow still clear
the outer finite-domain evidence.

Run:

```sh
cargo test -p garnet-check --test match_coverage loop_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4W accepts matches after possible `while`/`for` body
assignments and after conditional assignments inside loop bodies without
reporting stale finite-domain diagnostics, while ordered shadowing tests prove
that loop-local declarations neither erase the outer domain nor hide earlier
outer assignments in the same loop body.

- [x] **Step 2S: Invalidate try-flow domains and isolate closure definitions**

`try` bodies, `rescue` handlers, and `ensure` blocks can write through paths
that safe-mode rejects separately from match coverage. Conservatively clear
outer finite match-domain evidence for possible writes in those blocks so the
checker does not report stale non-exhaustiveness after an invalidated value.
Uninvoked closure literals are also a boundary: defining a closure must not
merge its body assignments into the enclosing statement flow before any call
effect analysis exists.

Run:

```sh
cargo test -p garnet-check --test match_coverage try_
cargo test -p garnet-check --test match_coverage uninvoked_closure
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4X removes stale finite-domain diagnostics after
`try`/`rescue`/`ensure` writes while preserving the existing safe-mode
`try`/`rescue` rejection, and accepts matches after an uninvoked closure
definition whose body would otherwise assign a finite `Bool` domain.

- [x] **Step 2T: Invalidate direct closure-literal invocation domains**

Direct invocation of a closure literal runs the closure body immediately, but
the checker still does not attempt a general stored-closure call-effect model.
Conservatively clear finite match-domain evidence for outer bindings assigned
inside directly invoked closure literals, including block-body and expression
body closures, while keeping uninvoked closure definitions isolated.

Run:

```sh
cargo test -p garnet-check --test match_coverage immediate_closure
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4Y accepts matches after direct closure-literal calls whose
bodies assign the match subject without reporting stale finite-domain
non-exhaustiveness diagnostics. Broader stored closure invocation/call-effect
analysis remains deferred.

- [x] **Step 2U: Invalidate direct local closure-literal binding calls**

Local closure literals bound in the current block and called directly run their
closure body at the call site. Track the conservative set of outer bindings
that such a local closure may assign, and clear finite match-domain evidence
when the local binding is invoked. Keep branch-joined and branch-rebound closure handling covered by later steps;
escaped closure, general higher-order call-effect analysis, and broader mutable closure
flow remain deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage local_closure_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4Z accepts matches after directly calling a local closure
literal binding whose body assigns the match subject without reporting stale
finite-domain non-exhaustiveness diagnostics.

- [x] **Step 2V: Invalidate branch-joined local closure binding calls**

When an `if` / `elsif` / `else` expression assigned to a local binding returns
closure literals from every branch, conservatively join the possible outer
writes from those closure bodies. A direct call to that local binding clears
finite match-domain evidence for the joined write set. Branch rebinding is covered by the next step; escaped closures, higher-order
calls, and broader mutable closure flow remain deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage branch_joined_local_closure_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AA accepts matches after directly calling a local closure
binding produced by an all-branch `if` expression whose closure bodies can
assign the match subject, without reporting stale finite-domain diagnostics.

- [x] **Step 2W: Invalidate branch-rebound local closure binding calls**

When every branch of an `if` / `elsif` / `else` flow leaves a local closure
binding with a known closure-literal effect, conservatively join those possible
outer writes. A later direct call to that rebound local binding clears finite
match-domain evidence for the joined write set. Escaped closures, higher-order
calls, and broader mutable closure flow remain deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage branch_rebound_local_closure_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AB accepts matches after all branches rebind a local closure
binding to closure literals that can assign the match subject, without
reporting stale finite-domain diagnostics after the direct local call.

- [x] **Step 2X: Invalidate direct local closure-alias calls**

When a local binding aliases another local binding with a known closure-literal
effect, copy that conservative outer-write set to the alias. A later direct
call to the alias clears finite match-domain evidence for the copied write set.
Escaped closures, general higher-order calls, and broader mutable closure flow
remain deferred.

Run:

```sh
cargo test -p garnet-check --test match_coverage local_closure_alias_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AC accepts matches after directly calling a local alias of a
known local closure-literal binding whose body can assign the match subject,
without reporting stale finite-domain diagnostics after the alias call.

- [x] **Step 2Y: Invalidate branch-joined local closure-alias calls**

When a local binding is assigned from an all-path branch expression whose tails
are direct aliases of known local closure bindings, copy the union of those
conservative outer-write sets to the alias. A later direct call to the alias
clears finite match-domain evidence for the copied write set. Branch-local
shadowing before the tail keeps the alias unknown.

Run:

```sh
cargo test -p garnet-check --test match_coverage branch_joined_local_closure_alias_call_assignment
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AD accepts matches after directly calling a branch-selected
local alias of known local closure-literal bindings whose bodies can assign the
match subject, without reporting stale finite-domain diagnostics after the
alias call, while preserving diagnostics for shadowed unknown branch tails.

- [x] **Step 2Z: Invalidate direct branch-selected closure expression calls**

When a call callee is itself an all-path branch expression whose tails resolve
to known local closure bindings, reuse the same conservative closure-effect
extraction as local aliases. A direct call to that branch-selected expression
clears finite match-domain evidence for the joined write set, while
branch-local shadowing before a tail keeps the callee unknown.

Run:

```sh
cargo test -p garnet-check --test match_coverage direct_branch
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AE accepts matches after directly calling an all-path
branch-selected closure expression whose closure bodies can assign the match
subject, without reporting stale finite-domain diagnostics, while preserving
diagnostics for shadowed unknown branch tails.

- [x] **Step 2ZA: Recognize immutable local boolean guard constants**

Track immutable local boolean constants in a separate guard-fact map. This lets
safe-mode match coverage treat `let always = true` guards as coverage and
`let never = false` guards as statically false/non-covering without broadening
into general expression evaluation. Mutable guard locals stay unknown.

Run:

```sh
cargo test -p garnet-check --test match_coverage bool_guard
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
```

Evidence: Phase 4AF accepts matches covered by immutable local true guards,
rejects immutable local false guards as statically unreachable/non-covering,
and keeps mutable boolean guards conservative.

- [x] **Step 2ZB: Recognize same-module top-level boolean guard constants**

Seed match-guard facts from same-module top-level boolean `const` items. This
lets safe-mode match coverage treat `const ALWAYS = true` guards as coverage
and `const NEVER = false` guards as statically false/non-covering without
const aliases, arithmetic, comparison, function-call, broader const expression
evaluation, or general expression evaluation. Function parameters with the same
name shadow the module const fact.

Run:

```sh
cargo test -p garnet-check --test match_coverage const_bool_guard
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AG accepts matches covered by same-module true const guards,
rejects same-module false const guards as statically unreachable/non-covering,
and keeps parameter-shadowed const guard names conservative.

- [x] **Step 2ZC: Recognize imported top-level boolean guard constants**

Resolve scoped named and glob imports of top-level boolean `const` facts into the
match-guard fact map. This lets safe-mode match coverage treat `use
Flags::{ALWAYS}` and `use Flags::*` boolean guards the same way as local
top-level constants while preserving parameter/local/pattern shadowing.

Run:

```sh
cargo test -p garnet-check --test match_coverage imported
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AH accepts matches covered by named-imported and
module-relative imported true const guards, rejects glob-imported false const
guards as statically unreachable/non-covering, and keeps parameter-shadowed
imported const guard names conservative.

- [x] **Step 2ZD: Recognize path-qualified boolean guard constants**

Resolve path-qualified top-level boolean `const` facts in match guards through
the same scoped const-fact index. This lets safe-mode match coverage treat
`Flags::ALWAYS` guards as coverage and `Flags::NEVER` guards as statically
false/non-covering, while keeping ambiguous paths and broad const expression
evaluation conservative.

Run:

```sh
cargo test -p garnet-check --test match_coverage path_qualified
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AI accepts matches covered by path-qualified true const
guards, rejects path-qualified false const guards as statically
unreachable/non-covering, and keeps arithmetic, comparison, function-call, and
broader const expression evaluation deferred.

- [x] **Step 2ZE: Resolve narrow boolean const guard aliases**

Resolve direct boolean const aliases through the scoped const-fact index without
general const expression evaluation. This lets safe-mode match coverage treat
path-valued aliases such as `Flags::ALWAYS = Core::RAW` as coverage when they
resolve to `true` and statically false/non-covering when they resolve to
`false`.

Run:

```sh
cargo test -p garnet-check --test match_coverage const_bool_alias
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AJ accepts matches covered by path-qualified true const
aliases, rejects path-qualified false const aliases as statically
unreachable/non-covering, and leaves arithmetic, comparison, function-call, and
broader const expression evaluation deferred.

- [x] **Step 2ZF: Fold basic boolean const guard expressions**

Fold basic boolean `not`, `and`, and `or` const expressions over
already-resolved boolean facts. This lets safe-mode match coverage treat
`Core::RAW and not false` as coverage when it resolves to `true`, and
`not Core::RAW or false` as statically false/non-covering when it resolves to
`false`, without general const evaluation.

Run:

```sh
cargo test -p garnet-check --test match_coverage boolean_const_expression
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AK accepts matches covered by true boolean const
expressions, rejects false boolean const expressions as statically
unreachable/non-covering, and leaves arithmetic, comparison, function-call,
recursive, and broader const expression evaluation deferred.

- [x] **Step 2ZG: Honor short-circuit boolean const guard expressions**

Honor decisive left operands for boolean `or` and `and` const expressions
without requiring the right operand to resolve. This lets safe-mode match
coverage treat `true or Missing::VALUE` as coverage and `false and
Missing::VALUE` as statically false/non-covering while still deferring general
const evaluation.

Run:

```sh
cargo test -p garnet-check --test match_coverage short_circuit
cargo test -p garnet-cli --test conformance_skeleton deferred_match_exhaustiveness_and_reachability
cargo test -p garnet-cli --test conformance_phase_gates match_exhaustiveness_handle_documents_active_partial_scope
```

Evidence: Phase 4AL accepts matches covered by true short-circuit boolean
const expressions, rejects false short-circuit boolean const expressions as
statically unreachable/non-covering, and leaves arithmetic, comparison,
function-call, recursive, cross-file/package, and broader const expression
evaluation deferred.

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
