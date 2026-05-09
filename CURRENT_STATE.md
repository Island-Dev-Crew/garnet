# Garnet Current State and Reviewer Guide

Date: 2026-05-09
Status: research-grade language/toolchain prototype

This is the first file a fresh MIT reviewer, contributor, or agent should read
after `README.md`. It separates current executable truth from historical proof,
research corpus material, generated artifacts, and local scratch.

## Current Truth

- **Repository root:** this directory, not the older `Garnet_Final/` bundle.
- **Active implementation:** Rust workspace crates at the repository root.
- **Current language status:** research-grade prototype, not a complete
  production language.
- **Canonical language spec:** `C_Language_Specification/GARNET_v1_0_Mini_Spec.md`.
- **Current implementation-vs-spec status:**
  `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md`.
- **Current runnable app evidence:** `examples/mvp_01_*.garnet` through
  `examples/mvp_10_*.garnet`; each must parse, check, and run.
- **Current first-user templates:** `garnet new --template cli`,
  `garnet new --template web-api`, and
  `garnet new --template agent-orchestrator`; each must test and run.

## What To Verify First

Use these commands from the repository root:

```sh
cargo fmt --all -- --check
cargo test -p garnet-cli --test examples
cargo test -p garnet-cli new_cmd
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
```

For the canonical app corpus:

```sh
for file in examples/mvp_*.garnet; do
  target/debug/garnet parse "$file"
  target/debug/garnet check "$file"
  target/debug/garnet run "$file"
done
```

For starter projects:

```sh
for template in cli web-api agent-orchestrator; do
  garnet new --template "$template" "/tmp/garnet-$template"
  (cd "/tmp/garnet-$template" && garnet test && garnet run src/main.garnet)
done
```

## Source Map

| Surface | Meaning | Current status |
|---|---|---|
| `garnet-parser-v0.3/` | active parser | current implementation |
| `garnet-interp-v0.3/` | active tree-walk interpreter | current implementation |
| `garnet-check-v0.3/` | safe-mode and CapCaps validator | current implementation |
| `garnet-memory-v0.3/` | Mnemos reference memory stores | current implementation |
| `garnet-actor-runtime/` | actor runtime crate | current implementation; managed source bridge active, full OS-thread CLI bridge still staged |
| `garnet-stdlib/` | capability-tagged primitives | current implementation |
| `garnet-cli/` | user-facing CLI and templates | current implementation |
| `garnet-convert/` | migration assistant | current implementation |
| `examples/mvp_*.garnet` | canonical app-level smokes | must parse/check/run |
| `examples/{multi_agent_builder,agentic_log_analyzer,safe_io_layer}.garnet` | design-scale examples | `multi_agent_builder` is runtime proof; `agentic_log_analyzer`/`safe_io_layer` remain parser/check references |
| `A_Research_Papers/` | academic research corpus | normative/research context |
| `B_Four_Model_Consensus/` | consensus/adjudication docs | research context |
| `C_Language_Specification/` | specs, matrices, roadmaps | normative + descriptive status |
| `D_Executive_and_Presentation/` | decks and presentation artifacts | communication material |
| `F_Project_Management/` | handoffs and verification history | historical/current project management |
| `archive/` | superseded historical material | audit trail only |
| `.omx/`, `.garnet-cache/`, `target/`, `dist/` | local workflow/build output | scratch/generated, not source truth |

## Tracked Surfaces For v0.5 Roadmap

| Surface | Governs | Verification |
|---|---|---|
| `F_Project_Management/v0_5_ROADMAP_INDEX.md` | roadmap table of contents | review before new language-completion work |
| `F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md` | canonical completion ledger and milestone plan | each milestone must activate or preserve conformance gates |
| `F_Project_Management/ROADMAPS/GARNET_v0_5_LANGUAGE_COMPLETION_ROADMAP.md` | seven-phase implementation plan | `cargo test -p garnet-cli --test conformance_phase_gates` |
| `F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md` | dogfood readiness phase ledger | dogfood-readiness Part 1 after each phase |
| `garnet-cli/tests/conformance_skeleton.rs` | executable conformance handles | `cargo test -p garnet-cli --test conformance_skeleton` |
| `garnet-cli/tests/dogfood_readiness_examples.rs` | semantic MVP output stability | `cargo test -p garnet-cli --test dogfood_readiness_examples` |
| `.github/workflows/ci.yml` `canonical-examples` job | public app proof surface | GitHub Actions on PR |

## Language-Completeness Path

Garnet becomes a complete language/toolchain by turning each partial/deferred
Mini-Spec row into executable conformance tests and implementation work. The
highest-leverage next milestones are:

1. **Conformance suite:** convert the Mini-Spec matrix into test modules with
   implemented rows as passing tests and deferred rows as explicit ignored
   roadmap tests.
