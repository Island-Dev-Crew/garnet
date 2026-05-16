#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_PATH="${ROOT}/target/macos/Garnet Studio.app"
DMG_PATH="${ROOT}/target/macos/GarnetStudio.dmg"
COPY_TO_DESKTOP=0
STRICT=0
STAMP="$(date +%Y%m%d-%H%M%S)"
OUTPUT_DIR="${ROOT}/target/macos/garnet-studio-notarization-preflight-${STAMP}"

usage() {
  cat <<'USAGE'
Usage: preflight_garnet_studio_notarization.sh [options]

Inspect Garnet Studio macOS signing/notarization readiness without submitting
anything to Apple.

Options:
  --app PATH             App bundle to inspect (default: target/macos/Garnet Studio.app)
  --dmg PATH             DMG to inspect (default: target/macos/GarnetStudio.dmg)
  --output-dir PATH      Evidence directory to write
  --copy-to-desktop      Copy evidence directory into ~/Desktop/dogfood
  --strict               Exit nonzero when distribution blockers are present
  -h, --help             Show this help
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --app)
      APP_PATH="$2"
      shift 2
      ;;
    --dmg)
      DMG_PATH="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --copy-to-desktop)
      COPY_TO_DESKTOP=1
      shift
      ;;
    --strict)
      STRICT=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

mkdir -p "${OUTPUT_DIR}/logs"
REPORT="${OUTPUT_DIR}/notarization-preflight-report.md"
DATA="${OUTPUT_DIR}/notarization-preflight-data.env"
COMMAND_LOG="${OUTPUT_DIR}/logs/commands.log"
CHECKS="${OUTPUT_DIR}/checks.tsv"
BLOCKERS=0
WARNINGS=0

run_capture() {
  local name="$1"
  local status
  shift
  {
    printf '$'
    printf ' %q' "$@"
    printf '\n'
    set +e
    "$@"
    status="$?"
    set -e
    printf 'exit=%s\n' "${status}"
  } >>"${COMMAND_LOG}" 2>&1
}

record() {
  local status="$1"
  local label="$2"
  local evidence="$3"
  local recommendation="$4"

  printf '%s\t%s\t%s\t%s\n' "${status}" "${label}" "${evidence}" "${recommendation}" >>"${CHECKS}"
  case "${status}" in
    blocker) BLOCKERS=$((BLOCKERS + 1)) ;;
    warning) WARNINGS=$((WARNINGS + 1)) ;;
  esac
}

command_exists() {
  command -v "$1" >/dev/null 2>&1
}

app_executable() {
  /usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "${APP_PATH}/Contents/Info.plist" 2>/dev/null || true
}

codesign_details() {
  codesign -dv --verbose=4 "$1" 2>&1 || true
}

echo "# Garnet Studio Notarization Preflight" >"${REPORT}"
cat >"${DATA}" <<DATA
app_path=${APP_PATH}
dmg_path=${DMG_PATH}
output_dir=${OUTPUT_DIR}
DATA

record "pass" "Evidence directory created" "${OUTPUT_DIR}" "Preserve this directory with the PR or Desktop dogfood bundle."

if [ -d "${APP_PATH}" ]; then
  record "pass" "App bundle exists" "${APP_PATH}" "None."
else
  record "blocker" "App bundle missing" "${APP_PATH}" "Run ./scripts/package_garnet_studio_macos.sh first."
fi

if [ -f "${DMG_PATH}" ]; then
  record "pass" "DMG exists" "${DMG_PATH}" "None."
else
  record "blocker" "DMG missing" "${DMG_PATH}" "Run ./scripts/package_garnet_studio_macos.sh first."
fi

EXECUTABLE_NAME="$(app_executable)"
if [ -n "${EXECUTABLE_NAME}" ] && [ -x "${APP_PATH}/Contents/MacOS/${EXECUTABLE_NAME}" ]; then
  record "pass" "App executable exists" "${APP_PATH}/Contents/MacOS/${EXECUTABLE_NAME}" "None."
