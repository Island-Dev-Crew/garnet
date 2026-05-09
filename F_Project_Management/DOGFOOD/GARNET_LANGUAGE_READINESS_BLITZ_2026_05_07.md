# Garnet Language Readiness Execution Blitz

Date: 2026-05-07
Branch: `codex/garnet-readiness-remediation`
Purpose: map PR #1, PR #2, and full-language readiness into an execution path
that can move Garnet from prototype/research-grade confidence toward an 85-90+
dogfood-readiness target.

## Executive Decision

PR #2 can be evaluated and merged independently of PR #1. Live git evidence
shows PR #1 is not an ancestor of PR #2, both target `main`, and their changed
file sets are separate except for CI-adjacent concerns. PR #2 is still a draft,
so it should not merge until its local and remote gates remain green after this
security/readiness addendum.

PR #1 should not merge in its original Part 1-audited form. Its concept is good,
but it needed Part 2 hardening because the first checker validated presence more
than contract substance. The Part 2 path is to land semantic checker tests and
contract-checker hardening before reconsidering merge readiness.

The complete language/toolchain remains in the low-to-mid 40s as a complete
language score until runtime/checker/memory semantics move from ignored handles
to active executable gates. PR #2 improves honesty and planning; it does not by
itself make deferred language semantics real.

## Live Evidence Consumed

| Surface | Evidence | Result |
|---|---|---|
| PR #1 metadata | `gh pr view 1 --json ...` | open, mergeable, green checks, head `feat/agent-documentation-contracts` |
| PR #2 metadata | `gh pr view 2 --json ...` | open draft, mergeable, green checks, head `codex/garnet-readiness-remediation` |
| Branch dependency | `git merge-base --is-ancestor origin/pr/1 origin/pr/2` | exit 1, PR #2 does not contain PR #1 |
| PR #2 parser parity | `cargo test -p garnet-parser --test parse_v1_parser_parity` | 4 passed |
| PR #2 conformance skeleton | `cargo test -p garnet-cli --test conformance_skeleton` | 6 passed, 8 ignored deferred semantics |
| PR #2 phase gates | `cargo test -p garnet-cli --test conformance_phase_gates` | 4 passed |
| PR #2 MVP dogfood | `cargo test -p garnet-cli --test dogfood_readiness_examples` | 1 passed |
| PR #2 examples | `cargo test -p garnet-cli --test examples` | 5 passed |
| Supply-chain security | `cargo audit`; `cargo deny --all-features check` | both exit 0; duplicate warnings only from deny |

## Readiness Ladder To 85-90+

| Order | Workstream | Why it moves the score | Required executable proof |
|---|---|---|---|
| 1 | Merge PR #2 after draft exit | establishes current-vs-deferred truth, MVP corpus, and phase gates | CI green plus local format/test/security gates |
| 2 | Harden and merge PR #1 | turns procedural memory from prose into a semantic CI gate | Python checker regressions, contract check, CI agent-contract job |
| 3 | Phase 1 finish parser parity | closes `do ... end` syntax gap without overstating runtime semantics | active parser test plus conformance matrix update |
| 4 | Phase 2 managed runtime | converts `yield`, `next`, dynamic dispatch, and structural protocols from ignored handles into running behavior | Phase 2A block/yield/next active; Phase 2B per-instance `@dynamic` method table active; Phase 2C protocol-typed managed parameter checks active; Phase 2D static inherent impl fallback and method_missing active; Phase 2E protocol method signature checks active; Phase 2F runtime `as Protocol` casts active; Phase 2G generic protocol substitution and core built-in typed signatures active; Phase 2H `@dynamic impl` dispatch tables active |
| 5 | Phase 3 actors + Sendable | makes agent-native examples executable instead of Rust-runtime-only | Phase 3D active: nonsendable actor payload rejection, managed actor handler dispatch, managed actor addresses, bounded mailbox source calls, and generated `agent-orchestrator` actor template smoke pass |
| 6 | Phase 4 safe mode | moves borrow/capability enforcement toward language law | active borrow-rule suite, NLL/lifetime negative probes, CapCaps bypass probes |
| 7 | Phase 5 traits/generics | prevents dynamic/trait claims from being parser-only | Phase 5C exact duplicate/orphan trait coherence, simple generic overlap rejection, qualified-path orphan discrimination, and interpreter-level generic instantiation active; specialization, imported-package coherence, native monomorphization, and dyn-trait check/run fixtures remain |
| 8 | Phase 6 Memory Core + cache security | raises Memory Core from reference stores toward production-grade semantics without leaving cache/privacy gaps invisible | Phase 6L episodic text snapshot save/load, malformed-file non-mutation, and cycle-aware root rehydration active; Phase 6K cycle-aware store-root retain/release lifecycles active across working, episodic, semantic, and procedural stores; Phase 6J kind-aware store allocator stats and policy-configured lazy episodic/semantic eviction active; Phase 6E bounded root-buffer trial-deletion, finalization-order, safe-mode exclusion, and allocator-owned root/edge decrement fixtures active; Phase 6F cache episode path privacy gate active; Phase 6G same-cache foreign-key and copied-cache replay warning gates active; Phase 6H copied `strategies.db` quarantine and bounded concurrent episode append stress active; Phase 6I keyed source-tree binding and 16-writer append soak active; next: production ARC allocator integration, pluggable persistence backends, and extended release-duration cache soak |
| 9 | Phase 7 release/proof/empirics | separates production packaging from academic novelty proof | signed/notarized installer smokes, datasets/scripts, proof stubs or proof repo |

## Feature Scorecards

