# Garnet — Frequently Asked Questions

Last updated: 2026-09-11 · latest tag <!-- truth:latest_tag -->v0.8.2<!-- /truth --> (research-grade milestone; the Release ships signed CLI binaries for macOS, Linux and Windows, plus an SBOM)

---

## What is Garnet?

A dual-mode, agent-native language platform. **Managed mode** (`def` + ARC + exceptions) feels Ruby-like. **Safe mode** (`@safe` + `fn` + ownership + `Result`) feels Rust-like. The **mode boundary** auto-bridges errors and ARC ↔ affine — so the same source file can host velocity-first orchestration code at the top level and rigor-first hot paths in `@safe` modules without any FFI between them.

## Why dual-mode? Why not just pick one?

Every team that builds ambitious software eventually makes the same bargain: Rust for the hot path, Ruby (or Python, Node) for the orchestration, painful FFI between. Garnet's claim is that bargain isn't necessary — the two registers of thought (the mathematical and the conversational) can live in one coherent grammar if the boundary between them is made visible. See [Paper III §1 "The Reconciliation"](A_Research_Papers/Paper_III_Garnet_Synthesis_v2_1.md) for the full argument.

## How does the mode boundary actually work?

A managed function calling a safe function sees raised exceptions where the safe code returned `Err(...)`. A safe function calling a managed function sees `Result<T, RaisedException>` where the managed code raised. ARC values flowing into safe scope auto-decay to affine references; affine values flowing back into managed scope re-promote to `Rc`. The compiler inserts the bridging adapters at the call sites; you write each function in the register that fits its job, and the boundary takes care of itself.

## What's the capability model?

Every function declares its OS-authority budget via `@caps(...)`: `@caps()` declares none, and `@caps(fs)` covers `fs::read_file`, `fs::write_file`, etc. `garnet check` follows named, acyclic calls from each annotated function and reports a declaration that leaves out a capability the chain needs: if `main()` declares `@caps()` and calls a `helper()` that calls `fs::read_file(...)`, the check reports that `main()` does not declare `fs`. Calls through function values, closures or call-graph cycles are not traced, an unannotated function's own body is not checked, and `garnet run` does not run the checker. At run time, the 15 entry-gated host primitives (file system, outbound network, process, environment, log-to-file) trap unless the program's entry function declares the capability; the `time` and UUID primitives are checked by `garnet check` only. See the [capability enforcement scope table](C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md).

Known capabilities: `fs`, `net`, `net_internal` (checker vocabulary; it does not change what `net::tcp_connect` may reach at run time), `env`, `time`, `proc`, `ffi` (checker vocabulary; there is no runtime FFI path), and `*` (wildcard — managed mode only; a safe-mode wildcard is a hard error).

The propagator runs at check time. The 15 gated primitives check the caps frames on each call; that cost is unmeasured.

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

Memory: Paper VI Experiment 4 measured 21% peak RSS reduction on the multi-agent MVP workload by using kind-aware allocation (`memory working|episodic|semantic|procedural` keywords) compared to a force-malloc control.

## Is Garnet production-ready?

**<!-- truth:latest_tag -->v0.8.2<!-- /truth --> is research-grade and not production-complete.** Specifically:

- **Ready**: scaffolding (`garnet new`), the four-language converter (`garnet convert`), deterministic + signed builds (`garnet build --deterministic --sign`), CapCaps checking plus the runtime entry gate on 15 primitives, scaffolded `garnet test`, the <!-- truth:primitive_count -->80<!-- /truth --> bridged stdlib registry primitives, parser fuzz harness, rules-based compiler advisory mode, the S16 LSP surface (diagnostics, hover, go-to-definition, document/workspace symbols, CST-precise rename), release-backed VSIX assets, signed Linux, macOS and Windows CLI release assets, and deterministic cross-machine CI.
- **Active-partial**: macOS Studio packaging without Developer ID notarization, Windows/Linux Studio target proof, bytecode VM performance path, LSP hover/go-to-def screenshot hardening, promo video human/aesthetic acceptance, proof/benchmark measurements, and provider-neutral advisory handoffs.
- **Pending**: Apple Developer ID notarization, signed `.pkg`, Windows `.msi`, Linux desktop package/runtime proof, Marketplace/OpenVSX publication, provider-backed LLM assist, mechanized proof, external empirical study data, and native backend lowering.

For prototype agents, research demos, and source-checkout dogfood, Garnet is useful today. For production-bearing infrastructure, keep waiting on the productization gates above.

## How do I migrate from Ruby / Rust / Python / Go?

