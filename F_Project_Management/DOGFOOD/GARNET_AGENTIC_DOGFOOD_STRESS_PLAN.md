# Garnet Agentic Dogfood Stress Plan

Date: 2026-05-15

This plan captures the first agent-facing stress matrix for Garnet. It is meant
to answer a sharper question than the canonical examples answer: can an agent use
Garnet across orchestration, migration, safe-mode, documentation, release, app,
and memory-analysis workflows without discovering a mismatch between advertised
behavior and executable behavior?

## Scope

The matrix lives in `scripts/run_agentic_dogfood_matrix.py` and emits a
falsifiable dogfood bundle with:

- `dogfood-readiness-data.json`
- `dogfood-readiness-matrix.md`
- `dogfood-readiness-findings.md`
- `dogfood-readiness-report.md`
- `dogfood-readiness-mutations.md`
- `dogfood-readiness-slide-deck.html`
- per-probe stdout/stderr logs
- `MANIFEST.sha256`

The current probe set covers 24 checks across eight domains:

| Domain | Probe count | Purpose |
|---|---:|---|
| Agent orchestration | 4 | routing, capability policy, expression evaluation, and canonical multi-agent example execution |
| Project scaffolding | 1 | generated `agent-orchestrator` template runs and tests |
| Safe mode and capabilities | 5 | valid safe programs, unsafe rejection, and advertised safe I/O example check/run |
| Migration assistant | 5 | Python/Ruby/Rust/Go conversion plus explicit unsupported-language rejection |
| Release integrity | 3 | deterministic manifest generation, verification, and tamper rejection |
| Developer experience | 2 | `garnet doc` extraction and `garnet fmt --check` |
| macOS app workbench | 2 | Garnet Studio self-test and XCTest workflow |
| Agent memory and analysis | 2 | advertised log analyzer check/run |

## Falsification History

The first full matrix run exposed six actionable gaps instead of hiding them:

- `examples/safe_io_layer.garnet` used `starts_with?` while the current runtime
  exposes `starts_with`.
- `examples/safe_io_layer.garnet` needed an explicit `@caps()` boundary for the
  runnable entrypoint and needed to unwrap propagated safe-mode errors before
  describing them.
- `examples/agentic_log_analyzer.garnet` used the same stale method spelling.
- `examples/agentic_log_analyzer.garnet` contained a recursive `to_f` stub that
  overflowed at runtime.
- `garnet convert --from <unknown>` could silently fall back to file-extension
  inference instead of honoring the explicit unsupported source language.
- `garnet doc` could not parse `///` documentation comments and missed docs
  placed before annotations.

Those failures became focused fixes and regression tests in this slice.

## Current Evidence

Latest local evidence bundle:

```text
/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260515-205403
```

Current result:

```text
readiness=100
passed=24/24
```

Manifest verification:

```sh
cd /Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260515-205403
shasum -a 256 -c MANIFEST.sha256
```

All files in the bundle verified at creation time.

## Next Gates

1. Add the matrix to CI once its runtime cost is acceptable for routine PR
   checks.
2. Add a Garnet Studio "Agentic Stress Tests" view or command button that runs
   the same harness and opens the latest bundle.
3. Keep signed/notarized macOS, web/PWA, iOS, Android, and promo-video
   distribution as productization lanes with their own evidence gates.
4. Keep production allocator-integrated ARC, native backend output, mechanized
   proof, and empirical validation separate from this dogfood score until those
   lanes have executable proof.
