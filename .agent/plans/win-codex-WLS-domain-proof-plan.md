# win-codex WLS Domain Proof Matrix Plan

Branch: `agent-win-codex/wls-domain-proof-matrix`

Reference context:
- `F_Project_Management/AGENT_COORDINATION_LEDGER.md` — post-S16 handoff request accepted for Windows/Linux Studio hardening.
- `F_Project_Management/GARNET_WINDOWS_LINUX_STUDIO_HANDOFF_2026_05_16.md`
- `F_Project_Management/GARNET_WINDOWS_LINUX_STUDIO_MVP_ARCHITECTURE_2026_05_17.md`
- `examples/README.md` — current executable and parser-scale example truth.
- `scripts/garnet_windows_linux_studio_status.py`
- `scripts/garnet_mit_readiness_status.py`

## Goal

Move the Windows/Linux Studio lane from broad "parse/check/run proof still needed" language to a reproducible, repo-owned domain proof matrix that can be run from Windows or Linux/WSL and opened from the Tauri shell. Keep calibrated honesty: executable examples are claimed only when the current CLI can parse, check, and run them; design drafts remain parser-scale unless the interpreter proves more.

## Scope

1. Add `scripts/smoke_garnet_studio_domain_matrix.py`.
   - Default suite: canonical MVP + agentic toolbelt + design-draft parse probes.
   - Core executable suite: `mvp_01_*` through `mvp_10_*`, `mvp_11_signed_hotreload.garnet`, and `mvp_11_signed_hotreload_mismatch.garnet`.
   - Agentic executable suite: `agent_toolbelt_01_*` through `agent_toolbelt_05_*`.
   - Design suite: `multi_agent_builder.garnet`, `agentic_log_analyzer.garnet`, `safe_io_layer.garnet` as parse-only design probes unless runtime support is present.
   - Expected-failure case passes only if `garnet run` fails with the expected BLAKE3 fingerprint mismatch.
   - Evidence bundle contains JSON, Markdown, per-command stdout/stderr files, and `MANIFEST.sha256`.
   - Bundle records `source_included=false` and `provider_api_called=false`.

2. Add meaningful tests in `scripts/test_smoke_garnet_studio_domain_matrix.py`.
   - Exercise the matrix with a fake CLI process, including expected success and expected trust-boundary failure.
   - Verify suite inventory and evidence files instead of string-only assertions.

3. Wire the matrix into Windows/Linux Studio.
   - Add a typed Tauri command for the matrix reporter.
   - Add a Release Evidence button and UI wiring.
   - Add the action to `garnet_windows_linux_studio_status.py` so it appears in machine-readable status.

4. Update readiness/status/doc surfaces conservatively.
   - Raise or refine the Windows/Linux distribution lane only if the new proof is runnable from a clean clone.
   - Keep Linux package, Windows ARM64, signing, winget, and full GUI polishing as open gates.
   - Add a changelog note under `[Unreleased]`.
   - Add a dogfood block or Windows/Linux note that reproduces the matrix.

## Verification

Focused:
- `python3 scripts/test_smoke_garnet_studio_domain_matrix.py`
- `python3 scripts/smoke_garnet_studio_domain_matrix.py --suite all`
- `python3 scripts/test_garnet_windows_linux_studio_status.py`
- `python3 scripts/test_garnet_windows_linux_studio_shell.py`
- `cargo test --manifest-path apps/garnet-studio/src-tauri/Cargo.toml --no-fail-fast`
- `npm run build` from `apps/garnet-studio`

Required before PR:
- `python3 scripts/garnet_mit_readiness_status.py`
- `cargo test --workspace --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`

Optional platform proof if time/tooling allows:
- Run the matrix under WSL Ubuntu against the Linux `target/debug/garnet` binary.
- Run Tauri `--studio-smoke` again after the new command is wired.

## Out of Scope

- No signing, winget, Linux package-format claim, or Windows ARM64 completion.
- No interpreter rewrites to make parser-scale design drafts runnable.
- No source inclusion in evidence bundles unless an explicit operator flag is added in a future PR.
