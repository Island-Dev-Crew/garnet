# Garnet Tier 2 — Ecosystem Specifications
**Covers:** Package Manager, Standard Library, Interoperability, Async/Concurrency
**Date:** April 16, 2026
**Companion to:** Mini-Spec v0.3, Compiler Architecture Spec v1.0
**Anchor:** *"Where there is no vision, the people perish." — Proverbs 29:18*

---

# Part A: Package Manager and Module System

## A.1 The `garnet` Package Manager

Garnet ships a unified CLI that handles build, test, format, lint, doc, REPL, and package management — following Cargo's proven model (71% admiration, Stack Overflow 2025, highest for any developer tool).

### A.1.1 Garnet.toml Manifest

```toml
[package]
name = "my-agent"
version = "0.1.0"
edition = "2026"
authors = ["Jon <jon@islanddevcrew.com>"]
license = "MIT OR Apache-2.0"
description = "An agent-native service built with Garnet"
repository = "https://github.com/islanddevcrew/my-agent"

# Entry points
[[bin]]
name = "my-agent"
path = "src/main.garnet"

[lib]
path = "src/lib.garnet"

# Dependencies
[dependencies]
garnet-http = "0.2"
garnet-memory = "0.1"
garnet-actors = "0.1"

[dev-dependencies]
garnet-test = "0.1"
garnet-bench = "0.1"

# Build profiles
[profile.debug]
opt-level = 0
debug = true
backend = "cranelift"        # fast compilation for development

[profile.release]
opt-level = 3
debug = false
backend = "llvm"             # maximum optimization for production
lto = true
```

### A.1.2 Dependency Resolution

- **Semantic versioning** (SemVer) for all published packages
- **Lock file** (`garnet.lock`) pins exact versions for reproducible builds
- **Registry:** `registry.garnet-lang.org` (centralized, like crates.io)
- **Git dependencies:** `garnet-http = { git = "https://github.com/...", branch = "main" }`
- **Path dependencies:** `my-lib = { path = "../my-lib" }` for local development
- **Deterministic resolution:** same inputs always produce the same lock file (Frontier 4: deterministic reproducible builds)

### A.1.3 Editions

Garnet uses an edition system (borrowed from Rust) for opt-in breaking changes:
- `edition = "2026"` — initial edition
- Future editions can change syntax, defaults, or deprecations
- Packages from different editions interoperate seamlessly
- `garnet fix --edition 2028` assists migration

---

# Part B: Standard Library Outline

## B.1 Design Philosophy

The standard library follows three principles:
1. **Batteries included for the common case** — HTTP, JSON, testing, and memory primitives ship out of the box
2. **Thin wrappers, not reimplementations** — safe-mode stdlib delegates to proven Rust crates via FFI where possible
3. **Dual-mode aware** — every stdlib type has both a managed-mode API (ergonomic, ARC-backed) and a safe-mode API (zero-cost, ownership-based)

## B.2 Module Map

```
std::
├── prelude         # Auto-imported: Option, Result, String, Array, Map, print, println
├── io              # File, Stdin, Stdout, Stderr, BufReader, BufWriter
├── net             # TcpStream, TcpListener, UdpSocket, HttpClient, HttpServer
├── fs              # read, write, create_dir, walk, watch
├── fmt             # format!, Display trait, Debug trait
├── collections     # Vec, HashMap, HashSet, BTreeMap, BTreeSet, VecDeque, LinkedList
├── string          # String, str methods, regex
├── math            # abs, min, max, sqrt, sin, cos, PI, E, random
├── time            # Instant, Duration, SystemTime, sleep, interval
├── json            # parse, stringify, Value, to_json/from_json derive
├── concurrency     # Channel, Mutex, RwLock, Barrier, Semaphore (safe mode)
├── actor           # ActorSystem, Mailbox, Supervision, spawn, send
├── memory          # WorkingStore, EpisodeStore, VectorIndex, WorkflowStore
│   ├── working     # Arena-allocated short-lived state
│   ├── episodic    # Append-only chronological log
│   ├── semantic    # Persistent vector-indexed knowledge store
│   └── procedural  # COW workflow/recipe store
├── test            # #[test], assert!, assert_eq!, assert_ne!, should_panic
├── bench           # #[bench], Bencher, criterion-style benchmarking
├── serde           # Serialize, Deserialize traits + derive macros
├── error           # Error trait, anyhow-style error chaining, thiserror-style derive
├── iter            # Iterator trait, map, filter, reduce, collect, zip, enumerate
├── path            # PathBuf, Path, join, extension, parent
├── env             # args, vars, current_dir
├── process         # Command, spawn, pipe, exit
└── crypto          # sha256, hmac, aes (thin wrappers over proven implementations)
```

## B.3 Memory Module Detail

The `std::memory` module is Garnet's most distinctive stdlib contribution. It provides the four memory-type base implementations that the language-level `memory` declarations compile to:

