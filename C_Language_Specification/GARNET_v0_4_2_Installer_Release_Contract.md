# Garnet v0.4.2 Installer and Release Contract

**Status:** org `v0.4.2` GitHub Release published; release-backed installer
smoke passed with the macOS tarball fallback.

This document is the truth-first contract for the v0.4.2 installer path. It
describes what the checked-in repository supports today, what is live in the
org release, and which signed/native distribution surfaces remain
credential-gated.

## Goal

```sh
curl --proto '=https' --tlsv1.2 -sSf https://garnet-lang.org/install.sh | sh
```

The installer must:

1. Detect OS and architecture.
2. Select the correct release artifact.
3. Download the artifact from GitHub Releases.
4. Verify the artifact against `SHA256SUMS`.
5. Install through the native installer or tar fallback.
6. Run `garnet --version`.
7. Fail clearly if the release, checksum file, or asset is missing.

## A. Installer Hosting Spec

- Public URL: `https://garnet-lang.org/install.sh`
- Canonical source: `installer/sh.garnet-lang.org/install.sh`
- GitHub Pages copy: `docs/install.sh`
- Sync rule: both files must be byte-for-byte identical.
- CI guard: `.github/workflows/linux-packages.yml` runs `cmp -s` and shellcheck
  against both copies.
- Content type: `text/plain` or `application/x-sh`
- Cache policy: short cache, recommended `max-age=300, must-revalidate`
- Rollback: revert the `docs/install.sh` copy and let GitHub Pages redeploy.

## B. Release Artifact Spec

Authoritative backend:

```text
https://github.com/Island-Dev-Crew/garnet/releases/download/v<version>/
```

The installer uses an explicit version tag by default instead of the moving
`latest` alias. Override `GARNET_TAG` or `GARNET_BASE_URL` only for smoke tests,
pre-release validation, or emergency rollback.

Expected v0.4.2 assets:

| Platform | Asset |
| --- | --- |
| Debian/Ubuntu x86_64 | `garnet_0.4.2-1_amd64.deb` |
| Fedora/RHEL x86_64 | `garnet-0.4.2-1.x86_64.rpm` |
| macOS universal | `garnet-0.4.2-universal.pkg` |
| Windows x86_64 | `garnet-0.4.2-x86_64.msi` |
| Tar fallback | `garnet-0.4.2-<triple>.tar.gz` |
| Checksums | `SHA256SUMS` |

The org `v0.4.2` release currently publishes the Linux `.deb`, Linux `.rpm`,
macOS `aarch64-apple-darwin` tarball fallback, and `SHA256SUMS` assets.
macOS `.pkg` and Windows `.msi` assets are still credential-gated and must be
attached manually or by future signing-aware workflows before claiming native
signed installer coverage on those platforms.

## C. Integrity Spec

`SHA256SUMS` is required. The installer downloads it before the asset and
refuses to install any file whose SHA-256 does not match.

Accepted `SHA256SUMS` filename forms:

```text
<sha256>  garnet_0.4.2-1_amd64.deb
<sha256> *garnet_0.4.2-1_amd64.deb
<sha256>  linux/garnet_0.4.2-1_amd64.deb
<sha256> *linux/garnet_0.4.2-1_amd64.deb
```

Release-signature verification is not yet implemented in the installer. Do not
claim minisign, Ed25519 release-key, or signature-pinned installer verification
until a public key is pinned and a failing verification path is tested.

Platform signing remains required before publication:

- macOS: Developer ID signed, notarized, and stapled `.pkg`.
- Windows: Authenticode signed and timestamped `.msi`.

## D. Release Pipeline Spec

Current automated release path:

1. Push `v0.4.2`.
2. `.github/workflows/linux-packages.yml` builds the release binary.
3. CI packages `.deb` and `.rpm`.
4. CI smoke-tests clean Ubuntu and Fedora installs.
5. CI shellchecks both installer copies and verifies they match.
6. CI uploads Linux packages and `SHA256SUMS` to the GitHub Release.

Manual/credential-gated release path:

1. Build macOS `.pkg` with `garnet-cli/macos/build-pkg.sh`.
2. Verify `pkgutil --check-signature`, `spctl --assess --type install`, and
   `xcrun stapler validate`.
3. Sign Windows MSI with Authenticode and verify with `signtool verify`.
4. Attach macOS and Windows assets to the same `v0.4.2` GitHub Release.
5. Regenerate or replace `SHA256SUMS` so every attached asset is listed.
6. Smoke-test `https://garnet-lang.org/install.sh` on macOS and Linux.
7. Document Windows install through direct MSI until `install.ps1` exists.

## Current Live State

As of 2026-05-15, the public org release exists at
`https://github.com/Island-Dev-Crew/garnet/releases/tag/v0.4.2`, points at
merge commit `6e945d6a151c2b97ae842c21aeeaa5a678ae65f5`, and contains:

- `garnet_0.4.2-1_amd64.deb`
- `garnet-0.4.2-1.x86_64.rpm`
- `garnet-0.4.2-aarch64-apple-darwin.tar.gz`
- `SHA256SUMS`

`./scripts/verify_org_release_smoke.sh` passed against `Island-Dev-Crew/garnet`
by downloading the org release checksums, falling back from the unavailable
signed macOS `.pkg` to the published macOS tarball, verifying SHA-256,
installing into a temporary prefix, and running `garnet --version`.

The repository copy `docs/install.sh` is present and synchronized with
`installer/sh.garnet-lang.org/install.sh`. A live public-domain smoke for
`https://garnet-lang.org/install.sh` remains a separate DNS/GitHub Pages check
unless that domain is verified during the release pass.
