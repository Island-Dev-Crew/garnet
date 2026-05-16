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
| `garnet-convert/` | migration assistant | current implementation for stylized Rust/Ruby/Python/Go only; broader language and gated LLM-assist lanes are planned, not implemented |
| `scripts/garnet_converter_status.py` | converter adoption inventory | current machine-readable truth for active converter lanes, planned languages, trust boundaries, and the future Garnet-aware assist contract |
| `scripts/garnet_assist_context_pack.py` | Garnet-aware assist context and prompt pack | current machine-readable context bundle plus provider-neutral prompt pack for future provider-backed converter assist; hashes current truth/spec/dogfood docs and preserves advisory-only gates without enabling LLM conversion |
| `scripts/garnet_converter_assist_plan.py` | planned-language converter assist plan | current deterministic planner for a single source file in an active or planned converter language; inventories safe-mode, memory, CapCaps, actor/orchestration, and migration risks without executing source or enabling LLM conversion |
| `scripts/garnet_mit_readiness_status.py` | MIT/productization objective inventory | current machine-readable truth that the tracked implementation plan is complete while notarization, mobile distribution, promo video, broad converter frontends, LLM assist, and proof/empirics remain open gates |
| `scripts/garnet_promo_video_status.py` | promo video readiness contract | current machine-readable storyboard/gate contract for a future 30-second HyperFrames promo; visual identity/source surfaces and `docs/promo/` composition source are tied to repo assets, local Desktop MP4/WebM and automated visual-QA evidence can promote the lane to `visual-qa-ready`, and no website-ready export is claimed |
| `scripts/render_garnet_promo_video.mjs` | promo video render harness | current local Chrome DevTools + `ffmpeg` harness for manifest-backed MP4/WebM/poster render evidence; this is not a substitute for visual QA or website export |
| `scripts/qa_garnet_promo_video.mjs` | promo visual-QA harness | current local `ffprobe`/`ffmpeg` harness for automated metadata and sample-frame QA evidence; this is not a substitute for website export or optional human aesthetic review |
| `scripts/garnet_studio_notarization_status.py` | macOS notarization preflight inventory | current machine-readable summary for notarization preflight bundles; preserves blocker/warning evidence, redacts credential values, and explicitly avoids claiming Apple submission or notarization |
| `scripts/garnet_adoption_surface_status.py` | repo/site adoption truth inventory | current machine-readable status linking the public hook, verified use cases, active converter lanes, planned language lanes, LLM-assist boundaries, and productization gates |
| `examples/mvp_*.garnet` | canonical app-level smokes | must parse/check/run |
| `examples/{multi_agent_builder,agentic_log_analyzer,safe_io_layer}.garnet` | design-scale examples | `multi_agent_builder`, `agentic_log_analyzer`, and `safe_io_layer` are covered by active agentic matrix probes |
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
| `scripts/garnet_mit_readiness_status.py` | broader MIT/productization objective status | `python3 scripts/test_garnet_mit_readiness_status.py`; `python3 scripts/garnet_mit_readiness_status.py` |
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
4. **Native/release toolchain:** keep the org `v0.4.2` release assets,
   checksums, and release-backed installer smoke green while separating signed
   macOS `.pkg`, Windows MSI, and live public-domain installer work from the
   current release claim.
5. **Garnet Studio app and web distribution:** build polished local product
   surfaces that reduce terminal-first friction without overstating release
   authority. The macOS workbench opens like a normal app, bundles the Garnet
   CLI plus the agentic matrix, status reporters, current-truth context docs,
   examples, and docs/PWA assets, verifies mounted-DMG copy-install smoke with
   manifest-backed Desktop evidence, and exposes
   health, examples, conversion, release status, agentic stress tests, and
   onboarding. The package script can use `APPLE_DEV_ID_APP` for a
   Developer ID hardened-runtime signature when that identity exists, while a
   notarization preflight records the current Developer ID, hardened-runtime,
   Gatekeeper, and notary credential blockers. A notarization status reporter
   turns that evidence bundle into JSON/Markdown for agents, PRs, and site
   truth without submitting to Apple or exposing credential values. Packaged
   matrix runs set `PYTHONDONTWRITEBYTECODE=1` so Python status reporters do
   not mutate signed app resources after codesign, and source-workspace-only
   cargo probes are recorded as explicit packaged-resource skips instead of
   hidden app failures. The docs site now has a seed
   installable PWA shell with
   manifest, icons, service worker, local HTTP smoke, dependency-free offline
   service-worker behavior simulation, local Chrome DevTools offline smoke,
   packaged-app resource smoke, and CI evidence gate. Do not claim signing,
   notarization, clean-machine Gatekeeper, TestFlight, App Store, mobile app,
   cross-browser certification, or offline IDE completion before those lanes
   are separately verified.
