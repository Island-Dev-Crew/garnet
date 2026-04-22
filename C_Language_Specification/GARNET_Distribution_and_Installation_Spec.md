# Garnet Distribution & Installation Specification
**Version:** 1.0
**Date:** April 16, 2026
**Companion to:** Compiler Architecture Spec v1.0, Tier 2 Ecosystem Specifications
**Anchor:** *"Where there is no vision, the people perish." — Proverbs 29:18*

---

## 1. Purpose and Vision

Garnet must be as easy to install and adopt as Python or C++. A developer on any mainstream platform should be able to run a single one-line command and have a working `garnet` toolchain within sixty seconds. This specification is the engineering contract for that vision.

The design mirrors Rust's `rustup` — proven across 10M+ users and considered best-in-class for language toolchain management — while adding Garnet-specific concerns (dual-backend selection, `.garnet-manifest` provenance, kind-aware allocator libraries).

---

## 2. Installation Entry Points

### 2.1 Unix-like (Linux, macOS)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://garnet-lang.org/install.sh | sh
```

- `install.sh` auto-detects platform (`uname -s -m`), downloads the correct `garnetup` binary, places it in `~/.garnet/bin/garnetup`, and appends `~/.garnet/bin` to the user's `PATH` via `.bashrc` / `.zshrc` / `.profile`.
- Exit code 0 on success; non-zero on any failure with a human-readable message (miette-style diagnostic, mirroring the compiler's error style).
- No `sudo` required for default user-level install; `GARNET_INSTALL_PREFIX=/usr/local curl ... | sudo sh` for system-wide install.

### 2.2 Windows

```powershell
Invoke-WebRequest -Uri https://garnet-lang.org/install.ps1 -UseBasicParsing | Invoke-Expression
```

- `install.ps1` downloads `garnetup.exe` to `%USERPROFILE%\.garnet\bin\` and updates the user PATH environment variable.
- No administrator privileges required for default user-level install.
- MSI installer available at `https://garnet-lang.org/garnet-windows-x64.msi` for managed deployments.

### 2.3 Offline / airgapped installation

```bash
# Download bundle on an internet-connected machine
curl -sSfLO https://garnet-lang.org/dist/garnet-0.3.0-x86_64-linux-gnu.tar.xz
sha256sum -c garnet-0.3.0-x86_64-linux-gnu.tar.xz.sha256

# Transfer and install on target machine
tar xf garnet-0.3.0-x86_64-linux-gnu.tar.xz
./garnet-0.3.0-x86_64-linux-gnu/install.sh
```

Offline bundles are self-contained: compiler, stdlib, both codegen backends (LLVM and Cranelift), and documentation. Bundle size: ~80MB compressed, ~220MB uncompressed.

---

## 3. The `garnetup` Toolchain Manager

### 3.1 Core commands

```
garnetup install <toolchain>     Install a toolchain (e.g. `stable`, `nightly`, `0.3.0`)
garnetup default <toolchain>     Set the default toolchain for `garnet` invocations
garnetup update                  Update all installed toolchains to latest
garnetup uninstall <toolchain>   Remove a toolchain
garnetup list                    List installed toolchains
garnetup list-available          List all toolchain versions available from registry
garnetup component add <name>    Install additional component (e.g. `garnet-doc`, `garnet-src`)
garnetup component list          List components for current toolchain
garnetup self update             Update garnetup itself
garnetup self uninstall          Remove garnetup and all toolchains
garnetup completions <shell>     Generate shell completions
```

### 3.2 Toolchain channels

| Channel | Update frequency | Stability guarantee | Recommended for |
|---|---|---|---|
| `stable` | 6 weeks (synced with editions) | Production use; no backward-incompat breaks within major version | Production deployments |
| `beta` | 6 weeks (one release ahead of stable) | Feature-complete; may have bugs | QA/staging |
| `nightly` | Daily | May include unstable features behind `--unstable` flag | Language development, experiments |
| `<x.y.z>` | Pinned | Exact version | Reproducible builds, security audits |

### 3.3 Toolchain storage layout

```
~/.garnet/
├── bin/
│   ├── garnetup
│   ├── garnet          → proxies to active toolchain's bin/garnet
│   └── garnet-doc      → proxies to active toolchain's bin/garnet-doc
├── toolchains/
│   ├── stable-x86_64-unknown-linux-gnu/
│   │   ├── bin/
│   │   │   ├── garnet       (the compiler + CLI)
│   │   │   ├── garnet-doc   (documentation generator)
│   │   │   └── garnet-fmt   (code formatter)
│   │   ├── lib/
│   │   │   ├── libgarnet_std.rlib    (standard library, safe-mode)
│   │   │   ├── libgarnet_std.garnet  (standard library source, for docs)
│   │   │   ├── libgarnet_alloc.a     (kind-aware allocator runtime)
│   │   │   └── llvm/                 (bundled LLVM, if selected)
│   │   ├── share/
│   │   │   ├── doc/                  (offline docs)
│   │   │   └── man/                  (man pages)
│   │   └── etc/
│   │       └── manifest.toml         (toolchain manifest)
│   └── nightly-2026-04-16-x86_64-unknown-linux-gnu/
│       └── ... (same layout)
├── registry/
│   ├── cache/           (downloaded package archives)
│   └── src/             (extracted package sources)
└── settings.toml        (garnetup user settings)
```

