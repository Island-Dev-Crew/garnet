#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Garnet Studio"
EXECUTABLE_NAME="GarnetStudio"
VERSION="${GARNET_VERSION:-0.4.2}"
APP_DIR="${ROOT}/target/macos/${APP_NAME}.app"
DMG_PATH="${ROOT}/target/macos/GarnetStudio.dmg"
PACKAGE_PATH="${ROOT}/apps/garnet-studio-macos"
SWIFT_BIN="${PACKAGE_PATH}/.build/release/${EXECUTABLE_NAME}"
GARNET_BIN="${ROOT}/target/release/garnet"

echo "==> Building Garnet CLI"
cargo build --release -p garnet-cli

echo "==> Building Garnet Studio SwiftUI executable"
swift build --package-path "${PACKAGE_PATH}" -c release

echo "==> Assembling ${APP_DIR}"
rm -rf "${APP_DIR}" "${DMG_PATH}"
mkdir -p "${APP_DIR}/Contents/MacOS" "${APP_DIR}/Contents/Resources"

cp "${SWIFT_BIN}" "${APP_DIR}/Contents/MacOS/${EXECUTABLE_NAME}"
cp "${GARNET_BIN}" "${APP_DIR}/Contents/Resources/garnet"
cp "${ROOT}/assets/garnet-logo.png" "${APP_DIR}/Contents/Resources/garnet-logo.png"
chmod 0755 "${APP_DIR}/Contents/MacOS/${EXECUTABLE_NAME}"
chmod 0755 "${APP_DIR}/Contents/Resources/garnet"

cat > "${APP_DIR}/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>${EXECUTABLE_NAME}</string>
  <key>CFBundleIdentifier</key>
  <string>org.garnet-lang.studio</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>${APP_NAME}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>LSMinimumSystemVersion</key>
  <string>14.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

printf 'APPL????' > "${APP_DIR}/Contents/PkgInfo"

if command -v codesign >/dev/null 2>&1; then
  echo "==> Applying ad-hoc signature"
  codesign --force --deep --sign - "${APP_DIR}" >/dev/null
else
  echo "warning: codesign unavailable; app bundle left unsigned" >&2
fi

echo "==> Running packaged app self-test"
"${APP_DIR}/Contents/MacOS/${EXECUTABLE_NAME}" --self-test

echo "==> Running bundled CLI smoke"
"${APP_DIR}/Contents/Resources/garnet" --version

echo "==> Running packaged app CLI smoke"
"${APP_DIR}/Contents/MacOS/${EXECUTABLE_NAME}" --smoke-test

if command -v hdiutil >/dev/null 2>&1; then
  echo "==> Creating DMG"
  hdiutil create \
    -volname "${APP_NAME}" \
    -srcfolder "${APP_DIR}" \
    -ov \
    -format UDZO \
    "${DMG_PATH}" >/dev/null
  hdiutil verify "${DMG_PATH}" >/dev/null
  shasum -a 256 "${DMG_PATH}"
else
  echo "warning: hdiutil unavailable; DMG not created" >&2
fi

echo "==> Done"
echo "    ${APP_DIR}"
if [ -f "${DMG_PATH}" ]; then
  echo "    ${DMG_PATH}"
fi
