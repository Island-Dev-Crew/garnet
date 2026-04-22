# Paper IV — Agentic Systems
## Addendum v1.0 (Phase 1D — Recursive Language Models + PolarQuant Bridge)

**Companion to:** `Paper_IV_Garnet_Agentic_Systems_v2_1_1.docx`
**Date:** April 16, 2026
**Author:** Claude Code (Opus 4.7) — Phase 1D
**Status:** Markdown extension to be folded into the next Paper IV .docx revision

---

## Purpose

The base Paper IV (.docx, v2.1.1) covers Garnet's agent-native memory
engineering: working / episodic / semantic / procedural memory cores
and how harness layers compose around them. Two pieces of material
that Gemini 3.1 Pro Deep Research contributed during the v2.1
four-model consensus did NOT make it into the .docx because Paper IV
froze before that synthesis pass:

1. **Recursive Language Models (RLMs)** — the paradigm Zhang, Kraska &
   Khattab introduced at MIT CSAIL (2025–2026) in which language
   models recursively invoke smaller language models on
   sub-problems, accumulating intermediate state that mirrors how
   compilers recursively process AST nodes.

2. **PolarQuant/QJL bridge** — an extended technical appendix that
   ties the compression reference (`GARNET_Compression_Techniques_Reference.md`)
   to Paper IV's memory-core narrative.

This addendum folds both into Paper IV without destabilising the
.docx structure. On the next Paper IV revision they will be promoted
into Appendix B (compression) and a new Appendix C (RLM).

---

## §A. Recursive Language Models (RLM) — the Gemini synthesis paradigm

### A.1 What RLMs are

A Recursive Language Model is an agent architecture in which a parent
LLM, when faced with a sub-problem it cannot solve in one forward pass,
*invokes a child LLM* (typically a smaller / cheaper / specialised
model) on the sub-problem and treats the child's output as a
black-box return value — the same way a function calls a sub-function.

The key insight Zhang et al. (CSAIL 2025) introduce is that the call
graph between parent and child models exhibits **the same structural
properties as a programming-language call graph**. Specifically:

- **Lexical scoping.** A child LLM's "scope" is the prompt the parent
  passes; it sees nothing outside that scope.
- **Return semantics.** A child returns a value (text, structured
  output, or a tool call) to the parent; control returns to the parent's
  cursor position.
- **Recursive depth.** A child MAY itself invoke a grandchild, with
  its own scope.
- **Memory partitioning.** Each level of the recursion has its own
  working memory; episodic memory (the call trace) is hierarchical.
- **Cost discipline.** The parent pays for the child's invocation; the
  child does not see the parent's accumulated context unless the parent
  explicitly forwards it.

This is *not* tool use in the OpenAI / Anthropic sense (which is a flat
loop of model → tool → model). RLM is a true call-graph recursion: the
child IS another model, with its own loop.

### A.2 Why RLM matters for Garnet

Garnet's Mini-Spec already encodes the language-side primitives that
RLM needs at the runtime layer:

| RLM property | Garnet primitive | Mini-Spec section |
|--------------|------------------|-------------------|
| Lexical scoping of child invocation | Closure capture rules | §5.3 (closures) + §5.4 (blocks) |
| Return semantics | Function return + `?` operator | §7.4 (boundary bridging) |
| Recursive depth bound | `@max_depth(N)` annotation | §10.1 |
| Memory partitioning | Per-actor memory units | §9.3 |
| Cost discipline | `@fan_out(K)` annotation + protocol declarations | §10.2 + §9.4 |

The four-model consensus point #4 ("agent-native language platform")
is materially advanced by recognising RLM as the *programming model*
that Garnet's primitives are designed for. Until this addendum the
connection was implicit; from v3.3 forward it is explicit.

### A.3 The Garnet ↔ RLM correspondence (formal)

For every RLM call graph G consisting of model-invocation nodes
{M₁, M₂, …, Mₙ} with parent→child edges {(Mᵢ, Mⱼ) | Mⱼ is invoked by Mᵢ},
the Garnet program that orchestrates G has:

