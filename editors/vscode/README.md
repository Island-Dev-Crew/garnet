# Garnet VSCode Extension

This is the VS Code extension for Garnet's S16 LSP surface.

It launches `garnet-lsp` and enables diagnostics, hover, go-to-definition, document/workspace symbols, CST-precise rename, rules-based quick fixes, and semantic tokens for `.garnet` files (the S16 surface; see `docs/status.html` Editor/LSP Adoption for the evidence table). The extension first checks `garnet.lsp.path`, then `GARNET_LSP`, then the bundled `server/garnet-lsp` inside the installed extension, then `target/release/garnet-lsp` inside the open workspace, then `garnet-lsp` on `PATH`.

Honest boundaries (current):

- incremental/error-recovery parsing is deferred (queued post-W-REBUILD, feeds the playground band)
- Marketplace/OpenVSX publication remains open — the VSIX is locally packaged
- the extension is a thin launcher; feature truth lives in `garnet-lsp` and its tests

Package locally from this directory:

```bash
cargo build -p garnet-lsp --release
npm install
npm run package
```

The package script copies the locally built release server into `server/` before creating the VSIX. That keeps local dogfood installs self-contained while leaving published VSIX distribution proof separate until the release gate has a published package artifact.
