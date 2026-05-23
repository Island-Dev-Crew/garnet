# PRD B — S16: LSP Precision Features

| Field | Value |
|---|---|
| **Slot** | win-codex (Codex Desktop GPT-5.5 Pro Extra High Fast, Windows) |
| **Slice** | S16 |
| **Status** | not-started → planned (HELD for S15-Compare reconciliation) → in-progress → review-ready → dogfood-passing → merged |
| **PR count** | 1 (substantive, after mock-first prep) |

---

## Goal

Upgrade Garnet's LSP from MVP (diagnostics, hover, go-to-def) to precision-tier:
**workspace symbols, rename, code-actions, semantic tokens.** All four are
CST-aware, not regex-driven.

## Why Windows Codex

Implementation-heavy slice with four distinct LSP feature surfaces. GPT-5.5 Pro's
token-thoroughness fits the volume. Each feature is well-bounded and architectural
decisions are scarce — this is execution, not deep design. Plus, Codex's
structured `/plan` and `/run` flow fits multi-feature implementation.

## Owned crates (writable)

- `garnet-lsp` — the primary LSP server crate
- `editors/vscode` — the VSCode extension

## Read-only crates

- `garnet-cst` — the CST trait, published by S15 PR-1
- `garnet-parser-v0.3` — just call `parse_cst`
- `garnet-check-v0.3` — existing diagnostics + S10 suggest rules (reuse, don't redefine)
- `garnet-stdlib` — for code-action surface knowledge

## Dependencies

- **HELD (v0.7 update)**: S16 substantive work is on hold until the
  **S15-Compare** CST reconciliation resolves which CST is canonical (see
  `GARNET_v0_7_SLICE_DOGFOOD.md`). #221 already merged ~578 lines of LSP work in
  `garnet-lsp/src/lib.rs` (plus `scripts/smoke_garnet_lsp_protocol.py`); LSP
  precision must not be built twice. Once Jon picks the winning CST, S16 targets
  it. **Do not delete #221's LSP work while S16 is held** — it is part of the
  comparison surface.
- After reconciliation: the chosen CST's trait surface must be published before
  substantive work; code against a mock impl of that trait until it lands.
- win-codex may read its PRD and draft a plan while held, but should not open a
  substantive S16 PR until Jon posts the reconciliation outcome in the ledger.

---

## Implementation Plan

### 1. Mock-first phase (while S15 PR-1 is in flight)

Create `garnet-lsp/src/mock_cst.rs` — a mock impl of the `CstNode` trait
sufficient to develop and unit-test all four features. Code all four feature
modules against the trait. Use the mock in unit tests.

### 2. Workspace symbols (`textDocument/documentSymbol`)

Walk CST emitting symbol tree:
- modules → functions → params
- Use `SyntaxKind` to classify each symbol's kind (function, struct, enum, etc.)

**Smoke test**: a workspace with N=10 garnet files surfaces all top-level
functions in the symbol tree.

### 3. Rename (`textDocument/rename`)

CST-aware rename:
- walk all references to the symbol via the CST
- emit a `WorkspaceEdit` with all positions
- preserve trivia (comments, whitespace) at every edit site

**Test cases**:
- rename a function
- rename a parameter (local scope)
- rename across files (within the same workspace folder)

**Honest partial**: cross-module rename only works when both modules are in the
same workspace folder. Cross-package rename is v0.8 work — document this in the
PR.

### 4. Code Actions (`textDocument/codeAction`)

Ship **at least three** code actions:

| Action | Trigger | Behavior |
|---|---|---|
| **AddCapsAnnotation** | A managed-mode `def` is missing `@caps(...)` | Insert `@caps(...)` above the function. Reuse S10's `suggest::ManagedFnMissingCaps` for detection. |
| **RefactorLongParameterList** | A function has ≥4 params | Offer to group them into a struct. Reuse S10's `suggest::LongParameterList` for detection; this action does the rewrite. |
| **AddExplicitReturnType** | A function lacks a return type annotation | Insert the inferred type. |

All three are CST-driven rewrites that **preserve trivia**.

### 5. Semantic Tokens (`textDocument/semanticTokens/full`)

CST-driven token classification (replaces the current regex-based highlight in
the VSCode extension's TextMate grammar).

Map `SyntaxKind` to LSP semantic-token categories:

```
keyword, function, type, parameter, comment, string, number,
operator, capability, attribute
```

**Smoke test**: a 100-line garnet file produces N tokens matching the expected
classification.

### 6. VSCode extension updates

- Wire the four new capabilities into the extension's `package.json`.
- Add three new commands:
  - `Garnet: Add @caps annotation`
  - `Garnet: Refactor long parameter list`
  - `Garnet: Add return type annotation`
- Each command invokes the corresponding code action.
- Bump extension version. Repackage `.vsix`.

### 7. Smoke test script

`scripts/smoke_garnet_lsp_precision.py`:
- Spawn `garnet-lsp`.
- Send LSP requests for each of the four feature surfaces.
- Validate responses against expected shapes.
- Run in CI as part of the workspace test gate.

---

## Dogfood block (verification)

```bash
cargo build -p garnet-lsp --release
cargo test -p garnet-lsp --no-fail-fast
(cd editors/vscode && npm install && npm run package)
python3 scripts/smoke_garnet_lsp_precision.py

# Manual confirmation in VSCode (record screenshots for PR body):
#   1. Install the .vsix
#   2. Open examples/mvp_01_*.garnet
#   3. Workspace symbols panel populated → screenshot
#   4. Rename a function → references update across files → screenshot
#   5. Code action "Add @caps" appears on a def missing it → screenshot
#   6. Semantic highlighting: capabilities highlighted as @attribute color → screenshot
```

---

## Out of scope

- LSP for safe-mode (`@safe fn`) precision features — currently managed mode only,
  same as MVP. Safe-mode is v0.8.
- Tree-sitter grammar (independently valuable for GitHub syntax highlighting and
  JetBrains, but a separate slice for v0.8).
- Cross-package rename (works only within a single workspace folder in v0.7).

---

## Coordination

- **Hard sync point**: wait for Jon to post the **S15-Compare reconciliation
  outcome** (which CST is canonical) in `AGENT_COORDINATION_LEDGER.md`. Only then
  watch for the chosen CST's trait-surface merge before starting substantive work.
- **Read-only access** to `garnet-check-v0.3`. If you need a new diagnostic
  surface, file a Handoff Request to **win-opus** (current owner of check
  modifications during v0.7).
- The three code actions reuse S10's `suggest::Rule` enum. **Do NOT** redefine
  the rules in LSP code; import from `garnet-check-v0.3::suggest`.

---

## Honest accounting hooks

- "S16 ships LSP precision for managed mode; safe mode (`@safe fn`) precision is
  v0.8."
- "Cross-workspace rename works within a single folder; cross-package rename is
  v0.8."
- "Three code actions ship in v0.7; the long-tail (add-suggested-tests,
  extract-fn, inline-let) is on the v0.8 roadmap."

---

## Done criteria

- [ ] PR merged with green CI.
- [ ] `AGENT_COORDINATION_LEDGER.md` updated: win-codex / S16 / MERGED.
- [ ] VSCode extension version bumped, packaged, smoke-tested.
- [ ] All four features pass the smoke script.
- [ ] PR body includes screenshots for each of the four manual confirmations above.
- [ ] `CHANGELOG.md` updated under `[Unreleased]`.
- [ ] Readiness reporter gains a "LSP precision" lane.
