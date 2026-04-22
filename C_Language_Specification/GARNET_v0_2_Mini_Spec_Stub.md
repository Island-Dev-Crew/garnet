# Garnet v0.2 Mini-Spec Stub

**Supersedes:** v0.1 Mini-Spec Stub (earlier April 12, 2026)
**Status:** Internal draft, open for revision
**Date:** April 12, 2026
**Changes in v0.2:** Affine type theory + RustBelt grounding (§3), new §5 on recursive execution guardrails, new OQ-7 on Memory Manager decay formula. All v0.1 content preserved.

> This document is a stub, not a specification. Normative terms follow RFC 2119 (MUST, SHOULD, MAY).

## 1. Scope

Unchanged from v0.1: (1) memory unit declarations, (2) `@safe` mode boundaries, (3) mode-crossing rules, (4) typed actor protocols. **New in v0.2:** (5) recursive execution guardrails.

## 2. Memory Units

### 2.1 Declaration form

```
memory-decl  := "memory" memory-kind ident ":" store-type
memory-kind  := "working" | "episodic" | "semantic" | "procedural"
store-type   := ident ("<" type-args ">")?
```

### 2.2 Semantics

A memory unit declaration MUST introduce a top-level binding whose static type is `store-type` and whose runtime identity is unique within the enclosing module. Two memory units of the same kind and ident in the same module MUST be a compile-time error. The four memory kinds are type-level tags, not independent types.

### 2.3 Out of scope

Retention policies, TTL semantics, ranking functions, privacy scopes, persistence guarantees, wire formats — all belong to the Memory Manager and Memory Core, not the language core.

## 3. Mode Boundaries

### 3.1 Formal grounding (new in v0.2)

Garnet's `@safe` mode is grounded in **affine type theory**, the substructural branch in which a resource may be used at most once. This is the same mathematical foundation underlying Rust's ownership discipline, which was formally verified by the **RustBelt project (Jung et al., POPL 2018, MPI-SWS)** using the **Iris framework for higher-order concurrent separation logic in Coq**. Garnet does not invent a new memory model in safe mode; it re-exports a proven one at module granularity rather than crate granularity. Any future soundness proof obligation for Garnet's boundary rules (§3.4) SHOULD reference the RustBelt methodology as the baseline.

### 3.2 Managed mode (default)

Managed mode MUST provide automatic reference counting with cycle detection for all reference types. Values MAY be mutated without ownership discipline. Managed mode is the default.

### 3.3 Safe mode

A module MAY be annotated `@safe`. A `@safe` module MUST enforce at compile time:
1. Single-owner (affine) semantics for all values declared within the module.
2. Aliasing-XOR-mutation for all references (`&T` or `&mut T`, not both).
3. Lexical or non-lexical lifetime inference on all references.
4. No implicit allocation on any path marked as a hot path.

### 3.4 Boundary rules

Unchanged from v0.1: (1) managed callers invoking `@safe` functions MUST satisfy ownership preconditions at call sites; (2) `@safe` returns to managed mode MUST either return an owned value adopted into ARC or a borrowed reference with a statically proven lifetime; (3) managed memory units MAY be read from safe mode under shared borrow, but MUST NOT be mutated except through a formal bridging API reserved for a later version.

## 4. Typed Message Protocols

### 4.1 Grammar

```
actor-decl    := "actor" ident "{" protocol-decl* handler-decl* "}"
protocol-decl := "protocol" ident "(" param-list ")" ("->" type)?
handler-decl  := "on" ident "(" param-list ")" block
```

### 4.2 Semantics

Actors MUST NOT share mutable state. All inter-actor communication MUST go through declared protocols. Undeclared protocol sends MUST be compile-time errors. Every declared protocol MUST have a handler. Type erasure and gradual typing are prohibited across actor boundaries even in managed mode.

## 5. Recursive Execution Guardrails (NEW in v0.2)

Garnet supports the Recursive Language Model pattern (Zhang, Kraska, Khattab — MIT CSAIL, 2025–2026) in which long contexts are loaded as persistent objects in sandboxed REPL environments and the model recursively spawns sub-instances to process localized chunks. Unbounded recursion introduces catastrophic cost and latency risks, so Garnet's compiler and runtime MUST collaboratively enforce:

### 5.1 Recursion depth limits (MUST)

Every recursive agent spawn site MUST be annotated with a static maximum recursion depth. The compiler MUST reject programs in which a `recurse` invocation could exceed its annotated depth along any reachable execution path. Default depth if unannotated is 1 (single layer of delegation).

### 5.2 Asynchronous fan-out caps (MUST)

Every parallel sub-agent spawn site MUST declare a maximum fan-out width. The runtime MUST reject attempts to exceed that width at execution time. This rule exists to make rate-limit exhaustion and runaway cost structurally impossible rather than merely discouraged.

### 5.3 Metadata validation (MUST)

All memory units passed as arguments to a recursive sub-agent MUST carry metadata sufficient for the Memory Manager to statically determine whether the recursive retrieval is justified compared to a simple semantic baseline search. The Memory Manager MAY refuse to route a recursive call whose metadata indicates that a flat retrieval would suffice. This rule exists to prevent agents from pathologically preferring expensive recursive paths when cheap ones would serve.

### 5.4 Scope

§5 constrains program structure, not program semantics. A compliant Garnet implementation MAY choose any enforcement strategy (static analysis, runtime checks, or both) that rejects non-compliant programs with clear diagnostics.

## 6. Open Questions

- **OQ-1.** How are memory-unit retention policies expressed in source?
- **OQ-2.** What is the bridging API for managed→safe mutation of a managed memory unit?
- **OQ-3.** What is the story for generics over memory kinds?
- **OQ-4.** What is the soundness proof obligation for §3.4 boundary rules? (RustBelt methodology SHOULD be the baseline.)
- **OQ-5.** How are actor protocols versioned across a running system?
- **OQ-6.** What, if anything, does the language surface expose about KV-cache compression hints? (Current position: nothing.)
- **OQ-7.** (NEW) How is the Memory Manager's controlled-decay formula expressed and tuned? Gemini's fourth-model review surfaced the Richmond Alake formulation of Relevance + Recency + Importance as the weighting function for memory unit retention ranking. Garnet SHOULD treat this as the default heuristic in reference Memory Manager implementations, but MUST allow tuning per memory kind and per domain harness. The exact source syntax for expressing decay weights is deliberately left unresolved in v0.2.
- **OQ-8.** (NEW) How does multi-agent access to a shared Memory Core maintain consistency under concurrent writes? Gemini flagged this as a live challenge introduced by the "one memory core, many harnesses" discipline; a future v0.3 stub SHOULD propose a shared-substrate protocol.

## 7. What a v0.2 implementation owes

Rungs 1–4 of the engineering ladder MUST be implementable against this stub. Specifically, a v0.2-compliant parser and interpreter MUST parse §2.1, §4.1, and §5.1–§5.3 grammars; MUST enforce §3.3 and §3.4 boundary rules; MUST reject programs violating §5 guardrails. Performance, codegen, real memory-core backing, and decay-formula tuning are OUT of scope for v0.2.

---

**Status:** v0.2 stub. Next artifact in the ladder: parser + AST against this grammar.