6. **Repo/site adoption surface:** keep the public hook and Garnet website copy
   tied to `scripts/garnet_adoption_surface_status.py` so active converter
   lanes, planned language lanes, LLM-assist boundaries, verified use cases,
   and open productization gates remain evidence-backed instead of promotional
   drift.
7. **Promo/video readiness:** keep the future HyperFrames or Remotion ad lane
   tied to `scripts/garnet_promo_video_status.py` so the storyboard,
   visual identity/source-surface lock, render/visual-QA gates, website export,
   and forbidden claims are explicit before a video artifact is embedded or
   advertised.
8. **Agentic dogfood stress:** keep the advanced multi-domain matrix green so
   agents can exercise orchestration, agent toolbelt examples, recovery diagnostics,
   adversarial input boundaries, migration, safe-mode, release-integrity,
   signed-release provenance, macOS
   notarization readiness, docs, app, and memory-analysis surfaces as one
   falsifiable workflow, with macOS CI preserving a downloadable evidence
   artifact for readiness-sensitive PRs. The matrix now records per-domain
   coverage adequacy separately from pass/fail readiness so under-tested
   product surfaces remain visible even when every current probe passes; the
   project-scaffolding domain now exercises all three canonical templates
   (`cli`, `web-api`, and `agent-orchestrator`) through scaffold/run/test
   probes, developer experience covers doc extraction plus formatter check and
   repair behavior, agent toolbelt coverage adds five runnable examples for
   triage routing, capability budgeting, memory recall, release evidence, and
   repair planning, adversarial-boundary coverage rejects parser depth bombs,
   missing entrypoint capability declarations, and unsafe legacy mutable
   declarations in `@safe` code, agent memory/analysis covers parse-time memory declaration
   surfacing plus check/run of the advertised log analyzer, and web/PWA
   productization covers offline-handler, full local-smoke, and local Chrome
   DevTools browser offline probes. Signed-release provenance covers key
   generation, signed deterministic manifests, signature-required unsigned
   manifest rejection, and signed manifest tamper rejection without preserving
   generated private keys in Desktop evidence bundles. The macOS app workbench covers self-test,
   workbench sample smoke against the matrix-built CLI, and XCTest locally
   while richer browser IDE, cross-browser validation, and signed native
   distribution remain separate gates.
   The migration-assistant domain also carries a converter adoption-status
   probe so Rust/Ruby/Python/Go support, planned broader languages, and gated
   LLM-assist claims remain machine-readable instead of living only in
   marketing copy. The planned Garnet-aware assist contract is provider- and
   model-optional: it documents required context, safe-mode/memory/CapCaps
   analysis targets, lineage, sandbox, `garnet check`, dogfood-bundle, and
   human-audit gates without claiming broad or active LLM conversion. The
   deterministic local assist context pack now hashes the current truth, public
   README, Mini-Spec, conformance matrix, and dogfood ledger so a future
   provider-backed assist lane has a concrete context source before any model
   or network dependency is introduced. The deterministic converter assist-plan
   reporter takes an active or planned language plus one source file and emits
   advisory migration risks, required gates, next steps, JSON/Markdown, and a
   manifest while keeping planned-language conversion inactive. The promo video
   readiness contract now locks the 30-second storyboard, brand assets, and
   source surfaces before the HyperFrames/Remotion composition gate,
   rendered-artifact gate, visual-QA gate, website-export gate, and forbidden
   claims can advance toward a shipped ad.
   The MIT-readiness accounting domain also probes
   `scripts/garnet_mit_readiness_status.py`, so agents and public-site copy can
   report `87/87` tracked slices without turning that into a false claim that
   notarization, mobile distribution, promo video, broad converter frontends,
   LLM assist, proof, or empirical validation are complete.
9. **Formal/empirical proof:** keep Paper V theorem sketches and Paper VI
   experiments separate from implemented guarantees until tests or proofs land.

