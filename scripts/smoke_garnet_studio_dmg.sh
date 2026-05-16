#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Garnet Studio"
EXECUTABLE_NAME="GarnetStudio"
DMG_PATH="${ROOT}/target/macos/GarnetStudio.dmg"
RUN_AGENTIC_MATRIX="${GARNET_STUDIO_DMG_SMOKE_AGENTIC:-1}"
STAMP="$(date +%Y%m%d-%H%M%S)"
OUTPUT_DIR="${ROOT}/target/macos/garnet-studio-dmg-smoke-${STAMP}"
COPY_TO_DESKTOP=0

usage() {
  cat <<'USAGE'
Usage: smoke_garnet_studio_dmg.sh [options] [DMG_PATH]

Mount a Garnet Studio DMG, copy the app into a temporary Applications-style
directory, run packaged smoke checks, and write manifest-verified evidence.

Options:
  --output-dir PATH      Evidence directory to write
  --copy-to-desktop      Copy evidence directory into ~/Desktop/dogfood
  -h, --help             Show this help
USAGE
}

POSITIONAL_DMG_SET=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --copy-to-desktop)
      COPY_TO_DESKTOP=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    -*)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
    *)
      if [ "${POSITIONAL_DMG_SET}" -eq 1 ]; then
        echo "error: multiple DMG paths provided" >&2
        usage >&2
        exit 2
      fi
      DMG_PATH="$1"
      POSITIONAL_DMG_SET=1
      shift
      ;;
  esac
done

mkdir -p "${OUTPUT_DIR}/logs"
REPORT="${OUTPUT_DIR}/dmg-smoke-report.md"
DATA="${OUTPUT_DIR}/dmg-smoke-data.env"
CHECKS="${OUTPUT_DIR}/checks.tsv"
COMMAND_LOG="${OUTPUT_DIR}/logs/commands.log"

record() {
  local status="$1"
  local label="$2"
  local evidence="$3"
  local recommendation="$4"

  printf '%s\t%s\t%s\t%s\n' "${status}" "${label}" "${evidence}" "${recommendation}" >>"${CHECKS}"
}

run_logged() {
  local name="$1"
  local status
  shift

  {
    printf '## %s\n' "${name}"
    printf '$'
    printf ' %q' "$@"
    printf '\n'
  } >>"${COMMAND_LOG}"

  set +e
  "$@" >"${OUTPUT_DIR}/logs/${name}.stdout.log" 2>"${OUTPUT_DIR}/logs/${name}.stderr.log"
  status="$?"
  set -e

  cat "${OUTPUT_DIR}/logs/${name}.stdout.log"
  cat "${OUTPUT_DIR}/logs/${name}.stderr.log" >&2
  printf 'exit=%s\n' "${status}" >>"${COMMAND_LOG}"
  return "${status}"
}

if ! command -v hdiutil >/dev/null 2>&1; then
  echo "error: hdiutil is required for DMG install smoke" >&2
  exit 2
fi

if [ ! -f "${DMG_PATH}" ]; then
  echo "error: DMG not found: ${DMG_PATH}" >&2
  exit 3
fi

record "pass" "Evidence directory created" "${OUTPUT_DIR}" "Preserve with the PR or Desktop dogfood bundle."
record "pass" "DMG exists" "${DMG_PATH}" "None."

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
run_logged "hdiutil-attach" hdiutil attach -readonly -nobrowse -mountpoint "${MOUNT_DIR}" "${DMG_PATH}"
ATTACHED=1

SOURCE_APP="$(find_app_bundle "${MOUNT_DIR}")"
INSTALLED_APP="${INSTALL_DIR}/${APP_NAME}.app"
record "pass" "App bundle found on mounted DMG" "${SOURCE_APP}" "None."

echo "==> Copying ${APP_NAME}.app from mounted DMG"
run_logged "ditto-copy-app" /usr/bin/ditto "${SOURCE_APP}" "${INSTALLED_APP}"
record "pass" "App copied into temporary Applications directory" "${INSTALLED_APP}" "None."

