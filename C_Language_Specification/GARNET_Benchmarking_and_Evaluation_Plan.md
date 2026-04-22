# Garnet Benchmarking & Evaluation Plan
**Version:** 1.0
**Date:** April 16, 2026
**Companion to:** Paper VI (Novel Frontiers), Compiler Architecture Spec §12
**Anchor:** *"Where there is no vision, the people perish." — Proverbs 29:18*

---

## 1. Purpose

This document specifies the experimental protocol for evaluating each of Paper VI's seven falsifiable hypotheses. Anticipating MIT defense and PLDI review, every novel claim has a corresponding benchmark, a baseline to compare against, and a success criterion.

Where data is pending (most measurements await Rung 3+ implementation), this plan commits to the methodology so that results produced later can be evaluated rigorously.

---

## 2. Evaluation Axes

Five measurement dimensions:

1. **Compilation performance** — lex/parse/typecheck/codegen throughput
2. **Runtime performance** — execution speed vs. Rust (safe mode) and Ruby (managed mode)
3. **LLM code-generation quality** — pass@1 correctness on a standardized benchmark (Paper VI §2.3)
4. **Memory performance** — peak allocation vs. general-purpose allocator (Paper VI §5.3)
5. **Development ergonomics** — time-to-first-correct-program across type-spectrum levels (Paper VI §3.4)

Each axis has a primary metric, one or more secondary metrics, and explicit falsification conditions.

---

## 3. Compilation Performance Benchmarks

**Baseline:** targets from Compiler Architecture Spec §12.

| Benchmark | Workload | Target | Falsification |
|---|---|---|---|
| Lex throughput | 100K-line synthetic corpus | <50ms | >150ms |
| Parse throughput | same corpus | <100ms | >300ms |
| Type-check managed | same corpus, Level 0 | <500ms | >2s |
| Type-check safe | same corpus, Level 3 | <2s | >8s |
| Full debug compile | 10K-line project | <1s | >5s |
| Full release compile | 10K-line project | <5s | >20s |
| REPL cold start | `echo 1 + 1 | garnet repl --batch` | <100ms | >500ms |
| Incremental rebuild | 1-file change in 10K project | <200ms | >1s |

**Method.** Write a synthetic corpus generator (parameterized by KLOC, function density, generic density). Run each benchmark 10 times, record median + 95th-percentile. Compare against targets.

**Comparison.** The 10K-line `garnet build --release` time will be compared against Rust 1.94.1 on functionally equivalent code (hand-ported). Target: match Rust within 1.5x.

---

## 4. Runtime Performance Benchmarks

### 4.1 Managed mode vs. Ruby 3.4

**Baseline.** Ruby 3.4 with YJIT enabled (the strongest Ruby JIT as of April 2026).

**Benchmark suite.** 100 tasks adapted from:
- The Ruby Benchmarks Suite (a community-standard set of algorithmic tasks)
- A subset of The Computer Language Benchmarks Game: n-body, spectral-norm, fasta, mandelbrot, pidigits, binary-trees, fannkuch-redux
- Five agent-workflow microbenchmarks (designed for this plan): message-pass, memory-retrieve, pipeline-transform, pattern-match, recursive-spawn

**Targets.**
- 80th-percentile task: within 2x of Ruby 3.4 + YJIT
- Worst-percentile task: within 5x of Ruby 3.4 + YJIT
- Best-percentile task: faster than Ruby 3.4 + YJIT

**Rationale.** Ruby 3.4 + YJIT is a rapidly moving target; matching it within 2x while offering Rust-level safety via `@safe` mode is the defensible equilibrium.

### 4.2 Safe mode vs. Rust 1.94

**Baseline.** Rust 1.94 with `--release` and default optimization level (O3).

**Benchmark suite.** The same benchmark tasks, ported to idiomatic Rust and idiomatic safe-mode Garnet.

**Targets.**
- 80th-percentile task: within 1.2x of Rust
- Worst-percentile task: within 2x of Rust
- Best-percentile task: matches Rust

**Rationale.** Paper V §6 argues that λ_safe is essentially Rust at module granularity. Performance should follow. 1.2x overhead accounts for the Garnet-specific boundary-validator code (not eliminable).

### 4.3 Kind-Aware Allocation (Paper VI Contribution 4)

**Hypothesis (Paper VI §5.3).** Kind-aware allocation reduces peak memory usage by at least 20% vs. general-purpose allocator on agent workloads.

**Method.**
1. Implement a reference workload: a build-agent processing 10,000 small build events with `memory episodic` + `memory procedural` usage.
2. Measure peak RSS with:
   - (A) General-purpose allocator (jemalloc or mimalloc)
   - (B) Kind-aware allocators (arena for working, log for episodic, persistent for semantic, COW for procedural)
