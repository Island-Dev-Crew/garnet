# Memory Core — Production Roadmap

**Subject:** Garnet's Memory Core (the architectural subsystem) and **Mnemos** (its v0.4.x reference implementation crate, `garnet-memory-v0.3/`).
**Status of this document:** Forward-looking. Work items are not committed to a delivery date here; that belongs in per-version handoffs in `F_Project_Management/`.
**As of:** 2026-05-08 (v0.5 readiness Phase 6O in progress).

---

## Why this document exists

Memory engineering is one of two load-bearing differentiators for Garnet (the other is the dual-mode managed/safe boundary). It is the primary target of Paper IV's "One Memory Core, Many Harnesses" architecture and Paper VI's contribution #4 ("kind-aware memory allocation").

Today's reference implementation (Mnemos) ships behavioural-contract stores so the rest of the language (parser, interpreter, checker) can target a stable Memory Core API while the production allocator path is built out separately. That separation is intentional, but it also means the Memory Core's *implementation surface area* is the largest single open block of work between v0.4.2 and a production-credible v0.5+ release.

This roadmap names that work, organizes it by tier, and pins each item to its Mini-Spec / Paper reference so the trail from research to ticket is one click.

The naming convention used throughout:

- **Memory Core** — the architectural noun. Stable across the maturity transition. Used in Mini-Spec, papers, conformance matrix, talks-as-architecture.
- **Mnemos** — the implementation. Used for the crate, code-side docs, talks-as-product. Mnemos matures from "v0.4.x reference stores" to "v0.5.x production allocator" without changing the noun.

---

## Tier 0 — what already ships in v0.4.2 (Mnemos reference stores)

Implemented, tested, behaviourally correct against the Mini-Spec §4 contract. The reference stores now expose kind-aware allocator statistics, cycle-aware root lifecycle evidence, a narrow episodic text snapshot path, guarded append-style episodic text log commits, a fixed typed episodic cache backend boundary, and Unix directory-sync durability evidence after accepted text commits. They are not full production-throughput backends, and persistence is not yet generalized across all memory kinds.

| Kind | Reference store | File | Tests |
|---|---|---|---|
| Working | `RefCell<Vec<T>>` arena with Phase 6K cycle-aware roots on push, clear, and drop | `garnet-memory-v0.3/src/working.rs` | `tests/basic.rs`, `tests/properties.rs` |
| Episodic | `RefCell<Vec<Episode<T>>>` append-style log with Phase 6K root release on policy eviction/drop, Phase 6L versioned text snapshot save/load, Phase 6M guarded text log commits, Phase 6N `.garnet-cache/episodic/episodes.mnemos` backend guardrails, and Phase 6O Unix directory-sync durability after atomic rewrite/rename | `src/episodic.rs` | ditto + `tests/persistence.rs` |
| Semantic | `RefCell<Vec<(Vec<f32>, T)>>` flat-cosine index with Phase 6K root release on policy eviction and drop | `src/semantic.rs` | ditto + `benches/vector.rs` |
| Procedural | `RefCell<BTreeMap<Version, T>>` COW store with Phase 6K root release on workflow replacement and drop | `src/procedural.rs` | ditto |
| Allocator surface | object-safe `KindAllocator`, `HeapKindAllocator`, `CycleAwareKindAllocator`, `AllocStats`, and `AllocRootStats` | `src/alloc.rs` | `tests/properties.rs` |
| Policy | `MemoryPolicy { score, should_retain }` | `src/policy.rs` | ditto |
| Cycle fixtures | deterministic rooted graph + bounded root-buffer trial-deletion, finalization-order, and safe-mode exclusion model | `src/cycle.rs` | `tests/cycle.rs`, `deferred_arc_cycle_detection` |

These will not be removed or replaced wholesale. Each tier below either upgrades the *backend* of one store or adds a *new* allocator surface that the existing public types can switch to.

---

## Tier 1 — Allocator integration (Mnemos v0.5.0)

