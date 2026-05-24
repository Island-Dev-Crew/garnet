# S16 LSP Precision Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship S16 LSP precision only after S15-Compare chooses the canonical CST: workspace symbols, rename, code actions, and semantic tokens for managed-mode Garnet.

**Architecture:** `garnet-lsp` remains the only server implementation. It will use a small CST adapter layer so the current in-parser CST from #221 or the future rowan `garnet-cst` crate can be selected without rewriting the feature modules. `editors/vscode` only declares and invokes LSP-backed capabilities; it must not duplicate language analysis.

**Tech Stack:** Rust, `tower-lsp`, Garnet parser/checker APIs, canonical CST trait selected by S15-Compare, VSCode TypeScript extension, Python stdio smoke harness.

---

## Status And Blocking Rule

S16 is HELD by `F_Project_Management/AGENT_COORDINATION_LEDGER.md` and `F_Project_Management/PRD_B_S16_LSP_PRECISION.md`.

Do not open the substantive S16 PR until the ledger contains the S15-Compare decision naming the canonical CST. This plan is allowed while held. Implementation starts only after that ledger entry exists.

Baseline already observed on 2026-05-24:

- `python3 scripts/garnet_mit_readiness_status.py` passed and reported `active-partial` at `78.0%`.
- `cargo test --workspace --no-fail-fast` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.

## Source Reading Notes

- PRD reference: `F_Project_Management/PRD_B_S16_LSP_PRECISION.md`, sections "Implementation Plan", "Dogfood block", "Honest accounting hooks", and "Done criteria".
- Contract reference: `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`, section `### S16 - LSP precision features`.
- Current LSP code: `garnet-lsp/src/lib.rs` already exposes diagnostics, hover, go-to-definition, document symbols, workspace symbols, rename, some quick fixes, and semantic tokens on top of `garnet_parser::parse_source_cst`.
- Current gap: #221 shipped useful S16-adjacent work, but the v0.7 contract says it is comparison surface until S15-Compare chooses the canonical CST.
- Current VSCode extension: `editors/vscode/package.json` is still described as LSP MVP and does not declare S16 command contributions.

## Writable Surface

Allowed by PRD:

- `garnet-lsp/**`
- `editors/vscode/**`

Allowed by the S16 contract as section-scoped or explicit new surfaces:

- `.agent/plans/win-codex-S16-plan.md`
- `scripts/smoke_garnet_lsp_precision.py`
- `scripts/garnet_mit_readiness_status.py`, adding only the `editor_lsp_precision` lane
- `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`, updating only the S16 block
- `CURRENT_STATE.md`, updating only the LSP/editor current-state section
- `CHANGELOG.md`, appending only a separate S16 bullet under `[Unreleased]`
- `F_Project_Management/AGENT_COORDINATION_LEDGER.md`, appending only under `win-codex`

Read-only unless a handoff request is accepted:

- `garnet-cst/**`
- `garnet-parser-v0.3/**`
- `garnet-check-v0.3/**`
- `garnet-stdlib/**`
- `README.md`
- `C_Language_Specification/GARNET_v1_0_Mini_Spec.md`

## File Structure

Target structure after S15-Compare unblocks S16:

- Modify `garnet-lsp/src/lib.rs`: keep the public server entry points and wire modules together.
- Create `garnet-lsp/src/cst_adapter.rs`: canonical CST adapter used by all precision features.
- Create `garnet-lsp/src/symbols.rs`: document and workspace symbol indexing.
- Create `garnet-lsp/src/rename.rs`: same-workspace rename edits, with parameter-local and function workspace cases.
- Create `garnet-lsp/src/code_actions.rs`: AddCapsAnnotation, RefactorLongParameterList, AddExplicitReturnType.
- Create `garnet-lsp/src/semantic_tokens.rs`: CST token classification and LSP token encoding.
- Create `garnet-lsp/src/mock_cst.rs`: test-only mock CST surface if the canonical trait requires a mock while S15 settles.
- Create `scripts/smoke_garnet_lsp_precision.py`: stdio smoke covering all four S16 surfaces.
- Modify `editors/vscode/package.json`: extension version, capability-facing command contributions, packaged VSIX name.
- Modify `editors/vscode/src/extension.ts`: register three commands that request the matching LSP code action.
- Modify `editors/vscode/README.md`: replace MVP caveats with S16 honest partial labels.

## Task 0: Unblock Gate

