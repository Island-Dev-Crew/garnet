#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Garnet Studio"
EXECUTABLE_NAME="GarnetStudio"
DMG_PATH="${1:-${ROOT}/target/macos/GarnetStudio.dmg}"
RUN_AGENTIC_MATRIX="${GARNET_STUDIO_DMG_SMOKE_AGENTIC:-1}"

if ! command -v hdiutil >/dev/null 2>&1; then
  echo "error: hdiutil is required for DMG install smoke" >&2
  exit 2
fi

if [ ! -f "${DMG_PATH}" ]; then
  echo "error: DMG not found: ${DMG_PATH}" >&2
  exit 3
fi

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/garnet-studio-dmg-smoke.XXXXXX")"
MOUNT_DIR="${TMP_ROOT}/mount"
INSTALL_DIR="${TMP_ROOT}/Applications"
ATTACHED=0
mkdir -p "${MOUNT_DIR}" "${INSTALL_DIR}"

cleanup() {
  if [ "${ATTACHED}" -eq 1 ]; then
    hdiutil detach "${MOUNT_DIR}" -quiet || true
  fi
  rm -rf "${TMP_ROOT}"
}
trap cleanup EXIT

find_app_bundle() {
  local search_root="$1"
  local found

  if [ -x "${search_root}/Contents/MacOS/${EXECUTABLE_NAME}" ]; then
    printf '%s\n' "${search_root}"
    return 0
  fi

  if [ -d "${search_root}/${APP_NAME}.app" ]; then
    printf '%s\n' "${search_root}/${APP_NAME}.app"
    return 0
  fi

  found="$(find "${search_root}" -maxdepth 2 -type d -name "${APP_NAME}.app" -print -quit)"
  if [ -n "${found}" ]; then
    printf '%s\n' "${found}"
    return 0
  fi

  return 1
}

echo "==> Mounting ${DMG_PATH}"
hdiutil attach -readonly -nobrowse -mountpoint "${MOUNT_DIR}" "${DMG_PATH}" >/dev/null
ATTACHED=1

SOURCE_APP="$(find_app_bundle "${MOUNT_DIR}")"
INSTALLED_APP="${INSTALL_DIR}/${APP_NAME}.app"

echo "==> Copying ${APP_NAME}.app from mounted DMG"
/usr/bin/ditto "${SOURCE_APP}" "${INSTALLED_APP}"

APP_EXECUTABLE="${INSTALLED_APP}/Contents/MacOS/${EXECUTABLE_NAME}"
GARNET_BIN="${INSTALLED_APP}/Contents/Resources/garnet"
MATRIX_SCRIPT="${INSTALLED_APP}/Contents/Resources/scripts/run_agentic_dogfood_matrix.py"
EXAMPLES_DIR="${INSTALLED_APP}/Contents/Resources/examples"

for required in "${APP_EXECUTABLE}" "${GARNET_BIN}" "${MATRIX_SCRIPT}"; do
  if [ ! -x "${required}" ]; then
    echo "error: copied app is missing executable asset: ${required}" >&2
    exit 4
  fi
done

if [ ! -d "${EXAMPLES_DIR}" ]; then
  echo "error: copied app is missing bundled examples: ${EXAMPLES_DIR}" >&2
  exit 5
fi

if command -v codesign >/dev/null 2>&1; then
  echo "==> Verifying copied app signature"
  codesign --verify --deep --strict "${INSTALLED_APP}"
fi

echo "==> Running copied app self-test"
"${APP_EXECUTABLE}" --self-test

echo "==> Running copied bundled CLI version smoke"
"${GARNET_BIN}" --version

echo "==> Running copied app workbench smoke"
"${APP_EXECUTABLE}" --smoke-test

if [ "${RUN_AGENTIC_MATRIX}" != "0" ]; then
  echo "==> Running copied app agentic matrix smoke"
  "${APP_EXECUTABLE}" --agentic-matrix-test
else
  echo "==> Skipping copied app agentic matrix smoke via GARNET_STUDIO_DMG_SMOKE_AGENTIC=0"
fi

echo "==> DMG install smoke passed"