```garnet
# std::memory::working
pub struct WorkingStore<T> {
  # Arena-allocated, bulk-freed at scope exit
  # Optimized for high-churn, short-lived data
}

# std::memory::episodic
pub struct EpisodeStore<T> {
  # Append-only log with timestamp indexing
  # Supports range queries: store.since(timestamp), store.recent(n)
  # Periodic compaction via configurable retention policy
}

# std::memory::semantic
pub struct VectorIndex<T> {
  # Persistent data structure with structural sharing
  # Vector similarity search: store.search(embedding, top_k)
  # CRUD with version tracking
}

# std::memory::procedural
pub struct WorkflowStore<T> {
  # Copy-on-write with version history
  # Supports: store.find(criteria), store.replay(version)
  # Execution traces are first-class values
}
```

The Memory Manager layer (OQ-7 decay formula, OQ-8 multi-agent consistency) sits ABOVE the stdlib implementations and is part of the runtime, not the language core — per four-model consensus point 8.

## B.4 Actor Module Detail

```garnet
# std::actor

pub struct ActorSystem {
  # The root supervisor and scheduler
  # Manages worker thread pool (default: num_cpus)
  # Provides spawn, shutdown, and monitoring APIs
}

pub trait Actor {
  # Auto-derived from `actor` declarations
  # Users don't implement this directly
}

pub struct Mailbox<M> {
  # Bounded channel for actor messages
  # Configurable capacity (default: 1024)
  # Back-pressure when full
}

pub enum SupervisionStrategy {
  OneForOne,      # restart only the failed child
  OneForAll,      # restart all children if one fails
  RestForOne,     # restart the failed child and all children started after it
}
```

---

# Part C: Interoperability Specification

## C.1 Rust FFI (safe mode)

Safe-mode Garnet modules are ABI-compatible with Rust. The mechanism:

```garnet
@safe
module CryptoBinding {
  # Import a Rust function via extern block
  extern "Rust" {
    fn sha256_digest(data: &[u8]) -> [u8; 32]
  }

  fn hash(borrow data: Bytes) -> Hash {
    let digest = sha256_digest(data.as_slice())
    Hash::from_bytes(digest)
  }
}
```

### How it works
- Safe-mode Garnet uses the same memory layout as Rust (repr(Rust) for structs, tagged unions for enums)
- `extern "Rust"` blocks declare functions from Rust libraries
- The Garnet compiler links against `.rlib` or `.a` files produced by `rustc`
- Shared types use `#[repr(C)]` for guaranteed layout compatibility

### Why this works (MIT-defensible)
Safe-mode Garnet's ownership model IS Rust's ownership model (Paper V §2.2, §6). The type systems are isomorphic for the subset of types that both languages support. The compiler can verify at link time that ownership contracts are satisfied across the FFI boundary.

## C.2 C ABI

```garnet
@safe
module SystemBinding {
  extern "C" {
    fn open(path: *const u8, flags: i32) -> i32
    fn read(fd: i32, buf: *mut u8, count: usize) -> isize
    fn close(fd: i32) -> i32
  }
}
```

- Standard C calling convention
- `*const T` and `*mut T` raw pointers available only in `@safe` modules with `extern "C"`
- All `extern "C"` calls are implicitly `unsafe` — the compiler trusts the programmer at this boundary

## C.3 Ruby Gem Embedding (managed mode)

```garnet
module LegacyIntegration {
  use ruby::VM

  def process_with_rails(data) {
    let vm = VM.new()
    vm.require("json")
    vm.eval("JSON.parse('#{data}')")
  }
}
```

- Managed-mode Garnet can embed a Ruby VM (CRuby/MRI)
- Values cross the boundary via JSON serialization (simple, robust) or via a native bridge (faster, more fragile)
- This enables incremental migration: existing Ruby gems continue to work while new code is written in Garnet
- The Ruby VM runs in a separate thread with its own GVL; Garnet's actor system handles the concurrency

## C.4 WebAssembly Compilation

```bash
garnet build --target wasm32-unknown-unknown
```

- Safe-mode code compiles to WASM via Cranelift's wasm32 target
- Managed-mode code requires bundling the Garnet bytecode VM as a WASM module
- Mixed-mode projects produce a WASM bundle containing both the native safe-mode code and the bytecode VM
- WASI support for server-side WASM (Cloudflare Workers, Deno, Wasmtime)

---

# Part D: Async/Concurrency Model

## D.1 Design Principle

