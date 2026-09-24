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
  benchmark status, benchmark no-run compile evidence, Windows clean-VM
  installer proof status, and notarization preflight status. They must not
  upgrade platform, package, provider, proof, or notarization claims without
  separate target-system evidence.
- **Committed clean-VM proof (T5a, path (a)).**
  `scripts/garnet_windows_clean_vm_installer_status.py` reads the newest
  committed bundle under `proofs/windows/studio-clean-vm/`. The Desktop root
  is only a fallback when no bundle is committed. An explicit root wins over
  both: `--evidence-root` for the standalone reporter, which is what the
  Studio installer-status command runs. `GARNET_WINDOWS_CLEAN_VM_EVIDENCE_ROOT`
  is read only by the aggregate `scripts/garnet_windows_linux_studio_status.py`.
  A bundle counts only when all of these hold:
  - its name is `<YYYYMMDD-HHMM>-<host>`;
  - it holds only regular files, each with one link and a nonzero inode, and
    each read from the handle it was checked on;
  - `MANIFEST.sha256` lists every file once and matches;
  - its JSON is strict, and `verified` is the literal `true`;
  - every gate passes, and the facts behind the gates are present;
  - the guest is x64 and its OS name is the guest's own `systeminfo` OS name:
    Windows 10, Windows 11 or Windows Server 2016/2019/2022/2025, optionally
    prefixed `Microsoft`. A hypervisor guest-type identifier is out of
    contract, and the recorder's fresh-guest gate applies the same rule;
  - the three evidence files are distinct, manifest-verified files named
    directly inside the bundle; the install log is not empty, and the
    screenshot is a PNG with a nonzero width and height, checked by both the
    recorder and the reader;
  - no directory on the way is a link, and each one can be listed.

  A failing newest bundle is reported, never replaced by an older one.
  **Threat model:** the reader checks committed content. A process that can
  write the checkout while it runs can write a consistent bundle outright,
  because the manifest is unsigned, so that process is out of scope; review
  settles who committed a bundle.
- `mac_domain_proofs` is the macOS Studio UI wrapper for
  `scripts/smoke_garnet_mac_domain_proofs.py`. It may record local Mac S105
  domain evidence under `target/mac-studio-domain-proofs/`, but it must not
  claim Windows/Linux proof ownership, production enforcement, or v1.0 status.
- **Version stamp:** `Cargo.toml` is the single version stamp (tauri.conf.json
  must not carry a duplicate `version` field) and it must equal
  `[workspace.package].version` in the root manifest. The crate is excluded
  from the workspace, so inheritance cannot enforce this — the CI gate is the
  shell contract test (agent-contracts job, repo-wide); the crate test
  `crate_version_matches_workspace_release_version` covers the local ladder
  (`cargo test --workspace` skips this crate). Consequence for release-prep:
  a workspace version bump must bump `Cargo.toml` + `package.json` here (with
  their lockfiles) in the same PR. Never reintroduce a hand-stamped second
  version.
- **Process discipline:** every spawned command runs through
  `run_process_with_timeout`: piped + thread-drained output (no pipe
  deadlocks), per-category timeout from settings (matrix categories get the
  larger budget), kill-on-timeout reported via `timed_out`, and a per-stream
  UI payload cap. Full output is written to the evidence bundle **before**
  capping. New commands must go through this path, not raw
  `Command::output()`.
- **Settings:** `settings.rs` persists `{mode, theme, command_timeout_secs,
  matrix_timeout_secs}` as JSON under the per-user config dir. Every write is
  validated/clamped in `StudioSettings::normalized`; a corrupt or missing file
  must never block startup (defaults win).
- **Evidence readers:** `list_evidence_files` / `read_evidence_text` are
  read-only and must stay constrained by `resolve_within_evidence_roots`
  (canonicalize both sides; reject anything outside the Studio evidence
  roots; size/entry caps). Do not widen them into a general filesystem read
  primitive.
- `get_truth_summary` reads `docs/truth.json` (the RB-0a truth surface) and
  must degrade to an explicit "not found" rather than inventing values. The
  frontend must not reintroduce hand-written release statistics.
- The simple/power interface modes hide power-only panels with CSS only; the
  panels (and their explicit-copy strings) stay in the DOM so the shell
  contract test keeps asserting them.

## Required Checks

Run these checks after changing this Tauri backend or its command contract:

```sh
cargo fmt --manifest-path apps/garnet-studio/src-tauri/Cargo.toml -- --check
cargo test --manifest-path apps/garnet-studio/src-tauri/Cargo.toml
python scripts/test_garnet_windows_linux_studio_shell.py
python scripts/test_garnet_windows_linux_studio_status.py
python scripts/test_garnet_windows_clean_vm_installer_status.py
npm --prefix apps/garnet-studio run build
```

For release-impacting changes, also run the frontend build, `--studio-smoke`,
and the relevant dogfood matrix/status checks from the repository root.