3. Report: peak RSS, allocation call count, avg allocation latency.

**Falsification:** if kind-aware does not reduce peak RSS by ≥20% across at least three distinct workloads, the hypothesis is falsified and Paper VI §5.3 must be revised.

### 4.4 Async/Actor Runtime

**Benchmarks.**
- Task spawn overhead (time between `spawn f()` and `f` beginning execution)
- Context switch latency (time between yield and resume)
- Mailbox throughput (messages/second in a ping-pong actor pair)

**Targets.**
- Task spawn: <500ns
- Context switch: <200ns
- Mailbox throughput: >10M msg/sec (single-pair, same machine)

These targets are comparable to Go goroutines (Google measurements) and Erlang processes (EFL team measurements).

---

## 5. LLM Code-Generation Benchmark (Paper VI Contribution 1)

### 5.1 Hypothesis (Paper VI §2.3)

An LLM fine-tuned on Garnet code achieves higher pass@1 than the same LLM fine-tuned on equivalent Rust code for programs of comparable complexity.

### 5.2 Benchmark design

**Scale.** 500 programming tasks total:
- 200 managed-mode (Levels 0–2): agent orchestration, API clients, data transforms, REPL commands
- 200 safe-mode (Level 3): data pipelines, concurrency primitives, cryptographic helpers, zero-copy parsers
- 100 mixed-mode: systems that use `@safe` hot paths inside managed orchestration

**Difficulty distribution.** Modeled on SWE-bench:
- Easy (40%): 20–50 line solutions, single-function
- Medium (40%): 50–200 line solutions, multi-function or single-module
- Hard (20%): 200+ line solutions, multi-module, requires design decisions

**Task format.** Each task has:
- A natural-language prompt
- A public specification (data shapes, contract)
- A hidden test suite (25–100 test cases per task)
- A reference solution (for inter-rater agreement + ceiling)

### 5.3 Method

**Models.** Four frontier LLMs to match the four-model consensus: Claude Opus 4.7 (latest), GPT-5.4 Pro, Grok 4.2 Expert, Gemini 3.1 Pro.

**Languages.** Garnet, Rust, Ruby (for task applicability).

**Metric.** pass@1 = fraction of tasks where the first generated solution compiles AND passes all hidden tests.

**Protocol.**
1. For each (model, language, task): generate a solution with temperature 0, no context hints beyond the public spec.
2. Compile. If compile fails, record pass@1 = 0 for that task and note the failure mode.
3. Run hidden tests. pass@1 = 1 iff all tests pass.
4. Repeat 3 times with temperature 0.3 to measure sensitivity.

**Success criterion for Paper VI §2.3.** Garnet pass@1 > Rust pass@1 across all 4 models at p < 0.05 significance (paired t-test across tasks).

### 5.4 Threats to validity

- **Bias:** models may have seen Garnet training data but not Rust — mitigated by using only public task specs, not private tests, during generation.
- **Selection bias:** tasks hand-selected by the paper authors — mitigated by open-sourcing the full task set and accepting community contributions.
- **Over-specification:** overly specified tasks reduce generative variance — balanced by the difficulty distribution (20% hard tasks allow design freedom).

---

## 6. Development Ergonomics (Paper VI Contribution 2)

### 6.1 Hypothesis (Paper VI §3.4)

Developers who start at Level 0 and incrementally add annotations achieve comparable safety outcomes to those starting at Level 3, with lower time-to-first-correct-program.

### 6.2 Method

**Participants.** 30 developers, stratified by experience: 10 with Rust exposure, 10 with Ruby exposure, 10 with neither.

**Task.** Build a small CLI tool (a log-analysis command with file I/O, regex matching, and a structured output format) from a natural-language spec.

**Two arms.**
- Arm A: Start at Level 0, iterate to Level 2 with optional type hints as bugs surface.
- Arm B: Start at Level 3 (safe mode), enforce full type + ownership discipline from line one.

**Measures.**
- Time-to-first-run (first compile success)
- Time-to-first-correct (passes all 10 integration tests)
- Bug density (bugs per 100 lines) during development
- Final safety (memory/concurrency bugs in the final version)

**Success criterion.** Arm A achieves time-to-first-correct within 30% of Arm B's, with Arm A's final-safety bug count ≤ 2x Arm B's.

### 6.3 Scale and timeline

Pilot study (5 developers) before Rung 3 completion; full study (30 developers) after Rung 3 ships with a usable REPL and LSP.

---

## 7. Compiler-as-Agent (Paper VI Contribution 3)

### 7.1 Hypothesis (Paper VI §4.4)

After 10+ compilations of the same codebase, a history-aware Garnet compiler produces better-optimized output than a stateless compiler.

### 7.2 Method