APP_EXECUTABLE="${INSTALLED_APP}/Contents/MacOS/${EXECUTABLE_NAME}"
GARNET_BIN="${INSTALLED_APP}/Contents/Resources/garnet"
MATRIX_SCRIPT="${INSTALLED_APP}/Contents/Resources/scripts/run_agentic_dogfood_matrix.py"
OFFLINE_PWA_SMOKE="${INSTALLED_APP}/Contents/Resources/scripts/smoke_garnet_web_pwa_offline.mjs"
ADOPTION_STATUS="${INSTALLED_APP}/Contents/Resources/scripts/garnet_adoption_surface_status.py"
ASSIST_CONTEXT="${INSTALLED_APP}/Contents/Resources/scripts/garnet_assist_context_pack.py"
ASSIST_PLAN="${INSTALLED_APP}/Contents/Resources/scripts/garnet_converter_assist_plan.py"
LLM_FEASIBILITY="${INSTALLED_APP}/Contents/Resources/scripts/garnet_converter_llm_feasibility.py"
CONVERTER_STATUS="${INSTALLED_APP}/Contents/Resources/scripts/garnet_converter_status.py"
MIT_STATUS="${INSTALLED_APP}/Contents/Resources/scripts/garnet_mit_readiness_status.py"
PROMO_STATUS="${INSTALLED_APP}/Contents/Resources/scripts/garnet_promo_video_status.py"
PROMO_EXPORT="${INSTALLED_APP}/Contents/Resources/scripts/export_garnet_promo_video_site.mjs"
PROMO_QA="${INSTALLED_APP}/Contents/Resources/scripts/qa_garnet_promo_video.mjs"
PROMO_RENDER="${INSTALLED_APP}/Contents/Resources/scripts/render_garnet_promo_video.mjs"
PROMO_SYNC="${INSTALLED_APP}/Contents/Resources/scripts/sync_garnet_promo_video_site.mjs"
READINESS_STATUS="${INSTALLED_APP}/Contents/Resources/scripts/garnet_readiness_status.py"
NOTARIZATION_STATUS="${INSTALLED_APP}/Contents/Resources/scripts/garnet_studio_notarization_status.py"
PROMO_ASSETS_DIR="${INSTALLED_APP}/Contents/Resources/assets"
PROMO_STUDIO_SOURCE="${INSTALLED_APP}/Contents/Resources/apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift"
PROMO_STUDIO_LOGO="${INSTALLED_APP}/Contents/Resources/apps/garnet-studio-macos/Sources/GarnetStudio/Resources/garnet-logo.png"
EXAMPLES_DIR="${INSTALLED_APP}/Contents/Resources/examples"
DOCS_DIR="${INSTALLED_APP}/Contents/Resources/docs"
PROMO_DESIGN="${DOCS_DIR}/promo/DESIGN.md"
PROMO_COMPOSITION="${DOCS_DIR}/promo/composition.html"
SPEC_DIR="${INSTALLED_APP}/Contents/Resources/C_Language_Specification"
PROJECT_DOGFOOD_DIR="${INSTALLED_APP}/Contents/Resources/F_Project_Management/DOGFOOD"

for required in "${APP_EXECUTABLE}" "${GARNET_BIN}" "${MATRIX_SCRIPT}" "${OFFLINE_PWA_SMOKE}" "${ADOPTION_STATUS}" "${ASSIST_CONTEXT}" "${ASSIST_PLAN}" "${LLM_FEASIBILITY}" "${CONVERTER_STATUS}" "${MIT_STATUS}" "${PROMO_STATUS}" "${PROMO_EXPORT}" "${PROMO_QA}" "${PROMO_RENDER}" "${PROMO_SYNC}" "${READINESS_STATUS}" "${NOTARIZATION_STATUS}"; do
  if [ ! -x "${required}" ]; then
    echo "error: copied app is missing executable asset: ${required}" >&2
    exit 4
  fi
done
record "pass" "Copied app executable assets exist" "app executable, bundled CLI, matrix script, status reporters, converter LLM feasibility reporter, promo render, visual-QA, website-export, site-sync harnesses, PWA smoke" "None."

for required_dir in "${EXAMPLES_DIR}" "${DOCS_DIR}" "${SPEC_DIR}" "${PROJECT_DOGFOOD_DIR}" "${PROMO_ASSETS_DIR}"; do
  if [ ! -d "${required_dir}" ]; then
    echo "error: copied app is missing bundled resource directory: ${required_dir}" >&2
    exit 5
  fi
done
record "pass" "Copied app resource directories exist" "examples, docs, specs, dogfood context, and promo assets" "None."

if [ ! -f "${DOCS_DIR}/service-worker.js" ]; then
  echo "error: copied app is missing bundled PWA service worker: ${DOCS_DIR}/service-worker.js" >&2
  exit 5
fi
record "pass" "Copied app bundles PWA service worker" "${DOCS_DIR}/service-worker.js" "None."

for required_file in \
  "${INSTALLED_APP}/Contents/Resources/README.md" \
  "${INSTALLED_APP}/Contents/Resources/CURRENT_STATE.md" \
  "${SPEC_DIR}/GARNET_v1_0_Mini_Spec.md" \
  "${SPEC_DIR}/GARNET_v0_4_2_Conformance_Matrix.md" \
  "${INSTALLED_APP}/Contents/Resources/F_Project_Management/GARNET_LANGUAGE_COMPLETION_IMPLEMENTATION_PLAN.md" \
  "${PROJECT_DOGFOOD_DIR}/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md" \
  "${PROMO_ASSETS_DIR}/garnet-logo.png" \
  "${PROMO_STUDIO_SOURCE}" \
  "${PROMO_STUDIO_LOGO}" \
  "${PROMO_DESIGN}" \
  "${PROMO_COMPOSITION}"; do
  if [ ! -f "${required_file}" ]; then
    echo "error: copied app is missing bundled context file: ${required_file}" >&2
    exit 5
  fi
done
record "pass" "Copied app bundles status reporter context documents" "README, current state, spec, conformance, plan, dogfood ledger, promo source-lock inputs, and promo composition source" "None."

