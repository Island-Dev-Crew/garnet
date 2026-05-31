# S70 — version-map source-of-truth correction (no tag)

## Reframe
Originally "v0.8.1 tag". At the S70 checkpoint (2026-05-31) Jon revised the
version mapping and asked to correct the S30–S80 docs into a proper source of
truth before continuing to S80. So S70 ships a **documentation correction +
docs-consistency gate**, not a tag.

## Corrected mapping (the new truth)
- The whole **S30–S80 run is cut as one `v0.8.0` tag at the END of S80**.
- **S60 and S70 are readiness checkpoints, not cuts** (both were deferred; no tag
  was ever pushed — only v0.4.2 / v0.5.0 are tagged).
- **S80 is the v0.8.0 cut decision** (was "v0.8.2 readiness decision").
- **S81+ is the runway to v0.8.1**; **1.0** is held much further out (~a year),
  gated on validation, not slice count.
- Academic-review bar: CMU / MIT / Rice / UC Berkeley → maximum rigor + honesty.

## Decision recorded
Continue through S80 on the current spine (no plan-now pivot). The
solutions-oriented real-world-proofs replan (S81+) is done at the S80 boundary
with the finished v0.8.0 in hand. Research read: the compass trajectory report +
the reconciled slice plan — both validate the spine and "integrate don't rebuild".

## Deliverables
- `F_Project_Management/GARNET_v0_8_VERSION_MAP.md` — authoritative source of truth.
- Correct `GARNET_v0_8_SLICE_DOGFOOD.md` band table + forward bands + honest label
  + S60-block correction + append S70 block.
- Dated correction banners on `SLICE_PLAN_RECONCILED_OPUS_X_CODEX.md`,
  `GARNET_v0_8_0_RELEASE.md`, `GARNET_v0_8_BETA_GATE.md` (preserve history, don't
  rewrite it).
- Retitle ledger `s70` → `source-of-truth-checkpoint`, `s80` → `v0.8.0-cut-decision`.
- `scripts/garnet_version_map_check.py` (+ `--gate`) + 5 unit tests; CI wiring.
- CHANGELOG entry; this plan.

## Verification
- `python3 scripts/test_garnet_version_map_check.py` → 5 OK.
- `garnet_version_map_check.py --gate` → rc 0.
- Ladder: fmt/diff clean; `cargo test --workspace` 0 failed (no Rust changed).

## Honest scope (do not soften)
Docs + a docs-consistency gate only. **No tag is cut, pushed, or authorized.**
Tagging stays a human release-truth decision; the single `v0.8.0` cut is the S80
decision.
