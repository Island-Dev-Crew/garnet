# Garnet Current vs Historical Ledger

Date: 2026-05-09

This ledger prevents historical handoffs from being misread as current
implementation truth.

## Reading Rule

When documents conflict, use this order:

1. Live command output from the current checkout.
2. `CURRENT_STATE.md`.
3. `README.md`.
4. `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md`.
5. Current CI workflow definitions.
6. Historical handoffs and archived milestone files.

## Current Proof Surfaces

| Claim surface | Current source | Verification |
|---|---|---|
| Workspace builds/tests | `Cargo.toml`, CI, command output | `cargo test --workspace --no-fail-fast` |
| Canonical app examples | `examples/mvp_*.garnet` | parse/check/run loop and `garnet-cli/tests/examples.rs` |
| First-user templates | `garnet-cli/templates/*` | `garnet new`, `garnet test`, `garnet run` |
| Language completeness | conformance matrix | implemented/partial/deferred rows |
| Release install path | README, installer, release workflow | source fallback works; native release assets require release publication |

## Historical or Descriptive Surfaces

| Surface | How to interpret it |
|---|---|
| `archive/history/_CANONICAL_DELIVERABLES_INDEX.md` | historical bundle index from the research/package era |
| `archive/history/GARNET_BUILD_INSTANTIATION_BRIEF.md` | early build-orchestration plan, not current repo setup |
| `archive/history/GARNET_PHASE7_COMPLETION_REPORT.md` | parser-phase milestone report, not current language-completeness evidence |
| `archive/examples/mvp-design-drafts/` | pre-remediation MVP application drafts, not current runtime proof |
| `F_Project_Management/GARNET_v3_2_HANDOFF.md` | historical snapshot; some example/check claims are superseded by current examples README and tests |
| `F_Project_Management/GARNET_v4_0_HANDOFF.md` | historical milestone; MVP scale claims must be checked against current runnable examples |
| `F_Project_Management/GARNET_v4_2_COMPLETE_PROJECT_STATE.md` | useful consolidated history, but path and release claims require live verification |

## Current Corrections From Dogfood Part 2

- The 10 canonical MVP examples are now compact executable smokes, not the old
  large design drafts.
- The three larger real-world examples remain parser-scale design drafts.
- The `agent-orchestrator` template now uses managed actor addresses, bounded
  mailbox calls, and actor-local memory while still succeeding on the current
  interpreter.
