# Garnet VSCode Extension

This is the S1 LSP MVP extension for Garnet.

It launches `garnet-lsp` and enables diagnostics, hover, and basic go-to-definition for `.garnet` files. The extension first checks `garnet.lsp.path`, then `GARNET_LSP`, then `target/release/garnet-lsp` inside the open workspace, then `garnet-lsp` on `PATH`.

Honest MVP boundaries:

- safe-mode hover is not in this MVP
- workspace symbols are deferred to S1.1
- rename is deferred
- CST-grade incremental precision is deferred

Package locally from this directory:

```bash
npm install
npm run package
```
