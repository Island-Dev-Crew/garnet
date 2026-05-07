# GARNET — BUILD INSTANTIATION BRIEF
**Per build-orchestration-skill methodology**

## Project lane
**Hybrid: Greenfield + Research/spec** — completing a language specification AND producing the first implementation artifacts, with doctoral-quality research deliverables as co-products.

## Execution path
**DEEP PATH** — new language system, sensitive architecture, multi-tier delivery, MIT presentation target.

## Active 52 practices
- **Spec-first**: Mini-Spec v0.3 before any new code
- **Phased delivery**: Tier 1 (critical) → Tier 2 (completeness) → Tier 3 (frontiers)
- **RPIT cycle per deliverable**: Research → Plan → Implement → Test
- **Self-verification**: every spec section cross-referenced against four-model consensus
- **Architecture docs**: compiler architecture spec as a first-class deliverable
- **Git checkpoints**: not applicable (no git repo), but file versioning via naming
- **Source of truth doc**: `_CANONICAL_DELIVERABLES_INDEX.md` updated at each gate
- **Feedback loops**: each tier gate produces lessons that inform the next

## Deferred 52 practices
- **Worktrees**: not needed — sequential spec work, not parallel code
- **Agent teams**: single agent is sufficient for spec authorship
- **Hooks/commands**: no repetitive automation needed
- **Issue-based dev**: no issue tracker, linear execution against gap analysis

## Selected supporting skills
- **Governing**: build-orchestration (this brief)
- **Specialist**: none needed — pure authorship work, no UI, no deployment

## Runtime stack
- **OMX**: active — Claude Code as direct execution engine
- **OMC**: inactive
- **OpenAgent**: inactive
- **Clawhip**: inactive
- **GitHub**: reference only (build-orchestration-skill repo)

## Verification stack
1. Every normative rule cross-checked against four-model consensus eight points
2. Every grammar production verified parseable (mental compilation against existing parser)
3. Every design decision accompanied by MIT-defensible rationale with citations
4. Tier gate review before advancing

## Done criteria
- Tier 1: Mini-Spec v0.3 + EBNF grammar + compiler architecture spec — all in Garnet_Final
- Tier 2: Module/package spec + stdlib outline + interop spec + async model — all in Garnet_Final
- Tier 3: Novel frontiers formalized — LLM-native compilation + progressive type spectrum + compiler-as-agent
- All files indexed in updated `_CANONICAL_DELIVERABLES_INDEX.md`

---

## COMPLETION STATUS (updated April 16, 2026, post-execution)

| Tier | Status | Artifacts |
|---|---|---|
| Tier 1 (MIT-critical) | ✅ COMPLETE | Mini-Spec v0.3 (+ Security Theorem, Monotonicity Theorem, Trait Coherence); EBNF grammar (90 productions); Compiler Architecture Spec (16 sections, including new §§11.2, 11.3, 14, 15) |
| Tier 2 (MIT-completeness) | ✅ COMPLETE | Tier 2 Ecosystem Specifications; Distribution & Installation Spec (NEW); Memory Manager Architecture (NEW, resolves OQ-7 + OQ-8) |
| Tier 3 (MIT-elevating / novel frontiers) | ✅ COMPLETE | Paper VI (7 contributions); Benchmarking & Evaluation Plan (NEW); Migration Guide Ruby/Python (NEW); Academic Submission Strategy (NEW) |
| Parser Rung 2.1 | ✅ SOURCE COMPLETE | `garnet-parser-v0.3/` with 17 src files (~3,237 lines), 10 test files (~141 tests), 6 example programs, README. Verification gate (`cargo build && cargo test`) pending Rust toolchain availability. |
| Canonical index | ✅ UPDATED | `_CANONICAL_DELIVERABLES_INDEX.md` enumerates all 25 canonical deliverables + 4 assets |
| Completion report | ✅ WRITTEN | `GARNET_PHASE7_COMPLETION_REPORT.md` documents the end-state |

**Total deliverables:** 25 canonical items (up from 17) + 4 architecture assets.
**Total lines written this session:** ~7,500 across documents + parser code.
**MIT defensibility:** all 11 Open Questions resolved or explicitly deferred; all 7 Paper VI contributions grounded in spec text.
**Next:** Rung 3 (managed interpreter + REPL) can commence as soon as parser verification runs on a Rust-equipped machine.
