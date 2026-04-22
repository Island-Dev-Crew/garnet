# GARNET v4.2 Phase 6A — Installer Scaffolding Handoff

**Purpose:** Document what landed in Phase 6A — the cross-platform installer configuration — and what remains as credential-dependent work the user (or CI) must finish with real signing identities + clean VMs.
**Last updated:** 2026-04-17 (Stage 6 Phase 6A close)
**Predecessor doc:** `GARNET_v4_2_STAGE6_KICKOFF.md`, `GARNET_v3_4_1_HANDOFF.md`
**Successor doc:** `GARNET_v4_2_HANDOFF.md` (end-of-Stage-6 close)
**Anchor:** *"A prudent man foreseeth the evil, and hideth himself: but the simple pass on, and are punished." — Proverbs 27:12*

---

## What shipped in Phase 6A

### Windows MSI (cargo-wix)

| Path | Purpose |
|------|---------|
| `garnet-cli/wix/main.wxs` | WiX XML source: product + upgrade GUID, per-machine install to `C:\Program Files\Garnet\bin\`, HKLM PATH entry, Start Menu shortcut, ARP entry, post-install `--version` smoke test. |
| `garnet-cli/wix/README.md` | Build steps (cargo-wix + candle/light), signtool invocation for the user's code-signing cert, verification with `signtool verify /pa`, and the 5 branding assets the user supplies. |
| `garnet-cli/Cargo.toml` `[package.metadata.wix]` | Stable `upgrade-guid` + `path-guid` so in-place upgrades work. |

### Linux .deb (cargo-deb)

| Path | Purpose |
|------|---------|
| `garnet-cli/Cargo.toml` `[package.metadata.deb]` | Full metadata: name, maintainer, extended description, `depends = "$auto, ca-certificates"`, assets table mapping binary + man page + docs + systemd unit to their FHS paths. |
| `garnet-cli/linux/garnet-actor.service` | Systemd unit for the actor runtime — **disabled by default**. Runs `ExecStartPre=garnet verify ... --signature` before starting so a tampered binary fails fast. Hardening: `NoNewPrivileges`, `ProtectSystem=strict`, `SystemCallFilter=@system-service`, etc. |
| `garnet-cli/linux/README.md` | Build + install + enable-service steps for Debian/Ubuntu/Mint. |

### Linux .rpm (cargo-generate-rpm)

| Path | Purpose |
|------|---------|
| `garnet-cli/Cargo.toml` `[package.metadata.generate-rpm]` | Full metadata mirroring the .deb assets table; requires `ca-certificates`. |

### macOS .pkg (productbuild + notarytool)

| Path | Purpose |
|------|---------|
| `garnet-cli/macos/build-pkg.sh` | End-to-end build driver: `cargo build --target x86_64-apple-darwin` + `aarch64-apple-darwin` → `lipo` → `codesign` → `pkgbuild` → `productbuild` → `productsign` → `xcrun notarytool submit --wait` → `xcrun stapler staple`. Refuses to run unless `APPLE_DEV_ID_INSTALLER`, `APPLE_DEV_ID_APP`, and `APPLE_NOTARY_PROFILE` env vars are set. |
| `garnet-cli/macos/distribution.xml` | productbuild distribution description: branded welcome/background/conclusion HTML, license accept, single-choice install. |
| `garnet-cli/macos/README.md` | Pre-build env-var setup, the 4 branded assets to drop in `macos/resources/`, post-build verification via `pkgutil --check-signature` + `spctl --assess` + `xcrun stapler validate`. |

### Universal shell installer

| Path | Purpose |
|------|---------|
| `installer/sh.garnet-lang.org/install.sh` | POSIX `/bin/sh` script mirroring rustup's UX. Prints wordmark, detects OS + arch, picks .deb/.rpm/.pkg/tar format, downloads asset, fetches `SHA256SUMS` and verifies, runs the native installer. Exits non-zero on any SHA-256 mismatch. |
| `installer/sh.garnet-lang.org/README.md` | Deployment notes (HSTS, `Content-Type: text/plain`, 5 min cache, INTEGRITY.txt convention), shellcheck guidance. |

### Man page

| Path | Purpose |
|------|---------|
| `garnet-cli/man/garnet.1` | groff-format manual page covering every subcommand (including v4.2 additions: `new`, `keygen`, `build --sign`, `verify --signature`, `convert`), the CapCaps capability inventory, worked examples for scaffolding + signed releases + Ruby→Garnet conversion. Shipped to `/usr/share/man/man1/garnet.1` by both .deb and .rpm; to `/usr/local/share/man/man1/garnet.1` by macOS .pkg. |

---

## What does NOT ship in Phase 6A — and why

Three categories of work require real-world artifacts outside the code repository:

### 1. Signing credentials

None of the following are checked in (nor should they be):

- Windows Authenticode code-signing certificate (`.pfx` + password). The user supplies this to `signtool sign` at release time.
- Apple Developer ID Application + Developer ID Installer certificates. Resident in the signer's macOS Keychain; the build script references them by identity name via environment variable.
- Apple notarytool keychain profile (`xcrun notarytool store-credentials`). Local to the signer's machine.
- Ed25519 release-signing key for the `SHA256SUMS` file served by `sh.garnet-lang.org`. Generated once via `garnet keygen` + rotated out-of-band.

**Action item:** at first-release time the user runs the sign/notarize steps from each platform's README. The config files are complete; no code change is required.

### 2. Branding assets

| Asset | Referenced by | Notes |
|-------|---------------|-------|
| `wix/License.rtf`, `wix/Banner.bmp`, `wix/Dialog.bmp`, `wix/Garnet.ico`, `LICENSE.txt` | Windows MSI | User's logo + an RTF license. |
| `macos/resources/background.png`, `welcome.html`, `conclusion.html`, `LICENSE.txt` | macOS .pkg | User's logo + the welcome copy. |
| PNG/SVG Garnet logo | README.md hero, docs favicon, REPL prompt banner | Phase 6C remainder. |

**Action item:** drop the assets into the referenced directories before cutting a release. Build scripts fail loudly on missing asset paths.

### 3. Clean-VM installation verification (Phase 6D)

The Phase 6A scaffolding is complete enough to run through on a clean Win11 / macOS Sonoma / Ubuntu 24.04 LTS VM; execution is deferred to Phase 6D (~5h) per the boot doc's plan. Success criterion: install → first-run in under 2 minutes on each.

**Action item:** run `curl -sSf https://sh.garnet-lang.org | sh` on each target VM; capture the timing; if any target exceeds 120s, profile the bottleneck before release.

