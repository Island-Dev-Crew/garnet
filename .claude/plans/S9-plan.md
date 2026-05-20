# S9 — Determinism CI Cross-Machine — Implementation Plan

Date: 2026-05-20
Contract: `F_Project_Management/GARNET_v0_5_SLICE_DOGFOOD.md` § S9
State: not-started → **planned**
Reviewer: Jon (Island Development Crew)

> S9 is one of the six v0.5.0 release gates. PR title: `S9: Determinism CI cross-machine`.
> Closes Paper VI Contribution 6 verification gap.

## 1. Why this slice can land now

- `garnet build --deterministic --sign <key>` already exists in v0.4.2; no compiler surface needs to be added.
- Local two-run sanity check confirms identical SHA-256 on the same machine with same source + same key.
- Cross-OS determinism risk is bounded: manifest contains source_hash + ast_hash + signature; ed25519 over the same canonical body produces byte-identical signature when the input bytes and key bytes are identical.
- No new Rust dependency; this slice is a workflow + fixture + readiness-lane.

## 2. Concrete artifacts

- `examples/det_fixture_01.garnet` (new) — minimal `@caps()` managed-mode program with a finite arithmetic loop. Tiny on purpose. Comment block documents why it must stay tiny.
- `.github/workflows/determinism.yml` (new) — three jobs:
  - `prepare-key` (ubuntu-latest): builds garnet, runs `garnet keygen det.key`, uploads as 1-day-retention artifact. Single key shared across the OS matrix so signatures match.
  - `build` (matrix `[ubuntu-latest, macos-latest]`): downloads the key, runs `garnet build --deterministic --sign det.key examples/det_fixture_01.garnet`, captures `shasum -a 256` of the resulting manifest, uploads per-OS hash.
  - `compare`: downloads both per-OS hashes, diffs them, fails with `::error::` annotation if they differ.
- `scripts/garnet_mit_readiness_status.py` — add a new "Determinism CI cross-machine" lane sourced from the workflow file's presence; `status="verified"` when the workflow file exists. Reflects the v0.5.0 release-gate readiness.
- `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json` — regenerated to include the new lane (so subsequent slices' `--check-no-regression` doesn't false-positive).
- `CHANGELOG.md` — `## [Unreleased]` Added entry under v0.5.0.

## 3. Honest partial labels (for the PR body)

- "Linux x86_64 + macOS aarch64 only" — Windows is not in the cross-OS comparison. Adding Windows requires a separate signed-runner key path and is left for a follow-up slice.
- "ed25519 only" — alternative key types (RSA, secp256k1) not in scope.
- "no multi-key rotation testing" — the gate proves manifests are byte-identical for a fixed key, not that key rotation produces a stable distinct manifest.
- "key lives 1 day as a CI artifact" — not committed to repo. Re-running an old CI workflow that needs the key requires a fresh run.

## 4. Dogfood block

Per contract S9 (runs in CI on matrix):

```bash
garnet build --deterministic --sign /tmp/keys/test.key examples/det_fixture_01.garnet
sha256sum examples/det_fixture_01.garnet.manifest.json > /tmp/hash-$RUNNER_OS.txt
# Upload as artifact; comparison job diffs the two hashes and fails if they differ.
```

The committed workflow is faithful to this, with two adaptations:
- `shasum -a 256` instead of `sha256sum` (macOS provides the former; the workflow uses `shasum` which is present on both).
- The compare job uses `cut -d' ' -f1` to drop the filename column before comparing — only the hash digest matters.

## 5. State-machine transitions

| Transition | Evidence |
|---|---|
| not-started → planned | this file |
| planned → in-progress | draft PR `S9: Determinism CI cross-machine` opens |
| in-progress → review-ready | new Determinism workflow's compare job green on PR |
| review-ready → dogfood-passing | Jon review + CHANGELOG + baseline regenerated |
| dogfood-passing → merged | squash-merge |

## 6. Risks and mitigations

- **Key generation is random per run, breaking determinism across jobs.** Mitigation: single `prepare-key` job generates once; matrix jobs download the same artifact.
- **`shasum -a 256` output format differs by OS.** Mitigation: workflow uses `cut -d' ' -f1` to extract just the hash.
- **Workflow needs to compile garnet on each matrix runner.** Mitigation: actions/cache for `~/.cargo` + `target` keyed by `Cargo.lock` hash. First runs are slower; cached runs are fast.
- **A future Rust/garnet change could alter manifest serialization order or hashing.** Mitigation: this gate runs on every PR, so such a change would surface immediately and require explicit acknowledgement (regression note in PR body per the contract).
- **The single shared key in CI is harmless from a security standpoint** — it never leaves the artifact retention window (1 day) and was never used to sign anything outside the CI run. Documenting this in the workflow header so a future contributor doesn't try to reuse it.

## 7. Out of scope

- Reproducibility beyond manifest hash (e.g. byte-for-byte binary output once a native backend exists). The current `garnet build` produces a manifest only.
- SLSA build provenance attestation — separate slice.
- GPG-signing `SHA256SUMS` for the universal installer — separate slice.
- Windows runner in the matrix.
