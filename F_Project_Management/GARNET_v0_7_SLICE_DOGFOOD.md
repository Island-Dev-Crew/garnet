# GARNET v0.7 SLICE DOGFOOD CONTRACTS

> **⚠️ SUPERSEDED (post-v0.8.1).** This document governed an earlier version's slice cycle. The current operating brief is `CLAUDE.md` + `F_Project_Management/GARNET_POST_0_8_1_SYSTEM_HANDOFF.md`; the latest tag is v0.8.1. Kept for historical context — not the current single source of truth.


Date: 2026-05-22
Purpose: Single source of truth for every v0.7 PR. Read by Claude Code,
Codex Desktop, Greptile/PR-Agent, and Jon. Update this file in the same
commit as the work it tracks.

The v0.6 successor of `F_Project_Management/GARNET_v0_6_SLICE_DOGFOOD.md`.
This file governs every PR titled `S15:`–`S19:` and any later v0.7 slice
added under § Slice Contracts.

> **S15/S16 build-both-then-compare note.** S15 and S16 were drafted as v0.6
> slices, and a Codex PR (**#221**) then merged a **hand-rolled, in-parser CST**
> (`garnet-parser-v0.3/src/cst.rs`, ~510 lines) plus ~578 lines of S16-adjacent
> LSP work. Rather than override or extend that, v0.7 takes a deliberate A/B
> path: **mac-opus builds a `garnet-cst` rowan crate independently and
> additively** (as if no CST existed, at full 1M scope), #221's in-parser CST is
> **preserved untouched as the comparison baseline**, and a **S15-Compare**
> checkpoint then extracts, diffs, and reconciles the two with Jon's fresh eyes.
> **S15-Compare on 2026-05-24 chose the rowan `garnet-cst` crate as canonical;**
> #221 is retained temporarily as a legacy migration oracle until rowan-backed
> LSP migration is green. The
> v0.6 S15/S16 sections are retained as the baseline, not deleted.

---

## Four-agent coordination

v0.7 is built by four agents in parallel. The slot → slice → writable-crate
assignment, the dependency graph, and the cross-cutting edit patterns are
defined in `F_Project_Management/AGENT_COORDINATION_LEDGER.md`. **Read that
ledger first.** Per-slice PRDs:

| Slot | Slice | PRD |
|---|---|---|
| mac-opus | S15 | `F_Project_Management/PRD_A_S15_CST_MIGRATION.md` |
| win-codex | S16 | `F_Project_Management/PRD_B_S16_LSP_PRECISION.md` |
| win-opus | S17 | `F_Project_Management/PRD_C_S17_STDLIB_LAYERS.md` |
| mac-codex | S18, S19 | `F_Project_Management/PRD_D_S18_S19_PACKAGES_LLM.md` |

**This file is section-scoped.** Each agent updates only its own slice's
contract block (`### S<N>`). Do not edit another slice's block.

---

## v0.7 thesis

v0.6 closed the load-bearing deferred lines from v0.5's scaffolds: the
interpreter consumes `.garnet/vendor/` (S12), a static registry stub
completes the `garnet add` loop (S13), and the bytecode VM lowers function
calls natively over an explicit call-frame stack (S14).

v0.7 turns the prototype into something an outside engineer can build *on
top of*. Concretely:

- The parser grows a **trivia-preserving CST** as a first-class crate so
  LSP precision, formatting, and refactoring stop being gated on the same
  missing layer (S15).
- The LSP graduates from MVP to **precision tier** — workspace symbols,
  rename, code actions, semantic tokens — all CST-aware, with S10's
  advisory rules wired into code actions (S16).
- The stdlib gets a **five-layer policy** and a compiler-enforced
  **`@stability(...)`** annotation, and grows from ~23 to ~50 primitives
  (S17).
- The ecosystem proves it works: the **first five Layer-2 packages** ship
  under `garnet-lang/*` against the layer policy (S18).
- The compiler-as-agent seam from S10 gains its **LLM-backed tier** —
  feature-flagged, non-deterministic, clearly separated from the
  deterministic rules tier (S19).

If all five land, the v0.7.0 release gate fires.

---

## Slice State Machine

Every slice moves through:

  not-started → planned → in-progress → review-ready → dogfood-passing → merged

| Transition | Required artifact |
|---|---|
| not-started → planned | Plan file at `.agent/plans/<slot>-S<N>-plan.md` referencing this contract and the slot's PRD by section |
| planned → in-progress | Draft PR open with title `S<N>: <short>`; STARTED entry appended to the ledger Status Board |
| in-progress → review-ready | CI green · PR body uses the dogfood-readiness headings · dogfood block run locally with output committed |
| review-ready → dogfood-passing | Jon reviewed; PR-Agent Grep Loop ≥ 4/5 |
| dogfood-passing → merged | Squash-merged · CHANGELOG.md updated · status reporter output committed if % moved · readiness baseline regenerated if a lane was added · ledger entry → MERGED |

Backward moves are allowed and require a one-line "regression note" in the PR body.

---

## Common Verification Primitives

Every slice's CI run executes these on top of its own block. These gates
were stood up in v0.5 and inherited unchanged:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
cargo deny check
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/garnet_conformance_matrix_check.py
python3 -m unittest discover scripts/ -p 'test_*.py'
```

---

## Cross-Slice Gates (every PR)

| Gate | Where enforced | Failure mode |
|---|---|---|
| `@caps` declared on new authority | `garnet check` in CI | Hard fail |
| Determinism preserved | S9 cross-machine matrix | Hard fail |
| Determinism job never spawned with `--llm` (v0.7+) | S19 CI guard | Hard fail |
| No new ambient `unsafe` | `cargo clippy` + audit | Hard fail |
| Honest voice in docs | Jon review | Block until corrected |
| Dogfood-readiness headings present | `.github/workflows/dogfood-readiness.yml` greps PR body | Hard fail |
| `cargo deny check` clean | CI | Hard fail |
| Node-24 action minimums | `scripts/test_github_actions_node24_readiness.py` | Hard fail |
| `rustfmt` clean per package | `cargo fmt --all -- --check` | Hard fail |
| MIT-readiness baseline regenerated if a lane was added | S0 `--check-no-regression` | Hard fail |
| Bytecode ABI stability respected (v0.6+) | S14 ABI test | Block on review |
| Crate ownership respected (no writes outside owned crates) | Ledger + Jon review | Roll back violating change; file Handoff Request |

---

## Slice Contracts

### S15 — Trivia-preserving CST migration (`garnet-cst`)

**Slot:** mac-opus · **PRD:** `PRD_A_S15_CST_MIGRATION.md` · **PR count:** 2 (trait stub + substantive)

**Goal:** Migrate Garnet's parse output from AST-only to a trivia-preserving
CST. Ship a **new `garnet-cst` crate** (rowan-backed) and add an opt-in CST
mode to the user-facing `garnet parse` command. The AST stays the semantic
reference; existing consumers (interp, check, vm) keep working unchanged via a
`cst_to_ast()` projection.

**Build-both-then-compare (READ FIRST).** #221 already merged a hand-rolled
in-parser CST at `garnet-parser-v0.3/src/cst.rs`. Build this rowan crate
**independently and additively** — cold from the Mini-Spec, as if no CST
existed. *(Superseded at RB-4a, 2026-06-12: the oracle was retired once
rowan-backed LSP coverage was green.)* **Do not modify or delete `garnet-parser-v0.3/src/cst.rs`** (#221) — it
is preserved as the **S15-Compare** baseline. Reconciliation is the S15-Compare
checkpoint below, not part of S15. See `PRD_A_S15_CST_MIGRATION.md`.

**Owned crates (writable):** `garnet-parser-v0.3` (CST mode opt-in only; do NOT
touch `src/cst.rs`), `garnet-cst` (NEW).

**New surfaces:**
- `garnet-cst/` — NEW crate: `SyntaxKind`, `GarnetLanguage: rowan::Language`,
  `SyntaxNode`/`SyntaxToken`, the `CstNode` trait, `parse_cst()`, `Parse<T>`,
  and the `cst_to_ast()` projection.
- `garnet-cli` — `garnet parse --mode cst <file>` flag (default remains AST);
  routes to the canonical rowan `garnet-cst` parser after S15-Compare.
- `garnet-cst/tests/` — roundtrip property test: for any UTF-8 input that
  parses without errors, `cst_to_source(parse_cst(input)) == input`
  (proptest, ≥ 1000 iterations in CI nightly).
- `garnet-cst/benches/parse_cst_vs_ast.rs` — Criterion bench.
- `garnet-cst/AGENTS.md` — trait surface, converter, stability tier.
- New lane in `scripts/garnet_mit_readiness_status.py`: `parser_cst_migration`.
- New crate appended to workspace `Cargo.toml` `members` (alphabetical).
- Regenerated baseline.

**Trait Publication Protocol (two PRs):**
1. **PR-1 — `S15: garnet-cst trait surface + stub`** (small, fast; target
   merge within 24h of open). Stub `parse_cst` returns a single CST node
   containing all source as trivia. The trait is real; the impl is trivial.
   (Historical note: S16 did not unblock on PR-1; it waited for S15-Compare to
   pick the canonical CST. PR-1 still published the rowan trait early.)
2. **PR-2 — `S15: trivia-preserving CST via rowan`** (substantive).

**Deps added:** `rowan` (must pass `cargo deny`).

**Dogfood block:**

```bash
cargo build -p garnet-cst --release
cargo test -p garnet-cst -p garnet-parser --no-fail-fast   # package name is `garnet-parser` (dir garnet-parser-v0.3)
cargo bench -p garnet-cst --bench parse_cst_vs_ast
cargo test --workspace --no-fail-fast   # existing consumers still pass
# Expect: roundtrip property test 1000/1000 clean; CST path ≤ 1.5× AST path
# (numbers committed); no regression for interp/check/vm.
```

**Honest partial labels available:**
- "S15 ships the CST representation; downstream migration to CST-first is v0.8 work."
- "Downstream consumers (interp, check, vm) still operate on the AST projection via `cst_to_ast()`."
- "CST parsing is approximately N× slower than AST parsing (committed bench numbers); optimization deferred to v0.8."
- "Roundtrip is source-preserving for canonical examples; recovery from malformed input is best-effort and may diverge."

**State:** merged — PR-1 (trait surface + stub) merged (#225); PR-2 (substantive rowan builder + `cst_to_ast` + bench + `parser_cst_migration` lane) merged (#226). Round-trip 100% on the corpus + proptest; `cst_to_ast` span-normalized structural parity vs `parse_source` on the corpus; bench ≈0.99× AST. S15-Compare recorded the canonical-CST choice: rowan `garnet-cst`.

---

### S15-Compare — CST reconciliation checkpoint (human-in-the-loop)

**Owner:** Jon (with Claude assist) · **Type:** review checkpoint, not autonomous agent work · **Trigger:** after S15 (rowan `garnet-cst`) reaches dogfood-passing.

**Goal:** With two independent trivia-preserving CSTs in the tree — #221's
hand-rolled in-parser CST (`garnet-parser-v0.3/src/cst.rs`) and S15's rowan
`garnet-cst` crate — extract both, compare them side by side with fresh eyes,
and decide how they reconcile.

**Procedure:**
1. **Extract** — surface both implementations and their tests/benches into a
   single comparison view (e.g.,
   `F_Project_Management/DOGFOOD/S15_CST_COMPARE.md`).
2. **Diff on substance**, not line count: coverage of
   `SyntaxKind`/`CstNodeKind` variants, trivia fidelity, roundtrip guarantees,
   error-recovery behavior, performance vs the AST path, API ergonomics for the
   LSP consumer, and test depth.
3. **List reconcilable points** — where the two agree, where each is stronger,
   and what a merged design would keep from each.
4. **Decide** (Jon) — keep the rowan crate, keep the in-parser CST, or merge the
   best of both into the canonical CST. Record the decision + rationale in the
   ledger so S16 and downstream know the target.

**Dogfood block:**

```bash
# Both implementations build and pass their own tests before comparison:
cargo test -p garnet-cst --no-fail-fast
cargo test -p garnet-parser --test cst_round_trip --no-fail-fast
# Comparison artifact exists and the canonical-CST decision is recorded:
test -f F_Project_Management/DOGFOOD/S15_CST_COMPARE.md
```

**Honest partial labels available:**
- "Two CST implementations coexisted during S15-Compare by design; after the
  decision, rowan is canonical and #221 is a temporary legacy oracle."
- "Reconciliation is a human review checkpoint; no automated merge of the two CSTs is claimed."

**Unblocks:** S16 (LSP precision targets the canonical CST chosen here).

**State:** complete — comparison artifact:
`F_Project_Management/DOGFOOD/S15_CST_COMPARE.md`. Decision: rowan
`garnet-cst` is canonical for v0.7/S16. #221's in-parser CST remains a
temporary legacy oracle until S16 migrates LSP behavior to rowan and passes.

---

### S16 — LSP precision features

**Slot:** win-codex · **PRD:** `PRD_B_S16_LSP_PRECISION.md` · **PR count:** 1 (substantive, after mock-first prep)

**Goal:** Upgrade the LSP from MVP (diagnostics, hover, go-to-def) to
precision tier: **workspace symbols, rename, code actions, semantic
tokens** — all CST-aware, not regex-driven. Wire S10's advisory rules into
code actions.

> **UNBLOCKED after S15-Compare.** #221 already merged ~578 lines of LSP work in
> `garnet-lsp/src/lib.rs`. S16 now targets the rowan `garnet-cst` crate chosen
> by S15-Compare, using `tokens.rs` helpers to preserve #221's token/span
> ergonomics. Do not delete #221's parser CST or LSP work until rowan-backed
> rename/semantic-token coverage is green; it remains the migration oracle.

**Owned crates (writable):** `garnet-lsp`, `editors/vscode`.

**New surfaces:**
- `garnet-lsp/src/mock_cst.rs` — mock impl of the `CstNode` trait for
  mock-first development while S15 PR-1 is in flight.
- Feature modules for `documentSymbol`, `rename`, `codeAction`,
  `semanticTokens/full` — all CST-driven, trivia-preserving at edit sites.
- Three code actions, each reusing `garnet-check-v0.3::suggest` (do NOT
  redefine the rules): **AddCapsAnnotation** (`suggest::ManagedFnMissingCaps`),
  **RefactorLongParameterList** (`suggest::LongParameterList`),
  **AddExplicitReturnType**.
- `editors/vscode/` — declare the four capabilities; add three commands;
  bump version; repackage `.vsix`.
- `scripts/smoke_garnet_lsp_precision.py` — spawns `garnet-lsp`, exercises
  all four surfaces; runs in the workspace test gate.
- `editor_lsp_precision` lane in `scripts/garnet_mit_readiness_status.py`.
- Regenerated baseline.

**Deps:** satisfied. S15 PR-1 and PR-2 are merged, and S15-Compare selected the
rowan `garnet-cst` crate as canonical. S16 now migrates LSP behavior to rowan
while preserving #221's parser CST as a temporary legacy oracle until this lane
is merged.

**Dogfood block:**

```bash
cargo build -p garnet-lsp --release
cargo test -p garnet-lsp --no-fail-fast
(cd editors/vscode && npm install && npm run package)
python3 scripts/smoke_garnet_lsp_precision.py
# Manual confirmation (screenshots in PR body): workspace symbols populated;
# rename updates references across files; "Add @caps" action appears on a def
# missing it; capabilities highlighted as @attribute color.
```

**Honest partial labels available:**
- "S16 ships LSP precision for managed mode; safe mode (`@safe fn`) precision is v0.8."
- "Cross-workspace rename works within a single folder; cross-package rename is v0.8."
- "Three code actions ship in v0.7; the long-tail (add-suggested-tests, extract-fn, inline-let) is on the v0.8 roadmap."
- "Semantic tokens use a static classification scheme; per-project token themes deferred."

**State:** review-ready locally — branch
`agent-win-codex/s16-rowan-lsp-precision` ports `garnet-lsp` rename and
semantic-token spans to canonical rowan `garnet-cst`, preserves parser/check
diagnostics, ships three code actions, bumps the VS Code extension to `0.7.0`,
adds `scripts/smoke_garnet_lsp_precision.py`, and adds the
`editor_lsp_precision` readiness lane. Local dogfood passes:
`cargo build -p garnet-lsp --release`, `cargo test -p garnet-lsp
--no-fail-fast`, `(cd editors/vscode && npm install && npm run package)`, and
`python3 scripts/smoke_garnet_lsp_precision.py`. PR/CI/Jon review are still
pending; do not call S16 merged until those complete.

---

### S17 — Stdlib expansion + layer policy + `@stability`

**Slot:** win-opus · **PRD:** `PRD_C_S17_STDLIB_LAYERS.md` · **PR count:** 1 (or 2 if spec doc + impl split)

**Goal:** Codify Garnet's **five-layer stdlib model**, add a first-class
compiler-enforced **`@stability(...)`** annotation, and expand the prelude +
`std::` from ~23 to ~50 primitives.

**Owned crates (writable):** `garnet-stdlib`, `garnet-check-v0.3` (`@stability`
enforcement surface + new capability set entries only).

**New surfaces:**
- `C_Language_Specification/GARNET_STDLIB_LAYER_POLICY.md` — formal doc:
  five-layer model, promotion criteria, deprecation policy, stability
  semantics table, and the first-order "capability surface + spec
  volatility = layer assignment" principle.
- `@stability(...)` enforcement in `garnet-check-v0.3` — **warning-level**,
  not error-level, for backwards compat. Missing annotation → warning;
  caller of `experimental` without `@uses(experimental)` → warning;
  `deprecated` → warning with migration hint; `frozen` → info.
- Explicit `@stability(...)` on every existing primitive (stable for 2+
  minor releases; experimental for current-release additions).
- ~27 new primitives across `core::iter`, `core::result`, `core::option`,
  `core::cmp`, `core::math` (Layer 0, no caps) and `std::env`,
  `std::process`, `std::json`, `std::regex`, `std::uuid`, `std::base64`,
  `std::log` (Layer 1) — all shipping `@stability(experimental)`.
- New capabilities `@caps(env)` and `@caps(proc)` in the central capability set.
- `scripts/garnet_stdlib_layer_gate.py` — primitives by layer, % with
  explicit `@stability`, deprecated primitives + removal target; hooked into
  `garnet_mit_readiness_status.py` as the `stdlib_layer_policy` lane.
- Regenerated baseline.

**Deps:** None upstream — can start immediately. **Possible Handoff** to
mac-opus (parser owner) if `@stability(...)` needs attribute syntax the
parser doesn't already handle.

**Dogfood block:**

> Note: the checker's **cargo package name is `garnet-check`** (its directory is
> `garnet-check-v0.3/`). Earlier drafts wrote `-p garnet-check-v0.3`, which cargo
> rejects as an invalid package id; the correct flag is `-p garnet-check`.

```bash
cargo build -p garnet-stdlib -p garnet-check --release
cargo test  -p garnet-stdlib -p garnet-check --no-fail-fast
python3 scripts/garnet_stdlib_layer_gate.py
# Expect: ≥ 50 primitives total (live: 77); ≥ 95% with explicit @stability
# (live: 100%); GARNET_STDLIB_LAYER_POLICY.md exists. Exit 0 on pass.
python3 -m unittest scripts.test_garnet_stdlib_layer_gate   # gate parser/aggregation tests
garnet check examples/mvp_01_*.garnet   # no NEW diagnostics on existing examples
```

**Honest partial labels available:**
- "Stdlib expanded from 23 to ~50 primitives in v0.7; many are `@stability(experimental)` and may evolve in v0.8."
- "`@stability` enforcement is at warning level, not error level, for backwards compat. Error-level enforcement is v0.8 work."
- "Layer 2 packages (LLM client, HTTP client, etc.) are NOT bundled with v0.7 binaries; they ship via the registry as `@garnet-lang/*` packages."

**Delivered (v0.7, honest scope):**
- ✅ Layer Policy doc, registry expansion **24 → 77 primitives** (40 Layer-0
  `core`, 37 Layer-1 `std`), 100% explicit `@stability`, `garnet_stdlib_layer_gate.py`
  + `stdlib_layer_policy` lane (79.6% → 80.4% on the merged tree), `@caps(env)` known cap, and a
  registry-driven `@stability` advisory at **primitive** call sites
  (`garnet-check/src/stability.rs`, non-fatal).
- ⚠️ **Pending parser handoff (mac-opus):** source-level `@stability(...)` /
  `@uses(experimental)` / `@migration(...)` on **user-defined** functions — the
  annotation parser rejects unknown names today, so the "missing annotation →
  warning" and "`@uses` opt-in" bullets above apply to primitives now and to
  user functions after the handoff lands. `@caps(proc)` already existed; S17
  adds only `@caps(env)`.
- ⚠️ **v0.8:** Garnet-source **execution** of the new Layer-0/1 primitives
  (registry surface + Rust host impls + unit tests ship now; interpreter
  dispatch is out of S17's crate ownership).

**State:** dogfood-passing (local: 138 stdlib + 137 checker tests; layer gate 77 prims / 100% `@stability`; workspace fmt/clippy/test green; `garnet check` on the example corpus shows no new diagnostics). Pre-existing Windows-only `scripts/` test failures (`shasum`/cp1252/exec-bit/SwiftPM) are unrelated to S17; CI (Linux) is the cross-cutting authority.

---

### S18 — First five Layer-2 packages (`garnet-lang/*`)

**Slot:** mac-codex · **PRD:** `PRD_D_S18_S19_PACKAGES_LLM.md` · **PR count:** in-repo template + example + registry entries; five external repo creations

**Goal:** Publish the first five Layer-2 packages under
`github.com/garnet-lang/` — the registry-seed content that proves the
ecosystem works end-to-end: `http-client`, `llm`, `cli`, `test-property`,
`log`. Each ships a working `v0.1.0` at `@stability(experimental)`.

**Owned (writable):** `tools/garnet-lang-template/` (in main repo); external
`github.com/garnet-lang/*` repos.

**New surfaces:**
- `tools/garnet-lang-template/` — scaffold: README (calibrated-honesty
  voice), dual MIT/Apache-2.0 license, CHANGELOG with `[Unreleased]`,
  `Garnet.toml` with `@stability` tier, `garnet/lib.garnet`, `tests/smoke.garnet`.
- `examples/garnet_lang_registry_seed/` — local filesystem-registry source
  proof for the first five packages at v0.1.0. This is the reproducible S13
  registry stub path; it is **not** external GitHub publication.
- Five external repos, each created independently against the layer policy
  S17 produces. **Pending:** `github.com/garnet-lang/` is Jon's manual org
  step and is not visible to the active token yet.
- Registry-stub `index.json` (S13) entries for each package, exercised through
  the actual implemented CLI shape:
  `garnet add --registry <filesystem-registry> <name>@0.1.0`.
- `examples/mvp_18_all_official_packages/` — consumer project that vendors all
  five packages from the local registry seed, uses one primitive from each, and
  runs via `garnet run src/main.garnet`.
- New lane in `scripts/garnet_mit_readiness_status.py`: `official_packages_seed`.

**Deps:** **HARD block on S17 / MERGED** — packages annotate per the Layer
Policy doc. The `garnet-lang/` GitHub org is **Jon's manual step** (prompt via
the ledger Shared Messages section if it isn't created). Source-level
`@stability(...)` on user/package functions still waits on the parser annotation
handoff, so v0.7 package stability is declared in `Garnet.toml` and docs while
source remains runnable.

**Dogfood block:**

```bash
cargo run -p garnet-registry-stub -- build examples/garnet_lang_registry_seed
cargo run -p garnet-registry-stub -- verify examples/garnet_lang_registry_seed
python3 scripts/smoke_garnet_lang_packages_seed.py
```

**Honest partial labels available:**
- "Five Layer-2 package seeds are local-registry-source-ready at `@stability(experimental)`."
- "External `github.com/garnet-lang/*` package repos are pending the `garnet-lang` org/manual authority."
- "`http-client` and `llm` expose descriptor-level source proof here; live transport remains future work."
- "Layer-2 packages are NOT bundled with v0.7 binaries; they ship via the registry once publication exists."

**State:** started / local-registry-source-ready on `agent-mac-codex/s18-llm-package`; external publication pending.

---

### S19 — Compiler-as-agent LLM tier (`garnet-suggest-llm`)

**Slot:** mac-codex · **PRD:** `PRD_D_S18_S19_PACKAGES_LLM.md` · **PR count:** 1 (feature-flagged crate)

**Goal:** Ship the LLM-backed tier of compiler-as-agent. The seam already
exists in S10's `suggest.rs`; this fills in the `LlmClient` trait and three
impls, **feature-flagged** (`llm`), **not built by default**, and clearly
separated from the deterministic rules tier.

**Owned (writable):** `garnet-suggest-llm/` (NEW crate, in main repo).

**New surfaces:**
- `garnet-suggest-llm/` — NEW crate with `[features] default = []; llm = []`.
  `suggest_for_module_with_llm(module, history, client)` is **additive** — it
  does not replace the deterministic `suggest_for_module`.
- `LlmClient` trait boundary in the Rust crate while S18 remains separate;
  the shared `garnet-lang/llm` package trait is a follow-up re-export point.
  Three provider-compatible impls: Anthropic, OpenAI, Ollama.
- CLI: `garnet check --suggest --llm <provider> [--llm-budget N]`. Without
  `--llm`: identical to S10. With `--llm`: deterministic findings + LLM
  findings, clearly separated, each LLM finding tagged
  `@stability(non-deterministic)`.
- Reproducibility log `.garnet-cache/llm-suggest-log.jsonl` (prompt hash,
  model, temperature, response, suggestions, timestamp).
- CI guard that **errors if any determinism job is spawned with `--llm`**.
- `benchmarks/paper_vi_exp3_compiler_as_agent/` — harness only:
  `codebase_versions/`, `run_stateless.sh`, `run_history_aware.sh`,
  `aggregate.py`, `analyze.py`.
- New lane in `scripts/garnet_mit_readiness_status.py`: `compiler_agent_llm_tier`.

**Deps:** S17 has landed the `@stability` vocabulary. The shared
`garnet-lang/llm` package trait remains S18 work; this S19 PR keeps a local
Rust trait boundary and marks the cross-package re-export as deferred.

**Dogfood block:**

```bash
cargo build --features llm -p garnet-suggest-llm --release
cargo test --features llm -p garnet-suggest-llm
python3 scripts/check_determinism_no_llm.py
python3 scripts/test_check_determinism_no_llm.py
bash benchmarks/paper_vi_exp3_compiler_as_agent/run_stateless.sh
bash benchmarks/paper_vi_exp3_compiler_as_agent/run_history_aware.sh
python3 scripts/garnet_mit_readiness_status.py

# Pending CLI handoff before release-gate reproduction:
# Set ANTHROPIC_API_KEY or OPENAI_API_KEY or OLLAMA_HOST.
# garnet check --suggest --llm anthropic examples/mvp_03_*.garnet
# Expected after handoff: deterministic + non-deterministic suggestions,
# clearly labeled, each with a stability tag. Reproducibility log appears:
# tail -1 .garnet-cache/llm-suggest-log.jsonl | python3 -m json.tool
```

**Honest partial labels available:**
- "S19's LLM tier is non-deterministic; the determinism CI gate does not run with `--llm`. This is explicit by design and noted in CHANGELOG."
- "Streaming, function calling, and vision are NOT in v0.7; they're v0.8 work."
- "Paper VI Experiment 3 (compiler-as-agent time-to-fix) harness ships in v0.7; **running** the experiment to produce h₃a/h₃b/h₃c results is a separate v0.7.1 task."
- "`garnet check --suggest --llm` is pending the read-only `garnet-cli` handoff; until that lands, S19 is `feature-gated-source-ready`, not shipped end-to-end."

**State:** PR-open / feature-gated-source-ready on `agent-mac-codex/s19-suggest-llm`
(PR #233);
public CLI dogfood is pending the ledgered `garnet-cli` handoff before
release-gate reproduction. PR-open is allowed only as
`feature-gated-source-ready`, not shipped end-to-end.

---

### S20 — Novel-composition dogfood + program-execution discovery

**Slot:** win-opus · **Type:** post-v0.7 dogfood extension (Jon-directed) · **PR count:** 1

**Goal:** Surface novel discoveries by **fusing** multiple Paper-VI contributions
into single runnable programs (the existing corpus proves each in isolation), and
prove they `garnet check` clean and `garnet run` with deterministic output.

**Owned (writable):** NEW `examples/novel_*.garnet`, NEW
`scripts/smoke_garnet_novel_compositions.py` (+ `test_*.py`), NEW
`C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md`, section-scoped
cross-cutting. **No edits** to any owned crate or to win-codex's
`smoke_garnet_studio_domain_matrix.py` (this complements, not duplicates, it).

**New surfaces:**
- `examples/novel_01_capability_budgeted_memory_agent.garnet` (caps-budget +
  memory-recall + agent pipeline → governance 16).
- `examples/novel_02_signed_provenance_pipeline.garnet` (BLAKE3 provenance +
  pipeline + determinism → verified content-addressed lineage).
- `examples/novel_03_release_gate_quorum.garnet` (release-gate + caps + provenance
  + memory quorum → APPROVED quorum 4).
- `scripts/smoke_garnet_novel_compositions.py` (+ unittest) and the
  `novel_composition_dogfood` readiness lane; story doc
  `GARNET_NOVEL_COMPOSITIONS.md`. Baseline regenerated.

**Deps:** none. Uses the proven runnable managed-mode subset + `crypto::blake3`.

**Dogfood block:**

```bash
cargo build -p garnet-cli --release
python3 scripts/smoke_garnet_novel_compositions.py    # 3/3 check clean + deterministic run
python3 -m unittest scripts.test_garnet_novel_compositions
# Expect: gate PASS; novel_01 governance 16; novel_02 verified fingerprint;
# novel_03 APPROVED quorum 4. Then the common gates + readiness --check-no-regression.
```

**Honest partial labels available:**
- "Novel compositions are modeled deterministically in managed mode (the proven
  runnable subset + `crypto::blake3`); they prove the composition shape executes
  and is reproducible, not live runtime integration."
- "Live actor mailboxes, Mnemos stores, and Ed25519 signing are tracked
  separately and not claimed by these programs."
- "The new S17 Layer-0/1 stdlib primitives are not interpreter-dispatched yet, so
  these programs use the proven runnable subset."

**State:** dogfood-passing (local: harness 3/3 PASS + 8 unittests; `garnet check`
clean + deterministic `garnet run` on all three; workspace gates green).

---

### S21 — Interpreter dispatch for the S17 Layer-0/1 stdlib + Mnemos × stdlib

**Slot:** win-opus · **Type:** post-v0.7 (Jon-directed; closes the S17 deferred line) · **PR count:** 1

**Goal:** Make the S17 Layer-0/1 stdlib primitives **execute from Garnet source**
(they were registry-declared but not interpreter-dispatched), and prove the
Mnemos memory core composes with the dispatched stdlib.

**Owned (writable, under Jon's direction):** `garnet-interp-v0.3` (eval.rs +
stdlib_bridge.rs — unowned crate; v0.7 lockdown was for the merged parallel
build), `garnet-stdlib` (mine), new example + dogfood/readiness edits. No edits
to garnet-lsp/cst/parser/suggest-llm/garnet-lang or win-codex's matrix.

**New surfaces:**
- `eval.rs` — fully-qualified-name-first resolution in `eval_path` (backward-compatible).
- `stdlib_bridge.rs` — qualified dispatch for `core::math` (6), `core::cmp` (4),
  `core::iter` (6, incl higher-order map/filter/fold via `call_value`),
  `std::base64` (2) + Rust trampoline tests.
- `examples/novel_04_dispatched_stdlib_pipeline.garnet` (runnable proof) +
  `scripts/smoke_garnet_novel_compositions.py` extended to include it.
- `garnet-interp-v0.3/tests/mnemos_stdlib_combination.rs` (4 Mnemos kinds × stdlib).
- `interp_stdlib_dispatch` readiness lane; baseline surgically extended.

**Dogfood block:**

```bash
cargo build -p garnet-cli --release
garnet run examples/novel_04_dispatched_stdlib_pipeline.garnet   # sum 35 / score 5 / tag c2NvcmU9NQ==
python3 scripts/smoke_garnet_novel_compositions.py               # 4/4 (novel_04 included)
cargo test -p garnet-interp -p garnet-stdlib --no-fail-fast      # incl bridge + Mnemos combo tests
# then the common gates + RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
```

**Honest partial labels available:**
- "`std::json` / `regex` / `uuid` / `env` / `process` / `log` dispatch is S22."
- "A full managed-mode `memory::` prim family (Garnet-callable Mnemos with a handle
  Value) is S22; S21 proves Mnemos × stdlib at the Rust/system level."
- "`core::cmp` / `core::iter` are Value-level bridges; the `garnet_stdlib` generics
  remain the tested Rust reference (Garnet's dynamic `Value` can't be the stdlib's `T`)."
- "`garnet check` on `novel_04` emits non-fatal `@stability` warnings (the new prims
  are `experimental`) — exit 0, by design."

**State:** dogfood-passing (local: 25 bridge tests + 2 Mnemos combo tests + harness
4/4; `core::math::sqrt(16.0)→4`, `core::iter::map([1,2,3,4],double)→[2,4,6,8]`,
`std::base64::encode("hi")→"aGk="` verified live; workspace gates green).

---

### S22 — Stdlib + memory runtime dispatch completion

**Slot:** win-codex · **Type:** post-S21 (Jon-directed; closes the S21 deferred line) · **PR count:** 1

**Goal:** Make the remaining S17 Layer-1 stdlib families and managed-mode
`memory::` Mnemos handles execute from Garnet source.

**New surfaces:**
- Parser path polish: selected keywords are accepted only as qualified path
  segments, so `std::regex::match`, `std::process::spawn`, and
  `memory::working` parse without making those words legal bare identifiers.
- `stdlib_bridge.rs`: dispatch for `std::json`, `std::regex`, `std::uuid`,
  `std::env`, `std::process`, `std::log`, and `memory::working|episodic|semantic|procedural`.
- `Value::Process` / `Value::ProcessStatus` managed process carriers.
- `garnet-interp-v0.3/tests/stdlib_s22_dispatch.rs` source-level integration proof.
- `examples/novel_05_s22_stdlib_memory_pipeline.garnet` and novel-composition
  harness extension to 5/5.
- `stdlib_memory_runtime_dispatch` readiness lane; baseline surgically extended.

**Dogfood block:**

```bash
cargo build -p garnet-cli
cargo test -p garnet-interp --test stdlib_s22_dispatch --no-fail-fast
cargo test -p garnet-interp stdlib_bridge --no-fail-fast
garnet run examples/novel_05_s22_stdlib_memory_pipeline.garnet  # uuid ee54a926-f375-5759-a5aa-67f7d8528cff
python3 scripts/smoke_garnet_novel_compositions.py             # 5/5 (novel_05 included)
python3 -m unittest scripts.test_garnet_novel_compositions
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```

**Honest partial labels available:**
- "`std::env` and `std::process` are proven in Rust integration tests rather
  than `novel_05`, because they mutate or launch host state."
- "`std::process::spawn` still uses the v0.7 whitespace-delimited command-line
  contract from `garnet_stdlib`; richer argv handling is v0.8+."
- "`std::log` is formatting-only; file sinks that require `@caps(fs)` remain v0.8+."

**State:** dogfood-passing locally (`stdlib_s22_dispatch` 4/4, `stdlib_bridge`
29 tests, novel-composition harness 5/5, readiness **82.9% / 34 lanes**).

---

### S23 — `std::process` structured argv + output capture

**Slot:** win-opus · **Type:** post-S22 (Jon-directed; closes the S22 deferred line) · **PR count:** 1

**Goal:** Replace the whitespace-split-only `std::process::spawn` contract with an
explicit-argv path, and add captured-output execution, so a managed Garnet program
can run a host command (program + literal argv) and consume its stdout/exit-code.

**New surfaces:**
- `garnet-stdlib` `process`: `spawn_args(program, [args])` (literal argv — spaces
  in an argument are not re-split) and `output(program, [args])` (run-to-completion
  capturing stdout/stderr/exit-code into an `Output`). `spawn`/`wait`/`exit_code`
  unchanged.
- `registry.rs`: `std::process::spawn_args` + `std::process::output` (Layer 1, cap
  `proc`, `@stability(experimental)`).
- `stdlib_bridge.rs`: qualified dispatch; `std::process::output` returns a
  `{code, stdout, stderr}` map.
- `garnet-interp-v0.3/tests/stdlib_s23_dispatch.rs` source-level integration proof.
- `process_runtime_completion` readiness lane; baseline surgically extended.

**Dogfood block:**

```bash
cargo build -p garnet-cli
cargo test -p garnet-stdlib process --no-fail-fast
cargo test -p garnet-interp --test stdlib_s23_dispatch --no-fail-fast
cargo test -p garnet-interp stdlib_bridge --no-fail-fast
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/smoke_garnet_novel_compositions.py
```

**Honest partial labels available:**
- "Process stdout/stderr are host-dependent (line endings, locale); the
  deterministic proof asserts substring + exit-code, not byte-exact full output."
- "Execution is synchronous managed-mode; no async/OS-thread or streaming-stdout
  claim."
- "`std::log` file sinks requiring `@caps(fs)` remain the next deferred line (S24)."

**State:** dogfood-passing locally (`stdlib_s23_dispatch` 3/3, `garnet-stdlib`
`process` 8/8, `stdlib_bridge` 34 tests, readiness **83.4% / 35 lanes**).

---

### S24 — `std::log` file sink (`@caps(fs)`)

**Slot:** win-opus · **Type:** post-S23 (Jon-directed; closes the S22/S23/S17 deferred line) · **PR count:** 1

**Goal:** Give `std::log` a real file sink so a managed Garnet program can persist
structured log lines to disk under an `@caps(fs)` authority — the loop from S23's
process-output capture to durable, capability-checked observability.

**New surfaces:**
- `garnet-stdlib` `log`: `to_file(path, level, message)` — formats the same
  `[LEVEL] message` line and **appends** it (create-if-missing) to `path`; returns
  the formatted line. Requires `@caps(fs)`. Formatting helpers unchanged.
- `registry.rs`: `std::log::to_file` (Layer 1, cap `fs`, `@stability(experimental)`).
- `stdlib_bridge.rs`: qualified dispatch (arity 3).
- `garnet-interp-v0.3/tests/stdlib_s24_dispatch.rs` source-level write→read-back proof.
- `log_file_sink_runtime` readiness lane; baseline surgically extended.

**Dogfood block:**

```bash
cargo build -p garnet-cli
cargo test -p garnet-stdlib log --no-fail-fast
cargo test -p garnet-interp --test stdlib_s24_dispatch --no-fail-fast
cargo test -p garnet-interp stdlib_bridge --no-fail-fast
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/smoke_garnet_novel_compositions.py
```

**Honest partial labels available:**
- "Line-append text sink only (create-if-missing); no rotation, structured/JSON
  sinks, or async writers."
- "Capability enforcement remains the checker's job; the registry tags
  `std::log::to_file` `@caps(fs)`."

**State:** dogfood-passing locally (`stdlib_s24_dispatch` 2/2, `garnet-stdlib`
`log` 6/6, `stdlib_bridge` 35 tests, readiness **83.9% / 36 lanes**).

---

### S25 — Host-effect composition capstone

**Slot:** win-opus · **Type:** post-S24 (Jon-directed; capstone of the S22–S24 arc) · **PR count:** 1

**Goal:** Prove the S22–S24 runtime surfaces compose end-to-end from Garnet source
into one capability-checked agent pipeline, and tell the story. Additive only — no
new language/stdlib surface.

**New surfaces:**
- `garnet-interp-v0.3/tests/host_effect_composition.rs` — `@caps(proc, fs)` program:
  `std::process::output` (S23) → `std::log::to_file` (S24) → `memory::episodic` (S22)
  → `read_file` → `crypto::blake3`, asserting token/recall/contents/exit-code/
  fingerprint (cfg command + unique temp path → deterministic).
- `examples/novel_06_observability_provenance_pipeline.garnet` — deterministic,
  cross-platform `@caps(fs)` composition (json + file-sink + episodic memory +
  blake3); novel harness now 6/6.
- `GARNET_NOVEL_COMPOSITIONS.md` novel_06 section; `host_effect_composition`
  readiness lane; baseline surgically extended.

**Dogfood block:**

```bash
cargo build -p garnet-cli
cargo test -p garnet-interp --test host_effect_composition --no-fail-fast
garnet run examples/novel_06_observability_provenance_pipeline.garnet  # provenance 791c7dcc…
python3 scripts/smoke_garnet_novel_compositions.py            # 6/6
python3 -m unittest scripts.test_garnet_novel_compositions
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```

**Honest partial labels available:**
- "The deterministic novel example omits the platform-variable process step; the
  process leg is proven in the cfg-guarded integration test."
- "Execution is synchronous managed-mode; no async actor / OS-thread runtime claim."

**State:** dogfood-passing locally (`host_effect_composition` 2/2, novel harness
6/6, readiness **84.3% / 37 lanes**).

---

### S26 — `core::result` combinator dispatch

**Slot:** win-opus · **Type:** post-S25 (Jon-directed; continues the S21/S22 registry→runtime arc) · **PR count:** 1

**Goal:** Make the 6 registered `core::result` primitives execute from Garnet source —
railway-oriented error handling without leaving the language.

**New surfaces:**
- `stdlib_bridge.rs`: `core::result::{ok,err,map,and_then,or_else,unwrap_or}` over
  `Result` Variants (Ok/Err) identical to the prelude builders; map/and_then/or_else
  higher-order via `call_value`; bound qualified (avoids the bare-`map` collision).
- `garnet-interp-v0.3/tests/core_result_dispatch.rs` source-level railway proof.
- `core_result_dispatch` readiness lane; baseline surgically extended.

**Dogfood block:**

```bash
cargo build -p garnet-cli
cargo test -p garnet-interp --test core_result_dispatch --no-fail-fast
cargo test -p garnet-interp stdlib_bridge --no-fail-fast
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```

**Honest partial labels available:**
- "`and_then`/`or_else` trust the callee to return a Result (dynamic typing); no static shape check."
- "Ergonomic method syntax (`result.map(..)`) is a later follow-on; S26 ships the qualified-function form."

**State:** dogfood-passing locally (`core_result_dispatch` 1/1, `stdlib_bridge` 41 tests
incl. 6 new, readiness **84.7% / 38 lanes**).

---

### S27 — `core::option` combinator dispatch

**Slot:** win-opus · **Type:** post-S26 (Jon-directed; sibling of S26) · **PR count:** 1

**Goal:** Make the 5 registered `core::option` primitives execute from Garnet source.

**New surfaces:**
- `stdlib_bridge.rs`: `core::option::{some,none,map,and_then,unwrap_or}` over `Option`
  Variants (Some/None) identical to the prelude builders; map/and_then higher-order
  via `call_value`; bound qualified.
- `garnet-interp-v0.3/tests/core_option_dispatch.rs` source-level proof.
- `core_option_dispatch` readiness lane; baseline surgically extended.

**Dogfood block:**

```bash
cargo build -p garnet-cli
cargo test -p garnet-interp --test core_option_dispatch --no-fail-fast
cargo test -p garnet-interp stdlib_bridge --no-fail-fast
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```

**Honest partial labels available:**
- "`and_then` trusts the callee to return an Option (dynamic typing); no static shape check."
- "Ergonomic method syntax (`option.map(..)`) is a later follow-on."

**State:** dogfood-passing locally (`core_option_dispatch` 1/1, `stdlib_bridge` 47 tests
incl. 6 new, readiness **85.1% / 39 lanes**).

---

### S28 — `core::iter` completion (zip / collect / chain)

**Slot:** win-opus · **Type:** post-S27 (Jon-directed; finishes the core:: registry→runtime arc) · **PR count:** 1

**Goal:** Dispatch the last three registered `core::iter` combinators so all 9 are
runnable from Garnet source.

**New surfaces:**
- `stdlib_bridge.rs`: `core::iter::zip` (pairs two arrays, shorter wins),
  `core::iter::chain` (concatenate), `core::iter::collect` (materialize — `Range`
  expands to ints, Array passes through). Value-level (not higher-order).
- `garnet-interp-v0.3/tests/core_iter_completion_dispatch.rs` source-level proof
  composed with the S21 `fold`.
- `core_iter_completion` readiness lane; baseline surgically extended.

**Dogfood block:**

```bash
cargo build -p garnet-cli
cargo test -p garnet-interp --test core_iter_completion_dispatch --no-fail-fast
cargo test -p garnet-interp stdlib_bridge --no-fail-fast
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```

**Honest partial labels available:**
- "`collect` materializes a `Range` or passes an Array through; there is no lazy
  iterator protocol to collect (eager map/filter already return arrays)."

**State:** dogfood-passing locally (`core_iter_completion_dispatch` 1/1, `stdlib_bridge`
52 tests incl. 5 new, readiness **85.5% / 40 lanes**).

---

### S29 — `@stability` error-level enforcement (opt-in)

**Slot:** win-opus · **Type:** post-S28 (Jon-directed; the named v0.8 "@stability error-level" deferral) · **PR count:** 1

**Goal:** Ship error-level `@stability` enforcement as an opt-in, fully within
garnet-check's stability surface, with the default unchanged (warning-level).

**New surfaces:**
- `garnet-check-v0.3/src/lib.rs`: FATAL `CheckError::StabilityError` variant, listed
  in `CheckReport::ok()` (garnet-cli already exits on `ok()` → no CLI change).
- `garnet-check-v0.3/src/stability.rs`: `GARNET_STABILITY_ERRORS` env gate; under it,
  experimental/deprecated → fatal `StabilityError` ("stability error: …"); frozen → info;
  default → unchanged warning advisories. Unit tests via the env-free `advise(error_mode)`.
- `stability_error_enforcement` readiness lane; baseline surgically extended.

**Dogfood block:**

```bash
cargo build -p garnet-cli
cargo test -p garnet-check stability --no-fail-fast
garnet check examples/novel_04_dispatched_stdlib_pipeline.garnet                            # warns, exit 0
GARNET_STABILITY_ERRORS=1 garnet check examples/novel_04_dispatched_stdlib_pipeline.garnet  # "stability error:", exit 1
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```

**Honest partial labels available:**
- "Error mode is process-global via env var; per-source `@uses(experimental)` opt-out
  still needs the S17 parser-annotation handoff (mac-opus)."
- "Default stays warning-level (opt-in), so CI and existing programs are unaffected."

**State:** dogfood-passing locally (garnet-check 39 lib tests incl. 4 new S29, CLI
end-to-end verified: default warns/exit 0, `GARNET_STABILITY_ERRORS=1` errors/exit 1,
readiness **85.9% / 41 lanes**).

---

### S30 — Functional-core composition capstone

**Slot:** win-opus · **Type:** post-S29 (Jon-directed; capstone of the S26-S28 functional-core arc) · **PR count:** 1

**Goal:** Prove the now-runnable functional `core::` surface (result/option/iter) composes
into railway-oriented pipelines from Garnet source. Additive composition + story.

**New surfaces:**
- `examples/novel_07_functional_core_pipeline.garnet` — deterministic `@caps()` pipeline
  (iter → result → option → `novel_07 final: 80`); novel harness now 7/7.
- `garnet-interp-v0.3/tests/functional_core_composition.rs` — integration test of both the
  Ok/Some success tracks and the Err(or_else-recovered)/None default tracks → `[20,40,0,80,7]`.
- `GARNET_NOVEL_COMPOSITIONS.md` novel_07 section; `functional_core_composition` lane.

**Dogfood block:**

```bash
cargo build -p garnet-cli
cargo test -p garnet-interp --test functional_core_composition --no-fail-fast
garnet run examples/novel_07_functional_core_pipeline.garnet     # novel_07 final: 80
python3 scripts/smoke_garnet_novel_compositions.py               # 7/7
python3 -m unittest scripts.test_garnet_novel_compositions
cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```

**Honest partial labels available:**
- "Pure managed-mode compute (no host effects); the host-effect composition is the S25 capstone."

**State:** dogfood-passing locally (`functional_core_composition` 1/1, novel harness 7/7,
readiness **86.2% / 42 lanes**).

---

## v0.7.0 Release Gate

Tag v0.7.0 only when all of:

- [ ] S15, S16, S17, S18, S19 in `merged` state, and **S15-Compare** decision
      recorded (canonical CST chosen) before S16 merges.
- [ ] `scripts/garnet_mit_readiness_status.py` reports a higher AND more
      granular % than the v0.6.0 close baseline (record the v0.6.0 close
      numbers here when v0.6.0 tags). Five new lanes expected:
      `parser_cst_migration`, `editor_lsp_precision`, `stdlib_layer_policy`,
      `official_packages_seed`, `compiler_agent_llm_tier`.
- [ ] `CHANGELOG.md` updated with each merged slice; the `[Unreleased]`
      block names every entry.
- [ ] `docs/blog/` v0.7 release post (Post 6) drafted using the
      substance-over-surface framing (v0.5 release post as template).
- [ ] Pre-tag clean-machine reproduction passes the v0.7 contract loop:

```bash
rm -rf /tmp/clean && mkdir /tmp/clean && cd /tmp/clean
curl -sSf https://garnet-lang.org/install.sh | sh
garnet new --template cli demo && cd demo

# S15 CST: parser exposes CST mode; roundtrip clean
garnet parse --mode cst src/main.garnet > /dev/null

# S16 LSP v0.3: precision features in VSCode
code --install-extension <published-garnet-vsix-v0.7.0>
# Manual: rename a symbol across two files; apply an "Add @caps" code action

# S17 stdlib: layer policy + @stability + ~50 primitives
garnet check src/main.garnet            # @stability warnings present, not errors

# S18 packages: local Layer-2 package seeds resolve from the filesystem registry
# External github.com/garnet-lang/* publication remains a separate authority gate.
python3 scripts/smoke_garnet_lang_packages_seed.py

# S19 LLM tier (opt-in; non-deterministic, excluded from determinism gate)
garnet check --suggest --llm anthropic src/main.garnet
```

- [ ] Release-asset workflows (Linux packages, macOS tarballs, VSIX) publish
      `v0.7.0` assets with a unified `SHA256SUMS`.
- [ ] `scripts/verify_org_release_smoke.sh` passes against
      `Island-Dev-Crew/garnet` release `v0.7.0`.

**Ordering:** S15 (rowan `garnet-cst`) built independently of #221's in-parser
CST, and **S15-Compare** selected rowan as canonical. **S16 is now unblocked**
and targets rowan while using #221 as a migration oracle. S17 must merge before
S18 starts substantive work. S19 may run in parallel with S18 once S17 ships
`@stability`. Slices may otherwise land out of order under the slice-per-PR
discipline.

---

## PR Body Template

```markdown
## Slice
S<N>: <short>

## Goal
<paste the Goal line from GARNET_v0_7_SLICE_DOGFOOD.md>

## State transition
<previous-state> → <new-state>

## What's in
- 
- 

## What's NOT in (honest partial)
- 
- 

## Dogfood Readiness

### Current truth
- [ ] origin/main tip: …
- [ ] readiness status: …

### Local verification
- [ ] cargo fmt --all -- --check
- [ ] cargo clippy --workspace --all-targets -- -D warnings
- [ ] cargo test --workspace --no-fail-fast
- [ ] python3 scripts/garnet_mit_readiness_status.py --check-no-regression
- [ ] python3 scripts/garnet_conformance_matrix_check.py
- [ ] <slice-specific dogfood block output attached>

### Remote verification
- [ ] PR dogfood evidence
- [ ] cargo test (matrix)
- [ ] clippy
- [ ] cargo-deny check

### Desktop dogfood bundle
- [ ] Bundle path: /Users/idc2.0/Desktop/dogfood/garnet-<slug>-<UTCstamp>/
- [ ] MANIFEST.sha256 sealed; manifest-verify.log records OK.

### Deferred / out of scope
- [ ] <verbatim from honest-partial labels>

## Status reporter delta
Before: <paste relevant scripts/garnet_*_status.py output>
After:  <paste same after this PR>

## Honesty anchors
- This PR does not claim production ARC complete.
- <slice-specific anchor>

## Regression note (if state moved backward)
<one line>
```

---

## Integration with Existing Scripts

Every slice's `merged` transition must verify against the appropriate status
reporter:

| Slice | Status reporter consulted |
|---|---|
| S15 | `garnet_mit_readiness_status.py` (new lane: `parser_cst_migration`) |
| S16 | `garnet_mit_readiness_status.py` (new lane: `editor_lsp_precision`) |
| S17 | `garnet_mit_readiness_status.py` (new lane: `stdlib_layer_policy`) + `garnet_stdlib_layer_gate.py` |
| S18 | `garnet_mit_readiness_status.py` (new lane: `official_packages_seed`) |
| S19 | `garnet_mit_readiness_status.py` (new lane: `compiler_agent_llm_tier`) |

New reporters are written in the same Python style and discipline as
existing ones: deterministic, manifest-backed, no claims beyond their
evidence.

### Windows/Linux Studio Domain Proof Addendum

This is a post-S16 Windows/Linux Studio hardening lane, not a replacement for
S18/S19. It closes the old "parse/check/run proof still needed" gap with a
reproducible domain matrix while keeping package/signing gates open.

```bash
python3 scripts/test_smoke_garnet_studio_domain_matrix.py
python3 scripts/smoke_garnet_studio_domain_matrix.py --suite all
python3 scripts/test_garnet_windows_linux_studio_status.py
python3 scripts/test_garnet_windows_linux_studio_shell.py
```

Honest scope:

- A pass means the selected examples parse, check, and run on the current CLI.
- `mvp_11_signed_hotreload_mismatch.garnet` passes only when Garnet rejects the
  bad reload with the expected BLAKE3 fingerprint diagnostic.
- This does not claim Linux package completion, Windows ARM64, Authenticode,
  winget, provider-backed conversion, or production readiness.

---

## Honesty Anchors (carry forward from v0.5/v0.6, plus v0.7 additions)

These phrases stay verbatim in the README, status outputs, and release blog
through v0.7 — they are brand equity:

Carried from v0.5/v0.6:

- "research-grade prototype (v0.x.x) — not production-complete"
- "tracked-slice ledger is complete, but that is not full MIT/productization
  completion"
- Paper VI scorecard: "4 supported, 2 partial (downgraded honestly), 0
  refuted, 1 pending-infra"
- "production allocator path tracked in MEMORY_CORE_ROADMAP.md"
- "human/aesthetic acceptance remains open"
- "bytecode ABI v0.2 is more stable than v0.1 but not yet a cross-version
  ABI promise"
- "registry stub serves a static index; no central registry, no auth, no
  publish flow"
- "package-manager resolver is local-path-first; remote sources,
  transitive deps, SemVer matching, and workspace mode remain deferred"
- "CST round-trip is source-preserving for canonical examples; recovery
  from malformed input is best-effort"

New for v0.7:

- "downstream consumers (interp, check, vm) still operate on the AST
  projection via `cst_to_ast()`; CST-first migration is v0.8 work"
- "LSP precision is managed-mode only; safe-mode precision and cross-package
  rename are v0.8"
- "`@stability` enforcement is warning-level for backwards compat;
  error-level enforcement is v0.8"
- "Layer-2 packages are not bundled with v0.7 binaries; they ship via the
  registry as `@garnet-lang/*`"
- "the LLM suggest tier is non-deterministic and excluded from the
  determinism gate by design"
- "Paper VI Exp 3 harness ships in v0.7; running it to produce h₃ results is
  v0.7.1"

If a slice would let one of these soften, the slice's PR body says so
explicitly and Jon decides.
