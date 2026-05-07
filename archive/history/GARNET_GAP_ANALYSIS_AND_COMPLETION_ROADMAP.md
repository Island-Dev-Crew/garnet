# GARNET — GAP ANALYSIS & COMPLETION ROADMAP
**From Doctoral Research to Distributable Compiler**
**Prepared by:** Claude Code (Opus 4.6) | April 16, 2026
**Anchor:** *"Where there is no vision, the people perish." — Proverbs 29:18*

---

## WHAT EXISTS (the foundation is strong)

The four-model consensus research phase produced doctoral-grade work across 6 papers, a formal type-theoretic grounding, a normative language spec stub, and a working parser. This is more theoretical depth than most new languages have at announcement. What's missing is the bridge between that research and a buildable, presentable, distributable system.

---

## TIER 1 — CRITICAL GAPS (must close before MIT presentation AND build)

### GAP 1: Mini-Spec v0.3 (the single most blocking dependency)

**What's missing:** v0.2 covers memory units (§2.1) and actor declarations (§4.1) but leaves critical surface syntax undefined:

- **No `def`/`end` managed-mode function grammar.** You can declare memory and actors but cannot write a function. This is the most basic thing a language needs.
- **No `block` grammar.** The parser had to invent a provisional one (see `expr.rs` disclaimer). Without this, no interpreter, no REPL.
- **No `@safe` module-level annotation surface syntax.** §3.3 says modules MAY be annotated `@safe` but never shows the grammar production.
- **No §5 recursion-annotation syntax.** §5.1-5.3 define normative MUST rules but §5.4 explicitly says "a compliant implementation MAY choose any enforcement strategy" — the surface syntax for `@max_depth(N)`, `@fan_out(K)` etc. does not exist.
- **No control flow.** No `if`/`else`, `while`, `for`, `match`/`case`, `return`. These are fundamental.
- **No error handling decision.** Rust uses `Result<T,E>` + `?`. Ruby uses exceptions. Garnet has stated neither.