---

## Build commands (quick reference)

Windows (PowerShell):

```powershell
rustup target add x86_64-pc-windows-msvc
cd garnet-cli
cargo build --release --target x86_64-pc-windows-msvc
cargo wix --nocapture
# Then signtool sign per wix/README.md
```

macOS:

```sh
export APPLE_DEV_ID_INSTALLER="Developer ID Installer: <name> (<TEAMID>)"
export APPLE_DEV_ID_APP="Developer ID Application: <name> (<TEAMID>)"
export APPLE_NOTARY_PROFILE="<profile>"
cd garnet-cli
./macos/build-pkg.sh
```

Debian/Ubuntu:

```sh
cargo install cargo-deb
cd garnet-cli
cargo build --release
cargo deb
```

Fedora/RHEL:

```sh
cargo install cargo-generate-rpm
cd garnet-cli
cargo build --release
cargo generate-rpm
```

---

## Verification status (Phase 6A close)

| Gate | Status |
|------|--------|
| `cargo test -p garnet-actor-runtime --release --lib` | ✅ 17 pass |
| `cargo test -p garnet-stdlib --release` | ✅ 74 pass |
| `cargo test -p garnet-convert --release` | ✅ 85 pass (61 unit + 24 corpus) |
| `cargo check --workspace --tests` | ✅ clean (1 pre-existing warning unchanged) |

Cargo.toml `[package.metadata.deb]` and `[package.metadata.generate-rpm]` blocks are read only by `cargo-deb` / `cargo-generate-rpm` at package time; cargo itself ignores them, so the workspace compile is unaffected.

---

## What remains in Stage 6

| Phase | Status | Scope |
|-------|--------|-------|
| Phase 0 — pre-MIT DX rigor | ✅ done | §20 + Comprehension Study Protocol |
| v3.4.1 bundle | ✅ done | Stdlib bridge + CapCaps propagator + ManifestSig |
| Phase 6A — installer scaffolding | ✅ done (this doc) | Win MSI + macOS PKG + Linux .deb/.rpm + sh.garnet-lang.org + man page |
| Phase 6B — `garnet new` scaffolding | ✅ done | 3 templates embedded via `include_str!` |
| Phase 6C — Logo + brand integration | 🟡 partial | Wordmark + `--version` + `--help` shipped; PNG/SVG + REPL banner + colored output + docs favicon + README hero deferred |
| Phase 6D — Verification on clean VMs | ⏳ pending | Win11 + macOS Sonoma + Ubuntu 24.04 install → first-run-under-2-minutes |
| Phase 6E — Final handoff + MIT package | ⏳ pending | `GARNET_v4_2_HANDOFF.md` + verification log + submission cut |

---

## Handoff pointer for Phase 6D

> "Phase 6A installer scaffolding is complete — every config file, build driver, and installer script ships in source. To close Stage 6 we need Phase 6D verification: spin up clean Win11 26H1 / macOS Sonoma (Apple Silicon) / Ubuntu 24.04 LTS VMs, run `curl --proto '=https' --tlsv1.2 -sSf https://sh.garnet-lang.org | sh` on each, time the install-to-first-run, and confirm the Phase 6D success criteria from the boot doc (install <2 min, `garnet --version` shows wordmark, `garnet new test_proj --template cli && cd test_proj && garnet build && garnet run` succeeds, clean uninstall). Before Phase 6D runs, the user supplies the 5 Windows branding assets (wix/*) and 4 macOS branding assets (macos/resources/*), plus the sign identities / notarytool profile. Phase 6C remainder (PNG/SVG logo, REPL banner, README hero) can land in parallel with Phase 6D."

---

*"For which of you, intending to build a tower, sitteth not down first, and counteth the cost, whether he have sufficient to finish it?" — Luke 14:28*

*Written by Claude Opus 4.7 at the Phase 6A close — 2026-04-17.*
