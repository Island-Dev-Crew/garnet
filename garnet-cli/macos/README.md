# macOS .pkg installer (v4.2 Phase 6A)

## Build

```sh
export APPLE_DEV_ID_INSTALLER="Developer ID Installer: <YOUR NAME> (<TEAMID>)"
export APPLE_DEV_ID_APP="Developer ID Application: <YOUR NAME> (<TEAMID>)"
export APPLE_NOTARY_PROFILE="<keychain-profile-name>"

cd garnet-cli
./macos/build-pkg.sh
# Output: target/macos/garnet-0.4.2-universal.pkg
```

## What build-pkg.sh does

1. Builds the CLI for both `x86_64-apple-darwin` and
   `aarch64-apple-darwin`.
2. Merges them with `lipo` into a universal binary.
3. Signs the binary with `codesign` using the user's Developer ID
   Application identity (runtime-hardened + timestamped).
4. Calls `pkgbuild` to produce a component package.
5. Calls `productbuild` with `distribution.xml` to apply Garnet
   branding (welcome/background/conclusion HTML pages + license).
6. Signs the `.pkg` with `productsign` using the user's Developer ID
   Installer identity.
7. Submits to `notarytool` and waits for approval.
8. Staples the notarization ticket with `xcrun stapler` so
   Gatekeeper accepts the package offline.

## Install

The user double-clicks the `.pkg`; macOS Installer.app shows the
Garnet branding + license; after accept, `garnet` is available at
both `/usr/local/garnet/bin/garnet` and `/usr/local/bin/garnet` (the
latter is a symlink for convenience).

## Assets to supply

The following branded assets must be placed in `macos/resources/`
before building:

| File                       | Purpose                                 |
|----------------------------|-----------------------------------------|
| `background.png`           | Welcome / install-screen background.     |
| `welcome.html`             | Welcome-screen copy + logo.              |
| `conclusion.html`          | Success message shown post-install.      |
| `LICENSE.txt`              | Plain-text license shown during accept.  |

The build script does NOT ship these — they're user-supplied branding.

## Verify (before distribution)

```sh
pkgutil --check-signature target/macos/garnet-0.4.2-universal.pkg
spctl --assess -vvv --type install target/macos/garnet-0.4.2-universal.pkg
xcrun stapler validate target/macos/garnet-0.4.2-universal.pkg
```

All three should report `valid` for a correctly signed + notarized +
stapled `.pkg`.
