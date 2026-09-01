# Garnet Priority Slices Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Land the gap-analysis P0/P1/P2 backlog plus a phase-collision convention and an MIT Windows/Linux lane as a sequence of small, truth-matched, individually-merged dogfood PRs.

**Architecture:** Each slice = branch from fresh `origin/main` → minimal change → local verify → Desktop dogfood bundle with verified `MANIFEST.sha256` → PR via `Navigata1` fork → squash-merge → re-sync → next. Truth-matched/"planned" language everywhere; CI `dogfood-readiness` gate must stay green. Website file is `docs/index.html` (GitHub Pages → garnet-lang.org); it changes ~hourly so rebase immediately before each push.

**Tech Stack:** Rust workspace (`garnet-*` crates), Python status scripts + `unittest`, static HTML/CSS/JS site, `gh` CLI, Cargo tests, `shasum` manifests.

---

## Per-slice ritual (applies to every Task)

1. `git fetch origin --quiet && git switch -c codex/<slug> origin/main`
2. Make the change (below).
3. Verify (below) — must be green.
4. Desktop bundle: `/Users/idc2.0/Desktop/dogfood/garnet-<slug>-<UTCstamp>/` with `dogfood-readiness-report.md`, change diff, verification log, `artifact-files.txt`, `MANIFEST.sha256`, `manifest-verify.log` (all OK).
5. Commit (specific files), `git push -u fork codex/<slug>`.
6. `gh pr create --repo Island-Dev-Crew/garnet --base main --head Navigata1:codex/<slug> --no-maintainer-edit --title ... --body-file ...` (body has the `## Dogfood Readiness` section with checked Current/Local/Remote/Desktop/Deferred evidence).
7. `gh pr merge <n> --repo Island-Dev-Crew/garnet --squash`; confirm MERGED; `git checkout main && git fetch origin && git merge --ff-only origin/main`.

---

### Task 1: Phase-ID collision convention

**Files:**
- Create: `scripts/garnet_phase_id.py`
- Create: `scripts/test_garnet_phase_id.py`
- Modify: `AGENTS.md` (add a "Phase ID allocation" rule), `CLAUDE.md` (one-line pointer)

**Step 1 — failing test:** `scripts/test_garnet_phase_id.py` asserts `next_phase_id(["4BH","4BI","6BS"]) == "6BT"`, that lowercase/whitespace are normalized, and that the CLI prints one token + newline.

**Step 2 — run, expect FAIL:** `python3 scripts/test_garnet_phase_id.py` → ModuleNotFound/AssertionError.

**Step 3 — implement** `scripts/garnet_phase_id.py`: scan `F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md`, `F_Project_Management/ROADMAPS/GARNET_v0_5_PHASE_OWNERSHIP_REGISTER.md`, and `git log --oneline -200` for `Phase ([0-9]+[A-Z]+)`; compute the max by (numeric, alpha-base-26) and emit the successor. `--check <id>` exits non-zero if `<id>` is already used (collision guard for CI/agents).

**Step 4 — run, expect PASS.**

**Step 5 — docs:** `AGENTS.md` gains: “Before choosing a phase identifier, run `python3 scripts/garnet_phase_id.py`; never hand-pick a letter; CI/agents may `--check`.” `CLAUDE.md` one-liner pointing to it.

**Verify:** `python3 scripts/test_garnet_phase_id.py`; `cargo fmt --all -- --check` (no-op, docs/py only); `git diff --check`.

**PR title:** “Add garnet_phase_id collision-prevention convention”.

---

### Task 2: MIT readiness Windows/Linux distribution lane

