# WV-1: test-runner entry-authority parity (Stage-1 HIGH probe 2, PR #410) — Windows proof (2026-07-12T08:25:59Z)

- item: `WV-1` | platform: windows | host: `NUCBOX_M2PRo_S` | os: `Windows-11-10.0.26200-SP0`
- repo: `C:\garnet` @ `79290e5914aaed680017dc0e85a759527d0ecd51` (fresh clean clone, outside OneDrive tree)
- rustc: `rustc 1.95.0 (59807616e 2026-04-14)` | cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- verdict: **PASS**

| command | argv | exit | expected | ok |
|---|---|---|---|---|
| test-entry-authority | `cargo test -p garnet-cli --test test_entry_authority` | 0 | 0 | ✅ |

Honesty scope:

- language/runtime-trap parity evidence only - NOT OS-sandbox enforcement
- Windows AppContainer application remains named-deferred; seccomp applies on Linux only
- @bounded (Wasmtime fuel), memory limits, time limits, @mailbox remain declared-not-enforced
- MANIFEST.sha256 is a plain SHA-256 integrity seal, not a cryptographic signature
- research-grade v0.x prototype; no production or 1.0 claim; tags/releases are Jon-only
