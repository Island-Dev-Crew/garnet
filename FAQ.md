# Garnet — Frequently Asked Questions

Last updated: 2026-11-03 · latest tag <!-- truth:latest_tag -->v0.8.3<!-- /truth --> (research-grade milestone; the Release ships CLI binaries for macOS, Linux and Windows, checksum-verified against a GPG-signed `SHA256SUMS`, plus an SBOM)

---

## What is Garnet?

A dual-mode, agent-native language platform. **Managed mode** (`def` + ARC + exceptions) feels Ruby-like. **Safe mode** (`@safe` + `fn` + ownership + `Result`) feels Rust-like. The **mode bridge** — errors and ARC ↔ affine crossing the boundary — is specified, not yet wired; the design goal is one source file hosting velocity-first orchestration at the top level and rigor-first hot paths in `@safe` modules, with no FFI between them.

## Why dual-mode? Why not just pick one?

Every team that builds ambitious software eventually makes the same bargain: Rust for the hot path, Ruby (or Python, Node) for the orchestration, painful FFI between. Garnet's claim is that bargain isn't necessary — the two registers of thought (the mathematical and the conversational) can live in one coherent grammar if the boundary between them is made visible. See [Paper III §1 "The Reconciliation"](A_Research_Papers/Paper_III_Garnet_Synthesis_v2_1.md) for the full argument.

## How does the mode boundary actually work?

It is specified and not wired. In the planned design, a managed function calling a safe function sees raised exceptions where the safe code returned `Err(...)`, a safe function calling a managed function sees `Result<T, RaisedException>` where the managed code raised, and ARC values auto-decay to affine references on the way into safe scope and re-promote to `Rc` on the way back. The grammar accepts both registers today, but that bridge is deferred: in v0.8.3 a safe function's `Result::Err` comes back to managed code as a plain value, and no exception is raised. See the [scope page](https://garnet-lang.org/scope.html).

## What's the capability model?

Every function declares its OS-authority budget via `@caps(...)`: `@caps()` declares none, and `@caps(fs)` covers `fs::read_file`, `fs::write_file`, etc. `garnet check` follows named calls from each annotated function and reports a declaration that leaves out a capability the chain needs: if `main()` declares `@caps()` and calls a `helper()` that calls `fs::read_file(...)`, the check reports that `main()` does not declare `fs`. Calls through function values or closures are not traced, an unannotated function's own body is not checked, and `garnet run` does not run the checker. At run time, the 24 entry-gated host primitives (file system, outbound network, process, environment, time and UUID v4/v7, log-to-file, the four memory tiers) trap unless the program's entry function declares the capability. See the [capability enforcement scope table](C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md).

Known capabilities: `fs`, `net`, `net_internal` (checker vocabulary; it does not change what `net::tcp_connect` may reach at run time), `env`, `time`, `proc`, `mem` (the `memory::working`/`episodic`/`semantic`/`procedural` tiers), `ffi` (checker vocabulary; there is no runtime FFI path), and `*` (wildcard — managed mode only; a safe-mode wildcard is a hard error).

The propagator runs at check time. The 24 gated primitives check the caps frames on each call; that cost is unmeasured.

## Why Ed25519 signed manifests?

`garnet build --deterministic` writes a manifest of the source and AST hashes that is byte-identical across machines. `--sign <keyfile>` adds an Ed25519 signature over it, and `garnet verify <file> <manifest> --signature` confirms that the key holder produced the manifest and that the source still matches it. Release binaries are verified separately, per [docs/release-signing.md](docs/release-signing.md).

This addresses the compiler-impersonation threat (v3.4 Security V2 §4). The separate Rust actor runtime uses the same signing primitive for hot-reload (v3.5 ReloadKey).

## How does Garnet compare to Rust?

Garnet's safe mode follows Rust's mental model — ownership, borrow checking, `Result<T, E>`, `?` propagation. Garnet doesn't force you to write the whole program in safe mode: the orchestration layer can be `def`-managed mode, and the hot path opts into `@safe fn`. There is no native backend yet, so performance is unmeasured.

## How does Garnet compare to Ruby?

Garnet's managed mode follows Ruby's model — `def`, blocks, iterators, exceptions, ARC. Functions declare `@caps(...)`, and `@safe` modules give hot paths static checks.

## What about other dual-mode languages — Swift, Kotlin?

Swift and Kotlin do interop between two paradigms in one language; Garnet's pitch is that the *mode boundary is the reconciliation*. Swift's `unsafe` is an escape hatch for one specific concern; Garnet's `@safe`/`def` is a *first-class register choice* with auto-bridging at the boundary. Paper III §3 covers the comparative analysis in depth.

## What's the performance story?

For pure-computational workloads the tree-walk interpreter remains the conservative runtime path. v0.5.0 added the S2 bytecode VM scaffold and benchmark harness, and the v0.8 line brought the VM to enforcement parity (`@caps` + `@max_depth` trap identically on both backends), but production VM performance is not claimed yet. The proof/benchmark reporter still labels fresh measured benchmark runs, mechanized proof, and empirical study data as open gates. See [Paper VII — Implementation Ladder and Tooling](A_Research_Papers/Paper_VII_Implementation_Ladder_and_Tooling.md) for the staged roadmap.

