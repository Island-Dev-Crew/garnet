# Agent Coordination Ledger — v0.7 Stream

**Single source of truth for the four-agent v0.7 build-out.**

This file is read by every agent at the start of every session, before any tool
call that writes. If you are an agent reading this: identify your slot, find your
slice, read your PRD, then append your STARTED entry under your slot's section
in the Status Board.

If you are Jon: this file is your dashboard. Each agent appends; you read.

---

## Mission

Close slices S15–S19 to land Garnet v0.7.0. Five slices, four agents working in
parallel. The split is engineered so each agent's writable crates do not overlap;
where slices depend on each other, the trait-first pattern unblocks the dependent
slice with a stub.

---

## Slot Assignments

| Slot | Machine | Harness | Slice(s) | Owned crates (writable) | Read-only |
|---|---|---|---|---|---|
| **mac-opus** | Mac | Claude Code Opus 4.7 1M Max | **S15** (CST — rowan crate, built cold) | `garnet-cst` (new), `garnet-parser-v0.3` (lexer/API dependency; **do not touch `src/cst.rs` #221**) | everything else |
| **win-codex** | Windows | Codex Desktop GPT-5.5 Pro Extra High Fast | **S16** (LSP precision — unblocked after S15-Compare) | `garnet-lsp`, `editors/vscode` | `garnet-cst`, `garnet-parser-v0.3`, `garnet-check-v0.3` |
| **win-opus** | Windows | Claude Code Opus 4.7 1M Max | **S17** (stdlib + layer policy + `@stability`) | `garnet-stdlib`, `garnet-check-v0.3` (stability surface only) | parser, interp, vm, lsp, cst |
| **mac-codex** | Mac | Codex Desktop GPT-5.5 Pro Extra High Fast | **S18** (packages) + **S19** (LLM tier) | `garnet-suggest-llm` (new), `garnet-lang/*` (external repos) | `garnet-check-v0.3`, `garnet-stdlib` |

---

> **v0.7 build-both-then-compare (#221).** A Codex PR (#221) already merged a
> hand-rolled in-parser CST (`garnet-parser-v0.3/src/cst.rs`) + ~578 lines of
> LSP. v0.7 does NOT override it: mac-opus builds a rowan `garnet-cst` crate
> independently and additively (preserve #221's `src/cst.rs` untouched), then a
> **S15-Compare** checkpoint (Jon) picks the canonical CST. S15-Compare on
> 2026-05-24 chose the rowan `garnet-cst` crate as canonical; #221 remains a
> temporary legacy migration oracle until rowan-backed LSP migration is green.
> See `GARNET_v0_7_SLICE_DOGFOOD.md` → S15-Compare.

## Dependency Graph

```
#221 in-parser CST (already merged) ──┐
                                       ├──► S15-Compare (Jon) ──► canonical CST
S15 (mac-opus): rowan garnet-cst ──────┘            │
  ├─→ PR-1: CST trait stub                          │
  └─→ PR-2: full rowan impl (built cold)            ▼
                                              S16 (win-codex)
                                              unblocked after S15-Compare;
                                              targets rowan garnet-cst

S17 (win-opus) ───────────────────────────────────────────┐
                                                          ▼
                                                     S18 (mac-codex)
                                                     starts after S17 MERGED

S17 (win-opus, soft) ─────────────────────────────────────┐
                                                          ▼
                                                     S19 (mac-codex)
                                                     can run in parallel with S18
                                                     once S17 ships @stability
```

**Critical sync points**:

1. **mac-opus builds the rowan `garnet-cst` crate cold** (independently of #221's
   in-parser CST), additively, preserving `garnet-parser-v0.3/src/cst.rs`. S17
   (win-opus) has no CST dependency and can start immediately in parallel.
2. **S15-Compare (Jon) picked the canonical CST** once mac-opus's S15 became
   dogfood-passing. Decision: rowan `garnet-cst` is canonical; #221 remains a
   temporary migration oracle.
3. **win-codex's S16 is unblocked** and targets rowan `garnet-cst`. Do not
   delete #221's parser CST until rowan-backed LSP migration is green.
4. **win-opus's S17 must MERGE before mac-codex starts S18 substantive work.**

---

## Cross-Cutting Files (Multi-Agent Coordination Required)

These files are touched by multiple agents. Use the edit pattern listed.

| File | Edit pattern | Lock holder |
|---|---|---|
| `CHANGELOG.md` | Append your slice as a separate bullet under `[Unreleased]`. Append-only. | none |
| `Cargo.toml` (workspace root) | Add your new crate name to alphabetically-sorted `members = [...]`. Append-sort. | none |
| `CURRENT_STATE.md` | Update only the section under your slice. Section-scoped. | none |
| `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md` | Update only your slice's contract block. Section-scoped. | none |
| `scripts/garnet_mit_readiness_status.py` | Add only your slice's lane definition. Section-scoped. | none |
| `README.md` | **DO NOT EDIT** without a Handoff Request approved by Jon. | Jon |
| `C_Language_Specification/GARNET_v1_0_Mini_Spec.md` | **DO NOT EDIT** without a Handoff Request approved by Jon. | Jon |

If you need to edit a cross-cutting file in a way the pattern doesn't permit,
file a Handoff Request in the section at the bottom of this ledger.

---

## Branch Naming

```
agent-<slot>/<slice>-<short>
```

Examples:
- `agent-mac-opus/s15-cst-trait-stub`   ← S15 PR-1 (small, first)
- `agent-mac-opus/s15-cst-rowan`        ← S15 PR-2 (substantive)
- `agent-win-codex/s16-lsp-rename`
- `agent-win-opus/s17-stability-annot`
- `agent-mac-codex/s18-llm-package`
- `agent-mac-codex/s19-suggest-llm`

PR title format: `S<N>: <short description>`.

---

## Status Board

**Each agent appends entries under their own slot section. Do NOT edit another
agent's section. Other agents read this section every session — keep your entries
accurate and current.**

### mac-opus (S15)
- [2026-05-22 20:25 CDT] STARTED agent-mac-opus/s15-cst-trait-stub — claiming S15. Building the rowan `garnet-cst` crate cold + additive per PRD A; parser `src/cst.rs` (#221) left untouched as the S15-Compare baseline. Baseline verified at main tip `c1fc957`: `cargo test --workspace` = 96 suites ok / 0 failed (exit 0); readiness 78.0%; `rowan` not yet in `Cargo.lock` (will add, must clear `cargo deny`). Confirmed `garnet_parser::lex_source*` is already `pub` (trivia-preserving), so `garnet-cst` reuses the lexer as a dependency — no parser-internal change required for PR-1/PR-2. Plan at `.agent/plans/mac-opus-S15-plan.md`; awaiting Jon approval before opening PR-1.
- [2026-05-22 20:59 CDT] PR-OPEN PR#225 (`agent-mac-opus/s15-cst-trait-stub`) — S15 PR-1: garnet-cst trait surface + stub. New rowan crate (SyntaxKind/GarnetLanguage/CstNode/Parse/parse_cst stub), 6 tests + doc-test + proptest. Local gates all green (fmt/clippy/workspace-test/build-release/`deny`/readiness-no-regression/agent-contracts); CI: `PR dogfood evidence` + `agent documentation contracts` pass, matrix/clippy/deny running. #221 `src/cst.rs` untouched (verified). Desktop bundle `garnet-s15-cst-trait-stub-20260523T015601Z` sealed. Paused before PR-2 per Jon. **Note:** 2 pre-existing docs-sync unittest failures on origin/main (`test_garnet_adoption_surface_status`, `test_garnet_converter_status`, from #221) — not S15, not in CI.
- [2026-05-22 21:18 CDT] REVIEW PR#225 — CI fully green (`gh pr checks` EXIT=0: 27 pass / 2 skip / 0 fail / 0 pending); `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`, not draft; PR-body dogfood gate passes; no review findings yet. Ready for PR-Agent Grep Loop + Jon's final merge call. PR-2 (substantive rowan builder) held until #225 merges, per Jon.
- [2026-05-23 03:22 UTC] MERGED PR#225 (squash `4110e1a` on origin/main) — S15 PR-1 trait surface + stub landed; `garnet-cst` crate present on main. #221 `src/cst.rs` baseline preserved.
- [2026-05-22 23:04 CDT] STARTED agent-mac-opus/s15-cst-rowan — S15 PR-2 (substantive). Branched from fresh `origin/main` (`4110e1a`). Built the rowan CST builder cold (direct recursive-descent over the token stream → real composite structure), typed nodes, and `cst_to_ast`. Validated: examples corpus + arbitrary-UTF-8 `proptest` round-trip 100% byte-identical; `cst_to_ast` span-normalized structural parity vs `parse_source` across the corpus; `parse_cst_vs_ast` bench ≈0.99× the AST path (under the 1.5× gate). Added `parser_cst_migration` readiness lane (78.0%→78.8%), regenerated baseline. #221 `src/cst.rs` still untouched. Preparing dogfood bundle + PR-2.
- [2026-05-22 23:12 CDT] PR-OPEN PR#226 (`agent-mac-opus/s15-cst-rowan`) — S15 PR-2: trivia-preserving CST via rowan (builder + nodes + `cst_to_ast` + bench). All 9 dogfood-block/gate commands green locally (fmt/build-release/`-p garnet-cst -p garnet-parser`/bench/workspace-test/clippy/`deny`/readiness-no-regression/agent-contracts, all EXIT 0). Roundtrip 100% (corpus + proptest); `cst_to_ast` span-normalized parity vs `parse_source` on the corpus; bench ≈0.99× AST. New `parser_cst_migration` lane (78.0%→78.8%); baseline regenerated. PR-body dogfood gate passes. Desktop bundle `garnet-s15-cst-rowan-20260523T040956Z` sealed. #221 `src/cst.rs` untouched (verified). Awaiting Jon review/merge; canonical-CST choice = S15-Compare.
- [2026-05-22 23:18 CDT] REVIEW PR#226 — CI fully green (`gh pr checks` EXIT=0: 27 pass / 2 skip / 0 fail / 0 pending); `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`, not draft; PR-body dogfood gate passes; no review findings yet. Ready for PR-Agent Grep Loop + Jon's final merge call. **S15 status:** PR-1 (#225) merged + PR-2 (#226) review-ready ⇒ the rowan `garnet-cst` is dogfood-passing. Next gate is **S15-Compare** (Jon, fresh eyes) to pick the canonical CST before S16 unblocks. I do not merge.
- [2026-05-24 15:15 CDT] MERGED PR#226 (squash `3e45625` on origin/main) — S15 PR-2 landed. S15-Compare recorded in `F_Project_Management/DOGFOOD/S15_CST_COMPARE.md`: rowan `garnet-cst` is canonical for v0.7/S16; #221 parser CST remains a temporary legacy oracle until rowan-backed LSP migration is green. Reconciliation preserved #221's useful token/span surface in `garnet-cst/src/tokens.rs` and added parser-CST token parity tests.

### win-codex (S16)
- [2026-05-24 16:54 CDT] STARTED agent-win-codex/s16-rowan-lsp-precision — fresh branch from origin/main `09d6703`; S15-Compare has selected rowan `garnet-cst`, so S16 is unblocked. Baseline green before edits: readiness 78.8%, `cargo test --workspace --no-fail-fast` exit 0, `cargo clippy --workspace --all-targets -- -D warnings` exit 0. Plan at `.agent/plans/win-codex-S16-plan.md`; proceeding with rowan-backed LSP migration while preserving #221 as migration oracle.
- [2026-05-24 18:23 CDT] PR-OPEN PR#230 — branch `agent-win-codex/s16-rowan-lsp-precision` pushed at `09b090c`; local S16 dogfood green and readiness reporter moved 78.8% -> 79.6%. Entering review/CI loop; S16 is not merged until PR#230 is green and reviewed.
- [2026-05-24 18:33 CDT] REVIEW PR#230 — first remote CI pass was green except `PR dogfood evidence`; root cause was missing required readiness-sensitive PR body headings, not code/test failure. PR body updated and locally validated with `scripts/check_dogfood_pr_body.py`; this ledger update retriggers remote CI against the corrected PR body. Not merged.
- [2026-05-24 18:39 CDT] REVIEW PR#230 — Grep Loop pass 1 returned 3/5 confidence with two fixable findings: VS Code refactor command filtered QuickFix while the server emitted RefactorRewrite, and the precision smoke verified semantic-token legend but not emitted category payload. Patched both; focused local proof passes (`cargo test -p garnet-lsp --no-fail-fast`, `npm run package`, `scripts/smoke_garnet_lsp_precision.py`). Not merged.
- [2026-05-24 18:46 CDT] MERGED PR#230 (squash `d539192` on origin/main) — S16 landed after Grep Loop pass 2 reached 5/5 and remote CI completed green (27 success / 2 skipped / 0 fail). `editor_lsp_precision` is verified in the readiness reporter at 79.6% overall; deferred scope remains safe-mode precision, cross-package rename, per-project semantic-token themes, and Marketplace/OpenVSX publication.

### win-opus (S17)
- [2026-05-24 16:25 CDT] STARTED agent-win-opus/s17-stdlib-layers — branched off fresh `origin/main` (`09d6703`, S15 fully merged + S15-Compare recorded). Baseline re-verified on this base: `cargo clippy --workspace --all-targets -- -D warnings` exit 0; readiness 78.8% / 26 lanes; `--check-no-regression` exit 0 (workspace test re-running). Plan at `.agent/plans/win-opus-S17-plan.md`. S17 has no upstream dep. Checked what landed per Jon: S15 CST is in, but the parser still hard-errors on unknown annotations (`functions.rs:96`; no `Annotation::Stability`) — so `@stability`/`@uses`/`@migration` source syntax is **not yet parseable**. Filed a Handoff Request to mac-opus (below). Shipping the parser-independent bulk now (Layer Policy doc, ≥50 prims with registry stability+layer metadata, `@caps(env)` known-cap, registry-driven stability warnings, `garnet_stdlib_layer_gate.py` + `stdlib_layer_policy` lane); source-level `@stability` enforcement on user fns labeled pending-handoff.
- [2026-05-24 17:45 CDT] PR-OPEN PR#231 (`agent-win-opus/s17-stdlib-layers`) — S17: stdlib expansion + layer policy + `@stability`. Merged current `origin/main` (`d103525`, S16) into the branch; cross-cutting conflicts (CHANGELOG/CURRENT_STATE/dogfood/readiness/baseline/ledger) resolved keeping **both** slices' additive edits; `mergeable=MERGEABLE`. All local gates green on the merged tree: `fmt --check`, `clippy -D warnings`, `cargo test --workspace`, readiness `--check-no-regression`, conformance matrix, layer-gate (77 prims / 100% `@stability` / PASS) + its 9-test unittest, and `garnet check examples/mvp_*.garnet` → 0 new diagnostics across all 13. Readiness 79.6% → **80.4%** (28 lanes; baseline regenerated to full snapshot). Pre-existing Windows-only `scripts/` failures (shasum / cp1252 / exec-bit / SwiftPM) are unrelated to S17; Linux CI is the cross-cutting authority.
- [2026-05-24 17:50 CDT] REVIEW PR#231 — self-driven grep-loop vs PRD-C + the S17 contract. Done-criteria all met: ≥50 prims (77), ≥95% explicit `@stability` (100%), layer-gate reports numbers, `GARNET_STDLIB_LAYER_POLICY.md` exists + referenced from `CURRENT_STATE.md`, CHANGELOG `[Unreleased]` updated, baseline regenerated. Crate ownership clean (writes only in `garnet-stdlib` + `garnet-check-v0.3` + permitted cross-cutting). **Confidence 5/5 on the shipped scope**; the parser-handoff half (source-level `@stability`/`@uses`/`@migration` on user fns) is honestly labeled pending, not a defect. Per stream convention I do **not** self-merge — ready for Jon's merge call. CI: dogfood-evidence re-running against the corrected PR body on the next sync; matrix/clippy/deny/VSIX/SBOM in progress.
- [2026-05-24 18:35 CDT] REVIEW PR#231 — **CI fully GREEN** (`gh pr checks` = 27 pass / 0 fail / 2 skip); `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`, not draft. Two CI-only issues found and fixed during review: (1) `cargo doc -D warnings` — ambiguous `[\`env\`]` intra-doc link (`env` is also the `env!` macro) → disambiguated with `mod@` (`3a921e1`); (2) agentic-matrix `check-advertised-safe-io-example` (expects "0 diagnostics") — bare `ok(...)`/`err(...)` builders were flagged because new `core::result::ok/err` share the name → `@stability` advisory now resolves **qualified-only** (`ddd7517`); `garnet check` on all 23 examples shows 0 stability diagnostics, qualified experimental calls still warn. Readiness 80.4% (28 lanes). **Ready for merge — awaiting Jon's call (no self-merge per convention).**

### mac-codex (S18, S19)
- (empty)

### Entry format

```
- [YYYY-MM-DD HH:MM TZ] <STATUS> <branch-name> — one-line summary
```

Where `STATUS` ∈ `{STARTED, PR-OPEN, REVIEW, BLOCKED, MERGED}`.

Example:
```
- [2026-05-23 09:15 CT] STARTED agent-mac-opus/s15-cst-trait-stub — opening PR-1 trait surface
- [2026-05-23 11:40 CT] PR-OPEN PR#221 — S15: garnet-cst trait surface + stub
- [2026-05-23 14:10 CT] MERGED PR#221
- [2026-05-23 14:15 CT] STARTED agent-mac-opus/s15-cst-rowan — substantive PR-2
```

---

## Shared Messages

**Cross-agent communication. Append timestamped messages. Read by all agents every
session.**

- [2026-05-24 15:15 CDT] FROM:Jon/Codex TO:all — S15-Compare complete: rowan `garnet-cst` is the canonical CST. S16 is unblocked and should target rowan, using #221's parser CST/LSP behavior only as a migration oracle. Do not delete `garnet-parser-v0.3/src/cst.rs` until rowan-backed LSP rename/semantic-token coverage is green.
- [2026-05-24 16:28 CDT] FROM:win-opus TO:mac-codex — S17's Layer Policy doc `C_Language_Specification/GARNET_STDLIB_LAYER_POLICY.md` is the spec your S18 Layer-2 packages (`@garnet-lang/*`) code against: five-layer model, promotion criteria (`@garnet-lang/*` → `std::`), deprecation policy, and the `@stability(...)` semantics table. Cap-set heads-up for S19: S17 adds `@caps(env)` as a known capability in `garnet-check-v0.3`'s known-cap gate (`lib.rs`), alongside fs/net/net_internal/time/proc/ffi/*. If S19's `@caps(net)` LLM-client enforcement touches that same gate, coordinate here so we don't edit the known-set in conflicting ways.
- [2026-05-24 16:55 CDT] FROM:win-opus TO:mac-codex — **Layer Policy doc is DRAFT-COMPLETE** at `C_Language_Specification/GARNET_STDLIB_LAYER_POLICY.md` (on branch `agent-win-opus/s17-stdlib-layers`; lands when the S17 PR opens). §1 five-layer model + concrete v0.7 module→layer table, §2 promotion criteria, §3 deprecation policy, §4 `@stability` tiers + the enforcement severity table, §5 the "capability surface + spec volatility" principle with the JSON-vs-LLM worked example. S18 packages: declare `@stability(experimental)` at v0.1.0 and target Layer 2 (`@garnet-lang/*`) — per §5, the `llm`/`http-client` packages stay Layer 2 (net authority + quarterly provider churn), not `std::`. Flag anything in the doc that's ambiguous for package authoring here.

### Message format

```
- [YYYY-MM-DD HH:MM TZ] FROM:<slot> TO:<slot|all> — message body
```

---

## Handoff Requests

**If an agent needs to modify a crate they don't own, append a request here. The
owner reviews and either accepts (and does the work) or proposes an alternative.
Do NOT proceed with the modification until the owner has accepted.**

- [2026-05-22 20:50 CDT] FROM:mac-opus TO:Jon (no v0.7 slot owns `garnet-cli`)
  Crate / File: `garnet-cli/src/cmd/parse.rs` (and CLI arg wiring)
  Change requested: add a `garnet parse --mode cst <file>` flag that routes to
  the rowan path (`garnet_cst::parse_cst`) and prints/round-trips the CST. The
  v0.7.0 release gate's clean-machine loop references `garnet parse --mode cst
  src/main.garnet`, but `garnet-cli` is read-only for S15 (mac-opus owns only
  `garnet-cst` + `garnet-parser-v0.3`).
  Why I need it: PRD A §4 lists a `--mode cst` flag, but the actual `garnet
  parse` command lives in `garnet-cli`. My S15 dogfood block does not require
  the CLI flag, so S15 ships the rowan path as a library API and files this so
  the release-gate line isn't silently dropped.
  Proposed alternative if you can't: keep S15 library-only and defer the CLI
  flag to the release-gate prep. **Sequencing note:** best actioned *after* PR-2
  lands the real builder, and ideally *after* S15-Compare picks the canonical
  CST, so the CLI wires to the winner rather than a stub or a soon-to-be-
  superseded impl.
  RESOLUTION: Accepted after S15-Compare selected rowan as canonical. This
  reconciliation branch wires `garnet parse --mode cst <file>` to
  `garnet_cst::parse_cst`; default `garnet parse <file>` remains AST mode.

- [2026-05-24 16:28 CDT] FROM:win-opus TO:mac-opus
  Crate / File: `garnet-parser-v0.3/src/ast.rs` (`Annotation` enum) + `garnet-parser-v0.3/src/grammar/functions.rs` (`parse_annotations`, the unknown-name arm at ~line 96)
  Change requested: Add three annotation variants + parse rules so the attribute machinery accepts them: `@stability(<ident>)` → `Annotation::Stability(StabilityTier, Span)` (tier ∈ {stable, experimental, frozen, deprecated}); `@uses(<ident>)` → `Annotation::Uses(String, Span)` (opt-in marker, e.g. `@uses(experimental)`); `@migration("<text>")` → `Annotation::Migration(String, Span)`. Today `parse_annotations` returns `ParseError::unexpected_token` on any name outside {max_depth, fan_out, require_metadata, safe, dynamic, caps, mailbox, nonsendable}, so source containing `@stability(...)` fails to parse outright. (`@caps(env)` needs NO parser change — it already parses as `Capability::Other("env")`; S17 whitelists it inside `garnet-check-v0.3`.)
  Why I need it: S17 ships the `@stability` semantics now — Layer Policy doc, registry stability tiers on all primitives, and registry-driven call-site warnings — all parser-independent. The remaining half is letting USER functions annotate their own `@stability/@uses/@migration` and enforcing it in `garnet-check-v0.3` (which I own and will wire immediately once the variants exist). I cannot author the parser change under crate ownership.
  Proposed alternative if you can't / not now: No blocker on the S17 merge — S17 ships and merges with primitive-stability enforcement; the user-annotation half lands in a small follow-up PR after these variants exist. Sequencing is up to you; nothing downstream (S18/S19) waits on it.
  RESOLUTION: <mac-opus fills this in>

### Handoff request format

```
- [YYYY-MM-DD HH:MM TZ] FROM:<slot> TO:<owner-slot>
  Crate / File: <path>
  Change requested: <one paragraph>
  Why I need it: <one paragraph>
  Proposed alternative if you can't: <optional>
  RESOLUTION: <owner fills this in>
```

---

## Glossary

- **PR-1, PR-2, ...** — when one slice ships via multiple PRs (e.g., S15 trait
  stub then full impl), they're numbered within the slice.
- **Trait-first** — when slice B depends on slice A, A ships a small "trait stub"
  PR first so B can begin mock-first against the stable interface.
- **Mock-first** — B builds against the trait with a mock impl; replaces the mock
  with the real impl after A's substantive PR lands.
- **Dogfood block** — the reproducible verification commands at the end of each
  slice's contract in `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`.
- **Calibrated honesty** — "shipped" only if reproducible from a clean clone.
  Otherwise "partial," "scaffold," "advisory," or "pending-infra" — labeled, not
  buried. The voice that built v0.4–v0.6; carry it forward.
