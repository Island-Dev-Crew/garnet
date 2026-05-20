# S1 Plan: LSP MVP

## Requirements Summary

Contract section: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` -> `### S1 - LSP MVP`.

Current checkout note: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` is absent on `main` at the start of this slice. The user-provided contract for S1 is therefore the planning source until the repo copy is added or a newer contract appears.

S1 goal: add a Garnet Language Server Protocol MVP for managed-mode editor adoption. The required capabilities are diagnostics, hover, and go-to-definition in VSCode. New surfaces are `garnet-lsp/`, `editors/vscode/`, and the workspace `Cargo.toml` update.

Slice states from the supplied v0.5 contract at baseline:

| Slice | State | v0.5.0 relevance |
| --- | --- | --- |
| S1 - LSP MVP | not-started | v0.5.0 blocking |
| S2 - Bytecode VM Scaffold | not-started | v0.5.0 blocking |
| S3 - `garnet add` + Manifest Spec | not-started | v0.5.1 acceptable |
| S4 - `garnet fmt` | not-started | v0.5.1 acceptable |
| S5 - Parser Fuzz Harness | not-started | v0.5.0 blocking |
| S6 - Memory Eviction Policy Benchmarks | not-started | v0.5.1 acceptable |
| S7 - Actor OS-Thread Bridge | not-started | v0.5.1 acceptable |
| S8 - Signed Hot-Reload Demo | not-started | v0.5.0 blocking |
| S9 - Determinism CI Cross-Machine | not-started | v0.5.0 blocking |
| S10 - Compiler-as-Agent Advisory Mode | not-started | v0.5.0 blocking |

Baseline verification before planning:

- `python3 scripts/garnet_mit_readiness_status.py`: active-partial, 57.9%.
- `python3 scripts/garnet_converter_status.py`: Rust/Ruby/Python/Go active; planned languages and LLM assist remain advisory-only.
- `python3 scripts/garnet_proof_benchmark_status.py`: active-scaffold; benchmark measurements not run, mechanized proof absent, empirical study pending.
- `cargo test --workspace --no-fail-fast`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: pass.

## Acceptance Criteria

- `garnet-lsp` is a new workspace crate with a binary server and testable library logic.
- The server supports:
  - `textDocument/didOpen` / `didChange` / `didSave` document tracking.
  - publish diagnostics from parser failures and checker errors.
  - hover for top-level symbols resolvable from the AST, including kind and signature; doc comments are best-effort source-text scans.
  - definition for top-level `def`, `fn`, `struct`, `enum`, `trait`, `protocol`, `actor`, `memory`, `const`, and `let` names.
- `editors/vscode` contains a minimal extension that launches `garnet-lsp`, registers Garnet files, and can be packaged as a VSIX.
- Honest partial labels are preserved in docs and PR body:
  - safe-mode hover not in MVP
  - workspace symbols deferred to S1.1
  - rename deferred
  - CST/incremental parser precision deferred
- No new OS authority is introduced in Garnet source. Any new Garnet examples or fixtures that contain functions with authority must declare `@caps(...)`.
- New dependencies are justified and verified with `cargo deny check` before the PR.

## Implementation Steps

1. Add the S1 contract file if still absent, using the user-provided `GARNET v0.5 SLICE DOGFOOD CONTRACTS` text as the daily truth source.
2. Create `garnet-lsp/` with `Cargo.toml`, `src/lib.rs`, and `src/main.rs`; add it to workspace members.
3. Implement a small source index that parses `garnet_parser::parse_source`, runs `garnet_check::check_module`, converts byte spans to LSP ranges, and indexes top-level symbols.
4. Implement LSP handlers with `tower-lsp` and `tokio` for document sync, diagnostics, hover, and definition.
5. Add focused `garnet-lsp` tests for parser diagnostics, checker diagnostics, hover content, definition locations, and line/column conversion.
6. Add `editors/vscode/` with `package.json`, TypeScript extension source, `tsconfig.json`, `.vscodeignore`, and README notes that describe MVP boundaries without overstating support.
7. Update status/current-state docs only as needed for S1: contract state to planned/in-progress/review-ready as the slice moves, CHANGELOG entry, and any reporter hook only if S1 changes the MIT readiness reporter.
8. Run the S1 verification ladder and record outputs in the PR body.

## Risks And Mitigations

- Risk: full LSP fidelity really wants a trivia-preserving CST. Mitigation: keep MVP honest and span-based; document CST/incremental precision as deferred.
- Risk: VSIX packaging may require npm tooling not already installed. Mitigation: use a minimal TypeScript extension and run `npm install && npm run package`; if local packaging is blocked by missing system tooling, document the exact blocker and do not claim VSIX proof.
- Risk: dependency policy may reject `tower-lsp` or new transitive licenses. Mitigation: run `cargo deny check` and include the summary in the PR.
- Risk: LSP diagnostics may become noisy if parser and checker errors use different location fidelity. Mitigation: prefer parser span when available, otherwise publish whole-document diagnostic with an honest message.

## Verification Steps

Required local verification:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
cargo deny check
cargo build -p garnet-lsp --release
(cd editors/vscode && npm install && npm run package)
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/garnet_conformance_matrix_check.py
```

Manual dogfood evidence required in the PR body:

```bash
code --install-extension editors/vscode/garnet-*.vsix
```

- Open `examples/mvp_01_*.garnet`, inject a syntax error, and confirm a diagnostic appears.
- Hover on `def greet` and confirm a signature panel appears.
- Go-to-def from a call site lands on the definition.

If the GUI dogfood cannot be fully performed inside this Mac session, the PR must say exactly which part was blocked and avoid claiming review-ready or dogfood-passing state.

## PR Shape

- Branch: `codex/s1-lsp-mvp`.
- PR title: `S1: LSP MVP`.
- Base repo: `Island-Dev-Crew/garnet`.
- Head repo: `Navigata1/garnet`.
- Commit message follows the Lore protocol.
- Do not tag `v0.5.0` from this slice.
