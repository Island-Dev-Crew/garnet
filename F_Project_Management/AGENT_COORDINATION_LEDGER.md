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
| **mac-opus** | Mac | Claude Code Opus 4.7 1M Max | **S15** (CST — rowan crate, build cold) | `garnet-cst` (new), `garnet-parser-v0.3` (CST mode opt-in; **do not touch `src/cst.rs` #221**) | everything else |
| **win-codex** | Windows | Codex Desktop GPT-5.5 Pro Extra High Fast | **S16** (LSP precision — **HELD until S15-Compare**) | `garnet-lsp`, `editors/vscode` | `garnet-cst`, `garnet-parser-v0.3`, `garnet-check-v0.3` |
| **win-opus** | Windows | Claude Code Opus 4.7 1M Max | **S17** (stdlib + layer policy + `@stability`) | `garnet-stdlib`, `garnet-check-v0.3` (stability surface only) | parser, interp, vm, lsp, cst |
| **mac-codex** | Mac | Codex Desktop GPT-5.5 Pro Extra High Fast | **S18** (packages) + **S19** (LLM tier) | `garnet-suggest-llm` (new), `garnet-lang/*` (external repos) | `garnet-check-v0.3`, `garnet-stdlib` |

---

> **v0.7 build-both-then-compare (#221).** A Codex PR (#221) already merged a
> hand-rolled in-parser CST (`garnet-parser-v0.3/src/cst.rs`) + ~578 lines of
> LSP. v0.7 does NOT override it: mac-opus builds a rowan `garnet-cst` crate
> independently and additively (preserve #221's `src/cst.rs` untouched), then a
> **S15-Compare** checkpoint (Jon) picks the canonical CST. **S16 is HELD until
> that decision.** See `GARNET_v0_7_SLICE_DOGFOOD.md` → S15-Compare.

## Dependency Graph

```
#221 in-parser CST (already merged) ──┐
                                       ├──► S15-Compare (Jon) ──► canonical CST
S15 (mac-opus): rowan garnet-cst ──────┘            │
  ├─→ PR-1: CST trait stub                          │
  └─→ PR-2: full rowan impl (built cold)            ▼
                                              S16 (win-codex)
                                              HELD until S15-Compare;
                                              targets the canonical CST

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
2. **S15-Compare (Jon) picks the canonical CST** once mac-opus's S15 is
   dogfood-passing. The decision is recorded here in the ledger.
3. **win-codex's S16 is HELD until the S15-Compare decision is posted**, then
   targets the canonical CST. Do not start substantive LSP work before then.
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

### win-codex (S16)
- (empty)

### win-opus (S17)
- (empty)

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

(empty)

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
  RESOLUTION: <owner fills this in>

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
