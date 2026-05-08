# Garnet Current State and Reviewer Guide

Date: 2026-05-08
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
Phase 4F is the current readiness slice. Phases 1-3D added parser parity,
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
region solving, generic/trait impl dispatch, dynamic places, drop discipline,
and two-phase borrows remain deferred.

## Historical Material

Older milestone files are preserved because they explain how the project got
here. They are not automatically true of current `main`. Use
`F_Project_Management/GARNET_CURRENT_VS_HISTORICAL_LEDGER.md` to decide whether
a claim is current implementation truth, historical proof, roadmap, or archive.
