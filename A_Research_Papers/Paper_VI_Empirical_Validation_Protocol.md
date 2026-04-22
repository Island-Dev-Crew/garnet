# Paper VI — Empirical Validation Protocol (v3.3 Phase 1C)

**Companion to:** `Paper_VI_Garnet_Novel_Frontiers.md`
**Date:** April 16, 2026
**Author:** Claude Code (Opus 4.7) — Phase 1C Stage 1
**Status:** Normative experimental protocols, ready to execute in Stage 4 (v4.0)
**Anchor:** *"Test all things; hold fast that which is good." — 1 Thessalonians 5:21*

---

## Purpose

Paper VI states seven novel contributions. Each carries a falsifiable hypothesis, but Paper VI itself does not specify *how* to test those hypotheses with sufficient precision that another researcher could replicate the experiment. This protocol document closes that gap.

For each of the seven contributions, this document specifies:

1. **Hypothesis (H)** — copied verbatim from Paper VI (or refined to remove ambiguity)
2. **Experimental protocol (P)** — exact steps to run, with operational decisions pre-committed
3. **Pass / fail criterion (C)** — what empirical result supports vs. refutes H
4. **Measurement harness (M)** — code or tool that executes P and emits the data needed for C
5. **Expected risk (R)** — what happens if H fails, and how Paper VI must be honestly updated

Pre-commitment matters. **The protocols below are written before the data is collected.** Anything subsequently added to "rescue" a failed hypothesis is a post-hoc rationalization, not science. If Stage 4 finds H₁ refuted, Paper VI Contribution 1 gets rewritten with the honest finding. This is the explicit commitment of the v3.3 plan: rigor over haste.

---

## Experiment 1 — LLM-Native Syntax: pass@1 study

**Maps to:** Paper VI Contribution 1 (LLM-Native Syntax Design)

### H₁ (hypothesis)

A state-of-the-art LLM (Claude Opus 4.x or GPT-5 generation), given a fixed-difficulty programming task with the same prompt structure, achieves **higher pass@1** when generating Garnet than when generating Rust for safety-equivalent tasks AND **comparable pass@1** to Ruby for ergonomics-equivalent tasks.

Refined: For 100 mid-difficulty tasks (≥50 LOC reference solution, requires data structures + control flow + at least one type), the difference `pass@1(Garnet) − pass@1(Rust)` is positive at p < 0.05 across at least 2 of 3 LLMs evaluated.

### P₁ (protocol)

1. **Build the benchmark.** Curate 100 programming tasks split: 40 managed-mode-style (data transformation, parsing, scripting), 40 safe-mode-style (memory-managed, ownership-relevant), 20 mixed-mode (dual-mode interaction). Tasks are pulled from public benchmarks (HumanEval, MBPP, SWE-bench Lite) and translated to triple-language reference solutions (Garnet, Rust, Ruby) by a human author. Each task has: a problem statement, a public test suite (≥3 cases), a reference solution, and a difficulty score (1–5).
2. **Pin the prompt template.** A single prompt template per language family. Prompts contain only: the problem statement, the language declaration ("Write the solution in Garnet" / "in Rust" / "in Ruby"), and the test-runner contract (function signature). No language-specific scaffolding hints.
3. **Generate.** For each (task, language, LLM) triple, sample n=10 completions at temperature 0.2. Compile each completion against the language's compiler, run the public test suite, record pass / fail / compile-error.
4. **pass@1 metric.** For each completion attempt, mark "pass" iff (a) compiles and (b) passes all public tests. pass@1 across 10 samples = (mean pass rate at n=1, computed via the standard Codex paper formula `1 − C(n−c, k)/C(n, k)` for k=1).
5. **Run on three LLMs.** Claude Opus 4.x, GPT-5 (whatever generation is current at v4.0), and Gemini 3 (or successor). This guards against finding a result that holds only for our preferred model.

### C₁ (pass/fail criterion)

H₁ is **supported** if `pass@1(Garnet) − pass@1(Rust) > 0` at p < 0.05 (paired Wilcoxon signed-rank test across 100 tasks) on at least 2 of the 3 evaluated LLMs.

H₁ is **refuted** if `pass@1(Garnet) ≤ pass@1(Rust)` at p < 0.05 on at least 2 of the 3 LLMs, OR if the result is statistically inconclusive on all 3 (n=100 too small to detect a real effect — power analysis below).

