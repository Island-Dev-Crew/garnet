# Studio Bootstrap Runner And Design Dossier Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the next Windows Studio slice from the merged #426 setup-script foundation into a controlled GUI bootstrap runner, while aligning the follow-on Studio roadmap with the June 28 design dossier.

**Architecture:** Treat the current Setup Assistant as the installer/bootstrap foundation, not the final Studio shape. The immediate implementation adds a typed, allowlisted Rust command for user-triggered bootstrap steps and visible evidence output. The follow-on design work reshapes Studio around the dossier principle: render structured Garnet trust contracts from CLI JSON, never recompute verdicts in the webview, and never imply enforcement the language does not prove.

**Tech Stack:** Tauri v2 Rust commands, TypeScript DOM frontend, Python shell-contract tests, Playwright e2e tests, Garnet CLI JSON surfaces (`diff-caps --machine`, `check --format json`, `agent-loop --record-dir`), repo dogfood evidence conventions.

---

## Current State

- PR #426 is merged on `origin/main` as `a0a93b4 feat(studio): add Windows bootstrap assistant (#426)`.
- Studio can display CLI health and generate operator-run PowerShell scripts under the dogfood evidence root.
- Studio does not yet run those bootstrap steps from the GUI.
- The design dossier at `C:\Users\IslandDevCrew\Downloads\GARNET_STUDIO_DESIGN_DOSSIER_2026-06-28.html` reframes the larger app goal: the app has broad command coverage, but too much of it dumps stdout instead of rendering structured trust contracts.

## Dossier Principles Folded Into This Plan

- **Render contracts, do not dump stdout.** Every new UI surface should prefer machine JSON and structured artifacts over raw `<pre>` output.
- **Never recompute verdicts in TypeScript.** The CLI owns trust verdicts; the webview renders them.
- **Green is not safe.** A 5/5 diff-caps result means no declared widening, not general safety.
- **Enforced and declared must stay visually separate.** `@caps` and `@max_depth` traps are different from declared-only bounds or platform sandbox claims.
- **Bootstrap runner is Phase 0 foundation.** It makes a downloaded Studio capable of getting to useful local runtime state; it is not the final Trust Cockpit.

## Phase 0: GUI Bootstrap Runner

**Files:**
- Modify: `scripts/test_garnet_windows_linux_studio_shell.py`
- Modify: `apps/garnet-studio/e2e/studio-ui.spec.ts`
- Modify: `apps/garnet-studio/src-tauri/src/commands.rs`
- Modify: `apps/garnet-studio/src-tauri/src/lib.rs`
- Modify: `apps/garnet-studio/index.html`
- Modify: `apps/garnet-studio/src/main.ts`
- Modify: `apps/garnet-studio/src/styles.css`
- Create: `F_Project_Management/STUDIO_WINDOWS_BOOTSTRAP_RUNNER_HANDOFF_2026_06_28.html`

- [ ] **Step 1: Write failing shell-contract assertions**

Assert that the backend exposes `studio_bootstrap_run_step`, that the frontend has run controls for preflight, Python install, CLI build, and environment configuration, and that the Tauri permission surface still avoids `tauri-plugin-shell` and stays `core:default`.

- [ ] **Step 2: Run shell contract red**

Run: `python scripts/test_garnet_windows_linux_studio_shell.py`

Expected: FAIL because `studio_bootstrap_run_step` and run-control IDs are absent.

- [ ] **Step 3: Write failing Playwright assertion**

Assert that the CLI Health panel exposes `Run Preflight`, `Install Python`, `Build CLI`, `Configure Env`, and a repo path input.

- [ ] **Step 4: Run e2e red**

Run: `npm --prefix apps/garnet-studio run test:e2e`

Expected: FAIL because the controls are absent.

- [ ] **Step 5: Add backend allowlist tests**

Add Rust tests proving that only `preflight`, `install-python`, `build-cli`, and `configure-env` are accepted, arbitrary command IDs are rejected, and repo-dependent steps require a valid Garnet repo shape (`Cargo.toml` plus `garnet-cli/Cargo.toml`).

- [ ] **Step 6: Implement typed runner**

