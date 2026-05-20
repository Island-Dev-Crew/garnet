# S5 — Parser Fuzz Harness — Implementation Plan

Date: 2026-05-20
Contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` § S5
State: not-started → **planned**
Reviewer: Jon (Island Development Crew)

> S5 is one of the six v0.5.0 release gates. PR title: `S5: Parser fuzz harness`.
> Security tie-in: primary defense against agent-generated-Garnet adversarial corpus.

## 1. Scope (in)
- `garnet-parser-v0.3/fuzz/` cargo-fuzz sub-workspace (separate `[workspace]` block so it does not affect the main workspace).
- `garnet-parser-v0.3/fuzz/Cargo.toml` — `libfuzzer-sys` + path-dep on `garnet-parser`.
- `garnet-parser-v0.3/fuzz/fuzz_targets/parse_input.rs` — wraps every parse call in a strict `ParseBudget`.
- `garnet-parser-v0.3/fuzz/corpus/parse_input/` — seed corpus copied from `examples/*.garnet`.
- `garnet-parser-v0.3/fuzz/AGENTS.md` — contract file with required phrases.
- Updates to root `AGENTS.md` and `scripts/check-agent-contracts.py`.
- `.github/workflows/fuzz-nightly.yml` — nightly + workflow_dispatch job; nightly Rust + cargo-fuzz install + `max_total_time` (default 3600s) + crash artifact upload.
- New "Parser fuzz harness (nightly)" lane in `garnet_mit_readiness_status.py`.
- Regenerated readiness baseline.
- `CHANGELOG.md` Added entry under [Unreleased].

## 2. Scope (out)
- Interpreter or checker fuzz targets — separate slice.
- Differential fuzzing against the archived v0.2 parser — follow-up.
- OSS-Fuzz upstream integration — future slice.
- Coverage-guided corpus minimization beyond the raw `examples/` seed.

## 3. Honest partials
- "ParseBudget is strict by design" — relaxing budgets for "deeper" coverage is an anti-goal.
- "Local validation is structural; real fuzz exercise runs in CI." cargo-fuzz needs nightly Rust + a lengthy install. PR body documents the local pre-merge check is workflow YAML lint + corpus existence + sub-workspace `cargo metadata --no-deps`.
- "1-hour default schedule" — workflow_dispatch can override `max_total_time`.
- "Linux-only runner today" — Windows / macOS fuzz are deferred.

## 4. Dogfood block (per contract S5)
```bash
cd garnet-parser-v0.3/fuzz
cargo +nightly fuzz run parse_input -- -max_total_time=60
# Expect: 0 panics, 0 hangs, memory bounded under default sanitizer limits.
```

## 5. State-machine transitions
| Transition | Evidence |
|---|---|
| not-started → planned | this file |
| planned → in-progress | draft PR `S5: Parser fuzz harness` opens |
| in-progress → review-ready | structural CI passes on PR; nightly exercise after merge |
| review-ready → dogfood-passing | Jon review + CHANGELOG |
| dogfood-passing → merged | squash-merge |

## 6. Risks and mitigations
- cargo-fuzz install adds CI setup time → mitigated by `actions/cache` for `~/.cargo`.
- Sub-workspace gotcha: fuzz crate is detached from main workspace; `cargo test --workspace` does NOT exercise it. This is intentional — crashes surface via the dedicated workflow, not the main CI.
- Crash triage is human-required. AGENTS.md forbids quietly deleting corpus inputs.
- 1-hour schedule may miss deeply-nested bugs → `workflow_dispatch` allows long runs.

## 7. What I need from Jon
None for S5 specifically. PR body flags the inherited local-only `garnet-vm/AGENTS.md` complaint from a parallel agent's S2 prep (not on S5's branch, so CI is unaffected).
