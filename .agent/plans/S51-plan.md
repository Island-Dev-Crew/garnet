# S51 Plan — signed release lanes

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S51.
Map: reconciled plan §154 — "signed release lanes" (v0.8 adoption/release band).
Branch: `codex/s51-signed-release`. Base: `origin/main` @ `2065d01` (S50).

## Landscape (found read-only)
Three signing lanes: (1) program-manifest `garnet build --sign` (Ed25519, CI
round-trip "signature valid") — ACTIVE; (2) release SHA256SUMS detached
signature — DEFERRED (TODO in linux-packages.yml; needs GPG/minisign); (3)
supply-chain `garnet seal` in-toto predicate → cosign — PARTIAL (cosign ABSENT).
Real gap: `garnet seal` told you to `cosign attest --predicate <output>` but had
no way to WRITE the predicate to a file.

## Deliverables
- `garnet seal --out <path>` (real CLI feature): write the predicate to a file;
  cosign hint names the path. (cmd/seal.rs args reworked to `&[String]`.)
- `scripts/garnet_signed_release_lanes.py`: inventory the 3 lanes + `--gate`
  guarding lane 1 (active). `--format md|json`.
- `garnet-cli/tests/seal_attestation.rs`: +1 test for `--out` (CI-gated via
  matrix). `scripts/test_garnet_signed_release_lanes.py`: 6 unit tests.
- Wire lanes test + `--gate` into ci.yml agent-contracts.
- `F_Project_Management/GARNET_SIGNED_RELEASE_LANES.md`: the 3-lane table + scope.

## Dogfood
- `garnet seal app.garnet --out pred.json` → valid in-toto JSON file; default
  stdout unchanged. `garnet_signed_release_lanes.py --format md` → lane 1 active
  (gated), lanes 2/3 deferred/partial; `--gate` exits 0.

## End-state / gates
- Full ladder green (workspace 0 failed incl. new seal test). CHANGELOG +
  contract S51 block + lanes doc. Ledger: `s50 → merged(5)` advanced this branch;
  `s51` advance rides with S52.

## Honest scope (do not soften)
- Garnet does NOT sign its own supply chain / bundle cosign/GPG/minisign.
- Lanes 2–3 deferred/partial by design (external tools absent); reported, not
  faked. Only lane 1 (owned end-to-end) is gated. No new readiness lane.
