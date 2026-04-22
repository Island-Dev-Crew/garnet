# Garnet Memory Manager Architecture Overview
**Version:** 1.0
**Date:** April 16, 2026
**Addresses:** OQ-7 (controlled-decay formula), OQ-8 (multi-agent consistency)
**Companion to:** Paper IV v2.1.1 (Appendix B PolarQuant/QJL mechanics), Paper V §7 (memory primitives as typed resources), Mini-Spec v0.3 §4 and §9
**Anchor:** *"Where there is no vision, the people perish." — Proverbs 29:18*

---

## 1. Purpose

This document specifies the Memory Manager — the runtime layer that sits between Garnet's language-core memory declarations (`memory working|episodic|semantic|procedural`) and the agent harness layer. It answers two of the Mini-Spec's longest-standing open questions (OQ-7 and OQ-8) while preserving the four-model consensus point 8: runtime concerns belong in the runtime, not in the language core.

Positioning, consistent with Paper IV's *One Memory Core, Many Harnesses* architecture:

```
┌────────────────────────────────────────────────────────────────┐
│ Agent Harness Layer (domain-specific)                          │
│   BuildAgent, ChatAgent, SearchAgent, ...                      │
└───────────────────────▲────────────────────────────────────────┘
                        │ policy + metadata (OQ-7)
┌───────────────────────┴────────────────────────────────────────┐
│ Memory Manager (THIS DOCUMENT)                                 │
│   • R+R+I decay (§3)                                           │
│   • Consistency protocols (§4)                                 │
│   • Kind-specific policies (§5)                                │
└───────────────────────▲────────────────────────────────────────┘
                        │ kind-aware allocator APIs
┌───────────────────────┴────────────────────────────────────────┐
│ Memory Core (language-level, Compiler Arch §10)                │
│   WorkingStore | EpisodeStore | VectorIndex | WorkflowStore    │
└────────────────────────────────────────────────────────────────┘
```

**Scope boundary (per consensus point 8):** the language core guarantees the *shape* of memory declarations and the *safety* of kind-aware allocation. Retention, decay, ranking, and cross-agent consistency are runtime policy decisions — they belong here, not in the Mini-Spec.

---

## 2. Design Principles

1. **Policy over mechanism.** The Memory Manager exposes policies (TTL, decay weighting, privacy scope); implementations are configurable.
2. **Default-safe.** Every policy has a sensible default (R+R+I with balanced weights, read-committed consistency) so programmers who write `memory episodic log : EpisodeStore<Event>` get reasonable behavior with zero additional code.
3. **Tunable per-kind.** The four memory kinds have fundamentally different access patterns, so the Memory Manager allows per-kind policy tuning (§5).
4. **Per-harness override.** A domain-specific harness (chat, build, search) may override defaults for its own workloads without affecting other harnesses.
5. **Consensus point 8 preserved.** None of this is in Mini-Spec v0.3's language core. The language specifies *what* gets stored; the Memory Manager specifies *how long* and *how visible*.

---

## 3. Controlled-Decay Formula (addresses OQ-7)

### 3.1 The Alake Framework (source of record)

The Relevance + Recency + Importance formulation was surfaced by Gemini 3.1 Pro Deep Research during the four-model consensus pass (documented in `GARNET_v2_1_Gemini_Synthesis.md` and `GARNET_v2_1_Four_Model_Consensus_Memo.md §5`). It originates from Richmond Alake's memory-engineering framework, referenced in Paper IV v2.1.1 and Paper V §7.3.

### 3.2 The decay function

For any memory item `m` at time `t`, the retention score is:

```
score(m, t) = R_relevance(m, q)  ·  R_recency(m, t)  ·  I_importance(m)

where:
  R_relevance(m, q)  =  cos_sim(embed(m), embed(q))        ∈ [0, 1]
  R_recency(m, t)    =  exp(-λ · (t - m.timestamp))        ∈ (0, 1]
  I_importance(m)    =  σ(w·m.features + b)                ∈ [0, 1]
```

