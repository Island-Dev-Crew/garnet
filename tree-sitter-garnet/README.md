# tree-sitter-garnet

A [tree-sitter](https://tree-sitter.github.io/) grammar for the Garnet language —
**adoption infrastructure** for editors (syntax highlighting, folding, structural
navigation).

This is the *syntax* grammar. It is intentionally separate from the Garnet LSP
(`garnet-lsp`, S44), which is a *semantic* service running on the compiler
frontend. The canonical grammar is the hand-written parser in `garnet-parser`;
`grammar.js` here mirrors its core surface for editor tooling.

## Status (honest)

- `grammar.js` covers the **core** constructs — functions + `@`-annotations,
  struct/enum/impl, actors + `memory` kinds, control flow, `match`,
  `try/rescue/ensure`, expressions (calls, binary, `|>` pipe), `#` and `///`
  comments. It is **not** an exhaustive reproduction of every form.
- The grammar is **structurally validated in CI**
  (`scripts/garnet_tree_sitter_check.py` loads `grammar.js` with Node and checks
  the expected named rules + grammar name).
- It is **not** compiled or corpus-tested in this repo's CI — `tree-sitter
  generate` and `tree-sitter test` require the tree-sitter CLI (not present in
  the build environment).

## Building it yourself

With the tree-sitter CLI installed:

```sh
cd tree-sitter-garnet
tree-sitter generate     # emits the C parser from grammar.js
tree-sitter test         # runs the corpus (add cases under test/corpus/)
```

The generated parser then drives editor highlighting for `*.garnet` files.
