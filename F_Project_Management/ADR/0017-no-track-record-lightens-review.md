# ADR 0017 — No track record or history lightens review

**Status.** Accepted (2026-10-04). Not implemented: the test that cache history
never changes a gate's output, and the retirement of the skip-check strategy, come
in a later trust-kernel PR.

## Context

Integrity rule 1 decides who merges a gate change, and rule 2 sends any widening
of declared authority to a human ruling. Three proposals or mechanisms would let
past results lighten that review:

- The strategy miner in `garnet-cli/src/strategies.rs` proposes
  `skip_check_if_unchanged` after the same source hash succeeds three times. Its
  own notes in `provenance.rs` say such a strategy turns the checker off. Today
  the CLI only prints a note on standard error when one applies.
- A lane that merges changes with no authority expansion without a human.
- A ledger of past agreement between reviewers and verdicts, used to pick that lane.

`CapsDiff::authority_expanded()` reads the program-wide aggregate only. A function
that gains `net` inside a module that already declares it reports no expansion
(`garnet-check-v0.3/src/caps_diff.rs`). The gain is listed in
`functions_caps_expanded`, which is shown to the reviewer.

## Decision

No track record, cache state or history of agreement lightens review. Each may
add scrutiny and none may remove it.

- No lane merges a change with less review than today, until the maintainer rules
  on a predicate under which it may. The predicate must hold that the aggregate
  is unchanged, that `functions_caps_expanded` is empty and that no trust-surface
  path is touched. A red test must prove it, with a per-function gain as the
  failing case.
- The skip-check strategy is retired, or kept only as a note behind a test that
  cache history never changes a gate's output.

## Alternatives rejected

- **A lane keyed on the machine verdict alone.** It would merge a per-function
  gain without review, against rule 2.
- **Keep the strategy as an untested note.** Nothing would then stop a later
  change from wiring it into a skip.
- **Leave this to the lane design.** Both conflicts stay open until then, and the
  lane design would inherit them.
