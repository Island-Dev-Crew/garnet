# Garnet Current State and Reviewer Guide

Date: 2026-05-06
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
| `garnet-actor-runtime/` | actor runtime crate | current implementation; CLI bridge still staged |
| `garnet-stdlib/` | capability-tagged primitives | current implementation |
| `garnet-cli/` | user-facing CLI and templates | current implementation |
| `garnet-convert/` | migration assistant | current implementation |
| `examples/mvp_*.garnet` | canonical app-level smokes | must parse/check/run |
| `examples/{multi_agent_builder,agentic_log_analyzer,safe_io_layer}.garnet` | design-scale examples | parser-scale references, not runtime proof |
| `A_Research_Papers/` | academic research corpus | normative/research context |
| `B_Four_Model_Consensus/` | consensus/adjudication docs | research context |
| `C_Language_Specification/` | specs, matrices, roadmaps | normative + descriptive status |
| `D_Executive_and_Presentation/` | decks and presentation artifacts | communication material |
| `F_Project_Management/` | handoffs and verification history | historical/current project management |
| `archive/` | superseded historical material | audit trail only |
| `.omx/`, `.garnet-cache/`, `target/`, `dist/` | local workflow/build output | scratch/generated, not source truth |

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

## Historical Material

Older milestone files are preserved because they explain how the project got
here. They are not automatically true of current `main`. Use
`F_Project_Management/GARNET_CURRENT_VS_HISTORICAL_LEDGER.md` to decide whether
a claim is current implementation truth, historical proof, roadmap, or archive.