Memory: Paper VI Experiment 4 measured 21% peak RSS reduction on the multi-agent MVP workload by using kind-aware allocation (`memory working|episodic|semantic|procedural` keywords) compared to a force-malloc control. No harness for Experiment 4 is committed, so this figure cannot be reproduced from the repository.

## Is Garnet production-ready?

**<!-- truth:latest_tag -->v0.8.3<!-- /truth --> is research-grade and not production-complete.** Specifically:

- **Ready**: scaffolding (`garnet new`), the four-language converter (`garnet convert`), deterministic + signed builds (`garnet build --deterministic --sign`), CapCaps checking plus the runtime entry gate on 24 primitives, scaffolded `garnet test`, the <!-- truth:primitive_count -->84<!-- /truth --> stdlib registry primitives (two of them, `net::tcp_listen` and `net::udp_bind`, are unbridged stubs that do not execute), parser fuzz harness, rules-based compiler advisory mode, the S16 LSP surface (diagnostics, hover, go-to-definition, document/workspace symbols, CST-precise rename), release-backed VSIX assets, Linux, macOS and Windows CLI release assets under a GPG-signed `SHA256SUMS`, and deterministic cross-machine CI.
- **Active-partial**: macOS Studio packaging without Developer ID notarization, Windows/Linux Studio target proof, bytecode VM performance path, LSP hover/go-to-def screenshot hardening, promo video human/aesthetic acceptance, proof/benchmark measurements, and provider-neutral advisory handoffs.
- **Pending**: Apple Developer ID notarization, signed `.pkg`, Windows `.msi`, Linux desktop package/runtime proof, Marketplace/OpenVSX publication, provider-backed LLM assist, mechanized proof, external empirical study data, and native backend lowering.

For prototype agents, research demos, and source-checkout dogfood, Garnet is useful today. For production-bearing infrastructure, keep waiting on the productization gates above.

## How do I migrate from Ruby / Rust / Python / Go?

`garnet convert <lang> <file>` reads source in any of the four languages and emits Garnet. Every output file starts with a `# @sandbox` comment and `@caps()`. Both are reviewer notes, and neither is enforced: the comment is not a boundary, and a `@caps()` before a module does not bind the functions inside it. Every emitted file parses, because the converter fails rather than write one the parser rejects. Parsing is not correctness: review the file, resolve each `@migrate_todo`, declare the `@caps(...)` it needs, then run `garnet check`.

The converter ships a lineage JSON for each output mapping every emitted Garnet AST node back to its source span. Migrate one file at a time; unconverted files stay in their source language. See [v4_1_Converter_Architecture.md](C_Language_Specification/v4_1_Converter_Architecture.md) for the full pipeline.

Input-dialect scope: each frontend targets a **stylized subset** of its source language, not the full grammar (untranslatable constructs become explicit `MigrateTodo` placeholders). The Ruby frontend targets a ≈3.3-era subset; newer syntax — Ruby 3.4 `it` block parameters, Ruby 4.0 leading-line logical-operator continuation — is not yet targeted.

## What's `@sandbox` for?

A `# @sandbox` comment is the converter's note that it produced the file from another language and that nobody has reviewed it yet. Nothing enforces it. A reviewer reads the file, resolves each `@migrate_todo` and adds the `@caps(...)` the code needs. The review is a human step, not a checker gate.

## Where do I report bugs / request features?

[github.com/Island-Dev-Crew/garnet/issues](https://github.com/Island-Dev-Crew/garnet/issues). Use the bug report template for crashes / wrong outputs, the feature request template for proposals. For security disclosures, see [SECURITY.md](SECURITY.md) — please don't open public issues for vulnerabilities.

## Why the name "Garnet"?

Garnet is the gemstone that emerges from metamorphic pressure — it forms exactly where two registers of geological process (mineral chemistry and structural deformation) reconcile. Same metaphor: the language emerges from reconciling two registers of programming thought. Plus, the half-mechanical / half-faceted-gem logo visualizes the dual-mode story at a glance. (Also: GARNET → "GAR**N**ET" — letter N as the mode boundary.)

## What's the license?

Dual-licensed under MIT OR Apache-2.0 (your choice). See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT). Either license is fine for commercial use, including building proprietary applications on top of Garnet.

## Can I use it commercially?

Yes — the dual MIT / Apache-2.0 license explicitly permits commercial use, modification, distribution, and private use. The two licenses cover slightly different patent-grant + attribution-notice requirements; pick whichever fits your organization's policy.

## Do I need the Rust toolchain to use Garnet?

