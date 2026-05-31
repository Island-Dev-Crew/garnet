# Garnet signed release lanes (S51)

Garnet's "signed release" posture is not one thing — it is three distinct lanes
with different owners and maturity. This document makes each explicit; the live
status is `scripts/garnet_signed_release_lanes.py --format md`, and `--gate`
(in CI) protects the one lane that is actually ACTIVE.

## The three lanes

| # | Lane | Status | Garnet-owned? |
|---|---|---|---|
| 1 | **Program-manifest signing** — `garnet build --sign <key>` (Ed25519 over the deterministic build manifest), verified to `signature valid` in `linux-packages.yml`. | ✅ **active** | yes |
| 2 | **Release-artifact signing** — a detached signature over `SHA256SUMS`. | ⏸ **deferred** | no (GPG/minisign) |
| 3 | **Supply-chain attestation** — `garnet seal [--out]` emits an in-toto predicate over the build + capability manifests, for `cosign attest --predicate`. | ◐ **partial** | no (cosign) |

## S51 changes

- **`garnet seal --out <path>`** — the predicate can now be written to a file, so
  it feeds straight into `cosign attest --predicate <path>`. Previously the seal
  hint said *"sign this predicate with cosign attest --predicate <output>"* but
  there was no `<output>` to point at (print-only). The cosign hint now names the
  written path.
- The lanes reporter + an **active-lane gate**: lane 1 (in-language manifest
  signing, which Garnet fully owns) must stay wired in CI; if the `--sign` →
  `signature valid` round-trip disappears from `linux-packages.yml`, the gate
  fails.

## Honest scope (do not soften)

Garnet does **not** sign its own supply chain and does **not** bundle
`cosign`/`GPG`/`minisign`. Lanes 2 and 3 are deferred/partial **by design** —
they depend on external signing tools that are not present in this environment.
Their status is reported truthfully, never faked. Only lane 1 is gated, because
it is the lane Garnet owns end-to-end.

```sh
python3 scripts/garnet_signed_release_lanes.py --format md   # this table (live)
python3 scripts/garnet_signed_release_lanes.py --gate        # active-lane regression guard
garnet seal <file.garnet> --out predicate.json               # write the in-toto predicate
```