else
  record "blocker" "App executable missing" "${APP_PATH}/Contents/MacOS/${EXECUTABLE_NAME:-<unknown>}" "Fix app bundle assembly before signing."
fi

if command_exists codesign; then
  run_capture "codesign-verify-app" codesign --verify --deep --strict "${APP_PATH}"
  if codesign --verify --deep --strict "${APP_PATH}" >/dev/null 2>&1; then
    record "pass" "App code signature verifies" "codesign --verify --deep --strict" "None for local integrity."
  else
    record "blocker" "App code signature does not verify" "codesign --verify --deep --strict" "Re-sign the app and nested code before distribution."
  fi

  codesign_details "${APP_PATH}" >"${OUTPUT_DIR}/logs/app-codesign-details.log"
  if grep -q '^Signature=adhoc$' "${OUTPUT_DIR}/logs/app-codesign-details.log"; then
    record "blocker" "Developer ID Application signature missing" "Signature=adhoc" "Sign with APPLE_DEV_ID_APP and hardened runtime before notarization."
  else
    record "pass" "Non-ad-hoc app signature present" "Signature is not adhoc" "Verify it is the intended Developer ID Application identity."
  fi

  if grep -q 'TeamIdentifier=not set' "${OUTPUT_DIR}/logs/app-codesign-details.log"; then
    record "blocker" "TeamIdentifier missing" "TeamIdentifier=not set" "Use a Developer ID Application certificate from the Apple Developer team."
  else
    record "pass" "TeamIdentifier present" "TeamIdentifier is set" "None."
  fi

  if grep -Eq 'flags=.*runtime' "${OUTPUT_DIR}/logs/app-codesign-details.log"; then
    record "pass" "Hardened runtime present" "codesign flags include runtime" "None."
  else
    record "blocker" "Hardened runtime missing" "codesign flags do not include runtime" "Sign with codesign --options runtime before notarization."
  fi
else
  record "blocker" "codesign unavailable" "command not found" "Install Xcode Command Line Tools."
fi

if command_exists spctl; then
  run_capture "spctl-assess-app" spctl -a -vv "${APP_PATH}"
  if spctl -a -vv "${APP_PATH}" >/dev/null 2>&1; then
    record "pass" "Gatekeeper assessment accepts app" "spctl -a -vv" "None."
  else
    record "warning" "Gatekeeper assessment rejects app" "spctl -a -vv rejected" "Expected for ad-hoc/non-notarized local builds; must pass after Developer ID signing and notarization."
  fi
else
  record "warning" "spctl unavailable" "command not found" "Run Gatekeeper assessment on macOS with Xcode tools."
fi

if command_exists xcrun && xcrun notarytool --help >/dev/null 2>&1; then
  record "pass" "notarytool available" "xcrun notarytool --help" "None."
else
  record "blocker" "notarytool unavailable" "xcrun notarytool --help failed" "Install/update Xcode Command Line Tools."
fi

if command_exists xcrun && (xcrun stapler validate 2>&1 || true) | grep -q 'Usage: stapler'; then
  record "pass" "stapler available" "xcrun stapler validate" "None."
else
  record "blocker" "stapler unavailable" "xcrun stapler validate failed before path validation" "Install/update Xcode Command Line Tools."
fi

if command_exists security; then
  run_capture "security-find-identity" security find-identity -p codesigning -v
  IDENTITY_COUNT="$(security find-identity -p codesigning -v 2>/dev/null | grep -c 'Developer ID Application' || true)"
  printf 'developer_id_application_identity_count=%s\n' "${IDENTITY_COUNT}" >>"${DATA}"
  if [ "${IDENTITY_COUNT}" -gt 0 ]; then
    record "pass" "Developer ID Application identity found" "count=${IDENTITY_COUNT}" "Set APPLE_DEV_ID_APP to the intended identity before signing."
  else
    record "blocker" "Developer ID Application identity missing" "count=0" "Install a Developer ID Application certificate in the keychain."
  fi
else
  record "blocker" "security tool unavailable" "command not found" "Run on macOS with standard security tooling."
fi

