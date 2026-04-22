# GARNET v4.2 — Phase 6D Linux Verification Report

**Purpose:** Document the end-to-end verification of the Linux `.deb` and `.rpm` installers. Both packages built from clean containers, installed into separate clean containers, and exercised the full v4.2 feature surface (scaffolding, Ed25519 signed manifests, signature verification).
**Date:** 2026-04-17
**Verifier:** Claude Code (Opus 4.7) in the Stage 6 Phase 6D execution session.
**Anchor:** *"By their fruits ye shall know them." — Matthew 7:20*

---

## Executive summary

**Linux passes Phase 6D's success criteria.** Both `.deb` (Debian/Ubuntu) and `.rpm` (Fedora/RHEL) packages produce a fully-functional `garnet` binary from a clean install. The v3.4.1 ManifestSig chain (keygen → deterministic build with signature → verify) round-trips cryptographically on real Linux. `garnet new` scaffolds projects from every template. No regressions vs. the hash-based verification already proven by the workspace cargo tests.

**macOS and Windows Phase 6D verification pending external infrastructure:**

- **macOS**: the user will transfer the workspace to their MacBook beside them and run `garnet-cli/macos/build-pkg.sh` with real Apple Developer ID credentials — prompt provided in §Handoff below.
- **Windows**: the user is concurrently completing the MinGW→MSVC toolchain switch per `GARNET_v4_2_STAGE6_KICKOFF.md` §"Windows toolchain blocker"; MSI build unblocks once `garnet.exe --version` no longer crashes at startup.

---

## What was actually executed

### 1. Clean build in `rust:1-bookworm` container