2. **Runtime bridge:** close parser/checker/runtime disagreements for user
   types, actor spawning, method dispatch, and richer stdlib methods.
3. **Memory Core productionization:** move from Mnemos reference stores toward
   allocator/runtime semantics described in `MEMORY_CORE_ROADMAP.md`.
4. **Native/release toolchain:** publish release assets, checksums, and platform
   install smokes before making low-friction adoption claims.
5. **Formal/empirical proof:** keep Paper V theorem sketches and Paper VI
   experiments separate from implemented guarantees until tests or proofs land.

The v0.5 seven-phase roadmap is now tracked in
`F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md` and
`F_Project_Management/ROADMAPS/GARNET_v0_5_LANGUAGE_COMPLETION_ROADMAP.md`.
Phase 4AX / 5C / 6P are the current readiness slices. Phases 1-3D added parser parity,
managed block/dynamic/protocol runtime slices, managed actor addresses, bounded
source mailboxes, and a generated actor-orchestrator template. Phase 4A
activates partial safe-mode borrow conformance for direct use-after-move
through `own` parameters and direct `mut`+`borrow` aliasing while preserving
managed ARC behavior. Phase 4B adds unambiguous same-module `own self` method
receiver move tracking and method receiver aliasing. Phase 4C uses simple
declared receiver types to distinguish same-named impl methods. Phase 4D adds
simple field-place tracking so same-field and parent/child `mut`+`borrow`
aliasing are rejected, moved fields cannot be reused, and distinct sibling
fields remain usable. Phase 4E adds conservative wildcard index-place tracking
so indexes of the same receiver conflict, nested index operand expressions stay
checked, and indexes under distinct sibling fields remain distinct.
Phase 4F activates a conservative lifetime-elision subset so reference returns
must tie to exactly one borrowed input lifetime, or to borrowed `self`, while
ambiguous multi-input and no-input reference returns are rejected. Full NLL
region solving, dynamic places, broader drop elaboration, and two-phase borrows
remain deferred. Phase 4G adds a B5 drop-discipline slice that rejects
overlapping places passed to multiple `own` parameters in the same call, while
distinct sibling fields remain usable. Phase 4H adds a first CFG-liveness slice:
safe-mode moves inside direct-returning `if`/`else` branches no longer poison
later code on paths that still continue, while moves in continuing branches
still merge conservatively. Phase 4I extends that first CFG-liveness slice so a
direct `return` terminates block scanning, and moves inside direct-returning
`while`/`loop` bodies do not poison later paths that only exist when the loop
body does not run. Full NLL region solving, nested/non-local terminators,
general loop fixed-point analysis, for-loop fixed-point liveness, dynamic
places, broader drop elaboration, and two-phase borrows remain deferred. Phase
4J adds the matching `for`-loop direct-return slice and restores any outer
binding shadowed by the loop variable after checking the loop body, so a loop
variable cannot erase a prior outer move. Phase 4K applies the same scoped
liveness discipline to `match` arm pattern bindings, so moving a pattern-local
binding does not poison an outer binding with the same name while real outer
moves in match arms still propagate. Phase 4L preserves full `match` arm
blocks in the parser, interpreter, capability walker, knowledge inventory, and
safe-mode borrow checker, so statements before an arm tail expression execute
and participate in move diagnostics instead of being discarded; guarded arms
also merge guard moves that can continue when a guard evaluates false. Phase
4M adds a scoped safe-mode match coverage pass for finite domains: `Bool` and
same-module enum subjects now reject non-exhaustive arms, duplicate covered
arms, and arms after an unguarded catch-all. Phase 4N extends that pass to
finite nested constructor payloads, so distinct nested enum payload arms are
tracked separately and payload wildcards can cover the nested finite domain.
Phase 4O resolves named, glob, module-qualified, and module-relative enum
imports for that coverage pass, so `use Types::{Status}` and `use Types::*`
can match through `Status::Ready` / `Status::Done` without treating every
short enum name as global. Phase 4P adds literal guard reasoning: `if true`
arms count as coverage and `if false` arms are rejected as statically
unreachable while staying non-covering. Phase 4Q adds open-domain literal
reachability for matches whose subject type is not finite: duplicate literal
arms and arms after `_`/catch-all patterns are rejected without claiming
open-domain exhaustiveness. Phase 4R lets immutable local boolean literals and
enum variant initializers drive the same finite-domain `match` coverage without
requiring an explicit local type annotation. Phase 4S extends that evidence to
direct `let mut` initializer and assignment flow, so finite reassignment seeds
coverage and non-finite reassignment clears inferred finite-domain state. Phase
4T adds a conservative `if`/`elsif`/`else` assignment join for that match
domain environment: only domains preserved by every possible branch survive,
and mixed finite/non-finite branch assignments clear stale finite-domain state.
Phase 4U carries that proof through nested `if`/`elsif`/`else` expressions
inside branch bodies only when every nested path definitely assigns the outer
match subject; missing nested `else` paths remain open-domain. Phase 4V makes
compound assignments an explicit invalidation boundary for finite match-domain
evidence, including direct statements and all-branch compound assignment joins,
so operator/type-dependent updates cannot preserve stale `Bool`/enum domains.
Phase 4W conservatively invalidates finite match-domain evidence after possible
`while`/`for`/`loop` body assignments, including conditional body assignments,
while preserving ordered loop-local shadowing. Phase 4X extends that conservative
boundary to `try`/`rescue`/`ensure` writes and prevents uninvoked closure literal
bodies from merging assignment domains into the surrounding flow; safe-mode
`try` rejection remains a separate diagnostic. Phase 4Y conservatively
invalidates finite domains after direct invocation of closure literals. Phase 4Z
extends that proof to directly called local closure-literal bindings. Phase
4AA carries the local closure effect through `if`/`elsif`/`else` expressions
when every branch returns a closure literal. Phase 4AB joins local
closure-effect maps after all-path branch rebinding of closure literals to an
existing binding. Phase 4AC copies known local closure effects through direct
local aliases. Phase 4AD carries known local closure effects through all-path
branch-selected direct aliases while respecting branch-local shadowing. Phase
4AE reuses that proven closure-effect extraction for directly called branch
expressions, so `(if ... { closure } else { closure })(args)` clears stale
finite match-domain evidence while shadowed unknown branch tails stay
conservative. Phase 4AF recognizes immutable local boolean guard constants, so
`let always = true` guards count as coverage and `let never = false` guards are
statically false/non-covering, while mutable guard locals remain unknown. Phase
4AG extends that guard-fact slice to same-module top-level boolean `const`
items, while function parameters with the same name shadow the const fact and
remain non-covering. Phase 4AH extends the same guard-fact evidence through
scoped named and glob imports of top-level boolean `const` items while preserving
function-parameter shadowing. Phase 4AI resolves path-qualified boolean const
guard expressions such as `Flags::ALWAYS` through the same scoped const-fact
index. Phase 4AJ resolves narrow boolean const aliases such as
`Flags::ALWAYS = Core::RAW` through that same index. Phase 4AK folds basic
boolean `not`/`and`/`or` const expressions over already-resolved boolean facts.
Phase 4AL honors decisive left operands for short-circuit `or` and `and` const
expressions without requiring the right operand to resolve.
Phase 4AM folds boolean const equality/inequality comparisons over already
resolved boolean facts.
Phase 4AN applies the same conservative boolean folding directly to match
guard expressions without requiring an alias const.
Phase 4AO folds narrow integer const equality/inequality comparisons in guard
facts. Phase 4AP folds narrow integer const relational comparisons (`<`,
`<=`, `>`, `>=`) over the same guard-fact domain. Phase 4AQ folds narrow
checked integer arithmetic (`+`, `-`, `*`, `/`, `%`, unary `-`) inside that
same guard-fact domain. Phase 4AR carries that integer fact domain through
same-module bare const identifiers and scoped named/glob imports of top-level
integer `const` items while preserving function-parameter shadowing and keeping
broader const comparison deferred. Phase 4AS extends the narrow equality and
inequality fact domain to static symbols and plain non-interpolated strings, so
`:ready` and `"ready"` const guards can be proven true or false while
interpolated strings, function-call evaluation, and broader non-numeric comparison remain
deferred. Phase 4AT extends the same equality/inequality fact domain to `nil`,
so `Core::EMPTY == nil` can count as coverage and `Core::EMPTY != nil` is
statically false/non-covering. Phase 4AU applies runtime-aligned equality
semantics across mixed known literal kinds, so `nil != false` can count as
coverage and `nil == false` is statically false/non-covering.
Phase 4AV extends the literal const-guard fact domain to finite floats and
runtime-aligned int-float equality, so `1.5 == 1.5` and `1 == 1.0` can count as
coverage while false float inequalities are statically false/non-covering and
non-finite float facts remain unknown.
Phase 4AW extends that finite numeric fact domain to runtime-aligned float and
int-float relational comparisons, so `1.5 < 2.0` and `2 <= 2.0` can count as
coverage while false finite-float relational guards are statically
false/non-covering.
Phase 4AX extends finite numeric fact evaluation through checked/runtime-aligned
float arithmetic, so `1.5 + 0.5 == 2.0` and `2 * 1.5 >= 3.0` can count as
coverage while overflow-to-infinity stays unknown.
Escaped and general higher-order closure call effects plus
broader mutable closure flow remain deferred. Full CFG NLL region solving, loop
fixed-point domain inference, broader mutable/escaped/general higher-order closure invocation/call-effect analysis,
nested/non-local terminators, cross-file/package imports, recursive/open payload
reasoning, non-finite floats, interpolated strings, broader non-numeric comparison, broader float edge-case reasoning, function-call, broader
const expression evaluation, broader inference, and broader non-literal guard
reasoning remain deferred.
Phase 5A activates
conservative trait coherence by rejecting exact duplicate trait impls and
orphan-rule violations while preserving impls where
either the trait or the type is local. Phase 5B activates interpreter-level
generic instantiation evidence for generic structs, generic impl methods, and
generic functions. Phase 5C adds a conservative generic-overlap coherence
gate for blanket-vs-concrete and renamed blanket impls, and makes qualified
paths package-aware enough that `Remote::Widget` no longer counts as local
only because a local `Widget` exists. Specialization, imported-package
coherence solving, native monomorphization, and zero-cost guarantees remain
deferred. Phase 6E
adds bounded Memory Core trial-deletion fixtures with trial candidates,
scan-black retained candidates, deterministic finalization-order reporting,
safe-mode affine allocation exclusion, root-buffer/decrement-event scheduling,
allocator-owned root/edge decrement fixtures, rooted retention, unrooted cycle
collection, unrooted acyclic retention, and kind-scheduled cross-kind
collection. Production allocator-integrated ARC and runtime finalizer
invocation remain deferred. Phase 6F adds a cache privacy
gate for compiler-as-agent episode logs: project-local absolute file paths are
recorded as stable relative labels, and external absolute paths are redacted to
`<external>/<file>` so accidental `.garnet-cache/` copies do not leak user,
temp, or CI workspace roots. Phase 6G adds CLI-level cache replay stress:
foreign machine-key episodes in the same cache and copied `.garnet-cache`
episodes are ignored and signaled as untrusted before they can surface stale
failure advice. Phase 6H wires strategy notes through provenance verification,
so copied same-machine `strategies.db` rows without local justifying episodes
are quarantined instead of applied, and adds a bounded concurrent episode-append
stress test. Phase 6I binds each episode to a keyed, non-reversible source-tree
identifier so same-machine `.garnet-cache` copies from another project root are
ignored before prior-failure or strategy advice can apply, and extends the
append stress into a 16-writer bounded soak that preserves valid NDJSON and all
verified records. Phase 6J starts Mnemos Tier 1 allocator integration: the four
Memory Core stores now expose a kind-aware allocator surface with allocation
stats, and policy-configured episodic/semantic stores perform lazy retention
eviction at read/search time. Phase 6K adds a cycle-aware allocator adapter and
has working, episodic, semantic, and procedural stores retain and release
observable roots on write, clear, policy eviction, replacement, and store drop.
Phase 6L adds a fenced episodic text snapshot path with delimiter-safe payload
encoding, malformed-file non-mutation, and cycle-aware root rehydration on
load. Phase 6M adds guarded append-style episodic text log commits: existing
logs are size-bounded and parsed as the store value type before extension,
corrupt, empty, type-invalid, or oversized logs are not carried forward,
projected oversize commits are rejected before file creation, accepted record
data is synced through a temp-file rewrite and rename, and the live store
mutates only after the on-disk commit is accepted. Phase 6N adds the default
typed episodic cache backend boundary at
`.garnet-cache/episodic/episodes.mnemos`: backend appends and loads use fixed
path components under a canonical project root, reject symlink/non-regular
targets, reject oversized loads before allocation, serialize rewrite-based
commits with an OS-backed lockfile on Unix/Windows, anchor Unix backend file
operations to the validated episodic directory handle, keep Unix cache
dirs/files private from creation time, and preserve corrupt/type-invalid
non-mutation behavior. This backend is distinct from the
CLI's signed NDJSON advisory cache and is not trusted compiler input without a
future MAC layer. Phase 6O adds a durability guardrail for the same text-commit
family: after an accepted temp-file rewrite and rename, Unix generic text
commits sync the parent directory and the typed cache backend syncs the
already-validated episodic directory handle. Non-Unix platforms keep the
existing file-sync behavior until a platform-specific directory-sync contract
is added. Phase 6P adds a dependency-free typed-cache source-tree binding:
`episodes.mnemos` records now include a non-path binding for the canonical
project root, and copied typed cache files from another root are rejected
before the live store is mutated. This is replay-hardening evidence, not a
cryptographic MAC.
Production ARC integration, runtime finalizer invocation, broad pluggable
persistence backends, and extended release-duration soak remain follow-up work.

## Historical Material

Older milestone files are preserved because they explain how the project got
here. They are not automatically true of current `main`. Use
`F_Project_Management/GARNET_CURRENT_VS_HISTORICAL_LEDGER.md` to decide whether
a claim is current implementation truth, historical proof, roadmap, or archive.
