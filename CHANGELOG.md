# Changelog

All notable changes to Garnet are recorded here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This file is updated in the same PR as the work it tracks (per the v0.5 slice
contract). Lines added here are part of the calibrated-honesty record — if a
slice ships labeled "partial," its CHANGELOG entry says so explicitly.

## Unreleased — W-PLAY Task 1: Garnet runs in WebAssembly (2026-07-11)

### `garnet-wasm` (new crate) + `garnet-interp` additive output capture

- **Added** the `garnet-wasm` crate: `run_source(src)` loads Garnet source
  under `main`'s `@caps` entry frame (the CLI run lane's authority gate),
  invokes `main`, and returns a `garnet.wasm.run/1` JSON result carrying the
  REAL captured program output and diagnostic — never a synthesized result.
  Builds green for `wasm32-unknown-unknown`; `wasm-pack` produces the
  browser module; proven end-to-end by a Node smoke executing the canonical
  hello in actual wasm (`Hello from Garnet!` captured) and by an
  authority-failure path (`proc::run` under `@caps()` fails closed — the
  proc/fs/net natives are absent from the wasm interpreter environment).
- **Added** `garnet_interp::output` — a thread-local output-capture sink for
  `print`/`println` (RB-7-style additive API): when no capture is active the
  natives keep their byte-identical stdout path, proven by the untouched
  workspace suite; a browser cannot observe process stdout, so without this
  the playground could not show real output.
- **Honest boundaries:** no live playground page yet (that is the next
  W-PLAY slice, with a Playwright trap before any "runs in your browser"
  claim); `wasm-opt` is disabled (unoptimized module, size revisited in the
  page slice); the wasm surface exposes `run_source` only — check/diff-caps
  surfaces land with the diff-caps demo slice.

## Unreleased — Studio Agent-Loop Console (Phase 5) (2026-06-28)

### Studio — Agent-Loop Console (`apps/garnet-studio`)

- **Added** a power-only "Agent-Loop Console" panel that renders an existing
  `garnet agent-loop --record-dir` dossier as a **four-gate pipeline** —
  `check → diff-caps → run → seal`. It **reads** a record directory from disk and
  renders the CLI's own verdict (`decision.md` + the trust artifacts) **verbatim**;
  it does **not** run agent-loop (which would execute the proposal) and never
  recomputes the decision.
- **Verdict read, never recomputed.** `studio_agent_loop_dossier` parses the
  verdict from `decision.md`'s heading (ACCEPTED / REJECTED-at-gate) and derives
  each gate's pass / reject / not-reached status from that verdict — not from which
  artifacts happen to exist. A contradiction test (ACCEPTED heading with no seal,
  and a REJECTED heading with a stray seal) pins the heading as the sole authority.
- **Honest seal surface.** The Seal gate reads "sealed (seal.json)" only when a seal
  was actually parsed; an accepted-but-seal-missing dossier shows the gate as
  not-reached and the seal panel says "seal.json missing or unparseable" rather than
  falsely claiming a seal or mis-stating the verdict. On a genuine reject, no seal is
  shown — the negative proof.
- **Structured dossier views, kept separate.** The authority (diff-caps) gate is a
  distinct drill-down (the verbatim `diff_caps.txt` band line + the
  `garnet-capability-manifest-v1` capability manifest); the transparency-log
  (caps-log) chain and the seal **provenance** are their own panels. The seal panel is
  labeled *autonomous acceptance — not a human approval*, keeping approval, widening,
  and seal provenance visibly separate. Note: the record-dir persists no
  `diff-caps --machine` JSON, so the authority drill-down renders the record-dir's
  verbatim banner (and re-running diff-caps is forbidden by the read-only contract)
  rather than re-using the Phase-2 machine-JSON card.
- **Honest scope.** `decision.md` is rendered verbatim, carrying the CLI's disclaimer:
  acceptance is "on capability + depth evidence" only (`@caps` + `@max_depth` are the
  enforced ceilings) — never a claim of full boundedness or safety; the agent is
  `simulated`, not a live LLM. The Studio never adds a "safe"/"bounded" claim of its own.
- **Tests.** 10 Rust tests (accept all-pass + every artifact parsed; reject-widen stops
  at diff-caps with no seal; reject-overdepth stops at run and surfaces the
  `@max_depth` trap; check-reject first-gate-only; unknown-rejection overstates no gate;
  missing decision.md → not a dossier; malformed seal / log / manifest degrade without
  panic; verdict-driven-by-decision contradiction; and disk-gated reads of the real
  `accept`, `reject-widen`, and `reject-overdepth` fixtures) + 9 Playwright renderer
  tests (verdict, gate order, reject pipelines, accepted-but-no-seal copy, wildcard
  widening warning, escaping). Reviewed via a 7-pass Judge+Auditor loop. Local ladder
  green on `NUCBOX_M2PRO_S`: studio crate 60/60, e2e 47/47, clippy clean, shell + status
  contracts, build + `--studio-smoke`. Scope: `apps/garnet-studio` (non-frozen),
  read-only fs — no scripts / CI / gate change, no frozen crate, no capability widening
  (`core:default`), no new crate dependency. Research-grade prototype (v0.x.x).

## Unreleased — Studio Enforced / Declared Legend (Phase 4) (2026-06-28)

### Studio — Enforced / Declared Legend (`apps/garnet-studio`)

- **Added** a power-only "Enforced / Declared Legend" panel that makes Garnet's
  calibrated honesty visible: which capability fences the runtime actually
  **enforces** (`@caps`, `@max_depth`), which are only **declared** (`@bounded`
  Wasmtime-fuel-only, `@mailbox`, `memory`, `time`), and which are platform-**deferred**
  (macOS/Windows OS-sandbox; seccomp is Linux-only).
- **Generated from source/CLI truth, not hand-written.** The status comes from a typed
  Rust fence catalog (the single source of truth — it mirrors the parser `Annotation`
  set and the named-deferred fence list) rendered by the pure `src/enforcement-legend.ts`;
  no enforced/declared status is spelled into markup.
- **Enforced only where the trap evidence holds.** For the two enforced fences a live
  `studio_enforcement_legend` runs a `garnet check --format json` **probe** (reusing the
  velocity plumbing — check-only, ephemeral, no seal) that re-confirms, this run, that the
  static gate still fires: an undeclared-`fs` call → `check.caps_coverage`; an
  out-of-range `@max_depth(100)` → `check.annotation_error`. The green "confirmed live
  this run" badge renders **only** when the probe actually ran AND reproduced the code;
  a ran-but-unconfirmed or no-CLI probe renders an honest "NOT confirmed" / "not probed",
  never a faked confirmation (the Phase-3 false-green discipline). Each probe's expected
  code is read **from** the catalog row, so the displayed code and the re-confirmed code
  cannot diverge.
- **Runtime trap labeled as attested, not re-run.** The probe confirms the *static* gate
  only; the runtime trap (`require_capability` host-authority denial; the recursion trap
  at N+1) is shown as **attested** (S99 / S100 / red-team) and explicitly *not re-run by
  this probe* — the two are never conflated.
- **Lazy, not eager.** The probe runs on first activation of the (power-only) panel, not
  at boot, so simple-mode startup never spawns the two `garnet check` subprocesses.
- **Tests.** 6 Rust tests (catalog completeness + honest status per fence; probe
  confirm/inconclusive; confirm-amid-noise vs empty-run; expected-code-from-catalog; a
  CLI-gated live test that asserts both probes actually confirm where a Garnet CLI is
  present, skipping loudly otherwise) + 8 Playwright renderer unit tests (confirmed only
  on a real confirm, false-green and no-CLI guards pinned on the structural class,
  declared/deferred rows carry no trap claim, ordering, escaping). Reviewed via a 7-pass
  Judge+Auditor loop. Local ladder green on `NUCBOX_M2PRO_S`: studio crate 50/50, e2e
  37/37, clippy clean, shell + status contracts, build + `--studio-smoke`. Scope:
  `apps/garnet-studio` (non-frozen) — no scripts / CI / gate change, no frozen crate, no
  capability widening (`core:default`), no new crate dependency. Research-grade prototype (v0.x.x).

## Unreleased — Studio Velocity Editor (Phase 3) (2026-06-28)

### Studio — Velocity Editor live check (`apps/garnet-studio`)

- **Added** a Velocity Editor to the Parse / Check / Run panel: a source buffer plus a
  debounced (200 ms) `studio_velocity_check` that runs `garnet check --format json` on the
  buffer and renders the language's own diagnostics (capability coverage, bounds, parse
  errors) live as you type. The backend deserializes the diagnostics JSON into a typed
  `VelocityCheckReport`; the pure renderer `src/velocity.ts` draws the list. The CLI's
  severity / code / message are rendered verbatim — never reclassified in the UI.
- **Ephemeral, no seal per keystroke.** A live check writes the buffer to an ephemeral
  scratch temp file (best-effort removed) and runs check with **no evidence bundle** — only
  the explicit Check / Run buttons seal. It runs **check only, never run** — no auto-execution.
- **Honest source locations.** Parse diagnostics carry a byte span → a precise, byte-accurate
  line (newline bytes counted in the UTF-8 encoding, so multibyte source is not mislocated);
  check diagnostics are message-only today → rendered as "whole buffer," never a faked line.
- **Fail-safe parsing (review fixes).** `CheckJson` requires both the `diagnostics` and
  `summary` keys, so a stale / wrong binary's bare `{}` or `[]` is **not** read as a clean run;
  the green "no diagnostics" line is gated on `summary.ok && errors == 0`, so a ran-but-not-ok
  result with no per-item diagnostics renders an honest "check reported a problem," never a
  pass; a non-JSON / no-CLI / timeout output degrades to an explicit "did not run." A
  `latestOnly` guard drops a stale out-of-order check result so a slow earlier run cannot
  overwrite a newer one's diagnostics.
- **Tests.** 9 Rust parse-contract tests (check-no-span, parse-with-span object, clean run,
  exit-1-is-still-a-result, non-JSON → ran=false, bare-object rejected, no-CLI refusal writes
  no temp file) + 8 Playwright unit tests for the pure renderer and the `latestOnly` race
  guard (incl. the false-green and out-of-order cases). Reviewed via a 7-pass Judge+Auditor
  loop, which caught the false-green, the lax JSON acceptance, and the untested race guard —
  all fixed and test-pinned. Local ladder green on `NUCBOX_M2PRO_S`: studio crate 44/44,
  e2e 28/28, clippy clean, shell + status contracts, build + `--studio-smoke`. Scope:
  `apps/garnet-studio` (non-frozen) — no scripts / CI / gate change, no frozen crate, no
  capability widening (`core:default`), no new crate dependency. Research-grade prototype (v0.x.x).

## Unreleased — Studio Diff-Caps Review Gate (Phase 2) (2026-06-28)

### Studio — Diff-Caps Review Gate (`apps/garnet-studio`)

- **Added** `studio_diff_caps(old_path, new_path)` + a power-only **Diff-Caps Review**
  panel that renders `garnet diff-caps --machine`'s `garnet.diff-caps.machine/1` verdict.
  The backend deserializes the machine JSON into a typed `DiffCapsReport`; the frontend
  renders it. The CLI is the single source of truth — **the band and verdict are rendered
  verbatim, never recomputed** — and the declared-surface-only **scope caveat is shown so
  a green 5/5 is not read as "safe."**
- **Honest verdict semantics.** Exit 1 ("authority expanded") is a **valid verdict** (the
  gate working), rendered as band 2/5 **"review required"** — not a generic failure and
  not "merge REFUSED" (diff-caps flags for review; the merge block is a separate CI rule,
  not something the CLI or this read-only panel performs). Exit 2 / unparseable output
  surfaces an honest error and never invents a verdict; a truncated machine JSON points at
  the evidence bundle rather than blaming the input paths.
- **All six diff dimensions rendered** (gained/removed caps, added/removed functions,
  per-function expansions, wildcard) — a function-only change is never mislabeled "no
  declared capability changes."
- **Tests.** 7 Rust parse-contract tests (exit 0/1/2, expansion-is-a-verdict,
  band-verbatim vs the exact three-clause CLI scope, unparseable-success degrades,
  truncation does-not-blame-paths) + 8 Playwright unit tests for the extracted pure
  renderer (`src/diff-caps.ts`). Reviewed via a 7-pass Judge+Auditor pass (which caught
  the "merge REFUSED" overclaim, the dropped function dimensions, and the truncation
  inversion — all fixed). Local ladder green on `NUCBOX_M2PRO_S`: studio crate 36/36, e2e
  19/19, clippy clean, shell + status contracts 8 + 17, build + smoke. Scope:
  `apps/garnet-studio` (non-frozen) — no scripts/CI/gate change, no frozen crate, no
  capability widening (`core:default`). Research-grade prototype (v0.x.x).

## Unreleased — Studio dead-weight cleanup (Phase 1) (2026-06-28)

### Studio — dead-weight removal (`apps/garnet-studio`)

- **Removed the Taxonomy UI panel** (nav button + section + its frontend loader and the
  orphaned `.taxonomy`/`.pills` CSS). It was a static mirror of a Rust constant. The
  backend `get_language_taxonomy` command is **kept** — it feeds `--studio-smoke` and the
  `taxonomy_preserves_copy_truth` test — so the contract is preserved and still asserted;
  only the redundant UI is gone (8 → 7 panels).
- **Removed the redundant status-bar evidence-copy button** (and its `button.sb-item`
  CSS). The evidence root is already shown in the Evidence panel and CLI Health output.
- **Relabeled the Advisory panel "local advisory evidence."** It now states plainly that
  the actions run repo Python scripts and write bundles to disk, that nothing is sent
  anywhere, and that no provider API is called — closing the "implies a backend/delivery"
  gap the design dossier flagged.
- **Tests.** e2e updated (7 panels + a positive dead-weight assertion: taxonomy panel
  gone, `#sb-evidence` gone, advisory = local-evidence). Reviewed via a 7-pass
  Judge+Auditor pass. Local ladder green on `NUCBOX_M2PRO_S`: studio crate 29/29, e2e
  10/10, shell + status contracts 8 + 17, build green. Research-grade prototype (v0.x.x).
