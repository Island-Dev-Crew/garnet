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
APPLE_DEV_ID_APP="${APPLE_DEV_ID_APP:-}"

echo "==> Building Garnet CLI"
cargo build --release -p garnet-cli

echo "==> Building Garnet Studio SwiftUI executable"
swift build --package-path "${PACKAGE_PATH}" -c release

echo "==> Assembling ${APP_DIR}"
rm -rf "${APP_DIR}" "${DMG_PATH}"
mkdir -p "${APP_DIR}/Contents/MacOS" "${APP_DIR}/Contents/Resources/scripts"
mkdir -p "${APP_DIR}/Contents/Resources/assets"
mkdir -p "${APP_DIR}/Contents/Resources/apps/garnet-studio-macos/Sources/GarnetStudio"
mkdir -p "${APP_DIR}/Contents/Resources/apps/garnet-studio-macos/Sources/GarnetStudio/Resources"
mkdir -p "${APP_DIR}/Contents/Resources/F_Project_Management/DOGFOOD"

cp "${SWIFT_BIN}" "${APP_DIR}/Contents/MacOS/${EXECUTABLE_NAME}"
cp "${GARNET_BIN}" "${APP_DIR}/Contents/Resources/garnet"
cp "${ROOT}/assets/garnet-logo.png" "${APP_DIR}/Contents/Resources/garnet-logo.png"
cp "${ROOT}/assets/garnet-logo.png" "${APP_DIR}/Contents/Resources/assets/garnet-logo.png"
cp "${ROOT}/apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift" "${APP_DIR}/Contents/Resources/apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift"
cp "${ROOT}/apps/garnet-studio-macos/Sources/GarnetStudio/Resources/garnet-logo.png" "${APP_DIR}/Contents/Resources/apps/garnet-studio-macos/Sources/GarnetStudio/Resources/garnet-logo.png"
cp "${ROOT}/scripts/run_agentic_dogfood_matrix.py" "${APP_DIR}/Contents/Resources/scripts/run_agentic_dogfood_matrix.py"
cp "${ROOT}/scripts/garnet_adoption_surface_status.py" "${APP_DIR}/Contents/Resources/scripts/garnet_adoption_surface_status.py"
cp "${ROOT}/scripts/garnet_assist_context_pack.py" "${APP_DIR}/Contents/Resources/scripts/garnet_assist_context_pack.py"
cp "${ROOT}/scripts/garnet_converter_advisory_bundle.py" "${APP_DIR}/Contents/Resources/scripts/garnet_converter_advisory_bundle.py"
cp "${ROOT}/scripts/garnet_converter_advisory_handoff.py" "${APP_DIR}/Contents/Resources/scripts/garnet_converter_advisory_handoff.py"
cp "${ROOT}/scripts/garnet_converter_advisory_review.py" "${APP_DIR}/Contents/Resources/scripts/garnet_converter_advisory_review.py"
cp "${ROOT}/scripts/garnet_converter_assist_plan.py" "${APP_DIR}/Contents/Resources/scripts/garnet_converter_assist_plan.py"
cp "${ROOT}/scripts/garnet_converter_llm_feasibility.py" "${APP_DIR}/Contents/Resources/scripts/garnet_converter_llm_feasibility.py"
cp "${ROOT}/scripts/garnet_converter_status.py" "${APP_DIR}/Contents/Resources/scripts/garnet_converter_status.py"
cp "${ROOT}/scripts/garnet_mac_side_continuation_status.py" "${APP_DIR}/Contents/Resources/scripts/garnet_mac_side_continuation_status.py"
cp "${ROOT}/scripts/garnet_mit_deck_outline.py" "${APP_DIR}/Contents/Resources/scripts/garnet_mit_deck_outline.py"
cp "${ROOT}/scripts/garnet_mit_demo_route.py" "${APP_DIR}/Contents/Resources/scripts/garnet_mit_demo_route.py"
cp "${ROOT}/scripts/garnet_mit_readiness_status.py" "${APP_DIR}/Contents/Resources/scripts/garnet_mit_readiness_status.py"
cp "${ROOT}/scripts/garnet_promo_video_status.py" "${APP_DIR}/Contents/Resources/scripts/garnet_promo_video_status.py"
cp "${ROOT}/scripts/garnet_readiness_status.py" "${APP_DIR}/Contents/Resources/scripts/garnet_readiness_status.py"
cp "${ROOT}/scripts/garnet_studio_notarization_status.py" "${APP_DIR}/Contents/Resources/scripts/garnet_studio_notarization_status.py"
cp "${ROOT}/scripts/export_garnet_promo_video_site.mjs" "${APP_DIR}/Contents/Resources/scripts/export_garnet_promo_video_site.mjs"
cp "${ROOT}/scripts/qa_garnet_promo_video.mjs" "${APP_DIR}/Contents/Resources/scripts/qa_garnet_promo_video.mjs"
cp "${ROOT}/scripts/render_garnet_promo_video.mjs" "${APP_DIR}/Contents/Resources/scripts/render_garnet_promo_video.mjs"
cp "${ROOT}/scripts/sync_garnet_promo_video_site.mjs" "${APP_DIR}/Contents/Resources/scripts/sync_garnet_promo_video_site.mjs"
cp "${ROOT}/scripts/smoke_garnet_web_pwa_offline.mjs" "${APP_DIR}/Contents/Resources/scripts/smoke_garnet_web_pwa_offline.mjs"
cp "${ROOT}/README.md" "${APP_DIR}/Contents/Resources/README.md"
cp "${ROOT}/CURRENT_STATE.md" "${APP_DIR}/Contents/Resources/CURRENT_STATE.md"
cp -R "${ROOT}/C_Language_Specification" "${APP_DIR}/Contents/Resources/C_Language_Specification"
cp "${ROOT}/F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md" "${APP_DIR}/Contents/Resources/F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md"
cp "${ROOT}/F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md" "${APP_DIR}/Contents/Resources/F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md"
cp -R "${ROOT}/docs" "${APP_DIR}/Contents/Resources/docs"
cp -R "${ROOT}/examples" "${APP_DIR}/Contents/Resources/examples"
chmod 0755 "${APP_DIR}/Contents/MacOS/${EXECUTABLE_NAME}"
chmod 0755 "${APP_DIR}/Contents/Resources/garnet"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/run_agentic_dogfood_matrix.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_adoption_surface_status.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_assist_context_pack.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_converter_advisory_bundle.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_converter_advisory_handoff.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_converter_advisory_review.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_converter_assist_plan.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_converter_llm_feasibility.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_converter_status.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_mac_side_continuation_status.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_mit_deck_outline.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_mit_demo_route.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_mit_readiness_status.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_promo_video_status.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_readiness_status.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/garnet_studio_notarization_status.py"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/export_garnet_promo_video_site.mjs"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/qa_garnet_promo_video.mjs"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/render_garnet_promo_video.mjs"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/sync_garnet_promo_video_site.mjs"
chmod 0755 "${APP_DIR}/Contents/Resources/scripts/smoke_garnet_web_pwa_offline.mjs"

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
  if [ -n "${APPLE_DEV_ID_APP}" ]; then
    echo "==> Applying Developer ID signature with hardened runtime"
    codesign --force --deep --options runtime --timestamp --sign "${APPLE_DEV_ID_APP}" "${APP_DIR}" >/dev/null
  else
    echo "==> Applying ad-hoc signature"
    codesign --force --deep --sign - "${APP_DIR}" >/dev/null
  fi
else
  echo "warning: codesign unavailable; app bundle left unsigned" >&2
fi

echo "==> Running packaged app self-test"
"${APP_DIR}/Contents/MacOS/${EXECUTABLE_NAME}" --self-test

echo "==> Running bundled CLI smoke"
"${APP_DIR}/Contents/Resources/garnet" --version

echo "==> Running packaged app CLI smoke"
"${APP_DIR}/Contents/MacOS/${EXECUTABLE_NAME}" --smoke-test

echo "==> Running packaged app agentic matrix smoke"
"${APP_DIR}/Contents/MacOS/${EXECUTABLE_NAME}" --agentic-matrix-test

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

  echo "==> Running DMG install smoke"
  "${ROOT}/scripts/smoke_garnet_studio_dmg.sh" --copy-to-desktop "${DMG_PATH}"
else
  echo "warning: hdiutil unavailable; DMG not created" >&2
fi

echo "==> Done"
echo "    ${APP_DIR}"
if [ -f "${DMG_PATH}" ]; then
  echo "    ${DMG_PATH}"
fi
