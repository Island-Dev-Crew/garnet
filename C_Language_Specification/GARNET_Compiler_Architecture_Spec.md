# Garnet Compiler Architecture Specification
**Version:** 1.0 (companion to Mini-Spec v0.3)
**Date:** April 16, 2026
**Anchor:** *"Where there is no vision, the people perish." — Proverbs 29:18*

---

## 1. Overview

The Garnet compiler (`garnetc`) transforms `.garnet` source files into executable artifacts through a multi-phase pipeline. The pipeline branches at the type-checking phase to handle managed mode and safe mode through distinct paths, reuniting at the output stage. This architecture directly implements the dual-mode design formalized in Paper V and specified in Mini-Spec v0.3.

```
                          ┌─────────────────────────────────────────┐
                          │           GARNET COMPILER               │
                          │                                         │
  .garnet ──► [Lexer] ──► [Parser] ──► [Name Resolver] ──►        │
              Phase 1      Phase 2      Phase 3                     │
                                           │                        │
                              ┌────────────┴──────────────┐         │
                              │                           │         │
                         [Managed TC]              [Safe TC]        │
                          Phase 4a                  Phase 4b        │
                              │                           │         │
                     [Boundary Validator]                  │         │
                          Phase 5                         │         │
                              │                           │         │
                      ┌───────┴────────┐          ┌──────┴──────┐  │
                      │                │          │             │  │
                 [Tree-Walk]    [Bytecode]    [LLVM IR]   [Cranelift]
                  Phase 6a      Phase 6b      Phase 6c     Phase 6d │
                      │                │          │             │  │
                   [REPL]        [Garnet VM]   [Native]    [WASM]  │
                                                                    │
                          └─────────────────────────────────────────┘
```

---

## 2. Phase 1: Lexer

**Input:** UTF-8 source bytes
**Output:** Token stream with spans

The lexer is a hand-rolled single-pass scanner (not a generated lexer). This choice was validated in the Rung 2 `garnet-parser` crate, where the hand-rolled approach achieved cleaner string-interpolation handling than any generator-based alternative.

### Key responsibilities
- Tokenize all v0.3 keywords (40+), operators, literals, and identifiers
- Track source spans (byte offset + length) for every token for diagnostic messages
- Handle string interpolation by re-entering expression lexing inside `#{}` segments
- Normalize line endings (CRLF → LF)
- Strip comments (single-line `#` only in v0.3)
- Produce a flat `Vec<Token>` — no lazy iteration, because the parser needs random access for lookahead

### Token categories
- Keywords (mode, declaration, control flow, error handling, ownership, guardrails)
- Identifiers
- Literals (integer, float, string, raw string, symbol, boolean, nil)
- Operators and punctuation
- Annotations (`@safe`, `@dynamic`, `@max_depth(N)`, `@fan_out(K)`, `@require_metadata`)
- String interpolation segments (`StrStart`, `StrPart`, `StrEnd`, `InterpStart`, `InterpEnd`)

**Performance target:** Lex 100K lines of Garnet in <50ms on modern hardware.

---

## 3. Phase 2: Parser

**Input:** Token stream
**Output:** Untyped Abstract Syntax Tree (`Module`)

Recursive-descent parser consuming the v0.3 EBNF grammar (90 productions). The existing `garnet-parser` crate implements the §4 (memory) and §9 (actor) productions from v0.2; v0.3 extends it with:

- Module declarations and imports (§3)
- `def` managed-mode functions (§5.1)
- `fn` safe-mode functions (§5.2)
- Closures (§5.3)
- Full control flow: `if`/`elsif`/`else`, `while`, `for`/`in`, `loop`, `break`/`continue`, `return` (§6)
- Pattern matching: `match`/`when` with exhaustiveness metadata (§6.3)
- Error handling: `try`/`rescue`/`ensure`, `raise`, `?` operator (§7)
- Variable declarations: `let`, `let mut`, `var`, `const` (§6.1)
- Type syntax: generics, function types, tuple types, ref types (§11)
- User-defined types: `struct`, `enum`, `trait`, `impl` (§11.3)
- Guardrail annotations: `@max_depth`, `@fan_out`, `@require_metadata` (§10)

### Expression parsing
The Pratt precedence climber from the v0.2 parser's `expr.rs` is promoted to canonical status and extended with:
- Pipeline operator `|>` (precedence 7)
- Range operators `..` / `...` (precedence 6)
- `?` as a postfix error-propagation operator (highest postfix precedence)
- `spawn` as a prefix expression operator