The biggest single jump in maturity. Phase 6J starts this tier: stores still use standard Rust collections (`Vec`, `BTreeMap`) for backing storage, but they now delegate allocation intent to a kind-aware allocator surface and policy-configured episodic/semantic stores enforce retention lazily on read/search. Phase 6K connects that allocator surface to observable store-root lifecycles through a cycle-aware adapter: writes retain roots, clear/policy eviction/replacement/drop release them, and `AllocRootStats` makes the behavior testable. Phase 6L starts a fenced episodic text snapshot path so root lifecycle evidence survives save/load recovery. Phase 6M extends that path with guarded append-style text log commits that refuse corrupt, empty, type-invalid, or oversized logs before mutating live memory. Phase 6N wraps the same typed text format in a fixed `.garnet-cache/episodic/episodes.mnemos` backend with path, size, permission, and serialization guardrails. Phase 6O adds Unix directory-sync durability after atomic text commit renames, using the validated episodic directory handle for the default cache backend. Full custom backends, production ARC finalizers, and broad persistence remain later Tier 2/Tier 3 work.

### T1.1 — Kind-aware allocator trait

Phase 6J defines an object-safe `KindAllocator` trait that knows the four memory kinds and records typed allocation requests through `AllocRequest` / `AllocStats`. Each store accepts an allocator with a default `HeapKindAllocator` that preserves the existing public constructors. Future custom slabs/arenas can slot in behind this surface without making the interpreter-facing store types generic over allocator implementations.

- **References:** Paper VI §4 (kind-aware memory allocation as one of the seven novel contributions); Mini-Spec §4.2.
- **Risk:** medium — reshapes every store's `new()` API. Mitigated by introducing the trait first as an additive parameter with a `Default` impl that matches today's behaviour.

### T1.2 — Eviction policy enforcement

Phase 6J wires `MemoryPolicy::score()` and `should_retain()` into policy-configured `EpisodeStore` and `VectorIndex` instances. Defaults remain unbounded for v0.4.x compatibility; stores created through `with_policy` lazily compact on `recent` / `since` / `snapshot` and `search`, enforcing retention thresholds and high-water caps without background threads.

- **References:** `MemoryPolicy::score(relevance, age, importance)` per `policy.rs:53`; the R+R+I decay model.
- **Decision in Phase 6J:** lazy at read/search for v0.5. Revisit background sweeps when production workloads and persistence exist.

### T1.3 — Cycle-aware store-root lifecycle

Phase 6K adds `CycleAwareKindAllocator`, `AllocRootStats`, and object-safe root
hooks on `KindAllocator`. Working, episodic, semantic, and procedural stores
retain roots when values enter the store and release those roots when the store
clears, policy eviction compacts, a workflow is replaced, or the store drops.
This makes the Tier 1 allocator boundary observable without claiming that the
bounded cycle fixture is the final production ARC collector.

- **References:** Mini-Spec §4.5; Paper V Addendum Theorem A.
- **Decision in Phase 6K:** keep the adapter fixture-backed and object-safe so
  stores can prove lifecycle behavior while the production allocator backend is
  still designed separately.

### T1.4 — Generics over memory kinds (Mini-Spec §4.4)

Currently explicitly deferred. Tier 1 introduces this — without it, library code that wants to be generic over "the user picks the kind at instantiation" has to monomorphize manually, which Paper VII flags as a tooling-ergonomics gap.

- **References:** Mini-Spec v1.0 §4.4 (currently 🟠 in the conformance matrix).
- **Pre-requisite:** §11.6 monomorphization needs to actually monomorphize (currently parsed-only) — see [conformance matrix](GARNET_v0_4_2_Conformance_Matrix.md) §11.6.

---

## Tier 2 — Production-grade backends (Mnemos v0.5.x — v0.6)

Each tier-2 item replaces one Tier-0 reference store with a backend that handles real workloads. They are independent — can land in any order.

### T2.1 — HNSW / IVF semantic index

`VectorIndex` today does flat cosine over a `Vec`. That is O(n) per query and breaks down past ~10k vectors. Tier 2 swaps the backend for HNSW (recommended) or IVF, exposing the same `add` / `query_top_k` / `len` API.

- **References:** Paper IV Appendix B (PolarQuant + QJL mathematical mechanics — eventual integration).
- **Open design question:** Pure-Rust impl (e.g. `instant-distance`) vs. wrap an established library. Recommend pure-Rust for the dependency-audit story (`cargo deny` already enforced in `.github/workflows/security.yml`).

### T2.2 — PolarQuant vector compression

