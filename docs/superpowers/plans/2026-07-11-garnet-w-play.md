# Garnet W-PLAY Implementation Plan — the live WASM playground

**Design authority:** `F_Project_Management/GARNET_LAUNCH_CONVERGENCE_DESIGN_2026_07_09.md`
(Workstream 2) + `F_Project_Management/GARNET_WASM_TARGET.md` (S55 path).
**Entry condition:** Truth Lock merged (Tasks 1–5; audit in Task 6).
**Baseline:** post-Truth-Lock `origin/main`.
**Exit evidence (GATE 3):** playground live at a URL; a first-time visitor sees
an authority diff in under 30 seconds, in-browser, nothing to install.

## Global Constraints

- The semantic core is frozen for this workstream: `garnet-check`,
  `garnet-interp`, `garnet-vm`, `garnet-cli` core, and `xtask` behavior may not
  change. W-PLAY **adds** a new crate (`garnet-wasm`) and site assets.
  The only permitted touches inside existing crates are (a) **build-surface**
  feature gates explicitly prescribed by `GARNET_WASM_TARGET.md`
  (miette `fancy` off for wasm), and (b) ONE **additive** interp API — a
  thread-local output-capture sink for `print`/`println`, following the
  RB-7 additive-accessor precedent: when no capture is active the natives
  write to stdout byte-identically (proven by the untouched workspace
  suite); the browser cannot observe process stdout, so without capture
  `run_source` could not return real program output and the playground
  would have to fake it — which is forbidden.
- Every claim ships with its trap: the playground page may not say "runs in
  your browser" until a Playwright test proves a browser executed Garnet
  source and rendered its real output.
- The static-gallery truth note stays until the moment live execution is
  proven, then the page states the new truth (recorded outputs replaced by
  live execution; the gallery examples become live starter programs).
- No tag, release, CI-policy, or launch action; those stay Jon-only.

## Measured starting state (2026-07-11, MacBook Pro lane)

- `rustup` wasm32-unknown-unknown target: installed.
- `wasm-pack` 0.15.0, `wasmtime` 46.0.1: installed.
- `cargo check -p garnet-interp --target wasm32-unknown-unknown` fails ONLY on
  `getrandom` (needs its `js`/wasm feature for browser targets); miette did not
  error before it — the `fancy` gate must still be verified once getrandom is
  resolved.
- `scripts/garnet_wasm_readiness.py` blockers: down to the repo-owned miette
  `fancy` feature-gate item.

### Task 1: `garnet-wasm` crate compiles and runs hello in wasmtime-adjacent smoke

**Files:**
- Create: `garnet-wasm/Cargo.toml`, `garnet-wasm/src/lib.rs`,
  `garnet-wasm/tests/run_source.rs`
- Modify (build-surface only, if required by the target build):
  `garnet-interp-v0.3/Cargo.toml` / `garnet-parser-v0.3/Cargo.toml`
  (feature-gate miette `fancy`), workspace `Cargo.toml` members.

**Steps:**
1. Red test first (native): `run_source("examples/hello.garnet source")`
   returns the same stdout the CLI records (`Hello from Garnet!`).
2. `garnet-wasm` exposes `run_source(src: &str) -> RunResult` (JSON-serializable:
   stdout, diagnostics, exit class) wrapping `garnet_interp::Interpreter`,
   plus `check_source(src) -> CheckResult` and
   `caps_surface(src) -> CapsManifest` for the diff demo.
3. Resolve `getrandom` for wasm32 (its `js`/`wasm_js` feature via the
   `garnet-wasm` crate's dependency table — scoped, not workspace-wide).
4. Feature-gate miette `fancy` so the wasm build carries the minimal renderer;
   native builds keep byte-identical diagnostics (prove with the existing
   workspace suite untouched).
5. `cargo build -p garnet-wasm --target wasm32-unknown-unknown` green;
   `wasm-pack build garnet-wasm` produces the npm-consumable pkg.
6. Full slice gate; PR; merge on green.

### Task 2: the live diff-caps demo (the thesis, touchable)

**Files:**
- Create: `docs/playground/live.js`, wasm pkg assets under
  `docs/playground/pkg/` (built artifact policy: committed only if Pages
  cannot build; document provenance + a rebuild script
  `scripts/build_playground_wasm.sh` with recorded SHA).
- Modify: `docs/playground.html` (live editor panel + capability-diff panel).

**Steps:**
1. Playwright red test: load the page in a clean browser context, type a
   starter program, click run, assert real output rendered.
2. Starter program (from the recorded gallery); an editable function; the user
   adds `fs::read_file` (or `proc`/`net`) → the caps panel diffs
   before/after surfaces and lights the expansion — same verdict semantics as
   `garnet diff-caps` (declared surface only; no bounds-delta claim).
3. The 30-second path: page load → visible starter → one edit → visible
   authority diff. Playwright asserts the full path under a stopwatch budget.
4. Truth swap: remove the static-preview badge ONLY in the same PR that
   proves live execution; page states what runs locally in-browser vs what
   remains CLI-only (`@caps` enforcement at runtime remains the CLI story;
   the playground demonstrates check-time surface diffs truthfully).

### Task 3: readiness + ledger flip with evidence

**Files:**
- Modify: `scripts/garnet_wasm_readiness.py` (blockers resolve),
  `scripts/garnet_playground_readiness.py` (live-execution lane),
  `scripts/garnet_launch_readiness_status.py`
  (`live_wasm_playground`: `remaining` → evidence-backed `pass`),
  regenerate `F_Project_Management/LAUNCH/LAUNCH_READINESS.md`.

**Steps:**
1. Extend reporters red-first: the live lane demands the Playwright proof
   artifact + built module hash before flipping.
2. Regenerate the pinned ledger; `--gate` continues to exit 1 until the
   shelf/S114/foundation rows are green — no over-claim.
3. Cross-OS: the Windows lane (WV-5) re-proves the wasm build + browser path
   from a clean Windows checkout before GATE 3 is called cross-OS-confirmed.

## GATE 3 evidence checklist

- [ ] Playwright proof: clean browser, starter visible, live run output.
- [ ] Authority-diff proof: `fs::read_file` edit lights the diff panel.
- [ ] <30s path measured and recorded.
- [ ] Live at `https://garnet-lang.org/playground.html` (Pages, `main:/docs`).
- [ ] Ledger row flipped with the artifacts named; no unbounded claims.
- [ ] Windows lane confirmation recorded (or explicitly pending, OS-stamped).
