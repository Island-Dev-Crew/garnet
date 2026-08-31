<p align="center">
  <img src="garnet-cli/assets/garnet-logo.png" alt="Garnet" width="120">
</p>

<h1 align="center">Garnet</h1>

<p align="center"><strong>Rust rigor. Ruby velocity. One coherent language —<br>
built for the code agents write and humans accept.</strong></p>

<p align="center"><em>No authority without evidence. Acceptance is a decision made on evidence the author cannot fake.</em></p>

<p align="center">
  <a href="https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.8.1">Release <!-- truth:latest_tag -->v0.8.1<!-- /truth --> · signed binaries</a> ·
  <a href="LICENSE">MIT OR Apache-2.0</a> ·
  <a href="https://garnet-lang.org">garnet-lang.org</a> ·
  <a href="https://garnet-lang.org/status.html">Verified today / Still open</a> ·
  <a href="C_Language_Specification/GARNET_v1_0_Mini_Spec.md">Mini-Spec v1.0</a> ·
  <a href="FAQ.md">FAQ</a>
</p>

---

## Why Garnet

Every ambitious team makes the same bargain: **Rust for the hot path, Ruby for the
orchestration, and a painful FFI between them** — or one language and its weakness swallowed
whole. Garnet refuses the bargain. Managed mode (`def` + ARC + exceptions) feels like Ruby.
Safe mode (`@safe fn` + ownership + `Result`) feels like Rust. The mode boundary auto-bridges
errors and ownership, and `garnet check` surfaces every boundary call site. One grammar, two
registers, no FFI between them.

And in 2026 there's a second bargain nobody should accept. AI agents now write the code;
**human review is the bottleneck**, and signatures alone can't tell you what a change is
*allowed to do* — the supply chain has already produced validly-signed malware. Garnet doesn't
ask you to trust that the model understood. **It makes acceptance a decision on capability
evidence your own toolchain recomputes — not the model's claims**: functions declare their
authority budget, the checker verifies declared budgets transitively, and `diff-caps` answers
*"what new authority am I granting?"* in one screen.

Safe by default. Fast when needed. Joyful always.

## Sixty seconds of Garnet

```garnet
# Managed mode — Ruby feel, zero ceremony
@caps(fs)
def load_config(path) {
  try {
    std::json::parse(fs::read_file(path))
  } rescue e {   # any fs/parse failure → nil (the checker nudges: name the type)
    nil
  }
}

# Safe mode — Rust rigor where it earns its keep
@safe
fn checksum(borrow data: String) -> String {
  crypto::blake3(data)   # BLAKE3, hex-encoded
}

# Agent-native: memory and actors are language, not libraries
actor Researcher {
  memory episodic events  : EpisodeStore<Event>
  memory semantic  facts  : VectorIndex<Fact>
  protocol recall(q: String) -> Array<Fact>
}

# And the checker holds the line — declared budget vs. actual use:
@caps(net)
def sneaky() {
  fs::read_file("/etc/passwd")
}
# $ garnet check
# caps coverage: function `sneaky` does not declare `fs`
# but transitively calls `fs::read_file` which requires it
```

## What no other language gives you

The pillars are individually precedented — Pony and Austral, Wasmtime and eBPF, Sigstore and
SLSA each do a piece well. **Garnet's bet is the integration**, enforced at the language layer
and aimed at agent-authored code:

- **`diff-caps`** — the capability-surface diff as an acceptance gate. When a dependency or an
  agent's PR changes what the code *can do*, you review the authority delta, not every line.
- **`@caps(...)`** — functions declare their OS-authority budget; the CapCaps propagator checks
  declared budgets transitively at check time, and the entry point must declare its budget. Under
  the `garnet` CLI, the gated host-authority primitives (fs, net, proc, env, log-to-file — 12
  runtime-gated + 3 entry-gated) additionally trap at run time unless the calling chain declares
  the authority; `time`/`uuid` and pure computation are checker-only. See the
  [capability enforcement scope table](C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md).
- **An enforced kernel** — `@caps` and `@max_depth` trap identically on both execution backends,
  with cross-OS trap parity recorded as evidence, not asserted.
- **The seal** — `garnet build --deterministic --sign` emits a byte-identical manifest plus an
  Ed25519 signature; releases ship a CycloneDX SBOM and GPG-signed checksums.