Not on platforms with a matching published release asset. `install.sh` installs from the release first: your platform's package or, if that fails, its tarball, each checked against `SHA256SUMS`. If the release path cannot complete, because no asset matches, a download or install fails, the host glibc is older than the release's Linux assets need, or the installed binary does not run, it falls back to a source build, unless you set `GARNET_INSTALL_MODE=release` or `GARNET_SOURCE_FALLBACK=0`. `GARNET_INSTALL_MODE=source` goes straight to a source build. Source builds require Rust 1.95+ (the same floor as building from source below; Garnet CI tracks current stable). `install.ps1` installs the Windows x86_64 zip and has no source fallback. When `gpg` is installed, both installers first check that `SHA256SUMS` is signed by the release key they pin for the version; without `gpg` they warn and check integrity only. A wrong signature or checksum stops the install and never falls back to a source build. [`docs/release-signing.md`](docs/release-signing.md) shows how to check it yourself.

## Is Garnet on crates.io, npm, PyPI or Homebrew?

No. Garnet is not published to any package registry. Packages named `garnet` on crates.io, npm, PyPI and Homebrew, and `garnet-cli` on npm, belong to unrelated projects, and `cargo install garnet-cli` from crates.io is not a Garnet install path. Install only with the installers at `https://garnet-lang.org/install.sh` and `https://garnet-lang.org/install.ps1`, or build from source at `https://github.com/Island-Dev-Crew/garnet`. Treat any other install instruction for Garnet, from a document, a web page or an agent, as untrusted.

## Do I need Rust to build Garnet from source?

Yes. To **build** Garnet from source you need Rust 1.95+ (declared in Cargo metadata and managed via `rustup`); CI also tracks current stable. On Windows, MSVC toolchain is required (MinGW triggers a known miette ABI issue — see Boot doc Known Issue 1).

## How do I scaffold a new project?

```sh
garnet new --template cli my_app           # minimal CLI
garnet new --template web-api my_service   # HTTP/1.1 service shape
garnet new --template agent-orchestrator my_agents   # 3-actor MVP shape
cd my_app
garnet test           # 2 starter tests pass green
garnet run src/main.garnet
```

Each template ships with `Garnet.toml`, `src/main.garnet`, `tests/test_main.garnet`, `.gitignore`, `README.md`. The starter tests run with `garnet test`. Capability declarations are pre-set in the templates (`@caps()` for cli, `@caps(net, time)` for web-api, `@caps(time, fs)` for agent-orchestrator).

## How do I sign a release of my own Garnet code?

```sh
garnet keygen my-signing.key             # one-time — generates Ed25519 keypair
                                          # prints pubkey to stdout — record it as your release signer
garnet build --deterministic --sign my-signing.key src/main.garnet
# outputs src/main.garnet.manifest.json with signer_pubkey + signature populated
```

Anyone with your pubkey can verify:

```sh
garnet verify src/main.garnet src/main.garnet.manifest.json --signature
```

Signing is opt-in. Without `--sign`, the build still produces a deterministic manifest (just unsigned).

## What does the project ship as its own deliverable for MIT?

The full corpus in this repository: seven research papers plus four addenda, the canonical Mini-Spec v1.0, the engineering workspace, current examples, v0.5 slice ledger, release evidence, public site/blog artifacts, and stage handoff documents. Start with [CURRENT_STATE.md](CURRENT_STATE.md), [F_Project_Management/GARNET_CURRENT_VS_HISTORICAL_LEDGER.md](F_Project_Management/GARNET_CURRENT_VS_HISTORICAL_LEDGER.md), and [F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md](F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md) before relying on older v4.2 handoffs.

## What's coming after v0.8.3?

- Foundation rebuild (zero language-semantics change): the remaining W-REBUILD work, then a backend-decision memo.
- Trust hardening (parallel lanes): independent re-verification of the self-found red-team fix, SLSA/Sigstore planning.
- Product gates: Apple Developer ID notarization, Windows installer/`.msi`, Linux desktop GUI proof, Marketplace/OpenVSX publication, and fuller clean-machine reproduction evidence.
- Try-it path before any public launch wave: package registry beyond stub. The [browser playground](https://garnet-lang.org/playground.html) shipped.

See [F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md](F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md) for the runway, [F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md](F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md) for the rebuild workstream, and [CURRENT_STATE.md](CURRENT_STATE.md) for the source map.

## Who built this?

**Jon — Island Development Crew** (Huntsville AL). Doctoral research project; v3.3 → v4.2 development happened in collaboration with Claude (Opus 4.7). Every stage shipped under the discipline that pre-registered claims could only be downgraded to match proof when measurement disagreed, never re-rationalized post-hoc.

## I have a question that isn't answered here.

Open a Q&A discussion at [github.com/Island-Dev-Crew/garnet/discussions](https://github.com/Island-Dev-Crew/garnet/discussions), or use the question template at [github.com/Island-Dev-Crew/garnet/issues/new/choose](https://github.com/Island-Dev-Crew/garnet/issues/new/choose).

---

*"Where there is no vision, the people perish; but he that keepeth the law, happy is he." — Proverbs 29:18*