Garnet solves the "colored functions" problem (identified in the original thesis as Rust's most significant ergonomic challenge) through a two-layer concurrency model:

1. **Actors** for coarse-grained parallelism (CPU-bound, isolated state)
2. **Structured async** for fine-grained I/O concurrency (within a single actor or function)

The key insight: actors handle parallelism (multiple cores), while async handles concurrency (multiple I/O operations). These are different problems and deserve different solutions.

## D.2 Structured Async (within functions)

```garnet
# Async functions use the same def/fn keywords — NO colored-function split
def fetch_all(urls) {
  # spawn creates a lightweight task (not a thread, not an actor)
  let tasks = urls |> map(|url| spawn Http.get(url))

  # await_all collects results — structured concurrency guarantees all tasks
  # complete before this function returns
  let responses = await_all(tasks)
  responses |> filter(|r| r.ok?)
}
```

### No colored functions
Garnet does NOT have separate `async def` / `def` keywords. Every function CAN be suspended if it calls a suspending operation (I/O, sleep, channel receive). The runtime transparently handles suspension and resumption. This is achieved through:

- **Green threads:** Managed-mode functions run on green threads (lightweight, user-space scheduled)
- **Work-stealing scheduler:** M:N threading model (M green threads on N OS threads)
- **Implicit suspension:** I/O operations yield the green thread automatically; no `await` keyword needed for simple cases
- **Explicit `spawn`:** Creates a new task for parallel execution
- **Explicit `await_all` / `await_any`:** Structured concurrency combinators for collecting results

**Design rationale (MIT-defensible):** Go proved that implicit suspension with green threads is viable at massive scale (goroutines serve billions of requests daily at Google). Rust chose explicit `async`/`await` because its ownership model requires knowing exactly when a value might be suspended (the `Pin` problem). Garnet avoids Rust's problem because managed-mode values are ARC-managed (no pinning needed) and safe-mode code that needs explicit control can use safe-mode channels instead of green-thread suspension.

## D.3 Channels (safe mode)

```garnet
@safe
module Pipeline {
  fn producer(own tx: Sender<Data>) {
    for i in 0..1000 {
      tx.send(Data::new(i))
    }
  }

  fn consumer(own rx: Receiver<Data>) -> Vec<Result> {
    let mut results = Vec::new()
    for data in rx {
      results.push(process(data))
    }
    results
  }

  fn run() -> Vec<Result> {
    let (tx, rx) = channel::<Data>(buffer: 64)
    spawn producer(tx)
    consumer(rx)
  }
}
```

Safe-mode code uses explicit channels for communication, following Rust's `mpsc` model. The ownership system ensures channels are used correctly — sending a value moves it into the channel, preventing data races at compile time.

## D.4 Actor-Async Interaction

Actors and async tasks coexist naturally:

```garnet
actor DataPipeline {
  memory working cache : WorkingStore<CacheEntry>

  protocol ingest(data: RawData) -> IngestResult

  on ingest(data) {
    # Inside a handler, we can use async operations freely
    let validated = spawn Validator.check(data)       # async task
    let enriched = Http.get("#{api_url}/enrich?id=#{data.id}")  # implicit async I/O
    let stored = cache.put(enriched)
    IngestResult::new(stored)
  }
}
```

Actor handlers run on the work-stealing scheduler. Within a handler, `spawn` creates child tasks. The actor processes one message at a time (sequential within the actor) but handler bodies can perform concurrent I/O.

## D.5 Structured Concurrency Guarantees

All spawned tasks are **scoped** — they cannot outlive their parent:

```garnet
def process_batch(items) {
  let results = scope |tasks| {
    for item in items {
      tasks.spawn(|_| transform(item))
    }
  }
  # All tasks are guaranteed complete here
  # No dangling tasks, no orphaned goroutines
  results
}
```

The `scope` block creates a task scope. All tasks spawned within the scope MUST complete before the scope exits. If any task panics, the scope cancels all remaining tasks and propagates the panic. This prevents the "orphaned goroutine" problem that plagues Go programs.

**Design rationale (MIT-defensible):** Structured concurrency was formalized by Martin Sustrik (2016) and refined by Nathaniel J. Smith's Trio library for Python. JEP 453 brought it to Java 21. Garnet adopts it as a first-class guarantee because agent systems MUST have predictable task lifecycles — a long-horizon agent that leaks tasks will eventually exhaust system resources.

---

# Part E: Verification Against Four-Model Consensus

| Consensus Point | Tier 2 Implementation |
|---|---|
| 1. Rust/Ruby complementary | C.1 (Rust FFI), C.3 (Ruby embedding) — interop with both ancestors |
| 2. Dual-mode correct | D.2 (no colored functions in managed), D.3 (channels in safe) |
| 3. Swift precedent | B.2 (ARC in stdlib mirrors Swift), D.4 (actor model mirrors Swift actors) |
| 4. Agent-native platform | B.3 (memory module), D.4 (actor-async interaction) |
| 5. One Memory Core, Many Harnesses | B.3 (stores are core), B.4 (ActorSystem is harness-layer) |
| 6. Memory primitives first-class | B.3 (four memory stores in stdlib) |
| 7. Typed actors compiler-enforced | B.4 (Actor trait auto-derived), D.5 (structured scope) |
| 8. TurboQuant = runtime | B.3 (Memory Manager is runtime, not stdlib core) |

---

*"In the multitude of counsellors there is safety." — Proverbs 11:14*
*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*
*"Where there is no vision, the people perish." — Proverbs 29:18*

**Tier 2 Ecosystem Specifications prepared by Claude Code (Opus 4.6) | April 16, 2026**
