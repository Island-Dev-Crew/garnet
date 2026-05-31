# S52 Plan — one-line install / readme check (Kelley)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S52.
Map: reconciled plan §155 — "one-line install / readme check (Kelley)".
Branch: `codex/s52-install-check`. Base: `origin/main` @ `65dc733` (S51).

## Gap
README.md (the `curl … install.sh | sh` fence) and
`installer/sh.garnet-lang.org/install.sh` (its self-documented bootstrap header)
can DRIFT — the #1 adoption footgun.

## Deliverables
- `scripts/garnet_install_readme_check.py`: extract the one-line install command
  from README + installer header; normalize; assert identical + canonical URL in
  both. `--format md|json`; `--gate` exits 1 on drift.
- `scripts/test_garnet_install_readme_check.py`: 6 unit tests (extraction,
  comment-marker strip, drift/none, real-repo consistency, gate).
- Wire test + `--gate` into ci.yml agent-contracts.
- CHANGELOG + contract S52 block.

## Dogfood
- `garnet_install_readme_check.py --format md` → commands match, URL in both,
  consistent; `--gate` exits 0.

## End-state / gates
- Full ladder green (zero Rust changed; workspace 0 failed). Ledger: `s51 →
  merged(5)` advanced this branch; `s52` advance rides with S53.

## Honest scope
- Doc-consistency check, NOT a live network install test. install.sh shellcheck
  is a separate CI job — not duplicated. No new readiness lane.
