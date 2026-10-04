# Garnet signed release lanes (S51)

Garnet's "signed release" posture is not one thing — it is three distinct lanes
with different owners and maturity. This document makes each explicit; the live
status is `scripts/garnet_signed_release_lanes.py --format md`, and `--gate`
(in CI) protects the two lanes that are ACTIVE.

## The three lanes

| # | Lane | Status | Garnet-owned? |
|---|---|---|---|
| 1 | **Program-manifest signing** — `garnet build --sign <key>` (Ed25519 over the deterministic build manifest), verified to `signature valid` in `linux-packages.yml`. | ✅ **active** | yes |
| 2 | **Release-artifact signing** — the tagged release job signs `SHA256SUMS` with `gpg --detach-sign` and uploads `SHA256SUMS.asc`; an unsigned tagged release fails closed unless deliberately allowed. Shipped since v0.8.1. | ✅ **active** | no (GPG, key held in the `release` environment) |
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

## T5b change (0.8.3)

- Lane 2 was still reported as **deferred**, keyed on a stale `TODO(release-security)`
  comment in `linux-packages.yml`, although `SHA256SUMS.asc` has shipped since v0.8.1.
  The comment is gone. The lane is now **active** only while the `release` job is
  the reviewed one: its text matches the SHA-256 pinned in the reporter
  (`RELEASE_JOB_SHA256`), the workflow's top level other than `jobs`, which the job
  inherits, matches `TOP_LEVEL_SHA256` as decoded YAML, and no other job in
  `linux-packages.yml`, nor its top
  level, has a `permissions` key or mentions `secrets` in the decoded YAML (read
  with PyYAML; without it the lane is not active). The reviewed job
  signs with `gpg --detach-sign --armor SHA256SUMS`, refuses an unsigned tagged
  release ("Require signed SHA256SUMS (fail-closed)"), publishes, then attaches
  `SHA256SUMS.asc`. Any edit to the job, or a writer elsewhere in the workflow,
  makes the lane report **broken** and `--gate` fail until a reviewed change moves
  the pin with the job.

## Scope (do not soften)

Garnet does **not** sign its own supply chain and does **not** bundle
`cosign`/`GPG`/`minisign`. Lane 2 uses GPG in the release job, with the key held
in CI. Lane 3 is partial **by design**: it depends on an external signing tool that
is not present in this environment. Every status is reported as it is, never faked.
Lanes 1 and 2 are gated: lane 1 because Garnet owns it end to end, and lane 2
because it is what makes a published `SHA256SUMS` trustworthy.

```sh
python3 scripts/garnet_signed_release_lanes.py --format md   # this table (live)
python3 scripts/garnet_signed_release_lanes.py --gate        # active-lanes regression guard
garnet seal <file.garnet> --out predicate.json               # write the in-toto predicate
```