The Memory Core sits on the same kind-aware infrastructure that Paper IV's Recursive Language Models target. PolarQuant compresses high-dimensional vectors to ~1 bit/dim with bounded similarity loss; for episodic retrieval at agent timescales, that's the difference between "fits in memory" and "doesn't."

- **References:** Paper IV Addendum v1.0 (Recursive Language Models + PolarQuant bridge); v3.3 Compression Techniques Reference.
- **Tier:** stretch within v0.5.x; gates on T2.1 landing first.

### T2.3 — Episodic persistence layer

`EpisodeStore` now has a narrow versioned text snapshot API, guarded append-style text commits, and a default typed cache backend boundary. For an agent harness that survives process restart under real workload pressure, episodic memory still has to mature toward a pluggable backend family, but the first default file target is now explicit at `.garnet-cache/episodic/episodes.mnemos`.

- **References:** Mini-Spec §4.2 (semantics — does not require persistence but does not forbid it); Paper VI Contribution 3 (compiler-as-agent uses persistent episodes today via `cache.rs`, so the pattern is in production already).
- **Phase 6L first slice:** `EpisodeStore::save_text` / `load_text` provide a deliberately narrow, dependency-free, versioned text snapshot API for `T: ToString + FromStr`. Payloads are hex-encoded so tabs/newlines cannot corrupt record boundaries, writes go through a sibling temp file and rename, malformed files are parsed all-or-nothing before mutating the live store, and recovery rehydrates cycle-aware store roots.
- **Phase 6M guardrail:** `EpisodeStore::append_text` commits individual records to the same versioned text format, size-bounds and validates any existing log against the store value type before extension, rejects projected oversize commits, refuses corrupt, empty, type-invalid, or oversized logs without mutating live memory, and syncs accepted record data through a temp-file rewrite before calling `append_at`.
- **Phase 6N backend boundary:** `EpisodeStore::append_cache_text` / `load_cache_text` bind the typed text format to fixed per-project `.garnet-cache/episodic/episodes.mnemos` storage. The backend canonicalizes the project root, creates private cache directories, rejects symlinked or non-regular cache targets, rejects oversized loads before reading into memory, serializes rewrite-based access with an OS-backed lockfile on Unix/Windows, anchors Unix backend file operations to the validated episodic directory handle, keeps Unix backend files at `0600` from creation time, preserves all-or-nothing parse behavior, and rehydrates cycle-aware roots on load. This is file-only Mnemos evidence, not the CLI signed NDJSON cache, not trusted compiler advice input, and not a broad pluggable persistence layer.
- **Phase 6O durability guardrail:** accepted `save_text`, `append_text`, and Unix prepared-cache commits now sync the containing directory after the temp-file rewrite is renamed into place. The default cache backend performs that sync through the already-open validated episodic directory handle, so the durability step does not re-follow a mutable path. Non-Unix platforms retain the prior file-data sync behavior until a platform-specific directory-sync contract is added.

### T2.4 — Procedural store transactional versioning

`WorkflowStore` uses copy-on-write with a `BTreeMap<Version, T>` — correct but unbounded. Tier 2 adds true transactional semantics (commit/rollback boundaries, optional snapshot pruning) so a procedural memory can be safely shared across reload cycles (the actor-runtime hot-reload path, `garnet-actor-runtime/src/statecert.rs`).

- **References:** Mini-Spec §4.2; v3.3 StateCert hot-reload integration.

---

## Tier 3 — Safe-mode integration (Mnemos v0.6+)

The hardest single Memory-Core item, and the most important for the language's safety story. Today's reference stores are managed-mode only.

### T3.1 — ARC + Bacon–Rajan cycle detection (Mini-Spec §4.5)

The biggest open spec item. Phase 6A added a bounded, deterministic reference
model in `garnet-memory-v0.3/src/cycle.rs`: rooted nodes stay live, unrooted
acyclic nodes are left for ordinary retention/eviction policy, unrooted cycles
are collected, and kind-partitioned scans collect a cross-kind component as a
whole when a matching kind triggers the scan. Phase 6B tightens that model into
a bounded trial-deletion pass with explicit trial candidates, mark-gray,
scan/scan-black, and collect-white behavior. Phase 6C adds deterministic
finalization-order reporting and safe-mode affine node exclusion so §4.5.3 and
§4.5.4 are executable at the reference-model layer. This is evidence for the
observable invariants, not the production collector. Phase 6D adds
`CycleRootBuffer` and `release_root_to_buffer` so decrement-triggered buffered
roots can drive collection instead of scanning every unrooted candidate.
Phase 6E wraps that behavior in `CycleAllocatorFixture`, and Phase 6K adds
`CycleAwareKindAllocator` so the four reference stores can retain and release
observable roots through the allocator surface on write, clear, eviction,
replacement, and drop.

