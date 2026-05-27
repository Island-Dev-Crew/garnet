# S22 Stdlib and Memory Runtime Completion Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the S21 deferred line by making the remaining S17 Layer-1 stdlib families and `memory::` Mnemos handles runnable from Garnet source.

**Architecture:** Keep S21's qualified-first dispatch model. Add focused bridge trampolines in `garnet-interp-v0.3/src/stdlib_bridge.rs`, only adding new `Value` handle variants where the host primitive requires state (`std::process`). JSON is marshaled into ordinary Garnet values, and `memory::working/episodic/semantic/procedural` return the existing `MemoryStore` value so method dispatch stays centralized.

**Tech Stack:** Rust 2021, `garnet-interp-v0.3`, `garnet-stdlib`, `garnet-memory`, Garnet CLI smoke scripts, Python readiness reporter.

---

## Files

- Modify `F_Project_Management/AGENT_COORDINATION_LEDGER.md`: append-only STARTED/PR/REVIEW/MERGED entries under `win-codex`.
- Create `garnet-interp-v0.3/tests/stdlib_s22_dispatch.rs`: source-level integration tests for JSON/regex/uuid/log/env/process/memory.
- Modify `garnet-parser-v0.3/src/parser.rs` and `garnet-parser-v0.3/src/grammar/expr.rs`: allow selected keywords as path segments only in qualified names, so official APIs such as `std::regex::match`, `std::process::spawn`, and `memory::working` can parse without making those words legal bare identifiers.
- Modify `garnet-interp-v0.3/src/value.rs`: add process handle/status carrier only if tests require it.
- Modify `garnet-interp-v0.3/src/stdlib_bridge.rs`: bind and implement S22 qualified primitives.
- Modify `garnet-interp-v0.3/Cargo.toml`: add direct `serde_json` dependency if JSON marshaling requires it.
- Create `examples/novel_05_s22_stdlib_memory_pipeline.garnet`: runnable proof combining stdlib dispatch and Mnemos handles.
- Modify `scripts/smoke_garnet_novel_compositions.py` and its unittest: include the new example with exact output.
- Modify `scripts/garnet_mit_readiness_status.py` and baseline JSON: add a granular S22 lane.
- Modify `F_Project_Management/GARNET_v0_7_SLICE_DOGFOOD.md`, `CURRENT_STATE.md`, and `CHANGELOG.md`: section-scoped, calibrated-honesty updates.

## Task 1: Red tests for runtime dispatch

- [x] Add `garnet-interp-v0.3/tests/stdlib_s22_dispatch.rs` with tests that call the desired public API from Garnet source:
  - `std::json::{parse,get,set,stringify}`
  - `std::regex::{match,find_all,replace}`
  - deterministic `std::uuid::new_v5`
  - `std::log::info`
  - `std::env::{set,get,vars}`
  - `std::process::{spawn,wait,exit_code}`
  - `memory::{working,episodic,semantic,procedural}` plus existing store methods.
- [x] Run `cargo test -p garnet-interp --test stdlib_s22_dispatch --no-fail-fast`.
- [x] Confirm the failures are missing dispatch/handle failures, not typos.
- [x] Keep the observed red state in view: before implementation, the S22 test file fails on keyword-path parsing for `match`, `spawn`, and `memory::working`, plus unresolved dispatch for `std::env::set`.

## Task 2: Bridge the remaining stdlib families

- [x] Add a parser helper for qualified path segments that accepts `memory`, `working`, `episodic`, `semantic`, `procedural`, `spawn`, and `match` only when they appear in a qualified path.
- [x] Bind the qualified names in `stdlib_bridge::install`.
- [x] Implement JSON conversion between `serde_json::Value` and ordinary `Value`.
- [x] Implement regex, uuid, env, process, and log trampolines through `garnet_stdlib`.
- [x] Add direct bridge unit coverage for names and error paths.
- [x] Run `cargo test -p garnet-interp stdlib_bridge --no-fail-fast` and the S22 integration test.

## Task 3: Add live memory constructors

- [x] Bind `memory::working`, `memory::episodic`, `memory::semantic`, and `memory::procedural`.
- [x] Return existing `Value::MemoryStore` handles backed by `MemoryBackend::for_kind`.
- [x] Prove those handles compose with existing `.push`, `.recent`, `.search`, and `.find` methods from Garnet source.
- [x] Keep OS authority out of these constructors; they are in-memory handles, no new `@caps`.

## Task 4: Dogfood proof and readiness lane

- [x] Add a deterministic `examples/novel_05_s22_stdlib_memory_pipeline.garnet`.
- [x] Extend `scripts/smoke_garnet_novel_compositions.py` and unittest expectations from 4 to 5 cases.
- [x] Add a S22 readiness lane that checks the dispatch code, integration test, example, and harness wiring.
- [x] Update the baseline only after the reporter passes `--check-no-regression`.

## Task 5: Project artifacts and validation

- [x] Update `CHANGELOG.md`, `CURRENT_STATE.md`, and the S22 dogfood block with exact commands and honest non-claims.
- [x] Run focused checks, then the full gates:
  - `cargo fmt --all -- --check`
  - `cargo test -p garnet-interp --test stdlib_s22_dispatch --no-fail-fast`
  - `python3 scripts/smoke_garnet_novel_compositions.py`
  - `python3 scripts/garnet_mit_readiness_status.py --check-no-regression`
  - `cargo test --workspace --no-fail-fast`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
- [ ] Open PR `S22: stdlib and memory runtime completion`, run the Grep Loop to 5/5, and record PR/REVIEW/MERGED state honestly in the ledger.