The `bin/garnet` proxy is ~1KB; it reads `~/.garnet/settings.toml` for the active toolchain and `exec`s the real binary. This means switching toolchains is instant.

### 3.4 Project-local overrides

A `garnet-toolchain.toml` file in any project directory overrides the default:

```toml
[toolchain]
channel = "1.2.0"
components = ["garnet-fmt", "garnet-doc"]
targets = ["x86_64-unknown-linux-gnu", "wasm32-unknown-unknown"]
profile = "minimal"
```

This ensures reproducible per-project builds regardless of the user's default toolchain.

---

## 4. Platform Matrix

Tier 1 platforms (fully tested, prebuilt binaries, issues treated as release blockers):

| Platform | Target triple | Status |
|---|---|---|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | Tier 1 |
| Linux aarch64 | `aarch64-unknown-linux-gnu` | Tier 1 |
| macOS x86_64 (Intel) | `x86_64-apple-darwin` | Tier 1 |
| macOS aarch64 (Apple Silicon) | `aarch64-apple-darwin` | Tier 1 |
| Windows x86_64 MSVC | `x86_64-pc-windows-msvc` | Tier 1 |

Tier 2 platforms (prebuilt binaries, best-effort support):

| Platform | Target triple |
|---|---|
| Linux x86_64 musl (Alpine, static) | `x86_64-unknown-linux-musl` |
| Windows x86_64 GNU | `x86_64-pc-windows-gnu` |
| FreeBSD x86_64 | `x86_64-unknown-freebsd` |
| Linux aarch64 musl | `aarch64-unknown-linux-musl` |

Tier 3 targets (compile-only, no prebuilt binaries, cross-compile support):

| Target | Use case |
|---|---|
| `wasm32-unknown-unknown` | Browser WebAssembly |
| `wasm32-wasi` | Server-side WebAssembly (Cloudflare Workers, Wasmtime) |
| `x86_64-unknown-none` | Bare-metal / kernel |
| `armv7-unknown-linux-gnueabihf` | Raspberry Pi, embedded Linux |
| `riscv64gc-unknown-linux-gnu` | RISC-V Linux |

---

## 5. Codegen Backend Bundling

Garnet ships with two codegen backends per Compiler Architecture Spec §7. The toolchain manager can select between them:

### 5.1 LLVM backend (default for release)

- Bundled LLVM version: 18.x (pinned)
- Bundle size: ~40MB (compressed), ~160MB (uncompressed)
- Invocation: `garnet build --release` (defaults to LLVM)
- Performance: within 1.2x of Rust on compute-bound code (after O3 + LTO)

### 5.2 Cranelift backend (default for debug)

- Bundled Cranelift version: 0.111.x (pinned)
- Bundle size: ~8MB (compressed), ~20MB (uncompressed)
- Invocation: `garnet build` (defaults to Cranelift in debug) or `garnet build --backend cranelift`
- Performance: 5-10x faster compilation than LLVM, ~30% slower runtime

### 5.3 Minimal install

`garnetup install stable --profile minimal` installs only the Cranelift backend, reducing toolchain size from ~220MB to ~60MB. LLVM can be added later with `garnetup component add llvm-backend`.

---

## 6. Package Registry Protocol

### 6.1 Registry endpoint

- **Primary:** `https://registry.garnet-lang.org` (managed by the Garnet Foundation)
- **Alternative registries:** Configurable via `[registries]` in project `Garnet.toml`
- **Mirror protocol:** Sparse index (same design as modern crates.io), HTTPS with HTTP/2

### 6.2 Package URL scheme

```
https://registry.garnet-lang.org/api/v1/packages/{name}/{version}/download
https://registry.garnet-lang.org/api/v1/packages/{name}/versions          (metadata)
https://registry.garnet-lang.org/api/v1/search?q={query}                  (search)
https://registry.garnet-lang.org/api/v1/packages/{name}/owners            (ownership)
```

### 6.3 Publishing

```
garnet login                     Authenticate with registry (stores token in ~/.garnet/credentials.toml)
garnet publish                   Publish the current package
garnet yank --version 0.1.0      Prevent new projects from depending on this version
garnet owner --add <username>    Add a co-owner to a package
```

Published packages are **immutable** — once version `0.1.0` of `garnet-http` is published, it can never be replaced. Yanked versions remain accessible to existing users (for lockfile reproducibility) but do not appear in new dependency resolutions.

### 6.4 Package security

- All package downloads use HTTPS with certificate pinning
- Each package has a SHA-256 digest recorded in the registry index; the client verifies before installation
- Packages may be signed with a cryptographic key; signed packages display a verified badge
- Vulnerability database: `garnet audit` queries an RSS feed of known CVEs against installed dependencies

---

## 7. Update Mechanism

### 7.1 Auto-update opt-in

