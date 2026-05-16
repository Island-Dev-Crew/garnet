# Garnet Apple Distribution Walkthrough

Status: operator walkthrough, not completed enrollment evidence.
Date: 2026-05-16.
Official references checked:

- Apple Developer Program enrollment: <https://developer.apple.com/programs/enroll/>
- Apple membership comparison and fee: <https://developer.apple.com/support/compare-memberships/>
- Developer ID signing: <https://developer.apple.com/developer-id/>
- Notarizing macOS software: <https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution>
- Developer ID certificate help: <https://developer.apple.com/help/account/create-certificates/create-developer-id-certificates/>
- Google Play Console setup note: <https://support.google.com/googleplay/android-developer/answer/6112435>

## What Must Be Done By The Account Holder

Apple enrollment, legal agreements, payment, D-U-N-S selection, identity verification, and certificate creation require the human account holder. An agent can prepare scripts and verify local artifacts, but should not accept legal agreements or enter payment credentials.

## Enrollment Choice

Choose the membership type before starting:

| Path | App Store seller name | Best fit |
| --- | --- | --- |
| Individual | Personal legal name | Solo release where personal seller name is acceptable. |
| Organization | Legal organization name | Island Development Crew or another legal entity should be displayed as seller. Requires a D-U-N-S Number tied to the legal entity. |

Apple lists Apple Developer Program membership at 99 USD per membership year, or local equivalent where available. Some nonprofit, educational, or government entities may qualify for a fee waiver.

## Enrollment Steps

1. Sign in with the Apple Account that will own the developer membership.
2. Confirm two-factor authentication is enabled.
3. Open <https://developer.apple.com/programs/enroll/>.
4. Start enrollment.
5. Select Individual or Organization.
6. For Individual, verify legal name, email, phone, and address.
7. For Organization, gather legal entity name, D-U-N-S Number, authority to bind the entity, public website, phone, and address.
8. Complete identity verification if Apple asks for it.
9. Accept the Apple Developer Program License Agreement.
10. Complete membership purchase.
11. Wait for Apple to activate the team.

## Certificates After Enrollment

Create these from Xcode or Apple Developer Certificates, Identifiers & Profiles:

- Developer ID Application: signs direct-distribution `.app` bundles and app binaries outside the Mac App Store.
- Developer ID Installer: signs `.pkg` installers outside the Mac App Store.
- Mac App Distribution: needed for Mac App Store distribution.
- Mac Installer Distribution: needed for Mac App Store package upload flows when applicable.

Garnet Studio direct DMG distribution needs Developer ID Application first. A future signed `.pkg` needs Developer ID Installer as well.

## Local Keychain Setup

After certificate creation, import certificates into the login keychain and verify:

```sh
security find-identity -p codesigning -v
```

Expected identities:

```text
Developer ID Application: <Name> (<TEAMID>)
Developer ID Installer: <Name> (<TEAMID>)
```

Do not paste private keys, app-specific passwords, or keychain credentials into repo files.

## Notary Profile Setup

Create an app-specific password or App Store Connect API key according to the account policy, then store a notarytool profile locally:

```sh
xcrun notarytool store-credentials "garnet-notary" \
  --apple-id "<APPLE_ID_EMAIL>" \
  --team-id "<TEAM_ID>" \
  --password "<APP_SPECIFIC_PASSWORD>"
```

Then export only the profile name for packaging:

```sh
export APPLE_NOTARY_PROFILE="garnet-notary"
export APPLE_DEV_ID_APP="Developer ID Application: <Name> (<TEAMID>)"
export APPLE_DEV_ID_INSTALLER="Developer ID Installer: <Name> (<TEAMID>)"
```

## Garnet Verification Commands

Run preflight before claiming notarization readiness:

```sh
./scripts/preflight_garnet_studio_notarization.sh --copy-to-desktop
python3 scripts/garnet_studio_notarization_status.py --bundle ~/Desktop/dogfood/<preflight-bundle>
```

Package and smoke the app:

```sh
./scripts/package_garnet_studio_macos.sh
scripts/smoke_garnet_studio_dmg.sh --copy-to-desktop target/macos/GarnetStudio.dmg
```

After signing is wired, the release path must additionally prove:

```sh
codesign --verify --deep --strict --verbose=2 "dist/Garnet Studio.app"
spctl --assess --type execute --verbose "dist/Garnet Studio.app"
xcrun notarytool submit target/macos/GarnetStudio.dmg --keychain-profile "$APPLE_NOTARY_PROFILE" --wait
xcrun stapler staple target/macos/GarnetStudio.dmg
xcrun stapler validate target/macos/GarnetStudio.dmg
spctl --assess --type open --verbose target/macos/GarnetStudio.dmg
```

## App Store Path

Direct Developer ID distribution and App Store distribution are separate:

- Direct distribution: Developer ID Application + notarization + stapled DMG or PKG.
- Mac App Store: App Store Connect app record, bundle ID, provisioning/profile setup, sandbox and entitlement review, archive upload, App Review.
- iOS/iPadOS App Store: requires an iOS app target. The current SwiftUI macOS Studio is not automatically an iOS app.

Do direct notarized macOS distribution first. Treat iOS/Android as a separate mobile product lane after the cross-platform Studio MVP is stable.

## Android / Google Play Note

For Android distribution, the official Google Play Console path currently lists a 25 USD one-time registration fee and developer verification requirements. That is a separate account and packaging lane from Apple Developer Program.

## Current Garnet Blockers

- No confirmed Developer ID Application identity in the local keychain yet.
- No confirmed Developer ID Installer identity in the local keychain yet.
- No confirmed notarytool keychain profile yet.
- Clean-machine Gatekeeper evidence has not been captured yet.
- App Store and mobile distribution are future lanes.
