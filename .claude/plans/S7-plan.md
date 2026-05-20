# S7 — Actor OS-Thread Bridge — Implementation Plan

Date: 2026-05-20 (post-v0.5.0 tag)
Contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` § S7
State: not-started → **planned** → in-progress
Reviewer: Jon (Island Development Crew)

> S7 is a v0.5.1-acceptable slice. PR title: `S7: Actor OS-thread bridge (`trust-report` + 3-thread fixture)`.
> Closes Paper VI Contribution 4's CLI-bridge surface gap (the runtime side already exists).

## 1. Why this slice is mostly a CLI bridge

`garnet-actor-runtime/src/runtime.rs` already spawns one OS thread per actor (`std::thread::spawn` per call to `Runtime::spawn`); the header documents the "Spawn-and-mailbox runtime: each actor gets one OS thread plus a mpsc" contract. CURRENT_STATE.md describes the gap as "managed source bridge active, full OS-thread CLI bridge staged" — meaning the runtime side works, but the CLI doesn't yet surface what it does. S7 lands that CLI bridge.

## 2. Scope (in)
- `examples/agent_orchestrator_3thread.garnet` (new) — three `actor` declarations (`Researcher`, `Synthesizer`, `Reviewer`) plus `@caps() def main()`. Parses, checks, and runs under the managed-mode interpreter.
- `garnet-cli/src/cmd/trust_report.rs` (~180 LOC, new) — implements `garnet trust-report <file>`:
  - Parses the source.
  - Walks AST, counts `Item::Actor`, `Item::Fn`, per-fn `@caps(...)` set.
  - Prints the literal `actors: N / threads: N` line that the contract's dogfood greps for.
  - Honest-scope footer: structural-only, not a live-runtime measurement.
  - Three inline unit tests cover the count + caps surface.
- `garnet-cli/tests/trust_report.rs` (new) — integration test asserts the dogfood block against the new example. Two tests: literal-line presence + per-actor enumeration.
- `garnet-cli/src/cmd/mod.rs` + `bin/garnet.rs` — register + dispatch `garnet trust-report`.
- New "Actor OS-thread bridge (`trust-report`)" lane in `scripts/garnet_mit_readiness_status.py` (verified 100%).
- Regenerated readiness baseline.
- `CHANGELOG.md` Added entry under `[Unreleased] — v0.5.1 in flight`.
- `.claude/plans/S7-plan.md` planning doc.

## 3. Scope (out)
- **Live-runtime instrumentation.** The report is STRUCTURAL — it counts `actor` declarations from the AST. It does NOT spawn the runtime or measure actual thread counts. The 1:1 actor:thread mapping follows from the runtime's existing contract, not from a live measurement.
- **Mailbox-size audit.** The runtime supports bounded mailboxes (`spawn_with_capacity`); the report does NOT verify capacity choices.
- **Sendable-boundary verification.** `garnet check` already enforces `@nonsendable` rejection at managed-mode boundaries; the report does NOT re-verify.
- **Transitive caps aggregation from `use` imports.** Today imports resolve to stdlib only; S3's vendored deps don't yet feed into the resolver. The report sums per-fn caps, not transitive.
- **Cross-actor message-graph visualization.** A separate slice can add this.

## 4. Honest partials (in code, lane, CHANGELOG, plan)
- Source-file doc header in `trust_report.rs` lists every "what it does NOT do" line.
- `run()` output ends with a "Honest scope:" footer that says "structural (AST-derived); does not spawn the runtime, measure live thread counts, or audit mailbox sizes."
- The MIT readiness lane's `deferred` field repeats them.
- Manifest + plan + CHANGELOG say the same.

## 5. Dogfood block (per contract S7)

```bash
garnet run examples/agent_orchestrator_3thread.garnet
garnet trust-report examples/agent_orchestrator_3thread.garnet \
  | grep -q "actors: 3 / threads: 3"
```

Verified locally:
- `garnet parse examples/agent_orchestrator_3thread.garnet` → 4 items (main + 3 actors).
- `garnet check ...` → `1 functions checked, 3 boundary call sites, 0 diagnostics`.
- `garnet run ...` → prints the example's intro lines and `=> 3`.
- `garnet trust-report ... | grep -q "actors: 3 / threads: 3"` → exit 0 (literal present).
- `cargo test -p garnet-cli --lib cmd::trust_report` → 3/3 pass.
- `cargo test -p garnet-cli --test trust_report` → 2/2 pass.
- `cargo clippy -p garnet-cli --all-targets -- -D warnings` → clean.
- `cargo fmt -p garnet-cli -- --check` → clean.

## 6. State-machine transitions
| Transition | Evidence |
|---|---|
| not-started → planned | this file |
| planned → in-progress | draft PR `S7: Actor OS-thread bridge ...` opens |
| in-progress → review-ready | CI green; workspace tests include the new integration test |
| review-ready → dogfood-passing | Jon review + CHANGELOG + spec |
| dogfood-passing → merged | squash-merge |

## 7. Risks and mitigations
- **Concurrency with S3's lane.** Both this slice and S3 add a lane to `garnet_mit_readiness_status.py`. Whoever merges first sets the baseline; the second slice will need to regenerate the baseline at merge time. Mitigation: each branch's baseline is correct vs its own commit base; CI's `--check-no-regression` catches regression but tolerates new lanes.
- **Live-vs-structural framing.** Someone could read `actors: 3 / threads: 3` as a live measurement. Mitigation: the immediately-next line is `(one OS thread + mpsc mailbox per actor per actor-runtime/src/runtime.rs)`, and the report ends with `Honest scope: structural (AST-derived); ...`. Both the lane evidence and the CHANGELOG entry repeat the structural framing.

## 8. What I need from Jon
None.