`garnet convert <lang> <file>` reads source in any of the four languages and emits Garnet. Every output file starts with `@sandbox` and `@caps()`. Both are reviewer notes: the parser does not accept the `@sandbox` line, so the emitted file does not pass `garnet check` until a reviewer removes it and adds the `@caps(...)` the code needs.

The converter ships a lineage JSON for each output mapping every emitted Garnet AST node back to its source span. Migrate one file at a time; unconverted files stay in their source language. See [v4_1_Converter_Architecture.md](C_Language_Specification/v4_1_Converter_Architecture.md) for the full pipeline.

Input-dialect scope: each frontend targets a **stylized subset** of its source language, not the full grammar (untranslatable constructs become explicit `MigrateTodo` placeholders). The Ruby frontend targets a ≈3.3-era subset; newer syntax — Ruby 3.4 `it` block parameters, Ruby 4.0 leading-line logical-operator continuation — is not yet targeted.

## What's `@sandbox` for?

A `@sandbox` line is the converter's note that it produced the file from another language and granted no capabilities. Nothing enforces it: the parser rejects the line, so the file fails `garnet check` until a reviewer reads it, removes the line and adds the `@caps(...)` the code needs. The review is a human step, not a checker gate.

## Where do I report bugs / request features?

[github.com/Island-Dev-Crew/garnet/issues](https://github.com/Island-Dev-Crew/garnet/issues). Use the bug report template for crashes / wrong outputs, the feature request template for proposals. For security disclosures, see [SECURITY.md](SECURITY.md) — please don't open public issues for vulnerabilities.

## Why the name "Garnet"?

Garnet is the gemstone that emerges from metamorphic pressure — it forms exactly where two registers of geological process (mineral chemistry and structural deformation) reconcile. Same metaphor: the language emerges from reconciling two registers of programming thought. Plus, the half-mechanical / half-faceted-gem logo visualizes the dual-mode story at a glance. (Also: GARNET → "GAR**N**ET" — letter N as the mode boundary.)

## What's the license?

Dual-licensed under MIT OR Apache-2.0 (your choice). See [LICENSE](LICENSE). Either license is fine for commercial use, including building proprietary applications on top of Garnet.

## Can I use it commercially?

Yes — the dual MIT / Apache-2.0 license explicitly permits commercial use, modification, distribution, and private use. The two licenses cover slightly different patent-grant + attribution-notice requirements; pick whichever fits your organization's policy.

## Do I need the Rust toolchain to use Garnet?

Not on platforms with a matching published release asset. The installers (`install.sh`, and `install.ps1` on Windows) prefer the signed release asset for your platform, verifies it against `SHA256SUMS` (GPG-signed — see [`docs/release-signing.md`](docs/release-signing.md)), and uses source fallback only when no matching package exists or when you force `GARNET_INSTALL_MODE=source`. Source fallback requires Rust 1.95+ (the same floor as building from source below; Garnet CI tracks current stable).

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

## What's coming after v0.8.2?

- Front door + truth guard: machine-generated public numbers (`docs/truth.json`), README/site truth wiring, and version-narrative cleanup — the W-REBUILD RB-0 band.
- Foundation rebuild (zero language-semantics change): caps bitset, crash-surface sweep, registry-derived stdlib dispatch, parser-substrate unification, environment rebuild — then a Jon-gated backend-decision memo.
- Trust hardening (parallel lanes): independent re-verification of the self-found red-team fix, SLSA/Sigstore planning.
- Product gates: Apple Developer ID notarization, Windows installer/`.msi`, Linux desktop GUI proof, Marketplace/OpenVSX publication, and fuller clean-machine reproduction evidence.
- Try-it path before any public launch wave: browser playground, package registry beyond stub.

See [F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md](F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md) for the runway, [F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md](F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md) for the rebuild workstream, and [CURRENT_STATE.md](CURRENT_STATE.md) for the source map.

## Who built this?

**Jon — Island Development Crew** (Huntsville AL). Doctoral research project; v3.3 → v4.2 development happened in collaboration with Claude (Opus 4.7). Every stage shipped under the discipline that pre-registered claims could only be downgraded to match proof when measurement disagreed, never re-rationalized post-hoc.

## I have a question that isn't answered here.

Open a Q&A discussion at [github.com/Island-Dev-Crew/garnet/discussions](https://github.com/Island-Dev-Crew/garnet/discussions), or use the question template at [github.com/Island-Dev-Crew/garnet/issues/new/choose](https://github.com/Island-Dev-Crew/garnet/issues/new/choose).

---

*"Where there is no vision, the people perish; but he that keepeth the law, happy is he." — Proverbs 29:18*