if [ -n "${APPLE_DEV_ID_APP:-}" ]; then
  record "pass" "APPLE_DEV_ID_APP configured" "environment variable is set" "Value intentionally not written to report."
else
  record "blocker" "APPLE_DEV_ID_APP not configured" "environment variable is empty" "Export the Developer ID Application identity before distribution signing."
fi

if [ -n "${APPLE_NOTARY_PROFILE:-}" ]; then
  record "pass" "APPLE_NOTARY_PROFILE configured" "environment variable is set" "Value intentionally not written to report."
else
  record "blocker" "APPLE_NOTARY_PROFILE not configured" "environment variable is empty" "Create a notarytool keychain profile and export its name."
fi

if command_exists hdiutil && [ -f "${DMG_PATH}" ]; then
  run_capture "hdiutil-verify" hdiutil verify "${DMG_PATH}"
  if hdiutil verify "${DMG_PATH}" >/dev/null 2>&1; then
    record "pass" "DMG checksum verifies" "hdiutil verify" "None."
  else
    record "blocker" "DMG checksum verification failed" "hdiutil verify" "Rebuild the DMG."
  fi
else
  record "warning" "DMG verification not run" "hdiutil unavailable or DMG missing" "Run on macOS after package build."
fi

if command_exists xcrun && [ -f "${DMG_PATH}" ]; then
  run_capture "stapler-validate-dmg" xcrun stapler validate "${DMG_PATH}"
  if xcrun stapler validate "${DMG_PATH}" >/dev/null 2>&1; then
    record "pass" "DMG has stapled notarization ticket" "xcrun stapler validate" "None."
  else
    record "warning" "DMG has no stapled notarization ticket" "xcrun stapler validate failed" "Expected before notarization; must pass after notarytool submit and stapler staple."
  fi
fi

{
  echo
  echo "## Decision"
  echo
  if [ "${BLOCKERS}" -eq 0 ]; then
    echo "No distribution blockers were detected by this preflight."
  else
    echo "Not notarization-ready: ${BLOCKERS} blocker(s) and ${WARNINGS} warning(s) detected."
  fi
  echo
  echo "## Checks"
  echo
  echo "| Status | Check | Evidence | Recommendation |"
  echo "| --- | --- | --- | --- |"
  while IFS=$'\t' read -r status label evidence recommendation; do
    printf '| %s | %s | `%s` | %s |\n' "${status}" "${label}" "${evidence}" "${recommendation}"
  done <"${CHECKS}"
  echo
  echo "## Boundary"
  echo
  echo "This preflight does not submit to Apple and does not claim notarization."
  echo "It records whether the local app/DMG has the signing, hardened-runtime,"
  echo "notarytool, stapler, and credential prerequisites needed for the next"
  echo "distribution gate."
} >>"${REPORT}"

cat >>"${DATA}" <<DATA
blockers=${BLOCKERS}
warnings=${WARNINGS}
strict=${STRICT}
copy_to_desktop=${COPY_TO_DESKTOP}
DATA

(
  cd "${OUTPUT_DIR}"
  find . -type f ! -name MANIFEST.sha256 ! -name MANIFEST.verify.log -print0 \
    | sort -z \
    | xargs -0 shasum -a 256 >MANIFEST.sha256
  shasum -a 256 -c MANIFEST.sha256 >MANIFEST.verify.log
)

echo "artifact_dir=${OUTPUT_DIR}"
echo "blockers=${BLOCKERS}"
echo "warnings=${WARNINGS}"

if [ "${COPY_TO_DESKTOP}" -eq 1 ]; then
  DESKTOP_DIR="${HOME}/Desktop/dogfood/$(basename "${OUTPUT_DIR}")"
  rm -rf "${DESKTOP_DIR}"
  mkdir -p "$(dirname "${DESKTOP_DIR}")"
  cp -R "${OUTPUT_DIR}" "${DESKTOP_DIR}"
  echo "desktop_copy=${DESKTOP_DIR}"
fi

if [ "${STRICT}" -eq 1 ] && [ "${BLOCKERS}" -gt 0 ]; then
  exit 1
fi
