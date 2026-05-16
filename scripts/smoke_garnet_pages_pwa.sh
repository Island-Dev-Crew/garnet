#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASE_URL="https://garnet-lang.org"
STAMP="$(date +%Y%m%d-%H%M%S)"
OUTPUT_DIR="${ROOT}/target/pages-pwa-readiness-${STAMP}"
COPY_TO_DESKTOP=0
STRICT=0

usage() {
  cat <<'USAGE'
Usage: smoke_garnet_pages_pwa.sh [options]

Verify that the live Garnet Pages domain serves the PWA shell that the local
docs smoke validates. This script intentionally uses Python stdlib HTTP
requests so zsh-specific variables such as `path` cannot break command lookup.

Options:
  --base-url URL         Site origin to verify (default: https://garnet-lang.org)
  --output-dir PATH      Evidence directory to write
  --copy-to-desktop      Copy evidence directory into ~/Desktop/dogfood
  --strict               Exit nonzero when live deployment blockers are present
  -h, --help             Show this help
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --base-url)
      BASE_URL="${2%/}"
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
REPORT="${OUTPUT_DIR}/pages-pwa-readiness-report.md"
DATA_JSON="${OUTPUT_DIR}/pages-pwa-readiness-data.json"
CHECKS="${OUTPUT_DIR}/checks.tsv"
BLOCKERS=0
WARNINGS=0

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

echo "# Garnet Live Pages PWA Smoke" >"${REPORT}"
record "pass" "Evidence directory created" "${OUTPUT_DIR}" "Preserve this directory with the PR or Desktop dogfood bundle."

set +e
python3 - "$BASE_URL" "$DATA_JSON" <<'PY'
import json
import sys
from html.parser import HTMLParser
from pathlib import PurePosixPath
from urllib.parse import urljoin
from urllib.request import Request, urlopen

base_url = sys.argv[1].rstrip("/") + "/"
output_path = sys.argv[2]


class LinkParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.manifest_href = None
        self.icons = []
        self.scripts = []

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "link" and attrs.get("rel") == "manifest":
            self.manifest_href = attrs.get("href")
        if tag == "link" and attrs.get("rel") in {"icon", "apple-touch-icon"}:
            self.icons.append(attrs)
        if tag == "script" and attrs.get("src"):
            self.scripts.append(attrs["src"])


def fetch(url):
    request = Request(url, headers={"User-Agent": "garnet-pages-pwa-smoke/1"})
    with urlopen(request, timeout=20) as response:
        body = response.read()
        return {
            "url": url,
            "status": response.status,
            "content_type": response.headers.get("content-type", ""),
            "content_length": response.headers.get("content-length"),
            "body": body,
            "text": body.decode("utf-8", errors="replace"),
        }


def expect(condition, message, errors):
    if not condition:
        errors.append(message)


errors = []
responses = {}
for url_path in ["/", "/status.html", "/manifest.webmanifest", "/service-worker.js", "/icons/garnet-192.png", "/icons/garnet-512.png"]:
    url = urljoin(base_url, url_path.lstrip("/"))
    try:
        responses[url_path] = fetch(url)
    except Exception as exc:  # noqa: BLE001 - evidence script must report live failures.
        errors.append(f"fetch failed for {url_path}: {exc}")

index = responses.get("/")
manifest_response = responses.get("/manifest.webmanifest")
worker_response = responses.get("/service-worker.js")
icon_192 = responses.get("/icons/garnet-192.png")
icon_512 = responses.get("/icons/garnet-512.png")

if index:
    parser = LinkParser()
    parser.feed(index["text"])
    expect(index["status"] == 200, "index did not return HTTP 200", errors)
    expect("text/html" in index["content_type"], "index content-type is not HTML", errors)
    expect(parser.manifest_href == "manifest.webmanifest", "index does not link manifest.webmanifest", errors)
    expect("serviceWorker.register('service-worker.js')" in index["text"], "index does not register service-worker.js", errors)
    studio_terms = {
        "Garnet Studio workbench": "index does not expose the Garnet Studio workbench section",
        "Codex Run": "index does not mention the Codex Run Studio workflow",
        "dist/Garnet Studio.app": "index does not mention the staged Studio app bundle",
        "Assist Plan": "index does not mention the Studio Assist Plan workflow",
        "Continuation Pulse": "index does not mention the Studio Continuation Pulse workflow",
    }
    for term, message in studio_terms.items():
        expect(term in index["text"], f"{message}: missing {term}", errors)
else:
    parser = LinkParser()
    studio_terms = {}

manifest = {}
if manifest_response:
    expect(manifest_response["status"] == 200, "manifest did not return HTTP 200", errors)
    expect("manifest" in manifest_response["content_type"] or "json" in manifest_response["content_type"], "manifest content-type is not manifest/json", errors)
    try:
        manifest = json.loads(manifest_response["text"])
    except json.JSONDecodeError as exc:
        errors.append(f"manifest JSON parse failed: {exc}")

if manifest:
    expect(manifest.get("display") in {"standalone", "fullscreen", "minimal-ui"}, "manifest display is not installable", errors)
    expect(manifest.get("start_url") in {"./", "."}, "manifest start_url is not scoped to docs root", errors)
    expect(manifest.get("scope") in {"./", "."}, "manifest scope is not docs root", errors)
    sizes = {icon.get("sizes"): icon for icon in manifest.get("icons", []) if isinstance(icon, dict)}
    for size in ["192x192", "512x512"]:
        expect(size in sizes, f"manifest missing {size} icon", errors)
        if size in sizes:
            src = sizes[size].get("src", "")
            expected_name = f"garnet-{size.split('x')[0]}.png"
            expect(PurePosixPath(src).name == expected_name, f"manifest {size} icon does not point at {expected_name}", errors)

if worker_response:
    expect(worker_response["status"] == 200, "service worker did not return HTTP 200", errors)
    expect("javascript" in worker_response["content_type"], "service worker content-type is not JavaScript", errors)
    for asset in ["index.html", "install.sh", "ladder.html", "manifest.webmanifest", "minispec.html", "novel.html", "status.html", "synthesis.html", "icons/garnet-192.png", "icons/garnet-512.png"]:
        expect(f'"{asset}"' in worker_response["text"], f"service worker does not cache {asset}", errors)

for label, response in [("192 icon", icon_192), ("512 icon", icon_512)]:
    if response:
        expect(response["status"] == 200, f"{label} did not return HTTP 200", errors)
        expect("image/png" in response["content_type"], f"{label} content-type is not image/png", errors)
        expect(len(response["body"]) > 1024, f"{label} response body is unexpectedly small", errors)

summary = {
    "base_url": base_url.rstrip("/"),
    "manifest_href": parser.manifest_href,
    "responses": {
        path: {
            "url": response["url"],
            "status": response["status"],
            "content_type": response["content_type"],
            "content_length": response["content_length"],
            "bytes": len(response["body"]),
        }
        for path, response in responses.items()
    },
    "manifest": {
        "name": manifest.get("name"),
        "short_name": manifest.get("short_name"),
        "display": manifest.get("display"),
        "start_url": manifest.get("start_url"),
        "scope": manifest.get("scope"),
        "icons": manifest.get("icons", []),
    },
    "studio_adoption_terms": {
        term: term in index["text"] if index else False for term in studio_terms
    },
    "errors": errors,
}
with open(output_path, "w", encoding="utf-8") as handle:
    json.dump(summary, handle, indent=2)
    handle.write("\n")

if errors:
    for error in errors:
        print(error, file=sys.stderr)
    raise SystemExit(1)
PY
PY_STATUS="$?"
set -e

if [ "${PY_STATUS}" -eq 0 ]; then
  record "pass" "Live Pages PWA surface validates" "${DATA_JSON}" "None."
else
  record "blocker" "Live Pages PWA surface failed validation" "${DATA_JSON}" "Inspect response metadata and wait for Pages deployment only if the failure is deployment-lag related."
fi

{
  echo
  echo "## Summary"
  echo
  echo "- Base URL: ${BASE_URL}"
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

echo "Garnet live Pages PWA smoke: blockers=${BLOCKERS} warnings=${WARNINGS}"
echo "Evidence: ${OUTPUT_DIR}"

if [ "${STRICT}" -eq 1 ] && [ "${BLOCKERS}" -gt 0 ]; then
  exit 1
fi
