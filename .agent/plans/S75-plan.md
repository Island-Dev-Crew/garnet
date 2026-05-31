# S75 — formal-verification feasibility

## Goal
Assess (not build) whether Garnet can offer a *provable* termination /
`@caps`-soundness story over a safe subset — the eBPF-verifier-path bet from the
compass trajectory research.

## What ships
- `C_Language_Specification/GARNET_FORMAL_VERIFICATION_FEASIBILITY.md` — the study:
  - foundations in tree (explosive.rs static identification + undecidability
    stance; the S74 safe subset; @bounded/default-ceiling policy);
  - feasibility by target: (a) bounded-loop termination (eBPF path) = feasible for
    a restricted subset; (b) @caps soundness = feasible only atop the S74 linear
    mode; (c) mechanized metatheory = research artifact, out of near-term scope;
  - decidability boundary (halting problem → restrict to a checkable subset);
  - verdict + recommendation (verified bounded-loop checker first; @caps after the
    linear mode; whole-language verification not feasible / not the goal).
- `scripts/garnet_formal_verification_feasibility.py` (+ `--gate`) — static
  anti-overclaim gate: study present + anchored + the cited foundation real.
- `scripts/test_garnet_formal_verification_feasibility.py` — 5 unit tests.
- CI agent-contracts (static gate + test); CHANGELOG; contract S75 block; this
  plan; ledger `s74 → merged`.

## Verification
- `python3 scripts/test_garnet_formal_verification_feasibility.py` → 5 OK.
- `garnet_formal_verification_feasibility.py --gate` → rc 0.
- Ladder: fmt/diff clean; `cargo test --workspace` 0 failed (no Rust changed).

## Honest scope (do not soften)
A feasibility STUDY only. No verifier, no termination proof, no SMT/proof-assistant
integration, no soundness theorem. Assessment, not implemented behavior.