### AST node design
Every AST node carries:
- `Span` (source location for diagnostics)
- `NodeId` (unique identifier for later passes to attach metadata)
- No type information (types are added in Phase 4)

**Performance target:** Parse 100K lines in <100ms.

---

## 4. Phase 3: Name Resolution

**Input:** Untyped AST
**Output:** Scoped AST with resolved names

### Responsibilities
- Build scope tree (modules → functions → blocks)
- Resolve all `use` imports to their target definitions
- Detect undefined names, shadowing warnings, unused imports
- Resolve `path::segments` to their fully-qualified targets
- Detect module-level `@safe` annotations and tag each scope with its mode (managed or safe)
- Build the symbol table mapping every `NodeId` to its `DefId` (definition identity)

### Mode tagging
Every scope is tagged as either `Managed` or `Safe`. This tag propagates inward: a `@safe` module makes all contained scopes safe. A managed module can contain inline `@safe` modules, creating safe islands. The name resolver identifies every mode-crossing call site and flags it for Phase 5 (boundary validation).

---

## 5. Phase 4: Type Checking (dual-path)

This is where the dual-mode architecture manifests most visibly. Two type checkers operate on different subsets of the scoped AST.

### 4a: Managed-Mode Type Checker

**Input:** Scoped AST nodes in managed scopes
**Output:** Typed AST with inferred types + ARC insertion points

Implements gradual typing (Mini-Spec §11.1, Levels 0–2):
- Type inference using bidirectional type checking (Dunfield & Krishnaswami, ICFP 2013)
- Dynamic types (`Dyn`) for unannotated positions
- Runtime boundary checks inserted where `Dyn` meets a concrete type
- ARC retain/release points computed from scope analysis
- Cycle detection hooks for ARC (weak reference insertion at detected cycles)

**Key design choice:** Managed-mode type inference is LOCAL (within function bodies), not global. This prevents Crystal's global-inference compile-time explosion problem (cited in the original thesis as a reason Crystal failed to scale). Function signatures with omitted types get `Dyn` boundaries.

### 4b: Safe-Mode Type Checker

**Input:** Scoped AST nodes in safe scopes
**Output:** Typed AST with ownership annotations + lifetime annotations

Implements affine typing (Mini-Spec §11.1, Level 3):
- Ownership checking: every value has exactly one owner
- Borrow checking: aliasing-XOR-mutation invariant
- Lifetime inference: non-lexical lifetimes (NLL) per Rust RFC 2094
- Move semantics: consumed values are invalidated
- `Send`/`Sync` trait derivation for actor message types

The formal basis is Paper V's λ_safe calculus (§3–§5), grounded in RustBelt/Iris methodology.

**Key difference from Rust:** Garnet's safe-mode checker operates at MODULE granularity, not crate granularity. This means lifetime inference is bounded to one file, preventing the cross-crate lifetime propagation that causes Rust's worst diagnostic cascades.

---

## 6. Phase 5: Boundary Validator

**Input:** Fully typed AST (both managed and safe portions)
**Output:** Validated AST with bridge nodes inserted

### Responsibilities
- Verify every mode-crossing call satisfies §8.4 boundary rules
- Insert error-bridging adapters per §7.4:
  - Managed → Safe call: wrap the call in implicit `try`, convert exceptions to `Result::Err(ManagedError(...))`
  - Safe → Managed call (return): unwrap `Ok(v)` to `v`, convert `Err(e)` to `raise SafeModeError(e)`
- Insert ARC adoption for owned values crossing from safe to managed
- Insert ownership verification for values crossing from managed to safe
- Emit clear diagnostics for boundary violations with suggestions for fixes

This phase has no equivalent in any existing compiler. It is a direct implementation of Paper V §4's bridging judgment and represents Garnet's most novel compiler contribution.

---

## 7. Phase 6: Code Generation (four targets)

### 6a: Tree-Walk Interpreter (Rung 3)

**Target:** Managed-mode code for the REPL and rapid prototyping
**Strategy:** Walk the typed AST directly, evaluating nodes recursively

