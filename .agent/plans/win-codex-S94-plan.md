# S94 Paper VI Exp 1 Harness Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or TDD-style inline execution task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the Paper VI Experiment 1 LLM pass@1 harness behind an explicit provider flag, while keeping provider-backed measurements honestly pending when credentials or the full corpus are absent.

**Architecture:** S94 is a benchmark/status slice, not a compiler or runtime change. The harness lives under `benchmarks/paper_vi_exp1_llm_pass_at_1/`, writes reproducible JSONL/JSON evidence, supports provider-free and deterministic fixture modes, and treats real providers as opt-in credential-gated work. A repository status script gates only the source-present/provider-free/fixture paths and never converts missing credentials into a fake result.

**Tech Stack:** Python standard library, JSON/JSONL result files, existing Garnet readiness/status-script conventions.

---

### Task 1: Status Tests First

**Files:**
- Create: `scripts/test_garnet_paper_vi_exp1_status.py`
- Create later: `scripts/garnet_paper_vi_exp1_status.py`

- [ ] Write tests that expect the S94 status reporter to expose an `ok` gate, count seed tasks, prove provider-free execution, prove fixture scoring, and keep provider-backed execution labeled `pending-credentials` / `pending-infra`.
- [ ] Run `python scripts/test_garnet_paper_vi_exp1_status.py` and confirm it fails because the status reporter does not exist yet.

### Task 2: Harness Skeleton And Seed Corpus

**Files:**
- Create: `benchmarks/paper_vi_exp1_llm_pass_at_1/README.md`
- Create: `benchmarks/paper_vi_exp1_llm_pass_at_1/tasks/manifest.json`
- Create task prompt/spec/reference files under `benchmarks/paper_vi_exp1_llm_pass_at_1/tasks/`

- [ ] Add a small seed manifest that represents the 500-task target without claiming corpus completion.
- [ ] Include paired Garnet/Rust task entries, public specs, and reference answers sufficient for provider-free/fixture harness validation.

### Task 3: Provider-Gated Runner

**Files:**
- Create: `benchmarks/paper_vi_exp1_llm_pass_at_1/run.py`
- Create: `benchmarks/paper_vi_exp1_llm_pass_at_1/aggregate.py`
- Create: `benchmarks/paper_vi_exp1_llm_pass_at_1/analyze.py`

- [ ] Implement `run.py --provider none|fixture|openai|anthropic|gemini|ollama`.
- [ ] Make `none` write pending rows and exit 0.
- [ ] Make `fixture` write deterministic measured rows so aggregation and pass/fail math are tested without a network call.
- [ ] Make real providers require an explicit execution flag plus credentials, otherwise write honest pending rows and exit 0.
- [ ] Implement aggregation and analysis outputs that distinguish `measured` from `pending` results.

### Task 4: Status Gate

**Files:**
- Create: `scripts/garnet_paper_vi_exp1_status.py`

- [ ] Inventory the harness and seed corpus.
- [ ] Run provider-free mode and fixture mode into `target/paper_vi_exp1_status/`.
- [ ] Emit JSON/Markdown with `ok: true` only when the harness source, provider-free run, fixture run, aggregate, and analyzer are all reproducible.
- [ ] Keep provider-backed execution and full 500-task corpus explicitly unclaimed.

### Task 5: Readiness And Documentation

**Files:**
- Modify: `scripts/garnet_mit_readiness_status.py`
- Modify: `CHANGELOG.md`
- Modify: `F_Project_Management/GARNET_v0_8_1_PLAN.md`
- Modify: `F_Project_Management/AGENT_COORDINATION_LEDGER.md`
- Modify: `.dogfood/goal.json`

- [ ] Add a committed-truth readiness lane for Paper VI Exp 1 harness wiring.
- [ ] Record S93 merged and S94 started/opened with calibrated scope.
- [ ] Update changelog and the v0.8.1 plan without claiming provider-backed pass@1 numbers.

### Task 6: Verification And PR

**Files:**
- Desktop bundle under `C:\Users\IslandDevCrew\Desktop\dogfood\garnet-s94-paper-vi-exp1-llm-pass1-*`

- [ ] Run focused S94 tests/status gate.
- [ ] Run workspace tests and clippy.
- [ ] Seal a Desktop dogfood bundle with logs and manifest.
- [ ] Open PR `S94: wire Paper VI Exp 1 provider-gated harness`.
- [ ] Merge only after PR body gate and remote checks are green.
