# Paper VI Experiment 3 — compiler-as-agent time-to-fix (S71)

Experiment 3 tests whether Garnet's compiler-as-agent — its episodic-history
mechanism (`.garnet-cache/episodes.log` + HMAC-verified strategies) — speeds up
time-to-fix as a codebase evolves. The harness lives at
`benchmarks/paper_vi_exp3_compiler_as_agent/` and ships the reproducible *shape*
of the study (10 evolving snapshots × stateless/history-aware lanes), not a fresh
result. `scripts/garnet_paper_vi_exp3_status.py` reports its status and is gated
in CI.

## Pre-registered H₃

- **h₃a** — `mean_time(B[6..10]) / mean_time(B[1..5]) < 0.90` (≥ 10% speedup).
- **h₃b** — ≥ 1 strategy hit per compilation in compiles 6–10.
- **h₃c** — all honored strategies re-derivable from HMAC-verified episodes.

## Recorded v4.0 outcome (partial-support)

From `GARNET_v4_0_PAPER_VI_EXECUTION.md` (800-LOC MVP 1 codebase):

- **h₃a — partial.** 6.5% speedup (CI [3.1%, 9.8%]) — real and statistically
  significant (p<0.01), but below the pre-registered 10% threshold.
- **h₃b — pass.** 1.4 strategies/compile in cycles 6–10 (≥ 1.0).
- **h₃c — pass.** `provenance.verify_strategy` 100%.

Paper VI Contribution 3 is honestly **downgraded** in the v4.0 revision:
"…the compiler-as-agent's measurable speedup is **6.5% (CI [3.1%, 9.8%])**; the
stronger 10% claim holds on larger codebases where pass-skipping compounds; v4.x
will re-run the experiment on a 5K-LOC test project."

## What S71 actually runs

`garnet_paper_vi_exp3_status.py` (+ `--gate`, CI) inventories the harness (10
snapshots, both lanes, aggregate/analyze) and **runs the provider-free mode** —
both lane scripts in harness-only mode plus `aggregate.py`, which emit the honest
"harness-only / no results invented" shape and exit 0. The gate fails if the
harness is missing/malformed or its provider-free run fails.

## Honest scope (do not soften)

- **h₃a's timing speedup is machine-dependent and is NOT re-measured here** — the
  determinism doctrine forbids inventing measurements. The recorded 6.5% partial
  stands; the 10% claim is downgraded honestly.
- The **provider-backed re-run** (and the `--llm` suggest variant the lane scripts
  reference) is **pending-infra** — no LLM provider or API credits in this
  environment, the same boundary as Exp 1. No model is called.
- The verbatim §C3 honesty anchor ("6.5% (CI [3.1%, 9.8%])") is surfaced, not
  softened. This is a status/run-verification layer, not a new measurement.
