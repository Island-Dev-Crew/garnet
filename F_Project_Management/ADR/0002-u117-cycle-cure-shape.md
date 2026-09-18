# ADR 0002 — Cure the capability propagator's cycle blind spot with an iterative SCC pass

**Status.** Accepted (2026-09-15), shape refined 2026-09-17. Not implemented:
no branch carries the cure. The finding is U-117 in
`F_Project_Management/W_TRUST/LANDING_ARC_5_REGISTER_SWEEP_2026-09-04.md`.

## Context

`garnet check` propagates capability requirements along named call edges. The
propagator returns an empty capability set for a function it is already visiting
and then memoises that answer, so a capability-bearing primitive reached only
through a call-graph cycle is not reported. Because roots are visited in name
order, whether the defect appears depends on what the functions are called: one
register program reports nothing and passes `garnet verify` 5/5, and the same
program with two functions renamed reports two errors.

`C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md` already states
this under "may not say", so the public copy is not wrong today. What is missing
is the fix.

The same traversal overflows the stack at roughly 8,400 chained functions.

## Decision

Replace the recursive walk in `transitive_caps`
(`garnet-check-v0.3/src/caps_graph.rs`) with an iterative DeRemer–Pennello
digraph algorithm over Tarjan strongly connected components, so every function
in a cycle receives the union of the cycle's capabilities. The deep-chain crash
is fixed by the same change, since the iterative form has no recursion to
overflow.

The cure lands with:

- red-first tests in a digest-bound sibling file, a CLI test, a test required by
  the verify gate, and a bounded-time dense-cycle test;
- a fixed-point reachability oracle as the evidence that the change only adds
  diagnostics and removes none, rather than a count of before-and-after runs;
- a rebuilt `docs/playground/pkg` with a re-captured browser proof, because the
  wasm readiness gate hashes `caps_graph.rs` and the checker tests;
- a CHANGELOG entry and a release note that disclose the behavior change, since
  programs that passed `check` will now fail it;
- a correction appended to the register, and a census update in the sweep record.

Only sentences that the cure makes false move with it. In particular the
landing page's "named, acyclic" wording is corrected in the later public-truth
change, not here, and this pull request writes no landed marker.

## Alternatives rejected

- **Leave it and rely on the scope document.** The document keeps the public
  copy accurate, but the defect makes a real program's missing capability
  invisible, and whether it is invisible depends on identifier names. That is
  the worst failure shape for a checker.
- **Memoise nothing.** It removes the wrong answer and replaces it with
  exponential re-traversal on the graphs where the cycle problem actually
  appears.
- **Keep the recursive walk and raise the stack size.** It moves the crash
  threshold and leaves the cycle blind spot untouched.
- **Fold the fix into the wider checker-gap work (funding goal 5).** The cycle
  case is separable, has a red test today, and blocks a public sentence. The
  remaining gaps — closures, function values, initializers, `method_missing`,
  and unannotated bodies — stay in that larger arc.