The remaining production item is the synchronous Bacon-Rajan trial-deletion
algorithm integrated with ARC-managed allocator roots. That final path keeps
the **kind-aware roots** design — the working/episodic/semantic/procedural
taxonomy gives the cycle collector partition information that
hardware-allocator-only languages cannot exploit.

- **References:** Mini-Spec §4.5 (with sub-rules .5.1 through .5.5); Paper V Addendum Theorem A (ARC + kind-partitioned cycle collection).
- **Pre-requisite for production collector:** Tier 1 allocator integration
  (cycle detector needs to walk the allocator's roots, not only the fixture
  graph).
- **Risk:** high — this is research-grade work and the spec acknowledges it as
  such. Plan: keep the Phase 6A/6B/6C/6D reference fixtures green, build the
  synchronous ARC-integrated variant, validate against Bacon-Rajan's published
  test cases, then measure kind-aware partitioning as an optimization.

### T3.2 — Safe-mode `Sendable` interaction

Memory units crossing actor boundaries need to satisfy `Sendable` (Mini-Spec §9.4). Today there is partial enforcement; Tier 3 closes the loop so that a `WorkingStore` cannot be sent across the actor boundary even by accident, while `EpisodeStore` and `VectorIndex` can be.

- **References:** Mini-Spec §9.4 (Sendable + Actor Isolation Theorem); conformance matrix §9.4 (currently 🟡).

### T3.3 — Mode-boundary audit hooks

Every read/write across the managed/safe boundary that touches a memory unit should emit a ModeAuditLog entry, so the existing `garnet-check-v0.3/src/audit.rs` machinery sees Memory Core operations as first-class. Today the audit log is fn↔def crossings only.

- **References:** `garnet-check-v0.3/src/audit.rs`; v3.5 Security Layer 3.

---

## Tier 4 — Tooling and observability (rolling)

Items that don't gate on Tier 1–3 but make the Memory Core useful to *operate*, not just *use*.

| Item | Description | Reference |
|---|---|---|
| T4.1 | `garnet inspect memory <store>` CLI subcommand — dump store contents + policy state | new |
| T4.2 | `MemoryHandle::stats()` — uniform per-kind metrics (size, score histogram, eviction count) | extends `lib.rs:24` |
| T4.3 | tracing-crate integration — every store op emits a span tagged with kind + handle name | new |
| T4.4 | LSP hover for `memory` declarations — show backend kind, capacity, current population | gates on Refactor #7 (LSP) |

---

## Sequencing principle

The above is not a strict serial order. The principle:

1. **Tier 1 lands first** — every later tier assumes a real allocator surface exists.
2. **Tier 2 and Tier 3 can interleave** — T3.1 (ARC) is harder than T2.1 (HNSW), but ARC is more strategically important. Recommend T2.1 + T3.1 in parallel, with T2.1 acting as the integration test bed for the new allocator API.
3. **Tier 4 is rolling** — pick items as they unblock specific external work (T4.4 unblocks editor experience; T4.1 unblocks ops debugging).

## What does NOT belong here

- **API churn for its own sake.** Mnemos's public types are stable. Production backends slot in behind them.
- **Vendor-specific allocator bindings.** Garnet ships pure-Rust by default; integrating tcmalloc / jemalloc / mimalloc is a downstream choice.
- **GC-style replacements.** Garnet's safe-mode story is ARC + cycle detection (Mini-Spec §4.5), not tracing GC. Switching collectors is out of scope.

## How to keep this document honest

When a Tier item lands, do the same thing the conformance matrix policy says to do: flip the row in the *same commit* that lands the work, and update the §4.x rows of the conformance matrix to match. Stale roadmaps are worse than no roadmap because they pretend to inform.
