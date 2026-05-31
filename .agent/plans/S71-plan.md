# S71 — Paper VI Exp 3 actual run (honest)

## Goal
Open the v0.8 validation runway (S71–S80) by actually running Paper VI Experiment
3 (compiler-as-agent time-to-fix, hypothesis h₃) — honestly, within the
determinism + no-faked-measurement doctrine.

## Why honest-partial
- h₃a is a **timing** speedup (machine-dependent); the determinism doctrine
  forbids inventing measurements, so it is NOT re-measured here.
- The provider-backed re-run (and the `--llm` lane the scripts reference) needs an
  LLM provider / API credits — **pending-infra**, same boundary as Exp 1.
- The recorded v4.0 outcome already exists: h₃a partial (6.5%, CI [3.1%, 9.8%],
  below 10%), h₃b/h₃c pass. S71 surfaces it verbatim and runs the part that CAN
  run provider-free.

## Deliverables
- `scripts/garnet_paper_vi_exp3_status.py` — inventory + provider-free harness run
  (both lanes harness-only + aggregate) + verbatim recorded outcome; `--gate`.
- `scripts/test_garnet_paper_vi_exp3_status.py` — 6 unit tests.
- `F_Project_Management/GARNET_PAPER_VI_EXP3.md` — the experiment + honest scope.
- CI wiring (test + gate); `.gitignore` the harness `out/`; CHANGELOG; contract
  S71 block; this plan; ledger `s70 → merged`.

## Verification
- `python3 scripts/test_garnet_paper_vi_exp3_status.py` → 6 OK.
- `garnet_paper_vi_exp3_status.py --gate` → rc 0 (harness ships + provider-free run).
- Ladder: fmt/diff clean; `cargo test --workspace` 0 failed (no Rust changed).

## Honest scope (do not soften)
h₃a NOT re-measured (machine-dependent); no LLM called; provider-backed re-run
pending-infra. Recorded 6.5% partial stands; 10% claim downgraded honestly;
verbatim §C3 anchor surfaced. Status/run-verification layer, not a new measurement.
