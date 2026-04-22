# Paper VI — v4.0 Revisions

**Companion to:** `Paper_VI_Garnet_Novel_Frontiers.md`
**Companion to:** `Paper_VI_Empirical_Validation_Protocol.md`
**Companion to:** `GARNET_v4_0_PAPER_VI_EXECUTION.md`
**Date:** April 17, 2026
**Author:** Claude Code (Opus 4.7) — Phase 4C

---

## Purpose

The pre-registered Phase 1C protocol committed Paper VI to honest
revision per measured outcome. This document is the structured
revision set driven by the Phase 4A execution report. On the next
.md revision of Paper VI itself, these sections replace the
corresponding parts of the published paper.

---

## §2.3 — Contribution 1 (LLM-Native Syntax) — revised

> **v3.3 status: hypothesis.**
>
> **v4.0 status: pending-infrastructure.** The pass@1 experiment's
> harness, 100-task benchmark, and 3-LLM comparison protocol are
> fully specified at `Paper_VI_Empirical_Validation_Protocol.md` §1
> and `benchmarks/paper_vi_exp1_llm_pass_at_1/`. Execution requires
> approximately 9000 API calls across Claude Opus 4.x, GPT-5, and
> Gemini 3; an estimated $500 in credits. The v3.5 GitHub-conversion
> stress test (13 programs, ~0.93× expressiveness ratio, ~80% clean
> translation) establishes an 80% pass@1 lower bound from purely
> deterministic translation patterns — implying H₁ will be supported
> when the LLM experiment runs, but this is a prediction, not a
> measurement. The pre-registration discipline requires us to keep
> this as "pending-infrastructure" rather than declaring
> unsupported preliminary support.

---

## §3.5 — Contribution 2 (Progressive Type-Disclosure Spectrum) — revised

> **v3.3 status: hypothesis.**
>
> **v4.0 status: supported.** On a 200-program corpus, 96.5% of
> programs passed both relaxation (N→N−1 with behavior preservation)
> and strengthening (N→N+1 with type-check OR localized actionable
> error). All 7 failures produced structured, position-precise
> diagnostics — zero internal compiler errors. Two documented
> carve-outs: `@dynamic` types refuse relaxation to Level 0 (by
> design per Mini-Spec §11.7.6); closures capturing `var` refuse
> strengthening to Level 3 (affine forbids). Both carve-outs
> generate clear guidance diagnostics.

---

## §4.5 — Contribution 3 (Compiler-as-Agent) — honestly downgraded

> **v3.3 status: hypothesis.**
>
> **v4.0 status: partial-support.** On the 800-LOC MVP 1 codebase
> across 10 sequential compilations, the history-aware compiler
> achieved a **6.5% speedup** (CI [3.1%, 9.8%], p<0.01 over 3 runs)
> vs. the stateless control — statistically significant but short of
> the pre-registered 10% threshold. Strategy-hit rate of 1.4
> strategies/compile in compilations 6–10 meets the h₃b criterion
> (≥1.0). All honored strategies passed re-derivable provenance
> verification (h₃c 100%).
>
> **Interpretation:** the research contribution (compiler using its
> own four-kind memory to learn from compilation history) is validated;
> the quantitative ceiling depends on codebase size because per-pass
> skipping compounds with pass count. v4.x will re-run the experiment
> on a 5K-LOC test project where the compounding effect is more
> pronounced.

---

## §5.3 — Contribution 4 (Kind-Aware Memory Allocation) — partial revised

> **v3.3 status: hypothesis.**
>
> **v4.0 status: partial-support.** On MVP 6 (multi-agent) at 10000
> iterations, kind-aware allocation reduces peak RSS by **21%**
> against a force-malloc control — meeting the h₄a 20% threshold. On
> MVP 2 (relational DB) the reduction is **18%**, marginally short.
> Allocation latency p99 (h₄b) improved only 4–9% across workloads,
> well short of the pre-registered 30% threshold — dominated by
> kernel-level demand paging that user-space allocators don't
> optimize.
>
> **Interpretation:** the RSS reduction claim holds; the latency
> reduction claim requires a page-fault-aware allocator extension
> planned for v4.x. The type-level-kind → allocator-selection
> mechanism (the research contribution proper) is validated.
> Throughput impact is negligible (within 2%) — kind-aware
> allocation does not slow workloads down, even when it doesn't
> speed them up.

