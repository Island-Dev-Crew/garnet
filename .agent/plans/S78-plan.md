# S78 — governance / RFC process (+ donate cap-manifest standard)

## Goal
Formalize how Garnet changes land via an RFC + edition process over ad-hoc BDFL,
and record the capability-manifest standard donation intent (the Opus graft,
reconciliation §174) — honestly, for a single-maintainer research-grade project.

## What ships
- `GOVERNANCE.md` — canonical governance: single-maintainer (Jon Isaac / Island
  Dev Crew) today; RFC + edition process as the change-landing mechanism; honest
  status (no steering committee / foundation). Consistent with the existing
  `docs/governance.html`.
- `rfcs/README.md` (the RFC process + lifecycle), `rfcs/0000-template.md`,
  `rfcs/0001-capability-manifest-standard.md` (Draft — standardize the
  capability-manifest format + OWASP/LF donation intent; references the real S68
  cap-manifest standard).
- `scripts/garnet_governance_status.py` (+ `--gate`, 6 tests) — static gate:
  GOVERNANCE.md honest + RFC process present + RFC-0001 references the real
  standard + marks the donation as intent/draft (not accepted).
- CI agent-contracts; CHANGELOG; contract S78 block; this plan; ledger
  `s77 → merged`.

## Honest scope (do not soften)
Single-maintainer governance for a research-grade prototype — no steering
committee, no foundation, no adopted standard. The OWASP/LF capability-manifest
donation is INTENT + a draft (RFC-0001), not accomplished. A governance/process
slice; no Rust changed.

## Verification
- `python3 scripts/test_garnet_governance_status.py` → 6 OK; `--gate` rc 0.
- Ladder: fmt/diff clean; `cargo test --workspace` 0 failed (no Rust changed).
