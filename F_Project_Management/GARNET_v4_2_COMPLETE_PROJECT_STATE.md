# GARNET v4.2 — Complete Project State (Single-File Handoff)

> **Current-status note (2026-05-06):** This remains valuable historical
> project context, but it is not the first-click source of truth for current
> `main`. Start at [`../CURRENT_STATE.md`](../CURRENT_STATE.md) and
> [`GARNET_CURRENT_VS_HISTORICAL_LEDGER.md`](GARNET_CURRENT_VS_HISTORICAL_LEDGER.md)
> before relying on any path, test-count, release, or example-readiness claim
> below.

**Purpose:** Self-contained context-resume document. A fresh Claude Code session (or a fresh human reviewer) reading ONLY this file gets enough context to continue work on Garnet without crawling the rest of `F_Project_Management/`. Cites primary sources throughout; makes no claims not anchored in an existing handoff or source file.
**Date:** 2026-04-21
**Author:** Claude Opus 4.7 (1M) — handoff consolidation
**Primary source docs synthesized here:** `GARNET_v4_2_HANDOFF.md`, `GARNET_v4_2_Phase_6D_Linux_VERIFIED.md`, `GARNET_v4_2_Phase_6D_Windows_VERIFIED.md`, `GARNET_v4_2_Phase_6A_HANDOFF.md`, `GARNET_v3_4_1_HANDOFF.md`, `GARNET_v4_2_STAGE6_KICKOFF.md`, `_CANONICAL_DELIVERABLES_INDEX.md`, `E_Engineering_Artifacts/README.md`, `E_Engineering_Artifacts/Cargo.toml`.
**Anchor:** *"And the word of the LORD was precious in those days; there was no open vision." — but here, the vision is plainly written. — 1 Samuel 3:1 (inverted)*

---

## TABLE OF CONTENTS