| Feature | Current dogfood status | 85+ condition |
|---|---|---|
| Parser parity | partial positive; current parser tests pass | `do ... end` and parser-vs-spec matrix complete |
| Managed runtime | Phase 2A active for syntactic block invocation, `yield`, and `next`; Phase 2B active for per-instance `@dynamic` method tables; Phase 2C active for protocol-typed managed parameter checks; Phase 2D active for static inherent impl fallback and `method_missing`; Phase 2E active for protocol method mode, arity, annotated parameter type, and required return type checks; Phase 2F active for runtime `as Protocol` casts; Phase 2G active for generic protocol substitution and core built-in typed signatures; Phase 2H active for `@dynamic impl` dispatch tables | full §11.7/§11.8 tests active and green; ordinary trait/generic coherence still pending |
| Converter | useful scaffold; sandbox posture exists | unsafe/eval/exec corpus gate across Rust/Ruby/Python/Go and migration TODO quality checks |
| Memory Core | Phase 6L adds a fenced episodic text snapshot path with delimiter-safe payload encoding, malformed-file non-mutation, and cycle-aware root rehydration; Phase 6K extends Mnemos Tier 1 with `CycleAwareKindAllocator` and observable store-root lifecycles across working, episodic, semantic, and procedural stores; Phase 6J starts the tier with a kind-aware allocator surface across all four stores plus policy-configured lazy eviction in episodic/semantic stores; Phase 6E bounded root-buffer trial-deletion, finalization-order, safe-mode exclusion, and allocator-owned root/edge decrement fixtures active; Phase 6F prevents absolute project/external paths from being persisted in compiler-as-agent cache episodes; Phase 6G warns while ignoring foreign machine-key and copied-cache replay episodes; Phase 6H quarantines copied/stale `strategies.db` rows whose justifying episodes do not re-verify and stresses bounded concurrent episode appends; Phase 6I rejects same-machine cache copies from another source tree and adds a 16-writer bounded append soak | production ARC allocator integration, pluggable persistence/privacy backends, and extended release-duration cache soak gates active |
| Safe mode | Phase 4G active for direct use-after-move, direct mut-aliasing, `own self` method receiver moves, method receiver aliasing, simple typed receiver disambiguation, simple field-place aliasing/field use-after-move, conservative index-place checks, nested index operand checks, same-call overlapping `own` drop-discipline rejection, and conservative lifetime-elision probes; CapCaps already active | full dynamic place tracking, full CFG NLL/lifetime probes, generic/trait impl dispatch, broader drop elaboration, and CapCaps bypass negatives active |
| Actors/Sendable | Rust runtime exists; Phase 3A rejects `@nonsendable` actor protocol/handler payloads before runtime; Phase 3B runs a source actor handler through managed `spawn Actor.handler(args)` dispatch; Phase 3C creates managed actor addresses with persistent state and bounded source mailboxes; Phase 3D makes `garnet new --template agent-orchestrator` generate a runnable actor project with episodic/semantic/procedural memory and bounded mailbox tests | full async actor-runtime OS-thread bridge |
| Security posture | supply-chain gates green; trust-boundary rubric now explicit | FS/net/db/converter/release trust-boundary probes active in CI |
| Release path | fork release available; org release path evidenced | org release published by authorized session plus live installer smoke |

## Merge And Publication Guidance

1. Treat PR #2 as the base readiness remediation PR. It can merge without PR #1 after draft exit and final gates.
2. Treat PR #1 as a separate procedural-memory feature PR. Merge only after the hardened checker and tests are pushed and CI is green.
3. Do not claim complete-language readiness above the 40s until at least Phase 2 runtime semantics and Phase 4 safe-mode hardening have active passing tests.
4. Do not claim 85-90 complete-language readiness until Memory Core, actors, traits/generics, security trust boundaries, and release integrity all have executable gates.
5. Keep generated dogfood reports as artifacts, but keep durable decisions in `F_Project_Management/DOGFOOD/`, roadmap files, conformance tests, and CI.

## Immediate Next Patch Queue

| Priority | Patch | Target branch |
|---|---|---|
| P0 | PR #1 semantic contract checker regressions and CI step | `feat/agent-documentation-contracts` |
| P0 | Security dogfood rubric and phase gate | `codex/garnet-readiness-remediation` |
| Done | `do ... end` parser failing test, then implementation | PR #3 |
| Done | Convert `deferred_blocks_and_yield` into active block/yield/next runtime test plus block-vs-closure boundary regression | `codex/phase2-block-yield-runtime` |
| Done | Convert `deferred_dynamic_dispatch` into active per-instance `@dynamic` method-table runtime test | `codex/phase2-dynamic-dispatch` |
| Done | Convert `deferred_structural_protocols` into active protocol-typed parameter check | `codex/phase2-structural-protocols` |
| Done | Add static inherent impl fallback and `method_missing` runtime dispatch after per-instance dynamic methods | `codex/phase2-static-impl-dispatch` |
| Done | Tighten structural protocol compatibility beyond name-only method presence | `codex/phase2-protocol-signatures` |
| Done | Parse and execute runtime `as Protocol` casts through the structural protocol gate | `codex/phase2-protocol-casts` |
| Done | Substitute generic protocol type parameters and add typed built-in method signature tables | `codex/phase2-protocol-generics-builtins` |
| Done | Register `@dynamic impl Type for Protocol` methods for dispatch and protocol satisfaction | `codex/phase2-dynamic-impl-dispatch` |
| P1 | FS/net source-level CapCaps negative tests | follow-up from PR #2 |
| P2 | Production Memory Core ARC integration, pluggable persistence backends, and extended release-duration cache write soak tests | follow-up milestone branch |
