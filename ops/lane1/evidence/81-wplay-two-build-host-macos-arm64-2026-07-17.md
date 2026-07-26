# W-PLAY deterministic rematerialization — host macOS arm64 — 2026-07-17

This is local repair evidence for Lane 1 blocker F1.  It preserves Lane 2A's
package method without weakening `garnet_wasm_readiness.py`.

- Build parent: `ae4a842605ab7130ffa2884f285b2e7a17560712`.
- Host: macOS arm64.
- Command: `python3 scripts/build_playground_wasm.py --materialize`, with an
  explicit temporary `PATH` containing Node 22.22.2, wasm-pack 0.15.0,
  wasm-bindgen-cli 0.2.118, and the repository's Rust 1.95 toolchain.
- Contract: the command snapshots a clean tracked tree, builds the exact
  three-file package twice in separate temporary Cargo targets, checks every
  byte with `require_identical_payloads`, rechecks the repository/tool snapshot
  after each build, and publishes only after both payloads match.
- Result: `reproducible=true`; two clean builds were byte-identical.

Exact builder output:

```json
{"artifacts":{"garnet_wasm.js":{"bytes":6581,"sha256":"bf72509961525b4eb2e0702f41da61b4e8087ceee98787b7a046d83a85791a6d"},"garnet_wasm_bg.wasm":{"bytes":2215266,"sha256":"4df19554877167c63fe683e7584a4d31cf10f9f19c2aae576870d00099febd3c"}},"build":{"profile":"release","target":"web","wasm_opt":false},"build_parent_commit_observed":"ae4a842605ab7130ffa2884f285b2e7a17560712","mode":"materialize","reproducible":true,"schema":"garnet.playground.wasm-build/1"}
```

Published package SHA-256 values:

```text
bf72509961525b4eb2e0702f41da61b4e8087ceee98787b7a046d83a85791a6d  docs/playground/pkg/garnet_wasm.js
4df19554877167c63fe683e7584a4d31cf10f9f19c2aae576870d00099febd3c  docs/playground/pkg/garnet_wasm_bg.wasm
81476add10c69abaa03dff8d31ef162a6d4ede7f5f05bd702a31e479dcf6f907  docs/playground/pkg/provenance.json
```

The refreshed provenance binds:

- canonical `Cargo.lock` SHA-256:
  `01b8986b1cee0ef6a53ac439bd018b54fc1dca825a8f845a259ed8001e6715fa`;
- 175 canonical source inputs with aggregate SHA-256
  `850cc02753dde0a7de3f89cd22d187e97d8641c9df4bdaad6d6387175d198d8c`;
- Rust/Cargo 1.95.0, Node 22.22.2, wasm-pack 0.15.0, and esbuild 0.25.12
  with their binary identities in `provenance.json`.

The first setup attempt correctly failed before publication because offline
wasm-pack could not find a local lock-matched wasm-bindgen executable.  After
provisioning wasm-bindgen-cli 0.2.118 in the temporary toolchain, the clean
two-build materialization above passed.  No failed-attempt bytes were
published.