H₁ is **inconclusive** if only 1 of 3 LLMs shows significance (cannot generalize).

**Power analysis.** With n=100 paired tasks, detecting a 5-percentage-point pass@1 difference at α = 0.05 requires σ ≤ 22.5 percentage points (standard deviation of the per-task pass@1 difference). Realistic σ on HumanEval-style benchmarks is 30–40 pp; therefore **a 10-pp effect is the minimum reliably detectable difference**. The benchmark MUST flag a smaller-than-10-pp result as inconclusive, NOT as supporting H₁.

### M₁ (measurement harness)

```
benchmarks/
  paper_vi_exp1_llm_pass_at_1/
    tasks/
      00_basic_fizzbuzz/  # task spec, reference (3 lang), tests (3 lang)
      01_…
      99_…
    runners/
      run_garnet.sh       # invokes garnet build + garnet test
      run_rust.sh
      run_ruby.sh
    llm_clients/
      claude.py           # wraps anthropic SDK with prompt template
      openai.py
      gemini.py
    aggregate.py          # collects all (task, lang, llm, sample) → pass/fail
    analyze.py            # computes pass@1 + paired Wilcoxon
```

`aggregate.py` outputs a single `results.csv` with columns: `task_id, lang, llm, sample_id, compiled, tests_passed, time_ms`. `analyze.py` consumes that CSV.

### R₁ (expected risk)

If H₁ is refuted, Paper VI Contribution 1 gets rewritten as: *"Garnet's syntax is designed FOR LLM generation; we measure this as a research contribution but do not yet beat Rust on pass@1. The languages identified as design influences (Ruby for ergonomics, Rust for safety) provide the floor; Garnet is closer to the Ruby floor on managed-mode tasks and approaches the Rust floor on safe-mode tasks."* This is the truthful framing if the data does not support the stronger claim.

---

## Experiment 2 — Progressive Type-Disclosure: bidirectional compatibility theorem

**Maps to:** Paper VI Contribution 2 (Progressive Type-Disclosure Spectrum)

### H₂ (hypothesis)

For every well-typed Garnet program P at type-discipline level N (per Mini-Spec §11.1, levels 0–3):

- **(Strengthening soundness)** P type-checks at level N+1 OR the type-checker emits a precise localized error indicating which annotation must be added.
- **(Relaxation safety)** P type-checks at level N−1 with the additional annotations becoming inert documentation.

This is the *Progressive Disclosure Monotonicity Theorem* stated in Mini-Spec §11.1. Experiment 2 tests it empirically on a corpus rather than only in proof-sketch form.

### P₂ (protocol)

1. **Corpus.** Take a 200-program corpus: 50 hand-written examples from Garnet docs + 50 from Rosetta Code-style equivalents + 50 generated by LLM at level 2 (Static) + 50 at level 3 (Affine).
2. **For each program P at level N:** mechanically derive a relaxed version `P_N-1` by removing all annotations introduced by level N (e.g., `let x: Int = 5` → `let x = 5`). Mechanically derive a strengthened version `P_N+1` by running the inference engine and committing the inferred annotations.
3. **Verify.** Run `garnet check` on `P`, `P_N-1`, `P_N+1`. Record pass / fail / specific error.
4. **Bidirectional compat metric.** P passes the bidirectional test iff:
   - relaxation: `P_N-1` type-checks AND its observable behavior (via test suite) is identical to P
   - strengthening: `P_N+1` either type-checks (success) OR the error is localized to a specific identifier with a suggested annotation (acceptable failure — programmer-actionable)

### C₂ (pass/fail criterion)

H₂ is **supported** if ≥ 95% of the corpus passes the bidirectional test for both directions. The 5% slack accounts for genuinely level-N-specific patterns (e.g., a `@dynamic` use that intrinsically refuses strengthening, but should still emit a precise error).

H₂ is **refuted** if any program produces an "unsuggestable" error at strengthening (e.g., type-checker panic, error without source location, "internal compiler error") OR if any program at relaxation produces silently different runtime behavior.

### M₂ (measurement harness)

```
benchmarks/
  paper_vi_exp2_progressive_disclosure/
    corpus/                       # 200 .garnet programs
    relaxer.rs                    # tool: remove level-N annotations
    strengthener.rs               # tool: infer + commit level-N+1 annotations
    test_runner.rs                # cargo xtask paper-vi-exp2
    report.csv                    # per-program: relax_ok, strengthen_ok, behavior_match
```

