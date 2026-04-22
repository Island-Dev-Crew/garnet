# Garnet: Project Origin Summary (MIT Submission Format)

**Title:** Garnet: A Dual-Mode, Agent-Native Programming Language Reconciling Rust's Safety with Ruby's Expressiveness

**Author:** Jon Isaac, Island Development Crew LLC, Huntsville, AL

**Project Period:** April 11–17, 2026 (7 calendar days from initial concept to verified working implementation)

**Contact:** Jony02@hotmail.com | garnet-lang.org

---

## 1. Origin

The Garnet project originated from a single question posed during personal research on April 11, 2026: given the industry's simultaneous investment in Rust (for systems-level safety guarantees) and Ruby (for developer expressiveness), is there a principled way to unify both paradigms within a single language — not as a meta-language or preprocessor, but as first-class, compiler-enforced modes within one coherent grammar?

Before any implementation began, the author conducted a structured multi-model research sequence involving five frontier AI systems (Claude, GPT, Grok, Gemini, and back to Claude for synthesis) across seven analytical passes to establish: (a) the novelty of the dual-mode approach, (b) the feasibility of the proposed memory primitive system, (c) the gap in existing language design literature, and (d) the minimum viable specification required for a working prototype. All models were instructed to evaluate the proposal at MIT-level rigor.

## 2. Architecture

Garnet introduces a dual-mode programming model:

- **Managed mode (`def`)**: Reference-counted memory, dynamic-ish type feel, Ruby-like surface syntax, optimized for developer velocity in orchestration code.
- **Safe mode (`@safe fn`)**: Ownership-based memory with a move-tracking borrow checker, static type enforcement, zero-cost abstractions, optimized for performance-critical hot paths.
- **Mode boundary**: A first-class compiler construct that auto-bridges between modes, audits crossings, and preserves velocity where safety is unnecessary while enforcing correctness where it matters.
- **First-class memory primitives**: Four declarative memory types (working, episodic, semantic, procedural) as language-level constructs, not library imports.
- **Typed actors**: Compiler-enforced message protocols for concurrent agent systems.
- **Capability annotations (`@caps`)**: Compile-time authority verification — no ambient authority.

## 3. Implementation State (as of April 17, 2026)

| Component | Status |
|-----------|--------|
| Lexer + Parser | 213 tests, 90 EBNF productions |
| Tree-Walk Interpreter | 372 tests, built-in REPL |
| Safe-Mode Checker | 35 tests, move-tracking borrow checker |
| Memory Primitives | 41 tests across 4 kinds |
| Actor Runtime | 33+5 hot-reload tests, concurrent execution with state migration |
| Standard Library | 22 bridged primitives, 74 tests |
| Multi-Language Converter | 85 tests (Rust/Ruby/Python/Go → Garnet) |
| CLI Binary | 12 smoke tests (parse/check/run/eval/repl/build/verify/convert/keygen/new) |
| Cross-Platform Installers | Linux verified (.deb + .rpm in Docker), Windows verified (MSVC), macOS pending |
| Cryptographic Signing | Ed25519 manifest verification, 12 tests |
| Determinism Harness | 7× consistency verification (xtask crate) |
| **Cumulative tests** | **1,244 source + 136 security** |
| **Research corpus** | **7 papers + 4 addenda** |
| **Specification** | **1,670-line Mini-Spec v1.0** |

## 4. Novel Contributions (Pre-Registered Findings)

Paper VI (Empirical Validation) reports seven pre-registered contributions across Phase 1C (registered) and Phase 4A (executed):

- **4 supported**: Dual-mode design, mode boundary as first-class construct, compiler-as-agent cache, actor hot-reload with state migration
- **2 partial**: Capability propagation across module boundaries, memory primitive R+R+I decay curves
- **0 refuted**
- **1 pending-infra**: Requires full compiler backend (LLVM) for performance benchmarking

## 5. Design Philosophy

Garnet's name derives from the gemstone — formed under pressure, compressed, hardened, and beautiful because of the force, not despite it. The design philosophy is reconciliation, not replacement:

- Rust's rigor without abandoning Ruby's readability
- Agent-native memory without requiring external infrastructure
- Compile-time safety without sacrificing developer velocity at the orchestration layer
- A language designed for the era of AI-assisted development, where agents write most code and humans read and verify it

## 6. Licensing

Garnet is dual-licensed under Apache License 2.0 and MIT License, matching Rust's own licensing model. Contributions are accepted under DCO 1.1.

## 7. Availability

- **Source code**: Pending public GitHub release (v0.1.0)
- **Website**: garnet-lang.org (registered, awaiting deployment)
- **Universal installer**: sh.garnet-lang.org (shellchecked, pending deployment)
- **Domain**: garnet-lang.org (registered by author)

## 8. Acknowledgments

This project was built by a single developer with no institutional affiliation or funding, using frontier AI systems as architectural collaborators during a constrained development window (April 11–17, 2026). The research validation sequence involved five AI systems providing independent analysis before implementation began.

---

*Submitted by Jon Isaac, Founder & CEO, Island Development Crew LLC*
*April 19, 2026*
