<p align="center">
  <img src="garnet-cli/assets/garnet-logo.png" alt="Garnet — mechanical rigor meets faceted gem" width="420">
</p>

<h1 align="center">Garnet</h1>

<p align="center">
  <strong>Rust rigor. Ruby velocity. One coherent language.</strong>
</p>

<p align="center">
  <a href="https://github.com/Island-Dev-Crew/garnet/actions"><img src="https://img.shields.io/github/actions/workflow/status/Island-Dev-Crew/garnet/linux-packages.yml?branch=main&label=CI&logo=github" alt="CI status"></a>
  <a href="https://github.com/Island-Dev-Crew/garnet/releases"><img src="https://img.shields.io/github/v/release/Island-Dev-Crew/garnet?color=%239C2B2E&label=release" alt="Release status"></a>
  <a href="https://github.com/Island-Dev-Crew/garnet/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-E5C07B" alt="License"></a>
  <a href="https://garnet-lang.org"><img src="https://img.shields.io/badge/site-garnet--lang.org-9C2B2E" alt="Website"></a>
  <a href="FAQ.md"><img src="https://img.shields.io/badge/docs-FAQ-blue" alt="FAQ"></a>
</p>

<p align="center">
  <a href="#install"><strong>Install</strong></a>  ·
  <a href="#quickstart"><strong>Quickstart</strong></a>  ·
  <a href="FAQ.md"><strong>FAQ</strong></a>  ·
  <a href="https://garnet-lang.org"><strong>Website</strong></a>  ·
  <a href="A_Research_Papers/"><strong>Research Papers</strong></a>  ·
  <a href="C_Language_Specification/GARNET_v1_0_Mini_Spec.md"><strong>Mini-Spec v1.0</strong></a>
</p>

---

Garnet is a dual-mode, agent-native language platform.

- **Managed mode** (`def` + ARC + exceptions) feels Ruby-like.
- **Safe mode** (`@safe` + `fn` + ownership + `Result`) feels Rust-like.
- The mode boundary auto-bridges errors and ARC-affine semantics.

First-class memory primitives (working / episodic / semantic / procedural) for
agent cores. Typed actors with bounded mailboxes + Ed25519 signed hot-reload.
Compiler-as-agent that learns from its own compilation history.

Single `garnet` CLI. Deterministic signed manifests. Dependency-graph audit built in.

## Install

```sh
curl --proto '=https' --tlsv1.2 -sSf https://garnet-lang.org/install.sh | sh
```

The universal installer is release-first and source-fallback:

- If `v0.4.2` release assets exist, it downloads the native package and
  verifies it against `SHA256SUMS`.
- If release assets are not published yet, it falls back to:

```sh
git clone https://github.com/Island-Dev-Crew/garnet
cd garnet/garnet-cli
cargo install --path . --locked
```

Use `GARNET_INSTALL_MODE=release` to require a native release package, or
`GARNET_INSTALL_MODE=source` to force source install.

| Platform      | Installer                                   | Integrity / release requirement   |
|---------------|---------------------------------------------|-----------------------------------|
| Linux (.deb)  | `garnet_0.4.2-1_amd64.deb`                  | SHA-256 checksummed               |
| Linux (.rpm)  | `garnet-0.4.2-1.x86_64.rpm`                 | SHA-256 checksummed               |
| macOS (.pkg)  | `garnet-0.4.2-universal.pkg`                | Apple Developer ID + notarized    |
| Windows (.msi) | `garnet-0.4.2-x86_64.msi`                   | Authenticode + timestamped        |

