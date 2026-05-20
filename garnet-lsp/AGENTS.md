# AGENTS.md - LSP Contract

## Scope

Owns Garnet Language Server Protocol behavior and editor-facing protocol tests.

## Stable Contracts

- Diagnostics must come from the existing parser and checker; do not invent a parallel language interpretation.
- Hover and go-to-definition are MVP top-level symbol features until a CST/incremental index lands.
- Editor UX claims require either VSIX dogfood evidence or stdio protocol smoke output.
- Keep deferred surfaces explicit: safe-mode hover, workspace symbols, rename, and CST-grade incremental precision.

## Required Checks

```sh
cargo test -p garnet-lsp
cargo build -p garnet-lsp --release
python3 scripts/smoke_garnet_lsp_protocol.py target/release/garnet-lsp
```