The `relaxer.rs` and `strengthener.rs` tools live under `xtask/` since they're test-only. They reuse the Rung-3 type-checker via in-process API calls.

### R₂ (expected risk)

If H₂ is refuted, the failure mode tells us where the spec is wrong. If strengthening produces unactionable errors, the inference engine is incomplete; Mini-Spec §11.1 needs to admit a weaker form of monotonicity ("strengthening is sound except at named carve-outs"). If relaxation changes behavior, we have a soundness bug — that's a P0 fix, not a paper rewrite. The paper must be updated with the carve-outs.

---

## Experiment 3 — Compiler-as-Agent: time-to-fix improvement

**Maps to:** Paper VI Contribution 3 (Compiler-as-Agent Architecture)

### H₃ (hypothesis)

A compilation-history-aware Garnet compiler produces measurably better outcomes after 10 sequential compilations of an evolving codebase compared to a stateless compiler. Specifically:

- **Compilation speed** (h₃a): mean time-per-compile in compilations 6–10 is < 90% of mean time-per-compile in compilations 1–5 (i.e., learned strategies skip historically-unproductive passes).
- **Strategy hits** (h₃b): in compilations 6–10, the strategy miner produces ≥ 1 hit per compilation that demonstrably reduces a specific pass's runtime, recorded by the existing episodes.log.
- **Provenance robustness** (h₃c): all strategies that the compiler honors in compilations 6–10 are re-derivable from HMAC-verified episodes (per v3.3 ProvenanceStrategy).

### P₃ (protocol)

1. **Codebase.** Take an evolving 800-LOC project (the v3.4 OS-sim MVP works well — sufficient surface area, real semantic content). Generate a sequence of 10 incremental versions: each version makes a small change (≤ 50 LOC diff) chosen to exercise different parts of the type checker / borrow checker / inliner.
2. **Run sequence A (stateless).** Wipe `.garnet-cache/` between every compile. Record per-compile elapsed wallclock time + per-pass timing.
3. **Run sequence B (history-aware).** Preserve `.garnet-cache/` across compiles. Same per-compile timing recorded.
4. **Metrics:**
   - `mean_time(B[6..10]) / mean_time(B[1..5])` — should be < 0.90 to support h₃a.
   - `mean_time(A[6..10]) / mean_time(A[1..5])` — control: should be ~1.0 (no learning across stateless compiles).
   - `strategies_hit_per_compile(B[6..10])` — should be ≥ 1.0 to support h₃b.
   - For each strategy hit in B[6..10], call `provenance::verify_strategy()` — should return Ok for all to support h₃c.
5. **Three runs of the full protocol.** Average across 3 to get a stable timing estimate.

### C₃ (pass/fail criterion)

H₃ is **supported** if all three sub-hypotheses h₃a, h₃b, h₃c hold across the 3 runs.

H₃ is **partially supported** if h₃c holds (provenance is robust) but h₃a fails (no measurable speedup). This is the most likely partial-failure mode and is acceptable — it means the compiler-as-agent learns and protects integrity, but the strategies aren't yet impactful enough to dominate compilation time on small codebases.

H₃ is **refuted** if h₃c fails — strategies are honored without re-derivable provenance.

### M₃ (measurement harness)

```
benchmarks/
  paper_vi_exp3_compiler_as_agent/
    codebase_versions/             # 10 .tar.gz snapshots of the evolving project
    run_stateless.sh               # rm -rf .garnet-cache && garnet build, ×10
    run_history_aware.sh           # garnet build, ×10
    timing_collector.rs            # parses garnet build --timings JSON
    provenance_audit.rs            # iterates strategies.db, calls verify_strategy
    report.csv                     # per-version per-mode timing + strategy stats
```

### R₃ (expected risk)

The most likely outcome at v4.0 is **partial support**: h₃c will pass (we built it carefully), h₃b will pass (the miner does generate strategies), but h₃a may produce only a 2–5% speedup on this small codebase — below the 10% threshold of "measurably better." If so, Paper VI Contribution 3 gets updated to: *"On evolving 800-LOC codebases, the compiler-as-agent's measurable speedup is in the 2–5% range. The contribution's stronger claim (10%+ speedup) requires a larger codebase to validate, and is moved to v4.x as a research direction."* This is the honest framing.

