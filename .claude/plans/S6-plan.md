# S6 — Memory Eviction Policy Benchmarks — Implementation Plan

Date: 2026-05-20 (post-v0.5.0 tag)
Contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` § S6
State: not-started → **planned** → in-progress
Reviewer: Jon (Island Development Crew)

> S6 is a v0.5.1-acceptable slice. PR title: `S6: Memory eviction policy benchmarks (per-kind Criterion + status reporter)`.
> Closes half of Paper VI Contribution 3's "production allocator path" gap.

## 1. Scope (in)
- `garnet-memory-v0.3/benches/eviction.rs` (new) — Criterion bench exercising `MemoryPolicy::score` + `should_retain` on a synthetic R+R+I corpus for each of the four Mnemos kinds (working / episodic / semantic / procedural), comparing the policy path against a naive FIFO baseline.
- `garnet-memory-v0.3/Cargo.toml` — adds the `[[bench]] name = "eviction" harness = false` entry.
- `scripts/garnet_memory_eviction_status.py` (new) — deterministic, manifest-backed reporter that inventories the bench file + per-kind coverage (does NOT run `cargo bench`; that's evidence the maintainer captures separately).
- `scripts/test_garnet_memory_eviction_status.py` (new) — three assertions: live-bench-file coverage, CLI Markdown output sanity, CLI JSON round-trip.
- `C_Language_Specification/MEMORY_CORE_ROADMAP.md` — new T4.5 row (Tier 4 tooling) with the ✅ marker.
- New "Memory eviction policy benchmarks" lane in `scripts/garnet_mit_readiness_status.py` (verified 100%).
- Regenerated readiness baseline.
- `CHANGELOG.md` Added entry under `[Unreleased] — v0.5.1 in flight`.
- `.claude/plans/S6-plan.md` planning doc.

## 2. Scope (out)
- **End-to-end store throughput under eviction** — `EpisodeStore`, `VectorIndex`, etc. already have their own bench targets in `benches/vector.rs`. Cross-cutting eviction benches live here so each kind's policy contract is visible in one place; coupling them to specific store backends is a separate slice.
- **Production allocator behaviour** — Tier 1 work in `MEMORY_CORE_ROADMAP.md`, not S6.
- **Memory footprint / allocation count / GC overhead measurement** — Criterion times wall-clock only; allocation profiling is a future Tier 4 add.
- **A fresh `cargo bench` measurement run embedded in the lane** — the reporter inventories the harness; the maintainer captures Criterion numbers as Desktop evidence.

## 3. Honest partials
- "Policy-cost measurement, not production allocator measurement" — phrased verbatim in the lane evidence text.
- "Synthetic items, not real store data" — the bench uses a deterministic R+R+I corpus to avoid coupling to a specific Item struct. Realistic enough to exercise the scoring path; not a substitute for real-workload measurement.
- "Naive baseline is FIFO with same-cost traversal" — the "naive" path still touches every item once so the comparison reflects traversal cost equally; the difference is policy.score work, not memory walk.

## 4. Dogfood block (per contract S6)
```bash
cargo bench -p garnet-memory-v0.3 --bench eviction > /tmp/evict.txt
python3 scripts/garnet_memory_eviction_status.py
# Expect: per-kind numbers committed as evidence artifact
```

Verified locally:
- `cargo bench -p garnet-memory --bench eviction --no-run` compiles the harness clean.
- `python3 scripts/garnet_memory_eviction_status.py` prints `Per-kind coverage complete: True` and shows ✅ for all four kinds × all three columns (bench_fn / naive_baseline / policy_path).
- `python3 scripts/test_garnet_memory_eviction_status.py` — 3 tests pass.

## 5. State-machine transitions
| Transition | Evidence |
|---|---|
| not-started → planned | this file |
| planned → in-progress | draft PR `S6: Memory eviction policy benchmarks` opens |
| in-progress → review-ready | CI green (`cargo bench --no-run` part of release workflows; status reporter test runs in `cargo test --workspace`-adjacent Python suite) |
| review-ready → dogfood-passing | Jon review + CHANGELOG |
| dogfood-passing → merged | squash-merge |

## 6. Risks and mitigations
- **Criterion benches are slow in CI.** Mitigation: the bench is registered via `[[bench]]` so it builds in CI but does NOT run automatically; the maintainer runs it on demand.
- **Synthetic corpus could drift from real workloads.** Mitigation: deterministic seed; the corpus shape is documented in the bench file's header so future revisions stay calibrated.
- **The reporter could pass while the bench is broken.** Mitigation: the reporter checks both the bench file's text patterns AND the Cargo.toml entry; `cargo bench --no-run` validates the bench compiles in CI's normal `cargo test`-adjacent jobs.

## 7. What I need from Jon
None.
