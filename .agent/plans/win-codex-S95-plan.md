# S95 Paper VI Exp 3 5K-LOC Harness Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or TDD-style inline execution task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend Paper VI Experiment 3 from the S19/S71 toy harness to a reproducible 5K-LOC rerun harness while preserving the honest boundary that h3a is not remeasured without provider-backed execution evidence.

**Architecture:** S95 stays under `benchmarks/paper_vi_exp3_compiler_as_agent/`. It adds deterministic 5K-LOC corpus generation, a 5K harness runner, aggregate/analyze support for measured rows when they exist, and a status gate that proves corpus scale plus provider-free execution. The gate must report `h3a` as unresolved/pending unless real measured rows are present.

**Tech Stack:** Python standard library, existing Exp 3 JSONL pattern, generated Garnet snapshots under `target/`, readiness/status-script conventions.

---

### Task 1: Status Tests First

**Files:**
- Create: `scripts/test_garnet_paper_vi_exp3_5k_status.py`
- Create later: `scripts/garnet_paper_vi_exp3_5k_status.py`

- [x] Write tests that expect the S95 status reporter to prove 10 generated snapshots, each at least 5,000 LOC, provider-free lane execution, aggregate/analyze output, and `h3a_status == "pending-provider-rerun"`.
- [x] Run `python scripts/test_garnet_paper_vi_exp3_5k_status.py` and confirm it fails because the reporter does not exist yet.

### Task 2: Deterministic 5K Corpus Generator

**Files:**
- Create: `benchmarks/paper_vi_exp3_compiler_as_agent/generate_5k_corpus.py`

- [x] Generate ten versioned Garnet snapshots under an output directory.
- [x] Make each snapshot at least 5,000 logical lines and deterministic byte-for-byte.
- [x] Include manifest metadata with snapshot id, LOC, sha256, and evolution index.

### Task 3: Provider-Gated 5K Runner And Analysis

**Files:**
- Create: `benchmarks/paper_vi_exp3_compiler_as_agent/run_5k.py`
- Create: `benchmarks/paper_vi_exp3_compiler_as_agent/aggregate_5k.py`
- Create: `benchmarks/paper_vi_exp3_compiler_as_agent/analyze_5k.py`

- [x] Provider-free mode writes stateless/history-aware rows for all ten snapshots with `pending-provider-rerun`.
- [x] Aggregation computes measured h3a only when measured timing rows exist.
- [x] Analysis preserves the v4.0 6.5% partial and says the stronger 10% claim is unresolved without measured 5K rows.

### Task 4: Status Gate And Readiness

**Files:**
- Create: `scripts/garnet_paper_vi_exp3_5k_status.py`
- Modify: `scripts/garnet_mit_readiness_status.py`

- [x] Run generator, provider-free runner, aggregate, and analyze into `target/paper_vi_exp3_5k_status/`.
- [x] Emit JSON/Markdown with `ok: true` only for the reproducible 5K harness, not for a provider-backed result.
- [x] Add a committed-truth readiness lane for the S95 harness.

### Task 5: Documentation And Ledger

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `F_Project_Management/GARNET_v0_8_1_PLAN.md`
- Modify: `F_Project_Management/AGENT_COORDINATION_LEDGER.md`
- Modify: `.dogfood/goal.json`

- [x] Mark S94 merged and S95 active.
- [x] Document S95 as harness-ready / provider-rerun-pending, not as a new h3a measurement.

### Task 6: Verification And PR

**Files:**
- Desktop bundle under `C:\Users\IslandDevCrew\Desktop\dogfood\garnet-s95-paper-vi-exp3-5k-loc-*`

- [ ] Run focused S95 tests/status gate.
- [ ] Run workspace tests and clippy.
- [ ] Seal a Desktop dogfood bundle.
- [ ] Open PR `S95: add Paper VI Exp 3 5K-LOC rerun harness`.
- [ ] Merge only after PR body gate and remote checks are green.