The installer fetches assets from
[`github.com/Island-Dev-Crew/garnet/releases`](https://github.com/Island-Dev-Crew/garnet/releases),
verifies the selected file against `SHA256SUMS`, and falls back to source in
auto mode when the release, checksum manifest, or package is missing. See [SECURITY.md](SECURITY.md)
for the supply-chain story and
[`GARNET_v0_4_2_Installer_Release_Contract.md`](C_Language_Specification/GARNET_v0_4_2_Installer_Release_Contract.md)
for the exact hosting, artifact, integrity, and release-pipeline contract.

## Quickstart

Create a project, run it, test it:

```sh
garnet new --template cli my_app
cd my_app
garnet test                            # 2 starter tests pass green
garnet run src/main.garnet
```

Three canonical templates ship with the CLI:

| `--template`           | Shape                                                     |
|------------------------|-----------------------------------------------------------|
| `cli`                  | Minimal CLI with `@caps()` entry point                    |
| `web-api`              | HTTP/1.1 service with `@caps(net, time)`                  |
| `agent-orchestrator`   | Researcher / Synthesizer / Reviewer actors with 3 memory kinds |

Produce a reproducible, signed release:

```sh
garnet keygen my.key
garnet build --deterministic --sign my.key src/main.garnet
garnet verify src/main.garnet src/main.garnet.manifest.json --signature
```

## Architecture snapshot

| Crate | Role |
|-------|------|
| `garnet-parser` | Lex + parse (Mini-Spec v1.0) |
| `garnet-interp` | Managed-mode tree-walk interpreter |
| `garnet-check`  | Safe-mode validator + CapCaps call-graph propagator |
| `garnet-memory` | **Mnemos** — reference implementation of Garnet's **Memory Core** (four cognitively-inspired kinds: working / episodic / semantic / procedural). Production allocator path tracked in [`MEMORY_CORE_ROADMAP.md`](C_Language_Specification/MEMORY_CORE_ROADMAP.md) |
| `garnet-actor-runtime` | Bounded-mailbox actors + Ed25519 signed hot-reload |
| `garnet-stdlib` | OS-I/O primitives with capability metadata |
| `garnet-cli`    | Top-level `garnet` binary |
| `garnet-convert` | Rust / Ruby / Python / Go → Garnet **migration assistant** (stylized parsers, sandbox-on output, emits a `migrate_todo.md` checklist — not a full transpiler) |

## Documentation

Full research corpus + language specification lives in this repository at:

- `A_Research_Papers/` — seven research papers + four addenda
- `C_Language_Specification/` — Mini-Spec v1.0 + canonical grammar
  - **[Conformance matrix (v0.4.2)](C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md)** — what's actually implemented vs. specified, per Mini-Spec section
  - **[Memory Core roadmap](C_Language_Specification/MEMORY_CORE_ROADMAP.md)** — production-path tiers for Mnemos (the v0.4.x reference implementation) → v0.5+ allocator
- `D_Executive_and_Presentation/` — comparative developer-experience study
- `F_Project_Management/` — stage handoffs + verification logs

## Capability model

Every function declares its OS-authority budget with `@caps(...)`. The
v3.4.1 CapCaps propagator enforces this transitively: a function that
calls `fs::read_file` (which requires `fs`) must declare `@caps(fs)`, or
inherit it via a caller that does. Known capabilities: `fs`, `net`,
`net_internal`, `time`, `proc`, `ffi`, `*` (wildcard — managed mode only).

## Community

- **Questions** → [FAQ.md](FAQ.md) first; [Discussions](https://github.com/Island-Dev-Crew/garnet/discussions) for open-ended back-and-forth
- **Bugs / feature requests** → [Issues](https://github.com/Island-Dev-Crew/garnet/issues/new/choose) (use the templates)
- **Security disclosures** → [SECURITY.md](SECURITY.md) — use a private GitHub Security Advisory, not a public issue
- **Want to contribute?** → [CONTRIBUTING.md](CONTRIBUTING.md) + [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

## Project status

**v4.2 is research-grade.** Ready for prototype agents and scripting. Production-bearing workloads should wait for v5.0 (bytecode VM). See [FAQ.md §"Is Garnet production-ready?"](FAQ.md#is-garnet-production-ready) for the honest scorecard.

Verification status at current `main`:

- ✅ Linux `.deb` (Ubuntu 24.04) + `.rpm` (Fedora 40) — verified end-to-end in Docker, all 6 Phase 6D gates pass
- ✅ Windows binary (MSVC) — verified end-to-end, all 6 Phase 6D gates pass
- ✅ 1193 workspace tests pass on current `main`; 136 security-specific tests across 4 hardening layers
- ✅ 22 stdlib primitives bridged through the interpreter
- ✅ The universal curl installer works before release publication by falling back to source install; native packages remain release-gated by `SHA256SUMS`
- ⏳ macOS `.pkg` and Windows `.msi` release signing/notarization remain credential-gated release steps

## Research

Garnet is a doctoral research project. Seven research papers + four addenda ship in [`A_Research_Papers/`](A_Research_Papers/). The canonical language specification (Mini-Spec v1.0) is at [`C_Language_Specification/GARNET_v1_0_Mini_Spec.md`](C_Language_Specification/GARNET_v1_0_Mini_Spec.md). Paper VI's seven novel contributions were pre-registered in Phase 1C (April 2026) and measured in Phase 4A — honest scorecard: **4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra** (Paper VI Exp 1 awaits LLM API credits).

## License

Dual-licensed under MIT OR Apache-2.0 (your choice). See [LICENSE](LICENSE). Either license is fine for commercial use.
