# S53 Plan — tree-sitter grammar

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S53.
Map: reconciled plan §33-37, §155 — tree-sitter is adoption infra (editor
highlighting), distinct from the LSP semantic service (S44).
Branch: `codex/s53-tree-sitter`. Base: `origin/main` @ `e41491d` (S52).

## Tooling reality → honest-partial
tree-sitter CLI ABSENT; node/npm PRESENT. ⇒ author real grammar.js + validate
STRUCTURE with node; do NOT `tree-sitter generate` / corpus-test (deferred).

## Deliverables
- `tree-sitter-garnet/grammar.js`: core grammar (functions + @-annotations,
  struct/enum/impl, actors + memory kinds, control flow, match, try/rescue/
  ensure, expressions incl. |> pipe, # / /// comments).
- `tree-sitter-garnet/package.json` + `README.md` (generate/test instructions +
  honest-partial note).
- `scripts/garnet_tree_sitter_check.py`: node-backed structural check (load
  grammar.js with a `grammar()` shim; assert name + core rules). `--gate` fails
  on a dropped rule; node-absent → no-op (presence only). `--format md|json`.
- `scripts/test_garnet_tree_sitter_check.py`: 5 unit tests.
- Wire test + `--gate` into ci.yml agent-contracts.

## Dogfood
- `garnet_tree_sitter_check.py --format md` → name garnet, 50 rules, no missing
  core rules; `--gate` exits 0.

## End-state / gates
- Full ladder green (zero Rust changed; workspace 0 failed). CHANGELOG + contract
  S53 block. Ledger: `s52 → merged(5)` advanced this branch; `s53` rides with S54.

## Honest scope (do not soften)
- CORE grammar (not exhaustive); structurally validated, NOT compiled (CLI
  absent). garnet-parser remains the canonical source of truth. No new lane.