- **Deferred — human-merge-only (integrity).** Removing the hard-coded, stale-by-design
  Release status tiles (the dossier's top honesty cleanup) requires editing the
  agentic-dogfood-matrix CI probe, which pins a tile string — and a PR may not modify the
  gate it merges under. That removal is prepared as a separate Jon-merge slice. Also
  deferred: the theme switcher and the CLI Health demotion.

## Unreleased — Studio Windows bootstrap runner (Phase 0) (2026-06-28)

### Studio — typed GUI bootstrap runner (`apps/garnet-studio`)

- **Added** `studio_bootstrap_run_step` — a typed, allowlisted Tauri command that runs
  one of four named Windows setup steps (`preflight`, `install-python`, `build-cli`,
  `configure-env`) from the CLI Health Setup Assistant. Each runs the **same**
  repo-generated PowerShell that `studio_bootstrap_write_scripts` emits — both now build
  their script set from one `BootstrapStep`-derived source, so Run and Write cannot
  drift — executed through the existing `run_process_with_timeout` path. **No Tauri
  shell/open/fs plugin** is added; the capability surface stays `core:default`. Anything
  off the four-step allowlist is refused **before** any process spawns or evidence bundle
  is created.
- **Repo-gated + Windows-gated.** `build-cli`/`configure-env` require a validated repo
  root (`GARNET_REPO`); `preflight`/`install-python` do not. The run path is
  **Windows-only** (the scripts use winget / User-scope env / LOCALAPPDATA): off Windows
  it refuses honestly rather than run under `pwsh` and report a hollow "Passed".
  `build-cli` (`cargo build --release`) gets the matrix timeout budget.
- **Evidence.** Every run writes the executed script + full stdout/stderr + a command
  manifest to a `bootstrap-run` evidence bundle before the UI payload is capped.
- **Honest UI copy.** The Setup Assistant states plainly that the steps run locally
  (Windows only), **can change the machine** (install software / set user env + PATH),
  that env/PATH changes take effect **only after restarting Studio**, and that
  build-cli/configure-env need a `GARNET_REPO` checkout. No "installed successfully",
  parity, or OS-sandbox claim. Scope is `apps/garnet-studio` (non-frozen): no workspace
  crate, no frozen crate, no CI workflow, no gate threshold touched. macOS/Linux runners
  are out of scope for this slice (no parity claimed).
- **Fixed (found by review).** A pre-existing #426 defect — `build-garnet-cli-from-repo.ps1`
  and `configure-garnet-env.ps1` emitted `$ErrorActionPreference` **before** their
  `param()` block, a PowerShell parse error — is corrected (param first). This slice is
  the first surface to *execute* those scripts, so the fix lands here with a regression
  test pinning param-first ordering. Verified with the PowerShell parser: the old form is
  1 parse error, the fixed form is 0.
- **Tests.** TDD red→green plus a 7-pass Judge+Auditor review (which caught and
  reproduced the param blocker): a focused unit-test set (allowlist refuses off-list input
  with no spawn/bundle; repo gate blocks/permits through the wired impl path; off-Windows
  refusal; param-first regression guard; Write/Run single-source equality; step intent
  mapping) + a Playwright e2e for the four run controls and the local-action copy. Local
  ladder green on `NUCBOX_M2PRO_S` (Windows 10.0.26200): studio crate 29/29, clippy clean,
  build, e2e 9/9, shell + status contracts 8 + 17, `--studio-smoke` passed. Research-grade
  prototype (v0.x.x), not production/1.0.

## Unreleased — WV-4 Studio Playwright ledger record (2026-06-27)

### WV-4 — Playwright Studio-UI harness (`apps/garnet-studio`)

- **Recorded** the WV-4 Playwright Studio-UI harness in this ledger. The harness
  itself landed in **#422** (squash `952b3be`), but that PR touched zero docs; this
  docs-only follow-up adds the canonical CHANGELOG entry and repoints the
  `apps/garnet-studio/playwright.config.ts` header at it (it previously cited "this
  PR's Deferred section", a GitHub-side reference rather than an in-repo record).
  The harness drives the **built Vite `dist`** in a headless Chromium and asserts the
  Studio overhaul's UI structure + pure-frontend behaviour: the launch splash holds
  then dismisses, all panels render, simple-mode hides the power-only panels, panel
  switching works, the safety-contract copy renders, tooltips are present, and the
  status bar reports version + mode.
- **Honest scope — structure/behaviour proof, NOT a CLI round-trip.** Outside the
  Tauri shell the `invoke()` calls reject by design and `main.ts boot()` degrades to a
  "browser preview", so the harness proves UI structure and pure-frontend behaviour
  only. Scope is `apps/garnet-studio` (non-frozen): no Rust, no frozen crate, no CI
  workflow, and no gate threshold touched — here or in #422.
- **Deferred — Tauri desktop-drive follow-up (the durable record the config header
  now cites).** Driving the **real desktop shell** — Run → `CommandResult`
  (stdout/exit_code), evidence-bundle → manifest path, and the **persisted**
  simple/power mode toggle — is NOT covered: those paths go through `invoke()` and
  reject in a plain browser. Exercising them needs `tauri-driver`/WebDriver and is
  flagged as the WV-4 desktop-drive follow-up (the Codex computer-use lane is the path
  for the live desktop GUI). Also deferred: a skippable **CI e2e job** — wiring a new
  CI job is a gate change and is human-merge-only per the integrity rules.
- **Local evidence (as reported on #422, not re-run in this docs slice):**
  `npm run test:e2e` → 7/7 on the Windows NUC (`NUCBOX_M2PRO_S`, Windows
  10.0.26200.8457), headless Chromium over the built dist; the Studio shell-contract
  gate (`scripts/test_garnet_windows_linux_studio_shell.py`) stays 7/7. The e2e suite
  does **not** run in CI (see the deferred CI e2e job), so no runner is reddened.
  Garnet remains a research-grade prototype (v0.x.x), not production/1.0; this changes
  nothing about that.

## Unreleased — Studio macOS parity + judged enhancement set (2026-06-12)

### Security — S114-FIX-2: deny-by-default capability mediation (close residual fail-open lanes)

- **Context:** the 2026-06-25 **independent, cross-lineage** re-verification (OpenAI
  Codex) of the S114 capability-enforcement claim found two HIGHs the Claude fleet
  missed — (1) top-level `let`/`const` initializers exercised `@caps` host authority
  with no active frame, accepted+sealed; (2) an invalid `@max_depth(9999)` was
  accepted+sealed at runtime. Both were fixed for the `run`/VM/`agent-loop` lanes in
  commit `4994867`. The Opus final review then **dynamically confirmed** that fix was
  incomplete: `require_capability` remained **fail-open** at `active_frames==0`, so the
  same load/eval-time host-authority bypass survived on `garnet eval`, `garnet test`,
  `garnet doctest`, `garnet repl`, and the `garnet run` vendored-dependency preload.
- **Fixed (deny-by-default / complete mediation):** the `garnet` binary now refuses a
  host-authority primitive reached with **no active `@caps` frame** on every lane
  (`garnet_interp::eval::set_strict_no_frame`, set in `main`; a process-global so it
  covers the stack-sized worker thread `garnet run` spawns). `test`/`doctest` loads are
  additionally framed under the file's `main` entry (parity with `garnet run`).
  Library/embedder callers (no Garnet program context) keep the permissive direct-call
  default, so the change is binary-scoped.
- **Fixed (annotation-range parity):** `@max_depth(N)` range (`1..=64`) is now validated
  at **load time** via a recursive pre-pass (`validate_module_max_depth`) covering top-level
  functions, **impl methods, and nested-module functions**, so `garnet run` refuses an
  out-of-range bound on an *uncalled* function anywhere `garnet check` does, on both backends
  — closing a `run`-accepts/`check`-rejects split. (The first cut validated only top-level
  `Item::Fn`; the `/code-review` self-pass caught that impl/nested-module bounds still slipped
  through, fixed here with regression tests for both.)
- **Red→green:** `garnet-cli/tests/s114_residual_lanes.rs` — `eval`/`test`/`doctest`/
  vendored-dep-preload all trap a top-level/eval-time undeclared `fs` read (verified via a
  nonexistent-path discriminator), and `run` rejects an uncalled invalid `@max_depth` on
  both backends. Full workspace suite green.
- **Honest scope:** enforced scope is unchanged — only `@caps` + `@max_depth`, both
  backends; seccomp Linux-only (unverified on darwin); `@bounded`/memory/time/`@mailbox`/
  OS-sandbox remain declared-not-enforced. S114 is independently-re-verified-with-fixes
  **pending Jon's acceptance**, not self-attested-closed; the relabel and tag stay Jon's.

### Foundation HARDEN — cyclic-value render guard (`Value::display`/`debug`)

- **Fixed (stack-overflow abort → bounded render):** `Value::display()`/`debug()`
  recursed through container values with **no cycle or depth guard**, so a
  self-referential value — `let a = [1]  a.push(a)` (an array containing
  itself; `push` shares the backing `Rc`) — drove infinite recursion into a
  **stack overflow** (a non-unwinding `SIGABRT`, exit `134`). That abort killed
  the `repl` session / `test` / `doctest` run *on render*, and crucially the
  panic firewall **cannot** catch it (`catch_unwind` only recovers unwinding
  panics). This is the named follow-up from the J8 firewall slice — closing the
  cause the firewall could not.
- **The guard:** `display`/`debug` now delegate to a shared
  `Value::render(quote, depth, visited)` that (1) tracks the `Rc` pointers of
  the **mutable** containers on the render path (`Array`/`Map`/`Struct`, the
  only `Rc<RefCell<_>>` shapes — every cycle must pass through one, so tracking
  those alone breaks every cycle) and renders a revisited container as `[...]` /
  `{...}` / `Name { ... }`; and (2) caps depth at 128 as a backstop for deep
  finite values (`Tuple`/`Variant` are immutable `Rc<Vec<_>>` and cannot
  self-reference, so they need only the cap). A self-cycle now renders
  `[1, [...]]`; the public `display`/`debug` signatures are unchanged.
- **Red→green:** `garnet-interp` `render_cycle_tests` (self-referential array →
  `[1, [...]]`, self-referential map → `{"self" => {...}}`, mutual a↔b cycle
  terminates, 300-deep finite value is depth-capped, ordinary nested value
  renders unchanged) + `garnet-cli/tests/cyclic_value_render.rs` (the REPL
  builds a cycle, renders it, and **keeps evaluating** — exit `0`, not `134`).
  Without the guard the REPL e2e test aborts with `signal 6` / "stack overflow,
  aborting".
- **Honest scope:** this stops the *render* from crashing; a self-referential
  value is still an `Rc` **reference cycle** that leaks (never reclaimed) — that
  is inherent to `Rc` and out of scope here (the fix is "don't abort on
  rendering," not "collect cycles"). No behavior change for acyclic values.
  Workspace 2085/0; clippy `--all-targets -D warnings` clean; no gate modified.

### Foundation HARDEN — process-abort firewall on the `eval`/`repl`/`test`/`doctest` lanes (J8)

- **Added** `garnet-cli/src/panic_firewall.rs` — a `catch_unwind`-based firewall
  giving the four interpreter-invoking lanes that ran the interpreter on the
  **main thread** the abort-protection the `run` lane already had (spawn a
  large-stack thread + `join`). Before this, an interpreter panic aborted the
  process (`eval`/`test`/`doctest` exit `101`) or killed the whole interactive
  session (`repl`). The interpreter is `!Send` (`Rc`-based `Env`), so it cannot
  be moved onto a spawned thread the way the run lane does — `catch_unwind`
  recovers the unwind **in place**, the only option for a `!Send`, main-thread
  interpreter. A custom panic hook, gated by a **thread-local** active flag,
  suppresses the default backtrace noise only on a firewalled thread, so other
  tests in the crate's test binary (and `#[should_panic]`) still report normally.
- **Wired** every interpreter entry on these lanes — both the EXECUTE path
  (`eval_expr_src`/`call_entry`) **and the LOAD path** (`load_source`). The load
  path matters because Garnet evaluates top-level `let`/`const`/`memory`
  initializers *during* `load_source` (`register_item`), so a poison
  initializer (`const X = (i64::MIN).abs()`) panics on load exactly like a test
  body does. Per-lane semantics: `eval` → controlled exit 1; `repl` → printed
  error, **session survives**; `test` → that test/file FAILED, **run continues**
  + summary; `doctest` → that fence/file fails, suite continues.
- **Fixed (same lane, found by adversarial review):** `garnet test`'s summary
  computed `passed = total_run - total_failed` (`usize`), which **underflowed
  and aborted with exit 101** when file-level parse/load failures outnumbered
  run tests (e.g. a single malformed test file). Now passes are counted
  directly (`total_passed`), so no subtraction can underflow.
- **Deterministic trap (red→green):** the reachable trigger is `i64::MIN.abs()`
  (`(0 - 9223372036854775807 - 1).abs()`), which overflows and panics.
  `garnet-cli/tests/panic_firewall_lanes.rs` (subprocess: `eval`/`test`/`doctest`/
  `repl`, execute + load + the underflow case) and unit tests in
  `cmd::repl::tests` / `cmd::doctest::tests` / `panic_firewall::tests` all panic
  the lane (exit `101`) without the wiring and degrade to a controlled exit `1`
  with it. **Two adversarial review rounds** (5 lenses then 2) drove this: round 1
  found the load-path gap (CRITICAL) and the underflow (HIGH) plus dupes; round 2
  confirmed all closed with zero new holes and a full grep proving no
  unfirewalled main-thread interpreter entry remains in `garnet-cli`.
- **Honest scope / named follow-up:** `catch_unwind` catches **unwinding**
  panics; it does **not** catch a **stack overflow** (a non-unwinding
  `SIGABRT`). A self-referential value (`let a = [1]  a.push(a)`) makes
  `Value::display()` recurse without a cycle/depth guard → exit `134`, still
  killing the lane on render. That is a distinct **interpreter** crack (a
  `Value::display`/`debug` recursion guard, `garnet-interp-v0.3/src/value.rs`),
  pre-dating this slice, and is a named follow-up — not closeable by the
  firewall. No "enforced/abort-proof" claim beyond what the trap tests prove.
  Workspace 2079/0; clippy `--all-targets -D warnings` clean; no gate modified.

### Foundation HARDEN — checked `+`/`-`/`*`/unary-`-` integer overflow (RFC-0002 implementation)

- **Changed (abort/wrap → diagnostic, both backends):** `i64` `+`, `-`, `*`,
  and unary `-` overflow was the worst-of-both-worlds the trust kernel can't
  attest — a **silent wrap in release** (wrong answer, no signal) and a **host
  panic in debug** (`attempt to {add,subtract,multiply,negate} with overflow`),
  so arithmetic semantics differed by build profile. Now `checked_add` /
  `checked_sub` / `checked_mul` / `checked_neg` yield a **deterministic
  `integer overflow: <lhs> <op> <rhs>` diagnostic** (`RuntimeError::Overflow`
  on the interpreter, the byte-identical `VmError::Runtime` on the VM) — the
  same RB-2 doctrine already shipped for `/` and `%`, now extended to the
  remaining operators per **RFC-0002 ("integer arithmetic is checked by
  default", Accepted by Jon 2026-06-12, verbatim: "extends the same discipline
  to `+`/`-`/`*`").** Profile-independent, loud, never an abort.
- **Sites sealed:** interpreter expression path (`eval.rs`, binary + unary),
  interpreter compound-assign path (`stmt.rs`, `+=`/`-=`/`*=`), and the VM
  (`vm.rs`, `apply_binary` + `apply_unary`). All five previously used bare
  `a + b` / `-i`.
- **Deterministic trap (red→green, the proof it is real):**
  `garnet-interp tests/overflow_guards.rs` gains add/sub/mul/neg + compound
  cases — **7 of them panic at the exact pre-fix lines** (`eval.rs:277/285/292`,
  `stmt.rs:167/168/169`) without the change and return the diagnostic with it.
  `garnet-cli tests/overflow_parity.rs` proves the user-facing guarantee:
  each overflowing operator **exits `1`** (controlled diagnostic, not exit-101
  abort) with the **byte-identical** `integer overflow: …` line on `--interp`
  and `--vm`. This also closes the latent flake where the
  `prop_vm_matches_interp_on_random_shadowing_programs` proptest (RB-4/PR-3,
  now on `main`) could panic on a generated `*`-overflowing program at
  `eval.rs:292` instead of reaching its `(Err, Err)` parity arm; 8000 random
  programs across two runs now pass with zero panics.
- **Perf note (per RFC-0002):** each checked op adds one overflow-check branch,
  perfectly predicted in the (overwhelmingly common) non-overflow case. Both
  backends already pay tree-walk enum dispatch (interp) / bytecode opcode
  dispatch + `Value` match (VM) per operation, so the relative cost of one
  predicted branch is negligible; full-suite wall-time was unchanged within
  noise. No dedicated arithmetic microbenchmark exists in-repo, so **no
  speedup/slowdown number is claimed** — only "no observed regression."
- **Honest boundary (named-deferred):** the **explicit wrapping escape-hatch**
  the RFC reserves for intentional modular arithmetic (a `wrapping_*` intrinsic
  or `@wrapping` block, surface TBD) is **NOT** in this slice — checked-by-
  default is shipped; the opt-in wrap surface is a follow-up. The pre-existing
  silent-release-wrap was a profile accident, not a sanctioned wrapping path,
  so nothing legitimate is removed. No "enforced" claim beyond the trap tests.
  Workspace 2062/0; `cargo clippy --workspace --all-targets -D warnings` clean;
  trust-kernel gates unchanged (no gate modified by this slice).

### RB-4b.3 — per-pass caps re-check on VM lowering (W-REBUILD Foundation · Directive 7)

- **Added** `garnet_vm::caps_recheck` — the GHC-Core "re-check the invariant
  after every lowering pass" pattern applied to the AST→bytecode lowering.
  The VM compiler is capability-BLIND (it drops `@caps` entirely); this
  re-establishes the invariant on the lowered artifact: **no native
  function's bytecode may require more host authority** (the union of registry
  caps of the primitives it `Call`s) **than the checker's per-function
  transitive verdict** (`garnet_check::caps_graph::check_caps_coverage`,
  RB-1's `CapSet`) **grants.** A lowering or future optimization pass that
  launders authority is caught at the pass that introduced it.
- **Deterministic trap (the proof it is real, not aspirational):**
  `planted_laundering_call_is_trapped` injects a `Call` to the fs-requiring
  `read_file` into a function whose source declared no fs, and the re-check
  rejects it (`widened == ["fs"]`). The check is SATISFIED on every real
  program (`caps_recheck_corpus` over the example corpus) — its value is the
  guard against future passes, not a finding on today's faithful lowering.
- **Shadow-correct resolution (adversarial-review fix, before merge):** call
  resolution mirrors `caps_graph::resolve_callee` — a bare `Call` naming a
  user function declared in the module resolves to that user fn FIRST, even
  when it is named like a cap-bearing primitive (`read_file`, `get`, `now_ms`,
  …). Without this, a user function shadowing a primitive name produced a
  FALSE laundering positive that rejected valid code; the regression test
  `user_function_shadowing_a_primitive_name_is_not_laundering` is red before
  the fix and green after.
- **Wiring:** `compile_source_rechecked` is the on-path entry (compile +
  re-check; a laundering becomes `VmError::Compile`); plain `compile_source`
  is unchanged (behavior-identical). `garnet-vm` gains acyclic `garnet-check`
  + `garnet-stdlib` deps.
- **Honest scope (no overclaim):** a STATIC cross-IR **caps-containment** check
  (one-directional: lowered ⊆ declared) with a trap — **NOT** runtime
  enforcement (the interp S90 `require_capability` / VM S92 entry frame own
  that) and **NOT** a backend (RB-6 decides that). Fallback (non-native)
  functions are skipped — they execute under interp S90 guards, so a re-check
  there is vacuous (disclosed). Embedding the verdict into the seal predicate
  is **RFC-gated (Jon)**, out of scope. No "enforced" claim beyond what the
  planted trap proves. Workspace 2013/0; enforcement-parity + all trust-kernel
  gates unchanged.

### W-REBUILD — final workstream report (Foundation band closeout · docs-only)

- **Added** `F_Project_Management/W_REBUILD/W_REBUILD_FINAL_REPORT.md` — the
  closeout report for the Foundation rebuild band (RB-0 … RB-7, all merged to
  `main` @ `ed75c59`). Carries the full slice ledger (PR → commit per slice), the
  invariants the workstream established (caps bitset, registry-derived dispatch,
  one CST substrate, the per-pass caps-recheck mechanism, edition-invariant caps,
  wasm/WASI portability, the declared-not-enforced `:caps` surface), the deferred
  Jon-owned next steps (RB-5 impl against Option C; the W-PLAY playground spike;
  optional RB-8; NUC cross-OS REPL verification), and the honest boundaries (not
  production, no backend built, the full Directive-7 vision not delivered, no
  release). Docs-only; no code/gate change.

### RB-7 — the REPL joy slice (W-REBUILD Foundation)

- **Rebuilt** `garnet repl` as the "joy" REPL in `garnet-cli/src/cmd/repl.rs` on
  [`reedline`](https://crates.io/crates/reedline) (v0.48): history, multiline
  input (brace/paren/bracket *and* dangling-annotation aware), Tab completion
  over REPL commands + every stdlib primitive (bare and qualified) + the live
  session bindings, and pretty-printed values (composite values carry a `: Type`
  tag).
- **`?doc <name>`** (and the `?<name>` shorthand) reads the live registry
  `PrimMeta` — module-qualified name, arity, required `@caps`, layer, stability,
  doc — and a user function's arity + declared caps.
- **`:caps`** shows the session's authority surface in two honest sections: what
  the loaded `@caps` functions *declare*, and the available primitives grouped by
  required capability. **Labeled NOT an enforced budget** — `@caps` is enforced
  per-function at entry (S90); a bare call at the prompt holds no capability
  frame. No "enforced" claim.
- **Architecture:** reedline + the ergonomics live in `garnet-cli` ONLY, so
  `garnet-interp` stays terminal-dependency-free and **still compiles to
  `wasm32-wasip1`** (the RB-6 portability is preserved). Command dispatch is a
  pure, unit-tested core (19 REPL tests) with a non-TTY plain fallback for pipes
  / CI / recorded demos. Two additive read-only interp introspection methods
  (`Interpreter::live_binding_names` / `lookup_binding`, `Env::local_names`) feed
  completion + `?doc`/`:caps`; no semantic change, VM/bytecode paths untouched.
- **Evidence:** recorded session `docs/demos/repl-session.txt` + doc page
  `docs/internals/repl.md`; Mac proof only — cross-OS verification handed to the
  NUC lane (`RB7_NUC_HANDOFF.md`), **not marked cross-OS-complete from one
  machine**. Workspace 2030/0.
- **Recorded** Jon's RB-6 §10 decision (2026-06-16): resolved-IR shape = Option C
  (`garnet-vm` bytecode as the future RB-5 target; tree-walk = oracle; D = back-
  half); wasm playground deferred to a separate W-PLAY spike (not in RB-7); RB-7
  proceeds now as the REPL slice only (no RB-5 / no indexed-frame rewrite).

### RB-6 — backend/IR decision memo (DRAFT) + RB-5 sequencing decision (W-REBUILD Foundation · docs-only)

- **Recorded** Jon's **RB-5 sequencing decision (Option C, 2026-06-14):** RB-5 is
  sequenced with RB-6, not rejected — the RB-5 baseline is RB-6's before-number;
  `(depth,slot)` + interner land in whichever execution representation RB-6
  chooses; no standalone AST name-representation change unless RB-6 keeps the AST
  as the execution substrate; preserve the just-stabilized substrate. Stamped in
  `W_REBUILD_SPEC.md` (RB-5 RESOLVED block) and the RB-5 STOP+REPORT §9.
- **Added** `F_Project_Management/W_REBUILD/RB6_BACKEND_IR_DECISION_MEMO.md` — the
  RB-6 decision memo (DRAFT; the decision is Jon's, escalated in §10). Carries the
  tree-walk before-number; a **wasm32 feasibility spike** (`wasm32-wasip1`
  **compiles today** (cargo `Finished` + 16 MB rlib), no source change; `wasm32-unknown-unknown`
  has ONE blocker — `getrandom/js` — trivial; host-authority touches concentrate
  in the `@caps` primitives = the WASI import boundary); IR options A–D + the
  custom-VM-as-third-path parity cost; the synergy ledger; the Stroustrup-linker
  doctrine; the **per-pass caps re-check HARD CONSTRAINT** (RB-4b.3 landed the
  mechanism, so Option C inherits it); and an **integrate-lean recommendation**
  (Option C — reuse `garnet-vm` bytecode as the resolved IR — now; Wasmtime/WASI
  as the strategic back-half) with a measured ~2–3× reopen threshold. No backend
  merged, no `.wasm` executed, no code/AST/`Value`/gate touched; feasibility-compile
  + memo only.

### RB-5 — environment rebuild · STOP+REPORT (W-REBUILD Foundation · docs-only)

- **Added** `F_Project_Management/W_REBUILD/RB5_ENV_REBUILD_STOP_REPORT_2026-06-14.md`
  — the scheduled STOP+REPORT after RB-5, with measured numbers, before the RB-6
  memo. RB-5 (string interner + `garnet-check` `(depth,slot)` resolution pass +
  indexed-frame `Env`) is **STOPPED at the design gate**: the measured baseline is
  captured (`eval_fib_15` 394.97 µs · `eval_array_1000_map_reduce` 262.56 µs ·
  `eval_expr_arithmetic` 1.475 µs on Apple M5 Pro / rustc 1.95.0), and the
  `(depth,slot)` indexed-frame rewrite is blocked on a substrate/IR decision that
  is Jon's: the AST has no node identity to attach resolution results to, the REPL
  accretes bindings incrementally (vs static whole-program resolution), and the
  `Env` is five chains with capture-by-reference closures. All three reduce to a
  name-representation change in the shared AST or a **resolved IR** — the very
  question RB-6 settles. Recommendation: **sequence `(depth,slot)` + interner with
  the RB-6 IR decision** (Option C); decision A/B/C/D escalated to Jon. No code,
  AST, gate, or `Value` touched.

### RB-4b.4 — editions note + spec reconciliation (W-REBUILD Foundation · docs-only)

- **Added** the **editions spec note** to `garnet-parser-v0.3/AGENTS.md`
  (parked there, not in the maintainer-owned `GARNET_v1_0_Mini_Spec.md`,
  following the `garnet-cst/AGENTS.md` CST-note precedent). It records the
  landed S32 mechanism as fact — `Edition::{V1_0 default, Next=v2.0}`, `async`
  reserved only under `v2.0`, the **one-canonical-IR invariant** (editions gate
  lexing only; AST/checker/interp/caps manifest edition-invariant by
  construction), and `Garnet.toml` `[project] edition = "…"` pinning — and the
  Directive-9 surface-collapse as design intent **bound to the existing
  RFC-gated edition vehicle, explicitly not yet built**. A new parser
  Stable-Contract bullet locks "editions gate lexing only; caps
  edition-invariant; new edition = RFC-gated."
- **Reconciled** `W_REBUILD_SPEC.md` §3 RB-4a/RB-4b against the landed reality:
  (1) the Directive-7 criterion (typed core IR carrying caps, re-checked *after
  every lowering pass*) marked **RESOLVED-PARTIAL** — RB-4b.3 delivered a static
  cross-IR containment check on the *one* AST→bytecode pass with a trap, NOT a
  typed-caps core IR and NOT a multi-pass property (those stay aspirational);
  (2) the Directive-9 editions design note marked **RESOLVED** (note now lives
  in the parser AGENTS.md); (3) the RB-4b decomposition block updated with all
  four sub-slices' merge commits; (4) the "4b" typed-views-over-green-tree
  accept-when marked **DEFERRED** (RB-4b.1 delivered a span-exact projection;
  AST and CST remain parallel, awaiting an adopter). No code or spec-semantics
  change; docs-only.

### RB-4b.2 — `SyntaxError` spans + the LSP single-parse finding (W-REBUILD Foundation)

- **Re-scoped from "typed views + LSP single-parse" (Jon, 2026-06-12)** after a
  measured blocker surfaced: dropping `parse_source` from the LSP would
  **degrade diagnostics** — `parse_cst`'s error recovery produces cascades
  (3 errors for `def broken( {`, **8** for `@@@ def` — three duplicate
  "expected annotation name" — vs `parse_source`'s single fail-fast error),
  which the RB-4b accept-when ("diagnostics preserved or improved") forbids.
  The typed-view layer also has zero adopters, so extending it now would be
  speculative. So the LSP **keeps `parse_source`** for its authoritative
  diagnostics; true single-parse is deferred until parser error-recovery is
  improved (recorded as a follow-up).
- **Changed:** `garnet_cst::SyntaxError` now carries a **`span`** (a range over
  the offending token) instead of a bare `offset` (an `offset()` accessor is
  retained). The builder anchors recovery errors at the next SIGNIFICANT
  token's full span — skipping the whitespace the error used to point at,
  matching `parse_source`'s anchoring — and the budget/lex error paths use the
  parser error's own span. This is the foundation a future single-parse needs
  to render range diagnostics.
- **Added:** `ParseError::span()` — the canonical span accessor on the parser's
  error type; the LSP's duplicate `parse_error_span` logic now calls it (DRY),
  and the CST builder reuses it for budget/lex error spans.
- **Improved:** `garnet parse --mode cst` reports errors as `line:col`
  (computed from the span) instead of a raw `byte N`.
- Proven by `garnet-cst/tests/syntax_error_spans.rs` (grammar / budget / lex
  error spans). No CST tree-shape change; LSP diagnostics unchanged (it keeps
  `parse_source`). The one behavior change is intended and an improvement: the
  CST `SyntaxError` anchor POSITION moves from the whitespace before the
  offending token to the token itself (matching `parse_source`'s anchoring),
  surfaced via the CLI `line:col` output (the only `SyntaxError`-span
  consumer). Workspace green.

### W-REBUILD stop-report rulings (docs) — Jon's nine J-queue decisions

- The nine RB-band stop-report decisions are **resolved** and recorded inline
  in `W_REBUILD_SPEC.md` (RESOLVED blocks on the RB-1/RB-2/RB-3/RB-4b
  accept-when) and in `RB_BAND_STOP_REPORT_2026-06-12.md`. Highlights: RB-1
  clone criterion amended honestly with **RB-5 as the accepted vehicle**;
  Directive-15 bounds-delta dropped from the day-one `--machine` payload
  (later RFC-gated manifest extension); the **`// FAIL-CLOSED:` allow-comment
  form blessed** as the second sanctioned pattern; RB-3 **LOC criterion
  retired** (the win was killing dispatch drift, +752 measured) and its
  **mechanism ratified** (78 adapter + 2 Unbridged + 4 BRIDGE_ONLY); runtime
  miette spans, the eval/repl/test panic firewall, and `trybuild` UI tests
  **scheduled as follow-ups**.
- **Added:** `rfcs/0002-integer-overflow-policy.md` — **integer arithmetic is
  checked-by-default** (runtime diagnostic, never a profile-dependent silent
  wrap or abort), with explicit wrapping operations where wanted. Design
  ruling (J5); extends the already-shipped `i64::MIN / -1` and `% 0`
  checked-diagnostic precedent to `+`/`-`/`*`; implementation is its own slice.
- Docs-only: no code, CI, gate, or release change.

- Studio macOS now meets the PR #391 shell standard rows 1–9 (ported the
  standard, not the Tauri code): single version stamp gated by the new
  `scripts/test_garnet_macos_studio_shell.py`; splash with 700ms/25s bounds;
  @AppStorage simple/power modes; validated+clamped settings (corrupt file
  never blocks boot) with a native Settings scene; ALL 14 process spawns
  through `StudioProcessRunner` (thread-drained pipes, timeouts, best-effort
  tree SIGKILL, timed_out/duration, 64KiB UI cap with honest marker); live
  truth tiles from docs/truth.json replacing hand-written stats; 33+ native
  hover-help tags; root-constrained evidence reader + in-app preview; ⌘1–5 /
  ⌘↩ keyboard, status bar, dark/light/system themes, reduce-motion a11y.
- 19-agent judge/audit workflow over every Studio feature; built the audited
  set (truth-decoder live-bug fix, stale 0.4.2 stamps, evidence preview, mode
  coherence, working light theme, ⌘O/drag-drop file open). Audit record:
  `F_Project_Management/STUDIO_MAC_FEATURE_JUDGE_AUDIT_2026_06_12.md`; async
  run path + locator dedup queued as the next slices.
- New contract: `apps/garnet-studio-macos/AGENTS.md` (+ checker registration).

## [Unreleased]

_Post-cut work (S121+) lands here. S121–S130 delivered: the Truth Sync Gate, doc
modernization, the v0.8.1 version bump + release guard + SBOM/signing wiring, the
signed re-cut, and this post-re-cut truth-sync._

### MIT readiness test pins re-anchored to post-#364 re-sealed committed evidence

- **Fixed (test truth, no reporter change):** six stale pins in
  `scripts/test_garnet_mit_readiness_status.py` for the
  `windows_linux_distribution` lane (68.0 → 71.0 ×5, 80.0 → 83.0 ×1) plus one
  stale evidence-phrasing assertion (`Verified bundle` → the committed
  `Committed Windows bundle` / `Committed WSL portability bundle` pair). The
  pins were honest when S107 (PR #356) wrote them — the committed domain-shell
  bundle was failing its manifest due to git CRLF→LF normalization after
  sealing — but PR #364 re-sealed those manifests against committed bytes,
  which legitimately restored `studio_domain_shell` verification and lifted
  the lane on every clean checkout. Desktop/machine-local evidence is NOT
  involved: all lifting evidence is committed under `proofs/**`. Honest gap
  left open: CI does not run this test file (it is absent from `ci.yml`'s
  script-test list), which is why the drift went unnoticed — adding it is a
  CI-gate change and stays human-merge-only per the integrity rules.

- **Fixed (version truth):** the Windows/Linux Studio shell no longer stamps
  `0.1.0` — `tauri.conf.json` drops its duplicate `version` field (Cargo.toml is
  the single stamp, set to the workspace release version), so the NSIS artifact
  is now `Garnet Studio_0.8.1_x64-setup.exe`. Drift is gated in CI by the shell
  contract test (which runs repo-wide in the agent-contracts job) and locally
  by a crate test against `[workspace.package].version` (the crate is
  workspace-excluded, so `cargo test --workspace` skips it — the crate half is
  local-ladder only). Operational consequence, on purpose: a future workspace
  version bump must bump the two Studio stamps (`src-tauri/Cargo.toml` +
  `package.json`, with their lockfiles) in the same PR or the agent-contracts
  job fails loudly.
- **Added (boot experience):** a real launch splash (brand, tagline, live boot
  status, version) that holds ≥700 ms and fades once preferences + CLI health
  resolve, with a hard 25 s ceiling so the splash always lifts; the window
  background color matches the default dark theme so launch no longer
  white-flashes (a persisted light theme briefly shows a dark frame before CSS
  applies — known cosmetic). Honest note: the prior Tauri shell had **no**
  splash — earlier splash memories trace to the retired Electron-era app.
- **Added (modes + settings):** Simple/Power interface modes (power-only panels
  stay in the DOM, hidden not removed, so contract copy is intact) with a
  Settings panel persisting mode/theme/timeouts to a per-user JSON file;
  values are validated and clamped on the Rust side.
- **Added (robustness):** every spawned command — including the boot-path CLI
  health probes (10 s budget) — runs with piped, thread-drained output, a
  per-category timeout (separate, larger budget for the matrix runs),
  best-effort **process-tree** kill on timeout (`taskkill /T` on Windows, own
  process group on Unix, so a wrapper's grandchildren don't survive), a
  `timed_out` result, duration reporting, and a 256 KiB-per-stream UI payload
  cap. When an evidence bundle exists the full streams are written to it
  before capping; when bundle creation failed, the truncation marker says so
  instead of pointing at a bundle that doesn't exist.
- **Added (truth surface):** the Release panel's hand-written stats tiles are
  replaced by live `docs/truth.json` values (version, tag, tracked slices,
  readiness, primitives, workspace tests) with the stamping commit shown, and
  an explicit "truth surface unavailable" state instead of guessed numbers.
- **Added (converter UX):** successful conversions render an in-app preview of
  the produced `.garnet` output via new read-only commands
  (`list_evidence_files`, `read_evidence_text`) that canonicalize and reject
  any path outside the Studio evidence roots (traversal/symlink-escape safe,
  size-capped).
- **Added (UX polish):** hover help (`data-tip`) across all controls, keyboard
  shortcuts (Ctrl+1…8 panels, Ctrl+Enter primary action), a status bar (shell
  version, CLI version, mode, copyable evidence root), copy buttons and
  collapsible long output on results, dark/light/system themes, focus-visible
  rings, and reduced-motion support.
- **Boundaries unchanged:** no provider APIs, no new Tauri permissions
  (`core:default` only), no new frontend dependencies, active-vs-advisory
  language split intact, evidence contract (`MANIFEST.sha256`,
  `source_included=false`) intact, and no platform/proof claim upgrades — the
  open-gates copy (clean-VM re-proof at the new stamp, Linux launch, signing,
  MSI, winget, notarization) still reads as open.

### RB-2 follow-up — interp `%= 0` is now "division by zero" (cross-backend parity)

- **Fixed (observable error-string change, interp only):** `a %= 0` in the
  interpreter's compound-assign path (`garnet-interp-v0.3/src/stmt.rs`,
  `compound_apply`) now returns `RuntimeError::DivByZero` ("division by
  zero") instead of falling through to the catch-all "compound assignment
  on unsupported types" — closing the pre-existing cross-backend divergence
  recorded in the RB-2 entry below (the VM lowers `%=` to the `Mod` opcode
  and already reported "division by zero"). One match arm added, mirroring
  the existing `/=`-by-zero arm. Proven red→green by a cross-backend parity
  test (`garnet-cli/tests/mod_zero_parity.rs`, `overflow_parity.rs` style:
  same exit code, same diagnostic line on both backends); `/= 0` is pinned
  alongside as a regression guard. No test or gate asserted the old string
  (verified by repo-wide grep before the change). Rescue-observable too:
  both the old `Message` catch-all and `DivByZero` are catchable by an
  untyped `rescue`, but the exception payload an interp `rescue` sees for
  `%= 0` changes to "division by zero" — now identical to `/= 0` and
  expression-path `% 0`.

### RB-4b.1 — substrate fidelity (W-REBUILD Foundation)

- **Changed:** `garnet_cst::cst_to_ast` is now **span-exact** with
  `parse_source` — the projected AST equals the recursive-descent parser's
  AST byte-for-byte *including spans* across the corpus (it was previously
  validated only span-NORMALIZED). `span_of` trims leading/trailing trivia +
  newlines, skips a leading `AttrList`/`pub` for item spans (parser items
  begin at their keyword; the CST keeps annotations + `pub` inside the node
  for round-trip, only the projected span trims them), extends brace `Block`
  spans to the bracketing `{`/`}` siblings, keeps `pub` for struct fields
  (the parser includes it there), and fixes the Module span to the raw full
  source range. **No CST tree-shape change** — round-trip, token-parity, and
  `cst_to_ast_parity` stay green.
- **Span correctness beyond the corpus (adversarial review):** the review
  found three transparent-wrapper constructs the parser strips from spans
  but the CST keeps as tokens — parenthesized sub-expressions (`(1+2)*3`),
  `dyn Trait` (inner-trait span excludes `dyn`), and parenthesized types
  (`x: (Int)`). All three are now span-exact: `span_of` sees through
  `ParenExpr` to its inner expr (iteratively — no added stack frames), the
  `dyn` inner trait span trims the keyword, and param spans derive their end
  from the lowered type rather than trailing grouping `)` tokens. Locked in
  by `span_exact_on_transparent_wrapper_constructs` (14 cases) so they
  cannot silently regress. Corpus span-exact test unaffected.
- **Known boundary (deep nesting):** the span projection recurses with the
  AST lowering, so `cst_to_ast` is stack-safe at the default budget's max
  nesting depth (256 — every default-path caller lives under it; proven by
  `cst_to_ast_is_safe_at_the_default_budget_depth`) but a caller that raises
  `max_depth` far past the default and then lowers a pathologically deep
  tree must bound depth itself. Recorded as a known limitation, not a
  default-path guarantee; a follow-up may make the span walk fully iterative.
- **Added:** `garnet_cst::parse_cst_with_budget_and_edition` and a public
  `garnet_parser::check_token_nesting`. The rowan path now applies the
  parser's fail-fast budget fences (source-bytes, token-nesting depth)
  **error-tolerantly** — recording `SyntaxError`s while still building a
  round-trippable tree — so `parse_cst` agrees with `parse_source` about
  which inputs are over-budget (closing the one error-verdict disagreement:
  deeply-nested input the parser rejected but the CST silently accepted).
  `parse_cst` is the default-budget+edition wrapper; both APIs proven via
  `tests/substrate_fidelity.rs` (red→green: 2 failing → 3 passing).
- **Why:** this makes the green tree a faithful substrate for the AST —
  groundwork for RB-4b.2 (typed views + LSP single-parse) and an eventual
  `parse_source` reroute, where span drift would silently move doc-comment
  extraction and miette caret positions. **No consumer behavior change on any
  well-formed input (or any input `parse_source` already accepts).** The one
  observable change is the intended convergence: `garnet parse --mode cst`
  now reports the budget violation in its `errors=N` summary count for
  over-budget input the parser already rejects (exit codes unchanged; the LSP
  is unaffected — it takes fail-fast errors from `parse_source`, not from
  `parse_cst`). Workspace 2004/0.

### RB-4a — rowan unification (W-REBUILD Foundation)

- **Removed:** `garnet-parser-v0.3/src/cst.rs` — #221's post-hoc parser CST,
  the self-described "temporary legacy migration oracle" — plus its four
  `parse_source_cst*` wrappers and `tests/cst_round_trip.rs`. The recorded
  deletion precondition (2026-05-24 coordination-ledger entry: rowan-backed
  LSP rename/semantic-token coverage green) was verified met before deletion:
  the LSP has been fully rowan-backed since S16 (zero legacy-CST imports
  anywhere in the workspace — the "remaining consumer paths" the spec
  anticipated turned out to be already migrated; this slice's real work was
  the oracle retirement and its truth surfaces). `garnet-cst` (rowan) is
  the one CST substrate (the AST remains a parallel structure until
  RB-4b's typed views; `tree-sitter-garnet/` is a separate editor-tooling
  grammar).
- **Changed (test architecture):** the S15-Compare reconciliation
  differential (`parser_cst_token_parity.rs`, rowan-vs-legacy token view)
  retired WITH its oracle and is succeeded by
  `garnet-cst/tests/token_view_parity.rs` — rowan's token view compared
  **directly against `garnet_parser::lex_source`** (the shared surface the
  legacy CST re-threaded). Broader admission criterion, stated precisely:
  the successor does not require parse success (the old differential did) —
  measured truth is the old test skipped ZERO corpus files (all 39 parse),
  so the no-parse-needed path is exercised by explicit
  lexable-but-unparseable inline cases, and a `compared == corpus` guard
  makes the corpus sweep non-vacuous. The all-corpus parse-success gate the
  legacy test incidentally provided is restored as
  `garnet-parser-v0.3/tests/examples_parse.rs` (review finding).
  Byte-identical round-trip coverage continues on the same corpus via the
  pre-existing `garnet-cst/tests/examples_roundtrip.rs` (+ proptest).
- **Truth surfaces:** the mit-readiness `parser_cst_layer` lane now cites
  the rowan substrate as its evidence (status/percent unchanged:
  verified/100; readiness stays 92.8, `--check-no-regression` PASS,
  `truth --check` ok) — the lane's intent (trivia-preserving CST with
  byte-identical proof) was already carried by `garnet-cst`.
  `CURRENT_STATE.md` + `garnet-cst` crate docs/AGENTS.md updated from
  "remains a temporary oracle" to the completed retirement.
- **Honest boundary:** error-recovery + incremental reparsing on the
  unified substrate stay queued for the playground band (unchanged); RB-4b
  (typed AST views + the per-pass caps re-check criterion) is the next
  slice, not this one. Pre-existing on main and NOT addressed here: 6
  machine-local `test_garnet_mit_readiness_status.py` failures from
  Desktop-evidence drift vs pinned distribution-lane percentages
  (identical failure set on clean `06fa88b`; flagged as a follow-up).

### RB-3 — registry-derived dispatch (W-REBUILD Foundation · keystone)

- **Changed:** the interpreter's native installation is now DERIVED — one
  loop joining `garnet_stdlib::registry::all_prims()` against the
  `#[garnet_primitive]` adapter table (new `garnet-prim-macros` proc-macro
  crate: per-adapter attribute + per-module `entries()` collector; zero new
  external deps — proc-macro2/quote/syn were already locked). The 82
  hand-written `define_native` registrations are deleted; binding mode and
  the runtime caps-backstop class now live in two new `PrimMeta` columns
  (`Binding`: 22 bare / 56 qualified / 2 unbridged; `Guard`: 12 gate /
  3 gate+entry / 65 declared), and the bridge stopped hand-duplicating
  arity (it reads the registry's existing arity column) — the registry row
  is the single dispatch declaration. The registry's old header claim ("the interpreter calls
  `all_prims()` at startup") was FALSE before this slice — the two lists
  were hand-synced; that drift class is retired, and the claim is now true.
- **Differential proof:** before deletion, the derived table was proven
  IDENTICAL to the verbatim legacy list — same 82 bound names, display
  names, arities, and adapter fn POINTERS (structural behavioral
  equivalence; evidence in the PR's prior commit + bundle). Permanent
  trap tests: registry-join totality (both directions), `BRIDGE_ONLY`
  exactness (the 4 caps-invisible `memory::*` natives, documented), and
  `guard_column_matches_runtime_backstop_behavior` — every bridged prim is
  driven from a `@caps()` frame and Gate/GateEntry rows must caps-trap
  while Declared rows must not, binding the Guard column to adapter
  behavior. Zero behavior change otherwise; all textual gate scripts
  (promotion, layer, caps-enforcement, ffi-authority) verified PASS
  against the rewired sources; mit-readiness unchanged (92.8).
- **truth.json:** `primitive_count`/`primitives_by_layer` still derive via
  `all_prims()` — which is now genuinely the dispatch source, so the
  truth-marker chain (README/FAQ/site) sits on the real registry. The
  stale `--version` banner ("22 bridged primitives") is now
  registry-derived.
- **Recorded deviations (method + mechanism):** (a) the spec's
  differential asks for "a shared fixture corpus through old and new
  dispatch"; what shipped is STRONGER-OR-EQUAL but different in method —
  table identity down to adapter **fn pointers** (the same machine code
  runs for every primitive, subsuming corpus execution) plus the
  guard-behavior sweep driving every bridged prim through the new
  dispatch. (b) "all_prims() and install() become derived": install() is
  derived; `all_prims()` itself remains the hand-written static table
  (now carrying the dispatch columns) — it IS the declaration, the gates
  parse it as text. (c) The spec's attribute shape carries metadata
  (module=/caps()/arity=/layer=/stability=/doc=) in the attribute; the
  implemented attribute carries only the key, metadata stays in the
  registry row — one declaration per primitive either way; this placement
  keeps the textual gates green and the checker/xtask dependency
  direction intact. (d) "all (80 at audit) primitives registered via
  attribute" reconciles as 78 adapter-registered + 2 deliberately
  Unbridged registry rows (pre-existing) + 4 attribute-registered
  BRIDGE_ONLY natives outside the registry. All four flagged for the
  RB-band stop report.
- **Honest partial (LOC criterion — spec deviation):** the spec expected
  "net LOC drops by roughly two thousand lines" via arm deletion. Measured
  reality: net **+752** lines (`git diff --stat`: +1954/−1202 across 16
  files, incl. the new macro crate, tests, and ledgers). Adapter BODIES could not be deleted — the
  caps-enforcement gate greps this file's literal `require_capability`
  text and the `Value`-conversion logic is real code, not boilerplate; what
  was deleted is the registration list and its drift class. Recorded as a
  deviation with the architecture rationale, flagged for the RB-band stop
  report.
- **Doc strings:** already present on all 80 rows; the row contract now
  names them load-bearing-to-be (RB-7 `?doc` WILL be their first consumer
  — nothing reads `PrimMeta.doc` yet).

### RB-2 — crash-surface sweep (W-REBUILD Foundation)

- **Changed (abort → diagnostic, both backends):** `i64::MIN / -1` and
  `i64::MIN % -1` were uncontrolled aborts in the interpreter (expression
  AND compound-assign paths) and the VM — surfacing as an unwind exit-101
  on the VM/eval lanes and as the generic "interpreter thread panicked"
  firewall line on the `run --interp` lane. Now checked division/remainder yields
  `RuntimeError::Overflow` / `VmError::Runtime` with the **identical**
  message (`integer overflow: <lhs> <op> <rhs>`) on both backends, proven
  red→green (`garnet-interp tests/overflow_guards.rs`) and by a
  trap-parity-style cross-backend test
  (`garnet-cli/tests/overflow_parity.rs`). The new `Overflow` error is not
  `rescue`-able (the abort it replaces was not either); making it rescuable
  is a language decision. Honest boundary: add/sub/mul overflow (wraps in
  release, aborts in debug) is an open language-policy decision —
  named-deferred, flagged for Jon's decision via the RB-band stop report,
  not silently changed.
- **Added (deny lints):** `#![deny(clippy::unwrap_used, clippy::expect_used)]`
  on `garnet-cli` (lib + bin targets), `garnet-interp`, `garnet-stdlib`
  (tests exempt via `cfg_attr`; integration tests/benches are separate
  crates outside the deny — the scope is the lib + bin targets). Production
  unwrap/expect sites: 8 found (9 raw occurrences — the doc example spans
  two lines), 8 resolved — 4 refactored away (let-else / pop-then-match / spawn-failure
  path), 2 allowlisted `// INVARIANT:` (len==1-guarded pop; Hmac
  any-key-length), 1 allowlisted `// FAIL-CLOSED:` (`machine_key` — cache
  integrity must not fail open; **a second sanctioned comment form beyond
  the spec's single INVARIANT pattern, recorded as a deviation** because
  calling it an invariant would be false), 1 doc-example rewritten to `?`
  (rewrite-only: clippy does not lint doctests, so the deny guards the 7
  code sites).
  Deny proven live by a planted-unwrap check (fires → removed → clean).
- **Added (malformed-input smoke):** `garnet-cli/tests/malformed_corpus_smoke.rs`
  drives `garnet check` + `run --interp` + `run --vm` over a 12-file
  malformed fixture corpus (+ non-UTF8 input, + the 13 parser fuzz seeds
  via `check`), asserting controlled 0/1/2 exits — no aborts, no panic
  exits. Fuzz: `cargo +nightly fuzz run parse_input -- -max_total_time=600`
  → **18,178,935 executions in 601 s, zero crashes** (machine-local,
  2026-06-12, this MacBook Pro; full stats in the RB-2 dogfood bundle
  log). **Scoped claim: no abort on this corpus + those 10 fuzz minutes on
  that machine — never "never panics."** Unbounded un-annotated recursion
  is excluded by design (S99 opt-in-ceiling boundary). Named-deferred: the
  `eval`/`repl`/`test` CLI lanes run the interpreter in-process with NO
  panic firewall (only `garnet run` has the thread join) — a runtime panic
  there is still an uncontrolled exit; firewalling those lanes is a
  follow-up slice.
- **Honest partial (miette spans — spec deviation):** the RB-2 spec text
  orders "miette diagnostics with spans"; parse-layer diagnostics already
  carry spans end-to-end, but the converted runtime aborts surface as
  span-less `RuntimeError`/`VmError` messages — threading spans through
  eval is deferred. **Recorded as a deviation from the spec accept-when
  (same treatment as the FAIL-CLOSED comment form), not re-scoped away**;
  flagged for the RB-band stop report.
- **Known pre-existing cross-backend divergence (unchanged by RB-2):**
  interp `%= 0` falls to the compound-assign catch-all ("compound
  assignment on unsupported types") while the VM reports "division by
  zero". Fixing it changes an observable error string — deferred to its
  own slice (closed by the RB-2 follow-up entry above).

### RB-1 — caps lattice → `CapSet(u16)` bitset (W-REBUILD Foundation)

- **Changed:** the CapCaps propagator's capability representation is now
  `garnet_check::CapSet` — a `Copy` `u16` bitset over the closed capability
  set (fs, net, net_internal, time, proc, ffi, env, `*`; bits 9–15 reserved,
  bit 8 = unknown-declared presence marker). Propagation = bitwise OR over
  the call graph; subset = `required & !declared == 0`; diff-caps delta =
  XOR. **Zero language-semantics change**: a proptest differential suite ran
  random cap-sets over random call graphs (cycles, wildcard, unknown names,
  qualified/bare prims) through the old `BTreeSet<String>` impl and the new
  bitset impl and required identical violations (content + order) and
  transitive sets before the old impl was deleted (same PR; evidence in the
  PR's first commit and the dogfood bundle). Permanent property coverage:
  the `CapSet`-vs-`BTreeSet` model suite (`capset.rs`) and the diff-caps
  reference-oracle suite (`caps_diff.rs`). A registry-drift trap test fails
  closed if a registry capability ever lacks a `CapSet` bit.
- **Added:** `garnet diff-caps --machine` (Directive 15) — deterministic
  single-line JSON verdict (`garnet.diff-caps.machine/1`: verdict, band,
  exit code, gained/removed caps, per-function expansions, wildcard flag)
  for agent reviewers. Purely additive: human text output is byte-stable
  (golden-pinned by test below the path-bearing header line) and exit codes
  are unchanged; exit 2 (usage/parse error) emits no JSON — stdout empty,
  error on stderr.
- **Honest partial (Directive-15 "bounds deltas"):** the RB-1 accept-when
  names "bounds deltas" in the machine payload. Bound annotations
  (`@bounded`/`@max_depth`/`@mailbox`) are not part of the declared-caps
  surface diff-caps reads, so that field is **deliberately absent** — the
  JSON `scope` field says so explicitly. Recorded as a spec deviation, not
  silently narrowed: closing it needs a caps-surface/manifest extension
  (human-merge-only territory) or an accept-when amendment — escalated to
  Jon in the RB-band stop report.
- **Honest partial (clone criterion):** the spec's "`.clone()` 201 → <40 in
  `garnet-check/src`" is recorded as measured-partial. Measured baseline at
  `f03d414` was 188 (not 201); capability-**set** clones were 7 of those and
  are now **0** (the R3 verdict's subject). Total now 185: the remaining
  mass is branch-state dataflow snapshots (match_coverage 115, borrow 23)
  and owned `String` map keys — the RB-5 env/interner rebuild is the
  designated vehicle; rewriting those passes here would have widened the
  slice and risked semantic drift for no caps benefit.
- **Perf note (machine-local, this Mac only):** 120-fn chain × 200
  propagations (release): old set-based ≈ 59 ms, new `CapSet` ≈ 33 ms.
  Nothing broader is claimed.

## [0.8.1] — 2026-06-05 (re-cut signed 2026-06-07)

> **Cut truth (S120):** **`v0.8.1` is cut** — Jon Isaac tagged annotated
> `v0.8.1` (the S119 merge) and pushed it to `Island-Dev-Crew/garnet` on
> 2026-06-05 under an explicit two-gate human confirmation. Tags on origin are
> now `v0.4.2`, `v0.5.0`, `v0.8.0`, **`v0.8.1`**. v0.8.1 is a **research-grade
> milestone, not production/1.0**.
>
> **Binary note (re-cut 2026-06-07, S130):** after the in-tree `version` was
> bumped 0.5.0→0.8.1 (S123) and the release pipeline gained a tag==version guard
> (S124) + SBOM/signing (S125), the `v0.8.1` Release was **re-cut from `8107c01`**
> and now ships **signed `garnet-0.8.1-*` CLI binaries** (`.deb`/`.rpm`/darwin
> tarballs) + a CycloneDX SBOM + a GPG-signed `SHA256SUMS.asc` (verify per
> `docs/release-signing.md`). The S91–S120 trust-kernel work is now in the
> published binary. (The original 2026-06-05 cut shipped the older
> `garnet-0.5.0-*` build; the `v0.8.0` tag still does.)

> **Post-cut release truth (2026-05-31, S83):** **`v0.8.0` is cut** — Jon Isaac
> tagged annotated `v0.8.0` → `cc165e8` (the S80 merge) and pushed it to
> `Island-Dev-Crew/garnet`; tags on origin are now `v0.4.2`, `v0.5.0`, **`v0.8.0`**.
> The entries from **S31–S80** shipped under that `v0.8.0` tag (a research-grade
> milestone, not production/1.0); entries from **P0 / S81+** are the **v0.8.1
> runway** (the Windows-audit burn-down + runtime-enforcement seeds). A full
> Keep-a-Changelog restructure into a dated `[v0.8.0]` section + any retroactive
> `v0.6.0`/`v0.7.0` tagging remains a deferred release-truth decision for Jon — not
> done here; this note records the cut truth in one place (closes WIN-S80-002).
>
> Earlier S31 note (preserved): only `v0.4.2`/`v0.5.0` were tagged before the
> v0.8.0 cut; the `v0.6.0`/`v0.7.0` "in flight" labels were planning targets that
> never cut a tag, and their slices (S11–S30) folded forward under `v0.8.0`.

### Added

- **S107 (v0.8.1 runway - Mac-Codex domain execution proof):** adds
  `scripts/smoke_garnet_mac_domain_proofs.py`, a manifest-backed recorder/gate
  for the independent macOS S105 domain row. The committed proof under
  `proofs/mac/domains/` runs the release `garnet` binary across all six S105
  domains: net-egress widening refusal, supply-chain `proc` escalation refusal,
  enforced `@max_depth` trap, accept-path provenance dossier, PR-review
  diff-caps wedge, and static `mcp-caps` authority-creep report. The accept path
  emits the four trust artifacts plus `decision.md` and verifies the
  transparency-log chain; negative/report domains are intentionally unsealed.
  The MIT readiness reporter now exposes `macos_domain_execution_proof` as the
  committed Mac row for S109 consolidation. **Honest scope:** this is macOS
  domain execution evidence only, not Windows/Linux completion, not macOS
  OS-sandbox enforcement, not Wasmtime fuel, not production readiness, and not a
  v1.0 claim.
- **Mac Studio UI proof (v0.8.1 runway - Mac-Codex Studio lane):** the packaged
  macOS Tauri app now exposes `Release / Readiness -> Mac Domain Proofs`, which
  invokes the same six-domain Mac proof recorder through the UI. The committed
  proof under `proofs/mac/studio-ui/` includes a window screenshot, the
  Computer Use click sequence, a rebuilt `.app` command log, and a copied
  verified target evidence bundle. The MIT readiness reporter now exposes
  `macos_studio_ui_domain_proof` as committed evidence. **Honest scope:** this
  is UI-wrapper evidence for the Mac proof recorder; it does not individually
  open every source file through a native picker, does not claim Windows/Linux
  ownership, and does not claim production/v1.0 enforcement.

- **S108 Linux row (v0.8.1 runway - UTM enforcement proof):** adds
  `scripts/garnet_linux_cross_os_enforcement_proof.py`, a manifest-backed
  recorder/gate for the independent Linux S108 row. The committed proof under
  `proofs/linux/enforcement/` reruns the S101 Stage-V gate and the bounded/caps
  integration traps on the Mac #2 UTM Debian 12 ARM64 guest, then applies the
  generated seccomp policy with `tools/seccomp-apply/prove.sh` for three
  deterministic denied-syscall trap runs plus a policy-driven allowed
  `@caps(fs, net)` socket case. The MIT readiness reporter now exposes
  `linux_cross_os_enforcement_proof` as committed S108 evidence. **Honest
  scope:** this is Linux S108 evidence for S109 consolidation only, not
  Windows/macOS OS-sandbox enforcement, not full S109 completion, not Wasmtime
  fuel, and not production/v1.0 readiness.

- **S109 Mac row (v0.8.1 runway - Mac-Codex cross-OS matrix row):** adds
  `scripts/smoke_garnet_mac_cross_os_matrix.py`, a manifest-backed recorder/gate
  for the Mac rows of the S109 trap matrix. The committed proof under
  `proofs/mac/matrix/` reruns the Mac Stage-V trap gates for `@max_depth`,
  `@caps`, and diff-caps rejection, compares against the committed Windows/WSL
  baselines, and records OS-independent accept artifacts as byte-identical where
  required. The accept seal is field-identical for the OS-independent subject,
  AST, capability-manifest, and attestation fields while the full JSON differs
  honestly on `prelude_hash`; `diff_caps.txt` full text differs only on absolute
  OS paths and has a matching path-independent verdict body. The MIT readiness
  reporter now exposes `macos_cross_os_matrix_row` as committed evidence.
  **Honest scope:** this is the Mac row for S109 consolidation, not full S109
  completion. The independent Linux S108 row was absent when this Mac-row bundle
  was recorded; WSL remains execution/portability only rather than Linux seccomp
  or OS-sandbox enforcement, and this is not a production/v1.0 claim.

- **S109 full matrix (v0.8.1 runway - cross-OS trap parity consolidation):**
  adds `scripts/garnet_cross_os_trap_parity_matrix.py`, a manifest-backed
  recorder/gate for the post-S108 Win×Mac×Linux trap matrix. The committed proof
  under `proofs/cross-os/matrix/` verifies the Windows S106 proof, the Mac S109
  row, and the Linux S108 UTM proof, then records a Linux UTM `diff-caps`
  rejection datapoint from Debian at `origin/main` `29f12e0`. The MIT readiness
  reporter now exposes `cross_os_trap_parity_matrix` as committed evidence.
  **Honest scope:** `cross_os_complete=true` applies only to the three named
  S109 traps (`@max_depth`, `@caps`, diff-caps rejection). WSL remains
  execution/portability evidence and is explicitly excluded from Linux
  enforcement; Linux seccomp is Linux-only evidence, not a Windows/macOS
  OS-sandbox claim; this is not Wasmtime fuel, production, release/tag, S120, or
  v1.0 readiness.

- **S117 consolidation (v0.8.1 runway - Linux/Tauri gate replay proof):**
  adds `scripts/smoke_garnet_studio_linux_gate_replay.py`, a manifest-backed
  recorder/gate that replays the current committed WSL/WSLg package, runtime,
  display, domain-shell, and Release / Readiness shell gates from one
  repo-owned command. The proof is committed under
  `proofs/linux/execution/studio-gate-replay/` with stdout/stderr from each
  child gate and a `MANIFEST.sha256`. The MIT readiness reporter now exposes
  `linux_tauri_gate_replay`, moving overall readiness to **92.3%** and the
  Windows/Linux distribution lane to **83.0%** when the existing Windows clean-VM,
  package, runtime, WSLg, domain-shell, Release / Readiness, and replay bundles
  all verify. **Honest scope:** this is a consolidation replay of WSL/WSLg
  execution/portability evidence only. It does not prove clean/non-WSL Linux
  desktop GUI install/launch, Linux seccomp, OS-sandbox enforcement, signed/SBOM
  release artifacts, winget, Windows ARM64, production readiness, or v1.0
  readiness.

- **S117 increment (v0.8.1 runway - Windows/WSL Studio Release / Readiness
  shell proof):** adds `scripts/smoke_garnet_studio_release_readiness_shell.py`,
  a manifest-backed recorder/gate for the Studio binary's
  `--studio-release-readiness-smoke` path on Windows and WSL. The command routes
  through the Tauri command wrappers behind the Release / Readiness status
  reporters (`windows_linux_studio_status`, objective pulse, converter status,
  and Windows clean-VM installer status), records copied Studio payloads, command
  logs, manifests, and honest-scope flags under
  `proofs/windows/studio-release-readiness-shell/` and
  `proofs/linux/execution/studio-release-readiness-shell/`. The MIT readiness
  reporter now exposes this as
  `windows_wsl_studio_release_readiness_shell_proof`, moving overall readiness to
  **92.1%** and the Windows/Linux distribution lane to **82.0%** when the
  Windows clean-VM, domain, Studio-smoke, WSL `.deb`, WSL `.rpm`, Xvfb
  runtime-start/window-capture, WSLg system install/launch, domain-shell, and
  Release / Readiness shell bundles all verify. **Honest scope:** the Windows row
  proves the local Studio command wrappers for repo-native status reporters; the
  WSL row is execution/portability evidence only. This does not prove a live GUI
  screenshot, clean/non-WSL Linux desktop GUI install/launch, Linux seccomp,
  OS-sandbox enforcement, signed/SBOM release artifacts, winget, Windows ARM64,
  production readiness, or v1.0 readiness.

- **S117 increment (v0.8.1 runway - Windows/WSL Studio domain-shell proof):**
  adds `scripts/smoke_garnet_studio_domain_shell.py`, a manifest-backed
  recorder/gate for the Studio binary's `--studio-domain-proof-smoke` path on
  Windows and WSL. The command routes through the Tauri command wrapper around
  the repo Domain Proof Matrix, records the Studio payload, command logs,
  manifests, and copied domain-matrix evidence under
  `proofs/windows/studio-domain-shell/` and
  `proofs/linux/execution/studio-domain-shell/`. The MIT readiness reporter now
  exposes this as `windows_wsl_studio_domain_shell_proof`, moving overall
  readiness to **92.0%** and the Windows/Linux distribution lane to **81.0%**
  when the Windows clean-VM, domain, Studio-smoke, WSL `.deb`, WSL `.rpm`,
  Xvfb runtime-start/window-capture, WSLg system install/launch, and
  domain-shell bundles all verify. **Honest scope:** the Windows row proves the
  local Studio command wrapper around the Domain Proof Matrix; the WSL row is
  execution/portability evidence only. This does not prove clean/non-WSL Linux
  desktop GUI install/launch, Linux seccomp, OS-sandbox enforcement,
  signed/SBOM release artifacts, winget, Windows ARM64, production readiness,
  or v1.0 readiness.

- **S117 increment (v0.8.1 runway - WSLg Linux Studio system package
  install/launch proof):** adds
  `scripts/smoke_garnet_studio_linux_wslg_install_launch.py`, a
  manifest-backed recorder/gate for building the Linux Tauri `.deb`, installing
  it inside WSL with `dpkg -i`, verifying `/usr/bin/garnet-studio
  --studio-smoke`, launching the installed binary through WSLg/X11, capturing
  the `Garnet Studio` window tree with `xwininfo`, and removing the package
  after the proof. The committed bundle under
  `proofs/linux/execution/studio-wslg-system-install/` records WSL `uname`,
  WSLg display variables, npm/Tauri build logs, `dpkg` before/install/remove
  status, installed binary metadata, smoke output, WSLg launch/window evidence,
  cleanup proof, and command logs. The MIT readiness reporter now exposes this
  as `linux_wsl_studio_wslg_system_install_launch`, moving overall readiness to
  **91.9%** and the Windows/Linux distribution lane to **80.0%** when the
  Windows clean-VM, domain, Studio-smoke, WSL `.deb`, WSL `.rpm`, Xvfb
  runtime-start/window-capture, and WSLg system install/launch bundles all
  verify. **Honest scope:** this is WSLg system-package install/launch
  portability evidence only; it does not prove clean/non-WSL Linux desktop GUI
  install/launch, Linux seccomp, OS-sandbox enforcement, signed/SBOM release
  artifacts, winget, Windows ARM64, production readiness, or v1.0 readiness.

- **S117 increment (v0.8.1 runway - WSL Linux Studio Xvfb virtual-display
  window-capture proof):** adds
  `scripts/smoke_garnet_studio_linux_wsl_xvfb_window.py`, a manifest-backed
  recorder/gate for launching the extracted Linux Tauri Studio binary under WSL
  `xvfb-run`, verifying an X11 window tree that contains `Garnet Studio`, and
  capturing a virtual-display screenshot artifact. The committed bundle under
  `proofs/linux/execution/studio-xvfb-window-capture/` records WSL `uname`,
  X11/Xvfb capture tooling (`xwininfo`, `xdpyinfo`, `xwd`, ImageMagick
  `convert`/`identify`), the source package proof, extracted-binary hash, window
  tree, screenshot hash/size, and command logs. The MIT readiness reporter now
  exposes this as `linux_wsl_studio_xvfb_window_capture`, moving overall
  readiness to **91.7%** and the Windows/Linux distribution lane to **79.0%**
  when the Windows clean-VM, domain, Studio-smoke, WSL `.deb`, WSL `.rpm`, Xvfb
  runtime-start, and Xvfb window-capture bundles all verify. **Honest scope:**
  this is WSL Xvfb virtual-display window-capture evidence only; it does not
  prove Linux desktop GUI install/launch, Linux seccomp, OS-sandbox enforcement,
  clean Linux install, privileged system package install, signed/SBOM release
  artifacts, winget, Windows ARM64, production readiness, or v1.0 readiness.

- **S117 increment (v0.8.1 runway - WSL Linux Studio Xvfb runtime-start proof):**
  adds `scripts/smoke_garnet_studio_linux_wsl_xvfb.py`, a manifest-backed
  recorder/gate for starting the extracted Linux Tauri Studio binary under WSL
  `xvfb-run` and requiring timeout exit `124` as the pass signal. The committed
  bundle under `proofs/linux/execution/studio-xvfb-runtime/` records WSL
  `uname`, `xvfb-run`/`timeout` tooling, display variables, stdout/stderr, the
  source package proof, extracted-binary hash, timeout duration, and the runtime
  command exit code. The MIT readiness reporter now exposes this as
  `linux_wsl_studio_xvfb_runtime`, moving overall readiness to **91.6%** and the
  Windows/Linux distribution lane to **78.0%** when the Windows clean-VM, domain,
  Studio-smoke, WSL `.deb`, WSL `.rpm`, and Xvfb runtime-start bundles all
  verify. **Honest scope:** this is WSL Xvfb runtime-start evidence only; it
  does not prove Linux desktop GUI install/launch, Linux seccomp, OS-sandbox
  enforcement, clean Linux install, privileged system package install,
  signed/SBOM release artifacts, winget, Windows ARM64, production readiness, or
  v1.0 readiness.

- **S117 increment (v0.8.1 runway - WSL Linux Studio `.rpm` package proof):**
  adds `scripts/smoke_garnet_studio_linux_wsl_rpm.py`, a manifest-backed
  recorder/gate for a WSL-driven Tauri Linux `.rpm` package build, RPM
  metadata/content inspection, payload extraction, and extracted-binary command
  smoke. The committed bundle under `proofs/linux/execution/studio-rpm-package/`
  records WSL `uname`, RPM tooling setup, Linux-side `npm install
  --include=optional`, frontend build, `tauri build --bundles rpm`, `rpm -qip`,
  `rpm -qlp`, `rpm2cpio | cpio` extraction, extracted binary listing, and the
  extracted Linux binary's non-GUI `--studio-smoke`. The MIT readiness reporter
  now exposes this as `linux_wsl_studio_rpm_package`, moving overall committed
  readiness to **91.4%** and the Windows/Linux distribution lane to **77.0%**
  when the Windows clean-VM, domain, Studio-smoke, WSL `.deb`, and WSL `.rpm`
  package/extract bundles all verify. **Honest scope:** this is WSL
  package-extract and command-smoke evidence only; it does not prove Linux
  desktop GUI install/launch, Linux seccomp, OS-sandbox enforcement, clean Linux
  install, privileged system package install, signed/SBOM release artifacts,
  winget, Windows ARM64, production readiness, or v1.0 readiness.

- **S117 increment (v0.8.1 runway - WSL Linux Studio `.deb` install/extract proof):**
  adds `scripts/smoke_garnet_studio_linux_wsl_deb_install.py`, a
  manifest-backed recorder/gate for WSL `.deb` extraction and extracted-binary
  command smoke. The committed bundle under
  `proofs/linux/execution/studio-package-install/` records WSL `uname`,
  Linux-side `npm install --include=optional`, frontend build, `tauri build
  --bundles deb`, `dpkg-deb --info`, `dpkg-deb --contents`,
  `dpkg-deb --extract`, extracted binary listing, and the extracted Linux
  binary's non-GUI `--studio-smoke`. The MIT readiness reporter now exposes this
  as `linux_wsl_studio_deb_install`, moving overall committed readiness to
  **91.3%** and the Windows/Linux distribution lane to **76.0%** when the
  Windows clean-VM, domain, Studio-smoke, WSL package-build, and WSL
  install/extract bundles all verify. **Honest scope:** this is WSL
  package-extract and command-smoke evidence only; it does not prove Linux
  desktop GUI install/launch, Linux seccomp, OS-sandbox enforcement, clean Linux
  install, privileged system package install, signed/SBOM release artifacts,
  winget, Windows ARM64, production readiness, or v1.0 readiness.

- **S117 increment (v0.8.1 runway - WSL Linux Studio `.deb` package proof):**
  adds `scripts/smoke_garnet_studio_linux_wsl_deb.py`, a manifest-backed
  recorder/gate for a WSL-driven Tauri Linux `.deb` package build. The committed
  bundle under `proofs/linux/execution/studio-package/` records WSL `uname`,
  Linux-side `npm install --include=optional`, frontend build, `tauri build
  --bundles deb`, the Linux binary's non-GUI `--studio-smoke`, and `dpkg-deb`
  package inspection. The MIT readiness reporter now exposes this as
  `linux_wsl_studio_deb_package`, moving overall committed readiness to **91.1%**
  and the Windows/Linux distribution lane to **75.0%** when the Windows clean-VM,
  domain, Studio-smoke, and WSL package bundles all verify. **Honest scope:** this
  is WSL package-build and command-smoke evidence only; it does not prove Linux
  desktop GUI install/launch, Linux seccomp, OS-sandbox enforcement, a clean Linux
  install, signed/SBOM release artifacts, winget, Windows ARM64, production
  readiness, or v1.0 readiness.

- **S117 increment (v0.8.1 runway - Windows/WSL Studio smoke proof):** adds
  `scripts/smoke_garnet_studio_windows_wsl.py`, a manifest-backed recorder for
  the Windows Tauri Studio `--studio-smoke` path plus a WSL command-contract
  replay. The committed Windows bundle under `proofs/windows/studio/` records
  frontend dependency install/build when needed, `npm run build`, the Tauri
  release build, the `--studio-smoke` invocation, and the generated
  `studio-smoke.json` with `source_included=false` and `provider_api_called=false`.
  The committed WSL bundle under `proofs/linux/execution/studio/` records WSL
  `uname`, the Windows/Linux Studio status JSON, and the status regression tests.
  The MIT readiness reporter now exposes this as `windows_wsl_studio_smoke`,
  moving committed readiness to **90.9%** when both bundles verify. **Honest
  scope:** this is a package-pipeline proof increment, not full S117 completion;
  WSL is execution/portability only and does not prove Linux seccomp,
  OS-sandbox enforcement, Wasmtime fuel, Linux desktop GUI launch, native Linux
  packages, signed MSI, winget, Windows ARM64, production readiness, or v1.0
  readiness.

- **S110 (v0.8.1 runway - Windows/WSL ultrapunch reproduction):** adds
  `scripts/smoke_garnet_ultrapunch_repro.py`, a manifest-backed recorder for the
  S104 ultrapunch replay. The committed Windows bundle under
  `proofs/windows/ultrapunch/` and WSL bundle under `proofs/linux/repro/` both
  replay ACCEPT (four trust artifacts retained plus transparency-log verification)
  and REJECT (capability widening refused by diff-caps, over-depth proposal refused
  by the enforced-kernel trap, no reject seal written). The MIT readiness reporter
  now exposes this as `windows_wsl_ultrapunch_repro`, moving committed readiness to
  **90.8%** when both bundles verify. **Honest scope:** the WSL row is
  portability-repro only; it does not prove Linux seccomp, OS-sandbox enforcement,
  Wasmtime fuel, Linux desktop/Tauri GUI launch, production readiness, or v1.0
  readiness.

- **S111 (v0.8.1 runway - Windows/WSL domain proof reproduction):** records
  committed Garnet Studio domain-matrix proof bundles for Windows and WSL under
  `proofs/windows/domains/` and `proofs/linux/execution/domains/`. Both bundles
  run the current 20-example `--suite all` matrix through parse/check/run
  (60/60 commands), preserve source omission and provider-free execution, and
  include the expected signed-hot-reload BLAKE3 mismatch rejection. The MIT
  readiness reporter now accepts this committed proof pair before falling back
  to Desktop-local evidence, so the domain-matrix lane can become committed
  evidence when both bundles verify. **Honest scope:** the WSL bundle is
  execution/portability evidence only; it does not prove Linux seccomp,
  OS-sandbox enforcement, Wasmtime fuel, or Linux desktop/Tauri GUI launch.

- **S106 Phase 1 (v0.8.1 runway - Windows cross-OS enforcement proof):** adds a
  committed Windows proof lane for the already-merged Stage V traps. The new
  `scripts/garnet_windows_cross_os_enforcement_proof.py` recorder/gate captures
  Windows evidence under `proofs/windows/enforcement/` and WSL evidence under
  `proofs/linux/execution/`, proving that the S101 `@max_depth` parity gate,
  `@caps(env/proc/fs/net)` host-authority traps, and the S92 program-entry
  `@caps(proc)` trap all rerun cleanly on this Windows box. The MIT readiness
  reporter now exposes the lane as committed evidence. **Honest scope:** WSL is
  execution/portability, not Linux seccomp or OS-sandbox enforcement; S103
  ultrapunch accept/reject reproduction and S105 domain execution remain Phase 2;
  Wasmtime fuel, memory/time ceilings, and `@mailbox` runtime ceilings remain
  named-deferred.

- **S91 (v0.8.1 substrate - net bridge + program-entry `@caps` frame):** closes
  two named interpreter-scoped enforcement gaps from S90. The `net::tcp_connect`
  bridge now calls `require_capability("net", ...)` before host network policy
  runs, so undeclared network authority traps deterministically with
  `requires @caps(net)` instead of falling through to NetDefaults. `garnet run
  --interp` now invokes `main` through `Interpreter::call_entry`, which installs
  a program-entry capability frame from `main`'s `@caps(...)` annotations before
  dispatch; this covers safe-mode `fn main` entry points that do not push a
  managed frame themselves. Adds three Rust behavior tests (net trap before
  connect policy, safe-entry trap without `@caps(env)`, safe-entry allow with
  `@caps(env)`) and expands `scripts/garnet_caps_enforcement_status.py` to gate
  net + program-entry evidence. Also reinitializes the active dogfood ledger for
  S91-S110 while archiving the finished S31-S80 ledger at `.dogfood/v0_8_goal.json`
  so v0.8.0 release gates stay reproducible. **Honest scope:** interpreter
  host-authority surfaces only; direct host/test calls outside a program frame
  remain allowed; the VM backend still does not enforce `@caps`; S99-S110 are
  ledger-reserved only and not started here.

- **S92 (v0.8.1 substrate - subprocess entry authority guard):** closes the
  interpreter-visible subprocess authority laundering gap. `std::process::spawn`,
  `std::process::spawn_args`, and `std::process::output` now require both a live
  call-chain `@caps(proc)` frame and a program-entry `@caps(proc)` frame, so
  `main @caps()` cannot route through a helper `@caps(proc)` to launch a child
  process. Adds two cross-OS CLI tests for the trap/control pair and
  `scripts/garnet_spawn_ffi_authority_status.py` (+ tests) so the guard and the
  FFI honesty boundary are machine-checkable. **Honest scope:** process launch
  bridges only; `wait`/`exit_code` still require call-chain `proc` but do not
  launch new authority; direct host/test calls outside a program-entry frame
  remain allowed; executable FFI runtime enforcement is deferred because no FFI
  bridge exists yet; Linux seccomp/OS-policy application and VM `@caps`
  enforcement remain unclaimed.

- **S93 (v0.8.1 substrate - static bounded-loop verifier):** adds a
  checker-visible static loop proof seed for the safe subset. `garnet check`
  now rejects uncheckable loops in `fn`, `@safe`, and `@bounded(...)` functions
  with `check.bounded_loop`, while accepting literal finite `for` loops over
  integer ranges/arrays, literal counter `while` loops, and loop bodies that
  exit before a second turn. Adds checker tests, CLI pass/reject tests, and
  `scripts/garnet_bounded_loop_verifier_status.py` (+ tests) so the static
  verifier is machine-checkable. **Honest scope:** static verifier only; this
  does not claim Wasmtime fuel, runtime loop metering, VM loop enforcement, or
  OS sandbox enforcement. Managed functions outside safe / `@bounded` scope are
  intentionally out of scope for this slice.

- **S94 (v0.8.1 substrate - Paper VI Exp 1 provider-gated harness):** adds
  `benchmarks/paper_vi_exp1_llm_pass_at_1/` with a seed-only task manifest,
  provider-free mode, deterministic fixture mode, JSONL result writing,
  aggregation, and cautious analysis output. The new
  `scripts/garnet_paper_vi_exp1_status.py` gate proves three seed tasks, a
  provider-free `pending-infra` run, and fixture-only scoring without a network
  call; the MIT readiness reporter now has a committed-truth S94 lane.
  **Honest scope:** no provider-backed pass@1 measurement is claimed, the full
  500-task corpus remains pending infrastructure, and real LLM execution stays
  behind explicit provider credentials plus `--execute-provider`.

- **S95 (v0.8.1 substrate - Paper VI Exp 3 5K-LOC rerun harness):** adds a
  deterministic 5K-LOC corpus generator and 5K rerun lane under
  `benchmarks/paper_vi_exp3_compiler_as_agent/`, with provider-free
  stateless/history-aware JSONL rows, aggregation, cautious Markdown analysis,
  and `scripts/garnet_paper_vi_exp3_5k_status.py` as the machine-checkable gate.
  The gate proves ten generated snapshots (each at least 5,000 LOC), twenty
  provider-free rerun rows, aggregate/analyze output, and the MIT readiness
  reporter's committed-truth S95 lane. **Honest scope:** no new 5K h3a timing
  measurement is claimed; the recorded v4.0 6.5% partial stands until
  provider-backed 5K runtime rows exist and are reviewed.

- **S96 (v0.8.1 substrate - linear/effect safe-mode seed):** adds the first
  static linear/effect checker increment in `garnet-check-v0.3`. The new pass
  reuses the CapCaps transitive surface and rejects non-entry safe helper
  functions that perform authority effects without an explicit
  ownership-qualified parameter boundary (`own`, `borrow`, `ref`, or `mut`).
  Adds focused checker tests, a machine-checkable
  `scripts/garnet_linear_effect_status.py --gate`, and a committed-truth MIT
  readiness lane. **Honest scope:** first static seed only; this is not
  whole-language linear typing, not VM/runtime capability enforcement, and not
  OS sandbox enforcement. Method-call effect resolution remains limited by the
  current cap graph.

- **S97 (v0.8.1 substrate - provenance seal chain):** extends `garnet seal` with
  `--provenance-chain`, a deterministic S97 block that validates self-declared
  `agent`, `model`, and canonical `prompt_sha256` attestation keys, binds them
  to the current seal's `source_blake3` and subject `artifact_blake3`, and
  records a `chain_blake3` over the canonical declared chain. Adds focused CLI
  tests, a machine-checkable `scripts/garnet_provenance_seal_chain_status.py
  --gate`, S97 attestation-spec text, and a committed-truth MIT readiness lane
  moving readiness to **90.0%**. **Honest scope:** this verifies binding to the
  sealed artifact only; it does not independently prove that the model executed
  the prompt, that the named agent produced the file, that the declared tool list
  is complete, or that the predicate is supply-chain signed.

- **S98 (v0.8.1 substrate - capability-manifest standard seed):** adds
  `garnet caps --standard-profile <path>`, a deterministic
  `capability-manifest/v1` draft/reference profile over the same declared
  capability surface used by the S36 Garnet-native manifest, S37 `diff-caps`, and
  S38 seal embedding. Adds
  `C_Language_Specification/GARNET_CAPABILITY_MANIFEST_STANDARD.md`, RFC-0001
  alignment, JSON test vectors, focused CLI tests, and
  `scripts/garnet_cap_manifest_standard_status.py --gate`; the MIT readiness
  reporter gets a committed-truth S98 lane. **Honest scope:** this is intent plus
  reference implementation seed only; no OWASP/LF or other standards body has
  adopted it, no multi-language ecosystem is claimed, and declared-surface
  manifests do not prove absence of undeclared authority or close the VM
  enforcement gap.

- **S86 (Windows audit binary-strict S80 cut-readiness):** adds
  `--binary-strict` to `scripts/garnet_v0_8_0_cut_readiness.py` plus
  `--windows-audit` as a named alias for the Windows burn-down lane. Default
  mode remains lenient and keeps `--no-run` for the S71/S72/S73 binary-backed
  gates so Python-only CI stays deterministic; strict mode removes `--no-run`
  for those three direct proofs and treats any failure as blocking. The JSON/MD
  output now records `mode` / `binary_strict`. Windows proof:
  `python -B scripts\garnet_v0_8_0_cut_readiness.py --gate --binary-strict
  --format json` exits 0 after S84 and S85 landed, closing WIN-S80-001 without
  cutting or authorizing any tag. **Honest scope:** this hardens the advisory
  S80 evidence gate; it does not change the historical `v0.8.0` tag truth and
  does not claim production readiness.

- **S83 (v0.8.1 runway — post-tag release-truth reconciliation):** reconciles the
  split truth the Windows audit flagged (WIN-S80-002) — `v0.8.0` is cut (Jon, `cc165e8`,
  2026-05-31), yet `GARNET_v0_8_0_CUT.md` still read "READY TO CUT (pending Jon)",
  the CHANGELOG header read "v0.8.0 in flight", and `.dogfood/goal.json` kept `s80`
  pending. Now recorded **in one place**: a post-cut note in `GARNET_v0_8_0_CUT.md`
  + the CHANGELOG `[Unreleased]` header (→ v0.8.1 runway) state that *both* are true
  — the tag was cut by Jon **and** the S80 PR produced cut-readiness evidence only.
  The ledger advances `s80 → merged(5)` with a `cut_record` (goal now **50/50**).
  `scripts/garnet_release_truth_status.py` (+ `--gate`, 5 tests, agent-contracts)
  enforces the two truths coexist. **Honest scope:** pure docs/ledger reconciliation
  — no code, no new tag (the full Keep-a-Changelog restructure stays a deferred
  decision for Jon); no Rust changed.
- **S90 (v0.8.1 runway — `@caps` host-authority runtime enforcement seed):** extends
  runtime enforcement (S89) from `@max_depth` to **capabilities**. The interpreter
  (`garnet-interp-v0.3/src/eval.rs` `require_capability` + `CapsGuard`, wired into the
  `std::env`/`std::process`/`fs::`/`std::log::to_file` bridges in `stdlib_bridge.rs`)
  **traps** when a managed function invokes a host-authority primitive whose required
  capability no frame in the call chain declared — `garnet run` skips the static
  checker, so this is the runtime backstop. A managed fn pushes its declared `@caps`
  onto a per-run thread-local context (RAII-unwound); a primitive is permitted iff the
  **union** of active frames' caps contains the requirement (or `@caps(*)`). Adds
  `garnet-cli/tests/caps_enforcement.rs` (5 cross-OS tests: env/proc/fs traps,
  declared-runs, pure-computation-unaffected), `scripts/garnet_caps_enforcement_status.py`
  (+ `--gate`, 5 tests, agent-contracts), and a `@caps` section in
  `GARNET_BOUNDED_ENFORCEMENT.md`. **Honest scope:** host-authority surfaces only
  (env/proc/fs/log-to-file); pure computation unaffected; a call outside any
  managed-program frame (direct host/test call) is **allowed** (no `@caps` context);
  the **VM** backend does not yet enforce `@caps`. Mac-authored + Mac-tested; the
  Windows trap re-proves via the cross-OS matrix (Windows-proof-pending).
- **S90 Windows proof (`@caps` runtime trap):** records the Windows lane proof for
  the already-merged Mac-authored S90 host-authority enforcement seed in
  `F_Project_Management/WINDOWS_AUDIT_S1_S80.md`. `python -B
  scripts\test_garnet_caps_enforcement_status.py` ran 5/5 OK,
  `python -B scripts\garnet_caps_enforcement_status.py --gate --format json`
  exited 0 with `ok=true`, and `cargo test -p garnet-cli --test caps_enforcement
  -- --nocapture` ran 5/5 OK. A direct Windows `garnet run --interp` fixture whose
  `@caps()` function calls `std::env::get("HOME")` exited 1 with
  `runtime error: capability: std::env::get requires @caps(env), not declared in
  the calling chain`. Honest scope unchanged: interpreter host-authority surfaces
  only; the VM backend still does not enforce `@caps`.
- **S89 (v0.8.1 runway — `@max_depth` runtime enforcement seed):** the first slice
  that makes the trust kernel **enforce** at runtime. A function declaring
  `@max_depth(N)` now **traps deterministically** when its recursion depth exceeds
  `N` — the interpreter (`garnet-interp-v0.3/src/eval.rs`, `call_fn`) tracks
  per-function recursion depth (thread-local, RAII-unwound on every return/error
  path) and returns `bounded: @max_depth(N) exceeded for ...`. Real enforcement
  (the interpreter refuses to recurse further), distinct from the S85 host-stack
  raise. Adds `garnet-cli/tests/bounded_enforcement.rs` (4 cross-OS tests: trap /
  within-ceiling / deterministic / unannotated-not-capped),
  `scripts/garnet_bounded_enforcement_status.py` (+ `--gate`, 5 tests,
  agent-contracts), and spec `C_Language_Specification/GARNET_BOUNDED_ENFORCEMENT.md`.
  **Honest scope:** this is the **ONE enforced ceiling** — `@bounded` (Wasmtime
  fuel), memory, time, and mailbox remain **declared-not-enforced**; unannotated
  functions are not capped (host stack, S85); the **VM** backend does not yet
  enforce `@max_depth` (the parity corpus has no over-ceiling program, so parity
  stays 33/33). Mac-authored + Mac-tested; the Windows trap re-proves via the
  cross-OS matrix (Windows-proof-pending).
- **S87 (v0.8.1 runway — Windows reporter hardening):** hardens the Windows
  proof/reporting lane without changing language semantics. Adds shared
  `scripts/garnet_reporter_io.py` UTF-8 stdout setup and wires it into the S6
  memory-eviction and MIT-readiness reporters so Markdown output survives
  cp1252-style Windows consoles. `scripts/garnet_mit_readiness_status.py` now
  quarantines local promo/temp probe failures as local evidence instead of
  aborting committed-truth readiness, and adds `--committed-only` to emit a
  machine-independent readiness subset. Windows proof: both reporters run in
  cp1252 mode; denied local promo probe degrades to a skipped local lane; two
  committed-only JSON snapshots under different local-evidence roots match SHA256
  `E4B06AAE505DBEC64E9F88FB0B9AC1325106A5CEEB91CF28C4DADEE1FB7C074A`.
  **Honest scope:** this proves the Windows side of the byte-stable surface; a
  Mac runtime comparison should use the same `--committed-only` command.
- **S88 (v0.8.1 runway — Windows release-tooling status):** adds
  `scripts/garnet_release_tooling_status.py` (+ `--gate`, 7 tests) to detect and
  honestly exercise the external tools that were absent during the Windows audit:
  `cosign`, `syft`, `cyclonedx`, and `wasmtime`. On this Windows machine, S88
  provisioned those tools through WinGet, then proved: local `cosign sign-blob` +
  `verify-blob` with an offline temp key (transparency-log verification explicitly
  disabled and named), `syft scan` emitting CycloneDX JSON, `cyclonedx validate`
  accepting a minimal CycloneDX 1.6 BOM, and `wasmtime` running a tiny WAT module
  with both `fuel=1000` and `epoch-interruption=y`/`timeout=1s`. **Honest scope:**
  this verifies local tool availability/runnability only; it does not sign a
  Garnet release artifact, publish an SBOM, or wire Wasmtime fuel/epoch metering
  into Garnet runtime.
- **S89 Windows proof (`@max_depth` runtime trap):** records the Windows lane
  proof for the already-merged Mac-authored S89 enforcement seed in
  `F_Project_Management/WINDOWS_AUDIT_S1_S80.md`. `python -B
  scripts\test_garnet_bounded_enforcement_status.py` runs 5/5 OK,
  `python -B scripts\garnet_bounded_enforcement_status.py --gate --format json`
  exits 0 with `ok=true`, `cargo test -p garnet-cli --test bounded_enforcement`
  runs 4/4 OK, and a direct over-ceiling fixture exits 1 with
  `runtime error: bounded: @max_depth(4) exceeded for deep (recursion depth 5)`.
  **Honest scope:** proof/accounting only; no interpreter/kernel logic changes,
  no VM `@max_depth` claim, and no Wasmtime-fuel, memory, time, mailbox, or
  undeclared-capability trap claim.
- **S81 Windows proof (uppercase `.GARNET` discovery):** records the Windows
  lane proof for the already-merged Mac-authored S81 collector fix in
  `F_Project_Management/WINDOWS_AUDIT_S1_S80.md`. `python -B
  scripts\test_garnet_garnet_ext_discovery_status.py` runs 5/5 OK,
  `python -B scripts\garnet_garnet_ext_discovery_status.py --gate --format json`
  exits 0, and `cargo run -q -p garnet-cli --bin garnet -- verify <temp-dir>`
  exits 1 after discovering uppercase `BAD.GARNET` while reporting lowercase
  `main.garnet` clean. **Honest scope:** proof/accounting only; no collector
  logic changes and no S82/S84/S85/S89/S90 proof claim.
- **S82 Windows proof (seal `source_blake3` LF/CRLF):** records the Windows
  lane proof for the already-merged Mac-authored S82 seal determinism fix in
  `F_Project_Management/WINDOWS_AUDIT_S1_S80.md`. `python -B
  scripts\test_garnet_seal_determinism_status.py` runs 5/5 OK, `python -B
  scripts\garnet_seal_determinism_status.py --gate --format json` exits 0, and
  fresh Windows LF/CRLF seal outputs both exit 0 with identical
  `predicate.source_blake3`:
  `096cb946361fbf2d821452449578fd8f5af3f2a70c3546e763e43d4374d168ad`.
  **Honest scope:** proof/accounting only; no manifest logic changes and no
  S81/S84/S85/S89/S90 proof claim.
- **S85 Windows proof (interpreter stack + VM parity):** records the Windows lane
  proof for the already-merged Mac-authored S85 interpreter large-stack fix in
  `F_Project_Management/WINDOWS_AUDIT_S1_S80.md`. `cargo run -q -p garnet-cli
  --bin garnet -- run --interp .\examples\mvp_function_call_demo.garnet` exits 0
  and prints `=> 7105`; `python -B scripts\garnet_interp_stack_status.py --gate
  --format json` exits 0; and `python -B scripts\garnet_vm_interp_parity.py
  --gate --format json` exits 0 with `binary_available=true`, `parity_ok=33`,
  `corpus_size=33`, and `divergent=[]`. **Honest scope:** proof/accounting only;
  no interpreter or VM logic changes and no unbounded recursion claim.

- **S85 (v0.8.1 runway — interpreter deep-recursion robustness):** the tree-walking
  interpreter (`garnet run --interp`) stack-overflowed on Windows (~1 MiB default
  thread stack) for `mvp_function_call_demo.garnet` while the VM succeeded, so the
  VM/interpreter parity campaign diverged on Windows (WIN-S73-001). The interpreter
  evaluation now runs on a thread with a **256 MiB explicit stack**
  (`std::thread::Builder::stack_size`, the `Interpreter` created inside so nothing
  non-`Send` crosses) on every platform — a robustness fix, not a Windows patch.
  Adds two cross-OS integration tests (`garnet-cli/tests/interp_deep_recursion.rs`:
  the audit fixture runs `=> 7105`; a 5000-deep recursion that overflows the default
  stack runs on the large one) + `scripts/garnet_interp_stack_status.py` (+ `--gate`,
  5 tests, agent-contracts). **Honest scope:** raises the recursion ceiling by
  hundreds× (closes WIN-S73-001 for deep-but-finite recursion) — it is **not** an
  unbounded guarantee; recursion past the large stack still overflows, which is the
  `@bounded` **enforcement** story (S89). Mac-authored + Mac-tested; the original
  Windows fixture re-proves via the cross-OS `cargo test` matrix (end-to-end check
  Windows-proof-pending in `WINDOWS_AUDIT_S1_S80.md`).
- **S82 (v0.8.1 runway — seal source-hash determinism LF/CRLF):** the seal
  predicate's `source_blake3` hashed raw source bytes, so an LF (Mac/Linux) and a
  CRLF (Windows `core.autocrlf`) checkout of the *same logical source* produced
  different predicates (WIN-S38-001). Two-layer fix: (1) `Manifest::build` now
  hashes **LF-normalized** source (`normalize_source_eol`, idempotent on LF — so
  existing LF seals are unchanged); (2) `.gitattributes` pins `*.garnet text eol=lf`
  as defense-in-depth. The canonicalization contract is documented in
  `C_Language_Specification/GARNET_ATTESTATION.md`. Adds Rust regressions (LF↔CRLF
  same `source_hash`+`ast_hash`; LF hash unchanged vs raw blake3) and
  `scripts/garnet_seal_determinism_status.py` (+ `--gate`, 5 tests, agent-contracts).
  **Honest scope:** only line endings are canonicalized (other whitespace still
  changes `source_blake3` by design); Mac-authored + Mac-unit-tested — the
  end-to-end Windows proof (fresh Windows checkout → matching `source_blake3`) is
  **Windows-proof-pending** in `WINDOWS_AUDIT_S1_S80.md`.
- **S81 (v0.8.1 runway — case-insensitive `.GARNET` discovery):** the shared target
  collector (`garnet-cli/src/cmd/verify_gate.rs`, `collect_targets`→`walk`) matched
  the extension case-sensitively, so Windows' case-insensitive filesystem silently
  skipped uppercase `.GARNET` files — a trust hole spanning `garnet verify`
  (WIN-S33-001), capability manifests (WIN-S36-001), `diff-caps` (WIN-S37-001), and
  sandbox-policy generation (WIN-S46-001). **One fix** — `eq_ignore_ascii_case` in
  the shared collector — closes all four, because `cap_manifest::surface_for_path`
  reuses the same collector. Adds two Rust unit tests (a `BAD.GARNET` directory
  fixture is discovered; an uppercase file target resolves) and
  `scripts/garnet_garnet_ext_discovery_status.py` (+ `--gate`, 5 tests,
  agent-contracts) as an anti-regression gate. **Honest scope:** Mac-authored +
  Mac-unit-tested (macOS preserves filename case, reproducing the skip); the
  end-to-end Windows proof (`garnet verify <dir with BAD.GARNET>` → exit 1) is
  recorded in `WINDOWS_AUDIT_S1_S80.md` as **Windows-proof-pending** for the Windows lane.
- **S84 (Exp 3 Windows WSL/bash path proof):** `scripts/garnet_paper_vi_exp3_status.py`
  now invokes the provider-free lane scripts by relative name from the harness
  cwd on Windows, so WSL `bash` no longer receives `C:\...` script paths it
  cannot resolve. `F_Project_Management/WINDOWS_AUDIT_S1_S80.md` records the
  Windows proof: `python -B scripts\test_garnet_paper_vi_exp3_status.py` runs
  6/6 OK and `python -B scripts\garnet_paper_vi_exp3_status.py --gate --format json`
  exits 0 with `provider_free_run_ok=true`. **Honest scope:** provider-backed
  H3A re-run remains pending-infra; no LLM provider is called.

- **P0 (v0.8.1 runway — Windows audit as tracked truth):** imports the Codex
  Windows audit of S1–S80 as committed evidence — `F_Project_Management/WINDOWS_AUDIT_S1_S80.md`
  (summary + the **14 open `WIN-*` findings** mapped to owning burn-down slices
  S81–S88 + the resolved `WIN-S70-001`), plus the machine ledgers
  `.dogfood/windows-core-audit.json` and `.dogfood/windows-audit-goal.json`
  committed verbatim. `scripts/garnet_windows_audit_status.py` (+ `--gate`, 5
  tests, agent-contracts) enforces that every open finding has an owning slice and
  the ledgers pin HEAD `cc165e8`. **Honest scope:** a tracking gate over imported
  audit evidence — it does not re-run the audit or claim any finding fixed; Windows
  proofs are recorded back into the doc by the Windows lane. No Rust changed.
- **S80 (v0.8.0 cut readiness — the whole S30–S80 run):**
  `scripts/garnet_v0_8_0_cut_readiness.py` (+ `--gate`, 5 tests, agent-contracts)
  aggregates the entire completion run into one **READY / NOT-READY-TO-CUT**
  verdict: the ledger has every slice **S31..S79 merged**, the S60
  release-readiness gate passes (band gates + 11 anti-rot sub-gates), and all
  **11 runway gates (S69..S79) pass** — current verdict **READY TO CUT (pending
  Jon)**. Doc `F_Project_Management/GARNET_v0_8_0_CUT.md`. **This does NOT cut,
  push, or authorize any tag** — cutting `v0.8.0` is a release-truth decision
  reserved for Jon (escalated); only `v0.4.2`/`v0.5.0` are tagged today.
  **Honest scope:** v0.8.0 is a **research-grade-prototype milestone**, not a
  production/1.0 claim; the deferred-for-v0.8.0 list stands. No Rust changed.
- **S79 (website/deck positioning reframe):** `F_Project_Management/GARNET_POSITIONING.md`
  is the canonical messaging — lead with the **integration + agent-code thesis**
  ("Garnet's bet is the integration, not the parts"), make **diff-caps** the
  headline, and **concede precedent honestly** (capability security / bounded
  execution / signed provenance are well-precedented). The landing page
  (`docs/index.html`) gains a matching reframed thesis section.
  `scripts/garnet_positioning_status.py` (+ `--gate`, 5 tests, agent-contracts) is
  a static anti-drift gate: both the doc and the landing page must carry the
  integration thesis + diff-caps headline + precedent concession.
  **Honest scope:** a positioning claim about novelty and fit, **not** a
  production-readiness or 1.0 claim. No Rust changed.
- **S78 (governance + RFC process):** `GOVERNANCE.md` formalizes how changes land
  through an **RFC + edition process** over ad-hoc BDFL fiat; `rfcs/` adds the
  process (`README.md`), a template (`0000-template.md`), and **RFC-0001** —
  *Draft* — proposing a cross-language **capability-manifest standard** and the
  intent to donate it to OWASP / the Linux Foundation. `scripts/garnet_governance_status.py`
  (+ `--gate`, 6 tests, agent-contracts) verifies the governance docs exist and
  that RFC-0001 marks the donation as **intent/draft, not an accepted standard**.
  **Honest scope:** **single-maintainer governance for a research-grade prototype**
  — no steering committee, no foundation, no adopted standard (the existing honest
  `docs/governance.html` framing is preserved); the donation is intent + a draft.
  No Rust changed.
- **S77 (external package pilot):** `garnet-registry-stub/tests/external_package_pilot.rs`
  drives the external-package flow end-to-end against the filesystem registry stub
  — publish → `resolve` by name+version → BLAKE3 content-address `verify_package`
  (tamper detection) → refuse a nonexistent dependency (`NotFound`) → `slopguard`
  flags a hallucinated near-miss (separator-confusable + edit-distance), answering
  the trajectory research's #1 supply-chain threat (slopsquatting). Runs in the
  `cargo test --workspace` matrix on every OS. `scripts/garnet_external_package_pilot_status.py`
  (+ `--gate`, 5 tests, agent-contracts) is the static gate; spec
  `C_Language_Specification/GARNET_EXTERNAL_PACKAGE_PILOT.md`. **Honest scope:** a
  **LOCAL filesystem registry-stub pilot, NOT a live public ecosystem** (no HTTP /
  publish / auth / SemVer / signatures); the slopguard is a heuristic, not a
  security guarantee. Rust touched: a new integration test only (no production code).
- **S76 (stdlib promotion wave):** the foundational **`core::*` layer** (30
  primitives — `core::iter/result/option/cmp/math`) is promoted from Experimental
  to **Stable**; the stdlib distribution moves 27/59 → **57 Stable / 29
  Experimental**. Promotion criteria: core layer (no host authority) + frozen
  semantics + test-covered. The **`std::*`** host-authority + evolving-API
  utilities (`env/process/json/regex/uuid/base64/log`) are **deliberately kept
  Experimental** — their warnings are correct. Effect: `novel_07` (core-only) now
  checks **0 diagnostics**; `novel_04–06` still warn (they use `std::*`).
  `scripts/garnet_stdlib_promotion_status.py` (+ `--gate`, 5 tests, agent-contracts)
  enforces the wave stayed **scoped** (all `core::*` Stable AND `std::*` still
  Experimental), so a future blanket flip can't pass silently. Spec
  `C_Language_Specification/GARNET_STDLIB_PROMOTION.md`. The Rust change is the
  `@stability` tier in the `garnet-stdlib` registry (the only Rust touched; two
  checker tests were repointed to a still-experimental `std::*` primitive).
  **Honest scope:** a stability judgement, not warning-suppression.
- **S75 (formal-verification feasibility study):**
  `C_Language_Specification/GARNET_FORMAL_VERIFICATION_FEASIBILITY.md` assesses
  whether Garnet can offer a *provable* termination / `@caps`-soundness story over
  a safe subset (the eBPF-verifier path). Verdict: a verified bounded-loop checker
  for the safe subset (eBPF-style, building on `explosive.rs`) is the feasible
  first provable increment; `@caps` soundness is feasible only atop the S74
  linear-capability mode; whole-language verification is **not feasible** (halting
  problem + ambient authority + FFI). `scripts/garnet_formal_verification_feasibility.py`
  (+ `--gate`, 5 unit tests, agent-contracts) is a static anti-overclaim gate
  grounding the study in real source. **Honest scope:** a feasibility **study
  only** — no verifier, no termination proof, no SMT/proof-assistant integration,
  no soundness theorem ships. No Rust changed.
- **S74 (safe-subset spec + linear/effect-typed mode graft):**
  `C_Language_Specification/GARNET_SAFE_SUBSET.md` specifies (1) the safe subset
  **as implemented today** — the typed, ownership-disciplined `fn` mode
  (`FnMode::Safe`) plus the fn↔def boundary audit (`audit.rs`) that closes the
  "hidden safe→managed escalation" threat class — and (2) a **proposed** optional
  linear/effect-typed rigor mode (Austral linear capabilities / Koka effects) for
  high-assurance components. `scripts/garnet_safe_subset_status.py` (+ `--gate`,
  5 unit tests, wired into agent-contracts) is a static anti-overclaim gate: it
  verifies the spec's "implemented today" claims are grounded in real source.
  **Honest scope:** the linear/effect mode is a **PROPOSAL — NOT IMPLEMENTED** (no
  linear type system, effect rows, or soundness proof ship here); this is a
  specification slice. No Rust changed.
- **S73 (VM / interpreter parity campaign):** `scripts/garnet_vm_interp_parity.py`
  (+ `--gate`, 7 unit tests) runs every `examples/*.garnet` program through **both**
  execution backends (`garnet run --interp` and `--vm`) and asserts they agree —
  **33/33 parity, 0 divergences** today. Parity is compared on the deterministic
  surface (**stdout + exit code**); stderr is ignored by design because the VM adds
  a cosmetic `vm error:` wrapper prefix and the episodic cache emits run-to-run
  stderr notes. CI: the canonical-examples job runs the binary-backed differential
  campaign; agent-contracts runs the static gate. Doc
  `F_Project_Management/GARNET_VM_INTERP_PARITY.md`. **Honest scope:** corpus-based
  parity over the shipped examples, **not** a proof of total backend equivalence;
  divergences are reported, not hidden; the stderr prefix is a documented cosmetic
  difference. No Rust changed.
- **S72 (self-hosted parser seed):** `examples/self_hosted_parser_seed.garnet` —
  a Garnet program that parses a subset of Garnet's **own** surface syntax
  (`def name(params) { ... }` declarations from an embedded source string),
  reporting each declaration's name, arity, and `@caps` managed status using only
  Stable, no-caps `str::` primitives (checks with 0 diagnostics; runs to
  `parsed defs: 3 managed: 1`). `scripts/garnet_self_hosted_parser_seed_status.py`
  (+ `--gate`, 5 unit tests) reports/gates it: the canonical-examples CI job runs
  the binary-backed check+run proof, the agent-contracts job runs the static
  well-formedness gate. Spec `C_Language_Specification/GARNET_SELF_HOSTED_PARSER.md`.
  **Honest scope:** a **SEED**, **not** the production parser (`garnet-parser-v0.3`)
  — no full AST/grammar (nested braces, expressions, types, comments); it neither
  replaces nor bootstraps the Rust parser. Full self-hosting remains roadmap.
- **S71 (Paper VI Exp 3 — actual run, honest):** `scripts/garnet_paper_vi_exp3_status.py`
  (+ `--gate`, wired into CI's agent-contracts job) inventories the
  compiler-as-agent time-to-fix harness (`benchmarks/paper_vi_exp3_compiler_as_agent/`:
  10 evolving snapshots × stateless/history-aware lanes) and **actually runs its
  provider-free mode** (both lanes harness-only + `aggregate.py`, which emit the
  honest "harness-only / no results invented" shape). It records the pre-registered
  H₃ and the v4.0 outcome verbatim — **h₃a partial (6.5% speedup, CI [3.1%, 9.8%],
  below the 10% threshold); h₃b and h₃c pass** — citing `GARNET_v4_0_PAPER_VI_EXECUTION.md`.
  Doc `F_Project_Management/GARNET_PAPER_VI_EXP3.md`; 6 unit tests. **Honest scope:**
  h₃a's timing speedup is machine-dependent and is **not re-measured** here, **no LLM
  is called**, and the provider-backed re-run is **pending-infra** (same boundary as
  Exp 1); the recorded 6.5% partial stands and the 10% claim is downgraded honestly.
- **S70 (version-map source-of-truth correction):** new authoritative
  `F_Project_Management/GARNET_v0_8_VERSION_MAP.md` records the corrected v0.8 tag
  mapping — **the whole S30–S80 run is cut as one `v0.8.0` tag at the end of S80;
  S60 and S70 are readiness checkpoints, not cuts; S81+ is the runway to v0.8.1;
  1.0 is held much further out (~a year), gated on validation.** The operative
  contract's band table + forward bands are corrected to match; historical
  artifacts (the reconciled slice plan, the S60 release doc, the beta gate) carry
  dated correction banners. `scripts/garnet_version_map_check.py` (+ 5 unit tests,
  wired into CI's agent-contracts job) locks the source of truth and fails if a
  doc drifts back to the superseded mapping. **Honest scope:** docs + a
  docs-consistency gate only — **no tag is cut, pushed, or authorized** (tagging
  stays a human release-truth decision); no Rust changed.
- **S69 (LLM-suggest v0.2 / Paper VI Exp 1 prep):** `scripts/garnet_llm_suggest_readiness.py`
  inventories the compiler-as-agent advisory's two tiers — the **rules tier** (S10,
  ACTIVE: `managed-fn-missing-caps`, `long-parameter-list`, `empty-function-body`,
  verified present in `suggest.rs`) and the **LLM tier** (pending-infra) — records
  the Paper VI Experiment 1 prep protocol, and quotes the scorecard verbatim.
  `--gate` (CI) gates the rules tier's presence; the LLM tier is **not** gated.
  Spec `C_Language_Specification/GARNET_LLM_SUGGEST.md`; 5 unit tests. **Honest
  scope:** the LLM tier is **pending-infra** — no model is wired or called and **no
  new firing advisory is added**; the rules tier is the active baseline. (Paper VI
  scorecard quoted verbatim, not softened.)
- **S68 (capability transparency log):** `garnet caps-log <file> --log <path>`
  appends an append-only, **BLAKE3-chained** capability entry (`index`, `program`,
  `caps`, `caps_blake3`, `prev_blake3`); `garnet caps-log --verify <path>`
  recomputes the chain — tampering with any earlier entry breaks the chain at the
  next (exit 1). The language-agnostic entry schema **seeds a cross-language
  capability-manifest standard** (the GRAFT). `C_Language_Specification/GARNET_CAPABILITY_TRANSPARENCY.md`
  + cross-OS proof `garnet-cli/tests/caps_log.rs` (2 tests). **Honest scope:** a
  **local, hash-chained STUB** — **not** a distributed/witnessed transparency log
  (no public log server, no signed tree head, no witness/gossip, no external
  inclusion proof); it gives tamper-evidence for a *local* file, not Rekor.
- **S67 (MCP tool-capability declarations):** brings the `@caps` lens to MCP/agent
  tools, addressing the documented MCP "absence of capability attestation". A
  `.mcpcaps` manifest names each tool's required caps (`tool: cap1, cap2`); new
  `garnet mcp-caps <file>` reports the per-tool + **aggregate** authority surface,
  flags **high-authority** tools (`ffi`/`proc`/`*`), and lists unknown caps.
  `--format json` (`schema garnet.mcp_caps/v1`, `"enforced":false`) makes a
  tool-set's authority diffable (a tool-set gaining `proc`/`ffi` is as visible as
  a program gaining a capability). `examples/mcp/agent_toolset.mcpcaps` + spec
  `C_Language_Specification/GARNET_MCP_CAPS.md` + cross-OS proof
  `garnet-cli/tests/mcp_caps.rs` (2 tests). **Honest scope:** **self-declared,
  not MCP-host enforced** — Garnet is not an MCP host and does not intercept tool
  calls; enforcing/verifying the declaration at the boundary is out of scope.
- **S66 (model/prompt/tool attestation):** extends the S65 seal authorship with a
  structured **attestation block**. `garnet seal <file> --attest <key>=<value>`
  (repeatable) records a deterministic (sorted) `"attestation"` object —
  conventionally `model`, `prompt_sha256`, `tool` (e.g. `mcp:filesystem`) — in the
  in-toto predicate, composing with `--authored-by`. Together: *who*
  (`authorship`), *what pipeline* (`attestation`), *what authority*
  (`capability_manifest`). Spec `C_Language_Specification/GARNET_ATTESTATION.md`
  + cross-OS proof `garnet-cli/tests/seal_attestation_block.rs` (4 tests); S65/S38
  seal tests unchanged. **Honest scope:** every field is **self-declared, not
  verified** (the `@caps` posture) — Garnet does not introspect the model, hash
  the live prompt, or enumerate the tools actually invoked; absent `--attest`
  records no block; auditing accuracy is out of scope.
- **S65 (AI-authorship provenance):** makes "who/what wrote this?" a first-class,
  attestable declaration. `garnet seal <file> --authored-by <provenance>` records
  an `"authorship"` field in the in-toto predicate (e.g. `ai:claude-opus-4-8`,
  `ai-assisted:…`, `human:jon`) — diffable, reviewable, and signable alongside the
  capability surface. Omitting it records **no** authorship claim (default
  predicate shape unchanged). Spec
  `C_Language_Specification/GARNET_AI_PROVENANCE.md` + cross-OS proof
  `garnet-cli/tests/ai_provenance.rs` (2 tests); existing `seal` tests unchanged.
  **Honest scope:** a **self-declared** fact, **not AI-detection** — Garnet records
  what the toolchain declares (the `@caps` posture: declared, not inferred);
  silence is honest, not an implicit "human"; verifying the declaration's accuracy
  is out of scope.
- **S64 (WASI interop):** closes the native-interop band by making the
  `@caps` → WASI capability mapping explicit — `fs`→preopens, `net`→wasi-sockets,
  `time`→clocks, `env`→environ (the S46 sandbox `wasi` policy *is* the mapping).
  `examples/ffi/wasi_clock.garnet` (`@caps(time, fs)`) checks clean and its
  sandbox WASI policy reflects the caps (`clocks:true`, `preopens:true`,
  `sockets:false`). Spec `C_Language_Specification/GARNET_WASI_INTEROP.md` +
  cross-OS proof `garnet-cli/tests/wasi_interop.rs` (2 tests). **Honest scope:**
  the WASI **authority mapping**, not a WASI **runtime** — Garnet does not compile
  to wasm or run under a WASI host (wasm32/wasm-pack/wasmtime absent, S55); the
  build + host execution are **deferred**.
- **S63 (C ABI proof):** establishes the C ABI as the canonical FFI contract and
  proves **compound native authority**. `examples/ffi/c_stat.garnet` binds a C
  `stat`-like symbol that touches the filesystem — needing **both** `@caps(ffi)`
  (native) and `@caps(fs)`. The model surfaces both: `garnet check` clean,
  `garnet sandbox` raises the ffi escape-hatch warning **and** enables fs WASI
  preopens, `garnet seal` attests `["ffi","fs"]`. Spec
  `C_Language_Specification/GARNET_C_ABI.md` (incl. the value↔C-type marshalling
  table) + cross-OS proof `garnet-cli/tests/c_abi_proof.rs` (3 tests). **Honest
  scope:** ships the C ABI *contract* + the compound-authority proof; **no FFI
  runtime** — the marshalling layer and a linked `.so`/`.dylib` are deferred.
- **S62 (Rust FFI proof):** proves a Garnet↔Rust `extern "C"` binding is a
  first-class, **attested** authority. `examples/ffi/rust_extern.garnet` (a
  `@caps(ffi)` Rust-wrapper) checks clean, runs, and `garnet seal` emits an
  in-toto predicate whose embedded capability manifest **attests `ffi`** — so a
  Rust-FFI binding is diffable (S37), reviewable (S49), and signable (S51) like
  any authority. Spec `C_Language_Specification/GARNET_RUST_FFI.md` + cross-OS
  proof `garnet-cli/tests/rust_ffi_proof.rs` (2 tests). **Honest scope:** proves
  the **authority + attestation** half; Garnet has **no FFI runtime** — the
  value↔C-ABI marshalling layer and a linked Rust `cdylib` are **deferred**, not
  added here.
- **S61 (FFI authority model):** codifies how foreign-function calls are
  governed — FFI is an **explicit, declared, diff-gated, sealed** authority, not
  an implicit escape hatch. A function wrapping a native call must declare
  `@caps(ffi)`, which then flows through the whole trust kernel: capability
  surface (S35), manifest (S36), `diff-caps` `GAINED ffi` (S37), seal (S38), and
  the `garnet sandbox` escape-hatch warning (S46). Ships
  `C_Language_Specification/GARNET_FFI_AUTHORITY.md`, the
  `examples/ffi/{no_native,native_boundary}.garnet` pair, and a cross-OS proof
  (`garnet-cli/tests/ffi_authority.rs`, 3 tests). **Honest scope:** Garnet has
  **no FFI runtime** — the interpreter does not execute `extern "C"` calls and
  this slice adds none; it ships the *authority model* (declaration → surface →
  diff → seal → sandbox-flag), not native-call execution. The model's value is
  transparency + review, not containment (the sandbox cannot constrain FFI).
- **S60 (v0.8.0 release readiness):** `scripts/garnet_v0_8_0_release_readiness.py`
  aggregates the whole v0.8 train into one verdict — both bands merged
  (hardening S41–S50 10/10, adoption S51–S59 9/9) and all 11 anti-rot sub-gates
  passing — rendering **READY TO TAG (pending Jon)** with the honest in/deferred
  inventory + verbatim honesty anchors. `--gate` (CI) fails unless the bands are
  merged and every sub-gate passes. 5 unit tests; `F_Project_Management/GARNET_v0_8_0_RELEASE.md`.
  **CRITICAL honest scope:** this **does not cut a tag**. Only `v0.4.2`/`v0.5.0`
  are tagged; cutting `v0.8.0` is a **release-truth decision for Jon**, escalated,
  not made autonomously. "READY TO TAG" is evidence-backed advice, not the act.
- **S59 (fuzz campaign):** grows the parser fuzz corpus with 5 newer-construct
  seeds (typed rescue, `@caps`, enum + exhaustive `match`, doctest fences,
  cross-boundary `Result`) — better coverage of the S42–S57 grammar — and adds
  `scripts/garnet_fuzz_campaign.py`, which inventories the campaign (target,
  crate, nightly run protocol, seed count) and `--gate`s that the harness stays
  wired (target file + Cargo `[[bin]]` + `fuzz-nightly.yml` reference + non-empty
  seed corpus). 5 unit tests. **Honest scope:** verifies the harness **exists and
  is wired**; it does **not** run the fuzzer and makes **no** bug-found (or
  bug-free) claim — crashes surface in the nightly `cargo fuzz run` job, and
  `cargo-fuzz` is absent in this environment.
- **S58 (benchmark campaign):** `scripts/garnet_benchmark_campaign.py` inventories
  the full Criterion benchmark campaign — all 6 harnesses (parser `parse`, CST
  `parse_cst_vs_ast`, interp `eval`, VM `parse_compile_execute`, memory `vector`
  + `eviction`) with what each measures and the per-bench run command — and
  `--gate` (CI) fails if a declared bench file or its Cargo `[[bench]]` entry
  disappears. Complements `garnet_benchmark_no_run.py` (compile evidence). 5 unit
  tests. **Honest scope:** inventories + verifies the harnesses **exist**; it does
  **not** run them and reports **no measurements** (Criterion numbers are
  environment-specific, recorded by an explicit campaign run, not fabricated here).
- **S57 (idiomatic open corpus):** a small, open corpus showing *what good Garnet
  looks like* — `examples/idiomatic/typed_errors.garnet` (typed `rescue e: T`, the
  S42 policy, never a catch-all) and `examples/idiomatic/state_machine.garnet`
  (exhaustive `match` over a finite enum, named `@caps`). The bar is high: each
  `garnet check`s to **0 diagnostics** (fully clean, not even a non-fatal
  advisory) and runs deterministically, proven by
  `scripts/garnet_idiomatic_corpus.py` (+ `examples/idiomatic/README.md`). 5
  tests. **Honest scope:** a style/discipline corpus, not a performance or
  coverage claim.
- **S56 (playground MVP):** upgrades `docs/playground.html` from an honest
  "planned" placeholder into a **static example gallery** — pick a real Garnet
  program, see its source and its recorded `garnet run` output (from
  `docs/playground/examples.json`, generated by `scripts/garnet_playground_build.py`).
  `scripts/garnet_playground_readiness.py` validates the gallery **and** that the
  page keeps its honesty markers (a `--gate` fails if it silently becomes a
  fake-editor claim). 4 unit tests. **Honest scope:** this is a **static gallery,
  not a live editor** — outputs are recorded, not computed in the browser; live
  in-browser execution needs the WASM build (S55, deferred). It deliberately
  does **not** ship a fake editor.
- **S55 (WASM hello-world):** ships the canonical `examples/hello.garnet` and a
  WASM-readiness reporter for the in-browser path. `scripts/garnet_wasm_readiness.py`
  inventories the path (the interpreter compiled to wasm is the in-browser
  execution model — Garnet has no wasm backend) and **names the concrete
  blockers**: no `wasm32` target, `wasm-pack`/`wasmtime` absent, and
  `garnet-interp` pulling `miette`'s `fancy` (terminal/backtrace) feature. Doc
  `F_Project_Management/GARNET_WASM_TARGET.md` records the build path. `--gate`
  guards only the owned bits (the example + the doc); 5 unit tests. **Honest
  scope:** **no wasm is built and no browser run is claimed** — the wasm build is
  deferred until the named blockers are resolved; the absent toolchain is an
  honest deferral, not a gated failure.
- **S54 (VS Code / OpenVSX / Marketplace path):** makes the VS Code extension
  marketplace-ready and documents the publish path. Added `keywords` to
  `editors/vscode/package.json` (discoverability); new
  `scripts/garnet_vscode_publish_readiness.py` asserts every marketplace-required
  field (`name`/`version`/`publisher`/`engines.vscode`/`repository`/`license`) +
  recommended field + the README/LICENSE files are present, and `--gate` (CI)
  fails on a regression. It reports the path: build VSIX ✅ → GitHub release asset
  on tag ✅ → OpenVSX (`ovsx publish`) / Marketplace (`vsce publish`) **deferred**.
  5 unit tests. **Honest scope:** the actual OpenVSX/Marketplace publish needs
  `OVSX_TOKEN`/`VSCE_PAT` credentials — credential/account territory, **deferred
  to a human**; this slice makes the extension publish-*ready*, it does not
  publish and bundles no credentials.
- **S53 (tree-sitter grammar):** adoption infrastructure for editors. New
  `tree-sitter-garnet/grammar.js` defines the core Garnet syntax (functions +
  `@`-annotations, struct/enum/impl, actors + `memory` kinds, control flow,
  `match`, `try/rescue/ensure`, expressions incl. the `|>` pipe, `#`/`///`
  comments) — the *syntax* grammar, distinct from the LSP *semantic* service
  (S44). `scripts/garnet_tree_sitter_check.py` structurally validates it (loads
  `grammar.js` with Node, asserts the grammar name + core rules) and `--gate`
  (CI) fails on a dropped rule. 5 unit tests; `tree-sitter-garnet/README.md`.
  **Honest scope:** a CORE grammar (not exhaustive), **structurally validated,
  not compiled** — `tree-sitter generate` + corpus tests need the tree-sitter
  CLI (absent here); the hand-written `garnet-parser` remains the source of truth.
- **S52 (one-line install / readme check):** a consistency gate for the curl|sh
  install path. `scripts/garnet_install_readme_check.py` extracts the one-line
  `curl … install.sh | sh` command from `README.md` and from
  `installer/sh.garnet-lang.org/install.sh`'s own header, and fails (`--gate`, in
  CI) if they drift or the canonical install URL is missing from either —
  closing the #1 adoption footgun (a README that documents an install command the
  installer no longer matches). 6 unit tests. **Honest scope:** a doc-consistency
  check, not a live network install test; `install.sh` remains separately
  shellcheck-gated (this does not duplicate that).
- **S51 (signed release lanes):** makes Garnet's three signing lanes explicit and
  gates the active one. **`garnet seal --out <path>`** now writes the in-toto
  predicate to a file (was print-only) so it feeds straight into `cosign attest
  --predicate <path>`; the cosign hint names the written path. New
  `scripts/garnet_signed_release_lanes.py` inventories the lanes —
  (1) program-manifest signing (`garnet build --sign`, **active**, CI-gated),
  (2) release-artifact `SHA256SUMS` signature (**deferred**, GPG/minisign),
  (3) supply-chain attestation (`garnet seal` → cosign, **partial**) — and its
  `--gate` (CI) protects lane 1 from silent regression. 3 Rust seal tests
  (incl. `--out`) + 6 reporter unit tests; doc
  `F_Project_Management/GARNET_SIGNED_RELEASE_LANES.md`. **Honest scope:** Garnet
  does not sign its own supply chain or bundle cosign/GPG/minisign; lanes 2–3 are
  deferred/partial by design and reported truthfully.
- **S50 (v0.8 beta gate):** closes the S41–S50 hardening band.
  `scripts/garnet_v0_8_beta_gate.py` is a band-completion **checkpoint** (not a
  release): it verifies the nine hardening slices S41–S49 are merged at
  confidence 5 in the goal ledger and that the band's anti-rot sub-gates
  (build-proof S47, proof-matrix S48) still pass, then reports what the band
  shipped and what is explicitly deferred for beta. `--gate` (wired into CI)
  fails unless the gate is OPEN; `--format md|json`; 8 unit tests; doc
  `F_Project_Management/GARNET_v0_8_BETA_GATE.md`. **Honest scope:** it does
  **not** cut a tag and does **not** claim production readiness — Garnet remains a
  *research-grade prototype (v0.x.x), not production-complete*; cutting
  `v0.8.0-beta` is a release-truth decision for Jon. The verbatim honesty anchors
  (incl. the Paper VI scorecard) are surfaced, not changed.
- **S49 (AI-PR-review-collapse wedge demo):** the launch narrative, runnable.
  `examples/wedge_pr_review/{before,after}.garnet` simulate an AI-suggested PR
  that silently widens authority `@caps(fs)` → `@caps(fs, net)` (an exfiltration
  path). Both versions `garnet check` **clean** — the escalation is invisible to
  the checker — yet `garnet diff-caps` (S37) flags `caps GAINED: net` /
  `AUTHORITY EXPANDED` and exits non-zero, and `garnet sandbox` (S46) shows egress
  flip `deny-all` → `allow`. `garnet-cli/tests/pr_review_wedge.rs` (3 tests) is
  the cross-OS CI-gated proof; `scripts/smoke_garnet_pr_review_wedge.py`
  (`--format md|json`) generates the narrative report (4 tests). Doc:
  `F_Project_Management/GARNET_PR_REVIEW_WEDGE.md`. **Honest scope:** the "human
  review collapses under AI volume" claim is the motivating thesis, **not** a
  measurement made here; this is a narrative composition of existing gates, not a
  new enforcement mechanism or a guarantee against all AI-PR risks.
- **S48 (12-domain / 7-novel proof matrix):** `scripts/garnet_proof_matrix.py`
  inventories the 12 application domains Garnet is demonstrated across (reusing
  the `CORE_12_CASES` single source of truth) and the 7 novel Paper VI
  contributions (by title), anchoring each contribution to in-repo evidence whose
  existence is checked. `--gate` (wired into CI) fails if a domain example or a
  contribution anchor disappears; `--format md|json`; 7 unit tests; doc
  `F_Project_Management/GARNET_PROOF_MATRIX.md`. **Honest scope:** an evidence
  *inventory*, not empirical proof — no measurement, mechanized-proof, or
  external-study claim. It does **not** re-adjudicate per-contribution verdicts;
  Paper VI's aggregate scorecard ("4 supported, 2 partial (downgraded honestly),
  0 refuted, 1 pending-infra") is quoted **verbatim** as an honesty anchor.
- **S47 (Windows/Linux/macOS build proof):** `scripts/garnet_build_proof.py`
  reports and **gates** cross-OS coverage from the CI matrix, distinguishing
  *behaves* (the OS is in the `cargo test --workspace` matrix in `ci.yml`) from
  *distributes* (deb/rpm + macos-cli-tarballs packaging). `--gate` runs in CI and
  fails if any of the three target OSes is dropped from the test matrix —
  silent cross-OS regression can't slip through. `--format md|json`; 7 unit
  tests. `F_Project_Management/GARNET_BUILD_PROOF.md` documents the status table
  plus the **Windows-propriety audit** (per-surface cross-platform status: pure
  logic CI-gated; determinism CI-gated; the S46 seccomp policy is a documented
  Linux-shaped gap; Windows CLI distribution deferred). **Honest scope:**
  CI-attested, **not** locally re-run (single-OS checkout) — the gate verifies the
  matrix covers all three OSes; it does not itself execute Windows/Linux.
- **S46 (caps-to-sandbox policy):** `garnet sandbox <file>` translates a module's
  declared `@caps` surface (S35) into three concrete sandbox policy artifacts — a
  seccomp profile (OCI-style default-deny syscall allowlist), a WASI capability
  set (preopens / sockets / clocks / env), and an egress rule
  (deny-all / loopback-only / allow). Pure mapper `garnet_cli::sandbox` +
  deterministic JSON (`schema garnet.sandbox/v1`); `--format human|json`. `ffi`
  and `proc` emit explicit escape-hatch warnings; `*` is permissive-with-warning.
  8 unit + 4 integration tests; mapping documented in
  `C_Language_Specification/GARNET_SANDBOX_POLICY.md`. **Honest scope: policy
  generation, not enforcement** — every artifact is marked `"enforced": false`;
  nothing runs under `wasmtime`, applies seccomp to a live process, or installs a
  firewall (runtime enforcement needs wasmtime/a Linux seccomp host, both out of
  scope and absent here).
- **S45 (slopsquatting guard):** `garnet add --registry` now flags an unknown
  package name that closely resembles a known one before anything is trusted —
  the live slopsquatting threat (hallucinated names attackers pre-register).
  New pure module `garnet_registry_stub::slopguard`: Damerau–Levenshtein (OSA)
  distance + separator-confusable (`foo-bar` vs `foo_bar`) detection, returning
  deterministically ordered near-misses; `RegistryIndex::known_names()` feeds it.
  When `resolve` reports an unknown **name** (not a missing version), the error
  is enriched — *"`reqests` is not in this registry — did you mean `requests`?
  … a slopsquatting risk; verify the source before adding."* 6 guard unit tests
  + 2 CLI integration tests (near-miss warns; version-miss stays quiet).
  **Honest scope:** the registry is a filesystem stub, so "known names" are the
  local index, not a global ecosystem feed; the guard is a prompt-to-verify
  heuristic, not a security guarantee. Resolver behavior/exit codes unchanged.
- **S44 (LSP safe-mode precision):** the checker now owns a single source of
  truth for diagnostic presentation — `garnet_check::Severity` plus
  `CheckError::severity()` / `CheckError::code()`. `garnet check` (S34
  structured diagnostics) and the LSP both consume it, so they cannot drift.
  The LSP previously mapped every non-`BoundaryNote` finding to a red `ERROR`;
  now safe-mode/capability violations are `ERROR`, boundary notes are `WARNING`,
  and advisories (over-catch S42, stability-advice) surface as `INFORMATION`
  with their canonical `check.*` code on `Diagnostic.code`. New tests: a
  garnet-check parity/invariant pair (severity↔code, and Error-severity ⇔ fatal)
  + two LSP tests (over-catch → INFORMATION, safe-mode → ERROR, both code-tagged).
  **Honest scope:** this is the *safe-mode precision* half of the slice;
  *cross-package* precision needs a module/package resolver (S45) and is
  deferred to ride with it — no cross-file resolution is claimed here.
- **S43 (docs-as-tests):** `garnet doctest <file>` makes documented examples
  executable — the "evidence not courtesy" discipline. It reuses `garnet doc`'s
  `///` extraction, lifts ` ```garnet ` fences (`garnet_cli::doctest::garnet_fences`),
  loads the file's own definitions, and runs each fence on the interpreter so an
  example can call the very function it documents. A fence passes if it evaluates
  without error; a `# => value` marker additionally asserts the displayed tail
  value. Human and `--format json` output; exit 1 iff any example fails. Ships a
  dogfooded demonstrator (`examples/documented_math.garnet`, 3 passing examples).
  6 unit + 3 runner-unit + 5 integration tests. **Honest scope:** runs on the
  interpreter (not the VM backend); fences see only the file's own definitions
  plus the stdlib (no cross-file imports, matching `garnet doc`); a doc-rot guard,
  not a replacement for the test suite.
- **S42 (typed Result / error policy):** codifies the typed-`Result`-first error
  policy and enforces a piece of it. `core::result` combinators (S26) and
  `try`/`rescue`/`ensure`/`raise` already exist; this slice adds
  `C_Language_Specification/GARNET_ERROR_POLICY.md` (two error channels; the
  over-catch anti-pattern — Ronacher's "agents over-catch exceptions") and the
  **over-catch advisory**: `garnet_check::overcatch_sites` flags catch-all
  `rescue` clauses (no exception type) and `garnet check` emits a **non-fatal**
  `check.over_catch` advisory (human + JSON) that never changes the exit code
  (excluded from `CheckReport::ok`). A typed rescue (`rescue e: T`) is not
  flagged. 3 unit + 3 integration tests. **Honest scope:** advisory only — no
  exit-code change, no auto-rewrite, no ban; no typed-exception hierarchy or
  checked-exceptions are introduced.

- **S41 (async/concurrency contract — first v0.8 hardening slice):** codifies
  Garnet's concurrency model as a canonical contract. The model is **actors**
  (not async/await — `async` is reserved for a future edition, S32), already
  built in `garnet-actor-runtime` (OS-thread + **bounded** mpsc mailbox closing
  the unbounded-mailbox DoS class; `@mailbox` override; Result-returning `ask`;
  hot reload). Adds `C_Language_Specification/GARNET_CONCURRENCY_CONTRACT.md` (the
  canonical contract, with an explicit deferred-scope section),
  `garnet_check::concurrency_surface` (per-actor protocols classified ask vs tell
  + handler count), and `garnet concurrency <file>`. 2 unit + 2 integration
  tests. **Honest scope:** documents what is BUILT — no new semantics; no
  async/await; `@nonsendable` cross-boundary enforcement and `@bounded` fuel
  enforcement are deferred (declared/reported, not enforced — no faking);
  structured concurrency / cancellation beyond actor lifecycle + the Result-`ask`
  is future work.

- **S40 (explosive-op / default-ceiling analysis — closes the v0.8 foundation
  band):** adds `garnet_check::explosive_ops` — a **compiler-exhaustive** AST
  visitor (every `Stmt`/`Expr` variant matched + recursed, so nested sites are
  never missed) that flags two unbounded constructs: an unconditional `loop`
  (static termination is undecidable, so every `loop` is flagged) and `spawn`
  (fan-out). Per function it reports the ops + whether each is governed by a
  declared bound (`@bounded` for loops, `@fan_out` for spawn) or the
  **default-ceiling policy** (`DEFAULT_LOOP_CEILING`, `DEFAULT_SPAWN_FANOUT`
  constants). `garnet ceilings <file>` reports it. 5 unit + 2 integration tests.
  **Closes Phase A (S31–S40).** **Honest scope:** static IDENTIFICATION +
  default-ceiling POLICY only — runtime enforcement lowers to the S39 `@bounded`
  / Wasmtime-fuel path and is deferred (wasmtime absent); no ceiling is faked.
  Explosive set = `loop` + `spawn` (recursion is already addressed by `@max_depth`
  + the caps call graph; unbounded collection growth is a follow-up).

- **S39 (@bounded — declared resource budgets; wrap-don't-rebuild):** adds the
  `@bounded(N)` annotation — a CPU/fuel budget of N Wasmtime-fuel units —
  threaded through all five `Annotation` sites (parser AST + grammar, the rowan
  CST converter, the checker's validation, the doc-span match).
  `garnet_check::bounded_functions(module)` extracts declared budgets (sorted by
  name) and `garnet bounds <file>` reports them. The checker rejects `@bounded(0)`
  (positive budget required); a negative literal is rejected at parse (a
  leading-minus token, consistent with `@mailbox` / `@max_depth`). 6 unit + 2
  integration tests. **Honest scope (wrap-don't-rebuild):** `@bounded(N)` declares
  and reports the budget; **enforcement lowers to Wasmtime fuel metering** (the
  lowering target). `wasmtime` / `wasm-tools` are absent in this environment, so
  budgets are declared, **not yet runtime fuel-enforced** (no fuel meter is
  faked). Mem bounds and a unified resource-bound syntax are follow-ups.

- **S38 (seal — in-toto build attestation; wrap-don't-rebuild):** adds
  `garnet seal <file>` — a deterministic **in-toto Statement (v1)** whose subject
  is the program's BLAKE3 AST digest and whose predicate embeds the deterministic
  build manifest (`manifest.rs`) and the S36 capability manifest (Garnet's native
  SBOM-equivalent extension). `garnet_cli::seal` builds the predicate; `cosign`
  signs it (detected, never required). Adds the `seal_attestation` readiness lane
  (committed-truth; headline 88.8% → 89.1%). 3 unit + 2 integration tests; output
  validated as JSON. **Honest scope (contract anchor):** "seal wraps
  in-toto/Sigstore/cosign — Garnet does not implement its own signing." `cosign`
  is **absent in this environment**, so the predicate is emitted **unsigned** and
  the wrapper prints the `cosign attest` command (it does not auto-sign);
  syft/cyclonedx are absent so the capability manifest is the SBOM-equivalent;
  per-file seal (per-package is a follow-up).

- **S37 (diff-caps — capability-surface diff gate; the headline novelty):** adds
  `garnet_check::diff_caps` (→ `CapsDiff`: aggregate added/removed, functions
  added/removed/expanded, wildcard-introduced) and `authority_expanded()` (a NEW
  aggregate capability or an introduced `@caps(*)`; a function re-declaring a cap
  already in the aggregate is NOT new program authority). `garnet diff-caps <old>
  <new>` exits non-zero iff authority expanded. **Completes the S33 graft:**
  `garnet verify --caps-baseline <old>` wires the diff into the fused band via
  `capability_band` (5 if no expansion, 2 if expanded; `min` governs), replacing
  the previously-stubbed capability signal. Adds the `capability_diff_caps`
  readiness lane (committed-truth; headline 88.6% → 88.8%) and consolidates a
  shared `surface_for_path` across `caps` / `diff-caps` / `verify`. 6 unit + 4
  integration tests. **Honest scope:** diff-caps reads the **declared** surface —
  it does not prove the absence of undeclared authority (the sandbox job, S46);
  `verify --caps-baseline` flags expansion via a low band (review signal),
  `diff-caps` is the hard gate; "two revisions" = two source paths the caller
  supplies (S37 does not itself drive git).

- **S36 (capability manifest — derived from annotations):** adds
  `garnet_cli::cap_manifest::CapabilityManifest` (schema
  `garnet-capability-manifest-v1` wrapping the S35 `CapabilitySurface`) and the
  `garnet caps <path>` command. Emits **deterministic JSON** (`{schema,
  aggregate, functions:[{name,caps}], wildcard}`, reusing
  `diagnostics::json_escape`) for a single file (per-program) or every `.garnet`
  under a directory (per-package, via `merge_surfaces` — union aggregate,
  sorted+deduped `(name,caps)`, OR-ed wildcard). Distinct from the build
  `Manifest` (which carries source/AST hashes but no caps). This is the artifact
  S37 `diff-caps` compares across revisions and S38 `seal` embeds. 4 unit + 3
  integration tests (binary via `CARGO_BIN_EXE`). **Honest scope:** captures the
  **declared** surface (S35) — it does not prove the absence of undeclared
  authority (that is the sandbox job, S46) and does not enforce the project
  `[caps]` budget; per-package same-name functions across files both appear
  (honest surface, not a resolver). No new readiness lane (mandated at S37/S38).

- **S35 (source annotations — the canonical capability surface):** the
  `@caps(...)` annotation *syntax* already existed (v3.4 CapCaps —
  `Annotation::Caps`, the `Capability` enum, parsing, transitive propagation), so
  per the contract's actual words this slice adds **the surface the manifest is
  derived from**: a first-class, deterministic `garnet_check::CapabilitySurface`
  (`aggregate`, `per_function`, `has_wildcard`) via `capability_surface(module)`,
  normalized through the canonical `Capability::as_str()` (sorted + deduplicated).
  Consolidates and **fixes** `garnet trust-report`, which built its caps surface
  via `format!("{c:?}").to_lowercase()` — mislabeling `net_internal` / `Other(_)`
  / wildcard caps — so the checker and trust-report now share one normalization.
  7 unit tests; the existing trust-report tests stay green. **Honest scope:** S35
  adds the surface artifact + a consolidation/bug-fix, **not** new syntax; it
  covers top-level functions' declared caps (the diff-caps surface) — actor
  methods, transitive/effective caps, and `[caps]`-budget enforcement are out of
  scope; S36 builds the per-program/package manifest artifact from this surface.

- **S34 (structured diagnostics — machine + human):** adds the reusable
  `garnet_cli::diagnostics` module (`Severity`, `Diagnostic { severity, code,
  message, span }`) and `garnet check [--format human|json]`. `human` (default)
  is unchanged miette/Display output; `json` emits **deterministic, hand-rolled
  JSON** (no `serde`, per the `manifest.rs` determinism stance): a `diagnostics`
  array with stable per-variant `code`s (`check.caps_coverage`,
  `parse.reserved_word`, …), lowercase `severity`, an escaped `message`, and an
  optional `{start,len}` span, plus a `summary`. Documents the **authoritative
  exit code** (0 clean / 1 fatal-diagnostic-or-parse-or-IO / 2 usage). 5 unit + 4
  integration tests (binary run via `CARGO_BIN_EXE`). **Honest scope:** the
  machine form is wired into `garnet check` only — `parse`/`verify` JSON and the
  MCP transport are follow-ups (S34 ships the structured type + its first
  consumer); check diagnostics have no spans yet (the `CheckError` variants are
  message-only), parse diagnostics do.

- **S33 (one-command `garnet verify` — acceptance gate):** adds
  `garnet verify <path>` — a single acceptance gate, distinct from the existing
  2-arg `garnet verify <file> <manifest.json>` deterministic-manifest verify (the
  dispatcher routes on positional-arg count). It runs **edition-aware parse +
  safe-mode check** over a `.garnet` file or every `.garnet` under a directory and
  emits a **fused merge-confidence band**: the internal local band (5 clean / 4
  advisory / 1 fatal) fused by `min` with an optional external-reviewer band
  (`--external-band`, Greptile at PR time) and a **pluggable capability signal**.
  Exits 0 on a clean tree, non-zero on a planted regression. Adds the
  `garnet_verify_gate` readiness lane (committed-truth; headline 88.3% → 88.6%);
  baseline regenerated. **Honest scope:** the capability-signal slot is a **stub
  until S37 `diff-caps`** (it never lowers the fuse while pending); the gate's
  internal band is the LOCAL acceptance signal that feeds the `dogfood-readiness`
  skill's PR-level falsification-ledger + Greptile fusion — `garnet verify` does
  not itself run cargo/CI; `garnet test` execution is not folded into the gate in
  S33.

- **S32 (edition / compatibility model — two-layer mechanism):** installs the
  compatibility-evolution mechanism before users exist to break.
  **Layer 1 (editions, parse-time):** a `garnet_parser::Edition` registry
  (`v1.0` default + a registered `v2.0` that exists *only* to prove the
  mechanism — not a shipped language version), resolved from `[project].edition`
  in `Garnet.toml` (the legacy `[package]` table and `edition = "garnet-0.3"`
  value are accepted as **deprecated aliases** with a one-line warning; an
  unknown edition is a hard error). The single edition-gated surface difference
  is the reserved word `async` (a free identifier under `v1.0`, rejected at lex
  time under `v2.0`), confined to the lexer so the grammar and AST are untouched.
  **One-canonical-IR invariant proven:** source valid in both editions parses to
  a byte-identical AST and an identical capability manifest (`Manifest::build`
  ast_hash). **Layer 2 (runtime settings, GODEBUG-style):** `GARNET_DEBUG=k=v`
  flips a CLI default (`diagnostics=verbose`) without changing the manifest;
  unknown keys warn, never error. Wired into `garnet check` and
  `garnet run --interp`; the three shipped `garnet new` templates move to the
  canonical `[project]` / `edition = "v1.0"` form. Adds the
  `edition_compatibility` readiness lane (committed-truth; headline 88.0% →
  88.3%) and regenerates the baseline. **Honest scope (mechanism + invariant
  only):** no per-edition syntax-migration catalog; `garnet run --vm` uses the
  default edition (the VM has a separate load path, per the S12/S14 split); no
  manifest `[runtime]` table yet (env var only — a `[runtime]` table would be a
  spec change, deferred to a future Handoff); no Mini-Spec edit (§16.3 already
  specifies the canonical edition form).

- **S31 PR-2 (deterministic readiness reporter — committed-truth / local-evidence split):**
  `scripts/garnet_mit_readiness_status.py` no longer mixes machine-independent
  committed evidence with machine-variable live probes. Each lane is tagged
  `evidence_class`: **committed** lanes (scored from committed repo state;
  byte-identical on every machine) feed the headline % and the
  `--check-no-regression` gate; **local** lanes (`windows_linux_distribution`,
  `windows_linux_domain_proof_matrix`, `promo_video` — live Windows/Linux build
  gates, the `~/Desktop` domain-proof bundle, local promo render) are reported in
  a separate "Local evidence (not scored, not gated)" section. This fixes the
  cross-machine false regression (`windows_linux_distribution` 60% on a Mac vs a
  70% baseline captured on a Windows-capable machine) that broke the hand-off
  gate, and makes the headline **byte-identical on every machine** (committed-truth
  **88.0%**). Adds the `reporter_determinism` lane, regenerates the committed
  baseline (43 lanes, `evidence_class`-tagged), and advances the goal ledger
  `s31 → merged`. **Honest scope:** the split is at the aggregation layer; per-lane
  committed-vs-live decomposition inside the wls/promo sub-reporters is future
  work, and the committed docs-site `78.0%` snapshot vs the live `88.0%` is a
  separate release-truth sync, not done here.

- **S31 PR-1 (v0.8 release truth + slice ledger + readiness contract):** lands
  the v0.8 map (`F_Project_Management/SLICE_PLAN_RECONCILED_OPUS_X_CODEX.md`) and
  the per-slice acceptance contracts (`F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md`,
  S31–S40 detailed, S41–S80 banded; v0.8.0@S60 / v0.8.1@S70 / v0.8.2@S80; **1.0
  held past S80**). **Adopts** the upgraded `dogfood-readiness` skill
  (`Navigata1/dogfood-readiness`: internal×external merge-confidence fusion via
  `min`, grep-loop-to-5/5, persisted goal-mode ledger) rather than rebuilding
  readiness machinery, and **reconciles** the garnet PR-body gate
  (`scripts/check_dogfood_pr_body.py`) to accept either `### Desktop dogfood
  bundle` (legacy) or `### Evidence bundle` (skill) — backward-compatible, locked
  by 4 new/updated unit tests. Initializes the S31→S80 goal ledger at
  `.dogfood/goal.json`. **Honest scope:** doctrine + ledger + gate reconciliation
  only — no runtime behavior changes and **no new readiness lane**; the
  deterministic-reporter fix (committed-truth split, machine-independent %) and
  its `reporter_determinism` lane land in **S31-PR2**.

- **S30 (functional-core composition capstone):** with the full functional `core::`
  surface now interpreter-dispatched (S26 result, S27 option, S28 iter), proves they
  **compose** into railway-oriented pipelines from Garnet source.
  `garnet-interp-v0.3/tests/functional_core_composition.rs` exercises BOTH tracks —
  `core::iter` collect/map/fold → 20; `core::result` Ok-railway → 40 and Err-railway
  recovered via `or_else` → 0; `core::option` Some → 80 and None default → 7
  (`[20,40,0,80,7]`). The deterministic, cross-platform
  `examples/novel_07_functional_core_pipeline.garnet` composes the happy path
  (iter → result → option → `novel_07 final: 80`) and joins the novel-composition
  harness (now **7/7**); the story is in
  `C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md`. New
  `functional_core_composition` readiness lane is `verified`; MIT readiness moves
  **85.9% -> 86.2%** (42 lanes; baseline surgically extended). Additive — a new
  example + a new test + docs; no parser/CST or owned-crate source change.
  **Honest scope:** pure managed-mode compute (no host effects); the host-effect
  composition is the separate S25 capstone.
- **S29 (`@stability` error-level enforcement, opt-in):** ships the Layer Policy §4
  "error-level enforcement is v0.8" line as an opt-in. `garnet-check-v0.3` adds a
  FATAL `CheckError::StabilityError` variant (listed in `CheckReport::ok()`), and
  `stability.rs` promotes experimental/deprecated call sites from non-fatal
  advisories to that fatal error when `GARNET_STABILITY_ERRORS=1` (or `true`) is set
  — frozen stays informational, `stable` silent. The **default is unchanged
  warning-level**, so existing programs and CI stay green. Proven end-to-end through
  the CLI with **no garnet-cli change** (it already exits on `report.ok()`):
  `garnet check examples/novel_04_*.garnet` warns and exits 0, while
  `GARNET_STABILITY_ERRORS=1 garnet check …` prints `stability error:` and exits 1.
  Unit tests cover the policy (experimental/deprecated→error, frozen→info,
  default→warning, via the env-free `advise(error_mode)` core) and the `ok()` fatal
  classification. New `stability_error_enforcement` readiness lane is `verified`; MIT
  readiness moves **85.5% -> 85.9%** (41 lanes; baseline surgically extended).
  **Honest scope:** error mode is process-global via env var; per-source
  `@uses(experimental)` opt-out still needs the S17 parser-annotation handoff.
- **S28 (`core::iter` completion — zip/collect/chain):** dispatches the last three
  registered `core::iter` combinators, so **all 9** are now runnable from Garnet
  source. `stdlib_bridge.rs` adds `core::iter::zip` (pairs two arrays, stopping at
  the shorter), `core::iter::chain` (concatenates), and `core::iter::collect`
  (materializes a sequence — a `Range` expands to its integers, an Array passes
  through). `garnet-interp-v0.3/tests/core_iter_completion_dispatch.rs` proves them
  from source composed with the S21 higher-order `fold`
  (`collect(1..4)`+`collect([10,20])` chained → fold-sum 36; `zip` stops at the
  shorter → `[36,5,3,2,2]`); bridge unit tests cover each plus the non-sequence type
  error. New `core_iter_completion` readiness lane is `verified`; MIT readiness moves
  **85.1% -> 85.5%** (40 lanes; baseline surgically extended). Additive — bridges +
  tests only. **Honest scope:** `collect` materializes a `Range` or passes an Array
  through; there is no lazy iterator protocol to collect (eager `map`/`filter`
  already return arrays).
- **S27 (`core::option` combinator dispatch):** makes the 5 registered
  `core::option` primitives runnable from Garnet source (sibling of S26).
  `stdlib_bridge.rs` dispatches `core::option::{some,none,map,and_then,unwrap_or}`
  at the Value layer over `Option` Variants (Some/None) identical to the prelude
  builders — `map`/`and_then` are higher-order via `call_value`, bound qualified to
  avoid the bare-`map` collision. `garnet-interp-v0.3/tests/core_option_dispatch.rs`
  proves it from source (map over Some, None pass-through, `and_then` chain +
  short-circuit, `unwrap_or` default → `[10,7,6,8,5,99]`); bridge unit tests cover
  each combinator, the None constructor, and the non-`Option` type error. New
  `core_option_dispatch` readiness lane is `verified`; MIT readiness moves
  **84.7% -> 85.1%** (39 lanes; baseline surgically extended). Additive — bridges +
  tests only. **Honest scope:** `and_then` trusts the callee to return an `Option`
  (dynamic typing); ergonomic method syntax is a later follow-on.
- **S26 (`core::result` combinator dispatch):** makes the 6 registered
  `core::result` primitives runnable from Garnet source, continuing the S21/S22
  registry-to-runtime arc. `stdlib_bridge.rs` dispatches
  `core::result::{ok,err,map,and_then,or_else,unwrap_or}` at the Value layer over
  `Result` Variants (Ok/Err) identical to the prelude builders — `map`/`and_then`/
  `or_else` are higher-order via `call_value`, bound under their qualified names so
  `core::result::map` does not collide with the bare `map` (Map constructor) on the
  last-segment fallback. `garnet-interp-v0.3/tests/core_result_dispatch.rs` proves a
  railway-oriented pipeline from source (map over Ok, Err pass-through, `and_then`
  chain + short-circuit, `or_else` recovery, `unwrap_or` default → `[10,7,6,8,0,5,99]`);
  bridge unit tests cover each combinator plus the non-`Result` type error. New
  `core_result_dispatch` readiness lane is `verified`; MIT readiness moves
  **84.3% -> 84.7%** (38 lanes; baseline surgically extended). Additive — bridges +
  tests only; no parser/CST or stdlib-registry change. **Honest scope:** `and_then`/
  `or_else` trust the callee to return a `Result` (dynamic typing); ergonomic method
  syntax (`result.map(..)`) is a later follow-on.
- **S25 (host-effect composition capstone):** proves the runtime surfaces
  completed across S22–S24 compose end-to-end from Garnet source.
  `garnet-interp-v0.3/tests/host_effect_composition.rs` runs a `@caps(proc, fs)`
  program that captures a host command's stdout (`std::process::output`, S23),
  appends a leveled line to a file (`std::log::to_file`, S24), keeps an episodic
  Mnemos trace (`memory::episodic`, S22), reads the sink back (`read_file`), and
  binds `crypto::blake3` provenance — asserting the composed token, recall count,
  file contents, exit code, and fingerprint (cfg-selected command + unique temp
  path → deterministic on every host). The deterministic, cross-platform
  `examples/novel_06_observability_provenance_pipeline.garnet` composes the
  side-effect-free subset (json + file-sink + episodic memory + blake3) and joins
  the novel-composition harness (now **6/6**); the story is in
  `C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md`. New
  `host_effect_composition` readiness lane is `verified`; MIT readiness moves
  **83.9% -> 84.3%** (37 lanes; baseline surgically extended). Additive only — a
  new example + a new test + docs/scripts; no parser/CST or owned-crate source
  change. **Honest scope:** the deterministic example omits the platform-variable
  process step (proven in the cfg-guarded integration test); execution is still
  synchronous managed-mode (no async actor/OS-thread claim).
- **S24 (`std::log` file sink with `@caps(fs)`):** closes the S22/S23-deferred
  `std::log` file-sink line (also named in the S17 lane's deferred list).
  `garnet-stdlib`'s `log` module adds `to_file(path, level, message)`, which
  formats the same `[LEVEL] message` line as `info`/`warn`/`error`/`debug` and
  **appends** it (plus a newline, create-if-missing) to a file — so unlike the
  pure formatters it requires `@caps(fs)` — and returns the formatted line so a
  caller can both persist and use it. `registry.rs` tags `std::log::to_file`
  Layer 1 / cap `fs` / `@stability(experimental)`; `stdlib_bridge.rs` dispatches
  it (arity 3). Proof is source-level: `garnet-interp-v0.3/tests/stdlib_s24_dispatch.rs`
  has a `@caps(fs)` Garnet `main` write two lines via `std::log::to_file` then read
  them back with `read_file`, asserting ordered contents; the stdlib unit tests
  prove append-not-truncate and the IO error path. New `log_file_sink_runtime`
  readiness lane is `verified`; MIT readiness moves **83.4% -> 83.9%** (36 lanes;
  baseline surgically extended). **Honest scope:** line-append text sink only
  (no rotation, structured/JSON sinks, or async writers); capability enforcement
  remains the checker's job (registry-tagged `@caps(fs)`).
- **S23 (`std::process` structured argv + output capture):** closes the
  S22-deferred process line. `garnet-stdlib`'s `process` module adds
  `spawn_args(program, [args])` — the program and each argument are handed to the
  OS **literally**, so an argument containing spaces is not re-split the way the
  whitespace-delimited `spawn(cmdline)` does — and `output(program, [args])`,
  which runs a child to completion and captures its **stdout/stderr/exit-code**
  (the v0.7 `spawn`/`wait` pair discarded captured output and kept only the exit
  code). `spawn`/`wait`/`exit_code` are unchanged. `stdlib_bridge.rs` dispatches
  both under their qualified names; `std::process::output` returns a
  `{code, stdout, stderr}` map for ergonomic consumption. Proof is source-level:
  `garnet-interp-v0.3/tests/stdlib_s23_dispatch.rs` runs a host command from a
  `@caps(proc)` Garnet `main`, asserts the captured stdout contains the marker and
  reports the exit code, and round-trips `spawn_args` + `wait`; the stdlib unit
  tests prove (on POSIX, via the `printf "%s"` discriminator) that a spaced
  argument survives as a single argv element. New `process_runtime_completion`
  readiness lane is `verified`; MIT readiness moves **82.9% -> 83.4%** (35 lanes;
  baseline surgically extended). **Honest scope:** process stdout/stderr are
  host-dependent (line endings, locale), so the deterministic proof asserts
  substring + exit-code, not byte-exact full output; execution is still
  synchronous managed-mode (no async/OS-thread or streaming-stdout claim);
  capability enforcement remains the checker's job, unchanged.
- **S22 (Stdlib + memory runtime dispatch completion):** closes the S21-deferred
  runtime surface. The parser now accepts selected keywords **only as qualified
  path segments**, so official APIs such as `std::regex::match`,
  `std::process::spawn`, and `memory::working` are callable without making those
  words legal bare identifiers. `stdlib_bridge.rs` now dispatches `std::json`
  (parse/get/set/stringify), `std::regex` (compile/match/find_all/replace),
  `std::uuid` (v4/v5/v7), `std::env` (get/set/vars), `std::process`
  (spawn/wait/exit_code via managed `Process` + `ProcessStatus` values),
  `std::log` (formatting), and `memory::working|episodic|semantic|procedural`
  constructors that return live Mnemos `MemoryStore` handles. Proof is
  source-level, not just host-unit-level: `garnet-interp-v0.3/tests/stdlib_s22_dispatch.rs`
  drives JSON/regex/uuid/log/env/process/memory from Garnet source, and
  `examples/novel_05_s22_stdlib_memory_pipeline.garnet` is included in the
  novel-composition smoke harness (now 5/5). New `stdlib_memory_runtime_dispatch`
  readiness lane is `verified`; MIT readiness moves **82.4% -> 82.9%** (34
  lanes; baseline regenerated). **Honest scope:** `std::env` and `std::process`
  are proven in Rust integration tests rather than the deterministic novel
  example because they mutate or launch host state; `std::process::spawn` still
  uses the v0.7 whitespace-delimited command contract; `std::log` remains
  formatting-only until the v0.8 `@caps(fs)` file-sink lane.

- **S21 (Interpreter dispatch for the S17 Layer-0/1 stdlib + Mnemos × stdlib):**
  closes the S17 deferred line — the new stdlib primitives now **execute from
  Garnet source**, not just sit in the registry. `garnet-interp-v0.3/src/eval.rs`
  resolves the **fully-qualified name first** (backward-compatible — `Storage::read_block`
  → top-level still works), so primitives bind under their full path
  (`core::math::sqrt`) without colliding with bare prelude builtins that share a
  last segment (`map` = Map ctor, `ok`/`err` = Result builders). `stdlib_bridge.rs`
  adds qualified dispatch for **`core::math`** (abs/sqrt/pow/floor/ceil/round —
  dispatches `garnet_stdlib::math`), **`core::cmp`** (min/max/clamp/ordering —
  Value-level), **`core::iter`** (map/filter/fold/take/drop/enumerate — map/filter/fold
  are **higher-order via `call_value`**, i.e. first-class-function combinators
  callable from managed Garnet), and **`std::base64`** (encode/decode — dispatches
  `garnet_stdlib::base64`): 18 newly-runnable primitives. Verified live
  (`core::math::sqrt(16.0)→4`, `core::iter::map([1,2,3,4], double)→[2,4,6,8]`,
  `std::base64::encode("hi")→"aGk="`) and via the new runnable
  `examples/novel_04_dispatched_stdlib_pipeline.garnet` (deterministic;
  `garnet check` emits the expected non-fatal `@stability` warnings on the
  experimental prims, exit 0). `garnet-interp-v0.3/tests/mnemos_stdlib_combination.rs`
  composes the **four Mnemos memory kinds** (working/episodic/semantic/procedural)
  with the dispatched stdlib (BLAKE3 provenance + base64) — real memory-core ×
  stdlib at the system level. New `interp_stdlib_dispatch` readiness lane
  (`verified`); MIT readiness rises; baseline surgically extended (per-lane floors
  preserved). **Honest scope:** `std::json` / `regex` / `uuid` / `env` / `process`
  / `log` dispatch and a full managed-mode `memory::` prim family (Garnet-callable
  Mnemos with a handle Value) are **S22**; `core::cmp`/`core::iter` are Value-level
  bridges (Garnet's dynamic `Value` can't be the stdlib's monomorphic `T`), with
  the `garnet_stdlib` generics as the tested Rust reference.

- **S20 (Novel-composition dogfood + program-execution discovery):** adds three
  runnable `examples/novel_*.garnet` programs that **fuse** multiple Paper-VI
  contributions per program (the existing corpus proves each in isolation):
  `novel_01` fuses capability-budget gating + cognitively-typed memory recall +
  the researcher→synthesizer→reviewer pipeline (capability-aware, memory-gated
  agent → deterministic governance score 16); `novel_02` fuses BLAKE3 signed
  provenance + a multi-stage pipeline + determinism (content-addressed,
  tamper-evident build lineage → verified fingerprint); `novel_03` fuses a
  release-evidence gate + capability budget + provenance + memory recall into a
  multi-signal release-governance quorum (APPROVED quorum 4). `scripts/smoke_garnet_novel_compositions.py`
  (+ `test_garnet_novel_compositions.py`, 8 tests) proves all three `garnet check`
  clean and `garnet run` with **exact deterministic** output; the composition
  story is `C_Language_Specification/GARNET_NOVEL_COMPOSITIONS.md`. New
  `novel_composition_dogfood` readiness lane (`verified`); MIT readiness
  81.3% → **81.9%** (32 lanes; baseline regenerated to the full snapshot, which
  also caught up two lanes that had drifted upward). Complements (does not
  duplicate) the #232 domain proof matrix. **Honest scope:** compositions are
  **modeled deterministically** in managed mode (the proven runnable subset +
  `crypto::blake3`) — they prove the composition *shape* executes and is
  reproducible, not live runtime integration of actor mailboxes / Mnemos stores /
  Ed25519 signing (tracked separately); the new S17 Layer-0/1 stdlib primitives
  are not interpreter-dispatched yet, so these programs use the proven subset.

- **Windows/Linux Studio Domain Proof Matrix:** adds
  `scripts/smoke_garnet_studio_domain_matrix.py`, a manifest-backed
  cross-platform smoke reporter for the current executable example corpus. The
  matrix runs 20 examples through `garnet parse`, `garnet check`, and
  `garnet run`: the 10 canonical MVP domains, signed hot-reload success,
  signed hot-reload mismatch rejection, five agent toolbelt programs, and the
  three agentic design programs. The mismatch case passes only when Garnet
  rejects it with the expected BLAKE3 fingerprint diagnostic. The Tauri Studio
  Release Evidence panel now exposes this reporter as "Domain Proof Matrix";
  MIT readiness tracks the Windows/Linux distribution sub-lane with a separate
  domain-proof lane, which is only `verified` when a manifest-backed matrix
  evidence bundle is present, while keeping Linux package choice, Windows
  ARM64, signing, and winget open.

- **S15 (Trivia-preserving CST via rowan — PR-1: trait surface + stub):** new
  `garnet-cst/` crate (rowan-backed), built **cold** for the v0.7
  build-both-then-compare A/B. Publishes the stable surface S16 (LSP precision)
  targets: the full `SyntaxKind` / `GarnetLanguage: rowan::Language` binding,
  `SyntaxNode` / `SyntaxToken` aliases, the `CstNode` trait, `Parse<T>`,
  `SyntaxError`, `cst_to_source`, and `parse_cst`. In PR-1 `parse_cst` is an
  **intentionally trivial stub** (whole source as one trivia leaf —
  byte-identical round-trip, no structural parsing); the structural
  recursive-descent builder and the `cst_to_ast` projection land in PR-2.
  `u16` ⇄ `SyntaxKind` conversion is `unsafe`-free. Ships 6 round-trip /
  invariant tests + 1 doc-test + a `proptest` proving the stub round-trips any
  UTF-8 input. Registered in workspace `Cargo.toml`, the root `AGENTS.md`
  contract index, and `scripts/check-agent-contracts.py`. Adds `rowan`
  (MIT/Apache-2.0; clears `cargo deny`). **Honest scope:** #221's in-parser CST
  (`garnet-parser-v0.3/src/cst.rs`) is preserved untouched as the S15-Compare
  baseline — this rowan crate is a *second, independent* implementation; the
  canonical-CST choice is the separate S15-Compare checkpoint (Jon), not this
  PR. No readiness lane yet — the `parser_cst_migration` lane + baseline
  regeneration land in PR-2 with the substantive evidence.

- **S15 (Trivia-preserving CST via rowan — PR-2: substantive builder + `cst_to_ast`):**
  `garnet-cst` gains a **direct recursive-descent CST builder** (`builder.rs`)
  over the trivia-preserving token stream, cold from Mini-Spec v1.0 §2–§11 —
  architecturally distinct from #221's AST-projection CST (the
  build-both-then-compare A/B). `parse_cst` now produces real composite
  structure (items, signatures, blocks, the 11-level expression tower,
  patterns, types) and round-trips **byte-identically** across the canonical
  example corpus + a `proptest` over arbitrary UTF-8 (round-trip is guaranteed
  by construction — every token emitted in order, plus a trailing flush, so it
  holds even for malformed input). Adds typed-node wrappers (`nodes.rs`,
  the S16-facing surface) and `cst_to_ast` (`convert.rs`) projecting onto
  `garnet_parser::ast::Module`, validated by **span-normalized structural
  parity** vs `parse_source` across the corpus (`tests/cst_to_ast_parity.rs`).
  New Criterion bench `parse_cst_vs_ast`: the CST path measures **≈0.99× the
  AST path** (well under the 1.5× gate). New readiness lane
  `parser_cst_migration` (`verified`); MIT readiness 78.0% → 78.8%; baseline
  regenerated. **Honest scope:** error-recovery parsing is best-effort
  (round-trip always holds; structure may flatten on malformed input);
  incremental re-parsing and CST-first migration of interp/check/vm are v0.8
  (consumers stay on `parse_source`, untouched); the **canonical-CST choice is
  the separate S15-Compare checkpoint (Jon)** — this is the second of two
  independent CSTs by design, not yet reconciled.

- **S15-Compare (CST reconciliation):** canonical CST decision recorded:
  the rowan-backed `garnet-cst/` crate is the v0.7/S16 target, while #221's
  in-parser CST stays temporarily as a legacy migration oracle. The useful part
  of #221 was preserved rather than discarded: `garnet-cst/src/tokens.rs` now
  exposes `TokenInfo`, `token_infos`, `token_kind`, `token_span`, and
  `identifier_spans`, giving LSP consumers the same `TokenKind` payload +
  byte-span ergonomics on top of rowan. New
  `tests/parser_cst_token_parity.rs` proves the rowan token view matches #221's
  parser CST token stream across the example corpus, excluding the parser's
  zero-width EOF sentinel. `garnet parse --mode cst <file>` now routes through
  the canonical rowan path and reports token count, CST error count, root kind,
  and byte-identical round-trip status; default `garnet parse <file>` remains
  AST mode.

- **S17 (Stdlib expansion + layer policy + `@stability`):** codifies Garnet's
  five-layer stdlib model in `C_Language_Specification/GARNET_STDLIB_LAYER_POLICY.md`
  (layer model, promotion/deprecation policy, the `@stability` semantics table,
  and the "capability surface + spec volatility = layer assignment" first-order
  principle). Expands `garnet-stdlib` from **24 → 77 primitives**: new Layer-0
  `core::` combinators (`iter`, `result`, `option`, `cmp`, `math`) and Layer-1
  `std::` modules (`json`, `regex`, `base64`, `env`, `process`, `uuid`, `log`),
  each a real Rust host function with behavioral unit tests (138 stdlib tests).
  Every primitive now carries an explicit `Layer` + `Stability` tier in
  `registry.rs` (existing 24 → `stable`; the 53 additions → `experimental`);
  `garnet_stdlib_layer_gate.py` enforces ≥ 50 primitives and ≥ 95% explicit
  `@stability` (live: 100%). Adds a compiler-enforced `@stability` advisory in
  `garnet-check-v0.3/src/stability.rs` — calls into `experimental`/`deprecated`
  primitives warn, `frozen` is info — **non-fatal** (exit code unchanged), read
  from the registry. Adds `@caps(env)` as a known capability (for `std::env`).
  New `stdlib_layer_policy` readiness lane (`verified`); MIT readiness
  79.6% → **80.4%** (on top of S16's lane; baseline regenerated to the full
  28-lane snapshot). New deps `serde_json`/`regex`/`rand`
  (already in the lockfile) + `sha1` (RustCrypto sibling of `sha2`); `base64`
  hand-rolled. **Honest scope:** `@stability` enforcement is **warning-level**
  for backwards compat (error-level is v0.8); source-level `@stability(...)` /
  `@uses(experimental)` / `@migration(...)` on **user-defined** functions is
  **pending a parser handoff** to mac-opus (the annotation parser rejects
  unknown names today) — primitive-stability enforcement ships now, user-function
  enforcement in a follow-up; the new Layer-0/1 primitives ship as registry
  surface + Rust host impls + unit tests, while **interpreter dispatch** of them
  to Garnet source is v0.8 (`garnet-interp` is outside S17's ownership);
  Layer-2 `@garnet-lang/*` packages are S18.

- **S16 (Rowan-backed LSP precision):** `garnet-lsp` now consumes the canonical
  rowan `garnet-cst` token/span surface for rename and semantic tokens while
  preserving parser/check diagnostics. The precision smoke
  `scripts/smoke_garnet_lsp_precision.py` proves document symbols, workspace
  symbols, cross-file function rename, scoped parameter rename, three code
  actions (`Add @caps`, long-parameter refactor, inferred return type), and
  semantic-token categories for `capability`, `attribute`, and `parameter`.
  `editors/vscode` is bumped to `0.7.0`, packages
  `garnet-0.7.0-lsp-precision.vsix`, and exposes the three Garnet quick-fix
  commands. MIT readiness gains an `editor_lsp_precision` lane and moves
  78.8% -> 79.6%. **Honest scope:** precision is managed-mode only;
  cross-package rename, safe-mode precision, per-project token themes, and
  Marketplace/OpenVSX publication remain v0.8/follow-up work.

- **S13 (Registry stub v0.1):** new `garnet-registry-stub/` crate — a
  filesystem-backed registry where an `index.json` (serde) maps
  `name → version → { path, BLAKE3-per-file }` over `<name>/<version>/`
  package directories. `garnet-registry-stub build|verify` generates and
  checks the index deterministically. `garnet add --registry <location>
  <name>@<version>` (in `garnet-cli/src/cmd/add.rs`) loads the index,
  resolves the exact version, verifies every file's BLAKE3 (refusing any
  index `path` that canonicalizes outside the registry root), and vendors
  the package into `.garnet/vendor/<name>/` with a registry-shaped
  `Garnet.toml` entry + `Garnet.lock` provenance. Because the S12 resolver
  loads vendored deps at `garnet run` time, a registry-resolved
  `use <name>::*` resolves end-to-end (`examples/registry_stub_fixture/` +
  `garnet-cli/tests/registry_add.rs`, 3 integration tests; 6 stub-crate
  unit tests incl. tamper-detection + path-traversal refusal). New
  "Registry stub v0.1 (S13)" lane in `garnet_mit_readiness_status.py`
  (verified 100 %); MIT lane count 23 → 24, headline 74.3 % → 75.4 %.
  Documented in `C_Language_Specification/GARNET_REGISTRY_v0_1.md`. Honest
  deferred list: HTTP(S) transport (filesystem / `file://` only); tarball
  packaging (packages are directories); auth / accounts / publish flow;
  signature verification (the index `signature` field is reserved but
  unread); SemVer ranges (exact `<name>@<version>` only); multi-registry
  resolution; transitive dependency resolution from the registry.

- **S14 (Bytecode VM v0.2 — explicit call-frame stack + ABI v0.2):**
  `garnet-vm/src/vm.rs` now executes native function calls on an explicit,
  heap-allocated call-frame stack (`Frame` + `run_frames` + `step`) instead
  of recursing in the host (Rust) language. Before S14, deep Garnet recursion
  overflowed the Rust stack (`countdown(100000)` via `--vm` aborted with a
  stack overflow); after S14, `countdown(200000)` and mutual recursion to
  depth 500 run to completion. The codec magic is version-bumped
  `GARNVM01` → `GARNVM02` and each function record carries an explicit
  `arity` field that the deserializer cross-checks against the parameter
  vector. New `garnet run --vm --dump-lowering` flag prints the
  native/fallback ratio (`lowered: N%`);
  `examples/mvp_function_call_demo.garnet` reports `lowered: 100%`. New
  workspace test `garnet-vm/tests/function_call.rs` (8 cases: deep recursion,
  mutual recursion deep + shallow, mixed arity, nested returns, ABI v0.2
  round-trip, arity-mismatch rejection, truncation rejection). New Criterion
  bench case for the call hot path. New
  "Bytecode VM v0.2 function-call lowering (S14)" lane in
  `garnet_mit_readiness_status.py` (verified 100 %); MIT lane count 22 → 23,
  headline 73.2 % → 74.3 %. Documented in
  `C_Language_Specification/GARNET_BYTECODE_v0_2.md` (v0.1 stays for archival
  reference). Honest deferred list: tail-call optimization (each call costs
  one heap frame); closures / captured environments / dynamic-receiver
  dispatch, pattern matching, try/rescue/ensure, struct/enum constructors all
  still fall back; `and`/`or` short-circuit native lowering (Ruby-style
  operand-returning semantics need value-preserving conditional-jump + `Dup`
  opcodes); `--vm`-path vendored-dependency pre-load (the S12 resolver is
  `--interp` only); stable cross-version bytecode ABI (`GARNVM02` is
  tightened, not frozen); production native-compiler proof.

- **S12 (Package-manager resolver contract):** `garnet-cli/src/cmd/run.rs::preload_dependencies`
  reads `Garnet.toml`'s `[dependencies]` table via the new
  `garnet-cli/src/cmd/add.rs::read_dependency_table`, walks each declared
  vendor directory, and pre-loads every `.garnet` source into the
  interpreter's global environment **before** the user source is loaded.
  `Item::Use(_)` in the interpreter stays a no-op; the vendored symbols
  are already in scope by the time `use <dep>::*` is reached. New
  workspace integration test `garnet-cli/tests/run_resolver.rs` covers
  the end-to-end round trip (and a guard test that bare-file runs
  outside any project still work). Four inline unit tests in
  `garnet-cli/src/cmd/run::tests` cover the `strip_top_level_main`
  defence that prevents a vendored dep's own `main` from shadowing the
  user's entry point. New "Package-manager resolver (S12)" lane in
  `garnet_mit_readiness_status.py` (verified 100 %); MIT lane count
  21 → 22, headline 71.9 % → 73.2 %. **Closes the S3 deferred line
  on resolver** (the existing "Garnet manifest + vendored deps" lane's
  deferred list no longer mentions resolver). Honest deferred list for
  S12: qualified-path resolution (`local_lib::hello()`), remote sources,
  transitive deps, SemVer matching, workspace mode, VM-path pre-load
  (S14 will harmonize), lockfile BLAKE3 verification at run time,
  name-collision handling between deps (last-loaded wins today),
  module-scoped `use local_lib::Foo::bar` paths.

- **S11 (v0.6 slice contract scaffold):** new
  `F_Project_Management/GARNET_v0_6_SLICE_DOGFOOD.md` ports the v0.5
  contract pattern (state machine, common verification primitives,
  cross-slice gates, PR body template, integration-with-scripts table,
  honesty anchors) to v0.6 and defines the v0.6.0 release gate plus
  contracts for S12–S16. New
  `F_Project_Management/ROADMAPS/GARNET_v0_6_LANGUAGE_RUNTIME_ROADMAP.md`
  records the v0.6 thesis ("v0.5 shipped scaffolds; v0.6 makes them
  load-bearing"), the confirmed slice order, what's explicitly deferred
  to v0.7+, the target lane delta (`71.9 % / 21 lanes / 12 verified` →
  `≥ 80 % / ≥ 25 lanes / ≥ 17 verified` after S16), and v0.6 honesty
  anchors. `F_Project_Management/DOGFOOD/GARNET_DOGFOOD_READINESS_SKILL.md`
  is refreshed in place from its v0.4.2 / 86 slices / `6e945d6` pulse to
  the current v0.5.x / 87 slices / `e43d378` pulse, with both readiness
  reporters distinguished (implementation-plan vs. MIT-lane). Scaffolding
  only — no reporter lanes added (those land with their respective
  slices, matching the S0 pattern); no baseline regeneration.

- **S18 (First five Layer-2 packages — local-registry source-ready):**
  adds `tools/garnet-lang-template/` as the reusable official-package scaffold
  and `examples/garnet_lang_registry_seed/` with local filesystem-registry
  v0.1.0 seeds for `http-client`, `llm`, `cli`, `test-property`, and `log`.
  `examples/mvp_18_all_official_packages/` plus
  `scripts/smoke_garnet_lang_packages_seed.py` vendors all five packages
  through the existing S13 registry stub and runs one primitive from each.
  New readiness lane `official_packages_seed` is labeled
  `local-registry-source-ready` (85.0%). **Honest scope:** this is source and
  filesystem-registry proof, not public package publication:
  `github.com/garnet-lang/` is a Jon/manual org step and is not visible to the
  active token yet; the five external `github.com/garnet-lang/*` repos and
  their CI are still pending. Source-level `@stability(...)` remains pending
  the parser annotation handoff, so v0.7 package stability is declared in
  `Garnet.toml`/docs while package source stays runnable.

- **S19 (Compiler-as-agent LLM tier — feature-gated source-ready):** new
  `garnet-suggest-llm/` crate behind the non-default `llm` Cargo feature. The
  crate runs S10 deterministic suggestions first, builds a prompt that treats
  those findings as ground truth, emits separate `LlmSuggestion` advisories
  tagged `@stability(non-deterministic)`, and writes
  `.garnet-cache/llm-suggest-log.jsonl` with prompt hash, provider/model,
  temperature, raw response, emitted suggestions, timestamp, token budget, and
  warnings. Provider-compatible Anthropic, OpenAI, and Ollama clients use an
  explicit `LlmTransport` boundary; no API key is written to the repro log.
  `scripts/check_determinism_no_llm.py` and its CI hook fail if the
  determinism workflow ever contains `--llm`. The Paper VI Exp 3 harness ships
  at `benchmarks/paper_vi_exp3_compiler_as_agent/` with ten codebase snapshots,
  stateless/history-aware runners, and aggregate/analyze scripts. New readiness
  lane `compiler_agent_llm_tier` is labeled
  `feature-gated-source-ready` (85.0%); after the S17 merge on current
  `origin/main`, the combined live MIT readiness pulse reports 82.1%.
  **Honest scope:** this is not a shipped end-to-end CLI claim yet:
  `garnet-cli` is read-only for mac-codex, so `garnet check --suggest --llm`
  is filed as a ledger handoff; the shared `garnet-lang/llm` package trait
  waits on S18 after S17; streaming, tools/function calling, vision, and
  provider-specific edge features remain v0.8+; running Paper VI Exp 3 to
  produce h3 results is v0.7.1 work.

### Fixed

- **CHANGELOG.md merge-conflict markers:** resolved the live
  `<<<<<<< HEAD / ======= / >>>>>>> 407e6ec (S3: garnet add …)` markers
  under `[Unreleased] — v0.5.1 in flight`. Both the S7 (PR
  [#213](https://github.com/Island-Dev-Crew/garnet/pull/213)) and S3 (PR
  [#211](https://github.com/Island-Dev-Crew/garnet/pull/211)) entries
  are legitimate; S7 merged first, and the conflict against the
  already-merged S7 CHANGELOG addition slipped through PR #211's merge.
  Resolution: drop the markers, keep both entries in merge order (S7
  first, then S3). No content changes to either entry.

## [Unreleased] — v0.5.1 in flight

### Added

- **S7 (Actor OS-thread bridge / `trust-report`):** new
  `garnet trust-report <file.garnet>` command (`garnet-cli/src/cmd/trust_report.rs`)
  produces a structural trust report including the literal line
  `actors: N / threads: N`, matching the contract's dogfood grep. The
  count is structural — `garnet-actor-runtime/src/runtime.rs` already
  spawns one OS thread per actor (its header documents the
  "Spawn-and-mailbox runtime" contract); S7 lands the CLI bridge that
  surfaces what the runtime does. New
  `examples/agent_orchestrator_3thread.garnet` is the three-actor
  fixture; `garnet-cli/tests/trust_report.rs` asserts the dogfood block
  on every `cargo test --workspace`. New "Actor OS-thread bridge" lane
  in `garnet_mit_readiness_status.py` (verified 100%). Honest deferred
  list documents that live-runtime instrumentation, mailbox/Sendable
  audit, and transitive caps aggregation are out of scope. Closes
  Paper VI Contribution 4's CLI-bridge surface gap.
- **S3 (`garnet add` + Manifest Spec v0.1):** new `garnet-cli/src/cmd/add.rs`
  implements `garnet add [--name <id>] <path>` to vendor a local Garnet
  directory into `.garnet/vendor/<name>/`, update `Garnet.toml`'s
  `[dependencies]` table, and write `Garnet.lock` with BLAKE3-per-file
  hashes. Lockfile output is deterministic (alpha-sorted deps, lex-sorted
  files, lowercase hex). Format documented in
  `C_Language_Specification/GARNET_MANIFEST_v0_1.md`. New
  "Garnet manifest + vendored deps" lane in
  `garnet_mit_readiness_status.py` (verified 100%). Honest deferred list
  documents that the interpreter does NOT yet resolve `use <dep>::*` at
  `garnet run` time, remote sources / transitive deps / SemVer matching /
  workspace mode / `garnet verify-deps` are all out of scope until later
  slices.
- **S6 (Memory eviction policy benchmarks):** `garnet-memory-v0.3/benches/eviction.rs`
  is a Criterion bench harness exercising `MemoryPolicy::score` +
  `should_retain` per Mnemos kind (working / episodic / semantic /
  procedural) against a naive FIFO baseline. `scripts/garnet_memory_eviction_status.py`
  inventories per-kind coverage; `scripts/test_garnet_memory_eviction_status.py`
  asserts the harness keeps all four kinds covered with both branches.
  New "Memory eviction policy benchmarks" lane in
  `garnet_mit_readiness_status.py` (verified 100%). Closes the S6 contract
  surface and half of Paper VI Contribution 3's production-allocator gap.
  Honest deferred list documents that a fresh Criterion measurement run,
  end-to-end store-throughput benches, and the production allocator path
  itself remain separate work.
- **S4 (Formatter idempotent baseline):** `garnet-cli/tests/fmt_idempotency.rs`
  proves that two passes of `garnet fmt --stdout` over every canonical
  `examples/{mvp_,det_}*.garnet` produce byte-identical output, and that
  three runs on the same input produce identical bytes. This makes the
  S4 contract goal (deterministic, idempotent source formatter) workspace-
  test-enforced. New "Formatter idempotent baseline" lane in
  `garnet_mit_readiness_status.py` (verified 100%). Honest deferred list
  documents that AST-driven semantic formatting, comment-preserving
  round-trip, and workspace-level fmt are NOT in scope until the parser
  grows a trivia-preserving CST.

## [0.5.0] — 2026-05-20

### Added

- **v0.5.0 organization release validation:** the `v0.5.0` GitHub Release now
  exists at `13a5805250dc0777ca9212f2214fff5d07247e7b` with Linux `.deb`/`.rpm`
  packages, macOS aarch64/x86_64 CLI tarballs, unified `SHA256SUMS`, and
  darwin-arm64/linux-x64 VSIX assets from green tag workflows. Release-only
  M5 evidence is sealed at
  `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-release-validation-20260520T142443Z`:
  `scripts/verify_org_release_smoke.sh` passed against the org release without
  source fallback, the installer honestly fell back from the unavailable `.pkg`
  to the aarch64 tarball, `garnet new --template cli` / `garnet test` /
  `garnet run` passed from the installed release binary, and the published
  darwin-arm64 VSIX produced the injected standalone VS Code diagnostic. This
  is still not Apple Developer ID notarization, a signed/notarized macOS `.pkg`,
  Marketplace/OpenVSX publication, or Windows/Linux target-runtime proof.
- **v0.5 macOS release tarball path:** `.github/workflows/linux-packages.yml`
  now stages macOS CLI tarballs for `aarch64-apple-darwin` and
  `x86_64-apple-darwin`, then composes one release-time `SHA256SUMS` covering
  Linux `.deb`/`.rpm` packages plus those tarballs. This closes the pre-tag
  workflow gap that would make an M5 Mac release-only installer smoke fail
  after publication. The tag-time publication and release-only smoke evidence
  are recorded in the v0.5.0 organization release validation entry above; this
  remains not a signed/notarized `.pkg`. Fresh local M5 file-backed release-mode
  evidence is sealed at
  `/Users/idc2.0/Desktop/dogfood/garnet-macos-cli-tarball-release-assets-20260520T135703Z`.
- **v0.5 release-backed VSIX path:** `scripts/package_garnet_vscode_extension.sh`
  now builds `garnet-lsp`, packages the VS Code extension with the bundled
  native server, writes host-labeled VSIX evidence, and can copy a sealed bundle
  to Desktop. `.github/workflows/vscode-extension.yml` builds those VSIX
  artifacts on PR/main/tag runs and publishes them as GitHub Release assets on
  `v*` tag pushes. `scripts/verify_org_release_smoke.sh` now fails the release
  smoke unless the matching release-backed VSIX asset exists and contains the
  extension entry point plus bundled server. The tag-time publication and
  release-backed diagnostic proof are recorded in the v0.5.0 organization
  release validation entry above; this remains not Marketplace publication or
  OpenVSX publication. Fresh M5 local evidence is sealed at
  `/Users/idc2.0/Desktop/dogfood/garnet-vscode-release-assets-20260520T133747Z`
  for `garnet-0.5.0-lsp-mvp-darwin-arm64.vsix`.
- **v0.5 release-gate evidence:** post-merge public installer source-fallback
  proof is recorded in
  `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-rc-merged-20260520T121820Z`, and
  Mac-local Cursor/VSIX diagnostic proof is recorded in
  `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-editor-gate-20260520T122611Z`.
  The latter includes the local `garnet-0.5.0-lsp-mvp.vsix`, installed
  `island-dev-crew.garnet@0.5.0` extension evidence, a screenshot showing
  `1 problem in this file` / `Errors: 1`, and protocol smoke JSON for
  diagnostics, hover, and go-to-definition. Clean standalone VS Code 1.121.0
  diagnostic proof is recorded in
  `/Users/idc2.0/Desktop/dogfood/garnet-v0-5-standalone-vscode-gate-20260520T130303Z`:
  the locally packaged VSIX contains `extension/server/garnet-lsp`, installs
  into isolated user-data/extensions directories, launches without
  `garnet.lsp.path`, and shows the injected syntax-error diagnostic.
- **S8 (Signed hot-reload BLAKE3 demo):** `examples/mvp_11_signed_hotreload.garnet`
  and `examples/mvp_11_signed_hotreload_mismatch.garnet` are runnable
  managed-mode demonstrations of the BLAKE3 fingerprint check that drives the
  Rust-runtime `actor.reload_signed` path. The success example exits 0 with
  `reloaded successfully` on stdout; the mismatch example exits 1 with
  `BLAKE3 fingerprint mismatch` on stderr. New "Signed hot-reload BLAKE3 demo"
  lane in `garnet_mit_readiness_status.py` (verified 100%). Honest deferred
  list documents that managed-mode `actor.reload_signed` syntax is NOT
  exposed yet — the demos use `crypto::blake3` and `raise` to reproduce the
  fingerprint-mismatch behaviour at the program level. Closes Paper VI
  Contribution 5 surface gap.
- **S10 (Compiler advisory mode, rules-based):** `garnet-check-v0.3/src/suggest.rs`
  ships a deterministic, no-LLM suggestion engine with three rules today —
  `managed-fn-missing-caps`, `long-parameter-list`, and `empty-function-body`.
  `garnet check --suggest <file.garnet>` surfaces them prefixed with the
  literal `compiler suggested:` so downstream tooling can grep. Corpus test
  `garnet-check-v0.3/tests/suggest_corpus.rs` proves ≥ 3 distinct rules fire on
  3 fixture programs. New "Compiler advisory mode (rules-based)" lane in
  `garnet_mit_readiness_status.py` (verified 100%). Closes Paper VI
  Contribution 7 surface for the rules-based tier; the LLM tier remains
  pending-infra.
- **S5 (Parser fuzz harness):** `garnet-parser-v0.3/fuzz/` cargo-fuzz
  sub-workspace with a single `parse_input` target wrapping every call to
  `garnet_parser::parse_source_with_budget` in a strict `ParseBudget`
  (4096-byte source cap, 1024-token cap, 32-depth cap, 512-byte literal
  cap). New `.github/workflows/fuzz-nightly.yml` runs `cargo +nightly
  fuzz run parse_input -- -max_total_time=3600` nightly + on-demand;
  crashes upload as artifacts for triage. Seed corpus is populated from
  canonical `examples/*.garnet` files. New "Parser fuzz harness
  (nightly)" lane in `garnet_mit_readiness_status.py` (`verified` 100%).
  `scripts/garnet_proof_benchmark_status.py` also inventories the fuzz
  harness as evidence while keeping accumulated nightly fuzz hours unclaimed.
  The fuzz sub-workspace carries explicit license metadata and a scoped
  `cargo deny --manifest-path garnet-parser-v0.3/fuzz/Cargo.toml check`
  record for `libfuzzer-sys`'s permissive NCSA component. Honest deferred list
  documents that the interpreter, checker, and archived v0.2 parser are NOT in
  scope today.
- **S9 (Determinism CI):** `.github/workflows/determinism.yml` builds
  `examples/det_fixture_01.garnet` with `garnet build --deterministic --sign
  <key>` on a matrix of ubuntu-latest and macos-latest. A `prepare-key` job
  generates a single short-lived ed25519 signing key and uploads it as an
  artifact so both OSs sign with identical key bytes; the `compare` job
  diffs the resulting per-OS SHA-256 manifest hashes and fails CI with an
  `::error::` annotation on divergence. Closes Paper VI Contribution 6
  verification gap. New "Determinism CI cross-machine" lane in
  `garnet_mit_readiness_status.py` (`verified` 100%); honest deferred list
  documents that Windows runner and Linux aarch64 are not yet in the
  cross-OS matrix.
- **S0 (housekeeping):** `scripts/garnet_conformance_matrix_check.py` — file-existence
  check on the conformance matrix's evidence column. Advisory by default; `--strict`
  opts into CI-fail behavior. Lands the gate before the existing matrix shorthand
  is repaired so future drift is catchable.
- **S0 (housekeeping):** `--check-no-regression` flag on
  `scripts/garnet_mit_readiness_status.py`. Compares live lane percentages against
  a committed baseline at
  `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json` and exits 1 on any
  drop. Lanes absent from the live output (slice removed/renamed) also trigger
  failure.
- **S0 (housekeeping):** baseline snapshot
  `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json` captured at
  54.2 % overall / 12 lanes from the 2026-05-20 main tip.
- **v0.5 slice contract:** `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` —
  single source of truth for every v0.5 PR. State machine, dogfood blocks,
  honesty anchors, PR template.
- **S1 (LSP MVP):** source-present `garnet-lsp/` language server and
  `editors/vscode/` extension launcher for diagnostics, hover, and basic
  go-to-definition. `scripts/smoke_garnet_lsp_protocol.py` proves those paths
  over stdio; local VSIX packaging now bundles `server/garnet-lsp` from the
  release build, and local install smoke passed in Cursor plus standalone VS
  Code 1.121.0 on this Mac.
- **S2 (Bytecode VM scaffold):** source-present `garnet-vm/` crate with a
  deterministic bytecode serializer, 15 native opcode families, function-level
  tree-walk fallback, `garnet run --vm` / `--interp` dispatch, a bounded
  Criterion VM/interpreter comparison harness, and
  `C_Language_Specification/GARNET_BYTECODE_v0_1.md`. The proof/benchmark
  reporter now inventories the VM harness, and the MIT reporter's proof lane is
  more granular while the overall objective remains active-partial.

### Honest partials

- The `v0.5.0` tag and GitHub Release exist with release-backed installer and
  darwin-arm64 VSIX diagnostic evidence, but that is not proof of Apple
  Developer ID notarization, a signed/notarized macOS `.pkg`,
  Marketplace/OpenVSX publication, or Windows/Linux target-runtime evidence.
- The current Mac has Cursor as `/usr/local/bin/code`, not the standalone VS
  Code CLI. Clean standalone VS Code diagnostic proof exists through an
  isolated downloaded VS Code 1.121.0 app, including the release-backed
  darwin-arm64 VSIX installed from the GitHub Release.
- The S1 LSP slice is source-present until Marketplace/OpenVSX publication and
  full manual VSCode hover/go-to-definition screenshots are attached to later
  review/release evidence. Safe-mode hover, workspace symbols, rename, and
  CST-grade incremental precision remain deferred.
- The S2 VM is a scaffold, not a production VM. It covers 15 opcode families for
  the MVP fixtures, falls back to the tree-walk interpreter at unsupported
  function boundaries, and does not claim a stable bytecode ABI, production
  native compiler proof, full safe-mode lowering, or standing benchmark
  measurements in the status reporter.
- The S5 fuzz harness is source-present with local 60-second dogfood evidence
  and scheduled nightly coverage, not a claim that one-hour nightly fuzz has
  already accumulated or that parser correctness is proven.

### Known Advisory Gates (inherited, not yet fixed)

- Conformance matrix shorthand: 9 path-like references in
  `C_Language_Specification/GARNET_v0_4_2_Conformance_Matrix.md` do not resolve
  to files on disk today. The new check surfaces these as advisory findings; a
  future slice will fix the matrix and flip the gate to strict.

## Historical record

For the v0.4.2 (research-grade) verification ledger and earlier phase logs, see
`F_Project_Management/GARNET_v4_2_HANDOFF.md` and the dated `GARNET_v*_HANDOFF`
files. Pre-CHANGELOG history was tracked in those handoff documents; from v0.5
onward this file is the canonical entry point.