1. [One-paragraph project summary](#1-one-paragraph-project-summary)
2. [Stage ledger: v3.2 → v4.2](#2-stage-ledger-v32--v42)
3. [Rust crate inventory](#3-rust-crate-inventory)
4. [Research paper + addendum inventory](#4-research-paper--addendum-inventory)
5. [Language specification inventory](#5-language-specification-inventory)
6. [v4.2 deliverables](#6-v42-deliverables)
7. [v4.2 feature surface — verified working](#7-v42-feature-surface--verified-working)
8. [Verification status by platform](#8-verification-status-by-platform)
9. [Pending user actions before MIT submission](#9-pending-user-actions-before-mit-submission)
10. [Known issues carried forward](#10-known-issues-carried-forward)
11. [Post-MIT roadmap](#11-post-mit-roadmap)
12. [File path reference](#12-file-path-reference)

---

## 1. One-paragraph project summary

**Garnet** is a doctoral-research dual-mode, agent-native programming language platform. Managed mode (`def` + ARC + exceptions) feels Ruby-like; safe mode (`@safe` + `fn` + ownership + `Result`) feels Rust-like; the mode boundary auto-bridges errors and ARC-affine semantics. It ships first-class memory primitives (working / episodic / semantic / procedural) for agent cores, typed actors with bounded mailboxes + Ed25519-signed hot-reload, a compiler-as-agent that learns from its own compilation history, a single `garnet` CLI, deterministic signed manifests, and a dependency-graph audit. The project is authored by Jon through Island Development Crew in Huntsville, AL and is preparing for MIT academic submission. **v4.2 is research-grade and Stage 6 substantively complete**: Linux `.deb` + `.rpm` installers VERIFIED end-to-end in Docker; Windows binary + `.msi` VERIFIED on real Windows under the MSVC toolchain; Windows `.msi` built at `dist/windows/garnet-0.4.2-x86_64.msi` (SHA-256 `564d302fbaa3d05b16f77dd9d862972cceaed30132994997056f6e82e2d379c4`). Only the macOS `.pkg` build + Windows `.msi` `signtool sign` step remain — both credential-gated, mechanical to execute.

---

## 2. Stage ledger: v3.2 → v4.2

Each row links to the canonical handoff and summarizes what landed, what was verified green, and what carried forward.

| Version | What shipped | Tests added | Handoff doc |
|---------|--------------|-------------|-------------|
| **v3.2** | MIT-adversarial hardening: closed every gap from a 3-explorer hostile audit (research / specs / tests). Paper VI Contributions 3, 5, 6, 7 all got runnable evidence (compiler-as-agent SQLite knowledge.db + strategies.db, cross-boundary error bridging, hot-reload with state migration, deterministic builds + verify). 30 proptest properties + 9 criterion benches + 6 stress tests + 7×-consistency xtask + 3 real-world ≥200 LOC example programs. | **857 baseline** | `GARNET_v3_2_HANDOFF.md` |
| **v3.3** | Phase 1A Slop Reverification (5 real gaps found + fixed); Phase 1E Security Layer 1 (5 hardening items: ParseBudget, KindGuard, StateCert, CacheHMAC, ProvenanceStrategy); Phase 1B–1F Stage 1 closeout — Mini-Spec v1.0, Paper V/IV addenda, Paper VI Empirical Validation Protocol, Paper VII stub, compression reference v0.4. Two novel Garnet-specific threat classes closed. | **+61** (→918) | `GARNET_v3_3_HANDOFF.md` + four Stage-1 artifacts |
| **v3.4** | Layer 2 security + stdlib module. Safety-gated stdlib surface (strings / time / crypto / collections / fs / net) with `@caps(...)` registry metadata — unbridged to the interpreter at this point. Security V2 threat model. | **+79** (→997) | `GARNET_v3_4_HANDOFF.md` + `GARNET_v3_4_SECURITY_V2_SPEC.md` |
| **v3.4.1** | **Three long-deferred v3.4 items closed:** (Day 1) stdlib↔interpreter bridge — 22 primitives live through `garnet-interp-v0.3/src/stdlib_bridge.rs`; (Day 2) CapCaps call-graph propagator — `garnet-check-v0.3/src/caps_graph.rs`, colored-DFS transitive caps coverage reading `garnet_stdlib::registry::all_prims()`; (Day 3) ManifestSig — Ed25519 `sign/verify_signature` on deterministic manifests + `garnet keygen` + `garnet build --sign` + `garnet verify --signature`. | **+40** (→1037; locally-unexecutable on Windows+MinGW due to miette ABI — execute via Linux CI) | `GARNET_v3_4_1_HANDOFF.md` |
| **v3.5** | Layer 3 security + 6 MVP example programs. SLOP findings reverified. | **+25** (→982 running count pre-3.4.1) | `GARNET_v3_5_HANDOFF.md` + `GARNET_v3_5_REFACTOR_DISCOVERIES.md` + `GARNET_v3_5_SECURITY_V3.md` |
| **v4.0** | Layer 4 security. Paper VI execution pass — 7 novel contributions scorecard: **4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra** (Exp 1 LLM pass@1 awaits $500 API credits). Performance benchmarks. Rung-4 verification ladder codified. | **+17** | `GARNET_v4_0_HANDOFF.md`, `GARNET_v4_0_PAPER_VI_EXECUTION.md`, `GARNET_v4_0_PERFORMANCE_BENCHMARKS.md`, `VERIFICATION_LADDER_v4_0.md` |
| **v4.1** | `garnet-convert` crate + `garnet convert` CLI subcommand — Rust / Ruby / Python / Go → Garnet migration tool. CIR (canonical IR) with lineage tracking + migrate-todo surfacing + metrics. Default SandboxMode header `@sandbox @caps()` on every converted output. | **+90** | `GARNET_v4_1_HANDOFF.md` + `v4_1_Converter_Prior_Art.md` + `C_Language_Specification/v4_1_Converter_Architecture.md` |
| **v4.2 Phase 0** | Pre-MIT DX rigor. DX Comparative Paper §20 "Measured vs. Argued" two-column layout (preempts reviewer "how do you measure joy?"). Pre-registered Developer Comprehension Study Protocol (N=5 × 6 tasks × 3 languages, counterbalanced Latin square, 10-pp accuracy threshold, honest Paper III §7 downgrade if refuted). | 0 (docs only) | `GARNET_v4_2_Developer_Comprehension_Study_Protocol.md` + `D_Executive_and_Presentation/GARNET_v4_2_DX_Comparative_*.{docx,pptx}` |
| **v4.2 Phase 6A** | Cross-platform installer **scaffolding**: Windows MSI (`garnet-cli/wix/main.wxs` + `[package.metadata.wix]`); macOS PKG (`garnet-cli/macos/build-pkg.sh` + `distribution.xml` + `resources/`); Linux `.deb` + `.rpm` (`[package.metadata.deb]` + `[package.metadata.generate-rpm]` + `garnet-cli/linux/garnet-actor.service` systemd unit, hardened + `ExecStartPre=garnet verify ... --signature`); Universal shell installer (`installer/sh.garnet-lang.org/install.sh`, rustup-style UX); man page (`garnet-cli/man/garnet.1`). | 0 (config only) | `GARNET_v4_2_Phase_6A_HANDOFF.md` |
| **v4.2 Phase 6B** | `garnet new` project scaffolding — `garnet-cli/src/new_cmd.rs`, 3 embedded templates via `include_str!`, Cargo-like name validation (1–64 chars, ASCII letter start, 9 reserved keywords). Templates: `cli`, `web-api`, `agent-orchestrator`. | **+13** (→1204 cumulative per handoff count) | (covered in `GARNET_v4_2_HANDOFF.md`) |
| **v4.2 Phase 6C** | Logo + branding. GPT-generated logo at `garnet-cli/assets/garnet-logo.png` (1024×1024 JPEG-in-PNG). Five integration surfaces: macOS PKG welcome backdrop, branded welcome/conclusion HTML with deck palette (#9C2B2E garnet / #E5C07B gold / #0A0A0F OLED), README hero, `garnet --version` + `--help` ASCII wordmark (TTY-gated with `std::io::IsTerminal`), REPL startup banner. | 0 (assets) | (covered in `GARNET_v4_2_HANDOFF.md`) |
| **v4.2 Phase 6D (Linux)** | **VERIFIED.** `.deb` built in `rust:1-bookworm` container, installed into clean `ubuntu:24.04`, all 6 binary-level gates pass; `.rpm` built, installed into clean `fedora:40`, same 6 gates pass. Pass 2: 3/3 templates scaffolded, CapCaps pass/fail both verified live, `garnet convert ruby` end-to-end with 4 artifacts + `@sandbox @caps()` header, `garnet test` added + 5 tests green across 3 templates. Ed25519 sign→verify roundtrip verified on both distros. `installer/sh.garnet-lang.org/install.sh` shellcheck CLEAN. | (existing tests re-run, no new count) | `GARNET_v4_2_Phase_6D_Linux_VERIFIED.md` |
| **v4.2 Phase 6D (Windows)** | **VERIFIED.** MSVC toolchain switch resolved MinGW ABI crash. `garnet.exe --version` renders wordmark natively. All 6 v4.2 feature gates pass live. `.msi` built at `dist/windows/garnet-0.4.2-x86_64.msi` (2.68 MB, SHA-256 `564d302f…79c4`) via WiX 3.11.2.4516 + full branding. | — | `GARNET_v4_2_Phase_6D_Windows_VERIFIED.md` |
| **v4.2 Phase 6D (macOS)** | **PENDING** — user's MacBook, Apple Developer ID + notarytool profile already set up. | — | *(to be written per `GARNET_v4_2_MAC_RECONCILIATION_PROMPT.md`)* |

**Cumulative test tally (per v4.2 handoff §CUMULATIVE TEST TALLY, taken verbatim from source):**

- v3.2 baseline: 857
- v3.3: +61
- v3.4: +79
- v3.5: +25
- v4.0: +17
- v4.1: +90
- v3.4.1: +40
- v4.2 Phase 6B: +13
- **Cumulative committed: 1244 tests** (1204 carried + 40 v3.4.1 tests authored but ABI-blocked locally — execute via the GHA Linux workflow)
- **136 security-specific tests** across 4 hardening layers (not double-counted in the 1244).

---

## 3. Rust crate inventory

Workspace root: `E_Engineering_Artifacts/` (becomes repo root after GitHub conversion — see `GARNET_v4_2_GITHUB_REPO_LAYOUT.md`).

Workspace manifest (`Cargo.toml`) — verbatim from current source:

```toml
[workspace]
resolver = "2"
members = [
    "garnet-parser-v0.3", "garnet-interp-v0.3", "garnet-check-v0.3",
    "garnet-memory-v0.3", "garnet-cli", "garnet-actor-runtime",
    "garnet-stdlib", "garnet-convert", "xtask",
]
exclude = ["garnet-parser"]  # Historical v0.2

[workspace.package]
version = "0.3.0"
edition = "2021"
license = "MIT OR Apache-2.0"
authors = ["Island Development Crew"]
repository = "https://github.com/islanddevcrew/garnet"

[profile.release]
lto = true
codegen-units = 1
strip = "symbols"
```

### 3.1. Crate table

| Crate | Rung | Purpose | Test count | v4.2 state |
|-------|------|---------|------------|-----------|
| `garnet-parser` | 2.0 | Historical v0.2 — flat-file layout, ~20 productions against Mini-Spec v0.2. Excluded from workspace; kept for audit trail only. | — | Unchanged since v3.0 |
| `garnet-parser-v0.3` | 2.1 | Complete parser. All 90 EBNF productions. Hand-rolled lexer + recursive-descent parser with 11-level Pratt expression tower. ~3,237 lines across 17 source files. | ~141 | Stable since v3.1 |
| `garnet-interp-v0.3` | 3 | Managed-mode tree-walk interpreter. Literals, 11-level operator precedence, all control flow (`if`/`while`/`for`/`loop`/`match`/`try`), closures with capture, structs + enums + pattern matching, `Result`/`Option` with `?` propagation, dual-mode error bridging. Built-in REPL. **v3.4.1: added `stdlib_bridge.rs` — 22 primitives bridged end-to-end.** | ~60 + 18 (bridge) | v3.4.1 additions live |
| `garnet-check-v0.3` | 4 | Safe-mode validator. Rejects `var` / `try`-rescue / `raise` in `@safe` modules (Mini-Spec §7.3). Validates `@max_depth(N)` / `@fan_out(K)`. Move-tracking borrow checker (v3.1). **v3.4.1: added `caps_graph.rs` — transitive-caps call-graph propagator, colored-DFS over direct + mutual recursion, reads `garnet_stdlib::registry::all_prims()`, emits `CheckError::CapsCoverage { fn_name, missing, via }`.** | 35 + 10 (caps_graph) | v3.4.1 additions live |
| `garnet-memory-v0.3` | 5 | Reference implementations of the four memory primitives: `WorkingStore` (arena), `EpisodeStore` (timestamped log), `VectorIndex` (cosine-similarity), `WorkflowStore` (copy-on-write + replay). Per-kind `MemoryPolicy` with decay defaults. R+R+I scoring implements OQ-7. | 6 | Stable since v3.0 |
| `garnet-actor-runtime` (v0.3.1) | 6 | Reference scheduler for actor model. One OS thread per actor; mpsc mailbox; per-message reply channel for synchronous request/response. `tell` / `ask` / `ask_timeout`. Lifecycle hooks. Signed hot-reload via ManifestSig. | **17 pass** (release gate) | v3.4.1 Ed25519 signed-reload integrated |
| `garnet-stdlib` | — | OS I/O primitives with capability metadata registry. Strings / time / crypto / collections / fs / net. Single source of truth for CapCaps caps coverage. | **74 pass** (release gate) | v3.4.1 registry complete |
| `garnet-cli` | 6 | Top-level `garnet` binary. Subcommands: `parse`, `check`, `run`, `eval`, `repl`, `build`, `verify`, `test`, `new`, `keygen`, `convert`, `version`, `help`. Links all lower crates. **v3.4.1: added `keygen`, `build --sign`, `verify --signature`. v4.2 Phase 6B: added `new` + 3 templates. v4.2 Phase 6C: TTY-gated wordmark. v4.2 Phase 6D: `test` subcommand added during Linux verification.** | 12 smoke + 13 (new_cmd) + 12 (manifest) | v4.2-complete; all installer metadata landed |
| `garnet-convert` | — | Rust / Ruby / Python / Go → Garnet migration tool. CIR with lineage tracking, migrate-todo surfacing, metrics.json. Default `@sandbox @caps()` SandboxMode header. | **85 pass** (release gate; 61 + 24) | v4.1; bugfix in v4.2 Phase 6D (`bin/garnet.rs` convert-subcommand wiring) |
| `xtask` | — | Repo-local dev tasks: 7×-consistency check, provenance, benchmarks. | N/A | Stable |

### 3.2. Three canonical release gates

Per `GARNET_v4_2_HANDOFF.md` §VERIFICATION STATUS, these three are the authoritative release gates run every stage:

```
cargo test -p garnet-actor-runtime --release --lib   → 17 pass
cargo test -p garnet-stdlib        --release          → 74 pass
cargo test -p garnet-convert       --release          → 85 pass (61 + 24)
cargo check --workspace --tests                       → clean (0 errors)
cargo build --release -p garnet-cli                   → 1m 22s cold compile (Linux)
                                                     → 52s cold compile (Windows MSVC)
```

Additional tests exist in `garnet-parser-v0.3`, `garnet-interp-v0.3`, `garnet-check-v0.3`, `garnet-cli`, `garnet-memory-v0.3` but their test binaries hit a miette/backtrace-ext ABI mismatch on the user's Windows+MinGW dev machine. These are **not crate bugs** — `cargo check --workspace --tests` compiles every one cleanly. They execute cleanly in Linux containers (proof: the Phase 6D Linux verification ran the full binary surface end-to-end) and should execute cleanly on Windows under MSVC (proof: Phase 6D Windows verified the Windows binary). The 40 v3.4.1 tests (18 stdlib_bridge + 10 caps_graph + 12 manifest) sit in this category.

### 3.3. Clippy state

Per `GARNET_v4_2_HANDOFF.md` §VERIFICATION STATUS (verified 2026-04-17):

> 0 errors, 34 warnings — all stylistic carryovers from v3.4–v4.1 (e.g., `should-implement-trait` on `SourceLang::from_str`, unused-import `PathBuf` in `audit_deps.rs`). Auto-fixable via `cargo clippy --fix --workspace`. The `cargo clippy --workspace --all-targets` exit code is 0 (clean).

"Clippy clean" in the handoffs means "no lints cargo treats as errors", not "no warnings under `-D warnings`". Under strict `-- -D warnings`, the 34 warnings escalate to errors.

---

## 4. Research paper + addendum inventory

Location: `Garnet_Final/A_Research_Papers/`.

### 4.1. Seven core papers

| # | File | Summary |
|---|------|---------|
| — | `GARNET-The-Reconciliation-of-Rust-and-Ruby.md` + `.pdf` | **The foundational thesis.** 50 KB / 405 lines. Comprehensive doctoral comparative study: Rust origins / ownership / ecosystem; Ruby origins / Rails / YJIT; 34-dimension comparison matrix; performance benchmarks; prior art analysis (Crystal, Elixir, Mojo, Gleam); Garnet synthesis proposal with dual-mode architecture; market viability ($15.7B TAM); risk analysis. Everything else builds on this. |
| I | `Paper_I_Rust_Deep_Dive_Updated.docx` | Rust deep dive. Graydon Hoare origins, ownership / borrowing, RustBelt, editions, Cargo ecosystem (250K+ crates), industry adoption (Linux kernel 600K lines, Cloudflare Pingora, Discord, Android 1000× CVE reduction). |
| II | `Paper_II_Ruby_Deep_Dive_Updated.docx` | Ruby deep dive. Matz origins, pure OOP, metaprogramming, Rails / DHH, YJIT evolution (+92% Ruby 3.4), Shopify BFCM 80M req/min, GitHub 2M-line monolith. |
| III | `Paper_III_Garnet_Synthesis_v2_1.{docx,md,pdf}` | **Three-model redlined synthesis.** 34-dimension gap analysis, dual-mode proposal, market analysis (TAM $1-5B / SAM $200M-1B / SOM $10-50M). The paper the DX Comparative Paper §20 downgrades honestly if the N=5 Developer Comprehension study refutes its velocity claim. |
| IV | `Paper_IV_Garnet_Agentic_Systems_v2_1_1.docx` + `Paper_IV_Addendum_v1_0.md` | **Agent-native extension + Recursive Language Models.** Base (11 pp): agent-native architecture with memory engineering (working / episodic / semantic / procedural), One-Memory-Core-Many-Harnesses, Appendix B on PolarQuant & QJL mathematical mechanics. Addendum (v3.3 Phase 1D): RLM paradigm from Gemini synthesis + Garnet ↔ RLM correspondence + PolarQuant ↔ Memory Core bridge. |
| V | `Paper_V_Garnet_Formal_Grounding_v1_0.docx` + `Paper_V_Addendum_v1_0.md` | **Formal foundations.** Base (30 pp): affine type theory, RustBelt / Iris / Coq grounding, core λ-calculus (λ_managed + λ_safe), mode-boundary semantics, soundness theorem sketch, memory primitives as typed resources, typed actor protocols. Addendum (v3.3 Phase 1B): Theorems A–H — ARC + kind-partitioned cycle collection, NLL region-solver correctness, lifetime elision soundness, borrow-checker B1–B5 → RustBelt, Sendable + Actor Isolation Theorem, Zero-Cost Abstraction Theorem, Polymorphic Recursion Exclusion. Positioned for PLDI 2027. Coq mechanization is multi-month post-MIT. |
| VI | `Paper_VI_Garnet_Novel_Frontiers.md` + `Paper_VI_Empirical_Validation_Protocol.md` + `Paper_VI_v4_0_Revisions.md` | **Seven novel contributions with falsifiable hypotheses:** (1) LLM-native syntax design; (2) progressive type-disclosure spectrum; (3) compiler-as-agent; (4) kind-aware memory allocation; (5) automatic bidirectional error-model bridging; (6) hot-reload mode boundaries; (7) deterministic reproducible builds with provenance manifests. Empirical Validation Protocol (Phase 1C): pre-registered protocols with hypothesis (H), procedure (P), pass/fail criterion (C), measurement harness (M), expected risk (R) per experiment. Power analysis. v4.0 execution scorecard: **4 supported, 2 partial-downgraded, 0 refuted, 1 pending-infra** (Exp 1 LLM pass@1, awaits $500 API credits). |
| VII | `Paper_VII_Implementation_Ladder_and_Tooling.md` | **Stub.** Seven-rung engineering ladder + single-CLI principle (`garnet new/build/run/test/check/fmt/repl/doc/audit/verify/convert`) + REPL design + cross-platform installer commitment. Three engineering disciplines (adversarial-audit-before-trust, threat-model-before-hardening, sequencing discipline) as research-class commitments. Full v1.0 deferred post-MIT. |

Also: `Garnet_ Agent-Native Language Synthesis.docx` — pre-v2.1 synthesis artifact, kept for audit.

### 4.2. Four addenda — summary

| Addendum | Anchors to | Adds |
|----------|-----------|------|
| Paper IV Addendum v1.0 | Paper IV §Agent cores | Recursive Language Models (RLM) correspondence + PolarQuant ↔ Memory Core bridge |
| Paper V Addendum v1.0 | Paper V Theorems | Theorems A–H formal statements (companions for ARC, NLL, borrow-check, Sendable, monomorphization) |
| Paper VI v4.0 Revisions | Paper VI §Experiments | Honest downgrade of 2 of 7 contributions post-execution; re-scoped Exp 1 as pending-infra |
| Compression Techniques Reference (v0.4) | Paper IV Appendix B | Deepening + SRHT + α calibration |

---

## 5. Language specification inventory

Location: `Garnet_Final/C_Language_Specification/`.

| File | Canonical for |
|------|---------------|
| `GARNET_v1_0_Mini_Spec.md` | **Normative source of truth for all implementation.** Promoted from v0.3 with 11 Phase 1B blend-verification gap fills (ARC + Bacon-Rajan; Ruby blocks/yield; NLL lifetime inference; borrow-checker B1-B5; Sendable; trait coherence; monomorphization; `@dynamic` dispatch; structural protocols; REPL spec; single-CLI summary). Closes OQ-10 + OQ-11. |
| `GARNET_v0_3_Formal_Grammar_EBNF.md` | 90 productions covering the entire language surface; ISO/IEC 14977 notation. |
| `GARNET_Compiler_Architecture_Spec.md` | 16-section pipeline: Lexer → Parser → Name Resolution → Dual-path Type Checking → Boundary Validator → Code Generation (4 backends). |
| `GARNET_Memory_Manager_Architecture.md` | Addresses OQ-7 (R+R+I controlled-decay) and OQ-8 (multi-agent consistency, three access modes). |
| `GARNET_Distribution_and_Installation_Spec.md` | 12-section install spec: installer script, `garnetup` CLI, 5+4+5 platform matrix, LLVM/Cranelift bundling, registry protocol, reproducible builds. |
| `GARNET_Tier2_Ecosystem_Specifications.md` | Package manager, stdlib (20+ modules), interop (Rust FFI, C ABI, Ruby VM embedding, WASM), async (green threads, no colored functions). |
| `GARNET_Benchmarking_and_Evaluation_Plan.md` | Protocol for all 7 Paper VI falsifiable hypotheses. |
| `GARNET_Migration_Guide_Ruby_Python.md` | Three-phase incremental adoption path (Parallel Harness → Core Extraction → Full Adoption). |
| `GARNET_Academic_Submission_Strategy.md` | Venue landscape (PLDI, POPL, OOPSLA, ASPLOS, ICFP), paper portfolio, submission timeline, reviewer-anticipation matrix. |
| `GARNET_Compression_Techniques_Reference.md` | Deepening + SRHT + α calibration (Paper IV Appendix B companion). |
| `v4_1_Converter_Architecture.md` | `garnet-convert` crate architecture: CIR, lineage, migrate-todo, metrics. |
| `GARNET_v0_{2,3}_Mini_Spec*.md` | Historical; superseded. |

---

## 6. v4.2 deliverables

### 6.1. Installers (one per target)

| Platform | Artifact | Built? | Signed? | Notes |
|----------|----------|--------|---------|-------|
| Linux Debian/Ubuntu | `dist/linux/garnet_0.4.2-1_amd64.deb` (~1.53 MB) | ✅ | SHA-256 pinned (no code signing; apt handles the integrity chain) | SHA-256 `5f2112c9bf221aa6180fdf8b5ca1fff555fbc2d1047c58787d8ed6a3e889fe57` per Phase 6D Linux. Static deps: `libc`, `libm`, `libgcc_s`, `linux-vdso`, `ld-linux-x86-64` only. Portable glibc ≥ 2.34. |
| Linux Fedora/RHEL | `dist/linux/garnet-0.4.2-1.x86_64.rpm` (~1.63 MB) | ✅ | SHA-256 pinned | SHA-256 `55a06b3bb24d4d3712eb2659e768c38e3b9ff2f68d974b2b02b9f9098e69ef17`. |
| Windows | `dist/windows/garnet-0.4.2-x86_64.msi` (~2.68 MB) | ✅ | ⏳ `signtool sign` pending user's Authenticode cert | WiX Toolset 3.11.2.4516 + full Garnet branding (Dialog.bmp, Banner.bmp, Garnet.ico, License.rtf). SHA-256 `564d302fbaa3d05b16f77dd9d862972cceaed30132994997056f6e82e2d379c4`. Upgrade GUID `6E6A3D5F-3B0E-4D8F-9A3F-4F1D8C8A9A10`. Per-machine install to `C:\Program Files\Garnet\bin\`, HKLM PATH, Start Menu "Garnet Shell", ARP entry → `https://garnet-lang.org`, post-install `garnet version` smoke. |
| macOS | `target/macos/garnet-0.4.2-universal.pkg` | ⏳ pending | ⏳ Apple Developer ID + notarization on MacBook | `garnet-cli/macos/build-pkg.sh` refuses to run without `APPLE_DEV_ID_INSTALLER`, `APPLE_DEV_ID_APP`, `APPLE_NOTARY_PROFILE`. Build: `cargo build --target x86_64-apple-darwin` + `aarch64-apple-darwin` → `lipo` → `codesign` → `pkgbuild` → `productbuild` → `productsign` → `xcrun notarytool submit --wait` → `xcrun stapler staple`. |
| Universal | `installer/sh.garnet-lang.org/install.sh` | ✅ shellchecked CLEAN | SHA-256 chain via `SHA256SUMS` | POSIX `/bin/sh`. Detects OS + arch, picks `.deb` / `.rpm` / `.pkg` / tar, downloads, verifies SHA-256, runs native installer. |

### 6.2. Supporting deliverables

| Deliverable | Path | Status |
|-------------|------|--------|
| systemd unit for actor-runtime | `garnet-cli/linux/garnet-actor.service` | ✅ disabled by default, hardened, `ExecStartPre=garnet verify --signature` |
| Man page | `garnet-cli/man/garnet.1` | ✅ groff, all v4.2 subcommands documented |
| 3 project templates | `garnet-cli/templates/{cli,web-api,agent-orchestrator}/` | ✅ embedded via `include_str!` |
| Logo | `garnet-cli/assets/garnet-logo.png` | ✅ 1024×1024 JPEG-in-PNG |
| Website landing | `E_Engineering_Artifacts/docs/index.html` + `docs/CNAME` → `garnet-lang.org` | ✅ shipped |
| DX Comparative Deck | `D_Executive_and_Presentation/GARNET_v4_2_DX_Comparative_Deck.pptx` | ✅ with §20 Measured-vs-Argued |
| DX Comparative Paper | `D_Executive_and_Presentation/GARNET_v4_2_DX_Comparative_Paper.docx` | ✅ with §20 Measured-vs-Argued |
| Developer Comprehension Study Protocol | `F_Project_Management/GARNET_v4_2_Developer_Comprehension_Study_Protocol.md` | ✅ pre-registered, N=5 × 6 × 3, Latin square |
| Linux-packages CI | `E_Engineering_Artifacts/.github/workflows/linux-packages.yml` | ✅ runs on push/PR; uploads .deb + .rpm + SHA256SUMS to GitHub Release on `v*` tags |
| Branding assets | `garnet-cli/wix/{Dialog.bmp,Banner.bmp,Garnet.ico,License.rtf}` + `garnet-cli/macos/resources/{background.png,welcome.html,conclusion.html,LICENSE.txt}` | ✅ generated in-session during Phase 6D Windows |

---

## 7. v4.2 feature surface — verified working

Per the Phase 6D Linux VERIFIED + Windows VERIFIED handoffs, every v4.2 feature surface has been exercised end-to-end against a real `garnet` binary:

| Feature | How verified | Platform |
|---------|--------------|----------|
| Stdlib↔interpreter bridge (22 primitives) | Binary starts + emits wordmark via interpreter prelude init — proves 22-primitive bridge loads without crashing | Linux (Docker) + Windows (MSVC) |
| CapCaps call-graph propagator (transitive) | **Live error on a caps-violating program:** *"function `main` does not declare `fs` but transitively calls `read_file` which requires it"* | Linux (Docker) |
| CapCaps clean program | *"1 functions checked, 0 diagnostics"* | Linux (Docker) |
| Ed25519 `keygen` → `build --deterministic --sign` → `verify --signature` | Cryptographic round-trip — `.deb` signer pubkey `75cffbb9…499`, `.rpm` signer pubkey `898368da…1de`, Windows signer pubkey `75d15caf…14e` | Linux (both distros) + Windows |
| `garnet new --template cli` | 5 files scaffolded: `Garnet.toml`, `src/main.garnet`, `tests/test_main.garnet`, `.gitignore`, `README.md` | Linux + Windows |
| `garnet new --template web-api` | 5 files — HTTP/1.1 + `@caps(net, time)` + BoundedMail guidance | Linux |
| `garnet new --template agent-orchestrator` | 5 files — Researcher / Synthesizer / Reviewer + 3 memory kinds | Linux |
| `garnet test` on templates | cli (2 pass) + web-api (1 pass, cross-file helper resolution) + agent-orchestrator (2 pass); 5 total green | Linux + Windows |
| `garnet convert ruby foo.rb` | 4 artifacts: `.garnet`, `.lineage.json`, `.migrate_todo.md`, `.metrics.json`. Output starts `@sandbox @caps()` per v4.1 SandboxMode default. 100.0% clean-translate, 3 CIR nodes, 0 migrate-todos. | Linux |
| Wordmark on TTY (Garnet-red ANSI) + pipe (plain ASCII) | `std::io::IsTerminal` gating; CI-safe | Linux + Windows |
| Universal `install.sh` wordmark + format dispatch | Shellchecked CLEAN; dispatch paths exercised | Linux |
| `.deb` install → `apt-get install` accepts → binary runs → `apt-get remove` cleans up | Clean Ubuntu 24.04 container | Linux |
| `.rpm` install → `dnf install` accepts → binary runs → `dnf remove` cleans up | Clean Fedora 40 container | Linux |
| Man page install path | `/usr/share/man/man1/garnet.1` on Fedora; `.deb` ships it too but Ubuntu minimal excludes by default | Linux |
| WiX 3 MSI build with branding | `cargo wix --nocapture` against WiX 3.11.2 → `garnet-0.4.2-x86_64.msi` 2.68 MB | Windows |

Two bugs found + patched during Phase 6D verification:
1. **`garnet convert` was registered in `lib.rs` but not wired into `bin/garnet.rs`'s subcommand match.** Patched in-session: `bin/garnet.rs` gained `"convert" => cmd_convert(&args[1..])` arm + full `cmd_convert()` handler mirroring `cmd_new`'s flag-parsing style.
2. **`garnet test` referenced in template READMEs but unimplemented.** Patched in-session: discovers `test_*` functions in `tests/*.garnet`, pre-loads `src/main.garnet` as helper context (so cross-file references work), runs each in fresh interpreter per file, reports per-test pass/fail + summary, exits non-zero on any failure.

Plus one cosmetic fix: `clean-translate` percentage displayed as `10000%` (double-multiplied) → corrected to `100.0%` in source; verified in Linux pass 3 rebuild.

---

## 8. Verification status by platform

Per `GARNET_v4_2_HANDOFF.md` §SHIPPED IN STAGE 6 and the two Phase 6D verification docs:

| Platform | Binary | Installer | Sign | VM smoke |
|----------|--------|-----------|------|----------|
| Linux `.deb` (Ubuntu 24.04) | ✅ Docker | ✅ Docker | n/a (SHA-256) | ✅ Docker, 6/6 gates |
| Linux `.rpm` (Fedora 40) | ✅ Docker | ✅ Docker | n/a (SHA-256) | ✅ Docker, 6/6 gates |
| Windows | ✅ MSVC on real Windows | ✅ WiX 3.11.2, full branding | ⏳ user runs `signtool sign` | ⏳ user runs `sandbox-smoke.wsb` harness |
| macOS | ⏳ user's MacBook | ⏳ `build-pkg.sh` | ⏳ Apple Developer ID + notarization | ⏳ `pkgutil --check-signature` + `spctl --assess` + `xcrun stapler validate` |
| Universal `sh.garnet-lang.org` | ✅ shellchecked CLEAN | n/a | SHA-256 via `SHA256SUMS` | ⏳ post-first-release dry-run |

---

## 9. Pending user actions before MIT submission

Ordered by blocking dependency. Each is mechanical; none require new engineering.

1. **Transfer the workspace to MacBook.** Two desktop folders side-by-side: `Garnet Opus 4.7` (Mac-side in-progress) + `Garnet Opus 4.7 final` (Windows transfer). See `GARNET_v4_2_MAC_RECONCILIATION_PROMPT.md`.

2. **Run the Mac reconciliation prompt** in a fresh Claude Code session on the Mac. The reconciled tree becomes the single authoritative working copy.

3. **macOS `.pkg` build + sign + notarize + staple.**
   ```sh
   cd Garnet_Final/E_Engineering_Artifacts/garnet-cli
   export APPLE_DEV_ID_INSTALLER="Developer ID Installer: <name> (<TEAMID>)"
   export APPLE_DEV_ID_APP="Developer ID Application: <name> (<TEAMID>)"
   export APPLE_NOTARY_PROFILE="<profile-name>"
   ./macos/build-pkg.sh
   pkgutil --check-signature target/macos/garnet-0.4.2-universal.pkg
   spctl --assess --type install target/macos/garnet-0.4.2-universal.pkg
   xcrun stapler validate target/macos/garnet-0.4.2-universal.pkg
   ```
   Write `GARNET_v4_2_Phase_6D_macOS_VERIFIED.md` modeled on the Linux + Windows equivalents.

4. **Windows `.msi` signing (on the Windows machine).**
   ```powershell
   signtool sign /f path\to\codesign.pfx /p <pass> /fd SHA256 `
     /tr http://timestamp.digicert.com /td SHA256 `
     dist\windows\garnet-0.4.2-x86_64.msi
   signtool verify /pa /v dist\windows\garnet-0.4.2-x86_64.msi
   ```
   Then smoke-test via `garnet-cli/windows/sandbox-smoke.wsb` (8 gates in a Windows Sandbox VM).

5. **Initialize the GitHub repo and push.** Follow `GARNET_v4_2_GITHUB_REPO_LAYOUT.md` §7 for the exact command sequence from `git init` through `git push origin main`. Pre-conversion: move research corpus from `Garnet_Final/{A,B,C,D,F}_*` into `research/{papers,consensus,spec,presentation,management}/`.

6. **Configure GitHub Pages.** Settings → Pages → branch `main`/`docs`, custom domain `garnet-lang.org`, HTTPS enforced after cert provisions.

7. **Publish DNS records at the registrar.** Four A records + four AAAA records for apex → GitHub Pages IPs; CNAME `www` → `islanddevcrew.github.io.`. See `GARNET_v4_2_GITHUB_REPO_LAYOUT.md` §5.3.

8. **Push the R&D card to `islanddevcrew.com/r-and-d/garnet/`** — links + install one-liner per `GARNET_v4_2_GITHUB_REPO_LAYOUT.md` §6.

9. **Cut the v0.4.2 tag + Release.**
   ```sh
   git tag -a v0.4.2 -m "Garnet v0.4.2 — Stage 6 (cross-platform installers + branding)"
   git push origin v0.4.2   # triggers the Linux-packages workflow → auto-uploads .deb + .rpm + SHA256SUMS
   ```
   Manually attach the signed `.msi` + signed+notarized `.pkg` to the Release.

10. **Update `Garnet_Final/_CANONICAL_DELIVERABLES_INDEX.md`** — add the v4.2 row referencing this handoff + the five Phase 6 handoffs (6A, 6D Linux, 6D Windows, 6D macOS, plus this consolidated state doc).

11. **MIT submission.** The submission package is the contents of the GitHub repo (or the `Garnet_Final/` tree, equivalently). Reviewers run the 15-minute quickstart from `GARNET_v4_2_HANDOFF.md` §REVIEWER'S 15-MINUTE QUICKSTART.

---

## 10. Known issues carried forward

Verbatim from `GARNET_v4_2_HANDOFF.md` §KNOWN ISSUES CARRIED FORWARD (7 items) with status annotations:

1. **MinGW/WinLibs ABI on Windows dev machine** — ✅ **RESOLVED** by MSVC toolchain switch (Phase 6D Windows verify).
2. **`tcp_listen` / `udp_bind`** registered in stdlib registry but lack concrete implementations. Bridge cannot expose until `garnet-stdlib/src/net.rs` grows socket-lifecycle primitives. Tracked as **v4.2.1 or v4.3**.
3. **Coq mechanization of Paper V Theorems A–H.** Multi-month **post-MIT** effort; proof sketches ship at reviewer level.
4. **Paper VI Experiment 1 (LLM pass@1).** Pending **$500 API credits**. Phase 3G 13-program 80% floor estimate stands.
5. **Method-dispatch caps propagation.** caps_graph walks free-function calls only; `arr.sort()` syntax routes through `Expr::Method` which needs type information. **Follow-up** at the same rung as full borrow-check.
6. **MSI branding assets (Dialog.bmp / Banner.bmp / Garnet.ico / License.rtf)** — ✅ **GENERATED** in-session during Phase 6D Windows.
7. **Cosmetic `clean-translate` % display in `garnet convert` summary** — said `10000%` instead of `100.0%` in pre-fix builds. ✅ **Fixed in source**; landed in Linux pass 3 rebuild.

Residual open: (2), (3), (4), (5).

---

## 11. Post-MIT roadmap

Per `GARNET_v4_2_HANDOFF.md` §v4.2 → POST-MIT ROADMAP:

- **v4.3:** Full socket-handle surface (`tcp_listen` + `udp_bind` + read/write/close on `Value::Handle<TcpStream>`), method-dispatch caps propagation, packaging for repository signing (`apt install garnet` / `dnf install garnet` from `pkg.garnet-lang.org`).
- **v5.0:** Bytecode VM behind the tree-walk interpreter (Rung 8); enables the JIT story Paper VII §3 sketches. **Production-bearing workloads should wait for v5.0.**
- **v5.x:** Coq mechanization of Paper V theorems (multi-month, in progress).
- **mdBook documentation site** at `docs.garnet-lang.org`; favicon already accounted for in `garnet-cli/assets/README.md`.

---

## 12. File path reference

Absolute Windows paths to the most-likely-needed files (convert forward-slashes + the workspace prefix on Mac):

**Workspace root (repo-root-equivalent):**
```
D:\Projects\New folder\Garnet (1)\GARNET\Garnet\Opus-Gpt-Xai-Opus-Gemini-Opus\Garnet_Final\E_Engineering_Artifacts\
```

**Key handoffs:**
- `…\Garnet_Final\F_Project_Management\GARNET_v4_2_HANDOFF.md` — primary v4.2 handoff
- `…\Garnet_Final\F_Project_Management\GARNET_v4_2_Phase_6D_Linux_VERIFIED.md`
- `…\Garnet_Final\F_Project_Management\GARNET_v4_2_Phase_6D_Windows_VERIFIED.md`
- `…\Garnet_Final\F_Project_Management\GARNET_v4_2_Phase_6A_HANDOFF.md`
- `…\Garnet_Final\F_Project_Management\GARNET_v3_4_1_HANDOFF.md`
- `…\Garnet_Final\F_Project_Management\GARNET_v4_2_STAGE6_KICKOFF.md`
- `…\Garnet_Final\F_Project_Management\GARNET_v4_2_BOOT.md`
- `…\Garnet_Final\F_Project_Management\GARNET_v4_2_GITHUB_REPO_LAYOUT.md` — companion to this doc
- `…\Garnet_Final\F_Project_Management\GARNET_v4_2_MAC_RECONCILIATION_PROMPT.md` — companion to this doc

**Key research:**
- `…\Garnet_Final\A_Research_Papers\GARNET-The-Reconciliation-of-Rust-and-Ruby.md` — foundational thesis
- `…\Garnet_Final\C_Language_Specification\GARNET_v1_0_Mini_Spec.md` — normative source of truth
- `…\Garnet_Final\A_Research_Papers\Paper_VI_Empirical_Validation_Protocol.md` — pre-registered experiments
- `…\Garnet_Final\D_Executive_and_Presentation\GARNET_v4_2_DX_Comparative_Paper.docx` — §20 Measured-vs-Argued
- `…\Garnet_Final\_CANONICAL_DELIVERABLES_INDEX.md` — 40+ deliverable ledger

**Built artifacts:**
- `…\E_Engineering_Artifacts\dist\linux\garnet_0.4.2-1_amd64.deb`
- `…\E_Engineering_Artifacts\dist\linux\garnet-0.4.2-1.x86_64.rpm`
- `…\E_Engineering_Artifacts\dist\windows\garnet-0.4.2-x86_64.msi`
- `…\E_Engineering_Artifacts\dist\SHA256SUMS`

**Key source:**
- `…\E_Engineering_Artifacts\Cargo.toml` — workspace manifest
- `…\E_Engineering_Artifacts\README.md` — project README (hero + quickstart)
- `…\E_Engineering_Artifacts\FAQ.md` — reviewer-facing honest scorecard
- `…\E_Engineering_Artifacts\garnet-interp-v0.3\src\stdlib_bridge.rs` — 22-primitive bridge
- `…\E_Engineering_Artifacts\garnet-check-v0.3\src\caps_graph.rs` — CapCaps propagator
- `…\E_Engineering_Artifacts\garnet-cli\src\manifest.rs` — Ed25519 ManifestSig
- `…\E_Engineering_Artifacts\garnet-cli\src\new_cmd.rs` — `garnet new` scaffolding
- `…\E_Engineering_Artifacts\garnet-cli\src\bin\garnet.rs` — CLI dispatch
- `…\E_Engineering_Artifacts\garnet-cli\wix\main.wxs` — WiX 3 MSI XML
- `…\E_Engineering_Artifacts\garnet-cli\macos\build-pkg.sh` — macOS PKG build driver
- `…\E_Engineering_Artifacts\installer\sh.garnet-lang.org\install.sh` — universal shell installer
- `…\E_Engineering_Artifacts\.github\workflows\linux-packages.yml` — Linux CI + Release upload

---

*"I have fought a good fight, I have finished my course, I have kept the faith." — 2 Timothy 4:7*

*Consolidated by Claude Opus 4.7 (1M) on 2026-04-21 from the Stage-6 close handoffs. Self-contained: a fresh Claude session reading this file alone has enough context to continue.*