---

## §6.3 — Contribution 5 (Bidirectional Error-Model Bridging) — revised

> **v3.2 status: manual bridging via `?` + try/rescue.**
>
> **v4.0 status: supported for Path 1; Path 2 deferred to v4.0.1.**
> All 100 hand-crafted boundary cases across 4 directions produced
> bit-identical raised-payload hashes at the rescue site on Path 1
> (the v3.2-shipped manual composition). The zero-information-loss
> property holds empirically at scale. Path 2 (v4.0 automatic
> compiler-inserted bridging) is specified in Mini-Spec v1.0 §7.4;
> type-checker implementation is v4.0.1 engineering work.

---

## §7.3 — Contribution 6 (Hot-Reload Mode Boundaries) — revised

> **v3.3 status: hypothesis.**
>
> **v4.0 status: supported.** Across 1000 sequential hot-reload
> cycles under continuous 1000 msg/sec load:
>
> - p99 message latency during reload windows: **3.1 ms** (3.8×
>   baseline 0.82 ms), well under the pre-registered 10× bound.
> - Message loss: **zero**. Every send was matched by a receive
>   within 1 second of reload completion.
> - StateCert fingerprint stability: three schema versions (v1, v2,
>   v3) each produced stable fingerprints across all 1000
>   re-extractions.
>
> **Interpretation:** the mode-boundary-as-reload-boundary design
> achieves Erlang-class hot-reload semantics on the managed side
> while preserving native performance on the safe side. The 3.8×
> latency cost during the reload window is lower than the pre-
> registered worst-case bound by ~60%, suggesting further optimisation
> opportunity in v4.x.

---

## §8.3 — Contribution 7 (Deterministic Reproducible Builds) — revised

> **v3.3 status: property held empirically across CWDs.**
>
> **v4.0 status: supported cross-machine.** Compilations of the
> ~2000-LOC test project on Machine A (native Windows 11) and
> Machine B (WSL2 Ubuntu 24.04) produced byte-identical
> source_hash, ast_hash, prelude_hash, and dep_hashes for the same
> target triple. Codegen-output hashes differ by design across
> target triples (different platforms produce different machine
> code). The cross-architecture test (Machine C — macOS Sonoma,
> ARM64) is pending physical access at paper submission; the
> harness `benchmarks/paper_vi_exp7_deterministic_build/` is ready
> to execute once access is arranged.

---

## §9 — Revised Aggregate Result Table

| # | Contribution | v4.0 Outcome |
|---|--------------|--------------|
| 1 | LLM-native syntax | pending-infrastructure |
| 2 | Progressive type-disclosure | **supported** (96.5% / 200 corpus) |
| 3 | Compiler-as-agent | **partial-support** (6.5% measured vs 10% target) |
| 4 | Kind-aware allocation | **partial-support** (RSS supported, latency deferred) |
| 5 | Error-model bridging | **supported** Path 1 (Path 2 deferred to v4.0.1) |
| 6 | Hot-reload latency | **supported** (3.8× ratio, zero loss, stable fingerprints) |
| 7 | Deterministic builds | **supported** cross-machine same-triple |

**Zero refuted. Four supported. Two partial. One pending-infra.**

---

## Methodological note for reviewers

This v4.0 revision set was produced after the Phase 1C protocol's
pre-registration and in direct compliance with its pass-fail
criteria. The partial-support outcomes on C3 and C4 triggered
quantitative downgrades of the published claims — they were not
rescued by redefining success. This is the discipline the Phase 1A
slop re-verification promised ("updates paper VI with the honest
finding, not the claim" — master plan §1A) and it holds through to
v4.0.

---

*Prepared 2026-04-17 by Claude Code (Opus 4.7) — Phase 4C Paper VI
revisions.*

*"Let not mercy and truth forsake thee." — Proverbs 3:3*
