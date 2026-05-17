# Garnet Priority Slices — Design

Date: 2026-05-17
Status: approved design (brainstorming output) → input to writing-plans
Base truth: `origin/main` @ `ea412a5` (PR #163 merged), MIT readiness 58.6%,
tracked implementation ledger 87/87.

## Problem

A chat gap analysis of garnet-lang.org produced a P0/P1/P2 backlog. The repo
is under heavy concurrent agent load (~40 PRs/day, `docs/index.html` edited
~hourly, a fast `Phase 6Bx` sequence). Two cross-cutting risks:

1. **Phase-ID collisions** between parallel agents (PR #74 and #75 both used
   "Phase 4BI"). No collision-prevention mechanism exists.
2. **Overclaiming risk**: several gap-analysis items describe CLI features
   (`garnet update`, channels, `garnet self uninstall`) and a playground that
   do not exist. The project's brand and CI dogfood gate forbid this.

## Constraints / decisions (user-approved)

- Coordination convention lands **first** (de-risks all later slices).
- Phone `+1 256-652-5534` goes in **SECURITY.md only**, not the public site.
  Email `jon-isaac@islanddevcrew.com` is the primary security contact.
- Toolchain-management copy uses **honest "planned" framing** (no fake commands).
- Every slice is its own dogfood PR: branch from fresh `origin/main` → change →
  verify → Desktop dogfood bundle → PR → merge → next. Honest/deferred-boundary
  language throughout. Small, reviewable, project micro-slice discipline.

## Approach (chosen)

"Coordination-gated micro-slice train." Alternatives rejected: batching the
website into ~4 large PRs (less reviewable, against project discipline);
website-first (leaves the collision risk the user explicitly asked to fix
unaddressed during the run).

## Slice sequence

1. **Phase-counter / collision convention.** Add a phase-ID allocation rule:
   a register section + a small `scripts/garnet_phase_id.py` that derives the
   next free phase id from the implementation plan + ownership register +
   recent git log, and an AGENTS.md/CLAUDE.md rule to call it before choosing a
   phase letter. Verification: unit test for the script; doc rule present.
2. **MIT readiness Windows/Linux lane.** `garnet_windows_linux_studio_status.py`
   landed in #163 but `garnet_mit_readiness_status.py` still has no Windows/
   Linux distribution lane (headline stuck at 58.6%). Add the lane, honestly
   scored from the status script. Verification: `test_garnet_mit_readiness_status.py`.
3. **Contact truth (P0-4).** SECURITY.md: replace placeholder with
   `jon-isaac@islanddevcrew.com` + the phone as a voice/SMS disclosure line.
   Footer: add Code of Conduct + Security (exist) + a new one-page Logo Policy.
4. **Prereq + arch + checksum callout (P0-2 + overlooked).** Install section:
   state Rust 1.75+ needed only for source; explicit supported-arch matrix
   (no Intel-mac/arm-linux/Windows binary in v0.4.2 → source); documented
   `shasum -c SHA256SUMS` verify step.
5. **Toolchain section (P0-1, honest).** "Managed by reinstall today;
   `garnet update`, stable/nightly channels, `garnet self uninstall` are
   PLANNED, not available."
6. **Getting-started page (P0-3).** `docs/getting-started.html` walking
   `garnet new` → first run; linked from install.
7. **GitHub star button in nav (P0-5).**
8. **P1 bundle (separate PRs):** Rust/Ruby/Garnet comparison table; animated
   terminal demo (CSS/JS, no fake output); `docs/playground.html` stub framed
   as planned; community section (Discussions); "A project by Island
   Development Crew" line.
9. **P2 bundle (separate PRs):** monthly blog stub; `robots.txt` + `sitemap.xml`
   + canonical; `prefers-reduced-motion` for starfield/animations; social links
   placeholder (only real handles).

## Success criteria

- Slice 1 merged before any website slice; no new phase-ID collisions after.
- MIT readiness reflects a Windows/Linux lane (score moves or lane visible).
- Every shipped claim is true today or explicitly labeled planned.
- Each slice: green local verification + verified Desktop dogfood manifest +
  merged PR before the next starts.

## Out of scope

Building real `garnet update`/channels (separate language work); a real WASM
playground; signed `.pkg`/MSI/notarization (separate authority lanes);
multilingual selector and build-freshness stamp (deferred P2 tail).
