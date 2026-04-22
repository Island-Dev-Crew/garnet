#!/bin/bash
# Garnet macOS .pkg builder — v4.2 Phase 6A.
#
# Produces a universal installer that:
#   * Ships a universal binary (x86_64 + arm64 merged with `lipo`)
#   * Installs to /usr/local/garnet/bin/garnet
#   * Symlinks /usr/local/bin/garnet → /usr/local/garnet/bin/garnet
#   * Installs the man page to /usr/local/share/man/man1/garnet.1
#   * Is signed with the user's "Developer ID Installer" certificate
#   * Is notarized with Apple's notary service
#   * Has a branded welcome screen (background image = Garnet logo)
#
# Environment prerequisites:
#   APPLE_DEV_ID_INSTALLER — the identity name of the installer cert
#     e.g. "Developer ID Installer: Jon Doe (ABCDEF1234)"
#   APPLE_DEV_ID_APP       — identity for the binary itself (codesign)
#   APPLE_NOTARY_PROFILE   — name of the keychain profile previously
#     created via `xcrun notarytool store-credentials`
#
# Not checked in: the user's signing identities live in the Keychain.
# This script refuses to run if any of the env vars above are unset.

set -euo pipefail

require_env() {
    local var="$1"
    if [[ -z "${!var:-}" ]]; then
        echo "error: environment variable $var is required" >&2
        echo "       see macos/build-pkg.sh header for the full set" >&2
        exit 1
    fi
}

require_env APPLE_DEV_ID_INSTALLER
require_env APPLE_DEV_ID_APP
require_env APPLE_NOTARY_PROFILE

VERSION="${GARNET_VERSION:-0.4.2}"
STAGING="$(mktemp -d -t garnet-pkg)"
SCRATCH="$(mktemp -d -t garnet-pkg-scratch)"
trap 'rm -rf "$STAGING" "$SCRATCH"' EXIT

echo "==> Building universal binary (x86_64 + arm64)"
cargo build --release --target x86_64-apple-darwin  -p garnet-cli
cargo build --release --target aarch64-apple-darwin -p garnet-cli

mkdir -p "$SCRATCH/garnet-root/usr/local/garnet/bin"
mkdir -p "$SCRATCH/garnet-root/usr/local/share/man/man1"
mkdir -p "$SCRATCH/garnet-root/usr/local/bin"

echo "==> Merging binaries with lipo"
lipo -create \
    target/x86_64-apple-darwin/release/garnet \
    target/aarch64-apple-darwin/release/garnet \
    -output "$SCRATCH/garnet-root/usr/local/garnet/bin/garnet"

chmod 0755 "$SCRATCH/garnet-root/usr/local/garnet/bin/garnet"

# /usr/local/bin symlink so `garnet` is discoverable without PATH edits.
ln -sf /usr/local/garnet/bin/garnet "$SCRATCH/garnet-root/usr/local/bin/garnet"

cp man/garnet.1 "$SCRATCH/garnet-root/usr/local/share/man/man1/"

echo "==> Signing binary"
codesign --force --options runtime --timestamp \
    --sign "$APPLE_DEV_ID_APP" \
    "$SCRATCH/garnet-root/usr/local/garnet/bin/garnet"

echo "==> Building component .pkg"
pkgbuild \
    --root "$SCRATCH/garnet-root" \
    --identifier org.garnet-lang.garnet \
    --version "$VERSION" \
    --install-location / \
    "$SCRATCH/garnet-component.pkg"

echo "==> Building distribution .pkg with branded UI"
productbuild \
    --distribution macos/distribution.xml \
    --resources macos/resources \
    --package-path "$SCRATCH" \
    "$SCRATCH/garnet-unsigned.pkg"

echo "==> Signing .pkg"
productsign \
    --sign "$APPLE_DEV_ID_INSTALLER" \
    --timestamp \
    "$SCRATCH/garnet-unsigned.pkg" \
    "$STAGING/garnet-$VERSION-universal.pkg"

echo "==> Notarizing (may take several minutes)"
xcrun notarytool submit \
    --keychain-profile "$APPLE_NOTARY_PROFILE" \
    --wait \
    "$STAGING/garnet-$VERSION-universal.pkg"

echo "==> Stapling notarization ticket"
xcrun stapler staple "$STAGING/garnet-$VERSION-universal.pkg"

OUT="target/macos"
mkdir -p "$OUT"
cp "$STAGING/garnet-$VERSION-universal.pkg" "$OUT/"

echo "==> Done"
echo "    $OUT/garnet-$VERSION-universal.pkg"
shasum -a 256 "$OUT/garnet-$VERSION-universal.pkg"