- **Agent-native memory** — `memory working|episodic|semantic|procedural` as language keywords,
  so the runtime knows which kind it's allocating.
- **Typed actors** — bounded mailboxes, compiler-checked protocols, Ed25519-signed hot-reload
  with BLAKE3 schema fingerprints.

[GitHub's language bar](https://api.github.com/repos/Island-Dev-Crew/garnet/languages) counted this repository as **38.7% Python** at the 2026-08-31 `f6d3285` main snapshot because the trust and verification harness is Python; the product compiler and runtime are Rust.

## Install

```sh
curl --proto '=https' --tlsv1.2 -sSf https://garnet-lang.org/install.sh | sh
```

Release-first, source-fallback. <!-- truth:latest_tag -->v0.8.1<!-- /truth --> ships signed
`.deb` / `.rpm` / macOS tarballs with a GPG-signed `SHA256SUMS` — verify per
[docs/release-signing.md](docs/release-signing.md). No matching package for your platform?
The installer builds from source (Rust 1.95+; CI also tracks current stable — per
[CONTRIBUTING.md](CONTRIBUTING.md)), or:

```sh
git clone https://github.com/Island-Dev-Crew/garnet
cd garnet/garnet-cli && cargo install --path . --locked
```

## Quickstart

```sh
garnet new --template cli my_app    # also: web-api, agent-orchestrator
cd my_app
garnet test                         # starter tests pass green
garnet run src/main.garnet

# produce a reproducible, signed build:
garnet keygen my.key
garnet build --deterministic --sign my.key src/main.garnet
garnet verify src/main.garnet src/main.garnet.manifest.json --signature
```

## Verified today / Still open

Garnet is a **research-grade prototype (<!-- truth:latest_tag -->v0.8.1<!-- /truth -->), not
production-complete** — and this README will never tell you otherwise.

| Verified today | Still open |
|---|---|
| Signed v0.8.1 release: `.deb`, `.rpm`, macOS tarballs, SBOM, GPG-signed sums | macOS `.pkg` notarization, Windows `.msi` (credential-gated) |
| `@caps` + `@max_depth` enforced on interpreter **and** VM; cross-OS trap parity recorded | OS-sandbox enforcement beyond Linux seccomp |
| Capability-bounded acceptance demo: agent code accepted *and refused* on evidence, sealed | **Under construction:** `garnet build --evidence` (W-SHIP; no shipping CLI flag); independent verification of the self-found red-team fix |
| <!-- truth:primitive_count -->80<!-- /truth --> capability- and stability-tagged stdlib primitives | Production VM performance (unbenchmarked, unclaimed) |
| LSP + VS Code extension (local VSIX), trivia-preserving CST, formatter baseline | Marketplace/OpenVSX publication; incremental parsing |
| Rust / Ruby / Python / Go migration assistant with lineage + `@sandbox` audit gate | Browser playground; package registry beyond stub |

The full ledger lives in [CURRENT_STATE.md](CURRENT_STATE.md); the readiness detail lives on
[the status page](https://garnet-lang.org/status.html). The evidence scorecard for the research
claims: 4 supported, 2 partial, 0 refuted, 1 pending-infra
([seven papers](A_Research_Papers/)).

## Learn more

[Getting started](https://garnet-lang.org/getting-started.html) ·
[Mini-Spec v1.0](C_Language_Specification/GARNET_v1_0_Mini_Spec.md) ·
[Conformance matrix](C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md) ·
[Capability model](FAQ.md#whats-the-capability-model) ·
[Script walkthroughs](docs/internals/script-walkthroughs.md) ·
[Research papers](A_Research_Papers/) ·
[Blog](https://garnet-lang.org/blog/)

## Community

Questions → [FAQ](FAQ.md), then [Discussions](https://github.com/Island-Dev-Crew/garnet/discussions) ·
Bugs → [Issues](https://github.com/Island-Dev-Crew/garnet/issues) ·
Security → [SECURITY.md](SECURITY.md) (private advisories, please) ·
Contributing → [CONTRIBUTING.md](CONTRIBUTING.md) + [Code of Conduct](CODE_OF_CONDUCT.md)

## License

Dual-licensed **MIT OR Apache-2.0** — your choice. Either is fine for commercial use.

---

<p align="center"><em>"Where there is no vision, the people perish." — Proverbs 29:18</em><br>
<sub>A project by Island Development Crew · Huntsville, AL</sub></p>
