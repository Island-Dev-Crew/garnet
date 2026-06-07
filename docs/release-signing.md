# Garnet release signing & verification

How to verify the integrity — and, when signing is enabled, the authenticity — of a
downloaded Garnet release.

> **Honest status (2026-06-07).** Every release ships a `SHA256SUMS` manifest
> (integrity). **The release signing key is now configured** (`GPG_SIGNING_KEY` is
> set), and the public key is published at
> [`docs/garnet-release-signing.pub.asc`](garnet-release-signing.pub.asc) with
> fingerprint **`04D5 6F91 F038 17DD FFEB  C62A C14D F6E7 1395 6ED1`**. The
> **`v0.8.1` Release (re-cut 2026-06-07) is signed** — it carries
> `SHA256SUMS.asc`. Earlier releases (e.g. `v0.8.0`, `v0.5.0`) predate signing and
> are **unsigned** (research-grade default), **not** tampered. Garnet is a
> research-grade prototype, not production/1.0.

## 1. Integrity — always available

Every GitHub Release attaches a `SHA256SUMS` covering each installer artifact. After
downloading the asset(s) and `SHA256SUMS` into one directory:

```sh
sha256sum --check --ignore-missing SHA256SUMS
```

Every line you downloaded must print `OK`.

## 2. Authenticity — when the release is signed

When the maintainer's signing key is configured, the release also attaches
`SHA256SUMS.asc` (a detached GPG signature) and the public key is published at
[`docs/garnet-release-signing.pub.asc`](garnet-release-signing.pub.asc) in this repo.

```sh
# one-time: import the published public key
gpg --import garnet-release-signing.pub.asc

# confirm you imported the right key — the fingerprint MUST be:
#   04D5 6F91 F038 17DD FFEB  C62A C14D F6E7 1395 6ED1
gpg --fingerprint jon-isaac@islanddevcrew.com

# verify the signature over the checksum manifest
gpg --verify SHA256SUMS.asc SHA256SUMS
```

A `Good signature from "Garnet Release Signing ..."` line means the checksums (and
therefore the artifacts that match them) were signed by the holder of that key. Chain
the two checks: verify the signature on `SHA256SUMS`, then verify each artifact
against `SHA256SUMS`.

## 3. What this does and does not attest

- **Does:** the artifacts match a checksum manifest signed by the maintainer's key.
- **Does not:** independently attest the *build* (no reproducible-build witness here),
  nor replace the in-language artifact provenance — Garnet's own `garnet build --sign`
  / `garnet verify` Ed25519 manifests and the in-toto `seal` are a separate, additive
  layer for `.garnet` artifacts. A CycloneDX **SBOM** (`garnet-sbom-cyclonedx.tgz`) is
  also attached to signed/unsigned releases alike.
- Cosign/Sigstore keyless signing and a reproducible-build attestation remain
  **deferred** (pre-1.0 work).

## For maintainers — enabling signing

1. Provision the private key as a repo secret (run on an admin account; the private
   key never leaves your machine except as the encrypted secret):
   ```sh
   gpg --armor --export-secret-keys <KEY_ID> | gh secret set GPG_SIGNING_KEY --repo Island-Dev-Crew/garnet
   gh secret set GPG_PASSPHRASE --repo Island-Dev-Crew/garnet   # only if the key has one
   ```
2. Publish the **public** key here so downloaders can verify:
   ```sh
   gpg --armor --export <KEY_ID> > docs/garnet-release-signing.pub.asc
   ```
3. Cut (or re-cut) a release tag. The `release` job in
   `.github/workflows/linux-packages.yml` signs `SHA256SUMS → SHA256SUMS.asc` when the
   secret is present, and publishes an honest `::notice::` (unsigned) when it is not.
