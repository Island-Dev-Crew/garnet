# Garnet v0.4.2 Release Publication Runbook

Date: 2026-05-06

This runbook is the release-side continuation of the dogfood readiness
remediation. It records exactly what can be published automatically, what still
requires credentials, and what permission is needed.

## Permission Gate

Official release assets must be published under:

```text
github.com/Island-Dev-Crew/garnet/releases/tag/v0.4.2
```

The publishing identity must have `WRITE`, `MAINTAIN`, or `ADMIN` permission on
`Island-Dev-Crew/garnet`. A `READ`-only identity can build artifacts locally but
cannot create the official release or upload assets.

Check permission:

```sh
gh repo view Island-Dev-Crew/garnet --json viewerPermission
```

## What The Existing Workflow Publishes

`.github/workflows/linux-packages.yml` publishes release assets when a `v*` tag
is pushed:

- `garnet_0.4.2-1_amd64.deb`
- `garnet-0.4.2-1.x86_64.rpm`
- `SHA256SUMS`

The workflow also smoke-tests the `.deb` in `ubuntu:24.04` and the `.rpm` in
`fedora:40` before the release job runs.

## Local Host Tarball Dry Run

For a local host tarball smoke:

```sh
cargo build --release -p garnet-cli
mkdir -p /tmp/garnet-v0.4.2-release-assets
cp target/release/garnet /tmp/garnet-v0.4.2-release-assets/garnet
(cd /tmp/garnet-v0.4.2-release-assets && tar -czf garnet-0.4.2-$(rustc -vV | awk '/host:/ {print $2}').tar.gz garnet)
(cd /tmp/garnet-v0.4.2-release-assets && shasum -a 256 *.tar.gz > SHA256SUMS)
```

The universal installer can consume tarballs when the user sets
`GARNET_FORMAT=tar`, or when a future installer fallback tries tar after a
missing platform-native package.

Dogfood result from this remediation run:

```text
/tmp/garnet-v0.4.2-release-assets/garnet-0.4.2-aarch64-apple-darwin.tar.gz
/tmp/garnet-v0.4.2-release-assets/SHA256SUMS
```

The local file-backed release install smoke passed with:

```sh
GARNET_PREFIX=/tmp/garnet-release-tar-install-smoke \
GARNET_INSTALL_MODE=release \
GARNET_FORMAT=tar \
GARNET_BASE_URL=file:///tmp/garnet-v0.4.2-release-assets \
GARNET_CHECKSUM_URL=file:///tmp/garnet-v0.4.2-release-assets/SHA256SUMS \
sh installer/sh.garnet-lang.org/install.sh
```

## Fast Org Release Smoke

From a checked-out branch where PRs are merged and the org tag is expected to exist, run:

```sh
./scripts/verify_org_release_smoke.sh
```

This verifies the tag first and then installs with:

```sh
GARNET_INSTALL_MODE=release GARNET_VERSION=v0.4.2 sh installer/sh.garnet-lang.org/install.sh
```

If either check fails, the script exits non-zero and marks the smoke as not release-backed.

## Official Publication Steps

Run only after the remediation branch is merged to `main` and the maintainer
identity has write permission:

```sh
git fetch --prune origin
git checkout main
git pull --ff-only origin main
cargo fmt --all -- --check
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
git tag -a v0.4.2 -m "Release Garnet v0.4.2"
git push origin v0.4.2
gh run watch --repo Island-Dev-Crew/garnet
gh release view v0.4.2 --repo Island-Dev-Crew/garnet
```

After the workflow succeeds, test the public release-only path:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://garnet-lang.org/install.sh \
  | GARNET_INSTALL_MODE=release sh
```

## Org-Blocked Continuation Path (what to do while waiting for org write access)

If this machine/account only has read permission on `Island-Dev-Crew/garnet`,
you can still complete the release-readiness lane preparation:

1. Confirm the fork has the expected tag and assets:

   ```sh
   gh api repos/Navigata1/garnet/releases --jq '.[0].tag_name, .[0].assets[].name'
   ```

2. Confirm the org release is absent (expected for this lane):

   ```sh
   gh api repos/Island-Dev-Crew/garnet/releases/tags/v0.4.2 --jq '.tag_name' || true
   ```

3. When an org maintainer signs in, use this exact sequence:

   ```sh
   gh release create v0.4.2 \
     --repo Island-Dev-Crew/garnet \
     --target main \
     --title "Release Garnet v0.4.2" \
     --notes "Promotes v0.4.2 readiness artifacts for org release consumption." \
     https://github.com/Navigata1/garnet/releases/download/v0.4.2/garnet-0.4.2-1.x86_64.rpm \
     https://github.com/Navigata1/garnet/releases/download/v0.4.2/garnet_0.4.2-1_amd64.deb \
     https://github.com/Navigata1/garnet/releases/download/v0.4.2/SHA256SUMS \
     https://github.com/Navigata1/garnet/releases/download/v0.4.2/garnet-0.4.2-aarch64-apple-darwin.tar.gz
   ```

4. Re-run the existing smoke:

   ```sh
   ./scripts/verify_org_release_smoke.sh
   ```

   The success condition is `garnet 0.4.2` after `GARNET_INSTALL_MODE=release`.

## Credential-Gated Assets

The following are still credential or platform gated:

- macOS `.pkg`: requires Apple Developer ID Installer certificate,
  notarization profile, `spctl`, and `stapler` validation.
- Windows `.msi`: requires Authenticode signing and timestamping.
- Optional signed `SHA256SUMS.asc`: requires a release GPG key secret.

Do not claim a frictionless native package release until these assets exist and
the public installer verifies them.
