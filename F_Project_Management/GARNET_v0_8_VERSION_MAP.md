# Garnet v0.8 version map — the source of truth (corrected 2026-05-31)

This is the **single authoritative version-tag mapping** for the S30–S80
completion run. Where any other document disagrees, this file governs. It
supersedes the earlier "v0.8.0 @ S60 / v0.8.1 @ S70 / v0.8.2 decision @ S80"
mapping (which survives, banner-corrected, only in historical artifacts).

## The corrected mapping

| Slices | Band | Tag semantics |
|---|---|---|
| S31–S40 | v0.8 foundation | (in flight — no tag) |
| S41–S50 | v0.8 hardening | beta **readiness gate** @ S50 (no tag) |
| S51–S60 | v0.8 adoption / release | **v0.8.0 readiness checkpoint @ S60** — *deferred, not a cut* |
| S61–S70 | native-interop + provenance | **source-of-truth checkpoint @ S70** — *no tag* |
| S71–S80 | v0.8 validation runway | **v0.8.0 CUT DECISION @ S80** — the single tag for the whole run |

- **The entire S30–S80 run is cut as exactly one tag — `v0.8.0` — at the *end*,
  after S80.** There is no `v0.8.1` and no `v0.8.2` tag inside this window.
- **S60 and S70 are readiness *checkpoints*, not cuts.** Both "tag" slices in the
  original plan were always escalated to Jon and were deferred; no tag was ever
  pushed. The honest record is: only `v0.4.2` and `v0.5.0` are tagged today.
- **S80 is the v0.8.0 cut decision** (was labelled "v0.8.2 readiness decision").
  The cut remains Jon's call — irreversible, release-strategy-bearing, reserved
  by the honesty anchors — and is never made autonomously.

## After S80

- **S81+ is the runway to v0.8.1.** The exact slice range is set in a future plan
  (candidates floated: S81–S100, then S81–S110). That arc adds a
  **solutions-oriented real-world-proofs** theme — agents that run real-world
  builds, tests, solutions, and simulations — and the public "what only Garnet
  does" positioning, to be planned with the finished v0.8.0 in hand.
- **S91 plan note:** `F_Project_Management/GARNET_v0_8_1_PLAN.md` now seeds the
  active v0.8.1 S91-S110 plan. `.dogfood/goal.json` is the active S91-S110
  ledger; `.dogfood/v0_8_goal.json` archives the finished S31-S80 ledger so
  v0.8.0 release gates remain reproducible while v0.8.1 advances.
- **1.0 is held much further out — plausibly ~a year** — and is gated on real
  testing, validation, and maturity, never on slice count.

## Why this changed

Jon's 2026-05-31 decision (at the S70 checkpoint): slow down for precision and
validation; make the S30–S80 run a coherent, defensible **single v0.8.0
milestone** rather than emitting premature sub-tags. This run will be reviewed by
**Carnegie Mellon, MIT, Rice, and UC Berkeley** before a public presentation, so
the calibrated-honesty doctrine and per-slice rigor are paramount: no faked
runtime enforcement, cross-platform proof, release readiness, or 1.0 claims.

## Enforcement

`scripts/garnet_version_map_check.py --gate` (CI, agent-contracts job) asserts
this file states the corrected mapping and that the operative contract
(`GARNET_v0_8_SLICE_DOGFOOD.md`) does not reintroduce a `v0.8.1`/`v0.8.2` *cut*
inside the S30–S80 window. Historical artifacts may retain the old text only
behind a dated correction banner.