- Safe-mode borrow checking now has an active CLI conformance slice for direct
  `own` use-after-move, direct `mut`+`borrow` aliasing, and unambiguous
  `own self` method receiver moves. It now uses simple declared receiver types
  to distinguish same-named impl methods, and it tracks simple field places for
  same-field/parent-child aliasing plus field use-after-move. It also tracks
  indexed places conservatively as wildcard sub-places, so indexes under the
  same receiver conflict while indexes under distinct sibling fields remain
  distinct, and nested index operand expressions stay checked. It now also
  enforces a conservative lifetime-elision subset for reference returns: no
  borrowed input and multiple borrowed inputs reject, while one borrowed input
  is accepted. Phase 4G adds a B5 drop-discipline slice that rejects overlapping
  places passed to multiple `own` parameters in one call, preventing same-call
  double-drop hazards for the same binding or parent/child place. Phase 4H adds
  direct-returning branch liveness: moves in branch bodies that return from the
  function no longer poison later code on paths that can still continue. Phase
  4I adds direct-return block and loop-body liveness: statements after a direct
  `return` are not borrow-checked, and moves inside direct-returning
  `while`/`loop` bodies no longer poison paths after the loop that only exist
  when the body does not run. Phase 4J adds the same direct-return liveness for
  `for` bodies and scopes the loop variable to the body so it cannot clear the
  moved state of an outer binding with the same name. Phase 4K scopes
  `match` arm pattern bindings before merging arm moves, so consuming a
  pattern-local binding no longer poisons a same-named outer binding while real
  outer moves in match arms still propagate. Phase 4L preserves full `match`
  arm blocks across the parser, interpreter, capability walker, knowledge
  inventory, and borrow checker, so statements before an arm tail expression
  execute and are checked; guard moves still merge when a guard can fail before
  a returning arm body runs. Phase 4M adds a scoped finite-domain match coverage
  pass for safe-mode `Bool` and same-module enum subjects, rejecting
  non-exhaustive matches, duplicate covered arms, and arms after unguarded
  catch-all arms. Phase 4N extends that pass to finite nested-constructor
  payloads so distinct nested enum payload cases are tracked separately and
  payload wildcards cover the nested finite domain. Phase 4O resolves named,
  glob, module-qualified, and module-relative enum imports for the coverage
  pass, letting alias prefixes such as `Status::Ready` cover the canonical
  `Types::Status::Ready` case without falling back to a global short-name
  search. Phase 4P adds literal guard reasoning so `if true` arms count as
  coverage and `if false` arms are rejected as statically unreachable while
  staying non-covering. Phase 4Q adds open-domain literal reachability so
  duplicate literal arms and arms after `_`/catch-all patterns reject even when
  the subject is not a finite `Bool` or enum domain. Phase 4R lets immutable
  local boolean literal and enum variant initializers seed that finite-domain
  match environment without explicit local type annotations. Phase 4S extends
  this to direct `let mut` assignment flow, where finite assignments seed the
  domain and non-finite assignments clear inferred finite-domain state. Phase
  4T adds conservative `if`/`elsif`/`else` assignment joins for that match
  environment, preserving a finite domain only when every possible branch
  agrees and clearing stale domains for mixed finite/non-finite branches. Phase
  4U extends that proof through nested `if` expressions inside branch bodies
  only when every nested path definitely assigns the outer subject. Phase 4V
  makes compound assignments an explicit invalidation boundary so
  operator/type-dependent updates cannot preserve stale finite domains. Phase
  4W conservatively invalidates finite domains after possible loop-body
  assignments while preserving ordered loop-local shadowing. Phase 4X adds
  `try`/`rescue`/`ensure` assignment invalidation and keeps uninvoked closure
  literal assignment bodies from leaking into the surrounding flow. Phase 4Y
  invalidates finite domains after direct closure-literal invocations. Phase 4Z
  extends that proof to directly called local closure-literal bindings. Phase
  4AA carries local closure effects through all-branch `if` expression closure
  returns. Phase 4AB joins local closure-effect maps after all-path branch
  rebinding of closure literals to an existing binding. Phase 4AC copies known
  local closure effects through direct local aliases. Phase 4AD carries known
  local closure effects through all-path branch-selected direct aliases while
  respecting branch-local shadowing. Phase 4AE reuses that proof for direct
  calls whose callee is an all-path branch-selected closure expression. Phase
  4AF recognizes immutable local boolean guard constants, so `let always =
  true` guards count as coverage and `let never = false` guards are statically
  false/non-covering, while mutable guard locals remain unknown. Phase 4AG
  extends that guard-fact evidence to same-module top-level boolean `const`
  items while preserving function-parameter shadowing. Phase 4AH extends that
  evidence through scoped named and glob imports of top-level boolean `const` facts
  while keeping parameter-shadowed imported const names conservative. Phase 4AI
  resolves path-qualified boolean const guard expressions through the same
  scoped const-fact index. Phase 4AJ resolves narrow boolean const aliases
  through that same index. Phase 4AK folds basic boolean `not`/`and`/`or`
  const expressions over already-resolved boolean facts. Phase 4AL honors
  decisive left operands for short-circuit boolean `or`/`and` const
  expressions without requiring the right operand to resolve. Phase 4AM folds
  boolean const equality/inequality comparisons over already-resolved boolean
  facts. Phase 4AN applies the same conservative boolean folding directly to
  match guard expressions without requiring an alias const. Phase 4AO folds
  narrow integer const equality/inequality comparisons in alias and direct
  match guard forms. Phase 4AP folds narrow integer const relational
  comparisons in alias and direct match guard forms. Phase 4AQ folds checked
  integer arithmetic inside alias and direct match guard forms while leaving
  non-integer/broader const comparison deferred. Phase 4AR carries those
  integer const facts through same-module bare identifiers and scoped named/glob
  imports of top-level integer `const` items while preserving parameter
  shadowing. Phase 4AS extends equality/inequality const guard facts to static
  symbols and plain non-interpolated strings while keeping call-backed/dynamic
  string interpolation and broader non-numeric comparison conservative. Phase 4AT extends the same narrow
  equality/inequality fact domain to `nil`. Phase 4AU applies runtime-aligned
  equality semantics across mixed known literal kinds for equality/inequality
  const guard facts. Phase 4AV adds finite float equality/inequality facts and
  runtime-aligned int-float equality. Phase 4AW adds finite float/int-float
  relational facts. Phase 4AX adds finite float/int-float arithmetic facts while
  keeping non-finite floats and overflow-to-infinity facts unknown. Phase 4AY
  carries narrow boolean and integer const-expression facts through immutable
  local guard aliases while keeping mutable local expression sources unknown.
  Phase 4AZ resolves path-qualified top-level const references inside those
  immutable local guard aliases while keeping calls and mutable sources
  conservative. Phase 4BA folds static interpolated string const facts whose
  interpolation bodies already resolve through the same narrow fact domain while
  keeping call-backed/dynamic interpolations unknown. Phase 4BB folds
  runtime-aligned static string relational guard facts while keeping mixed-kind
  relational facts unknown. Phase 4BC folds runtime-aligned static boolean
  relational guard facts while keeping nil/symbol and mixed-kind relational
  facts unknown.
  Full
  Rust-grade CFG NLL, nested/non-local terminators, general loop fixed-point
  analysis, loop fixed-point domain inference, broader mutable/escaped/general higher-order closure invocation/call-effect analysis, cross-file/package
  imports, non-finite floats, call-backed/dynamic interpolated strings, broader non-boolean non-string non-numeric comparison,
  function-call, and broader const expression evaluation beyond immutable local aliases and path-qualified const references, recursive/open payload reasoning, broader expression/type
  inference, open-domain exhaustiveness/range reasoning, broader non-literal guard
  reasoning, dynamic places, broader drop elaboration, and generic/trait impl
  dispatch remain roadmap work, not current truth.