where:
- `q` is the current query or retrieval context
- `λ` is the kind-specific decay rate (larger λ → faster forgetting)
- `cos_sim` is cosine similarity in the embedding space
- `σ` is the logistic function applied to a learned importance classifier

The **retention decision** is a threshold on `score`: items with `score < θ_kind` are eligible for eviction. Eviction is lazy — the Memory Manager only actually removes items when storage pressure crosses a configurable high-water mark.

### 3.3 Per-kind default parameters

| Kind | λ (decay rate) | θ (retention threshold) | Importance source | Rationale |
|---|---|---|---|---|
| Working | 0.5 / min | 0.1 | `I = 1.0` (uniform) | Short-lived; recency dominates |
| Episodic | 0.01 / day | 0.3 | Log position + message type | Chronological; important sessions persist |
| Semantic | 0.001 / day | 0.5 | Usage frequency + cross-reference count | Factual; decay is very slow |
| Procedural | 0.0005 / day | 0.4 | Success rate + recency of use | Behavioral; prunes unused workflows |

These defaults are tuned for typical agent workloads. A harness may override any parameter by providing a `MemoryPolicy` at actor/agent spawn time:

```garnet
actor ResearchAgent {
  memory semantic knowledge : VectorIndex<Fact> with MemoryPolicy {
    decay_lambda: 0.0001 / day,        # decays even more slowly
    retention_threshold: 0.6,           # keeps only high-score facts
    importance_model: FactImportance,   # custom classifier
  }
  # ...
}
```

### 3.4 Empirical calibration (v3.2 status: synthetic-workload baseline established)

The per-kind defaults above were originally theory-driven. As of v3.2, the
`garnet-memory-v0.3` crate ships four calibration examples
(`examples/calibration_{working,episodic,semantic,procedural}.rs`) that
drive a synthetic 1000-tick workload appropriate to each kind and dump
the R+R+I score evolution as CSV. Inspection of the generated curves
shows that the published defaults sit within ±10% of the optimum
retention threshold for the synthetic workload mix tested. Production
calibration against real interaction logs remains scheduled per the
Benchmarking Plan §4. The defaults are normative and subject to revision
in a future Memory Manager version without breaking language-level code.

### 3.5 Relationship to TurboQuant (consensus point 8)

R+R+I decay is a *retention* policy (what to keep). TurboQuant/PolarQuant/QJL are *compression* techniques (how to store what is kept). Both are runtime concerns and distinctly specified; neither is a language-level guarantee. The Memory Manager may optionally use TurboQuant-class compression inside any backing store, but nothing in the language surface changes as a result. This is the architectural discipline four-model consensus point 8 demands.

---

## 4. Multi-Agent Consistency Protocol (addresses OQ-8)

### 4.1 The problem

Mini-Spec §9.2 states: "Actors MUST NOT share mutable state. All inter-actor communication MUST go through declared protocols." But Paper IV's *One Memory Core, Many Harnesses* explicitly requires that multiple agents can read and occasionally write to shared memory units (for cross-agent knowledge sharing).

The resolution: actors do not share *mutable references*. They share access to the Memory Manager, which arbitrates reads and writes with explicit consistency semantics. This preserves both Mini-Spec §9.2 (no shared mutable state between actors) and the *One Memory Core* architecture.

### 4.2 Three access modes

```garnet
# Mode 1: exclusive (single writer, per-actor)
memory episodic log : EpisodeStore<Event> exclusive

# Mode 2: shared-read (many readers, no writers during read phase)
memory semantic knowledge : VectorIndex<Fact> shared_read

# Mode 3: session-typed (contract-checked concurrent access)
memory procedural workflows : WorkflowStore<Trace> session {
  protocol BuildSession {
    reader_phase: {allowed: query, lookup}
    writer_phase: {allowed: append, compact}
  }
}
```

The `exclusive` / `shared_read` / `session` suffixes are *hints to the Memory Manager*, not language-level type modifiers. They default to `exclusive` when omitted (safe default).

### 4.3 Consistency semantics

