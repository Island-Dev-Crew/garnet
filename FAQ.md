# Garnet — Frequently Asked Questions

Last updated: 2026-04-17 · v4.2

---

## What is Garnet?

A dual-mode, agent-native language platform. **Managed mode** (`def` + ARC + exceptions) feels Ruby-like. **Safe mode** (`@safe` + `fn` + ownership + `Result`) feels Rust-like. The **mode boundary** auto-bridges errors and ARC ↔ affine — so the same source file can host velocity-first orchestration code at the top level and rigor-first hot paths in `@safe` modules without any FFI between them.

## Why dual-mode? Why not just pick one?

Every team that builds ambitious software eventually makes the same bargain: Rust for the hot path, Ruby (or Python, Node) for the orchestration, painful FFI between. Garnet's claim is that bargain isn't necessary — the two registers of thought (the mathematical and the conversational) can live in one coherent grammar if the boundary between them is made visible. See [Paper III §1 "The Reconciliation"](A_Research_Papers/Paper_III_Garnet_Synthesis_v2_1.md) for the full argument.

## How does the mode boundary actually work?

A managed function calling a safe function sees raised exceptions where the safe code returned `Err(...)`. A safe function calling a managed function sees `Result<T, RaisedException>` where the managed code raised. ARC values flowing into safe scope auto-decay to affine references; affine values flowing back into managed scope re-promote to `Rc`. The compiler inserts the bridging adapters at the call sites; you write each function in the register that fits its job, and the boundary takes care of itself.

Every boundary crossing is logged in the `ModeAuditLog` (v3.5 Security Layer 3) — reviewers read one file to enumerate every trust boundary in the program.

## What's the capability model?

Every function declares its OS-authority budget via `@caps(...)`. A function with `@caps()` can do pure computation only — no filesystem, no network, no clock. A function with `@caps(fs)` can call `fs::read_file`, `fs::write_file`, etc. The compiler enforces this transitively: if `main()` calls `helper()` which calls `fs::read_file(...)`, then `main()` must declare `@caps(fs)` — or `helper()` must.