```bash
garnetup self update              # Update garnetup itself
garnetup update                   # Update all installed toolchains
garnetup update stable            # Update only stable
garnetup update --dry-run         # Show what would be updated without updating
```

Garnet does NOT auto-update in the background. Users explicitly opt into updates.

### 7.2 Update channel authenticity

- All update manifests are signed with a release key (minisign or SSH-compatible signature)
- `garnetup self update` verifies the signature before replacing the binary
- Public key fingerprint is embedded in the `garnetup` binary at build time

### 7.3 Rollback

```bash
garnetup default 1.2.0            # Pin to a specific previous version
garnetup list                     # Show installed versions
garnetup uninstall 1.3.0          # Remove a buggy version after rolling back
```

---

## 8. Vendoring and Reproducibility

For air-gapped or highly regulated environments, Garnet supports vendored dependencies:

```bash
garnet vendor                     # Downloads all dependencies into ./vendor/
garnet build --offline --frozen   # Builds using only vendored deps + lockfile, no network
```

Combined with the `garnet-toolchain.toml` pinning and `garnet.lock` (SemVer-exact version pinning), this guarantees that the same source tree produces the same binary on any Tier 1 machine — the Paper VI Contribution 7 (Deterministic Reproducible Builds) in practice.

---

## 9. Integration With External Tools

### 9.1 CI/CD

Garnet provides official GitHub Actions and pre-built Docker images:

```yaml
# .github/workflows/ci.yml
- uses: garnet-lang/setup-garnet@v1
  with:
    toolchain: stable
    components: garnet-fmt, garnet-doc
- run: garnet fmt --check
- run: garnet test
- run: garnet build --release
```

Docker images:
- `garnetlang/garnet:stable` (latest stable)
- `garnetlang/garnet:1.2.0-slim` (minimal, Cranelift-only)
- `garnetlang/garnet:nightly` (for language development)

### 9.2 IDE support

- **VS Code:** `garnet-vscode` extension published on the VS Code Marketplace. Provides LSP-backed diagnostics, formatting, quick-fixes, refactoring.
- **JetBrains:** `garnet-intellij` plugin on the JetBrains Marketplace for IntelliJ IDEA, RubyMine, RustRover, CLion.
- **Neovim:** `garnet.nvim` with built-in LSP client configuration.
- **Emacs:** `garnet-mode` with `lsp-mode` / `eglot` integration.

All editor integrations connect to the `garnet-lsp` server, a separate binary shipped with the toolchain (or installed as a component: `garnetup component add lsp`).

### 9.3 Build system integration

- **Make/CMake:** `garnet build --emit object` produces `.o` files for linking into existing C/C++ projects.
- **Bazel:** `rules_garnet` Starlark rules for `garnet_library`, `garnet_binary`, `garnet_test`.
- **Nix:** `garnet-nix` overlay provides `garnetup` and per-version derivations.

---

## 10. Uninstallation

```bash
garnetup self uninstall
```

- Removes everything under `~/.garnet/`
- Removes PATH modifications from shell RC files (prompts before modifying)
- Leaves project `garnet-toolchain.toml` files untouched (user data)

The uninstall is fully reversible by re-running the install command — no lingering system state.

---

## 11. Implementation Timeline

| Milestone | Deliverable | Target |
|---|---|---|
| M1 (post-Rung 3) | `garnet` CLI with `build`, `run`, `test`, `fmt`, `doc`, `repl` | Q3 2026 |
| M2 | `garnetup` toolchain manager (Linux x86_64 only) | Q4 2026 |
| M3 | Registry backend + publishing flow | Q1 2027 |
| M4 | Tier 1 platforms complete (macOS, Windows, aarch64) | Q2 2027 |
| M5 | Tier 2 platforms + offline bundles + IDE extensions | Q3 2027 |
| M6 | Provenance manifests + `garnet verify` + reproducible build guarantee | Q4 2027 |

This timeline aligns with the engineering ladder (Rungs 3-6) and the PLDI 2027 submission calendar.

---

## 12. Comparison to Existing Toolchains

| Feature | rustup | pyenv | nvm | **garnetup** |
|---|---|---|---|---|
| One-line install | ✓ | ✓ | ✓ | ✓ |
| Multiple toolchains | ✓ | ✓ | ✓ | ✓ |
| Per-project override | ✓ | ✓ | ✓ | ✓ |
| Offline install | ✓ | partial | partial | ✓ |
| Signed updates | ✓ | ✗ | ✗ | ✓ |
| Integrated codegen backends | ✗ (single) | n/a | n/a | ✓ (LLVM + Cranelift) |
| Provenance manifests | ✗ | ✗ | ✗ | ✓ |
| Sub-minute first install | partial | ✗ | ✓ | ✓ (target) |

Garnet's distribution story combines Rustup's security and per-project pinning with Node.js's install speed, plus novel provenance (Paper VI Contribution 7).

---

*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*
*"Where there is no vision, the people perish." — Proverbs 29:18*

**Distribution & Installation Specification prepared by Claude Code (Opus 4.7) | April 16, 2026**