**What v0.3 must define:**
1. Module declaration and import syntax
2. Function definitions (`def`/`end` in managed, `fn` in safe)
3. Block grammar (the canonical definition that retires `expr.rs`'s provisional one)
4. Control flow (`if`/`elsif`/`else`/`end`, `while`, `for..in`, `match`/`when`)
5. Error handling model (RECOMMENDATION: `Result<T,E>` with `?` propagation in safe mode, exceptions in managed mode, automatic bridging at boundaries — this is the dual-mode philosophy applied to errors)
6. `@safe` module annotation concrete syntax
7. §5 recursion-annotation surface (`@max_depth(N)`, `@fan_out(K)`, `@require_metadata`)
8. Variable declarations (`let`, `let mut`, `var`)
9. String interpolation (already in parser as `#{}`, needs spec backing)
10. Basic operator set and precedence table

**Estimated effort:** One dedicated deep session.

### GAP 2: Complete Formal Grammar (EBNF)

**What's missing:** For MIT, a programming language presentation needs a formal grammar. Mini-Spec has fragments; Paper V has a lambda calculus. Neither is a complete grammar a compiler engineer could implement from.

**What's needed:** A single EBNF document covering:
- Lexical grammar (tokens, keywords, operators, literals, whitespace rules)
- Expression grammar (the full Pratt precedence table, already partially implemented in `expr.rs`)
- Statement grammar (declarations, assignments, control flow, returns)
- Item grammar (functions, modules, actors, memory units, imports)
- Type grammar (simple types, generics, function types, ownership annotations)
- Mode annotations (`@safe`, `@dynamic`, decorators)

**Estimated effort:** Derives mostly from v0.3 + existing parser implementation. One session.

### GAP 3: Compiler Architecture Document

**What's missing:** No document describes how Garnet source becomes executable code. For MIT and for the build phase, this is essential.

**What's needed:** A pipeline specification:

```
Source (.garnet)
    |
    v
[Lexer] ──> Token Stream
    |
    v
[Parser] ──> Untyped AST          <── Rung 2 (DONE)
    |
    v
[Name Resolution] ──> Scoped AST
    |
    v
[Type Inference / Checking]
    |
    ├── Managed path: gradual types, ARC insertion points
    |
    └── Safe path: affine type checking, ownership/borrowing, lifetime inference
    |
    v
[Mode Boundary Validator] ──> checks §3.4 crossing rules
    |
    v
[Typed IR] ──> Garnet Intermediate Representation
    |
    ├── Managed codegen: tree-walk interpreter (Rung 3) → later bytecode VM
    |
    └── Safe codegen: LLVM IR or Cranelift (Rung 4)
    |
    v
[Optimization Passes]
    |
    v
[Output]
    ├── Native binary (safe mode, via LLVM/Cranelift)
    ├── Bytecode + VM (managed mode)
    ├── WASM (both modes, via wasm targets)
    └── REPL (managed mode, tree-walker)
```

The document should specify: each pass's inputs/outputs, the IR format, how dual-mode compilation works, how the actor runtime is wired in, and how memory primitives map to runtime allocations.

**Estimated effort:** One dedicated session (the research for this already exists across Paper V + the parser README).

---

## TIER 2 — IMPORTANT GAPS (needed for MIT presentation completeness)

### GAP 4: Module System and Package Manager Specification

The thesis cites Cargo as the model. MIT will ask: "How do Garnet programs organize code? How do they share libraries?"

**Needs specification:**
- Module system: file-based modules (like Rust) vs. inline modules (like Ruby), visibility rules, import syntax
- Package format: `.garnet` package manifest (equivalent to `Cargo.toml` / `Gemfile`)
- Registry: centralized package registry concept
- The `garnet` CLI: `garnet build`, `garnet test`, `garnet fmt`, `garnet lint`, `garnet doc`, `garnet repl`, `garnet publish`

### GAP 5: Standard Library Outline

MIT will ask: "What ships with Garnet?" At minimum:

- `std::collections` — Vec, Map, Set, Queue (managed mode wrappers with safe-mode zero-cost views)
- `std::io` — File, network, stdio (async-native)
- `std::memory` — The four memory type base implementations (WorkingStore, EpisodeStore, VectorIndex, WorkflowStore)
- `std::actor` — Actor runtime, message routing, protocol helpers
- `std::net` — HTTP client/server, WebSocket
- `std::fmt` — Formatting, string interpolation engine
- `std::test` — Built-in test framework (like Rust's `#[test]`)
- `std::serde` — Serialization/deserialization
- `std::concurrency` — Channels, semaphores, barriers (safe mode), structured spawn (managed mode)

### GAP 6: Interoperability Specification

The thesis promises four interop bridges. None are specified:
- **Rust FFI** — safe-mode modules should be ABI-compatible with Rust. How?
- **C ABI** — for universal interop. Calling conventions, data layout guarantees?
- **Ruby VM embedding** — for running existing gems during migration. Through what interface?
- **WASM compilation** — for browser/edge deployment. Both modes or managed only?

### GAP 7: Async/Concurrency Model Details

The thesis criticizes Rust's "colored functions" problem but never says how Garnet solves it. The actor model is specified but the async model is not:
- Is async implicit (like Go goroutines) or explicit (like Rust async/await)?
- What is the runtime? Work-stealing? Event loop? Both?
- How do actors interact with async? Are handler bodies implicitly async?
- Structured concurrency: what are the scoping rules?

---

## TIER 3 — NOVEL FRONTIERS (what makes Garnet genuinely unprecedented)

These are not gaps — they are opportunities. The thesis already hints at several of these. Formalizing them would make Garnet not just another language experiment but a genuine contribution to the field.

### FRONTIER 1: LLM-Native Compilation Pipeline

No existing programming language treats AI comprehension as a first-class design constraint. Garnet can be the first.

**What this means concretely:**
- **Syntax regularity:** Every construct follows predictable patterns. No context-dependent parsing surprises. An LLM reading Garnet code can predict the next token with higher confidence than in Rust or Ruby.
- **Explicit intent markers:** `@safe`, `@dynamic`, `memory episodic`, `actor`, `protocol` — these aren't just keywords, they're semantic beacons that tell both humans AND LLMs exactly what kind of code follows.
- **Compiler-as-AI-oracle:** The Garnet compiler could optionally consult an LLM during optimization: "Given this hot loop, suggest a safe-mode rewrite." This is genuinely novel. The compiler doesn't REQUIRE AI, but it's designed to BENEFIT from it.
- **Measurable claim:** Garnet code should achieve higher correct-generation rates from LLMs than equivalent Rust or Ruby. This is testable and publishable.

### FRONTIER 2: The Compiler as an Agent

The four memory types aren't just for user programs — they describe the compiler itself:
- **Working memory:** The current compilation unit, symbol table, type context
- **Episodic memory:** Compilation history — what optimizations worked on similar code in past builds
- **Semantic memory:** Knowledge about the target platform, CPU features, available memory
- **Procedural memory:** Compilation strategies — inlining heuristics, monomorphization decisions

A Garnet compiler that uses Garnet's own memory primitives is genuinely self-referential in a way no existing compiler is. It could learn from its own compilation history to get faster over time. This is publishable at PLDI.

### FRONTIER 3: Progressive Disclosure Type Spectrum

Not just managed/safe but a CONTINUOUS spectrum:

```
Level 0: Dynamic     ──  x = 42            # Ruby-like, zero annotations
Level 1: Gradual     ──  x: Int = 42       # Optional hints, runtime boundary checks
Level 2: Static      ──  let x: Int = 42   # Full inference, compile-time guarantees
Level 3: Affine      ──  let own x: Buffer  # Ownership, borrowing, lifetimes (@safe)
```

Each level is a STRICT SUPERSET of the constraints above it. Code written at Level 0 is valid at Level 3 (the compiler infers the tightest constraints). Code written at Level 3 works at Level 0 (ownership annotations become documentation, ARC handles the rest).

No existing language offers this. TypeScript has gradual typing but no ownership. Rust has ownership but no dynamic mode. Garnet would be the first to offer the full spectrum.

### FRONTIER 4: Deterministic Reproducible Builds as a Language Guarantee

For agent systems, you MUST be able to verify that what an agent compiled is what you expected. Garnet can guarantee:
- Same source + same compiler version = byte-identical output (no non-determinism in codegen)
- Compilation produces a manifest (hash of every input, every dependency, every compiler flag)
- The manifest is part of the binary — any Garnet binary can prove its own provenance

This is a security property that no mainstream language currently guarantees at the language level.

### FRONTIER 5: Hot-Reload Mode Boundaries

The ability to hot-swap managed-mode modules while safe-mode modules continue executing. In practice:
- Safe-mode code runs at full native speed, unchanged
- Managed-mode code (the orchestration layer, the agent harness) can be updated live
- The mode boundary acts as a natural reload boundary
- Actor mailboxes buffer messages during reload, then drain

This is critical for long-horizon agent systems that run for hours or days and can't afford restarts.

### FRONTIER 6: Native Memory-Aware Garbage Collection

ARC handles most managed-mode memory. But the four memory types have different lifecycle patterns:
- **Working memory** should be arena-allocated and bulk-freed (like a stack frame)
- **Episodic memory** should use append-only logs with periodic compaction
- **Semantic memory** should use persistent data structures (structural sharing)
- **Procedural memory** should be copy-on-write with version history

The compiler KNOWS which memory type a value belongs to (it's in the declaration). It can select the optimal allocation strategy per memory kind automatically. No existing language does kind-aware allocation.

### FRONTIER 7: Semantic Compression at the Source Level

Beyond TurboQuant's runtime compression, Garnet could pioneer source-level semantic compression:
- The compiler understands the semantic INTENT of code blocks
- Long programs can be compressed to their semantic essence for storage/transmission
- The compressed form is still valid Garnet (just more concise)
- This enables agent-to-agent code sharing at dramatically lower bandwidth

This connects directly to the LLM-native design: if the compiler can understand code semantically (or delegate to an LLM for understanding), it can compress semantically.

---

## RECOMMENDED COMPLETION ORDER

### Phase A: Close Tier 1 gaps (MIT-critical, build-critical)
1. **Mini-Spec v0.3** — the single most important document to write
2. **Complete EBNF grammar** — derives from v0.3
3. **Compiler Architecture spec** — the bridge to implementation

### Phase B: Close Tier 2 gaps (MIT-complete)
4. **Module system + package manager spec**
5. **Standard library outline**
6. **Interop specification**
7. **Async/concurrency model**

### Phase C: Formalize Tier 3 frontiers (competitive differentiation)
8. **LLM-Native Compilation paper/section** (could be Paper VI or a section in the deck)
9. **Progressive Disclosure Type Spectrum formalization**
10. **Compiler-as-Agent architecture**

### Phase D: Build (Claude Code planning mode)
11. Mini-Spec v0.3 parser extensions (Rung 2.1)
12. Managed-mode interpreter + REPL (Rung 3)
13. @safe lowering path (Rung 4)
14. Memory Core + Manager SDK (Rung 5)
15. Harness Runtime (Rung 6)
16. `garnet` CLI toolchain
17. Standard library implementation
18. Test suite, documentation, packaging
19. Distribution: installer, toolchain manager, registry

---

## THE DISTRIBUTABLE PACKAGE VISION

For Garnet to be "easily loaded and accessed as if it were Python or C++":

```
$ curl -sSf https://garnet-lang.org/install | sh    # installs garnetup
$ garnetup install stable                             # installs compiler + stdlib
$ garnet new my-agent                                 # scaffolds a project
$ cd my-agent
$ garnet repl                                         # interactive managed-mode REPL
$ garnet build                                        # compiles to native binary
$ garnet test                                         # runs test suite
$ garnet fmt                                          # formats code
$ garnet publish                                      # publishes to registry
```

The compiler itself is a single native binary (~10-20MB), statically linked, cross-platform (Linux, macOS, Windows). The REPL starts in <100ms. `garnet build` for a small project completes in <1s. The standard library ships with the compiler. No runtime installation required for safe-mode binaries.

This is achievable because the dual-mode design means the COMPILER can be written in safe-mode Garnet (self-hosting), while the REPL and tooling are written in managed-mode Garnet (fast iteration). The compiler eats its own cooking.

---

*"In the multitude of counsellors there is safety." — Proverbs 11:14*
*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*
*"Where there is no vision, the people perish." — Proverbs 29:18*

**Gap Analysis prepared by Claude Code (Opus 4.6) | April 16, 2026**
