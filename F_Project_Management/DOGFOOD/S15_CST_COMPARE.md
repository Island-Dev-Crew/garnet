# S15 CST Compare — Canonical Decision

Date: 2026-05-24
Owner: Jon / Codex assist
Repository source of truth: `github.com/Island-Dev-Crew/garnet`, `origin/main`
after PR #226 (`3e45625`).

## Decision

Use the rowan-backed `garnet-cst/` crate from PR #226 as Garnet's canonical
CST for v0.7 and S16. Keep #221's in-parser CST
(`garnet-parser-v0.3/src/cst.rs`) temporarily as a legacy compatibility oracle
until the LSP is migrated and green against rowan.

This is not a winner-take-all deletion. The best part of #221 is its simple
LSP-facing token surface: `TokenKind` payloads paired with byte `Span`s. That
surface is now preserved in `garnet-cst/src/tokens.rs` as `TokenInfo`,
`token_infos`, `token_kind`, `token_span`, and `identifier_spans`, with a corpus
parity test proving rowan token metadata matches #221's parser CST token stream
(except the parser's zero-width EOF sentinel).

## Side-by-side Findings

| Surface | #221 in-parser CST | #226 rowan `garnet-cst` |
|---|---|---|
| Construction | Parses AST first, then overlays AST spans and raw lexer tokens into `CstNode` intervals. | Direct recursive-descent over the trivia-preserving token stream. |
| Valid-source dependency | Requires `parse_source_cst` to parse the AST successfully before a CST exists. | Builds a source-preserving CST for any input that lexes; lex failures are preserved under an error leaf. |
| Round-trip | Byte-identical on parser-crate and workspace examples that parse. | Byte-identical on the corpus plus arbitrary UTF-8 proptest inputs; malformed structure still round-trips. |
| Structure depth | Coarse AST-projection kinds such as `Stmt` and `Expr`; useful but not a syntax tree foundation. | Rich `SyntaxKind` coverage for items, statements, types, patterns, blocks, and the full expression tower. |
| Editor ergonomics | Strong: `CstToken { kind: TokenKind, span: Span }` is easy for rename and semantic tokens. | Strong after reconciliation: `TokenInfo` and identifier helpers expose the same payload/span view on top of rowan. |
| Rowan/incremental path | None; bespoke tree structs. | Built on rowan green nodes, suitable for editor tooling and future incremental work. |
| AST migration | Already paired with `parse_source_cst`, but only by construction from AST. | `cst_to_ast` span-normalized parity with `parse_source` across the corpus. |
| Current consumers | `garnet-lsp` still uses it for #221-era rename/semantic-token behavior. | Canonical target for S16 and new `garnet parse --mode cst` CLI surface. |

## What To Keep From Each

- Keep from #226: rowan tree, direct token-stream parser, rich syntax kinds,
  malformed-source round-trip, `cst_to_ast` parity, benchmarked performance,
  and the `garnet-cst` crate boundary.
- Keep from #221: the proven LSP consumer shape where token payloads and byte
  spans are available without forcing editor code to reason about raw rowan
  ranges.
- Do not keep: a permanent second canonical CST. #221 remains source-present
  only as a migration oracle until S16 has rowan-backed LSP coverage.

## Quantified Deltas (mac-opus verification, 2026-05-24)

Independently confirmed by the rowan-crate author on the
`codex/s15-cst-compare-rowan-canonical` branch:

- **Node-kind granularity: 25 → 202.** #221's `CstNodeKind` has 25 variants,
  with `Stmt` and `Expr` as catch-alls (a call, a binary op, and an `if` are all
  just `Expr`). Rowan's `SyntaxKind` has 202 variants — every token plus
  fine-grained item/statement/expression/pattern/type kinds — which is what LSP
  semantic tokens, rename precision, and code actions need to discriminate on.
- **Parse cost: ≈0.99× the AST path.** The `parse_cst_vs_ast` Criterion bench
  over the `mvp_*` corpus measures the rowan CST path at ≈115 µs vs the AST path
  at ≈116 µs — i.e., no meaningful overhead, well under the 1.5× S15 gate. So the
  richer tree is not paid for in parse time, and there is no perf reason to
  retain #221's AST-projection construction.
- **Reconciliation verified green.** Re-ran `cargo test -p garnet-cst` on this
  branch: roundtrip, `cst_to_ast` parity, typed-node, and the
  `parser_cst_token_parity` oracle all pass — the token surface ported into
  `garnet-cst/src/tokens.rs` matches #221's parser-CST token stream (modulo the
  parser's zero-width EOF sentinel).

## Verification

Commands run from `/Users/IDC2.5/Desktop/Garnet`:

```sh
cargo test -p garnet-cst --no-fail-fast
cargo test -p garnet-parser --test cst_round_trip --no-fail-fast
cargo test -p garnet-lsp --no-fail-fast
cargo test -p garnet-cst --test parser_cst_token_parity --no-fail-fast
cargo test -p garnet-cli --test cli_smoke --no-fail-fast
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
cargo deny check
python3 scripts/check-agent-contracts.py
python3 scripts/test_check_agent_contracts.py
python3 scripts/test_garnet_mit_readiness_status.py
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```

Observed result: all commands above passed locally after Cargo/Rust installation
on this workstation. `cargo deny check` emitted pre-existing warning-level
duplicate/license-allowance notices and exited 0.

## Follow-up Contract

1. S16 targets `garnet-cst` as canonical and migrates LSP rename/semantic-token
   collection to the rowan token helpers.
2. `garnet parse --mode cst <file>` routes to `garnet_cst::parse_cst`; default
   `garnet parse <file>` remains AST mode.
3. #221's `garnet-parser-v0.3/src/cst.rs` is not deleted in this change. Remove
   or deprecate it only after rowan-backed LSP migration is green and a follow-up
   ledger entry says the legacy oracle is no longer needed.