if command -v codesign >/dev/null 2>&1; then
  echo "==> Verifying copied app signature"
  run_logged "codesign-verify-copied-app" codesign --verify --deep --strict "${INSTALLED_APP}"
  record "pass" "Copied app signature verifies" "codesign --verify --deep --strict" "None for local integrity; Developer ID/notarization remains a separate preflight."
fi

echo "==> Running copied app self-test"
run_logged "copied-app-self-test" "${APP_EXECUTABLE}" --self-test
record "pass" "Copied app self-test passed" "${APP_EXECUTABLE} --self-test" "None."

echo "==> Running copied bundled CLI version smoke"
run_logged "copied-bundled-cli-version" "${GARNET_BIN}" --version
record "pass" "Copied bundled CLI version smoke passed" "${GARNET_BIN} --version" "None."

echo "==> Running copied app workbench smoke"
run_logged "copied-app-workbench-smoke" "${APP_EXECUTABLE}" --smoke-test
record "pass" "Copied app workbench smoke passed" "${APP_EXECUTABLE} --smoke-test" "None."

echo "==> Running copied bundled PWA offline handler smoke"
run_logged "copied-packaged-pwa-offline-handler" "${OFFLINE_PWA_SMOKE}" --docs-dir "${DOCS_DIR}" --output "${OUTPUT_DIR}/packaged-pwa-offline-handler.json"
record "pass" "Copied bundled PWA offline handler smoke passed" "${OUTPUT_DIR}/packaged-pwa-offline-handler.json" "None."

if [ "${RUN_AGENTIC_MATRIX}" != "0" ]; then
  echo "==> Running copied app agentic matrix smoke"
  run_logged "copied-app-agentic-matrix-smoke" "${APP_EXECUTABLE}" --agentic-matrix-test
  record "pass" "Copied app agentic matrix smoke passed" "${APP_EXECUTABLE} --agentic-matrix-test" "None."
else
  echo "==> Skipping copied app agentic matrix smoke via GARNET_STUDIO_DMG_SMOKE_AGENTIC=0"
  record "warning" "Copied app agentic matrix smoke skipped" "GARNET_STUDIO_DMG_SMOKE_AGENTIC=0" "Run without the override before claiming full packaged matrix evidence."
fi

DMG_SHA="$(shasum -a 256 "${DMG_PATH}" | awk '{print $1}')"
APP_EXECUTABLE_SHA="$(shasum -a 256 "${APP_EXECUTABLE}" | awk '{print $1}')"
GARNET_BIN_SHA="$(shasum -a 256 "${GARNET_BIN}" | awk '{print $1}')"

cat >"${DATA}" <<DATA
dmg_path=${DMG_PATH}
dmg_sha256=${DMG_SHA}
installed_app=${INSTALLED_APP}
app_executable=${APP_EXECUTABLE}
app_executable_sha256=${APP_EXECUTABLE_SHA}
bundled_garnet=${GARNET_BIN}
bundled_garnet_sha256=${GARNET_BIN_SHA}
agentic_matrix=${RUN_AGENTIC_MATRIX}
output_dir=${OUTPUT_DIR}
copy_to_desktop=${COPY_TO_DESKTOP}
DATA

{
  echo "# Garnet Studio DMG Install Smoke"
  echo
  echo "## Decision"
  echo
  echo "DMG install smoke passed for the mounted, copied app bundle."
  echo
  echo "## Artifact"
  echo
  echo "- DMG: \`${DMG_PATH}\`"
  echo "- DMG SHA-256: \`${DMG_SHA}\`"
  echo "- Copied app executable SHA-256: \`${APP_EXECUTABLE_SHA}\`"
  echo "- Copied bundled Garnet CLI SHA-256: \`${GARNET_BIN_SHA}\`"
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
  echo "This smoke proves local mounted-DMG copy/install behavior with the packaged app."
  echo "It does not claim Developer ID signing, notarization, stapling, clean-machine"
  echo "Gatekeeper acceptance, TestFlight, App Store, iOS, Android, or web-store distribution."
} >"${REPORT}"

(
  cd "${OUTPUT_DIR}"
  find . -type f ! -name MANIFEST.sha256 ! -name MANIFEST.verify.log -print0 \
    | sort -z \
    | xargs -0 shasum -a 256 >MANIFEST.sha256
  shasum -a 256 -c MANIFEST.sha256 >MANIFEST.verify.log
)

echo "artifact_dir=${OUTPUT_DIR}"

if [ "${COPY_TO_DESKTOP}" -eq 1 ]; then
  DESKTOP_DIR="${HOME}/Desktop/dogfood/$(basename "${OUTPUT_DIR}")"
  rm -rf "${DESKTOP_DIR}"
  mkdir -p "$(dirname "${DESKTOP_DIR}")"
  cp -R "${OUTPUT_DIR}" "${DESKTOP_DIR}"
  echo "desktop_copy=${DESKTOP_DIR}"
fi

echo "==> DMG install smoke passed"