**Files:**
- Modify: `scripts/garnet_mit_readiness_status.py` (add a lane sourced from `scripts/garnet_windows_linux_studio_status.py`, landed in #163)
- Modify: `scripts/test_garnet_mit_readiness_status.py` (assert the new lane row exists and is scored against current evidence)

**Steps:** failing test for a "Windows/Linux distribution" lane row → run FAIL → implement lane (status/percent derived from the windows-linux status script; mark blocked/deferred sub-items explicitly, do not inflate the headline) → run PASS → commit.

**Verify:** `python3 scripts/test_garnet_mit_readiness_status.py`; `python3 scripts/garnet_mit_readiness_status.py | head -25` shows the lane; headline only moves if the script truly justifies it.

**PR title:** “Add Windows/Linux distribution lane to MIT readiness accounting”.

---

### Task 3: Contact truth (SECURITY.md + footer P0-4)

**Files:** Modify `SECURITY.md` (replace `jon@island-dev-crew.example` → `jon-isaac@islanddevcrew.com`; add `Voice/SMS (security only): +1 256-652-5534`). Create `docs/logo-policy.html` (one page: name/mark usage, no-endorsement). Modify `docs/index.html` footer: add “Code of Conduct” (→ GitHub `CODE_OF_CONDUCT.md`) and “Logo Policy” (→ `/logo-policy.html`) links; keep existing Security link.

**Verify:** `python3 - <<'…'` HTML parser feed parses `index.html` and `logo-policy.html` with no exceptions; footer `<footer>` 1/1; `grep -c island-dev-crew.example SECURITY.md` → 0.

**PR title:** “Real security contact + footer Code of Conduct / Logo Policy”.

---

### Task 4: Prerequisite + supported-arch + checksum callout (P0-2 + overlooked)

**Files:** Modify `docs/index.html` install section (id=`install`, ~line 1118). Add, above the tabs: a callout — “Prebuilt release packages need no toolchain. Source install requires Rust 1.75+.” Add an explicit support matrix: v0.4.2 ships `aarch64-apple-darwin` tarball, `amd64` .deb, `x86_64` .rpm + `SHA256SUMS`; **Intel macOS, ARM Linux, and Windows fall back to source**. Add a `shasum -a 256 -c SHA256SUMS` verify snippet.

**Verify:** HTML parses; `pre` open/close balanced; 0 stale phrases; asset names match `gh release view v0.4.2 --json assets`.

**PR title:** “Install: prerequisites, supported-arch matrix, checksum verification”.

---

### Task 5: Toolchain management section (P0-1, truth-matched)

**Files:** Modify `docs/index.html` (new subsection under install). Copy: “Today Garnet is updated by re-running the universal installer (or `cargo install --path` for source). `garnet update`, stable/nightly channels, and `garnet self uninstall` are **planned and not yet available**.” No fake command shown as runnable.

**Verify:** HTML parses; `grep -i "garnet update" docs/index.html` only appears inside the “planned” block; CLI truly lacks it (`target/debug/garnet --help`).

**PR title:** "Install: truth-matched toolchain-management (planned) section".

---

### Task 6: Getting-started page (P0-3)

**Files:** Create `docs/getting-started.html` (same theme tokens as index.html): install → `garnet new --template cli my_app` → `garnet test` → `garnet run src/main.garnet`, with the real expected output. Link it from the install section and nav.

**Verify:** HTML parses; the commands shown match a real run of `target/debug/garnet new/test/run` in `/tmp`.

**PR title:** “Add getting-started walkthrough page”.

---

### Task 7: GitHub repo button in nav (P0-5)

**Files:** Modify `docs/index.html` `<nav class="top">` (~line 745): add a GitHub link/button to `Island-Dev-Crew/garnet` (static “GitHub ★” label; no fabricated star count).

**Verify:** HTML parses; nav still single-line responsive (visual check via preview panel).

**PR title:** “Promote GitHub repo into top nav”.

---

### Task 8: P1 bundle (separate PRs, one per item)

8a Rust/Ruby/Garnet comparison table section (8–10 evidence-scoped axes).
8b Animated terminal demo (CSS/JS typing of a *real* `garnet new→test→run`; no invented output).
8c `docs/playground.html` stub explicitly “planned (browser WASM), not yet available”.
8d Community section (GitHub Discussions link only — real surface).
8e “A project by Island Development Crew” line under hero.

Each: HTML parses + visual preview check; own dogfood bundle + PR + merge.

---

### Task 9: P2 bundle (separate PRs, one per item)

9a `docs/robots.txt` + `docs/sitemap.xml` + `<link rel=canonical>`.
9b `prefers-reduced-motion` media query disabling starfield/animations.
9c Monthly blog stub `docs/blog/index.html` (“first post planned”).
9d Footer social row — only real handles; omit any that don’t exist.

---

## Success criteria

- Task 1 merged before any `docs/index.html` slice; `garnet_phase_id.py --check` rejects a used id.
- MIT readiness shows a Windows/Linux lane (evidence-derived score).
- Every shipped claim verified-true or labeled planned; CI dogfood gate green on each PR.
- Each slice merged before the next starts; main re-synced between slices.

## Out of scope

Real `garnet update`/channels; real WASM playground; signed `.pkg`/MSI/notarization; multilingual selector; build-freshness stamp.
