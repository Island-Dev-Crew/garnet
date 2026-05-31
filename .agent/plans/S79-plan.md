# S79 — website / deck reframing

## Goal
Reframe Garnet's public messaging to the integration + agent-code thesis (compass
report positioning recommendations): lead with the integration (not pillar-by-
pillar novelty), headline diff-caps, concede precedent honestly.

## What ships
- `F_Project_Management/GARNET_POSITIONING.md` — canonical messaging: one-sentence
  thesis (evidence-native language for agent-written code); "integration, not the
  parts"; precedent-concession table (Austral/Wasmtime/Sigstore stronger on each
  pillar); diff-caps headline; the "why a new language?" answer (Lattner).
- `docs/index.html` — additive reframed `<section class="thesis">` (S79) carrying
  the same thesis; existing sections untouched (section tags balanced 16/16).
- `scripts/garnet_positioning_status.py` (+ `--gate`, 5 tests) — static anti-drift
  gate: doc AND landing page both carry integration thesis + diff-caps headline +
  precedent concession + agent-code target.
- CI agent-contracts; CHANGELOG; contract S79 block; this plan; ledger `s78 →
  merged`.

## Honest scope (do not soften)
A positioning claim about novelty and fit, NOT a production-readiness or 1.0 claim.
Garnet remains a research-grade prototype (v0.x); the pillars are conceded as
well-precedented. No Rust changed.

## Verification
- `python3 scripts/test_garnet_positioning_status.py` → 5 OK; `--gate` rc 0.
- index.html `<section>` tags balanced; landing page renders the new section.
- Ladder: fmt/diff clean; `cargo test --workspace` 0 failed (no Rust changed).