---

## Experiment 4 — Kind-Aware Memory Allocation: memory benchmark

**Maps to:** Paper VI Contribution 4 (Kind-Aware Memory Allocation)

### H₄ (hypothesis)

For programs that use the four memory kinds (working / episodic / semantic / procedural), Garnet's kind-aware allocation reduces:

- **Peak memory** (h₄a) by ≥ 20% vs. a general-purpose allocator (system malloc) on representative agent workloads
- **Allocation latency** (h₄b) p99 by ≥ 30% vs. system malloc

### P₄ (protocol)

1. **Workloads.** Two representative agent workloads from the v3.4 MVPs:
   - **Multi-agent orchestrator (MVP 6)** — exercises all 4 kinds simultaneously
   - **Relational DB (MVP 2)** — exercises working (query state) + episodic (WAL log) primarily
2. **Configurations.**
   - A: kind-aware allocators (baseline Garnet behavior)
   - B: all-malloc (force every kind to use system malloc via `--allocator=malloc` flag)
3. **Measurement.** Run each workload at three scales: 100, 1000, 10000 iterations. Record:
   - peak RSS (via `getrusage(RUSAGE_SELF).ru_maxrss`)
   - allocation latency histogram (per-allocation timing collected via instrumented allocator)
   - throughput (operations / second)