This is the first implementation target (Rung 3 of the engineering ladder). It supports:
- All managed-mode constructs
- ARC memory management (using host language's RC<RefCell<T>> if written in Rust)
- Actor message passing via an in-process mailbox system
- Memory unit declarations backed by in-memory stores

**Performance target:** Comparable to Ruby 3.4 interpreter (not YJIT), approximately 10-50x slower than native.

### 6b: Bytecode Compiler + VM (post-Rung 3)

**Target:** Managed-mode code for production
**Strategy:** Compile typed AST to a register-based bytecode, execute on a custom VM

The bytecode VM provides:
- Faster execution than tree-walking (~3-10x)
- JIT compilation hooks (future: a Garnet equivalent of YJIT, potentially written in safe-mode Garnet itself)
- Serializable bytecode for distribution
- Debug information for step-through debugging

### 6c: LLVM IR (Rung 4)

**Target:** Safe-mode code for maximum performance
**Strategy:** Lower typed AST to LLVM IR via the `inkwell` or `llvm-sys` Rust crates

Safe-mode Garnet compiles through LLVM like Rust does:
- Monomorphized generics (zero-cost abstraction)
- Deterministic deallocation (no GC, no ARC — ownership semantics handle everything)
- Full LLVM optimization pipeline (O0–O3, LTO)
- Native binary output for the host platform

**Performance target:** Within 2x of equivalent Rust for compute-bound code. Within 1.2x after LLVM optimization passes.

### 6d: Cranelift (alternative safe-mode backend)

**Target:** Safe-mode code with faster compilation
**Strategy:** Lower to Cranelift IR for debug builds

Cranelift compiles 5-10x faster than LLVM at the cost of ~30% slower runtime. Use for:
- Debug builds (`garnet build --debug`)
- WASM output (`garnet build --target wasm32`)
- Rapid iteration during development

### Output matrix

| Mode | Debug Build | Release Build | REPL | WASM |
|---|---|---|---|---|
| Managed | Tree-walk / Bytecode VM | Bytecode VM | Tree-walk | Bytecode + wasm-micro-runtime |
| Safe | Cranelift | LLVM | N/A (compile first) | Cranelift → wasm32 |
| Mixed | Both paths | Both paths | Managed only | Both → wasm |

---

## 8. The `garnet` CLI Toolchain

```
garnet build [--release] [--target TARGET]    # compile project
garnet run [file.garnet]                       # compile and execute
garnet repl                                    # interactive managed-mode REPL
garnet test                                    # run test suite
garnet fmt                                     # format source files
garnet lint                                    # run linter (Clippy-equivalent)
garnet doc                                     # generate documentation
garnet new <name>                              # scaffold new project
garnet add <package>                           # add dependency
garnet publish                                 # publish to registry
garnet check                                   # type-check without codegen
```

The CLI is a single native binary. It embeds the lexer, parser, both type checkers, the boundary validator, the tree-walk interpreter (for REPL), and either LLVM or Cranelift for native codegen.

---

## 9. Project Layout

```
my-project/
├── Garnet.toml              # project manifest (Cargo.toml equivalent)
├── src/
│   ├── main.garnet          # entry point
│   ├── lib.garnet           # library root (if library project)
│   └── ...
├── tests/
│   └── ...
├── examples/
│   └── ...
├── target/
│   ├── debug/               # debug build artifacts
│   └── release/             # release build artifacts
└── garnet.lock              # dependency lockfile
```

### Garnet.toml

```toml
[package]
name = "my-agent"
version = "0.1.0"
edition = "2026"

[dependencies]
garnet-http = "0.1"
garnet-memory = "0.1"

[dev-dependencies]
garnet-test = "0.1"
```

---

## 10. Memory Management Architecture

### Managed mode: ARC with cycle detection

- Every heap allocation is reference-counted
- The compiler inserts `retain` (increment) and `release` (decrement) calls based on scope analysis
- A background cycle detector runs periodically (configurable frequency)
- Weak references (`Weak<T>`) are available for breaking cycles explicitly

### Safe mode: ownership + deterministic deallocation

- No reference counting overhead
- Values are dropped when their owner goes out of scope
- Borrowing extends access without ownership transfer
- The compiler inserts `drop` calls at scope boundaries

### Kind-aware allocation (novel, per gap analysis Frontier 6)

The compiler knows each value's memory kind from its declaration:

| Memory Kind | Allocation Strategy | Rationale |
|---|---|---|
| Working | Arena-allocated, bulk-freed | Short-lived, high churn — arena avoids per-object overhead |
| Episodic | Append-only log, periodic compaction | Chronological, rarely mutated — log structure is natural |
| Semantic | Persistent data structure (structural sharing) | Factual, frequently read — sharing avoids copy overhead |
| Procedural | Copy-on-write with version history | Behavioral, versioned — COW enables rollback |

This optimization is transparent to the programmer — they declare `memory episodic log : EpisodeStore<Event>` and the compiler selects the append-only allocator automatically.

---

## 11. Actor Runtime

Actors are the concurrency primitive. The runtime provides:

- **Mailbox system:** Each actor has a typed mailbox (bounded channel). Messages are queued and processed sequentially by the actor's single handler thread.
- **Spawn semantics:** `spawn ActorType.protocol(args)` creates a new actor instance (or sends to an existing one, depending on the actor's lifecycle annotation — singleton vs. per-request).
- **Supervision:** Parent actors can monitor children. If a child panics, the parent receives a `Down` message with the error.
- **Scheduling:** Work-stealing scheduler distributes actors across OS threads. The number of worker threads defaults to the number of CPU cores.

### Actor isolation guarantee

Actors MUST NOT share mutable state (Mini-Spec §9.2). The compiler enforces this by:
1. All message types must be `Send` (safe mode) or deep-cloned (managed mode)
2. Captured mutable state in handler closures is rejected
3. Memory units declared inside an actor are private to that actor instance

### 11.2 Green-Thread Scheduler (addresses Mini-Spec OQ-9)

Garnet's async model (specified in `GARNET_Tier2_Ecosystem_Specifications.md §D`) uses green threads to eliminate the colored-function problem. The runtime mechanism:

**M:N scheduling.** M green threads run on N OS threads (default N = number of CPU cores). Each OS thread is a worker with its own run queue. A work-stealing scheduler rebalances load when a worker's queue empties.

**Suspension points.** A green thread suspends (yields its OS thread) at any call that performs I/O, acquires a lock, awaits a channel, sleeps, or explicitly `await_all` / `await_any`. The compiler inserts save/restore stubs at these call sites; callers see no async coloring in their signatures.

**Actor integration.** Each actor handler runs on a green thread. Within the handler body, any green-thread-safe call (I/O, `spawn`, channel ops) suspends the handler without blocking the underlying OS thread. The mailbox mechanism (described above) queues incoming messages while a handler is suspended — the actor still processes one message at a time from its perspective.

**Structured concurrency.** `scope { ... }` blocks create a task scope; all green threads spawned within MUST complete before the scope exits. If any fails, the scope cancels remaining tasks and propagates the failure. This is the language-level guarantee that `GARNET_Tier2_Ecosystem_Specifications.md §D.5` specifies; this section describes how the runtime enforces it (cancellation via cooperative yield-point polling).

**Blocking calls.** When a green thread calls a truly blocking operation (e.g., unfriendly FFI), the runtime transparently moves it to a dedicated blocking pool and returns the OS thread to normal duty. This is the same strategy Tokio uses for `spawn_blocking`; Garnet does it implicitly so programmers don't need to think about thread categories.

**Performance target.** Task spawn overhead < 500ns, context switch < 200ns (comparable to Go goroutines). These targets are empirical — a Benchmarking Plan entry (`GARNET_Benchmarking_and_Evaluation_Plan.md`) tracks them.

### 11.3 Hot-Reload Synchronization (addresses Paper VI Contribution 6)

Paper VI Contribution 6 claims that Garnet supports hot-reloading managed-mode code while safe-mode code continues running. This section specifies the synchronization protocol that makes it safe.

**The core insight.** The mode boundary IS the reload boundary:

- Safe-mode code (native binaries, linked once) NEVER reloads — it is invariant during any hot-reload window.
- Managed-mode code (bytecode VM) swaps bytecode at defined points.
- Mode-crossing calls are where old-vs-new bytecode distinctions can matter.

**The synchronization protocol.**

```
Time ───────────────────────────────────────────────────────────────────►

Safe mode      ████████████████████████████████████████████████████████
                         │                           │
                         │ (unchanged)               │
                         │                           │
Actor mailbox   ▒▒▒▒▒▒▒▒▒██████████████ buffer ████████▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
                         ▲              ▲            ▲
                         │              │            │
Managed mode    AAAAAAAAA│BBBBBBBBBBBB  │CCCCCCCCCCCC│DDDDDDDDDDDDDDDDDD
                         │              │            │
                         │              │            │
                         T₀             T₁           T₂
                         │              │            │
                   Reload begins   In-flight        Drain begins
                                   handler(s)
                                   complete
                                   (barrier)

A = v1 bytecode
B = in-flight handlers finishing on v1
C = (no new messages processed — mailbox buffers them)
D = v2 bytecode with buffered messages drained
```

**Invariants.**

1. **In-flight isolation.** Any handler that was executing at T₀ continues with v1 bytecode until it returns. A handler never observes a mix of v1 and v2 code within its own execution.
2. **Mailbox buffering.** Between T₀ and T₂, new messages are buffered in the actor mailbox. No messages are lost; none are duplicated.
3. **Safe-mode continuity.** Safe-mode code keeps running throughout T₀–T₂. It may call managed-mode code; the call returns correctly because safe code only sees the "current" managed entry point (which is v1 through T₁, v2 after).
4. **Memory unit durability.** Actor-scoped memory units survive reload if and only if their declaration is preserved in v2. If v2 removes a memory unit, the old data is discarded; if v2 adds one, it starts empty. Mid-spec memory migration requires user-written migration handlers (not part of the hot-reload core protocol).

**Failure modes.**

- **v2 bytecode fails to load.** The runtime reverts to v1, the buffered messages drain against v1, and the user gets a diagnostic. No crash.
- **In-flight handler deadlocks.** The runtime has a configurable timeout (default 30 seconds); if exceeded, the handler is forcibly terminated, its actor panics, and standard supervision (Mini-Spec §9) takes over.
- **Safe-mode call to removed managed entry point.** Compiler catches this at v2 link time; hot-reload never completes with dangling references.

**Falsifiable measurement.** Message-delivery latency during the reload window SHOULD stay under 100ms end-to-end for typical handler durations (< 10ms). This is the hypothesis in Paper VI §7.3.

---

## 12. Compilation Performance Targets

| Metric | Target | Comparable To |
|---|---|---|
| Lex 100K lines | <50ms | Faster than `rustc` lexer |
| Parse 100K lines | <100ms | Comparable to `swc` |
| Type check (managed) 100K lines | <500ms | Comparable to TypeScript |
| Type check (safe) 100K lines | <2s | Comparable to Rust (incremental) |
| Full compile (debug) 10K lines | <1s | Faster than Rust, comparable to Go |
| Full compile (release) 10K lines | <5s | Comparable to Rust |
| REPL startup | <100ms | Comparable to Ruby `irb` |
| Incremental rebuild (1 file changed) | <200ms | Faster than Rust |

---

## 13. Self-Hosting Roadmap

The ultimate goal is a self-hosting Garnet compiler written in Garnet:

1. **Bootstrap (current):** Compiler written in Rust, compiling Garnet to native code
2. **Stage 1:** Rewrite the managed-mode type checker in managed Garnet (proving the language is expressive enough for complex programs)
3. **Stage 2:** Rewrite the safe-mode type checker in safe Garnet (proving the ownership system works for real compiler code)
4. **Stage 3:** Full self-hosting — the Garnet compiler compiles itself
5. **Stage 4:** The compiler uses its own memory primitives for compilation caching (Frontier 2: compiler-as-agent)

---

## 14. Compiler Memory System (addresses Paper VI Contribution 3)

Paper VI Contribution 3 (Compiler-as-Agent) claims that the Garnet compiler itself uses Garnet's four memory types to learn from its own compilation history. This section specifies the storage formats and consultation points.

### 14.1 The `.garnet-cache/` directory

Every Garnet project containing a `Garnet.toml` has a per-project cache at `<project>/.garnet-cache/` and a per-user cache at `~/.garnet/cache/`. The per-project cache holds compilation-specific data; the per-user cache holds cross-project knowledge.

```
<project>/.garnet-cache/
├── episodes.ndjson     (working + episodic: compilation event log)
├── knowledge.db        (semantic: codebase facts, SQLite)
├── strategies.db       (procedural: validated optimization recipes, SQLite)
└── manifest.lock       (cache schema version + integrity hash)
```

### 14.2 Episodes log (episodic memory for the compiler)

Format: newline-delimited JSON (NDJSON), one event per line, append-only. Each event:

```json
{
  "timestamp": "2026-04-16T14:30:00.123Z",
  "compilation_id": "c4f3b891",
  "pass": "inlining",
  "subject": "fast_path::process",
  "decision": "inline",
  "rationale": "hot in prior build",
  "outcome_pending": true,
  "outcome": null
}
```

After the next benchmark run or production measurement, an `outcome` record updates the entry:

```json
{"compilation_id": "c4f3b891", "pass": "inlining", "subject": "fast_path::process",
 "outcome_applied": "2026-04-16T16:00:00Z", "outcome": "p99 latency -12%, code size +3%"}
```

**Retention.** R+R+I decay per `GARNET_Memory_Manager_Architecture.md §3.3` episodic defaults (λ = 0.01/day, θ = 0.3). Compaction runs daily; entries below threshold are archived to cold storage.

### 14.3 Knowledge database (semantic memory for the compiler)

SQLite schema capturing facts about the codebase:

```sql
CREATE TABLE function_stats (
    fq_name TEXT PRIMARY KEY,           -- fully-qualified name
    call_count INTEGER,                 -- across all compilations
    avg_body_size INTEGER,              -- lines of IR
    avg_callsite_count INTEGER,
    has_hot_label BOOLEAN,              -- tagged by profiler or #[hot]
    last_updated TIMESTAMP
);
CREATE TABLE type_usage (
    type_name TEXT PRIMARY KEY,
    instance_count INTEGER,
    avg_size_bytes INTEGER,
    monomorphization_count INTEGER      -- how many concrete instances in safe mode
);
CREATE TABLE platform_capabilities (
    target_triple TEXT,
    cpu_features TEXT,                  -- AVX2, NEON, etc.
    memory_model TEXT,
    PRIMARY KEY (target_triple)
);
```

**Consultation point (Phase 4).** During type checking, the checker queries `function_stats` to decide whether to speculatively inline small hot functions ahead of Phase 6's normal inlining pass. This does not change correctness — only pass ordering.

**Consultation point (Phase 6).** During code generation, the LLVM backend consults `type_usage` to prioritize monomorphizations of high-instance-count generic types, reducing cache misses.

### 14.4 Strategies database (procedural memory for the compiler)

SQLite schema capturing validated optimization recipes:

```sql
CREATE TABLE strategies (
    id INTEGER PRIMARY KEY,
    pattern TEXT,                       -- IR pattern (e.g., "hot loop with array copy")
    action TEXT,                        -- optimization to apply (e.g., "vectorize with prefetch")
    success_count INTEGER,
    failure_count INTEGER,
    avg_speedup REAL,                   -- geometric mean across successes
    last_validated TIMESTAMP
);
```

A strategy is invoked when: (a) its pattern matches current IR, (b) success_count / (success_count + failure_count) > 0.8, (c) last_validated is within 90 days. Otherwise the compiler falls back to its default heuristic and records a new observation.

**Consultation point (Phase 6, optimization passes).** Each pass consults the strategies DB before applying its default heuristic. This lets the compiler improve over time without code changes, which is the Paper VI Contribution 3 claim made concrete.

### 14.5 Cache invalidation

- **Compiler version change.** The whole cache is versioned; a compiler upgrade invalidates it.
- **Source hash change.** Per-file episodes are invalidated when the source file's SHA-256 changes.
- **Explicit reset.** `garnet clean --cache` nukes the whole cache.

### 14.6 Privacy

No cache content is ever transmitted off the user's machine without explicit opt-in. A future `garnet share-insights` command (post-Rung 5) may offer anonymized, aggregate sharing of strategies DB entries to a community repository, but it is strictly opt-in.

---

## 15. Deterministic Reproducible Builds (addresses Paper VI Contribution 7)

Paper VI Contribution 7 claims language-level reproducibility with provenance manifests. This section specifies the engineering.

### 15.1 The `--deterministic` flag

`garnet build --deterministic` (default in v0.3 release profile) guarantees that the same source + same compiler version + same target triple + same dependency lockfile produces byte-identical output.

Concrete measures:
- **No timestamps in output.** All `DateTime::now()` calls in codegen replaced with `0`.
- **Sorted iteration.** Symbol tables, type tables, dependency lists, module orders — all sorted lexicographically before serialization.
- **Seeded PRNG.** Any randomness in optimization (e.g., hash-based inlining heuristics) uses a fixed seed derived from `sha256(source_root_hash || compiler_version)`.
- **Canonical LLVM flags.** Set `-fno-asynchronous-unwind-tables`, `-frandom-seed=<fixed>`, disable profile-guided opts (which introduce non-determinism from runtime profiles).
- **Path canonicalization.** All embedded path strings are stripped to relative-from-project-root; no absolute user paths leak into binaries.

### 15.2 The provenance manifest format

```json
{
  "manifest_version": "1",
  "garnet_version": "0.3.0",
  "build_timestamp_utc": "2026-04-16T14:30:00Z",
  "target_triple": "x86_64-unknown-linux-gnu",
  "profile": "release",
  "deterministic": true,
  "sources": {
    "src/main.garnet": "sha256:abc123...",
    "src/lib.garnet": "sha256:def456...",
    "Garnet.toml": "sha256:789abc...",
    "garnet.lock": "sha256:fedcba..."
  },
  "dependencies": {
    "garnet-http": {"version": "0.2.4", "sha256": "..."},
    "garnet-memory": {"version": "0.1.8", "sha256": "..."}
  },
  "compiler_flags": ["--release", "--deterministic", "--lto"],
  "output_hash_pre_embed": "sha256:xyz789...",
  "output_size_bytes": 4392128,
  "signature": null
}
```

The `output_hash_pre_embed` is the hash of the binary BEFORE the manifest is embedded (so the hash is stable even though embedding changes the final binary).

### 15.3 Binary embedding

Platform-specific embedding preserves the manifest while avoiding interference with standard tooling:

- **ELF (Linux, BSD):** Manifest is added as a new section `.garnet_manifest` with `SHT_NOTE` type. Max size 4KB uncompressed (larger manifests compressed with zstd and flagged accordingly).
- **Mach-O (macOS):** New segment `__GARNET`, section `__manifest`. Same 4KB soft limit.
- **PE (Windows):** Resource `RT_CUSTOMDATA` with name `GARNET_MANIFEST`. Same 4KB soft limit.
- **WASM:** Custom section `garnet.manifest` per WebAssembly spec §2.5.

### 15.4 The `garnet verify` command

```bash
$ garnet verify ./my-agent
  ✓ Extracted manifest (.garnet_manifest section, 1.2KB)
  ✓ Compiler version: garnetc 0.3.0 ― matches current toolchain
  ✓ Target triple: x86_64-unknown-linux-gnu ― matches current host
  ✓ All 8 source files match recorded hashes
  ✓ All 2 dependencies match lockfile hashes
  ✓ Binary hash (minus manifest) matches output_hash_pre_embed
  ✓ Build is deterministically reproducible

$ garnet verify --rebuild ./my-agent
  # As above, plus actually rebuilds from source and byte-compares
  ✓ Rebuild produced byte-identical binary
```

The rebuild variant is the strongest check — proof that the binary was built from the declared source. It is also the slowest (requires a full release build), so routine `garnet verify` uses only hash-based verification.

### 15.5 Signatures (optional)

If the manifest's `signature` field is populated, `garnet verify` additionally validates the Ed25519 signature against either:
- A key embedded in the binary (for self-signed builds), or
- A key in the user's `~/.garnet/trusted_keys.toml` (for publisher-signed builds).

This enables supply-chain verification at the language level — a property no mainstream language currently offers.

### 15.6 Non-determinism escape hatches

Rare programs need non-determinism (e.g., cryptographic key generation at build time). The `--allow-nondeterminism` flag opts out of the guarantee; the resulting binary's manifest has `deterministic: false`, and `garnet verify` will report but not fail on the difference.

---

## 16. Relationship to Paper V

This architecture specification is the engineering counterpart to Paper V's formal treatment:

| Paper V Concept | Architecture Component |
|---|---|
| λ_managed sub-calculus | Phase 4a: Managed-Mode Type Checker |
| λ_safe sub-calculus | Phase 4b: Safe-Mode Type Checker |
| Bridging judgment | Phase 5: Boundary Validator |
| Progress + preservation | Type checker soundness (each phase preserves well-typedness) |
| Non-interference | Boundary validator ensures no unsafe state leaks across modes |
| Iris separation logic model | Safe-mode type checker's ownership reasoning |
| R+R+I decay formula (OQ-7) | Kind-aware allocation in §10 + Memory Manager SDK (Rung 5) |
| Session-typed actor protocols (OQ-8) | Actor Runtime in §11 |

---

*"In the multitude of counsellors there is safety." — Proverbs 11:14*
*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*
*"Where there is no vision, the people perish." — Proverbs 29:18*

**Compiler Architecture Spec v1.0 prepared by Claude Code (Opus 4.6) | April 16, 2026**
