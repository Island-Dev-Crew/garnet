# S47 Plan — Windows / Linux / macOS build proof (+ Windows-propriety audit)

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S47.
Map: reconciled plan §148-149 — build proof + the Windows-propriety audit
("does every attribute behave cross-platform, or is it just packaging?").
Branch: `codex/s47-build-proof`. Base: `origin/main` @ `088f97d` (S46).

## Hard constraint → honest-partial
Single-OS (macOS) checkout; cannot run Windows/Linux locally. The cross-OS
evidence lives in CI (`ci.yml` test job matrix `[ubuntu, windows, macos]` running
`cargo test --workspace`; `linux-packages.yml` deb/rpm + macos-cli-tarballs). So
S47 aggregates + gates that coverage and audits portability — it does not re-run
the foreign-OS builds.

## Deliverables (real, testable behavior — not a pure doc)
- `scripts/garnet_build_proof.py`: parse `ci.yml`/`linux-packages.yml`, classify
  each OS on **behaves** (in the `cargo test --workspace` matrix) and
  **distributes** (packaging). `--format md|json`; `--gate` exits 1 if any of the
  three loses the *behaves* proof. Dependency-free (regex/text, no PyYAML).
- `scripts/test_garnet_build_proof.py`: 7 unit tests (matrix parsing, real-CI
  coverage, gate passes, markdown).
- Wire into `ci.yml` `agent-contracts` job: run the test + `--gate`.
- `F_Project_Management/GARNET_BUILD_PROOF.md`: status table + Windows-propriety
  audit (per-surface cross-platform status).

## Dogfood
- `garnet_build_proof.py --format md` → all three behave; Linux+macOS distribute,
  Windows CLI packaging reported as a gap. `--gate` exits 0 today; would exit 1
  if an OS were dropped from the test matrix.

## End-state / gates
- Full ladder green (zero Rust changed; workspace tests confirm 0 failed).
  CHANGELOG + contract S47 block + build-proof doc. Ledger: `s46 → merged(5)`
  advanced this branch; `s47` advance rides with S48.

## Honest scope (do not soften)
- **CI-attested, not locally re-run.** The gate checks the matrix covers all
  three OSes; it does not execute Windows/Linux.
- "Build proof" = builds + `cargo test --workspace` per OS in CI; not a parity
  claim. Windows CLI distribution deferred; S46 seccomp policy a Linux-shaped gap.
- No new readiness lane (not mandated).
