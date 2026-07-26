# W-PLAY committed-package verification — host macOS arm64 — 2026-07-17

- Exact clean head: `978b93765eb9f930979ba0da17b6451850155d1a`.
- Command: `python3 scripts/build_playground_wasm.py --verify-reproducible`
  with the exact temporary toolchain recorded in evidence 81.
- Result: `reproducible=true`.

The verifier independently built the exact package twice from the clean head,
required the two payloads to be byte-identical, and then required every byte to
equal the committed three-file package.

```json
{"artifacts":{"garnet_wasm.js":{"bytes":6581,"sha256":"bf72509961525b4eb2e0702f41da61b4e8087ceee98787b7a046d83a85791a6d"},"garnet_wasm_bg.wasm":{"bytes":2215266,"sha256":"4df19554877167c63fe683e7584a4d31cf10f9f19c2aae576870d00099febd3c"}},"build":{"profile":"release","target":"web","wasm_opt":false},"build_parent_commit_observed":"978b93765eb9f930979ba0da17b6451850155d1a","mode":"verify-reproducible","reproducible":true,"schema":"garnet.playground.wasm-build/1"}
```

This local macOS arm64 verification is repair evidence, not the replacement
PR's cross-OS acceptance evidence.