Add `BootstrapRunRequest`, `BootstrapStep`, repo validation, script selection by enum only, and `studio_bootstrap_run_step_impl`. The command must create a fresh `bootstrap-run` evidence bundle, write the audited scripts there, and run PowerShell through `run_process_with_timeout`.

- [ ] **Step 7: Add frontend controls**

Add a repo path input and explicit buttons for preflight, Python install, CLI build, and environment configuration. Every button must state that the action is local, user-triggered, evidence-recorded, and not a provider/network-agent path.

- [ ] **Step 8: Seven-pass handoff**

Write an HTML handoff with judge/auditor rows for product fit, security, command contract, evidence, Windows UX, test coverage, and Mac continuation.

## Phase 1: Dead-Weight And Honesty Cleanup

**Purpose:** Apply the dossier's "clear the dead weight" recommendation without breaking command coverage.

- [ ] Remove or demote static/stale UI elements only after tests assert replacement behavior.
- [ ] Replace hard-coded status tiles with live truth/reporter-derived data or remove them.
- [ ] Demote static taxonomy from a top-level surface unless it is generated from source.
- [ ] Relabel advisory flows as local evidence only; no backend handoff or provider implication.

## Phase 2: Diff-Caps Review Gate

**Purpose:** Ship the dossier's recommended first substantial Trust Cockpit feature.

- [ ] Add a typed backend wrapper around `garnet diff-caps --machine`.
- [ ] Render `garnet.diff-caps.machine/1` as a verdict card from JSON.
- [ ] Show gained/lost caps per function and hard refusal on widening.
- [ ] Render the CLI caveat verbatim: no declared widening does not prove absence of undeclared authority.
- [ ] Do not recompute bands or verdicts in TypeScript.

## Phase 3: Velocity Editor

**Purpose:** Give Studio a daily-use authoring loop while staying honest about current diagnostic precision.

- [ ] Add a buffer-backed editor path that runs `garnet check --format json` with debounce.
- [ ] Render diagnostics from JSON. Use exact spans when present; otherwise anchor to line/function and say so.
- [ ] Add capability gutter hints and quick-fix proposals only when they are derived from CLI diagnostics.
- [ ] Keep live checks out of sealing/evidence by default so each keystroke does not create dogfood noise.

## Phase 4: Enforced/Declared Legend

**Purpose:** Turn calibrated honesty into a UI invariant.

- [ ] Generate enforced/declared status from repo source or a CLI-reported map, not from hand-written HTML.
- [ ] Badge `@caps` and `@max_depth` as enforced only where trap evidence exists.
- [ ] Badge `@bounded`, mailbox, fan-out, and platform sandbox claims as declared/deferred unless deterministic traps prove otherwise.

## Phase 5: Agent-Loop Console

**Purpose:** Compose the Trust Cockpit and Velocity Loop into the agent supervision surface.

- [ ] Render an `agent-loop --record-dir` dossier as a four-gate pipeline.
- [ ] Reuse the Phase 2 diff-caps card as the authority gate drill-down.
- [ ] Add caps-log and seal panels as structured dossier views.
- [ ] Keep approval, widening, and seal provenance visually separate; never launder a widening into "green".

## Verification Ladder For Any Implementation PR

Run the relevant narrow tests first, then the full Studio ladder:

```sh
cargo fmt --manifest-path apps/garnet-studio/src-tauri/Cargo.toml -- --check
cargo test --manifest-path apps/garnet-studio/src-tauri/Cargo.toml
python scripts/test_garnet_windows_linux_studio_shell.py
python scripts/test_garnet_windows_linux_studio_status.py
npm --prefix apps/garnet-studio run build
npm --prefix apps/garnet-studio run test:e2e
```

For release-impacting Studio changes, also run:

```sh
python scripts/garnet_mit_readiness_status.py
cargo run --manifest-path apps/garnet-studio/src-tauri/Cargo.toml -- --studio-smoke
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
```

## Approval Gate

Do not open or merge a new implementation PR from this branch until the merger assessment and this integrated plan-of-attack are approved. After approval, resume at Phase 0 with TDD red tests.