**Corpus.** Three medium-sized Garnet projects (~10K LOC each), compiled on a dedicated machine over 14 consecutive days with realistic edit patterns (simulated from real Git histories of Rust projects).

**Measures, per compilation N:**
- Compilation time (total, and per-pass)
- Output binary size
- Runtime performance of output (on a 10-benchmark micro-suite)
- Optimization decisions made (from `episodes.ndjson`)

**Comparison.** Compare compilations 11–14 against a "stateless" control run with the cache cleared each time.

**Success criterion.**
- Runtime of outputs from cache-enabled compilations faster than stateless by ≥5% (geomean)
- Compilation time not worse by more than 10% overhead (reading cache should be fast)

---

## 8. Hot-Reload Mode Boundaries (Paper VI Contribution 6)

### 8.1 Hypothesis (Paper VI §7.3)

Zero-downtime orchestration updates with message-delivery latency increase bounded to the reload window duration.

### 8.2 Method

**Setup.** A long-running Garnet agent (build pipeline, 10 concurrent build-requests/second) with:
- Safe-mode hot path (compression, hashing) — ~40% of CPU
- Managed-mode orchestration — ~60% of CPU

**Load.** Sustained 10 req/sec for 60 seconds.

**Action.** At T+30s, hot-reload the managed-mode module (a new bytecode version with a minor orchestration change).

**Measures.**
- Request success rate (1-loss) across the reload window
- P50/P95/P99 request latency in 1-second buckets across the reload window
- Zero data races, zero corruption (assert invariants)

**Success criterion.**
- 0 lost requests (mailbox buffering works)
- P99 latency increase during reload < 100ms beyond steady-state
- Full recovery to steady-state within 500ms of reload completion

---

## 9. Deterministic Reproducible Builds (Paper VI Contribution 7)

### 9.1 Hypothesis (Paper VI §8.3)

Same source + same compiler version + same target triple produces byte-identical binaries across distinct machines.

### 9.2 Method

**Setup.** Build the same Garnet project on:
- Machine A: Ubuntu 24.04, x86_64
- Machine B: Debian 12, x86_64 (different base image but same triple)
- Machine C: Docker container on macOS host, x86_64-unknown-linux-gnu target

**Target.** `garnet build --release --deterministic` on each machine.

**Measures.**
- SHA-256 of output binary from each machine
- SHA-256 of the `.garnet-manifest` extracted from each
- `garnet verify --rebuild` success rate on a random sample of published binaries

**Success criterion.**
- Byte-identical binaries across A, B, C
- `garnet verify --rebuild` success rate 100% on 50 sample binaries from the registry

### 9.3 Weaker target

If full byte-identical proves unachievable due to LLVM non-determinism, the fallback success criterion is:
- Binaries differ only in padding / debug-info sections
- `strip`-ed binaries are byte-identical
- `garnet verify` against a stripped-binary hash is 100% successful

This fallback still beats every other mainstream language's reproducibility guarantee.

---

## 10. Execution Timeline

| Phase | Benchmarks | Dependency | Timeline |
|---|---|---|---|
| Pilot | §3 (compilation perf, synthetic) | Rung 3 parser + partial interpreter | Q3 2026 |
| Pilot | §6 ergonomics (5 developers) | Rung 3 REPL + LSP | Q4 2026 |
| Alpha | §4.1 (managed vs Ruby) | Rung 3 managed interpreter | Q4 2026 |
| Beta | §4.2 (safe vs Rust) | Rung 4 LLVM codegen | Q1 2027 |
| Beta | §4.3 kind-aware | Rung 5 Memory Core + allocators | Q2 2027 |
| Submission | §5 LLM benchmark | Paper VI + 500 tasks + trained models | Q3 2026 (preliminary for PLDI), Q2 2027 (expanded) |
| Post-submit | §7 compiler-as-agent | Rung 5 strategies DB | Q3 2027 |
| Post-submit | §8 hot-reload | Rung 6 harness runtime | Q4 2027 |
| Post-submit | §9 reproducible builds | Rung 4 + 5 codegen stability | Q4 2027 |
| §6 full | 30-dev study | Rung 3 polish + LSP production-ready | Q1 2027 |

---

## 11. Open-Sourcing Commitments

All benchmarks, datasets, and methodology will be open-sourced under MIT license at the time of Paper VI submission (October 2026):

- `garnet-bench/` — the benchmark harness and synthetic corpus generator
- `garnet-llm-tasks/` — the 500-task LLM evaluation set
- `garnet-ergonomics-study/` — anonymized participant data and analysis scripts from §6

This addresses the replicability critique preemptively and maximizes scientific value.

---

*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*
*"Where there is no vision, the people perish." — Proverbs 29:18*

**Benchmarking & Evaluation Plan prepared by Claude Code (Opus 4.7) | April 16, 2026**
