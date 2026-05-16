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
- per-domain coverage adequacy data for the 3-5 probe bar
- per-probe stdout/stderr logs
- `MANIFEST.sha256`

The current probe set covers 30 checks across ten domains, or 28 checks when
the SwiftUI/macOS app workbench probes are intentionally skipped in headless CI.
Functional readiness and coverage adequacy are intentionally separate: a domain
can pass every current probe while still needing more independent probes before
the suite satisfies the requested 3-5 probe coverage bar.

| Domain | Probe count | Coverage status | Purpose |
|---|---:|---|---|
| Agent orchestration | 4 | adequate | routing, capability policy, expression evaluation, and canonical multi-agent example execution |
| Project scaffolding | 1 | needs expansion | generated `agent-orchestrator` template runs and tests |
| Agent recovery and diagnostics | 4 | adequate | malformed source, missing source, undefined eval symbol, and missing manifest failures stay actionable |
| Safe mode and capabilities | 5 | adequate | valid safe programs, unsafe rejection, and advertised safe I/O example check/run |
| Migration assistant | 5 | adequate | Python/Ruby/Rust/Go conversion plus explicit unsupported-language rejection |
| Release integrity | 3 | adequate | deterministic manifest generation, verification, and tamper rejection |
| Developer experience | 2 | needs expansion | `garnet doc` extraction and `garnet fmt --check` |
| Web/PWA productization | 2 | needs expansion | service-worker offline handler evidence plus full local PWA smoke |
| macOS app workbench | 2 | needs expansion | Garnet Studio self-test plus either SwiftPM XCTest or packaged-app CLI smoke |
| Agent memory and analysis | 2 | needs expansion | advertised log analyzer check/run |

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
/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260515-234617
```

Current result:

```text
readiness=100
passed=30/30
```

The latest bundle also records domain coverage adequacy. It marks agent
orchestration, recovery/diagnostics, safe mode/capabilities, migration, and
release integrity as coverage-adequate; project scaffolding, developer
experience, web/PWA productization, macOS app workbench, and agent
memory/analysis remain `needs-expansion` despite all current probes passing;
web/PWA is now `2/3` after adding the full local PWA smoke.

Manifest verification:

```sh
cd /Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260515-234617
shasum -a 256 -c MANIFEST.sha256
```

All files in the bundle verified at creation time. This bundle was produced by
running the source-checkout matrix with app workbench probes enabled and
copying the completed evidence directory to Desktop dogfood storage. The matrix
now includes the App-9 web/PWA offline service-worker handler probe, the App-13
local PWA readiness smoke probe, and writes `readiness-slice-status.md` plus
`domain_coverage` JSON so completion and coverage adequacy evidence travel
with the bundle.

## Next Gates

1. Keep the headless 27-probe matrix in CI while app workbench probes remain
   covered by local/package/DMG gates.
2. Keep Garnet Studio's "Agentic Tests" surface wired to this same harness for
   both source-checkout and packaged-app runs.
3. Add a lower-cost CI variant if the full matrix remains too expensive for
   every PR.
4. Fill `needs-expansion` domains with focused 3-5 probe coverage slices before
   treating the green score as broad exhaustion evidence.
5. Keep signed/notarized macOS, web/PWA, iOS, Android, and promo-video
   distribution as productization lanes with their own evidence gates.
6. Keep production allocator-integrated ARC, native backend output, mechanized
   proof, and empirical validation separate from this dogfood score until those
   lanes have executable proof.