**Files:**
- Read: `F_Project_Management/AGENT_COORDINATION_LEDGER.md`
- Read: `F_Project_Management/DOGFOOD/S15_CST_COMPARE.md`

- [ ] **Step 1: Confirm S15-Compare decision exists**

Run:

```powershell
Select-String -Path F_Project_Management\AGENT_COORDINATION_LEDGER.md -Pattern 'S15-Compare|canonical CST'
```

Expected: a ledger entry from Jon naming the canonical CST.

- [ ] **Step 2: Confirm comparison artifact exists**

Run:

```powershell
Test-Path F_Project_Management\DOGFOOD\S15_CST_COMPARE.md
```

Expected: `True`.

- [ ] **Step 3: If the decision is absent, stop**

Append a `BLOCKED` entry under `win-codex`:

```markdown
- [YYYY-MM-DD HH:MM CT] BLOCKED agent-win-codex/s16-lsp-precision - waiting for S15-Compare canonical CST decision; no substantive LSP edits made.
```

Do not modify `garnet-lsp` or `editors/vscode`.

## Task 1: Branch And Current Baseline

**Files:**
- Modify: `F_Project_Management/AGENT_COORDINATION_LEDGER.md`

- [ ] **Step 1: Use the required branch**

Run:

```powershell
git switch main
git pull --ff-only
git switch -c agent-win-codex/s16-lsp-precision
```

Expected: branch name is `agent-win-codex/s16-lsp-precision`.

- [ ] **Step 2: Re-run the pre-edit baseline**

Run:

```powershell
python3 scripts/garnet_mit_readiness_status.py
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: all commands pass before implementation edits.

- [ ] **Step 3: Append in-progress ledger entry only after S15-Compare exists**

Append:

```markdown
- [YYYY-MM-DD HH:MM CT] STARTED agent-win-codex/s16-lsp-precision - S15-Compare selected <canonical-cst>; beginning substantive S16 implementation.
```

## Task 2: Canonical CST Adapter

**Files:**
- Create: `garnet-lsp/src/cst_adapter.rs`
- Modify: `garnet-lsp/src/lib.rs`
- Test: `garnet-lsp/src/cst_adapter.rs`

- [ ] **Step 1: Write adapter tests**

Add tests proving the adapter can enumerate tokens with spans and classify identifier tokens from this source:

```garnet
/// Friendly greeting
def greet(name) {
  name
}
```

Expected tokens include `def`, `greet`, `name`, braces, whitespace/trivia, and the doc comment span.

- [ ] **Step 2: Implement the adapter against the selected CST**

If S15-Compare keeps the #221 parser CST, wrap `garnet_parser::parse_source_cst`.

If S15-Compare selects rowan `garnet-cst`, add only an LSP-local dependency in `garnet-lsp/Cargo.toml` and wrap the published trait. Do not edit parser or CST crates.

- [ ] **Step 3: Run adapter checks**

Run:

```powershell
cargo test -p garnet-lsp cst_adapter --no-fail-fast
```

Expected: adapter tests pass.

## Task 3: Workspace And Document Symbols

**Files:**
- Create: `garnet-lsp/src/symbols.rs`
- Modify: `garnet-lsp/src/lib.rs`
- Test: `garnet-lsp/src/symbols.rs`

- [ ] **Step 1: Write symbol tests**

Add a test workspace with three in-memory files:

```garnet
module Alpha {
  def greet(name) { name }
}
```

```garnet
struct User { name: String }
enum Status { Ready, Done }
```

```garnet
actor BuildAgent {
  protocol build(spec: BuildSpec) -> BuildResult
  on build(spec) { spec }
}
```

Expected: document symbols include nested module/function shape, and workspace symbols return `greet`, `User`, `Status`, and `BuildAgent`.

- [ ] **Step 2: Move existing symbol collection into `symbols.rs`**

Preserve current `SymbolInfo` behavior, but add container names and child symbols when the CST/AST provides nesting.

- [ ] **Step 3: Run symbol checks**

Run:

```powershell
cargo test -p garnet-lsp symbols --no-fail-fast
```

Expected: all symbol tests pass.

## Task 4: Rename Precision

**Files:**
- Create: `garnet-lsp/src/rename.rs`
- Modify: `garnet-lsp/src/lib.rs`
- Test: `garnet-lsp/src/rename.rs`

- [ ] **Step 1: Write failing rename tests**

Test cases:

- Function rename in one file changes the definition and call sites.
- Function rename across two open files changes both files.
- Parameter rename changes only references in the function body.
- Parameter rename does not change same-name bindings in another function.
- Rename refuses a keyword and returns `None`.

- [ ] **Step 2: Implement scoped rename**

Use the CST adapter for token spans and AST symbol information for scope. Do not use raw regex replacement. Keep the honest boundary: same workspace folder only; cross-package rename is v0.8.

- [ ] **Step 3: Run rename checks**

Run:

```powershell
cargo test -p garnet-lsp rename --no-fail-fast
```

Expected: all rename tests pass, including local parameter scope.

## Task 5: Code Actions

**Files:**
- Create: `garnet-lsp/src/code_actions.rs`
- Modify: `garnet-lsp/src/lib.rs`
- Test: `garnet-lsp/src/code_actions.rs`

- [ ] **Step 1: Write code-action tests**

Use these inputs:

```garnet
def main() {
}
```

Expected: AddCapsAnnotation inserts `@caps()` immediately above `def main`.

```garnet
@caps()
def configure(host, port, user, password) {
  host
}
```

Expected: RefactorLongParameterList offers a workspace edit that introduces a config struct scaffold and changes the function signature in a conservative advisory edit.

```garnet
@caps()
def meaning() {
  42
}
```

Expected: AddExplicitReturnType changes `def meaning()` to `def meaning() -> Int`.

- [ ] **Step 2: Reuse S10 suggestions**

Call `garnet_check::suggest::suggest_for_module`. Use `ManagedFnMissingCaps` and `LongParameterList` directly. Do not modify `garnet-check-v0.3`.

- [ ] **Step 3: Implement AddExplicitReturnType locally**

Infer only obvious literals in v0.7: integer to `Int`, float to `Float`, string to `String`, booleans to `Bool`, symbol to `Symbol`, `nil` to `Nil`. If the final expression is not obvious, do not offer the action.

- [ ] **Step 4: Run code-action checks**

Run:

```powershell
cargo test -p garnet-lsp code_actions --no-fail-fast
```

Expected: all three required actions are present and produce trivia-preserving edits.

## Task 6: Semantic Tokens

**Files:**
- Create: `garnet-lsp/src/semantic_tokens.rs`
- Modify: `garnet-lsp/src/lib.rs`
- Test: `garnet-lsp/src/semantic_tokens.rs`

- [ ] **Step 1: Write semantic-token tests**

Use a 100-line generated source that includes comments, attributes, capabilities, function names, type names, strings, numbers, and operators.

Expected token categories:

```text
keyword, function, type, parameter, comment, string, number, operator, capability, attribute
```

- [ ] **Step 2: Implement static CST classification**

Use the canonical CST token stream. Classify `@caps`, `@safe`, `@dynamic`, `@nonsendable`, `@max_depth`, `@fan_out`, `@require_metadata`, and `@mailbox` as `attribute`; capability identifiers inside `@caps(...)` as `capability`; function parameters as `parameter`.

- [ ] **Step 3: Run semantic-token checks**

Run:

```powershell
cargo test -p garnet-lsp semantic_tokens --no-fail-fast
```

Expected: token encoding is ordered, delta encoded, and non-empty.

## Task 7: VSCode Extension Wiring

**Files:**
- Modify: `editors/vscode/package.json`
- Modify: `editors/vscode/src/extension.ts`
- Modify: `editors/vscode/README.md`

- [ ] **Step 1: Update extension metadata**

Set version to the v0.7-compatible extension version chosen for the release, and change the package output name to include `lsp-precision`.

- [ ] **Step 2: Add commands in `package.json`**

Add:

```json
{
  "command": "garnet.addCapsAnnotation",
  "title": "Garnet: Add @caps annotation"
}
```

```json
{
  "command": "garnet.refactorLongParameterList",
  "title": "Garnet: Refactor long parameter list"
}
```

```json
{
  "command": "garnet.addReturnTypeAnnotation",
  "title": "Garnet: Add return type annotation"
}
```

- [ ] **Step 3: Register command handlers**

Each command should call `vscode.commands.executeCommand('vscode.executeCodeActionProvider', uri, range, 'quickfix')`, select the matching title, and apply its workspace edit.

- [ ] **Step 4: Package extension**

Run:

```powershell
Set-Location editors\vscode
npm install
npm run package
Set-Location ..\..
```

Expected: `.vsix` package is produced.

## Task 8: Precision Smoke Script

**Files:**
- Create: `scripts/smoke_garnet_lsp_precision.py`

- [ ] **Step 1: Build release LSP**

Run:

```powershell
cargo build -p garnet-lsp --release
```

Expected: `target/release/garnet-lsp.exe` exists on Windows.

- [ ] **Step 2: Create stdio smoke**

The smoke script must initialize the server, open at least two `.garnet` files, and validate:

- document symbols
- workspace symbols
- rename across open files
- AddCapsAnnotation
- RefactorLongParameterList
- AddExplicitReturnType
- semantic tokens with the v0.7 token categories

- [ ] **Step 3: Run the smoke**

Run:

```powershell
python3 scripts/smoke_garnet_lsp_precision.py target\release\garnet-lsp.exe
```

Expected: JSON output with `"status": "pass"`.

## Task 9: Readiness And Honest Documentation

**Files:**
- Modify: `scripts/garnet_mit_readiness_status.py`
- Modify: `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`
- Modify: `CURRENT_STATE.md`
- Modify: `CHANGELOG.md`
- Modify: `F_Project_Management/AGENT_COORDINATION_LEDGER.md`

- [ ] **Step 1: Add `editor_lsp_precision` readiness lane**

The lane is `verified` only when the precision smoke passes and the VSIX package exists. Otherwise it is `active-partial`.

- [ ] **Step 2: Update only the S16 dogfood block**

Record the exact commands and output bundle path. Keep these honest partials verbatim:

- `S16 ships LSP precision for managed mode; safe mode (@safe fn) precision is v0.8.`
- `Cross-workspace rename works within a single folder; cross-package rename is v0.8.`
- `Three code actions ship in v0.7; the long-tail is on the v0.8 roadmap.`
- `Semantic tokens use a static classification scheme; per-project token themes deferred.`

- [ ] **Step 3: Append CHANGELOG entry**

Under `[Unreleased]`, add one S16 bullet naming LSP precision and its boundaries.

- [ ] **Step 4: Append ledger PR-OPEN and REVIEW entries as the PR moves**

Use the ledger format exactly.

## Task 10: Verification And PR

**Files:**
- No new source files beyond prior tasks.

- [ ] **Step 1: Run S16 dogfood block**

Run:

```powershell
cargo build -p garnet-lsp --release
cargo test -p garnet-lsp --no-fail-fast
Set-Location editors\vscode
npm install
npm run package
Set-Location ..\..
python3 scripts/smoke_garnet_lsp_precision.py target\release\garnet-lsp.exe
```

Expected: all pass.

- [ ] **Step 2: Run inherited common checks**

Run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/garnet_conformance_matrix_check.py
python3 -m unittest discover scripts/ -p 'test_*.py'
```

