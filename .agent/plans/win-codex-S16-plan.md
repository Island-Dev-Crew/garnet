# S16 Rowan LSP Precision Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move Garnet LSP precision from the legacy #221 parser CST to the canonical rowan `garnet-cst` crate, then harden workspace symbols, rename, code actions, semantic tokens, VS Code packaging, and dogfood proof for S16.

**Architecture:** Keep AST semantics from `garnet_cst::cst_to_ast(parse.syntax())`, and use rowan token helpers (`token_infos`, `identifier_spans`) for trivia-preserving edit spans. The legacy `garnet_parser::parse_source_cst` path remains untouched as the migration oracle; S16 removes its use from `garnet-lsp` only after rowan-backed tests are green.

**Tech Stack:** Rust `garnet-lsp`, rowan-backed `garnet-cst`, existing `garnet-check-v0.3::suggest` rules, VS Code `vscode-languageclient`, Python stdlib LSP smoke harness.

---

## Execution Status

Status as of 2026-05-24 18:11 CDT: implemented locally on
`agent-win-codex/s16-rowan-lsp-precision` and ready for PR review. The detailed
checklist below is the original work plan; completion evidence is recorded in
the S16 dogfood block and in the PR validation summary.

Fresh proof after the final rename-scope fix:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --no-fail-fast`
- `python3 scripts/garnet_mit_readiness_status.py --check-no-regression`
- `python3 scripts/garnet_conformance_matrix_check.py`
- `wsl -d Ubuntu -- sh -lc "python3 -m unittest discover scripts/ -p 'test_*.py'"` from the worktree
- `cargo build -p garnet-lsp --release`
- `(cd editors/vscode && npm install && npm run package)`
- `python3 scripts/smoke_garnet_lsp_precision.py`
- `python3 scripts/smoke_garnet_lsp_protocol.py target\release\garnet-lsp.exe`

Known harness note: Windows-native `python3 -m unittest discover scripts/ -p
'test_*.py'` still has pre-existing POSIX assumptions (`shasum`, executable
bits, and Bash path handling). The same script suite passes under Ubuntu WSL.

---

## Required Context

References read before the plan:

- `F_Project_Management/AGENT_COORDINATION_LEDGER.md`: S16 is now unblocked; S15-Compare selected rowan `garnet-cst`.
- `F_Project_Management/PRD_B_S16_LSP_PRECISION.md`: owned crates are `garnet-lsp` and `editors/vscode`; read-only crates include `garnet-cst`, `garnet-parser-v0.3`, and `garnet-check-v0.3`.
- `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`: S16 must preserve #221 as a migration oracle until rowan-backed coverage is green.

Baseline on branch `agent-win-codex/s16-rowan-lsp-precision` from `origin/main` `09d6703`:

- `python3 scripts/garnet_mit_readiness_status.py`: 78.8%.
- `cargo test --workspace --no-fail-fast`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.

## File Map

- Modify `garnet-lsp/Cargo.toml`: add `garnet-cst` as an owned-crate dependency for the LSP server.
- Modify `garnet-lsp/src/lib.rs`: replace legacy parser CST imports/usages with rowan CST helpers; add focused helpers for symbol spans, scoped rename, semantic token classification, and code-action edits.
- Modify `garnet-lsp/src/main.rs` only if the server startup path needs no-op metadata cleanup; otherwise leave it untouched.
- Modify `editors/vscode/package.json`: declare S16 commands, bump the VSIX package version, and package to an S16 filename.
- Modify `editors/vscode/src/extension.ts`: register commands that invoke server code actions from the active editor.
- Create `scripts/smoke_garnet_lsp_precision.py`: S16 dogfood smoke that validates initialize capabilities, document symbols, workspace symbols, cross-file rename, three code actions, and semantic-token categories.
- Modify `scripts/garnet_mit_readiness_status.py`: add or refine the `editor_lsp_precision` lane only in the S16 section.
- Modify `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`: update only the S16 state/evidence block.
- Modify `CHANGELOG.md`: append one S16 bullet under `[Unreleased]`.
- Modify `F_Project_Management/AGENT_COORDINATION_LEDGER.md`: append `PR-OPEN`, `REVIEW`, and `MERGED` entries as the PR moves.

## Task 1: Rowan CST Ingestion In LSP

**Files:**
- Modify: `garnet-lsp/Cargo.toml`
- Modify: `garnet-lsp/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Add tests that prove analysis and rename use rowan token spans, not the legacy parser CST:

