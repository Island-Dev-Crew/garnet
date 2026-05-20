# S4 — Formatter Idempotent Baseline — Implementation Plan

Date: 2026-05-20 (post-v0.5.0 tag)
Contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` § S4
State: not-started → **planned** → in-progress
Reviewer: Jon (Island Development Crew)

> S4 is a v0.5.1-acceptable slice. PR title: `S4: Formatter idempotent baseline (workspace-test-enforced)`.

## 1. Why this slice can land now without CST work

The S4 contract goal is "deterministic, idempotent source formatter." The existing v0.4.2 `garnet fmt` already meets that contract literally — it normalizes whitespace + line endings + final newline, then re-parses to refuse any byte change that would break the source. The local dogfood block (from the contract) passes today on all 13 canonical examples.

What v0.4.2 did NOT have is a workspace-test that enforces idempotency on every push. S4 lands that test plus the readiness lane that surfaces the gate.

## 2. Scope (in)
- `garnet-cli/tests/fmt_idempotency.rs` (new, ~125 LOC) — two integration tests:
  - `canonical_examples_are_idempotent_under_fmt` — runs `garnet fmt --stdout` on every `examples/{mvp_,det_}*.garnet`, re-runs it on the output, asserts byte-identical.
  - `formatter_is_deterministic_within_run` — three runs on the same input produce identical bytes (catches non-determinism from e.g. unstable HashMap iteration).
- New "Formatter idempotent baseline" lane in `scripts/garnet_mit_readiness_status.py` (verified 100% when both `fmt.rs` and the test exist).
- Regenerated readiness baseline.
- `CHANGELOG.md` Added entry under a new `[Unreleased] — v0.5.1 in flight` section.
- `.claude/plans/S4-plan.md` planning doc.

## 3. Scope (out)
- AST-driven semantic formatting (alignment, spacing rules, import sorting). Requires a trivia-preserving CST in `garnet-parser-v0.3`. Tracked as a separate slice.
- Comment-preserving round-trip. Parser currently drops trivia.
- Pretty-printer for malformed input recovery.
- Workspace-level `garnet fmt --workspace` (each file at a time today).

## 4. Honest partials
- "Whitespace + line-ending + terminal-newline normalization only" — the current `cmd/fmt.rs` doc header already says this; nothing softened.
- "Canonical corpus is the gate, not arbitrary input" — S5 parser fuzz harness is the unbounded-input gate. S4 covers the 13 committed examples.

## 5. Dogfood block (per contract S4)
```bash
for f in examples/*.garnet; do
  garnet fmt --stdout "$f" > /tmp/once
  garnet fmt --stdout /tmp/once > /tmp/twice
  diff /tmp/once /tmp/twice || { echo "fmt not idempotent: $f"; exit 1; }
done
```

Verified locally on all 13 canonical examples and via the new `cargo test -p garnet-cli --test fmt_idempotency` (2 tests passing).

## 6. State-machine transitions
| Transition | Evidence |
|---|---|
| not-started → planned | this file |
| planned → in-progress | draft PR `S4: Formatter idempotent baseline` opens |
| in-progress → review-ready | CI green (workspace tests include `fmt_idempotency`) |
| review-ready → dogfood-passing | Jon review + CHANGELOG |
| dogfood-passing → merged | squash-merge |

## 7. Risks and mitigations
- **Test depends on `target/debug/garnet` binary.** The test calls `cargo build -p garnet-cli` as a fallback so first-time CI runs don't fail on missing binary. Mitigation: explicit `ensure_binary_built` helper.
- **Corpus could grow without the test noticing.** Mitigation: the test enumerates `examples/{mvp_,det_}*.garnet` dynamically — adding a new fixture automatically extends coverage.
- **Future formatter changes could regress idempotency silently.** Mitigation: this test runs in workspace `cargo test`, so every PR runs it.

## 8. What I need from Jon
None.
