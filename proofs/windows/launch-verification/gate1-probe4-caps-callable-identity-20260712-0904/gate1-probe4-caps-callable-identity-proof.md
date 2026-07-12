# Gate-1 probe 4: capability callable-identity (PR #412) on Windows — Windows proof (2026-07-12T09:04:20Z)

- item: `Gate-1 probe 4` | platform: windows | host: `NUCBOX_M2PRo_S` | os: `Windows-11-10.0.26200-SP0`
- repo: `C:\garnet` @ `58d9aeb102c77920ef8577c33935d16850fa99bc` (fresh clean clone, outside OneDrive tree)
- rustc: `rustc 1.95.0 (59807616e 2026-04-14)` | cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- verdict: **PASS**

| command | argv | exit | expected | ok |
|---|---|---|---|---|
| caps-callable-identity | `cargo test -p garnet-check --test caps_callable_identity` | 0 | 0 | ✅ |

Honesty scope:

- language/runtime-trap parity evidence only - NOT OS-sandbox enforcement
- Windows AppContainer application remains named-deferred; seccomp applies on Linux only
- @bounded (Wasmtime fuel), memory limits, time limits, @mailbox remain declared-not-enforced
- MANIFEST.sha256 is a plain SHA-256 integrity seal, not a cryptographic signature
- research-grade v0.x prototype; no production or 1.0 claim; tags/releases are Jon-only