4. **Metrics:**
   - peak_rss_ratio = `peak_rss(B) / peak_rss(A)` — should be ≥ 1.20 to support h₄a (i.e., A is 20% smaller)
   - alloc_p99_ratio = `alloc_p99(B) / alloc_p99(A)` — should be ≥ 1.30 to support h₄b
   - throughput_ratio = `throughput(A) / throughput(B)` — should be ≥ 0.95 (kind-aware mustn't slow us down)

### C₄ (pass/fail criterion)

H₄ is **supported** if both h₄a AND h₄b hold on at least one of the two workloads at the largest scale (10000 iterations), AND throughput is not degraded by > 5%.

H₄ is **partially supported** if only h₄a OR h₄b holds.

H₄ is **refuted** if neither h₄a nor h₄b holds at the 10000-iter scale.

### M₄ (measurement harness)

```
benchmarks/
  paper_vi_exp4_kind_aware_alloc/
    workloads/
      multi_agent.garnet
      relational_db.garnet
    runners/
      run_kind_aware.sh
      run_malloc.sh
    instrumented_alloc/
      mod.rs                 # allocator wrapper that records every alloc/free
    report.csv
```

The `instrumented_alloc` wrapper compiles in via `--allocator=instrumented` flag added in Phase 2A.

### R₄ (expected risk)

This experiment depends on the v3.4 stdlib's allocator implementations actually selecting kind-appropriate strategies. If v3.4 ships a placeholder ("kind-aware = malloc with a tag") because of time pressure, h₄a and h₄b will both be refuted because there's no real differentiation. In that case, Paper VI Contribution 4 must be updated to: *"v3.4 demonstrates the kind-aware annotation surface; v4.x will deliver the differentiated allocators that achieve the 20%/30% targets."* This is the honest gap acknowledgment if the v3.4 implementation slips.

---

## Experiment 5 — Error-Model Bridging: zero-loss audit

**Maps to:** Paper VI Contribution 5 (Bidirectional Error-Model Bridging)

### H₅ (hypothesis)

For 100 hand-crafted error-bridging cases spanning the four directions (managed→safe, safe→managed, double-bounce, type-mismatch loud-fail), the original error payload is preserved bit-identical across the bridge in all cases — zero information loss.

### P₅ (protocol)

1. **Build the test corpus.** 25 cases per direction × 4 directions = 100 cases. Each case: a managed function calling a safe function (or vice versa) with a specific error payload. The payload is content-addressed (BLAKE3 hash of its serialization).
2. **Run each case twice.**
   - Path 1: through the manual `try`/`rescue` + `?` composition (v3.2/v3.3 shipped behavior)
   - Path 2: through the automatic bridging compiler insertion (v4.0 target)
3. **Capture the error payload at the rescue site.** Hash it.
4. **Compare to the originally-raised payload's hash.**

### C₅ (pass/fail criterion)

H₅ is **supported** if all 100 cases on Path 1 AND all 100 on Path 2 produce a rescued payload whose hash equals the raised payload's hash.

H₅ is **partially supported** if Path 1 is 100/100 but Path 2 has hash mismatches — this means the v4.0 automatic bridging is buggy and must be fixed before v4.0 ships.

H₅ is **refuted** if Path 1 itself loses information — this is a soundness bug in v3.2/v3.3 and is a P0.

### M₅ (measurement harness)

```
benchmarks/
  paper_vi_exp5_error_bridging/
    cases/
      managed_to_safe/
        case_01.garnet
        …
      safe_to_managed/
      double_bounce/
      type_mismatch/
    payload_hasher.rs
    runner.rs            # cargo xtask paper-vi-exp5
    report.csv           # per-case: path1_match, path2_match
```

Note: Path 2 (automatic bridging) requires the v4.0 type-checker. If automatic bridging is not yet implemented at v4.0 ship, the experiment runs only Path 1, and the v4.0 paper notes that Path 2 validation is deferred to v4.x.

### R₅ (expected risk)

The Path 1 result is the high-confidence one — v3.3's `boundary_errors.rs` tests already validate small-N versions of this. Scaling to 100 cases is unlikely to find new failures. The Path 2 result depends on v4.0 implementation work; if it fails, Paper VI Contribution 5's "automatic" claim must be honestly downgraded — exactly what the Phase 1A slop reverification did at v3.3.

---

## Experiment 6 — Hot-Reload Latency: p99 under 1000 reloads

**Maps to:** Paper VI Contribution 6 (Hot-Reload Mode Boundaries)

### H₆ (hypothesis)

During 1000 sequential hot-reloads of an actor under continuous message load:

- **Latency bound (h₆a):** Message-delivery latency p99 increase during reloads is bounded — specifically, p99 latency during the reload window stays below 10× the no-reload baseline.
- **Zero loss (h₆b):** No messages are dropped (every send is matched by a receive within 1 second of reload completion).
- **State preservation (h₆c):** State extracted via §9.4 StateCert / `extract_state` survives all 1000 reloads; the final state matches the initial state plus all observed message effects.

### P₆ (protocol)

1. **Setup.** Single counter actor receiving 1000 msg/sec from a generator actor. Counter has v1, v2, v3 schemas alternating across reloads.
2. **Run 1000 reload cycles.** Each cycle: send N messages, trigger reload, send N more messages, verify count.
3. **Measure.** Per-message latency (send→reply round-trip). Compute p99 latency in the reload window vs. baseline window.
4. **Compare.** ratio = `p99(reload_window) / p99(baseline_window)` — should be ≤ 10 for h₆a.
5. **Audit.** Compare final counter value to expected. Compare StateCert fingerprints of v1/v2/v3 across all 1000 reloads — should be stable per schema version.

### C₆ (pass/fail criterion)

H₆ is **supported** if all three sub-hypotheses hold.

H₆ is **partially supported** if h₆b and h₆c hold (no loss, state preserved) but h₆a fails (latency exceeds 10× ratio). This means the reload mechanism is correct but slow — usable for orchestration, not for hot paths.

H₆ is **refuted** if either h₆b or h₆c fails — a correctness violation in the reload mechanism.

### M₆ (measurement harness)

```
benchmarks/
  paper_vi_exp6_hot_reload/
    actor_v1.garnet
    actor_v2.garnet
    actor_v3.garnet
    runner.rs              # spawns generator + counter, drives 1000 reloads
    latency_collector.rs   # per-message timing histogram
    report.csv
```

Already most of the infrastructure exists — `garnet-actor-runtime/tests/reload.rs` covers the per-reload correctness; this experiment scales it to 1000 cycles with latency measurement.

### R₆ (expected risk)

p99-during-reload of 10× baseline is generous; current measurements suggest 3–5×. h₆a is likely supported. h₆b is the "could fail catastrophically" risk — if a single message gets dropped under load, that's a reload-mechanism bug. h₆c is reasonably high confidence given v3.3 StateCert tests.

---

## Experiment 7 — Deterministic Builds: two-machine hash test

**Maps to:** Paper VI Contribution 7 (Deterministic Reproducible Builds)

### H₇ (hypothesis)

Compiling the same Garnet project on two different machines (different hardware, OS, distribution) with the same compiler version + same dependencies produces byte-identical manifest hashes.

### P₇ (protocol)

1. **Project.** A non-trivial Garnet project of ~2000 LOC drawing on the v3.4 MVPs.
2. **Machine A.** Native Windows 11 with Garnet built from main.
3. **Machine B.** WSL2 Ubuntu 24.04 LTS with Garnet built from main.
4. **(Stretch)** **Machine C.** macOS Sonoma on Apple Silicon. Comparison of the cross-architecture manifests requires care — the target triple differs by design, but the source-artifact portion of the manifest should match.
5. **Procedure.** On each machine: `git clone`, `garnet build --deterministic`, capture `manifest.json`. Diff the manifests.
6. **Decompose.** The manifest has multiple hash fields (source_hash, ast_hash, prelude_hash, dep_hashes, manifest_hash). Compare each independently.

### C₇ (pass/fail criterion)

H₇ is **supported** for cross-machine same-OS-family if Machine A and Machine B produce identical source_hash, ast_hash, prelude_hash, dep_hashes (codegen-output hash MAY differ if the target triple differs).

H₇ is **strongly supported** if Machine A, B, and C all produce identical source_hash, ast_hash, prelude_hash, dep_hashes — i.e., all source-derived fields match across architectures.

H₇ is **refuted** if any of the source-derived fields differ on any pair of machines with the same target triple.

### M₇ (measurement harness)

```
benchmarks/
  paper_vi_exp7_deterministic_build/
    project/                    # the 2000-LOC test project
    capture_manifest.sh         # garnet build --deterministic + cp manifest.json
    diff_manifests.py           # field-by-field comparison + summary report
    report.md
```

### R₇ (expected risk)

source_hash / ast_hash / prelude_hash already proven byte-identical across cwds in Phase 1A — those are very high confidence. dep_hashes depend on dep tree resolution being deterministic, which is a property of the package manager (yet to ship — currently we have no deps beyond stdlib + workspace crates). codegen-output hash will differ across target triples by design.

The most likely failure is a non-determinism in the rusqlite-backed knowledge.db serialization, which would surface in dep_hashes once we have non-trivial deps. The fix is to canonicalize the SQLite serialization explicitly, which is already done for episodes.log + strategies.db via length-prefixed canonical encoding.

---

## Aggregate Pass/Fail Reporting (v4.0 deliverable)

After all 7 experiments run, the v4.0 deliverable updates Paper VI's contribution sections with the structured outcome:

```
| # | Contribution | H supported | Partial | Refuted | Notes |
|---|--------------|-------------|---------|---------|-------|
| 1 | LLM-native syntax | … | … | … | … |
| 2 | Progressive disclosure | … | … | … | … |
| 3 | Compiler-as-agent | … | … | … | … |
| 4 | Kind-aware allocation | … | … | … | … |
| 5 | Error-model bridging | … | … | … | … |
| 6 | Hot-reload | … | … | … | … |
| 7 | Deterministic builds | … | … | … | … |
```

Each cell is filled in honestly. Refutations and partials trigger Paper VI rewrites — not "we'll re-run with a different methodology" rationalizations.

---

## Pre-Registration Statement

Per the standard of pre-registered research, this protocol document constitutes the methodological commitment for the seven empirical claims of Paper VI. The protocols above are fixed at v3.3 Phase 1C (April 2026). Any modifications between this commitment and Stage 4 execution will be tracked with a redline.

The prediction directions (which way each hypothesis points) are stated explicitly in each H section. The pass / fail thresholds are quantitative and pre-committed. Statistical tests are named.

This is the discipline an MIT review panel — and the Garnet research team itself — should expect from a doctoral-class language project.

---

## Cross-references

- Paper VI: `Paper_VI_Garnet_Novel_Frontiers.md`
- Mini-Spec v1.0: `../C_Language_Specification/GARNET_v1_0_Mini_Spec.md`
- Paper V Addendum v1.0: `Paper_V_Addendum_v1_0.md`
- v3.3 Slop Reverification: `../F_Project_Management/GARNET_v3_3_SLOP_REVERIFICATION.md`
- Master plan Phase 4A: `~/.claude/plans/i-ll-follow-plan-mode-proud-lollipop.md`

---

*Prepared 2026-04-16 by Claude Code (Opus 4.7) — Phase 1C deliverable. Pre-registered protocols for the seven Paper VI empirical claims.*

*"Test all things; hold fast that which is good." — 1 Thessalonians 5:21*