- Bind-mounted a staged copy of the workspace (minus `target/` and the host's MSVC-targeting `.cargo/config.toml`) at `/work`.
- `cargo build --release -p garnet-cli` — 1 m 22 s cold compile.
- `cargo install cargo-deb cargo-generate-rpm` (isolated to `CARGO_TARGET_DIR=/root/cbuild` so the crate-installs did not clobber the workspace target).
- `cargo deb --no-build` → `target/debian/garnet_0.3.0-1_amd64.deb` (1.44 MB).
- `cargo generate-rpm -p garnet-cli` → `target/generate-rpm/garnet-0.3.0-1.x86_64.rpm` (1.54 MB).

Both artifacts copied into the repository at:

- `E_Engineering_Artifacts/target/debian/garnet_0.3.0-1_amd64.deb`
- `E_Engineering_Artifacts/target/generate-rpm/garnet-0.3.0-1.x86_64.rpm`
- `E_Engineering_Artifacts/dist/linux/` (parallel copy for distribution).

### 2. `.deb` smoke test in clean `ubuntu:24.04` container

Six gates, all PASS:

| # | Gate | Result |
|---|------|--------|
| 1 | `apt-get install garnet_*.deb` succeeds | ✅ 146 CA-certificates, openssl bundled as transitive dep |
| 2 | `garnet --version` renders wordmark + 6-crate version table | ✅ Visible full banner including "CapCaps v3.4.1" + "22 bridged primitives" |
| 3 | `garnet --help` renders subcommand table including v4.2 additions | ✅ `new`, `keygen`, `build --sign`, `verify --signature`, `convert` all listed |
| 4 | `garnet new --template cli /tmp/my_app` scaffolds 5 files | ✅ `Garnet.toml`, `src/main.garnet`, `tests/test_main.garnet`, `.gitignore`, `README.md`; next-steps hint renders |
| 5 | `garnet parse /tmp/my_app/src/main.garnet` parses the generated file | ✅ `parsed /tmp/my_app/src/main.garnet (1 items, safe=false)` |
| 6 | `garnet keygen` → `build --deterministic --sign` → `verify --signature` | ✅ signer_pubkey = `75cffbb9…499`; signature matches on verify |

`apt-get remove garnet` cleans up binary + man page entry afterward.

**Shared-library footprint (via `ldd`):** `libc`, `libm`, `libgcc_s`, `linux-vdso`, `ld-linux-x86-64`. Nothing else. Every primitive (stdlib / crypto / actor runtime) statically linked. Portable across glibc ≥ 2.34 targets.

### 3. `.rpm` smoke test in clean `fedora:40` container

Same 6 gates, all PASS — with one bonus:

| # | Gate | Result |
|---|------|--------|
| 1 | `dnf install garnet-*.rpm` succeeds | ✅ |
| 2 | `garnet --version` wordmark + version table | ✅ |
| 3 | **Man page at `/usr/share/man/man1/garnet.1`** | ✅ (Ubuntu minimal containers exclude man pages by default; Fedora keeps them. Both packages ship it correctly.) |
| 4 | `garnet new --template agent-orchestrator /tmp/agents` | ✅ 5 files including the three-actor MVP 6 shape |
| 5 | `garnet keygen` → `build --deterministic --sign` → `verify --signature` | ✅ signer_pubkey = `898368da…1de`; signature matches on verify; manifest reports 4 items (one module + three actors) |
| 6 | `dnf remove garnet` cleans up | ✅ |

### 4. Universal shell installer (`installer/sh.garnet-lang.org/install.sh`)

- Linted in `koalaman/shellcheck:stable` with `--shell=sh --severity=warning`: **CLEAN**, zero issues.
- Not yet dry-run against the real `sh.garnet-lang.org` endpoint (no release hosted yet); will be exercised during the first signed-release rollout.

---

## Pass 2 (widened smoke) — additional coverage

The initial pass 1 covered the `cli` template + signed manifest. Pass 2 added the remaining v4.2 feature surface:

| Feature exercised | Result |
|-------------------|--------|
| `garnet new --template cli /tmp/sample-cli` | ✅ 5 files |
| `garnet new --template web-api /tmp/sample-web-api` | ✅ 5 files (HTTP/1.1 + `@caps(net, time)`) |
| `garnet new --template agent-orchestrator /tmp/sample-agent-orchestrator` | ✅ 5 files (Researcher/Synthesizer/Reviewer + 3 memory kinds) |
| `garnet check` on caps-clean program (`@caps(fs)` + `read_file`) | ✅ "1 functions checked, 0 diagnostics" |
| `garnet check` on caps-violating program (`@caps()` + `read_file`) | ✅ Flagged: *"caps coverage: function `main` does not declare `fs` but transitively calls `read_file` which requires it"* — this is the v3.4.1 Day 2 propagator running live on real Linux |
| `garnet convert ruby /tmp/hello.rb` | ✅ 4 artifacts: `.garnet`, `.lineage.json`, `.migrate_todo.md`, `.metrics.json`. Output starts `@sandbox @caps()` per v4.1 SandboxMode default. 100.0% clean translation, 3 CIR nodes, 0 migrate-todos. |
| `garnet test` on all 3 scaffolded templates (Phase 6E addition) | ✅ cli (2 pass), web-api (1 pass — verifies cross-file helper resolution: test calls `timestamp()` defined in src/main.garnet), agent-orchestrator (2 pass — pure helpers + bridged crypto). Per-test report with pass/fail summary; exit non-zero on failure. |

### Bug found + patched in pass 2

The `convert` subcommand was registered in `lib.rs` but **never wired into `bin/garnet.rs`'s match** — invoking `garnet convert ruby foo.rb` printed "unknown subcommand: convert". Patched in this session:

- `bin/garnet.rs` — added `"convert" => cmd_convert(&args[1..])` arm + a full `cmd_convert()` handler that mirrors `cmd_new`'s flag-parsing style. Supports positional `<lang> <file>` plus `--lang`, `--out`, `--strict`, `--fail-on-todo`, `--fail-on-untranslatable`, `--quiet` flags.

A cosmetic follow-up: `clean-translate` percentage was printed as `10000%` instead of `100.0%` (double-multiplied; fixed in source). **Landed in pass 3 rebuild** — verified output: `migrate_todo  = 0 (100.0% clean-translate)`.

A second gap closed in pass 4: `garnet test` subcommand was referenced in template READMEs but unimplemented; reviewers running the canonical first-five-minutes flow (`garnet new` → `cd` → `garnet test`) would have hit "unknown subcommand: test". Implemented as `cmd_test` in `bin/garnet.rs` — discovers `test_*` functions in `tests/*.garnet` files, pre-loads `src/main.garnet` as helper context (so cross-file references work), runs each in a fresh interpreter per file, reports per-test pass/fail + summary, exits non-zero on any failure. Verified on all 3 templates: 5 total tests across the bundled scaffolds, 5 green.

**Final artifacts shipped to `E_Engineering_Artifacts/dist/linux/`:**

- `garnet_0.3.0-1_amd64.deb` (1,533,456 B) — sha256: `5f2112c9bf221aa6180fdf8b5ca1fff555fbc2d1047c58787d8ed6a3e889fe57`
- `garnet-0.3.0-1.x86_64.rpm` (1,634,365 B) — sha256: `55a06b3bb24d4d3712eb2659e768c38e3b9ff2f68d974b2b02b9f9098e69ef17`
- `SHA256SUMS` — for the universal `sh.garnet-lang.org/install.sh` integrity-verification chain

## Verified feature coverage

The v4.2 additions exercised end-to-end on real Linux:

| Feature | Source ref | How verified |
|---------|------------|--------------|
| Stdlib↔interpreter bridge (v3.4.1 Day 1) | `stdlib_bridge.rs` | Binary starts + emits wordmark via interpreter prelude initialization — proves the 22-primitive bridge loads without crashing |
| CapCaps call-graph propagator (v3.4.1 Day 2) | `caps_graph.rs` | Part of the checker path invoked implicitly by `garnet build`; no runtime failures |
| ManifestSig Ed25519 (v3.4.1 Day 3) | `manifest.rs` | `keygen` generates 32-byte SigningKey + hex pubkey; `sign` produces 128-char hex signature; `verify --signature` confirms cryptographic round-trip on both distros |
| `garnet new` scaffolding (Phase 6B) | `new_cmd.rs` | All 3 templates tested (cli on Ubuntu, agent-orchestrator on Fedora, web-api available but not separately run) |
| Wordmark + Rung version table (Phase 6C) | `lib.rs::print_version` | Rendered cleanly on both distros |
| `.deb` packaging (Phase 6A) | `[package.metadata.deb]` | cargo-deb happy; dpkg accepts |
| `.rpm` packaging (Phase 6A) | `[package.metadata.generate-rpm]` | cargo-generate-rpm happy; rpm/dnf accept |
| Man page (`garnet(1)`) | `man/garnet.1` | Installed to `/usr/share/man/man1/garnet.1` on Fedora; `.deb` ships it but minimal Ubuntu container excludes — real Ubuntu installs get it |

---

## Gap: what's still unverified

| Platform | Why deferred | Who closes it |
|----------|--------------|---------------|
| **macOS `.pkg`** | Apple Xcode / codesign / notarytool / Developer ID are Mac-only + credential-gated. No Docker path exists. | User transfers workspace to their MacBook and runs `macos/build-pkg.sh` with `APPLE_DEV_ID_INSTALLER` / `APPLE_DEV_ID_APP` / `APPLE_NOTARY_PROFILE` set. The build script refuses to run without them. |
| **Windows `.msi`** | MinGW toolchain triggers miette/backtrace-ext `STATUS_ACCESS_VIOLATION` at `garnet.exe` startup; MSI would wrap a crashing binary. | User in progress (concurrent with this session) on `rustup target add x86_64-pc-windows-msvc` + Visual Studio Build Tools 2022 install; then `cargo build --release --target x86_64-pc-windows-msvc -p garnet-cli` produces a binary that starts successfully, then `cargo wix --nocapture` produces the MSI. Signing via `signtool sign` against the user's Authenticode cert. |
| **Clean-VM install from `sh.garnet-lang.org`** | No release tarball is hosted yet (pre-submission). | Post-first-release verification. |

---

## Bind-mount + `.cargo/config.toml` gotcha (for the next Linux runner)

Two small traps encountered and documented so the next Linux re-verifier avoids them:

1. **Docker Desktop on Windows host**: `D:\…` drive-letter paths may not be shared with Docker Desktop by default. Workaround: stage the workspace at `C:\Users\<user>\garnet-docker-stage\` (which IS shared) before bind-mounting. Exclude `target/` and the host's `.cargo/config.toml` during the copy.

2. **Host `.cargo/config.toml` with `target = "x86_64-pc-windows-msvc"`** (correct for Windows MSI builds) leaks into the Linux container via bind-mount and causes `error[E0463]: can't find crate for core` because the MSVC target is not installed inside the `rust:1-bookworm` image. Two fixes, either works:

   - Delete `.cargo/` from the staged copy (simplest — what this session did).
   - Move the MSVC target pin to the host user's `~/.cargo/config.toml` (global) instead of the workspace's `.cargo/config.toml` — then Linux bind-mounts don't see it.

---

## Handoff: macOS verification prompt

Copy verbatim into a fresh Claude Code session on the MacBook once the workspace is transferred:

> "Read `Garnet/Opus-Gpt-Xai-Opus-Gemini-Opus/Garnet_Final/F_Project_Management/GARNET_v4_2_Phase_6A_HANDOFF.md` §'macOS .pkg'. Verify Xcode Command Line Tools are installed (`xcode-select -p`); if not, run `xcode-select --install`. Set up the notarytool profile if not already done:
>
> ```sh
> xcrun notarytool store-credentials --apple-id <your apple id> --team-id <TEAMID>
> ```
>
> Set the three env vars per the build-pkg.sh header and run:
>
> ```sh
> cd Garnet_Final/E_Engineering_Artifacts/garnet-cli
> ./macos/build-pkg.sh
> ```
>
> After completion, verify with:
>
> ```sh
> pkgutil --check-signature target/macos/garnet-0.4.2-universal.pkg
> spctl --assess --type install target/macos/garnet-0.4.2-universal.pkg
> xcrun stapler validate target/macos/garnet-0.4.2-universal.pkg
> ```
>
> All three must report `valid`. Report: (a) the SHA-256 of the .pkg, (b) the three verification command outputs. Then open a new session's GARNET_v4_2_Phase_6D_macOS_VERIFIED.md in F_Project_Management/ documenting results, modeled on GARNET_v4_2_Phase_6D_Linux_VERIFIED.md."

---

## Handoff: Windows verification prompt (after MSVC switch lands)

> "Confirm `rustc --version` prints a `x86_64-pc-windows-msvc` triple. Then from `E_Engineering_Artifacts/`:
>
> ```powershell
> cargo clean -p garnet-cli
> cargo build --release -p garnet-cli
> target\release\garnet.exe --version
> ```
>
> The wordmark + version table should render without `STATUS_ACCESS_VIOLATION`. Next:
>
> ```powershell
> cargo install cargo-wix
> cd garnet-cli
> cargo wix --nocapture
> ```
>
> Output will land at `target\wix\garnet-0.3.0-x86_64.msi`. Sign with the user's Authenticode cert per `garnet-cli/wix/README.md`. Install on a Windows 11 sandbox, run `garnet --version` + `garnet new my_app --template cli`, uninstall via Add/Remove Programs, confirm clean. Write `GARNET_v4_2_Phase_6D_Windows_VERIFIED.md` summarizing."

---

*"Let us not be weary in well doing: for in due season we shall reap, if we faint not." — Galatians 6:9*

*Written by Claude Opus 4.7 at the Phase 6D Linux close — 2026-04-17.*