The v0.5 seven-phase roadmap is tracked in
`F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md` and
`F_Project_Management/ROADMAPS/GARNET_v0_5_LANGUAGE_COMPLETION_ROADMAP.md`.
As of PR #75, the tracked implementation-plan checkbox ledger is at `87/87`
slices: PR #74 closed Milestone 7 (org `v0.4.2` release published and the
org-backed installer smoke passed, taking the ledger to `86/86`), and PR #75
added the Phase 4BI nil/nil relational const-guard slice (Milestone 4 Step
2ZY), taking it to `87/87`. This closes the current readiness sprint, not the
full language: signed native installers, production allocator-integrated ARC,
full native backend, mechanized proof, and empirical validation remain
post-v0.4.2 lanes. Phases 1-3D added parser parity,
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
call-backed/dynamic string interpolation, function-call evaluation, and broader
non-numeric comparison remain deferred. Phase 4AT extends the same
equality/inequality fact domain to `nil`,
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
Phase 4AY carries the same narrow const-expression fact rules through
immutable local guard aliases, so `let always = limit + 1 == 3` can cover a
finite match arm while `let never = limit + 1 < 3` is statically
false/non-covering. Mutable local expression sources remain unknown.
Phase 4AZ resolves path-qualified top-level constants inside those immutable
local guard aliases, so `let always = Core::LIMIT + 1 == 3` can use the same
coverage facts without widening to calls or mutable sources.
Phase 4BA folds static interpolated string const facts whose interpolation
bodies already resolve through the same narrow `ConstFact` evaluator, so
`"re#{"ad"}y"` can compare as `"ready"` while call-backed/dynamic
interpolations stay unknown.
Phase 4BB folds runtime-aligned static string relational guard facts, so
`Core::LABEL < "rust"` can count as coverage while false string comparisons are
statically false/non-covering and mixed string/symbol relational comparisons
stay unknown.
Phase 4BC folds runtime-aligned static boolean relational guard facts, so
`Core::RAW < true` follows the managed runtime's `false < true` ordering while
false boolean relational comparisons become statically false/non-covering and
mixed boolean/nil relational comparisons stay unknown.
Phase 4BI folds runtime-aligned static `nil` relational guard facts, so
`Core::EMPTY <= nil` follows the managed runtime's `nil <=> nil = Equal`
ordering and counts as coverage while `Core::EMPTY < nil` is statically
false/non-covering and mixed nil/integer (and other non-runtime-comparable)
relational facts stay unknown because the runtime raises on them.
Escaped and general higher-order closure call effects plus
broader mutable closure flow remain deferred. Full CFG NLL region solving, loop
fixed-point domain inference, broader mutable/escaped/general higher-order closure invocation/call-effect analysis,
nested/non-local terminators, cross-file/package imports, recursive/open payload
reasoning, non-finite floats, call-backed/dynamic interpolated strings, broader non-boolean non-string non-numeric comparison, broader float edge-case reasoning, function-call, broader
const expression evaluation beyond immutable local aliases and path-qualified const references, broader inference, and broader non-literal guard
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
cryptographic MAC. Phase 6Q promotes one root-buffer/finalizer/safe-mode
interaction into the concrete `CycleAwareKindAllocator` surface:
allocator-owned root release can now report buffered collection candidates,
deterministic finalization order, collected nodes, and safe-affine exclusion
without callers manually owning a fixture graph. This is production-facing
allocator evidence, not production ARC completion.
Phase 6R extends this with the allocator-facing buffered edge-removal
collection path: threshold-driven `CycleAwareKindAllocator::remove_edge` now
reports trial candidates, finalization order, collected nodes, and
`root_stats` updates at the wrapper layer across all four `MemoryKind`s,
while safe-affine allocations remain excluded from ARC cycle collection. This
is sibling partial-pass evidence, not production ARC.
Phase 6S adds allocator-owned finalizer logging to the same concrete allocator:
plain `release_root`, `collect_roots`, and `remove_edge` calls now record
deterministic finalization order through a configured finalizer sink without
requiring the caller to pass a callback. This is runtime-finalizer invocation
evidence at the allocator boundary, not user-payload destructor semantics or
full production ARC.
Phase 6U adds an opt-in keyed MAC path for the typed episodic cache backend:
`append_cache_text_with_mac` and `load_cache_text_with_mac` write and verify
BLAKE3-keyed record MACs over the source-tree binding, timestamp, and encoded
payload, rejecting tampered payloads and foreign keys before live store
mutation. The legacy unsigned typed-cache API remains available for backward
compatibility, so this is signed typed-cache evidence rather than a claim that
every Memory Core persistence backend is cryptographically trusted.
Phase 6V promotes that signed-cache trust boundary into the agentic dogfood
matrix with dedicated memory-persistence integrity probes for signed round-trip,
tampered payload rejection, and foreign-key rejection, so future app/source
dogfood bundles exercise the new security surface directly.
Phase 6W extends the same matrix with an `agent adversarial boundaries` domain:
parser depth-budget bomb rejection, missing `main` `@caps` rejection, and
`@safe` legacy `var` rejection. The first strict run after adding the probes
failed `58/60` because two checker diagnostics were asserted on stderr while
the CLI contract reports them on stdout; the corrected strict source matrix
passes `60/60` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-034939`.
Phase 6X closes the packaged-app drift found by the DMG path: the failed
packaged bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-040023`
recorded `57/60` because the bundled matrix attempted source-workspace cargo
tests under `Contents/Resources/Cargo.toml`. The fixed packaged and copied-DMG
matrices accept `60/60` with `skipped=3`, preserving per-probe logs that name
the source-workspace-only boundary while the source checkout continues to run
those signed-cache probes with `skipped=0`. Current Desktop evidence:
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-040631`
(source, `60/60`, `skipped=0`),
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-040415`
(copied app, `60/60`, `skipped=3`), and
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-040414`
(DMG install smoke; DMG SHA-256
`e00d8e246fb10e339adf04c98b3f8654d841e8dd709a3ab213bd09cf7f1b34aa`).
Phase 6Y adds deterministic converter assist planning for active/planned
source languages without activating broad conversion: the new per-file reporter
hashes TypeScript or other known source, inventories migration risks, writes
JSON/Markdown plus a manifest, and is covered by three matrix probes. Current
Desktop evidence:
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-042402`
(source, `63/63`, `skipped=0`).
Phase 6Z closes the immediate packaged-app resource drift introduced by that
reporter: `Garnet Studio.app` now stages and chmods
`scripts/garnet_converter_assist_plan.py`, and the mounted-DMG smoke treats it
as a required executable bundled asset before running the copied-app matrix.
Current Desktop evidence:
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-043627`
(source, `63/63`, `skipped=0`),
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-043725`
(packaged app),
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-043735`
(copied app),
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-043735`
(DMG install smoke; DMG SHA-256
`be625f7cff7500d67223de47fdeb195e2e59876ecd2968f06976d5971ef5e8b3`).
Phase 6AA makes that packaged assist-plan capability visible in the macOS
workbench: the Converter panel now offers an `Assist Plan` action, can target
planned languages such as TypeScript/JavaScript/Swift/Java/C++/C#, and keeps
the normal `Convert` action limited to the active deterministic converter
frontends. This remains deterministic planning evidence, not provider-backed
LLM conversion or broad planned-language conversion. Current Desktop evidence:
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-045430`
(packaged app),
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-045440`
(copied app), and
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-073443`
(DMG install smoke; DMG SHA-256
`1eecabdb8357233a9e4c3a279379c1d4c39a51795809fe8862ffdc3bdce0c1e4`).
Phase 6AB adds a local Codex Run-button path for the Studio workbench:
`script/build_and_run.sh` builds the SwiftPM package, stages
`dist/Garnet Studio.app`, launches it as a real macOS app bundle, and supports
`--verify`, `--debug`, `--logs`, and `--telemetry`; `.codex/environments/environment.toml`
wires the Codex app `Run` action to that script. Verified local evidence:
`python3 scripts/test_garnet_studio_run_button.py` and
`./script/build_and_run.sh --verify`.
Phase 6AC aligns the repo/site adoption surface with that app work: the public
site now has a `Garnet Studio workbench` section for Codex Run,
`dist/Garnet Studio.app`, and planned-language `Assist Plan`, while
`scripts/garnet_adoption_surface_status.py` ties the same hook to source
evidence. This is site/current-truth UX, not a notarization, broad converter,
or provider-backed LLM conversion claim. Verified local evidence:
`python3 scripts/test_garnet_adoption_surface_status.py`,
`scripts/smoke_garnet_web_pwa.sh --copy-to-desktop --strict`,
`scripts/smoke_garnet_web_pwa_offline.mjs --docs-dir docs --output target/service-worker-offline-check-phase6ac.json`,
and a Browser smoke of `http://127.0.0.1:8765/index.html#studio`.
Phase 6AD promotes that site truth into the live Pages smoke contract:
`scripts/smoke_garnet_pages_pwa.sh` now treats missing `Garnet Studio
workbench`, Codex Run, `dist/Garnet Studio.app`, or `Assist Plan` copy as a
strict blocker, and `.github/workflows/web-pwa-readiness.yml` runs
`python3 scripts/test_smoke_garnet_pages_pwa.py` so PRs cannot weaken the live
deployment guard while still avoiding a live-domain dependency in PR CI.
Verified evidence includes `python3 scripts/test_smoke_garnet_pages_pwa.py`,
`scripts/smoke_garnet_pages_pwa.sh --copy-to-desktop --strict`, and Desktop
bundle `/Users/idc2.0/Desktop/dogfood/pages-pwa-readiness-20260516-053350`.
Phase 6AE makes the future converter-assist lane easier to hand to a model or
agent without making it active: `scripts/garnet_assist_context_pack.py` now
emits a provider-neutral prompt contract and `garnet-assist-prompt-pack.md`
alongside the hashed context pack. The prompt explicitly forbids source
execution, active-conversion claims, broad planned-language support claims, and
unchecked safe-output claims while requiring lineage, `@sandbox`, `garnet
check`, dogfood bundle, and human audit gates. The refreshed agentic dogfood
matrix passes `64/64` with `skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-054543`.
Phase 6AF broadens planned-language assist-plan dogfood without broadening
conversion: JavaScript, Swift, Java, and C++ fixtures now exercise web-agent,
Apple-app, JVM-service, and native-memory migration risks through the same
advisory reporter. Java `CompletableFuture`/executor-style orchestration terms
are recognized as actor/orchestration risks, and the source-checkout matrix
passes `68/68` with `skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-055816`.
Phase 6AG starts the promo/ad productization lane without overclaiming it:
`scripts/garnet_promo_video_status.py` defines the requested 30-second
storyboard and render/visual-QA/website gates while explicitly reporting that
no rendered promo video or website-ready export exists. Phase 6AH then locks
the visual identity and source-surface packet for the future render by hashing
canonical logo/PWA icon assets and proving the public site, Garnet Studio,
agentic dogfood matrix, and MIT readiness reporter surfaces are present. Phase
6AI adds `docs/promo/DESIGN.md` plus a HyperFrames-compatible
`docs/promo/composition.html` that registers the `garnet-promo-main` timeline
for a 30-second composition source. The promo lane now reports
`composition-ready` at 50.0%; the source-checkout matrix passes `73/73` with
`skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-065332`.
Phase 6AJ adds `scripts/render_garnet_promo_video.mjs`, a local headless
Chrome/CDP plus `ffmpeg` harness that renders the composition into
manifest-backed MP4, WebM, poster, JSON, and Markdown artifacts. Local Desktop
evidence at `/Users/idc2.0/Desktop/dogfood/garnet-promo-video` verifies
30.000-second 1920x1080 MP4/WebM renders at 12 fps, and the source-checkout
matrix adds a sixth promo-video probe and passes `74/74` with `skipped=0` in
Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-071841`.
Visual QA, website-ready export, and repo/site overclaim checks remain open
gates.
Phase 6AK adds `scripts/qa_garnet_promo_video.mjs`, an automated visual-QA
evidence harness that verifies MP4/WebM metadata, extracts three representative
frames, writes JSON/Markdown plus `MANIFEST.sha256`, and promotes the local
promo lane to `visual-qa-ready` at 80.0% only while website export remains
open. Local visual-QA evidence lives at
`/Users/idc2.0/Desktop/dogfood/garnet-promo-video-visual-qa`; the current
source-checkout matrix passes `75/75` with `skipped=0` in Desktop bundle
`/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-073242`, and
the refreshed copied-DMG evidence lives at
`/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-073443` for
DMG SHA-256
`1eecabdb8357233a9e4c3a279379c1d4c39a51795809fe8862ffdc3bdce0c1e4`. The
current MIT/productization objective reports 57.3% when that local evidence is
present.
Production allocator-integrated ARC, broad
pluggable persistence backends, and extended release-duration soak remain
follow-up work.

## Historical Material

Older milestone files are preserved because they explain how the project got
here. They are not automatically true of current `main`. Use
`F_Project_Management/GARNET_CURRENT_VS_HISTORICAL_LEDGER.md` to decide whether
a claim is current implementation truth, historical proof, roadmap, or archive.