```rust
#[test]
fn rowan_identifier_spans_drive_rename_sites() {
    let source = "def greet(name) {\n  greet(name)\n}\n";
    let parse = garnet_cst::parse_cst(source);
    let spans = garnet_cst::identifier_spans(parse.syntax(), "greet");

    assert_eq!(spans.len(), 2);
    assert_eq!(&source[spans[0].start..spans[0].end()], "greet");
    assert_eq!(&source[spans[1].start..spans[1].end()], "greet");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p garnet-lsp rowan_identifier_spans_drive_rename_sites -- --exact`

Expected: fails to compile because `garnet-lsp` does not depend on `garnet-cst`.

- [ ] **Step 3: Add the minimal dependency and adapter**

Add to `garnet-lsp/Cargo.toml`:

```toml
garnet-cst = { version = "0.1.0", path = "../garnet-cst" }
```

In `garnet-lsp/src/lib.rs`, replace legacy CST imports with rowan helpers:

```rust
use garnet_cst::{cst_to_ast, identifier_spans, parse_cst, token_infos, TokenInfo};
```

Add:

```rust
fn parse_rowan(source: &str) -> (Module, garnet_cst::SyntaxNode) {
    let parsed = parse_cst(source);
    let module = cst_to_ast(parsed.syntax());
    (module, parsed.root)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p garnet-lsp rowan_identifier_spans_drive_rename_sites -- --exact`

Expected: pass.

## Task 2: Precision Rename Surface

**Files:**
- Modify: `garnet-lsp/src/lib.rs`

- [ ] **Step 1: Write failing tests**

Add tests for function rename, parameter-local rename, and cross-file edit planning:

```rust
#[test]
fn rename_function_uses_rowan_identifier_tokens_only() {
    let source = "/// greet docs\ndef greet(name) {\n  greet(name)\n}\n";
    let edits = rename_text_edits_for_source(source, "greet", "hello");

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "hello");
    assert_eq!(edits[1].new_text, "hello");
}

#[test]
fn rename_parameter_stays_inside_declaring_function() {
    let source = "def greet(name) {\n  name\n}\n\ndef other(name) {\n  name\n}\n";
    let edits = rename_scoped_text_edits(source, "name", "person", position_for(source, "greet(name)"));

    assert_eq!(edits.len(), 2);
    for edit in edits {
        assert!(edit.range.start.line < 3);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p garnet-lsp rename_function_uses_rowan_identifier_tokens_only rename_parameter_stays_inside_declaring_function`

Expected: tests fail because helper functions do not exist and current rename is unscoped.

- [ ] **Step 3: Implement minimal rowan rename helpers**

Implement helpers that:

- call `parse_cst(source)` once per document,
- use `identifier_spans(parsed.syntax(), name)` for trivia-preserving token spans,
- classify a target as a parameter when the target span lies inside a function parameter span,
- restrict parameter renames to that function span,
- keep function/type/global renames workspace-folder scoped by the open document map.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p garnet-lsp rename_`

Expected: pass.

## Task 3: Code Actions

**Files:**
- Modify: `garnet-lsp/src/lib.rs`

- [ ] **Step 1: Write failing tests**

Add tests for all three S16 actions:

```rust
#[test]
fn code_action_add_caps_inserts_before_existing_docs_and_def() {
    let source = "/// entry point\ndef main() {\n}\n";
    let edit = add_caps_edit(source, find_function_in_module(&analyze_module(source), "main").unwrap());

    assert_eq!(edit.new_text, "@caps()\n");
    assert_eq!(edit.range.start, Position::new(1, 0));
}

#[test]
fn code_action_refactor_long_parameter_list_offers_struct_scaffold() {
    let source = "def build(a, b, c, d) {\n  a\n}\n";
    let actions = code_actions_for_source(source);

    assert!(actions.iter().any(|a| a.title.contains("Refactor long parameter list")));
}