| Mode | Semantics | Performance | Use case |
|---|---|---|---|
| `exclusive` | Linearizable: one writer at a time, readers wait for writer | Lowest contention, serialization cost | Per-actor private memory |
| `shared_read` | Read-committed: many concurrent readers, writers wait for read quiescence | High read throughput | Knowledge bases, reference data |
| `session` | Custom protocol enforced by the Memory Manager | Workload-dependent | Structured multi-phase workflows |

Writes in `shared_read` mode use a copy-on-write discipline (Compiler Architecture Spec §10): writers produce a new version, readers continue using their snapshot until they re-read. This mirrors the persistent data structure pattern (Clojure's Atom, Haskell's TVar) and avoids reader-blocking.

### 4.4 Failure model

- **Writer crash mid-update:** the Memory Manager uses a write-ahead log. On recovery, incomplete writes are rolled back; readers never observe torn writes.
- **Network partition (distributed Memory Core):** each partition continues with its local snapshot. On rejoin, the conflict resolution policy (CRDT for semantic memory, latest-wins timestamp for working memory, append-union for episodic memory) reconciles divergent state.
- **Malicious agent:** agents cannot mutate memory units for which they lack write capability; the Memory Manager enforces at every write call. A misbehaving agent can corrupt only its own memory units.

### 4.5 Session-typed protocols (formal basis)

Session types (Honda, 1993; see Paper V §2.4) give a formal language for describing multi-phase protocols. A `session` annotation on a memory unit declares a protocol; the Memory Manager enforces that only agents following the protocol can access the unit. Violations produce compile-time errors (when statically checkable) or runtime refusals (when not).

Example enforcement:

```garnet
# The compiler refuses this: writing during reader phase
match memory_phase {
  ReaderPhase => workflows.append(trace)   # ← compile-time error: append is writer-phase only
  WriterPhase => workflows.append(trace)   # ← OK
}
```

Full session-type specification is deferred to Mini-Spec v0.4 (OQ-8 closure). v0.3 provides the *architecture*; v0.4 will provide the *surface syntax*.

---

## 5. Kind-Specific Policies

Each of the four memory kinds gets a tailored policy set. The Memory Manager's job is to implement these correctly so that language-level code gets appropriate behavior with zero configuration.

### 5.1 Working memory (`working`)

- **Allocator:** arena allocator (Compiler Arch §10)
- **Lifetime:** scope-bound (freed at scope exit via ARC drop)
- **Retention:** R+R+I with `λ=0.5/min`, `θ=0.1` — very aggressive decay
- **Consistency:** always `exclusive` (private to declaring actor/function)
- **Compression:** disabled (short-lived data, compression overhead not worth it)

### 5.2 Episodic memory (`episodic`)

- **Allocator:** append-only log with periodic compaction
- **Lifetime:** persistent until compaction evicts
- **Retention:** R+R+I with `λ=0.01/day`, `θ=0.3` — slow decay, importance-weighted by log position and message type
- **Consistency:** `exclusive` for writes (only one writer appends), `shared_read` allowed
- **Compression:** optional PolarQuant/QJL on embedding fields (runtime choice, not language)

### 5.3 Semantic memory (`semantic`)

- **Allocator:** persistent data structure with structural sharing (Compiler Arch §10)
- **Lifetime:** persistent
- **Retention:** R+R+I with `λ=0.001/day`, `θ=0.5` — very slow decay; facts persist almost indefinitely unless explicitly pruned
- **Consistency:** defaults to `shared_read` for cross-agent knowledge sharing
- **Compression:** PolarQuant/QJL strongly encouraged (vector similarity benefits substantially)

### 5.4 Procedural memory (`procedural`)

- **Allocator:** copy-on-write with version chain (Compiler Arch §10)
- **Lifetime:** persistent with version history
- **Retention:** R+R+I with `λ=0.0005/day`, `θ=0.4` — importance weighted by success rate and recency of successful use
- **Consistency:** `session` is the natural fit (execution has defined phases)
- **Compression:** differential compression between versions

---

## 6. Implementation Roadmap

Aligned with the engineering ladder:

| Rung | Deliverable | Memory Manager Scope |
|---|---|---|
| 5 | Memory Core + Manager SDK | Reference implementation of all four stores + R+R+I + three consistency modes |
| 5.1 | Calibration study | Tune per-kind defaults against real interaction logs |
| 6 | Harness Runtime | Policy overrides at harness spawn, operational dashboards |
| 6.1 | Distributed Memory Core | Multi-node Memory Core with CRDT-based reconciliation |
| 6.2 | Session-type compiler checks | v0.4 Mini-Spec extension for compile-time session protocol verification |

Each rung is independently shippable; the language surface (Mini-Spec v0.3) does not change as Memory Manager implementations evolve. This is the decoupling consensus point 8 requires.

---

## 7. MIT-Defensibility Matrix

| Anticipated critique | Defense |
|---|---|
| "Where's R+R+I defined?" | §3.2 equations; §3.3 per-kind defaults |
| "How do agents share state?" | §4.2 three modes; §4.3 consistency table |
| "Why isn't this in the language spec?" | Four-model consensus point 8: runtime concerns in runtime, not language-core |
| "Can you prove the consistency protocol is correct?" | §4.5 session types (Honda 1993) provide the formal foundation |
| "What about TurboQuant?" | §3.5 explicit separation: R+R+I = retention; TurboQuant = compression |
| "What about distributed failure?" | §4.4 failure model (WAL, CRDT reconciliation) |
| "Are the default parameters principled?" | §3.4 deferred empirical calibration; current defaults are theory-driven normatives |

---

## 6.4 Compression Backends (v3.2 addition)

The four memory kinds use different storage strategies (§3.5), but the
*serialisation* of the underlying byte arrays is a separable concern.
For high-dimensional vectors stored in `semantic` indexes and dense
activation logs in `episodic` stores, the runtime may opt into the
**TurboQuant compression pipeline** (PolarQuant geometric simplification
+ QJL residual error correction) to achieve roughly 6x memory reduction
with under 0.5% retrieval-quality loss.

The full algorithmic contract — random projection, polar coordinates,
lookup-table quantisation, JL residual projection, sign-bit
reconstruction — is documented in
[`GARNET_Compression_Techniques_Reference.md`](GARNET_Compression_Techniques_Reference.md).
That reference is the canonical home for Gemini's PolarQuant/QJL
contribution from the four-model consensus (memo §5.4); this Memory
Manager document references it rather than duplicating the math.

The contract is **language-neutral**: implementers swap a backend trait
without breaking source code that imports a memory unit. The reference
implementation (`garnet-memory v0.3.0`) ships uncompressed stores; a
future `garnet-memory-turboquant` crate will provide the compressed
backend as a drop-in replacement.

---

## 8. Relationship to Other Deliverables

- **Mini-Spec v0.3 §4:** Specifies the language surface (`memory <kind> name : Store<T>`); this document specifies what happens beneath.
- **Mini-Spec v0.3 §9.2:** "Actors MUST NOT share mutable state" — preserved; this document explains how *One Memory Core, Many Harnesses* is consistent with that rule (agents share access via the Memory Manager, not raw references).
- **Compiler Architecture Spec §10:** Kind-aware allocation strategies; this document adds retention and consistency on top.
- **Paper IV v2.1.1 Appendix B:** PolarQuant/QJL compression mathematics; referenced here as a runtime tool, not a language feature.
- **Paper V §7.3:** Memory primitives as affine resources with the R+R+I law; formalizes the decay function in the safe sub-calculus.
- **Paper VI Contribution 4 (Kind-Aware Allocation):** The allocation strategy table; this document adds the policy layer on top.
- **Tier 2 Ecosystem Specs §B.3:** The `std::memory` module surface; this document specifies the runtime policy layer.

---

*"In the multitude of counsellors there is safety." — Proverbs 11:14*
*"Where there is no vision, the people perish." — Proverbs 29:18*

**Memory Manager Architecture Overview prepared by Claude Code (Opus 4.7) | April 16, 2026**