- One actor per Mᵢ (per Mini-Spec §9 typed-actor protocol contract)
- One protocol declaration per (parent, child) edge specifying the
  request and response message types
- A `@max_depth(d)` annotation on the entry actor where d = depth(G)
- `@fan_out(k)` on every parent that issues > 1 simultaneous child
  invocation
- Memory units of kind `episodic` to record the call trace, of kind
  `working` to hold per-invocation transient state, and of kind
  `semantic` to hold the cross-invocation learned facts

This is not a mapping the user has to construct manually — it is a
*compiled* mapping. The proposed v4.x feature `garnet rlm scaffold
<call-graph.json>` would generate the actor skeleton from a call-graph
description.

### A.4 Where RLM intersects Paper VI Contribution 3 (Compiler-as-Agent)

Paper VI C3 says the Garnet *compiler* uses Garnet's own four memory
kinds to learn from compilation history. RLM says *agents in general*
do the same at the model layer. The two claims are independent but
mutually reinforcing:

- A compiler-as-agent IS an RLM whose parent model is the optimiser
  and whose children are individual pass invocations
- An RLM benefits from kind-aware memory allocation (Paper VI C4)
  because parent-level episodic memory has a different access pattern
  (sequential write, range read) than child-level working memory
  (high write, short-lived) — exactly the pattern Mini-Spec §4.5
  partitions allocator strategies for.

So the v4.0 empirical experiments (Phase 1C deliverable) implicitly
test RLM-relevant behaviour as a side effect of testing C3 and C4.

### A.5 Open questions on RLM

1. **Determinism across child invocations.** If parent → child → parent
   is deterministic per-invocation but children are LLM samples with
   temperature > 0, the call graph is not deterministic. Garnet's
   Paper VI C7 (deterministic builds) does NOT apply at the agent
   layer; we should be honest that runtime non-determinism is part of
   the RLM contract.

2. **Cost accounting.** Parent's billing includes all transitive
   children. The harness layer (Paper IV §5–§7) needs primitives to
   track this; current `episodic` memory captures it but lacks a
   first-class API.

3. **Child timeout / partial result.** What does a parent see if a
   child times out mid-call? RLM literature is divided; Garnet should
   adopt the v0.4 actor-protocol-versioning model (Mini-Spec §9.2 OQ-5)
   to carry timeout semantics into the protocol type.

These are deferred to the v4.x agent-runtime work, not v3.3.

---

## §B. PolarQuant ↔ Memory Core bridge (technical appendix)

### B.1 Where compression sits in Paper IV's narrative

Paper IV §3 introduces the four memory cores. §6 covers the harness
layer. Compression was implicitly present (TurboQuant referenced as
runtime-relevant) but never given a section. This addendum closes
that gap.

### B.2 The Memory-Core compression contract

A Garnet memory unit declared `memory semantic K : VectorIndex<T>`
exposes a *uniform* interface to the program (insert, search,
remove, iterate). The implementation behind that interface MAY:

- Store vectors uncompressed (default, `garnet-memory v0.3.0`)
- Store vectors PolarQuant-compressed (planned `garnet-memory-turboquant`
  crate; tracked in Memory Manager Architecture §6.4)
- Use a hybrid (e.g., recent vectors uncompressed in working set,
  older vectors compressed)

The choice is invisible to the user program. This is the architectural
discipline mandated by four-model consensus point 8 (TurboQuant =
runtime, not language core).

### B.3 Compression decision per memory kind

| Kind | Default compression | Justification |
|------|---------------------|---------------|
| working | None | Short-lived; compression overhead exceeds savings |
| episodic | Optional dictionary compression on payload field | Sequential writes, range reads, payloads are repetitive |
| semantic | PolarQuant + QJL strongly recommended | Vector fields are large and access pattern (top-k similarity) tolerates lossy compression |
| procedural | Differential compression between versions | Versioned writes, sparse mutation |

