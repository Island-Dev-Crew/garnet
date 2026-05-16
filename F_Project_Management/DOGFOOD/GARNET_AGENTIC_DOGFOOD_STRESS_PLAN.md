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

The current probe set covers 28 checks across nine domains, or 26 checks when
the SwiftUI/macOS app workbench probes are intentionally skipped in headless CI:

| Domain | Probe count | Purpose |
|---|---:|---|
| Agent orchestration | 4 | routing, capability policy, expression evaluation, and canonical multi-agent example execution |
| Project scaffolding | 1 | generated `agent-orchestrator` template runs and tests |
| Agent recovery and diagnostics | 4 | malformed source, missing source, undefined eval symbol, and missing manifest failures stay actionable |
| Safe mode and capabilities | 5 | valid safe programs, unsafe rejection, and advertised safe I/O example check/run |
| Migration assistant | 5 | Python/Ruby/Rust/Go conversion plus explicit unsupported-language rejection |
| Release integrity | 3 | deterministic manifest generation, verification, and tamper rejection |
| Developer experience | 2 | `garnet doc` extraction and `garnet fmt --check` |
| macOS app workbench | 2 | Garnet Studio self-test plus either SwiftPM XCTest or packaged-app CLI smoke |
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

Latest source-checkout evidence bundle:

```text
/Users/idc2.0/Desktop/dogfood/agentic-dogfood-recovery-domain-full
```

Current result:

```text
readiness=100
passed=28/28
```

Manifest verification:

```sh
cd /Users/idc2.0/Desktop/dogfood/agentic-dogfood-recovery-domain-full
shasum -a 256 -c MANIFEST.sha256
```

All files in the bundle verified at creation time. This bundle was produced by
running the source-checkout matrix with app workbench probes enabled and
copying the completed evidence directory to Desktop dogfood storage. The matrix
now also writes `readiness-slice-status.md` so completion percentage evidence
travels with the bundle.

## Next Gates

1. Keep the headless 26-probe matrix in CI while app workbench probes remain
   covered by local/package/DMG gates.
2. Keep Garnet Studio's "Agentic Tests" surface wired to this same harness for
   both source-checkout and packaged-app runs.
3. Add a lower-cost CI variant if the full matrix remains too expensive for
   every PR.
4. Keep signed/notarized macOS, web/PWA, iOS, Android, and promo-video
   distribution as productization lanes with their own evidence gates.
5. Keep production allocator-integrated ARC, native backend output, mechanized
   proof, and empirical validation separate from this dogfood score until those
   lanes have executable proof.
