#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOCS_DIR="${ROOT}/docs"
STAMP="$(date +%Y%m%d-%H%M%S)"
OUTPUT_DIR="${ROOT}/target/web-pwa-readiness-${STAMP}"
COPY_TO_DESKTOP=0
STRICT=0

usage() {
  cat <<'USAGE'
Usage: smoke_garnet_web_pwa.sh [options]

Verify the Garnet docs web surface has installable PWA metadata, an offline
service worker shell, and fetchable static assets.

Options:
  --output-dir PATH      Evidence directory to write
  --copy-to-desktop      Copy evidence directory into ~/Desktop/dogfood
  --strict               Exit nonzero when PWA blockers are present
  -h, --help             Show this help
USAGE
}

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
REPORT="${OUTPUT_DIR}/web-pwa-readiness-report.md"
DATA="${OUTPUT_DIR}/web-pwa-readiness-data.env"
CHECKS="${OUTPUT_DIR}/checks.tsv"
COMMAND_LOG="${OUTPUT_DIR}/logs/commands.log"
OFFLINE_BEHAVIOR="${OUTPUT_DIR}/service-worker-offline-behavior.json"
BLOCKERS=0
WARNINGS=0
SERVER_PID=""

cleanup() {
  if [ -n "${SERVER_PID}" ]; then
    kill "${SERVER_PID}" >/dev/null 2>&1 || true
    wait "${SERVER_PID}" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

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

run_capture() {
  local name="$1"
  local status
  shift
  {
    printf '## %s\n' "${name}"
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

require_file() {
  local path="$1"
  local label="$2"

  if [ -f "${path}" ]; then
    record "pass" "${label} exists" "${path}" "None."
  else
    record "blocker" "${label} missing" "${path}" "Add the file before claiming web/PWA readiness."
  fi
}

echo "# Garnet Web/PWA Readiness Smoke" >"${REPORT}"
cat >"${DATA}" <<DATA
docs_dir=${DOCS_DIR}
output_dir=${OUTPUT_DIR}
DATA

record "pass" "Evidence directory created" "${OUTPUT_DIR}" "Preserve this directory with the PR or Desktop dogfood bundle."

require_file "${DOCS_DIR}/index.html" "Landing page"
require_file "${DOCS_DIR}/manifest.webmanifest" "Web app manifest"
require_file "${DOCS_DIR}/service-worker.js" "Service worker"
require_file "${DOCS_DIR}/icons/garnet-192.png" "192px PWA icon"
require_file "${DOCS_DIR}/icons/garnet-512.png" "512px PWA icon"

if grep -q '<link rel="manifest" href="manifest.webmanifest">' "${DOCS_DIR}/index.html"; then
  record "pass" "Landing page links manifest" "docs/index.html" "None."
else
  record "blocker" "Landing page does not link manifest" "docs/index.html" "Add a manifest link in the document head."
fi

if grep -q "serviceWorker.register('service-worker.js')" "${DOCS_DIR}/index.html"; then
  record "pass" "Landing page registers service worker" "docs/index.html" "None."
else
  record "blocker" "Landing page does not register service worker" "docs/index.html" "Register service-worker.js from the landing page."
fi

set +e
python3 - "$DOCS_DIR" "$OUTPUT_DIR/manifest-summary.json" <<'PY'
import json
import sys
from pathlib import Path

docs = Path(sys.argv[1])
summary_path = Path(sys.argv[2])
manifest_path = docs / "manifest.webmanifest"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

required = {
    "name": str,
    "short_name": str,
    "start_url": str,
    "scope": str,
    "display": str,
    "theme_color": str,
    "background_color": str,
    "icons": list,
}
errors = []
for key, typ in required.items():
    if key not in manifest:
        errors.append(f"missing {key}")
    elif not isinstance(manifest[key], typ):
        errors.append(f"{key} has wrong type")

if manifest.get("display") not in {"standalone", "fullscreen", "minimal-ui"}:
    errors.append("display is not installable")

icons = manifest.get("icons", [])
sizes = {icon.get("sizes"): icon for icon in icons if isinstance(icon, dict)}
for size in ("192x192", "512x512"):
    icon = sizes.get(size)
    if not icon:
        errors.append(f"missing icon size {size}")
        continue
    src = icon.get("src")
    if not src:
        errors.append(f"missing src for {size}")
        continue
    if not (docs / src).is_file():
        errors.append(f"icon file missing: {src}")

summary = {
    "name": manifest.get("name"),
    "short_name": manifest.get("short_name"),
    "start_url": manifest.get("start_url"),
    "scope": manifest.get("scope"),
    "display": manifest.get("display"),
    "icon_sizes": sorted(sizes),
    "errors": errors,
}
summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
if errors:
    raise SystemExit("; ".join(errors))
PY
MANIFEST_STATUS="$?"
set -e
if [ "${MANIFEST_STATUS}" -eq 0 ]; then
  record "pass" "Manifest has installable fields and icons" "manifest-summary.json" "None."
else
  record "blocker" "Manifest validation failed" "manifest-summary.json" "Fix manifest fields and icon references."
fi

for asset in "getting-started.html" "index.html" "install.sh" "ladder.html" "manifest.webmanifest" "minispec.html" "novel.html" "playground.html" "status.html" "stdlib.html" "synthesis.html" "blog/index.html" "blog/feed.xml" "releases.xml" "icons/garnet-192.png" "icons/garnet-512.png"; do
  if grep -q "\"${asset}\"" "${DOCS_DIR}/service-worker.js"; then
    record "pass" "Service worker caches ${asset}" "docs/service-worker.js" "None."
  else
    record "blocker" "Service worker does not cache ${asset}" "docs/service-worker.js" "Add the asset to OFFLINE_ASSETS."
  fi
done

run_capture "service-worker-offline-behavior" node "${ROOT}/scripts/smoke_garnet_web_pwa_offline.mjs" --docs-dir "${DOCS_DIR}" --output "${OFFLINE_BEHAVIOR}"
if node "${ROOT}/scripts/smoke_garnet_web_pwa_offline.mjs" --docs-dir "${DOCS_DIR}" --output "${OFFLINE_BEHAVIOR}" >/dev/null 2>&1; then
  record "pass" "Service worker serves cached navigation offline" "service-worker-offline-behavior.json" "None."
else
  record "blocker" "Service worker offline behavior check failed" "service-worker-offline-behavior.json" "Fix install/fetch behavior before claiming offline PWA readiness."
fi

PORT="$(python3 - <<'PY'
import socket
sock = socket.socket()
sock.bind(("127.0.0.1", 0))
print(sock.getsockname()[1])
sock.close()
PY
)"

(
  cd "${DOCS_DIR}"
  python3 -m http.server "${PORT}" --bind 127.0.0.1
) >"${OUTPUT_DIR}/logs/http-server.log" 2>&1 &
SERVER_PID="$!"

for _ in $(seq 1 40); do
  if curl -fsS "http://127.0.0.1:${PORT}/" >/dev/null 2>&1; then
    break
  fi
  sleep 0.25
done

for url_path in "/" "/manifest.webmanifest" "/service-worker.js" "/icons/garnet-192.png" "/icons/garnet-512.png"; do
  run_capture "curl-${url_path//\//_}" curl -fsSI "http://127.0.0.1:${PORT}${url_path}"
  if curl -fsSI "http://127.0.0.1:${PORT}${url_path}" >/dev/null 2>&1; then
    record "pass" "Local HTTP fetch succeeds for ${url_path}" "http://127.0.0.1:${PORT}${url_path}" "None."
  else
    record "blocker" "Local HTTP fetch failed for ${url_path}" "http://127.0.0.1:${PORT}${url_path}" "Fix the docs static surface before publishing."
  fi
done

{
  echo
  echo "## Summary"
  echo
  echo "- Blockers: ${BLOCKERS}"
  echo "- Warnings: ${WARNINGS}"
  echo "- Evidence: ${OUTPUT_DIR}"
  echo
  echo "## Checks"
  echo
  echo "| Status | Check | Evidence | Recommendation |"
  echo "|---|---|---|---|"
  while IFS=$'\t' read -r status label evidence recommendation; do
    printf '| %s | %s | `%s` | %s |\n' "${status}" "${label}" "${evidence}" "${recommendation}"
  done <"${CHECKS}"
} >>"${REPORT}"

(
  cd "${OUTPUT_DIR}"
  find . -type f ! -name MANIFEST.sha256 -print0 | sort -z | xargs -0 shasum -a 256 >MANIFEST.sha256
)

if [ "${COPY_TO_DESKTOP}" -eq 1 ]; then
  DESKTOP_DIR="${HOME}/Desktop/dogfood/$(basename "${OUTPUT_DIR}")"
  mkdir -p "$(dirname "${DESKTOP_DIR}")"
  rm -rf "${DESKTOP_DIR}"
  /usr/bin/ditto "${OUTPUT_DIR}" "${DESKTOP_DIR}"
  echo "Copied evidence to ${DESKTOP_DIR}"
fi

echo "Garnet Web/PWA readiness smoke: blockers=${BLOCKERS} warnings=${WARNINGS}"
echo "Evidence: ${OUTPUT_DIR}"

if [ "${STRICT}" -eq 1 ] && [ "${BLOCKERS}" -gt 0 ]; then
  exit 1
fi