Per-kind defaults are codified in Memory Manager Architecture §6.4
and can be overridden per-deployment via the runtime config (not the
language).

### B.4 The bridge to Paper VI Contribution 4

Paper VI C4 (kind-aware memory allocation) and the compression
contract are orthogonal but compose: the kind-aware allocator selects
*where in memory* the vectors live (arena, append log, persistent
node, COW allocator); the compression contract selects *how the bytes
are laid out within that memory*. Both are runtime concerns, both
derive from the memory-kind annotation in the source.

A future research contribution (post-MIT) is the *kind-aware
compression-aware allocator*: a single allocator strategy per kind
that already understands the compression scheme appropriate for that
kind. Today this is a layered design (allocator wraps compression);
fusing the two is a v4.x optimisation.

### B.5 Cross-references

- `GARNET_Compression_Techniques_Reference.md` v0.4 (PolarQuant + QJL
  full derivations + SRHT for non-power-of-2 dim + α calibration +
  re-seed schedule)
- Memory Manager Architecture §3.5, §6.4, §10.7
- Paper VI Contribution 4 (kind-aware allocation)
- Mini-Spec v1.0 §4.5 (cycle detection — orthogonal to compression but
  uses the same per-kind partitioning idea)

---

## §C. What this Addendum does NOT cover

Out of scope for Phase 1D:

- **Implementation of `garnet-memory-turboquant`.** Tracked as a v4.x
  research crate; v3.4 ships the `VectorIndex` interface, v4.x ships
  the compressed backend.
- **Integration of RLM into the v3.4 stdlib.** RLM is a research
  paradigm, not a stdlib feature. The relevant Mini-Spec primitives
  (closures, actors, `@max_depth`, `@fan_out`) are already present;
  the runtime that orchestrates RLM call graphs is a v4.x research
  artifact.
- **Empirical evaluation of RLM-on-Garnet.** Defer to the same Phase
  4A experimental window that runs the seven Paper VI experiments
  (Phase 1C deliverable).

These omissions are documented as open questions in §A.5 and §B —
NOT papered over. Reviewers reading both Paper IV (.docx) and this
addendum should have a complete picture of where Garnet's
agent-runtime story sits at v3.3 end-of-Stage-1.

---

## §D. Promotion path

This addendum is the canonical Paper IV companion from Phase 1D
forward. On the next Paper IV .docx revision (planned v3.0 or v4.0),
the contents of §A through §C will be folded in directly:

- §A → Appendix C (Recursive Language Models)
- §B → Appendix B (Compression Bridge), absorbing the cross-references
  to the standalone compression reference
- §C → footnote on the addendum origin

Until that revision, the .docx + this markdown together constitute
Paper IV's normative content; in case of conflict, this addendum
supersedes (because it has been reviewed in conjunction with the
Phase 1B Mini-Spec v1.0 promotion).

---

## §E. References (for the addendum specifically)

1. Zhang, Z., Kraska, T., Khattab, O. "Recursive Language Models." MIT
   CSAIL technical report, 2025–2026.
2. Tropp, J.A. "Improved Analysis of the Subsampled Randomized Hadamard
   Transform." Adv. Adaptive Data Analysis, 2011 — for the SRHT bound
   used at Compression Reference §8.1.
3. Bingham, E., Mannila, H. "Random Projection in Dimensionality
   Reduction." KDD 2001 — for the JL constant derivation.
4. `GARNET_v2_1_Gemini_Synthesis.md` — the original Gemini contribution
   that introduced RLM into the Garnet conversation.
5. `Garnet_ Agent-Native Language Synthesis.docx` — Gemini's full
   deep-research treatment, of which this addendum is the
   markdown-canonical extraction.

---

*Prepared 2026-04-16 by Claude Code (Opus 4.7) — Phase 1D Paper IV
Addendum. RLM paradigm + PolarQuant bridge consolidation.*

*"Give instruction to a wise man, and he will be yet wiser." — Proverbs 9:9*
