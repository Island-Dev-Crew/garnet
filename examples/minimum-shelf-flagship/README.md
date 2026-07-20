# Minimum Shelf flagship

This is Garnet's one local Core Ring Tier 1 MCP package. It exports only
`garnet.core.double`, accepts exactly `{ "value": <i64> }`, declares no
capabilities, and doubles the value inside the Garnet interpreter.

`tool.seal.json` is a deterministic in-toto predicate emitted by `garnet seal`.
It is intentionally labeled unsigned: repository trust comes from the package
bytes pinned in the Garnet binary and reviewed Git history, not from an
unavailable external signer.

The package is served only through `garnet mcp-serve --package <this-directory>`.
Missing, changed, resealed, or path-substituted package files are rejected before
the interpreter host is constructed.
