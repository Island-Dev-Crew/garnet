# S10 — Compiler-as-Agent Advisory Mode — Implementation Plan

Date: 2026-05-20
Contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` § S10
State: not-started → **planned**
Reviewer: Jon (Island Development Crew)

> S10 is one of the six v0.5.0 release gates. PR title: `S10: Compiler advisory mode (rules-based, no LLM)`.
> Closes Paper VI Contribution 7 surface for the rules-based tier; the LLM tier remains pending-infra.

## 1. Scope (in)
- `garnet-check-v0.3/src/suggest.rs` (new) — deterministic, no-LLM suggestion engine with three rules:
  1. `ManagedFnMissingCaps` — managed `def` without `@caps(...)` annotation.
  2. `LongParameterList` — function with ≥ 4 parameters.
  3. `EmptyFunctionBody` — function with no statements and no tail expression.
- `garnet-cli/src/cmd/check.rs` — adds `--suggest` flag handling.
- `garnet-cli/src/bin/garnet.rs` — dispatches `garnet check --suggest <file>`.
- `garnet-check-v0.3/tests/suggest_corpus.rs` — corpus test: 4 assertions including "≥ 3 rules fire" and "each fixture triggers its target rule."
- `garnet-check-v0.3/tests/suggest_corpus_fixtures/*.garnet` — 3 fixtures, one per rule.
- New "Compiler advisory mode (rules-based)" lane in `scripts/garnet_mit_readiness_status.py`, verified 100% when both `suggest.rs` and `suggest_corpus.rs` exist.
- Updated test in `scripts/test_garnet_mit_readiness_status.py` asserting the new lane + deferred LLM tier.
- Regenerated readiness baseline.
- `CHANGELOG.md` Added entry under [Unreleased].

## 2. Scope (out)
- LLM-derived suggestions. The contract explicitly separates the rules-based tier (this slice) from the LLM tier (pending-infra; depends on Paper VI Exp 1 budget).
- Auto-apply / quick-fix wiring into LSP code-actions. Meshes naturally with S1 follow-ups; out of S10.
- Cross-module suggestions. Current rules are intra-module only.
- Configurable rule severity per project (TOML / config).
- Suggestion ranking or de-duplication beyond the natural rule-fire order.

## 3. Honest partials
- "Three rules today" — the contract requires "at least 3 detectable patterns." We ship exactly 3. The framework is structured for adding more (one `inspect_*` function + one fixture + one corpus assertion), but S10 itself does not expand beyond the contracted minimum.
- "All rules are AST-only" — no type-flow analysis, no data-flow analysis. The rules trigger on syntactic shape only. A pattern like "unused parameter" needs a binding-use analysis that the current AST walk does not perform.
- "Suggestions are advisory, not errors" — `garnet check --suggest` still exits 0 on a clean program; the suggestions print in addition to (not in place of) existing diagnostics.
- "LLM tier remains pending-infra" — repeated in the CHANGELOG entry, the MIT-readiness lane's `deferred` field, and this plan to preserve the calibrated honesty anchor.

## 4. Dogfood block (per contract S10)
```bash
garnet check --suggest examples/mvp_03_*.garnet | grep -q "compiler suggested"
cargo test -p garnet-check-v0.3 --test suggest_corpus
# Expect: at least 3 detectable patterns produce suggestions on the corpus
```

Verified locally: `garnet check --suggest examples/mvp_03_compiler_bootstrap.garnet` produces 3 `compiler suggested:` advisories on the managed `def` functions in that file. `cargo test -p garnet-check --test suggest_corpus` passes 4 assertions including the "≥ 3 rules" gate.

## 5. State-machine transitions
| Transition | Evidence |
|---|---|
| not-started → planned | this file |
| planned → in-progress | draft PR `S10: Compiler advisory mode (rules-based, no LLM)` opens |
| in-progress → review-ready | CI green; suggest_corpus test runs in `cargo test --workspace` |
| review-ready → dogfood-passing | Jon review + CHANGELOG |
| dogfood-passing → merged | squash-merge |

## 6. Risks and mitigations
- **False positives** — the `ManagedFnMissingCaps` rule fires on every managed `def` without `@caps(...)`, which is most managed code today. Mitigation: rule is named `compiler suggested` (not an error); does not block CI; opt-in via `--suggest`.
- **Bikeshedding on which rules ship** — three rules are intentionally conservative. Adding more is a future slice with its own review.
- **`Rule::ManagedFnMissingCaps` may noise out review** for files with many `def`s. Mitigation: output is one line per suggestion; reviewer can filter by rule id.

## 7. What I need from Jon
None for S10. The slice is self-contained Rust + tests + a CLI flag.