- Trait coherence now has an active conservative checker slice: exact duplicate
  trait impls and orphan impls where neither trait nor type is local reject,
  simple generic blanket-vs-concrete and renamed blanket impl overlaps reject,
  and qualified external type paths no longer pass by short-name collision,
  while local-trait, local-type, and qualified local-module impls remain
  accepted. Specialization, imported-package coherence, and native
  monomorphization remain roadmap work.
- Generic instantiation now has interpreter-level evidence for generic struct
  construction, generic impl method dispatch, and generic function calls. This
  is not native monomorphization or a zero-cost backend guarantee.
- Memory Core ARC/cycle work now has Phase 6E executable reference evidence:
  `CycleGraph`, `CycleRootBuffer`, and `CycleAllocatorFixture` fixtures expose
  decrement-triggered buffered roots, allocator-owned root/edge decrement
  scheduling, threshold-driven collection, trial candidates, scan-black
  retained candidates, deterministic collect-white finalization order,
  safe-mode affine exclusion from ARC trial candidates, rooted retention,
  unrooted cycle collection, unrooted acyclic retention for ordinary
  retention/eviction, and kind-partitioned cross-kind scans. Phase 6J adds a
  kind-aware allocator surface to all four stores and makes policy-configured
  episodic/semantic stores perform lazy eviction on read/search. Phase 6K adds
  `CycleAwareKindAllocator` and verifies observable store-root retain/release
  lifecycles on write, clear, policy eviction, workflow replacement, and store
  drop. Phase 6L adds fenced `EpisodeStore` text snapshot save/load with
  delimiter-safe payload encoding, malformed-file non-mutation, and
  cycle-aware root rehydration. Phase 6M adds guarded append-style text log
  commits that size-bound and validate existing logs as the store value type
  before extension and avoid live-store mutation on corrupt, empty,
  type-invalid, or oversized persistence files. Phase 6N adds a fixed typed
  episodic cache backend at `.garnet-cache/episodic/episodes.mnemos` with
  canonical project-root pathing, symlink/non-regular rejection, pre-read size
  bounds, OS-backed lockfile serialization on Unix/Windows, private Unix permissions,
  concurrent append preservation, and cycle-aware root rehydration. Phase 6Q
  adds concrete `CycleAwareKindAllocator` root-release evidence for buffered
  trial candidates, deterministic finalization order, collected nodes, and
  safe-affine exclusion through the allocator-facing API. This is not the
  production allocator-integrated Bacon-Rajan collector, runtime finalizer
  path, broad pluggable persistence backend, or CLI signed NDJSON
  advisory-cache trust layer.
- Compiler-as-agent cache privacy now has Phase 6F executable evidence:
  absolute paths inside the active project are persisted as stable relative
  labels, while external absolute paths are redacted to `<external>/<file>`.
  Phase 6G extends this to CLI-level replay stress: same-cache foreign
  machine-key episodes and copied `.garnet-cache` episodes are ignored,
  counted, and warned as untrusted instead of surfacing stale prior-failure
  advice. Phase 6H wires CLI strategy notes through ProvenanceStrategy, so
  copied same-machine `strategies.db` rows with missing local justifying
  episodes are quarantined before they can influence diagnostics, and bounded
  concurrent episode append stress preserves all verified records. Phase 6I
  adds keyed source-tree binding, so same-machine cache copies from another
  project root are skipped before prior-failure or strategy advice can apply,
  and a 16-writer/1920-record bounded append soak preserves parseable NDJSON
  plus all verified records. Extended release-duration soak remains follow-up
  work.
- CI has an explicit canonical MVP example job in addition to the Rust test
  suite.

## Archive Boundary

`archive/` means "kept for audit trail only." Nothing under `archive/` should
be cited as current executable proof unless it is promoted back into the active
source tree with tests.

## Research Layout Note

The project already contains a proposed `research/` reorganization in
`F_Project_Management/GARNET_v4_2_GITHUB_REPO_LAYOUT.md`. That broad path move
should be done as a separate migration with link checking. This remediation pass
adds the current-state guide and historical ledger first so readers can navigate
the existing layout safely.