Known capabilities: `fs`, `net`, `net_internal` (lifts NetDefaults' RFC1918/loopback denial), `time`, `proc`, `ffi`, `*` (wildcard — managed mode only; safe-mode wildcard is a hard error).

The propagator runs at compile time; runtime cost is zero.

## Why Ed25519 signed manifests?

`garnet build --deterministic` produces a byte-identical manifest across machines (same source → same hash → same manifest, regardless of when or where you build). Adding `--sign <keyfile>` attaches an Ed25519 signature over the manifest. Anyone with the public key can run `garnet verify --signature` and confirm the binary they downloaded came from an authorized signer AND has not been tampered with.

This closes the "compiler impersonation" threat in v3.4 Security V2 §4. Hot-reload uses the same signing primitive (v3.5 ReloadKey).

## How does Garnet compare to Rust?

Garnet's safe mode IS Rust's mental model — ownership, borrow checking, `Result<T, E>`, `?` propagation, zero-cost abstractions. The distinction: Garnet doesn't force you to write the whole program in safe mode. The orchestration / scripting / glue layer can be `def`-managed mode; the hot path opts into `@safe fn`. You get Rust where you need it, not where you don't.

## How does Garnet compare to Ruby?

Garnet's managed mode IS Ruby's mental model — `def` + blocks + iterators + exceptions + ARC. The distinction: every function declares `@caps(...)` so there's no ambient authority, and the boundary to `@safe` modules gives you a place to put the code that absolutely must not have surprises. You get Ruby's velocity where it makes sense, with a typed, capability-checked safety net underneath when you need it.

## What about other dual-mode languages — Swift, Kotlin?

Swift and Kotlin do interop between two paradigms in one language; Garnet's pitch is that the *mode boundary is the reconciliation*. Swift's `unsafe` is an escape hatch for one specific concern; Garnet's `@safe`/`def` is a *first-class register choice* with auto-bridging at the boundary. Paper III §3 covers the comparative analysis in depth.

## What's the performance story?

For pure-computational workloads the tree-walk interpreter (Rung 3) is in the same order of magnitude as Ruby — appropriate for the v0.3 research release. The next rung (Rung 8 / v5.0) is a bytecode VM behind the same surface; that closes the gap to Rust on the hot path. See [Paper VII — Implementation Ladder and Tooling](A_Research_Papers/Paper_VII_Implementation_Ladder_and_Tooling.md) for the staged roadmap.

Memory: Paper VI Experiment 4 measured 21% peak RSS reduction on the multi-agent MVP workload by using kind-aware allocation (`memory working|episodic|semantic|procedural` keywords) compared to a force-malloc control.

## Is Garnet production-ready?

**v4.2 is research-grade.** Specifically:

- **Ready**: scaffolding (`garnet new`), the four-language converter (`garnet convert`), deterministic + signed builds (`garnet build --deterministic --sign`), CapCaps enforcement, scaffolded `garnet test`, the 22 bridged stdlib primitives. Linux + Windows binaries verified end-to-end.
- **Pending v4.3**: full socket-handle surface (`tcp_listen`, `udp_bind`, read/write/close), method-dispatch caps propagation for `arr.method()` calls, package-repo signing for `apt install garnet` / `dnf install garnet`.
- **Pending post-MIT**: bytecode VM for performance, Coq mechanization of Paper V theorems, LLM pass@1 study (Paper VI Exp 1 — pending API budget).

For prototype agents and scripting — green. For production-bearing infrastructure — wait for v5.0.

## How do I migrate from Ruby / Rust / Python / Go?

`garnet convert <lang> <file>` reads source in any of the four languages and emits Garnet. Every output file starts `@sandbox` + `@caps()` (v4.0 SandboxMode default — the converter never grants caps automatically; a human audits each file before lifting the sandbox via `@sandbox(unquarantine)` and adding the explicit `@caps(...)` based on what the code actually does).

The converter ships a lineage JSON for each output mapping every emitted Garnet AST node back to its source span. Cargo-style migration: convert one file at a time, FFI-call the rest, repeat until done. See [v4_1_Converter_Architecture.md](C_Language_Specification/v4_1_Converter_Architecture.md) for the full pipeline.

## What's `@sandbox` for?

A `@sandbox` annotation is the converter's "I produced this from another language; please don't trust me yet" header. While `@sandbox` is in effect, the function cannot be called from production code (the checker rejects the call site). A human reviewer reads the converted code, satisfies themselves it's safe, then changes `@sandbox` to `@sandbox(unquarantine)` and adds the appropriate `@caps(...)`. This is the audit gate that prevents converter output from silently entering a trusted code path.

## Where do I report bugs / request features?

[github.com/Island-Dev-Crew/garnet/issues](https://github.com/Island-Dev-Crew/garnet/issues). Use the bug report template for crashes / wrong outputs, the feature request template for proposals. For security disclosures, see [SECURITY.md](SECURITY.md) — please don't open public issues for vulnerabilities.

## Why the name "Garnet"?

Garnet is the gemstone that emerges from metamorphic pressure — it forms exactly where two registers of geological process (mineral chemistry and structural deformation) reconcile. Same metaphor: the language emerges from reconciling two registers of programming thought. Plus, the half-mechanical / half-faceted-gem logo visualizes the dual-mode story at a glance. (Also: GARNET → "GAR**N**ET" — letter N as the mode boundary.)

## What's the license?

Dual-licensed under MIT OR Apache-2.0 (your choice). See [LICENSE](LICENSE). Either license is fine for commercial use, including building proprietary applications on top of Garnet.

## Can I use it commercially?

Yes — the dual MIT / Apache-2.0 license explicitly permits commercial use, modification, distribution, and private use. The two licenses cover slightly different patent-grant + attribution-notice requirements; pick whichever fits your organization's policy.

## Do I need the Rust toolchain to use Garnet?

Not after release assets are published. The intended user install is `curl --proto '=https' --tlsv1.2 -sSf https://garnet-lang.org/install.sh | sh` (or a native `.deb` / `.rpm` / `.pkg` / `.msi` from [Releases](https://github.com/Island-Dev-Crew/garnet/releases)). Until the first `v0.4.2` GitHub Release is cut, use the source install from the README, which does require Rust.

To **build** Garnet from source you need Rust 1.95+ (managed via `rustup`). On Windows, MSVC toolchain is required (MinGW triggers a known miette ABI issue — see Boot doc Known Issue 1).

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

The full corpus in this repository — 7 research papers + 4 addenda, the canonical Mini-Spec v1.0, the engineering workspace (9 Rust crates, 10 MVP programs, 1244 tests), the DX Comparative Paper + Deck, and 11 stage handoff documents (v3.3 → v4.2). The complete history is reproducible commit-by-commit. See [GARNET_v4_2_HANDOFF.md](F_Project_Management/GARNET_v4_2_HANDOFF.md) §"Reviewer's 15-Minute Quickstart" for the suggested reading order.

## What's coming in v4.3?

- Full socket handle surface (`tcp_listen`, `udp_bind`, persistent stream handles)
- Method-dispatch caps propagation (`arr.contains(x)` style calls — currently caps_graph only walks free-function calls)
- Package-repository signing so `apt install garnet` / `dnf install garnet` works from `pkg.garnet-lang.org`
- Documentation site at `docs.garnet-lang.org` (mdBook scaffold)

See [GARNET_v4_2_HANDOFF.md](F_Project_Management/GARNET_v4_2_HANDOFF.md) §"v4.2 → POST-MIT ROADMAP" for the longer view.

## Who built this?

**Jon — Island Development Crew** (Huntsville AL). Doctoral research project; v3.3 → v4.2 development happened in collaboration with Claude (Opus 4.7). Every stage shipped under the discipline that pre-registered claims could only be downgraded honestly when measurement disagreed, never re-rationalized post-hoc.

## I have a question that isn't answered here.

Open a Q&A discussion at [github.com/Island-Dev-Crew/garnet/discussions](https://github.com/Island-Dev-Crew/garnet/discussions), or use the question template at [github.com/Island-Dev-Crew/garnet/issues/new/choose](https://github.com/Island-Dev-Crew/garnet/issues/new/choose).

---

*"Where there is no vision, the people perish; but he that keepeth the law, happy is he." — Proverbs 29:18*
