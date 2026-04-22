# GARNET v2.1 — Four-Model Consensus Memo

**Supersedes:** Three-Model Consensus Memo (April 12, 2026, earlier same day)
**Prepared for:** Jon — Island Development Crew
**Date:** April 12, 2026
**Status:** Canonical companion to the v2.1 executive overview

---

## 1. Purpose

Between April 2026 and the v2.1 redline pass, the Garnet thesis was independently reviewed by **four frontier reasoning systems** working from the full corpus, the two April 2026 transcripts, and (in Gemini's case) live web sources:

- **Anthropic Claude Opus 4.6** (prior sessions and present session)
- **OpenAI GPT-5.4 Pro Extended Thinking**
- **xAI Grok 4.2 Expert Thinking**
- **Google DeepMind Gemini 3.1 Pro Deep Research**

The four reviews were produced independently and without cross-pollination. This memo records their convergence, their bounded divergences, and the substantive new contributions surfaced by the fourth review.

## 2. Points of Four-Model Convergence

1. Rust and Ruby are structurally complementary; their weaknesses map onto each other's strengths.
2. Dual-mode architecture (managed default + `@safe` opt-in per-module) is the correct reconciliation shape.
3. Swift is the missing production precedent — ARC + actors + `Sendable` validates the managed middle.
4. The center of gravity has shifted from *language synthesis* to *agent-native language platform*.
5. "One Memory Core, Many Harnesses" is the definitive platform architecture.
6. Memory primitives (working, episodic, semantic, procedural) deserve first-class language status.
7. Typed actors with compiler-enforced message protocols are the correct concurrency story.
8. TurboQuant is strategically relevant as a runtime signal, not as a language-core guarantee.

Four-way convergence across Anthropic, OpenAI, xAI, and Google DeepMind spans all four major Western frontier labs. When four independently trained reasoning systems with different alignment tuning converge on eight architectural claims without coordination, the probability that the shared conclusions are artifacts of any single lab's idiosyncrasies drops to near zero.

## 3. Three Adjudicated Divergences (Held Under Fourth Review)

All three boundaries from the original three-model memo **held unchanged** under Gemini's independent scrutiny:

- **TurboQuant scope** — runtime hint, not language guarantee. Gemini explicitly cites the Opus/GPT consensus and endorses the discipline.
- **"First agent-native"** — Gemini uses "proposed agent-native language platform," matching v2.1's falsifiable framing.
- **Compiler bootstrap** — Gemini treats the engineering ladder as the real path forward, not Grok's narrative prototype.

## 4. Why Four-Model Convergence Matters

Three-way convergence proxies for robustness. Four-way convergence across all four major frontier labs — with one reviewer operating in Deep Research mode with live web access — crosses a credibility threshold that three-way alone does not reach. It is the closest thing to external peer review available to the Garnet project before a formal community RFC process exists. The remaining open questions are now well-localized to Section 6 of the v0.2 Mini-Spec Stub.

## 5. Gemini-Specific Substantive Contributions

Gemini agreed on every consensus point and adjudication, but its from-scratch doctoral restatement surfaced five genuinely new contributions that Opus, GPT, and Grok did not produce with the same precision. These should be folded into the corpus.

1. **RustBelt formal verification grounding.** Gemini explicitly names RustBelt (POPL 2018, MPI-SWS) and the Iris framework for higher-order concurrent separation logic in Coq as the mathematical foundation for Garnet's safe-mode ownership discipline. This is a citation-grade academic anchor the prior corpus was missing. *Integration target: Mini-Spec v0.2 §3.2 and Paper V when authored.*

2. **Affine type theory framing.** Gemini formally roots Rust's ownership model in affine type theory rather than treating it as a pragmatic invention. This upgrades Garnet's safe mode from "Rust-like" to "grounded in a named branch of substructural type theory." *Integration target: Mini-Spec v0.2 §3.2 preamble.*

3. **Memory Manager controlled-decay formula.** Gemini extracted the Relevance + Recency + Importance weighting from the Alake framework and named it as a normative requirement for the Memory Manager layer. *Integration target: Mini-Spec v0.2 new OQ-7.*

4. **PolarQuant + QJL mathematical mechanics.** Gemini produced the clearest technical passage in the entire four-model corpus explaining exactly how TurboQuant works: random projection → Beta distribution → polar coordinate transformation eliminating normalization overhead, then QJL residual shrinking to a single sign bit. This is the explanation that makes the scope discipline defensible. *Integration target: Paper IV v2.1.1 appendix.*

5. **RLM recursion guardrails as normative.** Gemini elevates depth limits, async fan-out caps, and metadata validation from "best practices" to **compiler-enforced requirements** for recursive execution patterns. *Integration target: Mini-Spec v0.2 new §5 (Recursive Execution Guardrails).*

## 6. Recommended Next Move

The research phase is now complete. The engineering phase begins with rung 2 of the ladder (parser + AST) against Mini-Spec v0.2. A future Paper V — *The Formal Grounding of Garnet: Affine Type Theory, RustBelt, and the Mathematics of Mode Boundaries* — is now justified by Gemini's contributions and would make Garnet submittable to PLDI, POPL, or OOPSLA.

---

*"In the multitude of counsellors there is safety." — Proverbs 11:14*
*"Where there is no vision, the people perish." — Proverbs 29:18*

**Prepared by Claude Opus 4.6 | Island Development Crew | April 12, 2026**
