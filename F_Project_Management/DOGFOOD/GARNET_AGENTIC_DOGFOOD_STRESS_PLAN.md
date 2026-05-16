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

The current probe set covers 37 checks across ten domains, or 34 checks when
the SwiftUI/macOS app workbench probes are intentionally skipped in headless CI.
Functional readiness and coverage adequacy are intentionally separate: a domain
can pass every current probe while still needing more independent probes before
the suite satisfies the requested 3-5 probe coverage bar.

| Domain | Probe count | Coverage status | Purpose |
|---|---:|---|---|
| Agent orchestration | 4 | adequate | routing, capability policy, expression evaluation, and canonical multi-agent example execution |
| Project scaffolding | 3 | adequate | generated `cli`, `web-api`, and `agent-orchestrator` templates scaffold, run, and test |
| Agent recovery and diagnostics | 4 | adequate | malformed source, missing source, undefined eval symbol, and missing manifest failures stay actionable |
| Safe mode and capabilities | 5 | adequate | valid safe programs, unsafe rejection, and advertised safe I/O example check/run |
| Migration assistant | 6 | adequate | Python/Ruby/Rust/Go conversion, explicit unsupported-language rejection, and converter adoption-status truth |
| Release integrity | 3 | adequate | deterministic manifest generation, verification, and tamper rejection |
| Developer experience | 3 | adequate | `garnet doc` extraction, `garnet fmt --check`, and formatter repair of dirty agent source |
| Web/PWA productization | 3 | adequate | service-worker offline handler, full local PWA smoke, and Chrome DevTools offline navigation |
| macOS app workbench | 3 | adequate | Garnet Studio self-test, SwiftPM smoke against the matrix-built CLI, and SwiftPM XCTest |
| Agent memory and analysis | 3 | adequate | advertised log analyzer parse-time memory declaration analysis plus check/run |

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
/Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-004751
```

Current result:

```text
readiness=100
passed=37/37
```

The latest merged bundle also records domain coverage adequacy. It marks agent
orchestration, recovery/diagnostics, safe mode/capabilities, migration, release
integrity, project scaffolding, developer experience, web/PWA productization,
and agent memory/analysis as coverage-adequate. The App-16 source app smoke
slice promotes macOS app workbench to `3/3` locally by adding a SwiftPM
`--smoke-test` probe that prefers the matrix-built Garnet CLI on `PATH`.

Manifest verification:

```sh
cd /Users/idc2.0/Desktop/dogfood/garnet-agentic-dogfood-20260516-004751
shasum -a 256 -c MANIFEST.sha256
```

All files in the bundle verified at creation time. This bundle was produced by
running the source-checkout matrix with app workbench probes enabled and
copying the completed evidence directory to Desktop dogfood storage. The matrix
now includes the App-9 web/PWA offline service-worker handler probe, the App-13
local PWA readiness smoke probe, the App-15 browser PWA offline probe, and
writes `readiness-slice-status.md` plus `domain_coverage` JSON so completion
and coverage adequacy evidence travel with the bundle.

Latest mounted-DMG install evidence bundle:

```text
/Users/idc2.0/Desktop/dogfood/garnet-studio-dmg-smoke-20260516-005718
```

The package script now runs the DMG smoke with `--copy-to-desktop`. That smoke
mounts the generated DMG, copies `Garnet Studio.app` to a temporary
Applications-style directory, verifies the copied app signature, runs
self-test, bundled CLI version, workbench, packaged PWA offline-handler, and
copied-app agentic matrix probes, then preserves `dmg-smoke-report.md`,
`dmg-smoke-data.env`, per-command logs, packaged PWA JSON, and
`MANIFEST.sha256`. The latest DMG SHA-256 is:

```text
91aeff3b93df1bffeb77524f705f0d9340eef7af845398753c79a1d360aaea46
```

## Next Gates

1. Keep the headless 34-probe matrix in CI while app workbench probes remain
   covered by local/package/DMG gates, with manifest-backed Desktop evidence
   for mounted-DMG copy/install smokes.
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
