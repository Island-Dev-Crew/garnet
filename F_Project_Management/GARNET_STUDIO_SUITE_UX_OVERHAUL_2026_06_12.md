# Garnet Studio Suite UX Overhaul — design + landing record (2026-06-12)

**Lane:** Windows NUC Claude (Fable 5). **Surfaces touched:** `apps/garnet-studio`
(Tauri shell, frontend, packaging stamps) + its contract tests/docs. **Surfaces
NOT touched:** the five W-REBUILD-frozen crates (`garnet-check`, `garnet-interp`,
`garnet-stdlib`, `garnet-parser`, `garnet-cst`), all gates/CI, all release
assets. Parallel-safe with RB-3+ per W_REBUILD_SPEC §5.

This document is episodic memory: the questions that started the pass, the
honest answers, what shipped, and what is deliberately deferred and to whom.

## 1 · The questions, answered

**"Is Garnet Studio useful?"** Yes — as a local evidence cockpit, and now also
as a usable first surface. Concretely it does five jobs: (1) parse/check/run on
local Garnet files; (2) the deterministic Rust/Ruby/Python/Go→Garnet converter
with lineage evidence — now with an in-app preview of the produced `.garnet`;
(3) the advisory pipeline (assist/bundle/review/handoff) for the non-active
languages; (4) one-click dogfood evidence (objective pulse, agentic matrix,
manifest-sealed bundles); (5) every repo readiness reporter behind buttons. Its
pre-overhaul weakness was not capability but *operability*: no guidance (zero
tooltips), no version truth (0.1.0 stamp), no protection from hung reporters,
and a wall of thirteen reporter buttons greeting every first-time user. The
overhaul addresses exactly those.

**"Did its splash screen hold during launch or installation?"** Honest answer:
**the Tauri shell never had a splash screen** — nothing in the prior
`index.html`/`main.ts`/`tauri.conf.json` implemented one. A remembered splash
almost certainly traces to the retired Electron-era app (the S1–S80 Windows
audit explicitly noted stale Electron popup evidence pointing at a different
package). The NSIS *installer* likewise has no custom splash/sidebar branding.
As of this pass the app **has** a real splash: brand mark, tagline, live boot
status ("Loading preferences… / Checking CLI health…"), and the shell version;
it holds a minimum 700 ms, dismisses only after settings + health resolve, has
a reduced-motion path and a strand-proof removal fallback, and the window
background color now matches the theme so launch cannot white-flash.
Installer-side branding (NSIS sidebar/header imagery) is a deferred cosmetic
follow-up — it needs designed assets, not code.

## 2 · What shipped (this PR)

| Area | Change |
|---|---|
| Version truth | Single stamp in `Cargo.toml` at the workspace release version; `tauri.conf.json` duplicate removed; NSIS artifact becomes `Garnet Studio_0.8.1_x64-setup.exe`; double drift-gate (crate test + shell contract test) |
| Boot | Splash (above) + `backgroundColor` no-flash |
| Modes | Simple (health, parse/check/run, convert, settings) vs Power (everything); persisted; power-only panels hidden, never removed |
| Settings | Per-user JSON (config dir), Rust-side validation/clamping, theme (dark/light/system), command + matrix timeouts |
| Robustness | Timeout + kill for every spawned process (separate matrix budget), `timed_out`/`duration_ms` in results, pipe-deadlock-proof readers, 256 KiB/stream UI cap with full output in evidence |
| Truth tiles | Live `docs/truth.json` values with stamping commit; explicit unavailable state; hand-written "87/87" tile removed |
| Converter UX | In-app `.garnet` output preview via evidence-root-constrained `list_evidence_files`/`read_evidence_text` (canonicalized, capped, traversal-safe) |
| Polish | Tooltips on every control, Ctrl+1…8 / Ctrl+Enter, status bar (versions, mode, copyable evidence root), copy buttons, collapsible long output, focus rings, reduced motion |
| Contracts | `core:default` only (no new permissions), no new npm deps, no provider paths, taxonomy/copy truth intact; AGENTS.md + CHANGELOG + CURRENT_STATE updated; shell contract test extended |

## 3 · The ampcode benchmark, applied honestly

The reference bar (ampcode.com) distills to: **responsive under load, ruthlessly
minimal, consistent across surfaces, clarity over decoration**. Applied here:
hung work can't freeze the surface (timeouts + async commands), the default
surface is minimal (simple mode) with full power one toggle away, the GUI and
CLI tell the same truth (truth.json is the single stats source), and the visual
language stays calm (one accent, monospace for evidence, no ornament).

## 4 · CUI/TUI trajectory — designed, deliberately not built here

The terminal experience of that caliber is a *runway item with an owner*, not a
side effect of a Studio PR:

- **RB-7 (lead lane, post-RB-3):** the REPL on reedline — completion, `?doc`,
  `:caps` live authority budget from `PrimMeta` — is the designed foundation of
  Garnet's interactive terminal experience. Building a TUI now, on this lane,
  would collide with that queued work and duplicate its primitives. The
  W-REBUILD spec already routes the REPL's **cross-OS proof to this NUC lane**
  — that is where this machine picks the thread back up.
- **What the CLI already does well for agents:** deterministic output,
  meaningful exit codes, `diff-caps --machine` (RB-1), evidence-bundle
  manifests. The agent-facing gap list (structured `--format json` on
  parse/check/run, a `garnet version --json`) belongs to the CLI surface after
  RB-3's registry-derived dispatch settles the primitive/metadata story —
  filed as a recommendation, not smuggled into this PR.
- **Bar for the eventual TUI** (from §3): sub-frame input latency, live
  streaming output with the same truncation honesty as the Studio, capability
  budget visible at all times (`:caps` as a persistent gutter, not a command),
  and zero decoration that doesn't carry information.

## 5 · Deferred, with owners

| Item | Owner / when |
|---|---|
| Clean-VM re-proof of the 0.8.1-stamped installer (Sandbox runbook) | NUC lane, next evidence pass — the installer name change invalidates nothing committed (the 0.1.0 proof stays labeled as the 0.1.0 artifact's proof) but the new stamp needs its own |
| Installer (NSIS) branding imagery | needs designed assets; cosmetic |
| Signing / SmartScreen story | Jon-owned, unchanged |
| Marketplace/winget/Linux packaging claims | unchanged; W-SHIP band, prepare-only |
| REPL/TUI | RB-7 lead lane; NUC takes cross-OS proof |
| CLI `--format json` surface | post-RB-3 recommendation |

## 6 · Verification (this PR's ladder)

Crate: fmt + 18 unit tests (settings clamps, timeout kill, payload caps,
traversal rejection, version sync, conf no-second-stamp). Repo: shell contract
test (7), status reporter tests (17), frontend `tsc`+vite build, then the
workspace ladder (fmt/clippy/test/doc) and smoke flags
(`--studio-smoke`, `--studio-domain-proof-smoke`) — results recorded in the PR
body per the dogfood-evidence gate.