#[test]
fn code_action_add_return_type_infers_literal_int() {
    let source = "def answer() {\n  42\n}\n";
    let actions = code_actions_for_source(source);

    assert!(actions.iter().any(|a| a.title.contains("Add return type `Int`")));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p garnet-lsp code_action_`

Expected: long-parameter and return-type action tests fail.

- [ ] **Step 3: Implement actions**

Reuse `garnet_check::suggest::suggest_for_module` for:

- `Rule::ManagedFnMissingCaps` → insert `@caps()` at the function span start.
- `Rule::LongParameterList` → add a quick-fix action that rewrites `def f(a, b, c, d)` to `def f(options)` and inserts `struct FParams { a: Any, b: Any, c: Any, d: Any }` above the function.

Implement local return-type detection without modifying `garnet-check-v0.3`:

- skip functions with `return_ty.is_some()`,
- infer `Int`, `Float`, `String`, `Bool`, or `Nil` from the tail expression when it is a literal,
- insert ` -> Type` immediately before the function body `{`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p garnet-lsp code_action_`

Expected: pass.

## Task 4: Semantic Tokens

**Files:**
- Modify: `garnet-lsp/src/lib.rs`

- [ ] **Step 1: Write failing tests**

Add tests proving the S16 categories exist and rowan tokens feed the classifier:

```rust
#[test]
fn semantic_legend_exposes_s16_categories() {
    let legend = semantic_token_legend();
    let names: Vec<_> = legend.token_types.iter().map(|t| t.as_str()).collect();

    assert!(names.contains(&"keyword"));
    assert!(names.contains(&"function"));
    assert!(names.contains(&"parameter"));
    assert!(names.contains(&"capability"));
    assert!(names.contains(&"attribute"));
}

#[test]
fn semantic_tokens_classify_caps_as_capability_and_attribute() {
    let source = "@caps(fs)\ndef main() {\n  fs::read_file(\"x\")\n}\n";
    let classified = classify_semantic_tokens(source);

    assert!(classified.iter().any(|t| t.text == "caps" && t.kind == "attribute"));
    assert!(classified.iter().any(|t| t.text == "fs" && t.kind == "capability"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p garnet-lsp semantic_`

Expected: fails because the legend does not expose `parameter`, `capability`, or `attribute` as separate token types.

- [ ] **Step 3: Implement semantic classification**

Create a stable legend:

```rust
const TOKEN_KEYWORD: u32 = 0;
const TOKEN_FUNCTION: u32 = 1;
const TOKEN_TYPE: u32 = 2;
const TOKEN_PARAMETER: u32 = 3;
const TOKEN_COMMENT: u32 = 4;
const TOKEN_STRING: u32 = 5;
const TOKEN_NUMBER: u32 = 6;
const TOKEN_OPERATOR: u32 = 7;
const TOKEN_CAPABILITY: u32 = 8;
const TOKEN_ATTRIBUTE: u32 = 9;
```

Use `token_infos(parse.syntax())` and previous non-trivia token context to classify names after `@` as `attribute`, names inside `@caps(...)` as `capability`, function declarations as `function`, parameter-list names as `parameter`, and type declarations/annotations as `type`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p garnet-lsp semantic_`

Expected: pass.

## Task 5: VS Code Extension Commands

**Files:**
- Modify: `editors/vscode/package.json`
- Modify: `editors/vscode/src/extension.ts`

- [ ] **Step 1: Write failing package/compile check**

Run: `cd editors/vscode && npm run compile`

Expected before edits: pass, but no S16 commands are contributed.

- [ ] **Step 2: Add commands and version bump**

In `package.json`, bump `"version"` to `"0.7.0"` and change package output to `garnet-0.7.0-lsp-precision.vsix`.

Add command contributions:

```json
"commands": [
  { "command": "garnet.addCapsAnnotation", "title": "Garnet: Add @caps annotation" },
  { "command": "garnet.refactorLongParameterList", "title": "Garnet: Refactor long parameter list" },
  { "command": "garnet.addReturnTypeAnnotation", "title": "Garnet: Add return type annotation" }
]
```

In `extension.ts`, register commands that call:

```ts
vscode.commands.executeCommand('editor.action.codeAction', {
  kind: vscode.CodeActionKind.QuickFix.value,
  apply: 'ifSingle'
});
```

- [ ] **Step 3: Run compile and package smoke**

Run: `cd editors/vscode && npm install && npm run package`

Expected: VSIX generated with the S16 filename.

## Task 6: Precision Smoke And Readiness Evidence

**Files:**
- Create: `scripts/smoke_garnet_lsp_precision.py`
- Modify: `scripts/garnet_mit_readiness_status.py`
- Modify: `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Write failing smoke**

Create `scripts/smoke_garnet_lsp_precision.py` by adapting the protocol smoke and adding assertions for:

- initialize advertises document symbols, workspace symbols, rename, code actions, and semantic tokens,
- two open documents allow cross-file function rename,
- parameter rename stays in one function,
- all three code actions are returned,
- semantic token legend includes `capability` and `attribute`,
- semantic token data is non-empty.

Run: `python3 scripts/smoke_garnet_lsp_precision.py`

Expected: fails until Tasks 1-5 land.

- [ ] **Step 2: Add readiness lane detail**

Update only the S16/editor LSP portion of `scripts/garnet_mit_readiness_status.py` so the lane distinguishes:

- legacy #221 LSP adoption,
- rowan-backed S16 precision migration,
- VSIX packaging evidence,
- manual screenshot evidence still needed before merge.

- [ ] **Step 3: Update docs with calibrated honesty**

Update only the S16 block in `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`:

- state moves to `review-ready` only after dogfood passes,
- honest partials stay verbatim: managed-mode only, single-folder rename only, cross-package rename v0.8.

Append to `CHANGELOG.md` under `[Unreleased]`:

```markdown
- S16: migrated LSP precision features to the canonical rowan `garnet-cst` surface, with managed-mode workspace symbols, single-folder rename, three quick fixes, semantic tokens, VS Code commands, and a reproducible precision smoke.
```

- [ ] **Step 4: Run S16 dogfood block**

Run:

```powershell
cargo build -p garnet-lsp --release
cargo test -p garnet-lsp --no-fail-fast
Push-Location editors\vscode; npm install; npm run package; Pop-Location
python3 scripts/smoke_garnet_lsp_precision.py
```

Expected: all pass.

## Task 7: Final Verification, PR, Review Loop, Merge

**Files:**
- Modify: `F_Project_Management/AGENT_COORDINATION_LEDGER.md`

- [ ] **Step 1: Full local verification**

Run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/garnet_conformance_matrix_check.py
python3 -m unittest discover scripts/ -p 'test_*.py'
```

Expected: all pass.

- [ ] **Step 2: Commit and push**

Commit message:

```text
S16: rowan-backed LSP precision
```

Push branch `agent-win-codex/s16-rowan-lsp-precision`.

- [ ] **Step 3: Open PR**

Open PR title:

```text
S16: rowan-backed LSP precision
```

Use the PR body template from `GARNET_v0_7_SLICE_DOGFOOD.md`, with exact local dogfood output summarized and honest partials preserved.

Append ledger entry:

```text
- [YYYY-MM-DD HH:MM CDT] PR-OPEN PR#<n> agent-win-codex/s16-rowan-lsp-precision — rowan-backed LSP precision dogfood passing locally; CI running.
```

- [ ] **Step 4: Grep Loop and review**

Run PR-Agent/Grep Loop until confidence is at least 5/5. Fix any valid findings with TDD. Append `REVIEW` once CI and review are clean.

- [ ] **Step 5: Merge and close ledger**

After green CI and 5/5 review confidence, merge the PR. Append:

```text
- [YYYY-MM-DD HH:MM CDT] MERGED PR#<n> agent-win-codex/s16-rowan-lsp-precision — S16 rowan-backed LSP precision merged; v0.7 release gate now waits on remaining slices.
```

## Self-Review

- Spec coverage: every PRD B feature maps to a task: workspace/document symbols are preserved and smoke-tested; rename is rowan-backed and scoped; code actions include all three S16 actions; semantic tokens expose S16 categories; VS Code commands/package are updated; dogfood script and readiness lane are updated.
- Placeholder scan: no implementation task uses an unspecified file or vague action.
- Type consistency: all Rust snippets use existing `Position`, `TextEdit`, `Rule`, `Module`, and rowan CST helper names verified from the current source tree.