Expected: all pass. If `cargo deny check` is available, run it and record output. If the binary is absent, record that it was unavailable locally and rely on CI for that gate.

- [ ] **Step 3: Open PR**

Title:

```text
S16: LSP precision features
```

Branch:

```text
agent-win-codex/s16-lsp-precision
```

PR body must use the template in `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`.

- [ ] **Step 4: Grep Loop**

After PR open, run the available PR-Agent or review tooling until the review confidence is at least `4/5`; target `5/5` if the tool supports that score. Record each review pass in the PR body or comments. Do not merge before CI is green and the review score is acceptable.

- [ ] **Step 5: Merge and ledger closeout**

After merge, append:

```markdown
- [YYYY-MM-DD HH:MM CT] MERGED PR#<number> - S16 LSP precision merged with green CI and dogfood-passing evidence.
```

## Self-Review

Spec coverage:

- PRD B workspace symbols: Task 3.
- PRD B rename: Task 4.
- PRD B code actions: Task 5.
- PRD B semantic tokens: Task 6.
- PRD B VSCode extension: Task 7.
- PRD B smoke script: Task 8.
- Dogfood and readiness reporter: Tasks 9 and 10.
- S15-Compare hold: Task 0.

Placeholder scan:

- This plan contains no implementation placeholder that authorizes vague work. The only deferred condition is the explicit S15-Compare gate from the ledger and PRD.

Type and naming consistency:

- Branch is consistently `agent-win-codex/s16-lsp-precision`.
- Readiness lane is consistently `editor_lsp_precision`.
- Smoke script is consistently `scripts/smoke_garnet_lsp_precision.py`.
