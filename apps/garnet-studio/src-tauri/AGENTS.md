# AGENTS.md - Windows/Linux Tauri Studio Shell Contract

## Scope

This directory owns the Rust backend for the Windows/Linux Studio MVP. It is a
Tauri v2 shell around existing Garnet surfaces, not a fork of the converter,
checker, parser, or macOS SwiftUI Studio implementation.

## Stable Contracts

- Keep the Windows/Linux Studio a thin wrapper over the Garnet CLI, repo Python
  status/advisory scripts, docs/PWA truth, and agentic dogfood matrix.
- Active conversion stays source-to-Garnet for Rust, Ruby, Python, and Go only.
- Advisory actions stay separate for JavaScript, TypeScript, Swift, Java, C,
  C++, C#, Perl, Kotlin, Shell, SQL, and Other.
- Do not add a provider API call path, provider credential path, or network
  handoff from this shell.
- Do not enable default source inclusion in advisory bundles or handoff
  packets.
- Do not mark advisory output safe. Human review, `garnet check`, dogfood
  evidence, and sandbox lineage remain required gates.
- Keep Tauri permissions minimal. Do not add shell/open/file-system permissions
  unless the command contract, tests, and security review are updated in the
  same change.
- Evidence for Windows/Linux Studio actions belongs under the Desktop dogfood
  root named `garnet-studio-windows-linux`.
- The Release / Readiness commands are wrappers around repo reporters only:
  Windows/Linux Studio status, converter fit/provider-options status,
  MIT objective/demo/deck evidence, Mac continuation boundaries, proof and
  benchmark status, benchmark no-run compile evidence, and notarization
  preflight status. They must not upgrade platform, package, provider, proof,
  or notarization claims without separate target-system evidence.

## Required Checks

Run these checks after changing this Tauri backend or its command contract:

```sh
cargo fmt --manifest-path apps/garnet-studio/src-tauri/Cargo.toml -- --check
cargo test --manifest-path apps/garnet-studio/src-tauri/Cargo.toml
python scripts/test_garnet_windows_linux_studio_shell.py
python scripts/test_garnet_windows_linux_studio_status.py
```

For release-impacting changes, also run the frontend build, `--studio-smoke`,
and the relevant dogfood matrix/status checks from the repository root.
