# WV-2: VM<->interp block-local scope parity (Stage-1 HIGH probe 3, PR #411) — Windows proof (2026-07-12T08:40:28Z)

- item: `WV-2` | platform: windows | host: `NUCBOX_M2PRo_S` | os: `Windows-11-10.0.26200-SP0`
- repo: `C:\garnet` @ `09d0c19ff904df694157097f4e7e08e0d620be54` (fresh clean clone, outside OneDrive tree)
- rustc: `rustc 1.95.0 (59807616e 2026-04-14)` | cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- verdict: **PASS**

| command | argv | exit | expected | ok |
|---|---|---|---|---|
| scope-shadowing-parity | `cargo test -p garnet-vm --test scope_shadowing_parity` | 0 | 0 | ✅ |

Honesty scope:

- language/runtime-trap parity evidence only - NOT OS-sandbox enforcement
- Windows AppContainer application remains named-deferred; seccomp applies on Linux only
- @bounded (Wasmtime fuel), memory limits, time limits, @mailbox remain declared-not-enforced
- MANIFEST.sha256 is a plain SHA-256 integrity seal, not a cryptographic signature
- research-grade v0.x prototype; no production or 1.0 claim; tags/releases are Jon-only
