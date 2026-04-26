# Memory Core — Production Roadmap

**Subject:** Garnet's Memory Core (the architectural subsystem) and **Mnemos** (its v0.4.x reference implementation crate, `garnet-memory-v0.3/`).
**Status of this document:** Forward-looking. Work items are not committed to a delivery date here; that belongs in per-version handoffs in `F_Project_Management/`.
**As of:** 2026-04-26 (Garnet release v0.4.2).

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

Implemented, tested, behaviourally correct against the Mini-Spec §4 contract. Not allocator-aware; not persistent; not production-throughput.

| Kind | Reference store | File | Tests |
|---|---|---|---|
| Working | `RefCell<Vec<T>>` arena | `garnet-memory-v0.3/src/working.rs` | `tests/basic.rs`, `tests/properties.rs` |
| Episodic | `RefCell<Vec<Episode<T>>>` append log | `src/episodic.rs` | ditto |
| Semantic | `RefCell<Vec<(Vec<f32>, T)>>` flat-cosine index | `src/semantic.rs` | ditto + `benches/vector.rs` |
| Procedural | `RefCell<BTreeMap<Version, T>>` COW store | `src/procedural.rs` | ditto |
| Policy | `MemoryPolicy { score, should_retain }` | `src/policy.rs` | ditto |

These will not be removed or replaced wholesale. Each tier below either upgrades the *backend* of one store or adds a *new* allocator surface that the existing public types can switch to.

---

## Tier 1 — Allocator integration (Mnemos v0.5.0)

The biggest single jump in maturity. Today's stores all allocate through standard Rust collections (`Vec`, `BTreeMap`). Tier 1 introduces a kind-aware allocator that the four stores delegate to, so retention policy and eviction become first-class instead of advisory.

### T1.1 — Kind-aware allocator trait

Define a `KindAllocator` trait that knows the four memory kinds and their retention semantics. Each store gets an associated allocator type. Reference impl uses bumpalo-style arenas for working, slab pools for episodic/procedural, and a flat vector for semantic — already a real win over `Vec::new()` because allocations can be tracked, capped, and reset per scope.

- **References:** Paper VI §4 (kind-aware memory allocation as one of the seven novel contributions); Mini-Spec §4.2.
- **Risk:** medium — reshapes every store's `new()` API. Mitigated by introducing the trait first as an additive parameter with a `Default` impl that matches today's behaviour.

### T1.2 — Eviction policy enforcement

Today `MemoryPolicy` exposes `score()` and `should_retain()` but no store calls them. Tier 1 wires the policy into actual eviction loops in episodic and semantic, so that capped stores have a defined behaviour at the boundary instead of growing unbounded.

- **References:** `MemoryPolicy::score(relevance, age, importance)` per `policy.rs:53`; the R+R+I decay model.
- **Open design question:** Synchronous eviction during write, vs. background sweep, vs. lazy at read. Recommend lazy at read for v0.5; revisit when production workloads exist.

### T1.3 — Generics over memory kinds (Mini-Spec §4.4)

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

`EpisodeStore` is in-memory today. For an agent harness that survives process restart, episodic memory has to persist. Tier 2 adds a pluggable persistence backend (default: append-only file in `.garnet-cache/episodic/`, mirroring the existing episode log structure in `garnet-cli/src/cache.rs`).

- **References:** Mini-Spec §4.2 (semantics — does not require persistence but does not forbid it); Paper VI Contribution 3 (compiler-as-agent uses persistent episodes today via `cache.rs`, so the pattern is in production already).

### T2.4 — Procedural store transactional versioning

`WorkflowStore` uses copy-on-write with a `BTreeMap<Version, T>` — correct but unbounded. Tier 2 adds true transactional semantics (commit/rollback boundaries, optional snapshot pruning) so a procedural memory can be safely shared across reload cycles (the actor-runtime hot-reload path, `garnet-actor-runtime/src/statecert.rs`).

- **References:** Mini-Spec §4.2; v3.3 StateCert hot-reload integration.

---

## Tier 3 — Safe-mode integration (Mnemos v0.6+)

The hardest single Memory-Core item, and the most important for the language's safety story. Today's reference stores are managed-mode only.

### T3.1 — ARC + Bacon–Rajan cycle detection (Mini-Spec §4.5)

The biggest open spec item (🟠 in the conformance matrix). Implements the synchronous trial-deletion algorithm with **kind-aware roots** — the working/episodic/semantic/procedural taxonomy gives the cycle collector partition information that hardware-allocator-only languages cannot exploit.

- **References:** Mini-Spec §4.5 (with sub-rules .5.1 through .5.5); Paper V Addendum Theorem A (ARC + kind-partitioned cycle collection).
- **Pre-requisite:** Tier 1 allocator integration (cycle detector needs to walk the allocator's roots, not the user's).
- **Risk:** high — this is research-grade work and the spec acknowledges it as such. Plan: build the synchronous variant first, validate against Bacon–Rajan's published test cases, then add kind-aware partitioning as a measurable optimization.

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
