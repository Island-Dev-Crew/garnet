# WV-3: truth-gate + verify examples, green AND fail-closed (Stage-1 HIGH probe 1, PR #409) — Windows proof (2026-07-12T08:54:09Z)

- item: `WV-3` | platform: windows | host: `NUCBOX_M2PRo_S` | os: `Windows-11-10.0.26200-SP0`
- repo: `C:\garnet` @ `08dea0c2a9ceb8e0aaa5cabfdf00c56f8d4a2375` (fresh clean clone, outside OneDrive tree)
- rustc: `rustc 1.95.0 (59807616e 2026-04-14)` | cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- verdict: **PASS**

| command | argv | exit | expected | ok |
|---|---|---|---|---|
| xtask-unit-tests | `cargo test -p xtask` | 0 | 0 | ✅ |
| truth-check-clean | `cargo run -q -p xtask -- truth --check` | 0 | 0 | ✅ |
| build-garnet-cli | `cargo build -p garnet-cli` | 0 | 0 | ✅ |
| verify-examples | `C:\garnet\target\debug\garnet.exe verify examples` | 0 | 0 | ✅ |
| failclosed-truth-field-drift | `C:\Program Files\Git\usr\bin\bash.exe -c sed -i 's/"primitive_count": 80/"primitive_count": 81/' docs/truth.json && cargo run -q -p xtask -- truth --check; rc=$?; git checkout -- docs/truth.json; exit $rc` | 1 | nonzero | ✅ |
| failclosed-verify-broken-source | `C:\Program Files\Git\usr\bin\bash.exe -c d=$(mktemp -d) && printf 'def broken( {{{\n' > "$d/bad.garnet" && target/debug/garnet.exe verify "$d"; rc=$?; rm -rf "$d"; exit $rc` | 1 | nonzero | ✅ |

Honesty scope:

- language/runtime-trap parity evidence only - NOT OS-sandbox enforcement
- Windows AppContainer application remains named-deferred; seccomp applies on Linux only
- @bounded (Wasmtime fuel), memory limits, time limits, @mailbox remain declared-not-enforced
- MANIFEST.sha256 is a plain SHA-256 integrity seal, not a cryptographic signature
- research-grade v0.x prototype; no production or 1.0 claim; tags/releases are Jon-only
- xtask truth --check is porcelain-insensitive by design: the -dirty suffix applies to the measurement commit recorded when stamping, not to the 6 compared fields; fail-closed for --check means field drift, proven below
- fail-closed probes run via the full Git Bash path (C:\Program Files\Git\usr\bin\bash.exe): bare 'bash' resolves to the System32 WSL wrapper on Windows, which did not faithfully propagate 'exit $rc' chains (recorded NUC-lane finding)
