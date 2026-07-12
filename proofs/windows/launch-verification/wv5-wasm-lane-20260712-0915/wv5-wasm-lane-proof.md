# WV-5: wasm lane build + Node smoke from clean Windows checkout (W-PLAY Task 1, PR #464) — Windows proof (2026-07-12T09:15:39Z)

- item: `WV-5` | platform: windows | host: `NUCBOX_M2PRo_S` | os: `Windows-11-10.0.26200-SP0`
- repo: `C:\garnet` @ `098260e48f5625c409d53646889f27922e08c1e9` (fresh clean clone, outside OneDrive tree)
- rustc: `rustc 1.95.0 (59807616e 2026-04-14)` | cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- verdict: **PASS**

| command | argv | exit | expected | ok |
|---|---|---|---|---|
| wasm-native-tests | `cargo test -p garnet-wasm` | 0 | 0 | ✅ |
| output-capture-tests | `cargo test -p garnet-interp --test output_capture` | 0 | 0 | ✅ |
| wasm32-build | `cargo build -p garnet-wasm --target wasm32-unknown-unknown` | 0 | 0 | ✅ |
| wasm-pack-web | `wasm-pack build garnet-wasm --target web --out-dir pkg-web` | 0 | 0 | ✅ |
| wasm-pack-nodejs | `wasm-pack build garnet-wasm --target nodejs --out-dir pkg-node` | 0 | 0 | ✅ |
| node-smoke | `node C:\Users\ISLAND~1\AppData\Local\Temp\claude\C--Users-IslandDevCrew-Desktop-Garnet-Opus-4-7-final\eee5f370-e621-4d7f-8220-180220100e8b\scratchpad\wv5_node_smoke.js` | 0 | 0 | ✅ |

Honesty scope:

- language/runtime-trap parity evidence only - NOT OS-sandbox enforcement
- Windows AppContainer application remains named-deferred; seccomp applies on Linux only
- @bounded (Wasmtime fuel), memory limits, time limits, @mailbox remain declared-not-enforced
- MANIFEST.sha256 is a plain SHA-256 integrity seal, not a cryptographic signature
- research-grade v0.x prototype; no production or 1.0 claim; tags/releases are Jon-only
- Node smoke proves wasm execution, not the browser page; the 'runs in your browser' claim waits for the Playwright trap in the W-PLAY page slice
- wasm-opt is disabled by Cargo.toml metadata - an unoptimized module is the recorded, expected outcome
- wasm-pack builds run sequentially (shared wasm-bindgen cache race avoided)
